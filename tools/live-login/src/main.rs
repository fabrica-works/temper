//! Borrow an existing access token without changing its login.
use serde_json::{Value, json};
use skein_lib::base64::{self, Alphabet};
use std::{fs, io::Write, net::ToSocketAddrs, os::unix::fs::OpenOptionsExt, path::Path, process::ExitCode};
#[cfg(test)]
mod tests;
#[derive(Debug, PartialEq)]
#[expect(clippy::struct_field_names, reason = "the account members mirror the handed-in account schema")]
struct Account {
    access_token: String,
    expires_at: u64,
    account_id: String,
}
fn borrowed(auth: &[u8], now: u64) -> Result<Account, String> {
    let invalid = || "live-login: borrowed access token unavailable".to_owned();
    let value: Value = serde_json::from_slice(auth).map_err(|_| invalid())?;
    let tokens = &value["tokens"];
    let text = |name| tokens[name].as_str().filter(|s| !s.is_empty()).map(str::to_owned).ok_or_else(invalid);
    let access_token = text("access_token")?;
    let account_id = text("account_id")?;
    let claim = access_token.split('.').nth(1).ok_or_else(invalid)?;
    let payload = base64::decode(Alphabet::Url, claim.as_bytes(), 65_536).map_err(|_| invalid())?;
    let payload: Value = serde_json::from_slice(&payload).map_err(|_| invalid())?;
    let expires_at = payload["exp"].as_u64().ok_or_else(invalid)?;
    if expires_at <= now.saturating_add(1200) {
        return Err("live-login: the access token expires within 20 minutes".to_owned());
    }
    Ok(Account { access_token, expires_at, account_id })
}
fn account(value: &Account) -> Value {
    json!({"access_token":value.access_token,"expires_at":value.expires_at,"account_id":value.account_id})
}
fn small_codex(text: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[small.codex]";
        } else if inside
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "model"
        {
            return value.trim().strip_prefix('"')?.split('"').next().filter(|s| !s.is_empty()).map(str::to_owned);
        }
    }
    None
}
fn model(models: Option<&Path>) -> Result<String, String> {
    if let Ok(value) = std::env::var("TEMPER_LIVE_MODEL")
        && !value.is_empty()
    {
        return Ok(value);
    }
    models
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| small_codex(&text))
        .ok_or_else(|| "live-login: set TEMPER_LIVE_MODEL or supply the small.codex model table".to_owned())
}
fn private(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "live-login: cannot encode configuration")?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "live-login: cannot create private output")?;
    file.write_all(&bytes).map_err(|_| "live-login: cannot write private output".to_owned())
}
// Gregorian dates from Unix epoch days, using March-based 400-year eras, as
// temper-client-view's chat_clock and skein-http's private put_date do.
// Expiry is in u64 seconds rather than Wall's u64 nanoseconds; days plus
// 719,468 and every calendar intermediate still fit u64.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * march_month + 2) / 5 + 1;
    let month = if march_month < 10 { march_month + 3 } else { march_month - 9 };
    (year + u64::from(month <= 2), month, day)
}
fn status_line(model: &str, expires_at: u64) -> String {
    let (year, month, day) = civil_from_days(expires_at / 86_400);
    let hour = expires_at / 3_600 % 24;
    let minute = expires_at / 60 % 60;
    format!(
        "live-login: model {model}, token valid until {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}Z; renew: codex login, then live-login; feeds the live tier only"
    )
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(args.len() == 2 || args.len() == 4 && args[2] == "--models") {
        return Err("live-login: AUTH_JSON OUT_DIR [--models MODELS_TOML]".to_owned());
    }
    let now = skein_shell::Clock::new().now().wall.as_nanos() / 1_000_000_000;
    let value = borrowed(&fs::read(&args[0]).map_err(|_| "live-login: cannot read borrowed login")?, now)?;
    let model = model(args.get(3).map(Path::new))?;
    let address = ("chatgpt.com", 443)
        .to_socket_addrs()
        .map_err(|_| "live-login: cannot resolve provider")?
        .find(std::net::SocketAddr::is_ipv4)
        .ok_or("live-login: provider has no IPv4 address")?;
    let output = Path::new(&args[1]);
    let account_file = output.join("live-account.json");
    let config = output.join("live-config.json");
    if config.exists() {
        fs::remove_file(&config).map_err(|_| "live-login: cannot replace public configuration")?;
    }
    private(&config, &json!({"account_file":account_file,"model":model,"address":address.to_string()}))?;
    private(&account_file, &account(&value))?;
    println!("{}", status_line(&model, value.expires_at));
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
