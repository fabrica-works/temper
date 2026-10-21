//! The demo's plan, expressed only as the person's tool calls (protocol/setup.md, 8.4).
use crate::demo::Demo;
use alloc::boxed::Box;
use jig_protocol_config as config;
use skein_lib::{Decimal, List};

/// The seed's acts, in order (protocol/setup.md, 8.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    SignIn,
    Key,
    Usable,
    Adopt,
    SignOut,
    Spent,
}
/// The next `temper ctl CLIENT WORDS` call: the link on its stdin when `link`.
#[derive(Debug)]
pub struct Call<'a> {
    pub words: &'a [Box<[u8]>],
    pub link: bool,
}
/// A call's status (None for a signal) and its two bounded streams.
#[derive(Debug)]
pub struct Ended<'a> {
    pub status: Option<u8>,
    pub stdout: &'a [u8],
    pub stderr: &'a [u8],
}
#[derive(Debug)]
pub enum Next<'a> {
    Call(Call<'a>),
    Done,
}
/// Words for the seed's stderr, including its prefix and newline.
#[derive(Debug)]
pub struct Failed {
    pub act: Act,
    pub why: Box<[u8]>,
}
/// The ctl harness's bound, including its tools guide (tests/support/ctl.rs).
pub const CALL_OUTPUT: u32 = 65_536;
/// At most this many bytes of a child's first stderr line are quoted.
pub const QUOTE_BYTES: usize = 256;
/// CLI sign-in refusal, for both text and JSON calls (client shell `host::fail`).
pub const SPENT_STATUS: u8 = 1;
pub const SPENT_STDERR: &[u8] = b"temper ctl: the sign-in link was refused\n";
pub const ADOPT_LINE: &[u8] = b"temper fake seed: act 4: the demo's repository is not adopted and the forges are not waited on; both wait on temper-14\n";

#[derive(Debug)]
pub struct Plan {
    act: Act,
    adding: bool,
    done: bool,
    provider: u32,
    sign_in: Box<[Box<[u8]>]>,
    accounts: Box<[Box<[u8]>]>,
    add: Box<[Box<[u8]>]>,
    wait: Box<[Box<[u8]>]>,
    sign_out: Box<[Box<[u8]>]>,
}
impl Plan {
    #[must_use]
    pub fn new(demo: &Demo) -> Self {
        Self {
            act: Act::SignIn,
            adding: false,
            done: false,
            provider: demo.provider(),
            sign_in: words(&[b"sign-in", b"-"]),
            accounts: words(&[b"accounts", b"--json"]),
            add: words(&[
                b"add-key",
                Decimal::of(u64::from(demo.provider())).as_bytes(),
                demo.llm().key(),
                b"--deployment",
                b"--key",
                b"demo-llm",
            ]),
            wait: words(&[b"wait", b"accounts", b"--until", b"usable"]),
            sign_out: words(&[b"sign-out"]),
        }
    }
    #[must_use]
    pub fn act(&self) -> Act {
        self.act
    }
    #[must_use]
    pub fn next(&self) -> Next<'_> {
        if self.done {
            return Next::Done;
        }
        let words = match self.act {
            Act::SignIn | Act::Spent => &self.sign_in,
            Act::Key => {
                if self.adding {
                    &self.add
                } else {
                    &self.accounts
                }
            }
            Act::Usable => &self.wait,
            Act::SignOut => &self.sign_out,
            Act::Adopt => unreachable!("the wait's end passes the call-free adoption act"),
        };
        Next::Call(Call { words, link: self.act == Act::SignIn || self.act == Act::Spent })
    }
    /// Move on after a call; the wait also reports the call-free fourth act.
    pub fn ended(&mut self, ended: &Ended<'_>) -> Result<Option<Box<[u8]>>, Failed> {
        assert!(!self.done, "no end after Done");
        if self.act == Act::Spent {
            if ended.status != Some(SPENT_STATUS) || ended.stderr != SPENT_STDERR {
                let why: &[u8] = if ended.status == Some(0) {
                    b"the spent link was accepted again"
                } else {
                    b"unexpected spent-link refusal"
                };
                return Err(failure(self.act, why, ended));
            }
            self.done = true;
            return Ok(Some(line(b"6", b"the spent link was refused")));
        }
        if ended.status != Some(0) {
            return Err(failure(self.act, b"call failed", ended));
        }
        let result = match self.act {
            Act::SignIn => {
                self.act = Act::Key;
                Some(line(b"1", b"signed in"))
            }
            Act::Key => {
                let found =
                    if self.adding {
                        false
                    } else {
                        match deployment_account(ended.stdout, self.provider) {
                            Some(found) => found,
                            None => return Err(failure(
                                self.act,
                                b"accounts document lacks accounts.deployment or a provider number, or is invalid JSON",
                                ended,
                            )),
                        }
                    };
                if self.adding || found {
                    self.act = Act::Usable;
                    Some(line(
                        b"2",
                        if found { b"the deployment's key was found" } else { b"the deployment's key was added" },
                    ))
                } else {
                    self.adding = true;
                    None
                }
            }
            Act::Usable => {
                self.act = Act::SignOut;
                Some(crate::read::joined(&[&line(b"3", b"accounts are usable"), ADOPT_LINE]))
            }
            Act::SignOut => {
                self.act = Act::Spent;
                Some(line(b"5", b"signed out"))
            }
            Act::Adopt | Act::Spent => unreachable!("no call for adoption; spent handled above"),
        };
        Ok(result)
    }
}
fn words(parts: &[&[u8]]) -> Box<[Box<[u8]>]> {
    let mut out = List::with_capacity(u32::try_from(parts.len()).expect("fixed argument count"));
    for part in parts {
        out.push((*part).into()).expect("argument count");
    }
    out.into_boxed()
}
fn line(act: &[u8], words: &[u8]) -> Box<[u8]> {
    crate::read::joined(&[b"temper fake seed: act ", act, b": ", words, b"\n"])
}
/// An act's failure always names the child's status and quotes only its first line.
#[must_use]
pub fn failure(act: Act, why: &[u8], ended: &Ended<'_>) -> Failed {
    let number: &[u8] = match act {
        Act::SignIn => b"1",
        Act::Key => b"2",
        Act::Usable => b"3",
        Act::Adopt => b"4",
        Act::SignOut => b"5",
        Act::Spent => b"6",
    };
    let status = match ended.status {
        Some(status) => Decimal::of(u64::from(status)).as_bytes().into(),
        None => Box::<[u8]>::from(b"signal".as_slice()),
    };
    let mut length = ended.stderr.len().min(QUOTE_BYTES);
    for (at, byte) in ended.stderr.get(..length).expect("bounded first line").iter().enumerate() {
        if *byte == b'\n' {
            length = at;
            break;
        }
    }
    Failed {
        act,
        why: line(
            number,
            &crate::read::joined(&[
                why,
                b" (status ",
                &status,
                b", stderr \"",
                ended.stderr.get(..length).expect("bounded first line"),
                b"\")",
            ]),
        ),
    }
}
fn member(d: &skein_json::Document, at: u32, name: &[u8]) -> Option<u32> {
    config::read::Object::new(d, at).ok()?.member(d, name)
}
fn deployment_account(bytes: &[u8], provider: u32) -> Option<bool> {
    let d =
        config::document(bytes, &config::DocumentLimits { bytes: CALL_OUTPUT, tokens: CALL_OUTPUT, depth: 32 }).ok()?;
    let accounts = member(&d, 0, b"accounts")?;
    let deployment = member(&d, accounts, b"deployment")?;
    let rows = config::read::list(&d, deployment, CALL_OUTPUT).ok()?;
    let mut found = false;
    for row in &rows {
        let at = member(&d, *row, b"provider")?;
        let number = config::read::integer(&d, at, u64::from(u32::MAX)).ok()?;
        found |= number == u64::from(provider);
    }
    Some(found)
}
