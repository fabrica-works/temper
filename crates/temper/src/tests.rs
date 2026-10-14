use super::configure;

#[test]
fn serve_without_a_configuration_path_is_a_usage_error() {
    assert_eq!(
        configure(["serve".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json\n".into())
    );
}
#[test]
fn an_unknown_command_prints_the_usage_line() {
    assert_eq!(
        configure(["unknown".into(), "config.json".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json\n".into())
    );
}
