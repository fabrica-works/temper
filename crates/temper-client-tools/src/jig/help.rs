//! Usage and records derived from the tools' table.
use super::table::{Argument, Keyed, Kind, Occurs, Place, Tool};
use skein_lib::{Overflow, Writer};
/// Write every form in the table's published argument order.
pub fn usage(tool: &Tool, out: &mut Writer) -> Result<(), Overflow> {
    for form in tool.forms {
        out.put(tool.name)?;
        for argument in *form {
            argument_usage(argument, out)?;
        }
        out.put(match tool.keyed {
            Keyed::Never => b"",
            Keyed::Optional => b" [--key NAME]",
            Keyed::Required => b" --key NAME",
        })?;
        out.put(b"\n")?;
    }
    Ok(())
}
fn argument_usage(argument: &Argument, out: &mut Writer) -> Result<(), Overflow> {
    out.put(b" ")?;
    let optional = argument.occurs != Occurs::Required;
    if optional {
        out.put(b"[")?;
    }
    match argument.place {
        Place::Positional => shown(argument, out)?,
        Place::Flag | Place::Switch => {
            out.put(b"--")?;
            out.put(argument.name)?;
            if argument.place == Place::Flag {
                out.put(b" ")?;
                shown(argument, out)?;
            }
        }
    }
    if optional {
        out.put(b"]")?;
    }
    if let Occurs::Repeated { .. } = argument.occurs {
        out.put(b"...")?;
    }
    Ok(())
}
fn shown(argument: &Argument, out: &mut Writer) -> Result<(), Overflow> {
    match argument.kind {
        Kind::Proposal => out.put(b"tN/K|pN/K"),
        Kind::Order => out.put(b"activity|priority"),
        Kind::Word { set } => {
            let mut separator = b"".as_slice();
            for word in set {
                out.put(separator)?;
                out.put(word)?;
                separator = b"|";
            }
            Ok(())
        }
        Kind::Number
        | Kind::Count
        | Kind::Text
        | Kind::Literal
        | Kind::File
        | Kind::Duration
        | Kind::Digest
        | Kind::Condition
        | Kind::Subject
        | Kind::Project
        | Kind::Turn
        | Kind::Cursor
        | Kind::Group
        | Kind::Tool => out.put(argument.shown),
    }
}
/// The table's complete list, following the agents' guide.
pub fn tools(table: &[&Tool], out: &mut Writer) -> Result<(), Overflow> {
    for row in table {
        usage(row, out)?;
        out.put(b"  ")?;
        out.put(row.line)?;
        out.put(b"\n")?;
    }
    Ok(())
}
/// One tool's forms, descriptions and result tag.
pub fn tool(tool: &Tool, out: &mut Writer) -> Result<(), Overflow> {
    usage(tool, out)?;
    out.put(tool.line)?;
    out.put(b"\nResult: ")?;
    out.put(tool.result)?;
    out.put(b"\n")?;
    for form in tool.forms {
        for argument in *form {
            out.put(b"  ")?;
            out.put(argument.name)?;
            out.put(b": ")?;
            out.put(argument.line)?;
            out.put(b"\n")?;
        }
    }
    if tool.name == b"wait" {
        wait(out)?;
    }
    Ok(())
}
fn wait(out: &mut Writer) -> Result<(), Overflow> {
    out.put(b"Subjects and --until conditions:\n  me: waiting\n  inbox: waiting|updates (no subject flags)\n  task N: ended|waiting-on-me|held|replied|phase:<phase>\n  tree N: ended\n  run N: ended\n  accounts: usable|settled\n  tasks: no condition; its read arguments except --after and --ended\nLive phases: ")?;
    let mut separator = b"".as_slice();
    for phase in super::subject::PHASES {
        if super::subject::live(*phase) {
            out.put(separator)?;
            out.put(super::subject::phase_word(*phase))?;
            separator = b"|";
        }
    }
    out.put(b"\nUse --until ended for done, failed, cancelled or ended; terminal phase conditions are usage errors.\nwaiting-on-me covers the task's tree. An ended task answers with its ended read.\n--since is the matching read's digest of the watched document, not its cursor.\n--since and --until together require both change and condition.\nTimeout: 300 seconds by default, at most 3600 seconds.\n")
}
