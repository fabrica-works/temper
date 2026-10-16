use crate::{
    Result, failure,
    fixture::{Fixture, R, encode},
};
use serde_json::{Value, json};
use std::{fs, path::Path, time::Duration};
use temper_throwaway_forgejo::{Forgejo, Outgoing, Settings};
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Holds,
    Fails(String),
    Unchecked(String),
}
#[derive(Debug)]
pub struct Fact {
    pub name: &'static str,
    pub outcome: Outcome,
}
impl Fact {
    #[must_use]
    pub fn line(&self) -> String {
        match &self.outcome {
            Outcome::Holds => format!("holds {}\n", self.name),
            Outcome::Fails(why) => format!("fails {}: {}\n", self.name, why.replace(['\r', '\n'], " ")),
            Outcome::Unchecked(why) => format!("unchecked {}: {}\n", self.name, why.replace(['\r', '\n'], " ")),
        }
    }
}
/// # Errors
/// Returns a file write failure.
pub fn write(out: &Path, facts: &[Fact]) -> Result<()> {
    fs::write(out.join("facts.txt"), facts.iter().map(Fact::line).collect::<String>())?;
    Ok(())
}
pub fn add(facts: &mut Vec<Fact>, name: &'static str, holds: bool, seen: &str) {
    facts.push(Fact { name, outcome: if holds { Outcome::Holds } else { Outcome::Fails(seen.into()) } });
}
struct Case {
    status: u16,
    body: Value,
    raw: Vec<u8>,
    head: String,
}
fn case(out: &Path, name: &str) -> Result<Case> {
    let bytes = fs::read(out.join("transcripts").join(name).join("response"))?;
    let end = bytes.windows(4).position(|w| w == b"\r\n\r\n").ok_or("case head missing")?;
    let head = std::str::from_utf8(&bytes[..end])?.to_owned();
    let raw = bytes[end + 4..].to_vec();
    let status = head.split_whitespace().nth(1).ok_or("case status missing")?.parse()?;
    Ok(Case { status, body: serde_json::from_slice(&raw).unwrap_or(Value::Null), raw, head })
}
#[must_use]
pub fn matches(exchange: &temper_throwaway_forgejo::Exchange, status: u16, message: &str) -> bool {
    exchange.status == status
        && serde_json::from_slice::<Value>(&exchange.body).is_ok_and(|body| body["message"] == message)
}
/// # Errors
/// Returns an exchange, provisioning or transcript-read failure.
pub fn evaluate(forgejo: &Forgejo, f: &Fixture, out: &Path) -> Result<Vec<Fact>> {
    let mut facts = Vec::new();
    initial(forgejo, f, out, &mut facts)?;
    listing(f, out, &mut facts)?;
    let changed = updated(f, &mut facts)?;
    bodies(f, out, &changed, &mut facts)?;
    pulls(f, out, &mut facts)?;
    orders(f, out, &mut facts)?;
    errors(out, &mut facts)?;
    for (name, token) in [
        ("effective-protection-name-for-a-writer", &f.writer.token),
        ("effective-protection-name-for-an-admin", &f.owner.token),
    ] {
        let row = crate::json(&f.exchange("GET", &format!("{R}/branches/main"), None, Some(token), None)?)?;
        add(
            &mut facts,
            name,
            row["effective_branch_protection_name"] == "main",
            &format!("effective_branch_protection_name={}", row["effective_branch_protection_name"]),
        );
    }
    let rate = fs::read_dir(out.join("transcripts"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| fs::read(e.path().join("response")))
        .collect::<std::io::Result<Vec<_>>>()?;
    add(
        &mut facts,
        "no-rate-limit-headers",
        rate.iter().all(|bytes| {
            !String::from_utf8_lossy(bytes).lines().any(|line| line.to_ascii_lowercase().starts_with("x-ratelimit-"))
        }),
        "X-RateLimit header present",
    );
    Ok(facts)
}
fn initial(forgejo: &Forgejo, f: &Fixture, out: &Path, facts: &mut Vec<Fact>) -> Result<()> {
    let mut settings = Settings::small();
    settings.max_response_items = None;
    settings.actions = false;
    let instance = forgejo.start(&f.instance.scratch().join("default-api"), &settings).map_err(|e| failure(&e))?;
    let defaults =
        crate::json(&instance.exchange(&Outgoing::get("/api/v1/settings/api", None)).map_err(|e| failure(&e))?)?;
    add(
        facts,
        "api-default-page-cap-50-and-30",
        defaults["max_response_items"] == 50 && defaults["default_paging_num"] == 30,
        &defaults.to_string(),
    );
    instance.stop().map_err(|e| failure(&e))?;
    for (name, recorded, status, message) in [
        ("pull-for-accepts-encoded-slash", "pull-for-encoded-head", 200, ""),
        ("update-conflict-is-409-merge-failed", "update-conflict", 409, "merge failed because of conflict"),
        ("merge-old-head-is-409-head-out-of-date", "merge-stale-head", 409, "head out of date"),
        ("merge-answers-200-with-no-body", "merge-pull", 200, ""),
    ] {
        let c = case(out, recorded)?;
        add(
            facts,
            name,
            c.status == status
                && (message.is_empty() || c.body["message"] == message)
                && (recorded != "merge-pull" || c.raw.is_empty()),
            &format!("HTTP {}: {}", c.status, c.body["message"]),
        );
    }
    let first = case(out, "pull-files-page-1")?;
    let second = case(out, "pull-files-page-2")?;
    add(
        facts,
        "pull-files-page-with-x-hasmore",
        first.body.as_array().is_some_and(|a| a.len() == 2)
            && second.body.as_array().is_some_and(|a| a.len() == 1)
            && first.head.to_ascii_lowercase().contains("x-hasmore: true")
            && second.head.to_ascii_lowercase().contains("x-hasmore: false"),
        &format!("{}; {}", first.head, second.head),
    );
    let compare = f.get(&format!("{R}/compare/{}...{}?page=1&limit=1", f.main, f.head))?;
    add(
        facts,
        "compare-ignores-page-and-limit",
        compare["files"].as_array().is_some_and(|a| a.len() == 3)
            && compare["commits"].as_array().is_some_and(|a| a.len() == 3),
        &format!(
            "files={}, commits={}",
            compare["files"].as_array().map_or(0, Vec::len),
            compare["commits"].as_array().map_or(0, Vec::len)
        ),
    );
    let made = case(out, "create-branch")?;
    let pull = case(out, "pull")?;
    add(
        facts,
        "create-branch-at-full-commit-id",
        made.status == 201 && made.body["commit"]["id"] == pull.body["head"]["sha"],
        &made.body["commit"]["id"].to_string(),
    );
    add(
        facts,
        "writer-protection-reads-are-403",
        case(out, "writer-branch-protection")?.status == 403 && case(out, "writer-branch-protections")?.status == 403,
        "writer protection read was not 403",
    );
    Ok(())
}
fn listing(f: &Fixture, out: &Path, facts: &mut Vec<Fact>) -> Result<()> {
    let mut rows = Vec::new();
    for page in 1..20 {
        let read = f.get(&format!(
            "{R}/issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=leastupdate&page={page}&limit=2"
        ))?;
        let page = read.as_array().ok_or("issue page missing")?;
        rows.extend_from_slice(page);
        if page.len() < 2 {
            break;
        }
    }
    add(
        facts,
        "leastupdate-orders-least-recently-updated-first",
        rows.windows(2).all(|r| r[0]["updated_at"].as_str() <= r[1]["updated_at"].as_str()),
        &rows.iter().map(|r| r["updated_at"].to_string()).collect::<Vec<_>>().join(","),
    );
    add(
        facts,
        "leastupdate-ties-ordered-by-number",
        rows.windows(2)
            .all(|r| r[0]["updated_at"] != r[1]["updated_at"] || r[0]["number"].as_u64() < r[1]["number"].as_u64()),
        "equal timestamps did not sort by number",
    );
    let at = utc(rows[0]["updated_at"].as_str().ok_or("updated time missing")?)?;
    let since = f.get(&format!("{R}/issues?state=all&since={}&sort=leastupdate&page=1&limit=2", encode(&at)))?;
    add(
        facts,
        "since-is-inclusive-at-the-second",
        since.as_array().is_some_and(|a| a.iter().any(|r| r["number"] == rows[0]["number"])),
        &since.to_string(),
    );
    let far = case(out, "issues-since-2554")?;
    add(
        facts,
        "since-in-2554-answers-an-empty-page-with-date",
        far.status == 200 && far.body == json!([]) && far.head.lines().any(|l| l.starts_with("Date:")),
        &format!("HTTP {} {}", far.status, far.body),
    );
    Ok(())
}
fn utc(time: &str) -> Result<String> {
    let mut year: u32 = time[..4].parse()?;
    let mut month: usize = time[5..7].parse()?;
    let mut day: u32 = time[8..10].parse()?;
    let hour: u32 = time[11..13].parse()?;
    let minute: u32 = time[14..16].parse()?;
    let mut minutes = hour * 60 + minute;
    if minutes < 330 {
        minutes += 1440;
        if day == 1 {
            if month == 1 {
                year -= 1;
                month = 12;
            } else {
                month -= 1;
            }
            day = [
                31,
                if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) { 29 } else { 28 },
                31,
                30,
                31,
                30,
                31,
                31,
                30,
                31,
                30,
                31,
            ][month - 1];
        } else {
            day -= 1;
        }
    }
    minutes -= 330;
    Ok(format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{}Z", minutes / 60, minutes % 60, &time[17..19]))
}
fn pause() {
    std::thread::sleep(Duration::from_millis(1100));
}
fn updated(f: &Fixture, facts: &mut Vec<Fact>) -> Result<(Value, Value)> {
    let issue = f.api("POST", &format!("{R}/issues"), &json!({"title":"time facts","body":"initial"}))?["number"]
        .as_u64()
        .ok_or("time issue missing")?;
    let route = format!("{R}/issues/{issue}");
    let before = f.get(&route)?;
    pause();
    let original = f.api("POST", &format!("{route}/comments"), &json!({"body":"original"}))?;
    let posted = f.get(&route)?;
    add(
        facts,
        "updated-moves-on-comment",
        posted["updated_at"] != before["updated_at"],
        &posted["updated_at"].to_string(),
    );
    pause();
    let comment = format!("{R}/issues/comments/{}", original["id"]);
    let edited = f.api("PATCH", &comment, &json!({"body":"edited"}))?;
    add(
        facts,
        "updated-stays-on-comment-edit",
        f.get(&route)?["updated_at"] == posted["updated_at"],
        &f.get(&route)?["updated_at"].to_string(),
    );
    pause();
    f.api("DELETE", &comment, &Value::Null)?;
    add(
        facts,
        "updated-stays-on-comment-delete",
        f.get(&route)?["updated_at"] == posted["updated_at"],
        &f.get(&route)?["updated_at"].to_string(),
    );
    f.branch(R, "facts-base", "landing")?;
    f.branch(R, "facts-head", "facts-base")?;
    f.put(R, "facts-head", "facts.txt", "first\n")?;
    let pull = f.pull(R, "facts-head", "facts-base")?["number"].as_u64().ok_or("facts pull missing")?;
    let route = format!("{R}/issues/{pull}");
    for (name, operation) in [
        ("updated-moves-on-review", 0),
        ("updated-moves-on-head-push", 1),
        ("updated-stays-on-head-status", 2),
        ("updated-stays-on-base-push", 3),
        ("updated-moves-on-close", 4),
        ("updated-moves-on-reopen", 5),
    ] {
        let before = f.get(&route)?;
        pause();
        match operation {
            0 => {
                f.api("POST", &format!("{R}/pulls/{pull}/reviews"), &json!({"event":"COMMENT","body":"time review"}))?;
            }
            1 => {
                f.put(R, "facts-head", "facts-new.txt", "new head\n")?;
            }
            2 => {
                let head = f.get(&format!("{R}/branches/facts-head"))?["commit"]["id"]
                    .as_str()
                    .ok_or("fact head missing")?
                    .to_owned();
                f.api("POST", &format!("{R}/statuses/{head}"), &json!({"state":"success","context":"time/test"}))?;
            }
            3 => {
                f.put(R, "facts-base", "facts-base.txt", "new base\n")?;
            }
            4 => {
                f.api("PATCH", &route, &json!({"state":"closed"}))?;
            }
            5 => {
                f.api("PATCH", &route, &json!({"state":"open"}))?;
            }
            _ => return Err("unknown time operation".into()),
        }
        let mut after = f.get(&route)?;
        // Head pushes update the issue through Forgejo's background queue.
        for _ in 0..100 {
            if operation != 1 || after["updated_at"] != before["updated_at"] {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
            after = f.get(&route)?;
        }
        add(
            facts,
            name,
            (after["updated_at"] != before["updated_at"]) == (operation != 2 && operation != 3),
            &format!("{} -> {}", before["updated_at"], after["updated_at"]),
        );
    }
    Ok((original, edited))
}
fn bodies(f: &Fixture, out: &Path, changed: &(Value, Value), facts: &mut Vec<Fact>) -> Result<()> {
    let comment = case(out, "post-comment")?;
    let review = case(out, "post-review-approved")?;
    add(
        facts,
        "unedited-comment-updated-equals-created",
        comment.body["updated_at"] == comment.body["created_at"],
        &comment.body["updated_at"].to_string(),
    );
    add(
        facts,
        "unedited-review-updated-equals-submitted",
        review.body["updated_at"] == review.body["submitted_at"],
        &review.body["updated_at"].to_string(),
    );
    add(
        facts,
        "edit-moves-comment-updated",
        changed.0["updated_at"] != changed.1["updated_at"],
        &changed.1["updated_at"].to_string(),
    );
    add(
        facts,
        "marker-line-feed-kept",
        comment.body["body"] == "<!-- temper:key 01000002 -->\ncomment text",
        &comment.body["body"].to_string(),
    );
    for (name, size) in [("body-of-65536-bytes-kept-whole", 65536), ("body-of-1048576-bytes-kept-whole", 1_048_576)] {
        let text = "x".repeat(size);
        let made = f.api("POST", &format!("{R}/issues"), &json!({"title":"body fact","body":text}))?;
        let read = f.get(&format!("{R}/issues/{}", made["number"]))?;
        add(
            facts,
            name,
            read["body"].as_str() == Some(text.as_str()),
            &format!("{} bytes", read["body"].as_str().unwrap_or_default().len()),
        );
    }
    Ok(())
}
fn pulls(f: &Fixture, out: &Path, facts: &mut Vec<Fact>) -> Result<()> {
    let edited = f.api(
        "PATCH",
        &format!("{R}/issues/{}", f.feature),
        &json!({"title":"pull edited","body":"pull edited body"}),
    )?;
    add(
        facts,
        "patch-issue-edits-a-pull-request",
        edited["title"] == "pull edited" && f.get(&format!("{R}/pulls/{}", f.feature))?["body"] == "pull edited body",
        &edited["title"].to_string(),
    );
    f.branch(R, "repeat-head", "landing")?;
    f.put(R, "repeat-head", "repeat.txt", "repeat\n")?;
    let first = f.pull(R, "repeat-head", "landing")?;
    f.api("PATCH", &format!("{R}/issues/{}", first["number"]), &json!({"state":"closed"}))?;
    let newest = f.pull(R, "repeat-head", "landing")?;
    let found = f.get(&format!("{R}/pulls/landing/repeat-head"))?;
    add(
        facts,
        "pull-for-names-the-newest-of-several",
        found["number"] == newest["number"],
        &found["number"].to_string(),
    );
    add(
        facts,
        "branch-path-takes-encoded-slash",
        case(out, "branch-with-slash")?.status == 200,
        "branch with encoded slash did not answer 200",
    );
    let pull = f.get(&format!("{R}/pulls/{}", f.feature))?;
    let base = f.get(&format!("{R}/branches/main"))?;
    add(
        facts,
        "base-sha-is-the-base-tip",
        pull["base"]["sha"] == base["commit"]["id"],
        &pull["base"]["sha"].to_string(),
    );
    let empty = case(out, "combined-status-empty")?;
    add(
        facts,
        "combined-status-without-status-counts-0",
        empty.body["total_count"] == 0,
        &empty.body["total_count"].to_string(),
    );
    let head = newest["head"]["sha"].as_str().ok_or("status head missing")?;
    for (state, context) in [("skipped", "skip/test"), ("success", "pass/test")] {
        f.api("POST", &format!("{R}/statuses/{head}"), &json!({"state":state,"context":context}))?;
    }
    let combined = f.get(&format!("{R}/commits/{head}/status?page=1&limit=2"))?;
    add(
        facts,
        "skipped-and-success-combine-as-passed",
        ["success", "warning", "skipped"].iter().any(|s| combined["state"] == *s),
        &combined["state"].to_string(),
    );
    Ok(())
}
fn orders(f: &Fixture, out: &Path, facts: &mut Vec<Fact>) -> Result<()> {
    for (name, recorded) in [("comments-come-oldest-first", "issue-comments"), ("reviews-come-oldest-first", "reviews")]
    {
        let read = case(out, recorded)?;
        let rows = read.body.as_array().ok_or("ordered rows missing")?;
        add(
            facts,
            name,
            rows.windows(2).all(|a| a[0]["id"].as_u64() < a[1]["id"].as_u64()),
            &rows.iter().map(|r| r["id"].to_string()).collect::<Vec<_>>().join(","),
        );
    }
    let route = format!("{R}/pulls/{}/reviews", f.feature);
    let response = f.exchange(
        "POST",
        &route,
        Some(b"{\"event\":\"APPROVED\",\"body\":\"dismissed body\"}"),
        Some(&f.writer.token),
        None,
    )?;
    let review = crate::json(&response)?;
    let id = review["id"].as_u64().ok_or("dismissal review missing")?;
    f.api("POST", &format!("{route}/{id}/dismissals"), &json!({"message":"dismiss fact"}))?;
    let dismissed = f.get(&format!("{route}/{id}"))?;
    add(
        facts,
        "dismissed-review-keeps-its-body",
        dismissed["body"] == "dismissed body" && dismissed["dismissed"] == true,
        &format!("{} {}", dismissed["state"], dismissed["body"]),
    );
    let remarks = case(out, "review-comments")?;
    add(
        facts,
        "remark-position-is-its-line",
        remarks.body[0]["position"] == 1,
        &remarks.body[0]["position"].to_string(),
    );
    add(facts, "raw-range-answers-206", case(out, "raw-file-range")?.status == 206, "raw Range did not answer 206");
    Ok(())
}
fn errors(out: &Path, facts: &mut Vec<Fact>) -> Result<()> {
    for (name, recorded, expected) in [
        ("merge-closed-is-405", "merge-closed", 405),
        ("merge-conflict-is-405", "merge-conflict", 405),
        ("merge-protected-is-405", "merge-protected", 405),
        ("merge-style-is-405", "merge-style-refused", 405),
        ("open-pull-exists-is-409", "open-pull-exists", 409),
        ("open-pull-no-changes-is-422", "open-pull-no-changes", 422),
        ("empty-title-is-422", "create-issue-empty-title", 422),
        ("moved-repository-is-a-redirect", "moved-repository", 301),
        ("archived-write-is-423", "archived-write", 423),
        ("delete-protected-branch-is-403", "delete-protected-branch", 403),
        ("delete-missing-branch-is-404", "delete-missing-branch", 404),
        ("create-branch-exists-is-409", "create-branch-exists", 409),
        ("unreadable-repository-by-path-is-404", "stranger-repository-by-path", 404),
        ("unreadable-repository-by-id-is-404", "stranger-repository-by-id", 404),
    ] {
        let read = case(out, recorded)?;
        add(
            facts,
            name,
            if expected == 301 { (300..400).contains(&read.status) } else { read.status == expected },
            &format!("HTTP {}: {}", read.status, read.body["message"]),
        );
    }
    Ok(())
}
/// Replaces only facts whose names begin with `prefix`, preserving every other line.
#[must_use]
pub fn replace_block(text: &str, prefix: &str, facts: &[Fact]) -> String {
    let mut result = String::new();
    let mut replaced = false;
    for line in text.split_inclusive('\n') {
        let mut words = line.split_whitespace();
        let component = matches!(words.next(), Some("holds" | "fails" | "unchecked"))
            && words.next().is_some_and(|name| name.starts_with(prefix));
        if component {
            if !replaced {
                result.extend(facts.iter().map(Fact::line));
                replaced = true;
            }
        } else {
            result.push_str(line);
        }
    }
    if !replaced {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.extend(facts.iter().map(Fact::line));
    }
    result
}

/// # Errors
/// Returns a transcript-read failure.
pub fn flags(out: &Path) -> Result<Vec<Fact>> {
    let plain = case(out, "compare")?.body;
    let reverse = case(out, "compare-reverse")?.body;
    let verification = case(out, "compare-verification-false")?;
    let files = case(out, "compare-files-false")?;
    let backwards = case(out, "compare-reverse-files-false")?.body;
    let shas = |body: &Value| {
        body["commits"].as_array().map(|rows| rows.iter().map(|row| row["sha"].clone()).collect::<Vec<_>>())
    };
    let names = |body: &Value| {
        body["files"].as_array().map(|rows| rows.iter().map(|row| row["filename"].clone()).collect::<Vec<_>>())
    };
    let commits = &files.body["commits"];
    let no_files = files.status == 200
        && commits.as_array().is_some_and(|rows| !rows.is_empty() && rows.iter().all(|row| row["files"].is_null()));
    let empty = files.body["files"] == json!([]) && backwards["files"] == json!([]);
    let verified = verification.status == 200
        && verification.body["total_commits"] == plain["total_commits"]
        && shas(&verification.body) == shas(&plain)
        && names(&verification.body) == names(&plain);
    let kept = files.body["total_commits"] == plain["total_commits"]
        && shas(&files.body) == shas(&plain)
        && backwards["total_commits"] == reverse["total_commits"];
    let seen = format!("Forward {}, reverse {}, verification {}", files.body, backwards, verification.body);
    let mut facts = Vec::new();
    for (name, holds) in [
        ("compare-files-false-lists-no-commits-files", no_files),
        ("compare-files-false-keeps-its-own-files-empty", empty),
        ("compare-verification-false-changes-nothing-kept", verified),
        ("compare-files-false-keeps-total-commits-and-shas", kept),
    ] {
        add(&mut facts, name, holds, &seen);
    }
    Ok(facts)
}
