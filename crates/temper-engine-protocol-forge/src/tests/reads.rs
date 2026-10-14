//! Document translations; the byte-level read-plan tests follow in 13.3b.
use crate::reading;
use alloc::boxed::Box;
use skein_lib::{Time, Wall};
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::document as d;

#[test]
fn revisions_and_provenance_follow_the_body_and_its_times() {
    let rows = [
        (
            "equal",
            kept(),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(1)),
            api::Provenance::Original,
            0xba78_16bf_8f01_cfea,
        ),
        (
            "later",
            kept(),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(3)),
            api::Provenance::Revised,
            0xba78_16bf_8f01_cfea,
        ),
        ("missing", kept(), Some(Wall::from_nanos(1)), None, api::Provenance::Unknown, 0xba78_16bf_8f01_cfea),
        (
            "long",
            d::Body::Long(5),
            Some(Wall::from_nanos(1)),
            Some(Wall::from_nanos(3)),
            api::Provenance::Unknown,
            0x46c3_95ed_52b0_b56f,
        ),
    ];
    for (name, body, created, updated, provenance, revision) in rows {
        assert_eq!(reading::provenance(created, updated, &body), provenance, "{name}: provenance");
        assert_eq!(reading::revision(&body, updated), revision, "{name}: revision");
    }
}
fn kept() -> d::Body {
    d::Body::Kept(Box::from(&b"abc"[..]))
}

#[test]
fn an_actions_status_reads_with_no_creator_and_an_absolute_link() {
    let status = d::CommitStatus {
        creator: None,
        updated_at: Wall::from_nanos(2),
        context: Box::from(&b"build"[..]),
        status: d::StatusState::Failure,
        description: Box::from(&b"failed"[..]),
        target_url: Box::from(&b"/fixture/specimen/actions/runs/1/jobs/0"[..]),
    };
    assert_eq!(
        reading::status(status, b"http://127.0.0.1:3000/fixture/specimen"),
        Ok(api::Status {
            at: Time::from_nanos(2),
            context: Box::from(&b"build"[..]),
            check: api::Check::Failed,
            description: Box::from(&b"failed"[..]),
            url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0"[..]),
            job: None
        }),
        "Actions path has its origin, no author or job"
    );
}

#[test]
fn a_local_status_link_gets_its_repositorys_origin() {
    let rows: [(&str, &[u8], &[u8]); 3] = [
        ("local", b"/job", b"https://forge.test:443/job"),
        ("other origin", b"https://ci.test/job", b"https://ci.test/job"),
        ("empty", b"", b""),
    ];
    for (name, link, expected) in rows {
        assert_eq!(reading::absolute(link, b"https://forge.test:443/owner/repo"), Ok(Box::from(expected)), "{name}");
    }
}

#[test]
fn combined_statuses_map_their_state_and_zero_contexts_override_it() {
    let rows = [
        ("pending", d::StatusState::Pending, api::Ci::Pending),
        ("success", d::StatusState::Success, api::Ci::Passed),
        ("warning", d::StatusState::Warning, api::Ci::Passed),
        ("skipped", d::StatusState::Skipped, api::Ci::Passed),
        ("error", d::StatusState::Error, api::Ci::Failed),
        ("failure", d::StatusState::Failure, api::Ci::Failed),
    ];
    for (name, state, expected) in rows {
        assert_eq!(reading::ci(Some(state), 0), api::Ci::None, "{name}: no contexts");
        assert_eq!(reading::ci(Some(state), 1), expected, "{name}: contexts");
    }
    assert_eq!(reading::ci(None, 0), api::Ci::None, "empty state and no contexts");
}

#[test]
fn status_states_map_to_the_clients_check() {
    for (name, state, expected) in [
        ("pending", d::StatusState::Pending, api::Check::Pending),
        ("success", d::StatusState::Success, api::Check::Passed),
        ("warning", d::StatusState::Warning, api::Check::Passed),
        ("skipped", d::StatusState::Skipped, api::Check::Passed),
        ("error", d::StatusState::Error, api::Check::Failed),
        ("failure", d::StatusState::Failure, api::Check::Failed),
    ] {
        assert_eq!(reading::check(state), expected, "{name}");
    }
}

#[test]
fn pending_reviews_are_skipped_and_dismissed_reviews_are_comments() {
    for (name, state, dismissed, expected) in [
        ("approved", d::ReviewState::Approved, false, Some(api::Verdict::Approve)),
        ("changes", d::ReviewState::RequestChanges, false, Some(api::Verdict::RequestChanges)),
        ("comment", d::ReviewState::Comment, false, Some(api::Verdict::Comment)),
        ("pending", d::ReviewState::Pending, false, None),
        ("request", d::ReviewState::RequestReview, false, None),
        ("dismissed", d::ReviewState::Approved, true, Some(api::Verdict::Comment)),
    ] {
        assert_eq!(reading::verdict(state, dismissed), expected, "{name}");
    }
}

#[test]
fn an_owner_is_read_as_admin() {
    for (name, permission, expected) in [
        ("none", d::Permission::None, api::Permission::None),
        ("read", d::Permission::Read, api::Permission::Read),
        ("write", d::Permission::Write, api::Permission::Write),
        ("admin", d::Permission::Admin, api::Permission::Admin),
        ("owner", d::Permission::Owner, api::Permission::Admin),
    ] {
        assert_eq!(reading::permission(permission), expected, "{name}");
    }
}

#[test]
fn an_object_id_must_match_its_repositorys_format() {
    use temper_forge_forgejo::id::{ObjectFormat, ObjectId};
    for (name, from, repository, expected) in [
        ("sha1", ObjectFormat::Sha1, ObjectFormat::Sha1, Some([0; 32])),
        ("sha256", ObjectFormat::Sha256, ObjectFormat::Sha256, Some([0; 32])),
        ("another format", ObjectFormat::Sha1, ObjectFormat::Sha256, None),
    ] {
        assert_eq!(reading::commit(ObjectId { format: from, bytes: [0; 32] }, repository), expected, "{name}");
    }
}

#[test]
fn a_summary_keeps_its_marker_and_a_long_body_carries_no_key_or_bytes() {
    let rows = [
        (
            "kept",
            d::Body::Kept(Box::from(&b"<!-- temper:key 01 -->\ntext"[..])),
            Some(Box::from([1])),
            Box::from(&b"<!-- temper:key 01 -->\ntext"[..]),
        ),
        ("long", d::Body::Long(30), None, Box::from(&b""[..])),
    ];
    for (name, body, key, expected_body) in rows {
        let summary = reading::summary(d::Issue {
            number: 7,
            pull: true,
            state: d::IssueState::Open,
            user: d::User { id: 2, login: Box::from(&b"writer"[..]) },
            labels: Box::from([]),
            updated_at: Wall::from_nanos(3),
            title: Box::from(&b"title"[..]),
            body,
        });
        assert_eq!(
            summary,
            api::Summary {
                number: 7,
                kind: api::Kind::Pull,
                state: api::State::Open,
                author: 2,
                labels: Box::from([]),
                updated: Time::from_nanos(3),
                title: Box::from(&b"title"[..]),
                key,
                body: expected_body
            },
            "{name}"
        );
    }
}
