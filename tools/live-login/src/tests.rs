use super::*;
use skein_lib::Writer;
fn jwt(exp: u64) -> String {
    let payload = format!("{{\"exp\":{exp}}}");
    let mut out = Writer::new(
        usize::try_from(
            base64::encoded_len(Alphabet::Url, u32::try_from(payload.len()).expect("small payload"))
                .expect("base64 length"),
        )
        .expect("small length"),
    );
    base64::encode(Alphabet::Url, payload.as_bytes(), &mut out).expect("JWT payload");
    format!("header.{}.signature", String::from_utf8(out.finish().into_vec()).expect("base64 text"))
}
fn auth(token: &str, account: &str) -> String {
    json!({"tokens":{"access_token":token,"account_id":account}}).to_string()
}
#[test]
fn a_token_expiring_within_twenty_minutes_is_refused() {
    let now = 1_800_000_000_u64;
    for (offset, expected) in [(1199, false), (1200, false), (1201, true)] {
        let result = borrowed(auth(&jwt(now + offset), "acct-1").as_bytes(), now);
        assert_eq!(result.is_ok(), expected, "expiry offset {offset}");
        if !expected {
            assert_eq!(
                result,
                Err("live-login: the access token expires within 20 minutes".to_owned()),
                "offset {offset}"
            );
        }
    }
}
#[test]
fn the_account_file_holds_three_members_and_nothing_else() {
    let value = borrowed(auth(&jwt(4_102_444_800), "acct-1").as_bytes(), 1_800_000_000).expect("valid account");
    let emitted = account(&value);
    assert_eq!(emitted.as_object().expect("account").len(), 3);
    assert_eq!(emitted["access_token"], jwt(4_102_444_800));
    assert_eq!(emitted["expires_at"], 4_102_444_800_u64);
    assert_eq!(emitted["account_id"], "acct-1");
}
#[test]
fn the_small_codex_model_is_read_from_its_table() {
    for (text, expected) in [
        ("[small.anthropic]\nmodel = \"other\"\n[small.codex]\nmodel = \"codex-small\"\n", Some("codex-small")),
        ("[working.codex]\nmodel = \"large\"\n", None),
        ("[small.codex]\nmodel = \"\"\n", None),
    ] {
        assert_eq!(small_codex(text).as_deref(), expected, "model table {text:?}");
    }
}

#[test]
fn the_status_line_names_utc_expiry_renewal_and_the_live_tier() {
    for (expires_at, utc) in [
        (0, "1970-01-01 00:00Z"),
        (951_827_696, "2000-02-29 12:34Z"),
        (1_792_577_099, "2026-10-21 10:04Z"),
        (4_107_542_399, "2100-02-28 23:59Z"),
        (4_107_542_400, "2100-03-01 00:00Z"),
    ] {
        assert_eq!(
            status_line("gpt-6-luna", expires_at),
            format!(
                "live-login: model gpt-6-luna, token valid until {utc}; renew: codex login, then live-login; feeds the live tier only"
            ),
            "expiry {expires_at}"
        );
    }
}
