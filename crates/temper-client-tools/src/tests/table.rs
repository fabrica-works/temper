use super::fixtures;
use crate::jig::{
    argv::{self, Unwritten, Usage},
    call::{Parsed, Project, Source, Value, Values},
    table::{Keyed, Kind, Occurs, Tool, positional},
};
use jig_client::contract::QueryOrder;
use skein_lib::List;
const READS: &[&[&str]] = &[
    &["me"],
    &["inbox", "--project", "shop", "--kind", "replies", "--root", "41"],
    &["tasks", "--project", "3", "--label", "chat", "--tracked", "--order", "priority"],
    &["tasks", "--project", "3"],
    &["task", "41", "--tree", "--activity", "5", "--run"],
    &["chat", "41", "--before", "0a1b"],
    &["activity", "41", "--limit", "32"],
    &["turn", "41", "3.7", "--part", "2", "--from", "16384"],
    &["run", "41"],
];
#[test]
fn each_read_parses_into_its_call_and_prints_back() {
    for words in READS {
        let call = fixtures::call(words);
        assert_eq!(fixtures::back(&call), fixtures::owned(words), "{words:?}");
        let written = fixtures::back(&call);
        let mut borrowed = List::with_capacity(u32::try_from(written.len()).expect("bounded words"));
        for word in &written {
            borrowed.push(word.as_ref()).expect("bounded words");
        }
        assert_eq!(crate::table::parse(borrowed.as_slice()), Ok(Parsed::Call { call, json: false }), "{words:?}");
    }
}
const HOST: &[&[&str]] =
    &[&["sign-in", "link"], &["sign-in", "-"], &["sign-out"], &["pending"], &["help"], &["help", "inbox"]];
#[test]
fn each_host_tool_parses_into_its_call_and_prints_back() {
    for words in HOST {
        assert_eq!(fixtures::back(&fixtures::call(words)), fixtures::owned(words), "{words:?}");
    }
    for words in fixtures::EVERY_TOOL {
        assert!(fixtures::parse(words).is_ok(), "{words:?}");
    }
}
const UNKNOWN: &[(&[&str], Usage)] = &[
    (&["absent"], Usage::Tool),
    (&["me", "--kind", "replies"], Usage::Unknown { at: 0 }),
    (&["me", "extra"], Usage::Extra { at: 0 }),
    (&["inbox", "--no-project"], Usage::Unknown { at: 0 }),
];
#[test]
fn an_unknown_tool_or_argument_is_a_usage_error() {
    for (words, error) in UNKNOWN {
        assert_eq!(fixtures::parse(words), Err(*error), "{words:?}");
    }
}
const MISSING: &[(&[&str], Usage)] = &[
    (&["tasks"], Usage::Missing { argument: 0 }),
    (&["task"], Usage::Missing { argument: 0 }),
    (&["turn", "41"], Usage::Missing { argument: 1 }),
    (&["inbox", "--root"], Usage::Missing { argument: 2 }),
    (&["inbox", "--root", "41", "--root", "42"], Usage::Twice { argument: 2 }),
    (&["task", "41", "--tree", "--tree"], Usage::Twice { argument: 1 }),
];
#[test]
fn a_missing_or_repeated_argument_is_a_usage_error() {
    for (words, error) in MISSING {
        assert_eq!(fixtures::parse(words), Err(*error), "{words:?}");
    }
}
const INVALID: &[(&[&str], Usage)] = &[
    (&["task", "x"], Usage::Kind { argument: 0 }),
    (&["task", "18446744073709551616"], Usage::Kind { argument: 0 }),
    (&["tasks", "--project", "4294967296"], Usage::Kind { argument: 0 }),
    (&["tasks", "--project", "3", "--order", "ended"], Usage::Kind { argument: 5 }),
    (&["tasks", "--project", "3", "--order", "changed"], Usage::Kind { argument: 5 }),
    (&["inbox", "--kind", "other"], Usage::Kind { argument: 1 }),
    (&["chat", "41", "--before", "AB"], Usage::Kind { argument: 1 }),
    (&["chat", "41", "--before", "a"], Usage::Kind { argument: 1 }),
    (&["turn", "41", "3.4294967296"], Usage::Kind { argument: 1 }),
    (&["activity", "41", "--limit", "33"], Usage::Rows { argument: 2 }),
    (&["activity", "41", "--limit", "0"], Usage::Rows { argument: 2 }),
];
#[test]
fn a_value_outside_its_kind_or_rows_is_a_usage_error() {
    for (words, error) in INVALID {
        assert_eq!(fixtures::parse(words), Err(*error), "{words:?}");
    }
}
#[test]
fn a_project_is_a_number_or_a_name() {
    for (word, project) in [("3", Project::Number(3)), ("shop", Project::Name(Box::from(*b"shop")))] {
        assert_eq!(
            fixtures::call(&["tasks", "--project", word]).values[0],
            Some(Values::One(Value::Project(project))),
            "{word}"
        );
    }
}
/// The parser checks stdin ownership for every table, including later text rows.
const STDIN_PAIR: Tool = Tool {
    name: b"pair",
    line: b"",
    result: b"",
    forms: &[&[
        positional(b"first", b"FILE", Kind::File, Occurs::Required, b""),
        positional(b"second", b"TEXT", Kind::Text, Occurs::Required, b""),
    ]],
    keyed: Keyed::Never,
    area: 1,
};
#[test]
fn a_second_stdin_is_a_usage_error() {
    assert_eq!(argv::parse(&[&STDIN_PAIR], &[b"pair", b"-", b"-"], &crate::limits::tools()), Err(Usage::Stdin));
}
#[test]
fn call_and_json_are_the_hosts_options() {
    assert_eq!(fixtures::parse(&["--call", "-", "--json"]), Ok(Parsed::File { file: Source::Stdin, json: true }));
    assert_eq!(
        fixtures::parse(&["--json", "--call", "input"]),
        Ok(Parsed::File { file: Source::Path(Box::from(*b"input")), json: true })
    );
    assert_eq!(fixtures::parse(&["me", "--json"]), Ok(Parsed::Call { call: fixtures::call(&["me"]), json: true }));
    for words in [&["--call"][..], &["--call", "-", "me"], &["me", "--json", "--json"]] {
        assert_eq!(fixtures::parse(words), Err(Usage::Json), "{words:?}");
    }
    assert_eq!(
        fixtures::back(&fixtures::call(&["sign-in", "--", "--json"])),
        fixtures::owned(&["sign-in", "--", "--json"])
    );
}
#[test]
fn tasks_default_to_number_and_unwritten_orders_are_refused() {
    let mut call = fixtures::call(&["tasks", "--project", "3"]);
    assert_eq!(call.values[5], Some(Values::One(Value::Order(QueryOrder::Number))));
    for order in [QueryOrder::Changed, QueryOrder::Ended] {
        call.values[5] = Some(Values::One(Value::Order(order)));
        assert_eq!(
            argv::back(crate::table::TOOLS, &call, &mut List::with_capacity(16)),
            Err(Unwritten::NoForm),
            "{order:?}"
        );
    }
}
