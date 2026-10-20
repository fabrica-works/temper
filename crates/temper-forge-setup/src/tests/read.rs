// Fixture parsing without a deployment keeps written paths.
use super::fixtures;
use crate::{
    limits,
    problem::{self, Problems},
    setup::{Mode, Setup},
};
use core::net::{Ipv4Addr, SocketAddr};
#[test]
fn an_owned_entry_with_a_runner_reads_into_its_members() {
    let document = fixtures::document(fixtures::TWO);
    let mut problems = Problems::empty();
    let setup =
        Setup::read(&document, 0, &limits::small(), &jig_protocol_config::Directories::working(), &mut problems)
            .expect("design example");
    assert!(problems.is_empty());
    let entry = setup.entry(2).unwrap();
    assert_eq!(&*entry.directory, b"forge-2");
    let Mode::Owned(owned) = &entry.mode else { panic!("owned") };
    assert_eq!(owned.listen, SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)));
    assert_eq!(owned.runner.as_ref().unwrap().capacity, 1);
}
#[test]
fn a_managed_entry_reads_its_admin_token_file() {
    let setup = fixtures::setup();
    assert_eq!(setup.entry(1).unwrap().mode, Mode::Managed { admin_token: b"secrets/forgejo-admin".as_slice().into() });
}
#[test]
fn an_owned_entry_reads_a_null_runner_as_none_and_refuses_a_missing_one() {
    let from = br#""runner":{"binary":"/opt/forgejo/forgejo-runner","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"},"#;
    let null = fixtures::changed(from, br#""runner":null,"#);
    let setup = Setup::read(
        &fixtures::document(&null),
        0,
        &limits::small(),
        &jig_protocol_config::Directories::working(),
        &mut Problems::empty(),
    )
    .unwrap();
    let Mode::Owned(owned) = &setup.entry(2).unwrap().mode else { panic!("owned") };
    assert_eq!(owned.runner, None);
    refused(&fixtures::changed(from, b""), b"application.setup[1].mode.owned.runner: missing\n");
}
fn refused(bytes: &[u8], expected: &[u8]) {
    let mut problems = Problems::empty();
    assert_eq!(
        Setup::read(
            &fixtures::document(bytes),
            0,
            &limits::small(),
            &jig_protocol_config::Directories::working(),
            &mut problems
        ),
        None
    );
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn an_entry_past_the_orgs_limit_is_refused() {
    let bytes = fixtures::changed(br#""orgs":["acme"]"#, br#""orgs":["acme","extra"]"#);
    let limits = skein_forge_setup::limits::Limits { orgs: 1, ..limits::small() }; // Below the shipped eight.
    let mut problems = Problems::empty();
    assert_eq!(
        Setup::read(
            &fixtures::document(&bytes),
            0,
            &limits,
            &jig_protocol_config::Directories::working(),
            &mut problems
        ),
        None
    );
    assert_eq!(&*problem::lines(&problems), b"application.setup[0].orgs: past its bound\n");
}
#[test]
fn a_member_outside_its_bounds_is_refused_naming_its_path() {
    for (from, to, expected) in [
        (
            br#""orgs":["acme"]"#.as_slice(),
            br#""orgs":[]"#.as_slice(),
            b"application.setup[0].orgs: invalid form\n".as_slice(),
        ),
        (br#""capacity":1"#, br#""capacity":0"#, b"application.setup[1].mode.owned.runner.capacity: invalid form\n"),
        (
            br#""labels":["temper-host:host"]"#,
            br#""labels":[]"#,
            b"application.setup[1].mode.owned.runner.labels: invalid form\n",
        ),
        (
            br#""forgejo":"/opt/forgejo/forgejo""#,
            br#""forgejo":"""#,
            b"application.setup[1].mode.owned.forgejo: invalid form\n",
        ),
        (
            br#""binary":"/opt/forgejo/forgejo-runner""#,
            br#""binary":"""#,
            b"application.setup[1].mode.owned.runner.binary: invalid form\n",
        ),
    ] {
        refused(&fixtures::changed(from, to), expected);
    }
}

#[test]
fn setup_paths_join_their_kind_keep_absolute_values_and_refuse_escaping_relatives() {
    let directories = jig_protocol_config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    for (from, to, kind, expected) in [
        (
            br#""directory":"forge-2""#.as_slice(),
            br#""directory":"forge-2""#.as_slice(),
            "directory",
            b"/data/forge-2".as_slice(),
        ),
        (br#""directory":"forge-2""#, br#""directory":"/kept/forge""#, "directory", b"/kept/forge"),
        (br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"forgejo""#, "forgejo", b"/cache/forgejo"),
        (br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"/kept/forgejo""#, "forgejo", b"/kept/forgejo"),
        (br#""binary":"/opt/forgejo/forgejo-runner""#, br#""binary":"runner""#, "runner", b"/cache/runner"),
        (br#""binary":"/opt/forgejo/forgejo-runner""#, br#""binary":"/kept/runner""#, "runner", b"/kept/runner"),
        (br#""admin_token":"secrets/forgejo-admin""#, br#""admin_token":"admin""#, "admin", b"/data/admin"),
        (br#""admin_token":"secrets/forgejo-admin""#, br#""admin_token":"/kept/admin""#, "admin", b"/kept/admin"),
    ] {
        let mut problems = Problems::empty();
        let setup = Setup::read(
            &fixtures::document(&fixtures::changed(from, to)),
            0,
            &limits::small(),
            &directories,
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty());
        let entry = setup.entry(2).unwrap();
        let Mode::Owned(owned) = &entry.mode else { panic!("owned") };
        let value = match kind {
            "directory" => &entry.directory,
            "forgejo" => &owned.forgejo,
            "runner" => &owned.runner.as_ref().unwrap().binary,
            "admin" => {
                let Mode::Managed { admin_token } = &setup.entry(1).unwrap().mode else { panic!("managed") };
                admin_token
            }
            _ => unreachable!(),
        };
        assert_eq!(value.as_ref(), expected, "{kind}");
    }
    for (from, to, path) in [
        (
            br#""directory":"forge-2""#.as_slice(),
            br#""directory":"../forge""#.as_slice(),
            "application.setup[1].directory",
        ),
        (
            br#""forgejo":"/opt/forgejo/forgejo""#,
            br#""forgejo":"../forgejo""#,
            "application.setup[1].mode.owned.forgejo",
        ),
        (
            br#""binary":"/opt/forgejo/forgejo-runner""#,
            br#""binary":"../runner""#,
            "application.setup[1].mode.owned.runner.binary",
        ),
    ] {
        let mut problems = Problems::empty();
        assert!(
            Setup::read(
                &fixtures::document(&fixtures::changed(from, to)),
                0,
                &limits::small(),
                &directories,
                &mut problems
            )
            .is_none()
        );
        assert!(problem::lines(&problems).starts_with(path.as_bytes()));
    }
}
