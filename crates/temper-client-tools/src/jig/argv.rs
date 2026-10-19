//! One table-driven argv parser and checker.
use super::{
    Limits,
    call::{Call, Parsed, Project, Source, Value, Values},
    table::{Argument, Kind, Occurs, Place, Tool},
};
use alloc::boxed::Box;
use jig_client::contract::{InboxGroup, QueryOrder, TurnName};
use skein_lib::{Decimal, List, Overflow, Writer};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    Tool,
    Unknown { at: u16 },
    Extra { at: u16 },
    Missing { argument: u8 },
    Twice { argument: u8 },
    Kind { argument: u8 },
    Rows { argument: u8 },
    Long { argument: u8 },
    Many { argument: u8 },
    Stdin,
    Key,
    Condition,
    Json,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unwritten {
    Overflow(Overflow),
    NoForm,
}
/// Parse argv after the host's directory.
pub fn parse(table: &[&Tool], words: &[&[u8]], limits: &Limits) -> Result<Parsed, Usage> {
    let count = u16::try_from(words.len()).or(Err(Usage::Tool))?;
    let mut args = List::with_capacity(u32::from(count));
    let mut json = false;
    let mut positional = false;
    for word in words {
        if !positional && *word == b"--json" {
            if json {
                return Err(Usage::Json);
            }
            json = true;
        } else {
            args.push(*word).or(Err(Usage::Tool))?;
            positional |= *word == b"--";
        }
    }
    let (name, words) = args.as_slice().split_first().ok_or(Usage::Tool)?;
    if *name == b"--call" {
        let [file] = words else {
            return Err(Usage::Json);
        };
        let file = if *file == b"-" { Source::Stdin } else { Source::Path(Box::from(*file)) };
        return Ok(Parsed::File { file, json });
    }
    let tool = tool_index(table, name).ok_or(Usage::Tool)?;
    let row = table.get(usize::from(tool)).ok_or(Usage::Tool)?;
    let mut error = Usage::Tool;
    for (index, arguments) in row.forms.iter().enumerate() {
        let form = u8::try_from(index).or(Err(Usage::Tool))?;
        match form_values(table, arguments, words, limits) {
            Ok(values) => {
                let call = Call { tool, form, values, key: None };
                check(table, &call, limits)?;
                return Ok(Parsed::Call { call, json });
            }
            Err(failed) => error = failed,
        }
    }
    Err(error)
}
fn tool_index(table: &[&Tool], word: &[u8]) -> Option<u8> {
    for (index, tool) in table.iter().enumerate() {
        if tool.name == word {
            return u8::try_from(index).ok();
        }
    }
    None
}
fn form_values(
    table: &[&Tool],
    arguments: &[Argument],
    words: &[&[u8]],
    limits: &Limits,
) -> Result<Box<[Option<Values>]>, Usage> {
    let count = u8::try_from(arguments.len()).or(Err(Usage::Tool))?;
    let mut values = List::with_capacity(u32::from(count));
    for _ in arguments {
        values.push(None).or(Err(Usage::Tool))?;
    }
    let mut skip = None;
    let mut positional = false;
    for (index, word) in words.iter().enumerate() {
        if skip == Some(index) {
            continue;
        }
        if !positional && *word == b"--" {
            positional = true;
            continue;
        }
        let at = u16::try_from(index).or(Err(Usage::Tool))?;
        let flag = !positional && word.starts_with(b"--");
        let mut found = None;
        for (slot, argument) in arguments.iter().enumerate() {
            let id = u8::try_from(slot).or(Err(Usage::Tool))?;
            if flag {
                if argument.place != Place::Positional && word.get(2..) == Some(argument.name) {
                    found = Some(id);
                    break;
                }
            } else if argument.place == Place::Positional && values.get(u32::from(id)) == Some(&None) {
                found = Some(id);
                break;
            }
        }
        let argument = found.ok_or(if flag { Usage::Unknown { at } } else { Usage::Extra { at } })?;
        let spec = arguments.get(usize::from(argument)).ok_or(Usage::Tool)?;
        let slot = values.get_mut(u32::from(argument)).ok_or(Usage::Tool)?;
        if slot.is_some() {
            return Err(Usage::Twice { argument });
        }
        let value = if spec.place == Place::Switch {
            Value::Switch
        } else {
            let text = if flag {
                let next = index.checked_add(1).ok_or(Usage::Tool)?;
                skip = Some(next);
                *words.get(next).ok_or(Usage::Missing { argument })?
            } else {
                *word
            };
            decode(table, spec.kind, text).ok_or(Usage::Kind { argument })?
        };
        valid(table, spec, &value, argument, limits)?;
        *slot = Some(Values::One(value));
    }
    for (index, argument) in arguments.iter().enumerate() {
        if argument.kind == Kind::Order {
            let index = u32::try_from(index).or(Err(Usage::Tool))?;
            let slot = values.get_mut(index).ok_or(Usage::Tool)?;
            if slot.is_none() {
                *slot = Some(Values::One(Value::Order(QueryOrder::Number)));
            }
        }
    }
    Ok(values.into_boxed())
}
/// The same validation is used for typed JSON calls.
pub fn check(table: &[&Tool], call: &Call, limits: &Limits) -> Result<(), Usage> {
    let tool = table.get(usize::from(call.tool)).ok_or(Usage::Tool)?;
    let args = tool.forms.get(usize::from(call.form)).ok_or(Usage::Tool)?;
    if args.len() != call.values.len() {
        return Err(Usage::Tool);
    }
    if call.key.is_some() {
        return Err(Usage::Key);
    }
    let mut stdin = false;
    for (index, (argument, values)) in args.iter().zip(&call.values).enumerate() {
        let argument_id = u8::try_from(index).or(Err(Usage::Tool))?;
        match values {
            None if argument.occurs == Occurs::Required => return Err(Usage::Missing { argument: argument_id }),
            None => {}
            Some(Values::One(value)) => {
                valid(table, argument, value, argument_id, limits)?;
                if *value == Value::Stdin {
                    if stdin {
                        return Err(Usage::Stdin);
                    }
                    stdin = true;
                }
            }
            Some(Values::Many(_)) => return Err(Usage::Many { argument: argument_id }),
        }
    }
    Ok(())
}
fn valid(table: &[&Tool], spec: &Argument, value: &Value, argument: u8, limits: &Limits) -> Result<(), Usage> {
    if spec.place == Place::Switch {
        return if *value == Value::Switch { Ok(()) } else { Err(Usage::Kind { argument }) };
    }
    let correct = match spec.kind {
        Kind::Number => {
            let Value::Number(_) = value else {
                return Err(Usage::Kind { argument });
            };
            true
        }
        Kind::Count => {
            let Value::Number(n) = value else {
                return Err(Usage::Kind { argument });
            };
            if *n == 0 || *n > u64::from(limits.rows) {
                return Err(Usage::Rows { argument });
            }
            true
        }
        Kind::Word { set } => {
            let Value::Word(index) = value else {
                return Err(Usage::Kind { argument });
            };
            set.get(usize::from(*index)).is_some()
        }
        Kind::Text => {
            if *value == Value::Stdin {
                return Ok(());
            }
            let Value::Text(text) = value else {
                return Err(Usage::Kind { argument });
            };
            if u64::try_from(text.len()).unwrap_or(u64::MAX) > limits.ask_bytes {
                return Err(Usage::Long { argument });
            }
            true
        }
        Kind::File => {
            if *value == Value::Stdin {
                return Ok(());
            }
            let Value::File(_) = value else {
                return Err(Usage::Kind { argument });
            };
            true
        }
        Kind::Project => {
            let Value::Project(project) = value else {
                return Err(Usage::Kind { argument });
            };
            match project {
                Project::Number(_) => true,
                Project::Name(name) => {
                    let mut digits = true;
                    for byte in name {
                        digits &= byte.is_ascii_digit();
                    }
                    !name.is_empty() && !digits
                }
            }
        }
        Kind::Turn => {
            let Value::Turn(_) = value else {
                return Err(Usage::Kind { argument });
            };
            true
        }
        Kind::Group => {
            let Value::Group(_) = value else {
                return Err(Usage::Kind { argument });
            };
            true
        }
        Kind::Order => {
            let Value::Order(_) = value else {
                return Err(Usage::Kind { argument });
            };
            true
        }
        Kind::Cursor => {
            let Value::Cursor(bytes) = value else {
                return Err(Usage::Kind { argument });
            };
            !bytes.is_empty()
        }
        Kind::Tool => {
            let Value::Tool(id) = value else {
                return Err(Usage::Kind { argument });
            };
            table.get(usize::from(*id)).is_some()
        }
    };
    if correct { Ok(()) } else { Err(Usage::Kind { argument }) }
}
fn decode(table: &[&Tool], kind: Kind, word: &[u8]) -> Option<Value> {
    Some(match kind {
        Kind::Number | Kind::Count => Value::Number(number(word)?),
        Kind::Word { set } => {
            let mut found = None;
            for (index, name) in set.iter().enumerate() {
                if *name == word {
                    found = Some(u8::try_from(index).ok()?);
                    break;
                }
            }
            Value::Word(found?)
        }
        Kind::Text | Kind::File if word == b"-" => Value::Stdin,
        Kind::Text => Value::Text(Box::from(word)),
        Kind::File => Value::File(Box::from(word)),
        Kind::Project => {
            if word.is_empty() {
                return None;
            }
            let mut digits = true;
            for byte in word {
                digits &= byte.is_ascii_digit();
            }
            Value::Project(if digits {
                Project::Number(u32::try_from(number(word)?).ok()?)
            } else {
                Project::Name(Box::from(word))
            })
        }
        Kind::Turn => {
            let mut dot = None;
            for (index, byte) in word.iter().enumerate() {
                if *byte == b'.' {
                    dot = Some(index);
                    break;
                }
            }
            let split = dot?;
            let end = split.checked_add(1)?;
            Value::Turn(TurnName {
                attempt: number(word.get(..split)?)?,
                number: u32::try_from(number(word.get(end..)?)?).ok()?,
            })
        }
        Kind::Cursor => Value::Cursor(hex(word)?),
        Kind::Group => Value::Group(match word {
            b"questions" => InboxGroup::Questions,
            b"proposals" => InboxGroup::Proposals,
            b"escalations" => InboxGroup::Escalations,
            b"person-tasks" => InboxGroup::PersonTasks,
            b"results" => InboxGroup::Results,
            b"replies" => InboxGroup::Replies,
            _ => return None,
        }),
        Kind::Order => Value::Order(match word {
            b"activity" => QueryOrder::Activity,
            b"priority" => QueryOrder::Priority,
            _ => return None,
        }),
        Kind::Tool => Value::Tool(tool_index(table, word)?),
    })
}
fn number(word: &[u8]) -> Option<u64> {
    if word.is_empty() {
        return None;
    }
    let mut value = 0_u64;
    for byte in word {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
fn hex(word: &[u8]) -> Option<Box<[u8]>> {
    if word.is_empty() || word.len().checked_rem(2)? != 0 {
        return None;
    }
    let mut out = Writer::new(word.len().checked_div(2)?);
    for pair in word.chunks_exact(2) {
        let mut value = 0_u8;
        for byte in pair {
            let nibble = match byte {
                b'0'..=b'9' => byte.checked_sub(b'0')?,
                b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?,
                _ => return None,
            };
            value = value.checked_mul(16)?.checked_add(nibble)?;
        }
        out.put(&[value]).ok()?;
    }
    Some(out.finish())
}
/// Write positionals followed by flags, in the row's order.
pub fn back(table: &[&Tool], call: &Call, out: &mut List<Box<[u8]>>) -> Result<(), Unwritten> {
    let row = table.get(usize::from(call.tool)).ok_or(Unwritten::NoForm)?;
    let args = row.forms.get(usize::from(call.form)).ok_or(Unwritten::NoForm)?;
    if args.len() != call.values.len() || call.key.is_some() {
        return Err(Unwritten::NoForm);
    }
    push(out, row.name)?;
    for positional in [true, false] {
        for (argument, slot) in args.iter().zip(&call.values) {
            let place = argument.place;
            if (place == Place::Positional) != positional {
                continue;
            }
            let Some(Values::One(value)) = slot else {
                if slot.is_some() {
                    return Err(Unwritten::NoForm);
                }
                continue;
            };
            if *value == Value::Order(QueryOrder::Number) {
                continue;
            }
            if *value == Value::Order(QueryOrder::Changed) || *value == Value::Order(QueryOrder::Ended) {
                return Err(Unwritten::NoForm);
            }
            if place != Place::Positional {
                let mut flag = Writer::new(argument.name.len().saturating_add(2));
                written(flag.put(b"--"))?;
                written(flag.put(argument.name))?;
                push(out, &flag.finish())?;
            }
            if place != Place::Switch {
                let word = encode(table, argument.kind, value)?;
                if place == Place::Positional && word.starts_with(b"--") {
                    push(out, b"--")?;
                }
                push(out, &word)?;
            }
        }
    }
    Ok(())
}
fn push(out: &mut List<Box<[u8]>>, bytes: &[u8]) -> Result<(), Unwritten> {
    match out.push(Box::from(bytes)) {
        Ok(()) => Ok(()),
        Err(_) => Err(Unwritten::Overflow(Overflow)),
    }
}
fn encode(table: &[&Tool], kind: Kind, value: &Value) -> Result<Box<[u8]>, Unwritten> {
    Ok(match value {
        Value::Number(n) => Box::from(Decimal::of(*n).as_bytes()),
        Value::Word(index) => {
            let Kind::Word { set } = kind else {
                return Err(Unwritten::NoForm);
            };
            Box::from(*set.get(usize::from(*index)).ok_or(Unwritten::NoForm)?)
        }
        Value::Text(text) | Value::File(text) => text.clone(),
        Value::Stdin => Box::from(b"-".as_slice()),
        Value::Project(Project::Number(n)) => Box::from(Decimal::of(u64::from(*n)).as_bytes()),
        Value::Project(Project::Name(name)) => name.clone(),
        Value::Turn(turn) => {
            let attempt = Decimal::of(turn.attempt);
            let number = Decimal::of(u64::from(turn.number));
            let prefix = attempt.as_bytes().len().checked_add(1).ok_or(Unwritten::Overflow(Overflow))?;
            let size = prefix.checked_add(number.as_bytes().len()).ok_or(Unwritten::Overflow(Overflow))?;
            let mut out = Writer::new(size);
            written(out.put(attempt.as_bytes()))?;
            written(out.put(b"."))?;
            written(out.put(number.as_bytes()))?;
            out.finish()
        }
        Value::Cursor(bytes) => {
            let size = bytes.len().checked_mul(2).ok_or(Unwritten::Overflow(Overflow))?;
            let mut hex = Writer::new(size);
            for byte in bytes {
                for nibble in [byte.checked_div(16).unwrap_or(0), byte.checked_rem(16).unwrap_or(0)] {
                    let digit = b"0123456789abcdef".get(usize::from(nibble)).ok_or(Unwritten::NoForm)?;
                    written(hex.put(&[*digit]))?;
                }
            }
            hex.finish()
        }
        Value::Group(group) => Box::from(match group {
            InboxGroup::Questions => b"questions".as_slice(),
            InboxGroup::Proposals => b"proposals",
            InboxGroup::Escalations => b"escalations",
            InboxGroup::PersonTasks => b"person-tasks",
            InboxGroup::Results => b"results",
            InboxGroup::Replies => b"replies",
        }),
        Value::Order(order) => Box::from(match order {
            QueryOrder::Number | QueryOrder::Changed | QueryOrder::Ended => return Err(Unwritten::NoForm),
            QueryOrder::Activity => b"activity".as_slice(),
            QueryOrder::Priority => b"priority",
        }),
        Value::Tool(id) => Box::from(table.get(usize::from(*id)).ok_or(Unwritten::NoForm)?.name),
        Value::Switch => return Err(Unwritten::NoForm),
    })
}

fn written(result: Result<(), Overflow>) -> Result<(), Unwritten> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(Unwritten::Overflow(error)),
    }
}
