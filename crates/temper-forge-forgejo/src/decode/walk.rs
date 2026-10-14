//! Static object tables, required masks and a bounded container stack.
use super::Malformed;
use crate::limits::Limits;
use skein_json::{Token, tokenizer};
use skein_lib::Stack;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    User,
    Settings,
    Repository,
    Search,
    Users,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Read {
    Number,
    Text,
    Boolean,
    Object(Kind),
    Array(Kind),
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Field {
    pub key: &'static [u8],
    pub read: Read,
}
fn fields(kind: Kind) -> &'static [Field] {
    match kind {
        Kind::User => &[Field { key: b"id", read: Read::Number }, Field { key: b"login", read: Read::Text }],
        Kind::Settings => &[
            Field { key: b"max_response_items", read: Read::Number },
            Field { key: b"default_paging_num", read: Read::Number },
        ],
        Kind::Repository => &[
            Field { key: b"id", read: Read::Number },
            Field { key: b"owner", read: Read::Object(Kind::User) },
            Field { key: b"name", read: Read::Text },
            Field { key: b"object_format_name", read: Read::Text },
            Field { key: b"default_branch", read: Read::Text },
            Field { key: b"default_merge_style", read: Read::Text },
            Field { key: b"allow_merge_commits", read: Read::Boolean },
            Field { key: b"allow_squash_merge", read: Read::Boolean },
            Field { key: b"allow_rebase", read: Read::Boolean },
            Field { key: b"html_url", read: Read::Text },
        ],
        Kind::Search => {
            &[Field { key: b"data", read: Read::Array(Kind::Users) }, Field { key: b"ok", read: Read::Boolean }]
        }
        Kind::Users => &[],
    }
}
#[derive(Debug)]
struct Frame {
    kind: Kind,
    touched: u64,
    seen: u64,
}
#[derive(Debug)]
enum Want {
    Start(Kind),
    Key,
    Value { field: Field, bit: u64 },
    Skip,
    Element,
    Done,
    Terminal,
}
#[derive(Debug)]
pub(crate) struct Walk {
    open: Stack<Frame>,
    want: Want,
}
#[derive(Debug)]
pub(crate) enum Move {
    Demand(tokenizer::Request),
    Scalar { kind: Kind, key: &'static [u8] },
    End(Kind),
    Done,
}
impl Walk {
    pub(crate) fn new(kind: Kind, limits: &Limits) -> Self {
        Self { open: Stack::with_capacity(limits.json.depth), want: Want::Start(kind) }
    }
    pub(crate) fn parent(&self) -> Option<Kind> {
        let frame = self.open.top()?;
        Some(frame.kind)
    }
    pub(crate) fn step(&mut self, event: &tokenizer::Event, limits: &Limits) -> Result<Move, Malformed> {
        match event {
            tokenizer::Event::Failed(error) => return Err(Malformed::Json(*error)),
            tokenizer::Event::Closed => return Err(Malformed::Shape { field: b"" }),
            tokenizer::Event::Token(_)
            | tokenizer::Event::Long(_)
            | tokenizer::Event::Skipped(_)
            | tokenizer::Event::Done => {}
        }
        let want = core::mem::replace(&mut self.want, Want::Terminal);
        match want {
            Want::Start(kind) => self.start(kind, event),
            Want::Key => self.key(event, limits),
            Want::Value { field, bit } => self.value(field, bit, event),
            Want::Skip => {
                match event {
                    tokenizer::Event::Skipped(_) => {}
                    tokenizer::Event::Token(_)
                    | tokenizer::Event::Long(_)
                    | tokenizer::Event::Done
                    | tokenizer::Event::Failed(_)
                    | tokenizer::Event::Closed => return Err(Malformed::Shape { field: b"" }),
                }
                self.want = Want::Key;
                Ok(Move::Demand(tokenizer::Request::Next))
            }
            Want::Element => {
                if event == &tokenizer::Event::Token(Token::ArrayEnd) {
                    self.end()
                } else {
                    self.start(Kind::User, event)
                }
            }
            Want::Done => {
                if event == &tokenizer::Event::Done {
                    Ok(Move::Done)
                } else {
                    Err(Malformed::Shape { field: b"" })
                }
            }
            Want::Terminal => Err(Malformed::Shape { field: b"" }),
        }
    }
    fn start(&mut self, kind: Kind, event: &tokenizer::Event) -> Result<Move, Malformed> {
        let expected = match kind {
            Kind::Users => Token::ArrayStart,
            Kind::User | Kind::Settings | Kind::Repository | Kind::Search => Token::ObjectStart,
        };
        if event != &tokenizer::Event::Token(expected) {
            return Err(Malformed::Shape { field: b"" });
        }
        if self.open.push(Frame { kind, touched: 0, seen: 0 }).is_err() {
            return Err(Malformed::Json(tokenizer::Error::TooDeep));
        }
        self.want = match kind {
            Kind::Users => Want::Element,
            Kind::User | Kind::Settings | Kind::Repository | Kind::Search => Want::Key,
        };
        Ok(Move::Demand(tokenizer::Request::Next))
    }
    fn key(&mut self, event: &tokenizer::Event, limits: &Limits) -> Result<Move, Malformed> {
        match event {
            tokenizer::Event::Token(Token::ObjectEnd) => self.end(),
            tokenizer::Event::Token(Token::Key(key)) => {
                let frame = self.open.top_mut().expect("key inside an object");
                for (index, field) in fields(frame.kind).iter().enumerate() {
                    if field.key == &**key {
                        let bit =
                            1u64.checked_shl(u32::try_from(index).expect("small field table")).expect("field mask");
                        if frame.touched & bit != 0 {
                            return Err(Malformed::Shape { field: field.key });
                        }
                        frame.touched |= bit;
                        self.want = Want::Value { field: *field, bit };
                        let demand = match field.read {
                            Read::Text => tokenizer::Request::Text(limits.string_bytes),
                            Read::Number | Read::Boolean | Read::Object(_) | Read::Array(_) => tokenizer::Request::Next,
                        };
                        return Ok(Move::Demand(demand));
                    }
                }
                self.want = Want::Skip;
                Ok(Move::Demand(tokenizer::Request::Skip))
            }
            tokenizer::Event::Token(
                Token::ObjectStart
                | Token::ArrayStart
                | Token::ArrayEnd
                | Token::String(_)
                | Token::Number(_)
                | Token::True
                | Token::False
                | Token::Null,
            )
            | tokenizer::Event::Long(_)
            | tokenizer::Event::Skipped(_)
            | tokenizer::Event::Done
            | tokenizer::Event::Failed(_)
            | tokenizer::Event::Closed => Err(Malformed::Shape { field: b"" }),
        }
    }
    fn value(&mut self, field: Field, bit: u64, event: &tokenizer::Event) -> Result<Move, Malformed> {
        if event == &tokenizer::Event::Token(Token::Null) {
            self.want = Want::Key;
            return Ok(Move::Demand(tokenizer::Request::Next));
        }
        let frame = self.open.top_mut().expect("value inside an object");
        frame.seen |= bit;
        let kind = frame.kind;
        match field.read {
            Read::Object(child) | Read::Array(child) => match self.start(child, event) {
                Ok(result) => Ok(result),
                Err(Malformed::Shape { .. }) => Err(Malformed::Shape { field: field.key }),
                Err(error) => Err(error),
            },
            Read::Number | Read::Text | Read::Boolean => {
                let actual = match event {
                    tokenizer::Event::Token(Token::Number(_)) => Read::Number,
                    tokenizer::Event::Token(Token::String(_)) => Read::Text,
                    tokenizer::Event::Token(Token::True | Token::False) => Read::Boolean,
                    tokenizer::Event::Long(_) => return Err(Malformed::TooLong { field: field.key }),
                    tokenizer::Event::Token(
                        Token::ObjectStart
                        | Token::ObjectEnd
                        | Token::ArrayStart
                        | Token::ArrayEnd
                        | Token::Key(_)
                        | Token::Null,
                    )
                    | tokenizer::Event::Skipped(_)
                    | tokenizer::Event::Done
                    | tokenizer::Event::Failed(_)
                    | tokenizer::Event::Closed => return Err(Malformed::Shape { field: field.key }),
                };
                if actual != field.read {
                    return Err(Malformed::Shape { field: field.key });
                }
                self.want = Want::Key;
                Ok(Move::Scalar { kind, key: field.key })
            }
        }
    }
    fn end(&mut self) -> Result<Move, Malformed> {
        let frame = self.open.pop().expect("container end");
        for (index, field) in fields(frame.kind).iter().enumerate() {
            let bit = 1u64.checked_shl(u32::try_from(index).expect("field index")).expect("field mask");
            if frame.seen & bit == 0 {
                return Err(Malformed::Missing { field: field.key });
            }
        }
        self.want = match self.parent() {
            None => Want::Done,
            Some(Kind::Users) => Want::Element,
            Some(Kind::User | Kind::Settings | Kind::Repository | Kind::Search) => Want::Key,
        };
        Ok(Move::End(frame.kind))
    }
}
