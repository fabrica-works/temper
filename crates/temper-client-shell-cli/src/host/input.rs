//! Load and check external input before DIRECTORY opens (client/tools.md, 8.2).
use super::{Class, Down, OneShot, Parsed, Read, Time, Up, arguments, io, limits};
use temper_client_tools::{
    help,
    jig::{
        self,
        call::{Source, Value, Values},
        table::Kind,
    },
    table::TOOLS,
};
impl OneShot {
    pub(super) fn prepare_start(&mut self) {
        match self.call.as_ref().expect("unstarted call") {
            Parsed::File { file: Source::Path(path), .. } => self.directory.load_call(path),
            Parsed::File { file: Source::Stdin, .. } => {
                self.input = Some(Vec::new());
                self.read_input();
            }
            Parsed::Call { call, json } if self.class == Class::Help => {
                let selected = call.values.iter().find_map(|slot| match slot {
                    Some(Values::One(Value::Tool(index))) => Some(*index),
                    _ => None,
                });
                let bytes = if let Some(index) = selected.filter(|_| *json) {
                    jig::json::write_tool(TOOLS[usize::from(index)]).expect("help record")
                } else {
                    let mut out =
                        skein_lib::Writer::new(usize::try_from(self.root_env.limits.tools.text).expect("help bound"));
                    help::text(selected, &mut out).expect("help fits text window");
                    let mut exact = skein_lib::Writer::new(out.written());
                    help::text(selected, &mut exact).expect("measured help");
                    exact.finish()
                };
                self.output.push(false, bytes);
                self.preparing = false;
                self.finish(0, false);
            }
            Parsed::Call { call, .. } if arguments::key_file(call).is_some() => {
                self.directory.load_key(arguments::key_file(call).expect("key file"));
            }
            Parsed::Call { call, .. } => {
                if self.class != Class::SignIn && call.values.contains(&Some(Values::One(Value::Stdin))) {
                    self.input = Some(Vec::new());
                    self.read_input();
                } else {
                    self.preparing = false;
                }
            }
        }
    }
    pub(super) fn prepare_progress(&mut self) {
        if !self.preparing || self.status.is_some() {
            return;
        }
        if self.file_input() {
            self.directory.progress(self.env.now);
        }
        let Some(bytes) = self.directory.link.take() else { return };
        match self.call.take().expect("input's call") {
            Parsed::File { file, json } => match jig::json::call(TOOLS, &bytes, file, &self.root_env.limits.tools) {
                Ok(call) => {
                    let specs = TOOLS[usize::from(call.tool)].forms[usize::from(call.form)];
                    if specs
                        .iter()
                        .zip(call.values.iter())
                        .any(|(spec, value)| spec.kind == Kind::File && value.is_some())
                    {
                        self.fail(b"usage: temper ctl DIRECTORY TOOL [ARGUMENT...]");
                        return;
                    }
                    self.class = arguments::class(&call);
                    self.directory.start(self.class);
                    self.call = Some(Parsed::Call { call, json });
                    self.prepare_start();
                }
                Err(_) => self.fail(b"usage: temper ctl DIRECTORY TOOL [ARGUMENT...]"),
            },
            Parsed::Call { call, json } if arguments::key_file(&call).is_some() => {
                let stripped = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
                if stripped.is_empty() || stripped.len() > temper_client_tools::limits::KEY_BYTES as usize {
                    self.fail(b"key file must contain 1 to 4096 bytes after one final newline");
                    return;
                }
                self.value = Some(skein_lib::Secret::new(stripped.into()));
                self.directory.start(self.class);
                self.call = Some(Parsed::Call { call, json });
                self.preparing = false;
            }
            Parsed::Call { mut call, json } => {
                for slot in &mut call.values {
                    if *slot == Some(Values::One(Value::Stdin)) {
                        *slot = Some(Values::One(Value::Text(bytes.clone())));
                    }
                }
                self.call = Some(Parsed::Call { call, json });
                self.preparing = false;
            }
        }
    }
    pub(super) fn preparation_pending(&self, now: Time) -> bool {
        self.preparing && self.file_input() && self.directory.pending(now)
    }
    fn file_input(&self) -> bool {
        match &self.call {
            Some(Parsed::File { file: Source::Path(_), .. }) => true,
            Some(Parsed::Call { call, .. }) => arguments::key_file(call).is_some(),
            _ => false,
        }
    }
    fn input_limit(&self) -> u32 {
        if self.class == Class::SignIn && !matches!(self.call, Some(Parsed::File { .. })) {
            limits::LINK
        } else {
            u32::try_from(self.root_env.limits.tools.ask_bytes).expect("input bound")
        }
    }
    pub(super) fn read_input(&mut self) {
        self.request(io::Request::Stream {
            stream: self.stdin.expect("stdin"),
            down: Down::Demand { read: Read::Arrived { max: self.input_limit() + 1 }, room: 0 },
        });
    }
    pub(super) fn input_event(&mut self, up: Up) {
        match up {
            Up::Bytes(bytes) => {
                let limit = self.input_limit() as usize;
                let input = self.input.as_mut().expect("reading input");
                if input.len() + bytes.len() > limit {
                    self.fail(b"input is too large");
                    return;
                }
                input.extend_from_slice(&bytes);
                self.read_input();
            }
            Up::End => self.directory.link = self.input.take().map(Vec::into_boxed_slice),
            Up::Failed(_) => self.fail(b"input could not be read"),
            Up::Room => unreachable!("stdin never writes"),
        }
    }
}
