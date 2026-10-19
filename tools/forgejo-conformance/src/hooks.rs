use crate::{
    Result,
    cases::Recorder,
    facts::{Fact, add, case},
    fixture::Fixture,
};
use serde_json::{Value, json};
use skein_throwaway_forgejo::Token;
use std::{collections::BTreeSet, path::Path, thread, time::Duration};
const ROUTE: &str = "/api/v1/orgs/ensemble/hooks";
const FIRST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const SECOND: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const EVENTS: &[&str] = &[
    "issues",
    "issue_assign",
    "issue_label",
    "issue_milestone",
    "pull_request",
    "pull_request_assign",
    "pull_request_label",
    "pull_request_milestone",
    "pull_request_sync",
    "pull_request_review_request",
    "issue_comment",
    "pull_request_comment",
    "pull_request_review_approved",
    "pull_request_review_rejected",
    "pull_request_review_comment",
    "push",
    "create",
    "delete",
    "workflow_job_success",
    "workflow_job_failure",
    "workflow_job_cancelled",
    "workflow_job_skipped",
];
fn body(port: u16, path: &str, secret: &str, events: &[&str], active: bool, create: bool) -> String {
    let kind = if create { "\"type\":\"forgejo\"," } else { "" };
    format!(
        "{{{kind}\"config\":{{\"url\":\"http://127.0.0.1:{port}/{path}\",\"content_type\":\"json\",\"secret\":\"{secret}\"}},\"events\":{},\"active\":{active}}}",
        serde_json::to_string(events).expect("event names")
    )
}
pub(crate) fn record(r: &mut Recorder<'_>, admin: &Token) -> Result<()> {
    r.send("setup-hook-list-none", "GET", ROUTE, None, Some(admin), None)?;
    let port = r.f.instance.port();
    let make = body(port, "hooks/1", FIRST, EVENTS, true, true);
    let other = body(port, "other", FIRST, &["push"], true, true);
    let mut ids = Vec::new();
    for (name, body) in [("setup-hook-made", &make), ("setup-hook-made-twin", &make), ("setup-hook-made-other", &other)]
    {
        // Every captured creation timestamp first appears in its own second.
        thread::sleep(Duration::from_secs(1));
        let answer = r.send(name, "POST", ROUTE, Some(body), Some(admin), None)?;
        ids.push(crate::json(&answer)?["id"].as_u64().ok_or("hook id absent")?);
    }
    for (name, query) in [
        ("setup-hook-list", ""),
        ("setup-hook-list-page-1", "?page=1&limit=1"),
        ("setup-hook-list-page-4", "?page=4&limit=1"),
    ] {
        r.send(name, "GET", &format!("{ROUTE}{query}"), None, Some(admin), None)?;
    }
    let first = format!("{ROUTE}/{}", ids[0]);
    for (name, events, active) in
        [("setup-hook-patched", EVENTS, true), ("setup-hook-patched-narrow", &["push"][..], false)]
    {
        let patch = body(port, "hooks/1", SECOND, events, active, false);
        r.send(name, "PATCH", &first, Some(&patch), Some(admin), None)?;
    }
    let patch = body(port, "hooks/1", SECOND, EVENTS, true, false);
    r.send("setup-hook-patch-missing", "PATCH", &format!("{ROUTE}/999999"), Some(&patch), Some(admin), None)?;
    r.send("setup-hook-deleted", "DELETE", &format!("{ROUTE}/{}", ids[1]), None, Some(admin), None)?;
    r.send("setup-hook-delete-missing", "DELETE", &format!("{ROUTE}/999999"), None, Some(admin), None)?;
    Ok(())
}
fn no_secret(value: &Value) -> bool {
    match value {
        Value::Object(fields) => !fields.contains_key("secret") && fields.values().all(no_secret),
        Value::Array(values) => values.iter().all(no_secret),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => true,
    }
}
pub(crate) fn facts(f: &Fixture, out: &Path) -> Result<Vec<Fact>> {
    let made = case(out, "setup-hook-made")?;
    let twin = case(out, "setup-hook-made-twin")?;
    let other = case(out, "setup-hook-made-other")?;
    let listed = case(out, "setup-hook-list")?;
    let patch = case(out, "setup-hook-patched-narrow")?;
    let live = f.get(ROUTE)?;
    let live_text = live.to_string();
    let live = live.as_array().ok_or("hook list absent")?;
    let secrets_hidden = captures_hide_secrets(out)?;
    let mut facts = Vec::new();
    for (name, holds) in [
        ("setup-hook-for-a-loopback-address-is-made", made.status == 201),
        (
            "setup-hook-lists-the-events-it-was-made-with",
            event_set(&made.body["events"]) == Some(EVENTS.iter().copied().collect())
                && listed.body.as_array().is_some_and(|hooks| {
                    hooks.iter().any(|hook| {
                        hook["id"] == made.body["id"]
                            && event_set(&hook["events"]) == Some(EVENTS.iter().copied().collect())
                    })
                }),
        ),
        ("setup-hook-answers-its-target-in-config-url", made.body["config"]["url"] == "http://127.0.0.1:3000/hooks/1"),
        ("setup-hook-is-json-and-active", made.body["config"]["content_type"] == "json" && made.body["active"] == true),
        (
            "setup-hook-never-shows-its-secret",
            secrets_hidden
                && no_secret(&Value::Array(live.clone()))
                && !live_text.contains(FIRST)
                && !live_text.contains(SECOND),
        ),
        (
            "setup-hook-patch-narrows-events-but-keeps-the-workflow-job-ones",
            patch.status == 200
                && patch.body["events"]
                    == json!([
                        "push",
                        "workflow_job_cancelled",
                        "workflow_job_failure",
                        "workflow_job_skipped",
                        "workflow_job_success"
                    ])
                && patch.body["active"] == false,
        ),
        (
            "setup-hook-patch-leaves-the-orgs-other-hook",
            live.iter().any(|hook| {
                hook["id"] == other.body["id"]
                    && hook["events"] == other.body["events"]
                    && hook["active"] == other.body["active"]
                    && hook["config"]["url"].as_str().is_some_and(|url| url.ends_with("/other"))
            }),
        ),
        ("setup-hook-patch-of-a-missing-id-is-404", case(out, "setup-hook-patch-missing")?.status == 404),
        (
            "setup-hook-listing-pages-by-limit",
            case(out, "setup-hook-list-none")?.body == json!([])
                && case(out, "setup-hook-list-page-1")?.body.as_array().is_some_and(|hooks| hooks.len() == 1)
                && case(out, "setup-hook-list-page-4")?.body == json!([]),
        ),
        (
            "setup-hook-of-one-url-can-be-made-twice",
            twin.status == 201
                && twin.body["id"] != made.body["id"]
                && twin.body["config"]["url"] == made.body["config"]["url"]
                && listed.body.as_array().is_some_and(|hooks| {
                    hooks.len() == 3
                        && hooks.iter().any(|hook| hook["id"] == twin.body["id"])
                        && hooks.iter().any(|hook| hook["id"] == made.body["id"])
                }),
        ),
        (
            "setup-hook-delete-is-204-and-leaves-the-orgs-other-hooks",
            case(out, "setup-hook-deleted")?.status == 204
                && live.len() == 2
                && live.iter().any(|hook| hook["id"] == made.body["id"])
                && live.iter().any(|hook| hook["id"] == other.body["id"])
                && live.iter().all(|hook| hook["id"] != twin.body["id"]),
        ),
        ("setup-hook-delete-of-a-missing-id-is-404", case(out, "setup-hook-delete-missing")?.status == 404),
    ] {
        add(&mut facts, name, holds, "captured hook or final live state differs");
    }
    Ok(facts)
}

fn event_set(value: &Value) -> Option<BTreeSet<&str>> {
    value.as_array().map(|items| items.iter().filter_map(Value::as_str).collect())
}

fn captures_hide_secrets(out: &Path) -> Result<bool> {
    [
        "list-none",
        "made",
        "made-twin",
        "made-other",
        "list",
        "list-page-1",
        "list-page-4",
        "patched",
        "patched-narrow",
        "patch-missing",
        "deleted",
        "delete-missing",
    ]
    .iter()
    .try_fold(true, |holds, name| -> Result<bool> {
        Ok(holds && no_secret(&case(out, &format!("setup-hook-{name}"))?.body))
    })
}
