//! Login observations in the existing accounts page.
use super::accounts::{number, omitted, run};
use crate::{accounts::words::Word as W, words::Words};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run};
use temper_client_domain::limits::AccountLimits;

pub(super) fn hosts(
    b: &mut Builder,
    w: &Words,
    hosts: &skein_lib::List<c::OwnerProgramHost>,
    counts: AccountLimits,
    text_per_node: u32,
) {
    // Hosts share the accounts page's priced account-row budget; excess hosts
    // are omitted so configured membership cannot grow the rendered page.
    for host in hosts.iter().take(usize::try_from(counts.accounts).expect("bounded hosts")) {
        b.open(Role::Paragraph);
        b.open(Role::Text);
        run(b, w, W::Worker);
        whole(b, w, &host.name, text_per_node);
        number(b, w, host.channel);
        if !host.connected {
            run(b, w, W::NotInContact);
        }
        if host.awaiting_logins {
            run(b, w, W::AwaitingLogins);
        }
        if host.cut {
            run(b, w, W::Cut);
        }
        b.close();
        b.close();
        for program in host.programs.iter().take(usize::try_from(counts.sources).expect("bounded programs")) {
            b.open(Role::Paragraph);
            b.open(Role::Text);
            whole(b, w, &program.name, text_per_node);
            match &program.state {
                None => run(b, w, W::Unknown),
                Some(c::OwnerProgramState::Usable) => run(b, w, W::Ready),
                Some(c::OwnerProgramState::NotSignedIn) => run(b, w, W::NeedsSignIn),
                Some(c::OwnerProgramState::SignedInOtherwise { method }) => {
                    run(b, w, W::SignedInOtherwise);
                    whole(b, w, method, text_per_node);
                }
                Some(c::OwnerProgramState::Failed { reason, detail }) => failure(b, w, *reason, detail, text_per_node),
            }
            match &program.sign_out {
                None => {}
                Some(c::OwnerProgramSignOut::SignedOut) => run(b, w, W::SignedOut),
                Some(c::OwnerProgramSignOut::Failed { reason, detail }) => {
                    failure(b, w, *reason, detail, text_per_node);
                }
            }
            if program.cleanup {
                run(b, w, W::Cleanup);
            }
            if program.cut {
                run(b, w, W::Cut);
            }
            b.close();
            b.close();
        }
        omitted(b, w, W::ProgramsOmitted, host.programs.len(), counts.sources);
    }
    omitted(b, w, W::ProgramsOmitted, hosts.len(), counts.accounts);
}
fn whole(b: &mut Builder, w: &Words, bytes: &[u8], text_per_node: u32) {
    b.run(Run::Plain, b" ");
    if bytes.len() <= usize::try_from(text_per_node / 8).expect("text bound fits") {
        b.run(Run::Plain, bytes);
    } else {
        run(b, w, W::Cut);
    }
    b.run(Run::Plain, b" ");
}
fn failure(b: &mut Builder, w: &Words, reason: c::OwnerProgramFailure, detail: &[u8], text_per_node: u32) {
    run(
        b,
        w,
        match reason {
            c::OwnerProgramFailure::Missing => W::Missing,
            c::OwnerProgramFailure::Deadline => W::Deadline,
            c::OwnerProgramFailure::Unreadable => W::Unreadable,
            c::OwnerProgramFailure::Exited => W::Exited,
            c::OwnerProgramFailure::Unconfigured => W::Unconfigured,
            c::OwnerProgramFailure::Unknown => W::Unknown,
            c::OwnerProgramFailure::Closed => W::Closing,
        },
    );
    whole(b, w, detail, text_per_node);
}
pub(super) fn failures(b: &mut Builder, w: &Words, hosts: &[c::AccountHostFailure], text_per_node: u32) {
    // The contract caps the retained answer at 128 configured workers.
    for host in hosts {
        b.open(Role::Paragraph);
        b.open(Role::Text);
        whole(b, w, &host.name, text_per_node);
        run(
            b,
            w,
            match host.why {
                c::AccountHostWhy::NotInContact => W::NotInContact,
                c::AccountHostWhy::Lost => W::Lost,
                c::AccountHostWhy::Failed => W::Failed,
            },
        );
        b.close();
        b.close();
    }
}
