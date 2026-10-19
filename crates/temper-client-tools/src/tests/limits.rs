use crate::limits;
#[test]
fn each_ceiling_is_within_the_client_limit_it_reads_from() {
    let client = limits::client().client.domain;
    assert_eq!(limits::ROWS, client.rows, "a page's rows are the client's rows per window");
    assert_eq!(limits::TEXT, client.text, "a result's text is the client's text");
    assert!(u64::from(limits::TEXT) <= client.document_bytes, "a result's text fits one document");
    assert!(limits::PART <= limits::TEXT && limits::PART <= client.turn_bytes, "a part fits the text and a turn");
    assert!(limits::WAIT <= limits::WAIT_MOST && limits::LOCK_TRY <= limits::LOCK, "each default within its most");
}
#[test]
fn the_tools_preset_is_the_terminals_but_its_rows() {
    let mut terminal = temper_client_shell_terminal::limits::native().client;
    terminal.client.domain.rows = limits::ROWS;
    terminal.client.domain.text = limits::TEXT;
    assert_eq!(limits::client(), terminal);
}
#[test]
fn the_tools_preset_has_a_worst_case() {
    assert!(temper_client_domain::worst_case(&limits::client()).is_some(), "the tools preset is priced");
}
#[test]
fn the_rows_repeats_are_within_the_client_limits() {
    use crate::jig::table::Occurs;
    let limits = limits::client();
    for row in crate::table::TOOLS {
        let most = if row.name == b"adopt" { limits.forge.checks } else { limits.client.domain.delegates };
        for form in row.forms {
            for argument in *form {
                match argument.occurs {
                    Occurs::Repeated { most: repeated } => assert!(repeated <= most, "{:?}", row.name),
                    Occurs::Required | Occurs::Optional => {}
                }
            }
        }
    }
}
