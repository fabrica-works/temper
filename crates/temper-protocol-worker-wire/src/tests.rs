//! Domain bounds and identity field rules (protocol/channel.md, sections 6 and 8).
mod fixtures;
use crate::v1;
use alloc::boxed::Box;
use skein_lib::List;

#[test]
fn limits_take_each_bound_from_its_source_field() {
    assert_eq!(
        fixtures::limits(),
        v1::Limits {
            merge_branch: 16,
            branch_branch: 16,
            writable_push: 16,
            repository_name: 16,
            repository_remote: 48,
            repository_git_some: 4096,
            items_repositories: 2,
            items_save_some: 16,
            left_landed: 2,
            files_files: 2,
            files_files_item: 8,
            diagnostic_output: 512,
            saved_repositories: 2,
            identity_token: 40,
            identity_login: 16,
            identity_name: 16,
            identity_email: 16,
        }
    );
    let bounds = crate::Bounds { repositories: 65, ..fixtures::BOUNDS };
    assert_eq!(crate::limits(&bounds), None, "past the repository ceiling");
}

#[test]
fn items_one_repository_past_their_bound_are_refused() {
    let limits = fixtures::limits();
    v1::Items::new(&limits, fixtures::items(2)).expect("at the bound");
    let problem = v1::Items::new(&limits, fixtures::items(3)).expect_err("past the count");
    assert_eq!(problem, v1::Problem { path: v1::Path::ItemsRepositories, reason: skein_codec::Reason::Bound });
}

#[test]
fn repository_byte_fields_stop_at_their_bounds() {
    let limits = fixtures::limits();
    for (program, bound, path) in
        [(false, 48_usize, v1::Path::RepositoryRemote), (true, 4096_usize, v1::Path::RepositoryGit)]
    {
        for past in [false, true] {
            let length = if past { bound.checked_add(1).expect("test length") } else { bound };
            let mut parts = fixtures::repository_parts(1, Box::from(&b"remote"[..]));
            let mut bytes = skein_lib::bytes::zeroed(length);
            bytes.fill(b'x');
            if program {
                parts.git = Some(bytes);
            } else {
                parts.remote = bytes;
            }
            let made = v1::Repository::new(&limits, parts);
            if past {
                assert_eq!(made.expect_err("past bound"), v1::Problem { path, reason: skein_codec::Reason::Bound });
            } else {
                made.expect("at bound");
            }
        }
    }
}

#[test]
fn saved_conflicts_past_their_count_are_refused() {
    let mut files = List::with_capacity(3);
    for _ in 0_u32..3 {
        files.push(Box::from(&b"file"[..])).expect("counted room");
    }
    let conflicts = v1::Files::new(&v1::CEILINGS, v1::FilesParts { files }).expect("within ceilings");
    let repository = v1::SavedRepository::new(
        &v1::CEILINGS,
        v1::SavedRepositoryParts { tag: 1, from_saved: false, landing: v1::Landing::Conflicted(conflicts) },
    )
    .expect("within ceilings");
    let mut repositories = List::with_capacity(1);
    repositories.push(repository).expect("counted room");
    let problem =
        v1::Saved::new(&fixtures::limits(), v1::SavedParts { repositories }).expect_err("past the conflicts count");
    assert_eq!(problem, v1::Problem { path: v1::Path::FilesFiles, reason: skein_codec::Reason::Bound });
}

#[test]
fn largest_parts_are_the_generated_worst_cases() {
    let limits = fixtures::limits();
    let sizes = crate::largest(&limits).expect("test limits fit");
    assert_eq!(u64::from(sizes.items), v1::Items::worst_case_bytes(&limits).expect("items"));
    assert_eq!(u64::from(sizes.left), v1::Left::worst_case_bytes(&limits).expect("left"));
    assert_eq!(u64::from(sizes.saved), v1::Saved::worst_case_bytes(&limits).expect("saved"));
    assert_eq!(u64::from(sizes.resource), v1::Preparation::worst_case_bytes(&limits).expect("preparation"));
    assert_eq!(u64::from(sizes.grant), v1::Identity::worst_case_bytes(&limits).expect("identity"));
    assert_eq!(sizes, crate::Largest { items: 8601, left: 76, saved: 1076, resource: 6, grant: 104 });
}

#[test]
fn identity_rules_refuse_each_forbidden_byte() {
    let original = fixtures::identity(b"forge-token");
    assert!(crate::valid(&original), "ordinary identity");
    let rows: [(&[u8], &[u8], &[u8]); 12] = [
        (b"a:b", b"name", b"email"),
        (b"a\x7f", b"name", b"email"),
        (b"login", b"", b"email"),
        (b"login", b"name", b""),
        (b"login", b"a\0b", b"email"),
        (b"login", b"a\nb", b"email"),
        (b"login", b"a<b", b"email"),
        (b"login", b"a>b", b"email"),
        (b"login", b"name", b"a\0b"),
        (b"login", b"name", b"a\nb"),
        (b"login", b"name", b"a<b"),
        (b"login", b"name", b"a>b"),
    ];
    for (row, (login, name, email)) in rows.iter().enumerate() {
        let parts = v1::IdentityParts {
            login: Box::from(*login),
            name: Box::from(*name),
            email: Box::from(*email),
            ..original.clone().into_parts()
        };
        let identity = v1::Identity::new(&fixtures::limits(), parts).expect("bounded valid text");
        assert!(!crate::valid(&identity), "forbidden row {row}");
    }
}
