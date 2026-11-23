use std::sync::atomic::{AtomicUsize, Ordering};

use crate::net::egress::Proxy;
use crate::step::signup::fresh_identity;
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::{clean_text, clock, gap, lg, ly, mask_email, mask_key, num};
use crate::wire::endpoint;
use crate::wire::envelope;
use crate::wire::shape;

pub fn execute() -> AppResult<()> {
    ly("The self test runs without touching the network");

    let identity = fresh_identity();
    check(
        identity.password.chars().count() >= 12,
        "A password clears the minimum length",
    )?;
    check(
        identity.nickname.starts_with("exuser"),
        "A nickname carries the prefix",
    )?;
    check(
        identity.device.agent.contains("Mozilla/5.0"),
        "A device looks like a browser",
    )?;
    check(
        !mask_email("ex-abcdef123456@euploading.com").contains("abcdef123456"),
        "A mailbox address is masked at the head",
    )?;

    check(
        shape::challenge(r#"{"challengeId":"c","csrfToken":"s","publicKey":"k"}"#).is_some(),
        "A challenge is read from the init reply",
    )?;
    check(
        shape::terms(r#"{"pendingSessionId":"p","csrfToken":"s","terms":[{"id":"37"}]}"#)
            .map(|got| got.ids == vec!["37"])
            .unwrap_or(false),
        "The consent list is read with its ids",
    )?;
    check(
        shape::one_time_code("code 524177").as_deref() == Some("524177"),
        "The six digit code is taken from the mail",
    )?;
    check(
        shape::one_time_code("2026-11-20 17:53").is_none(),
        "A date is not read as a code",
    )?;
    check(
        shape::fresh_key(r#"{"api_key":{"api_key":"sk-abc"}}"#).as_deref() == Some("sk-abc"),
        "The key is read from the fresh reply",
    )?;
    check(
        shape::fresh_key(r#"{"api_keys":[{"masked_key":"sk-****sveI"}]}"#).is_none(),
        "A stored masked key is not read as a key",
    )?;
    check(
        shape::refusal(r#"{"detail":"Malformed request body"}"#).is_some(),
        "A refusal is read from the detail field",
    )?;
    check(
        shape::model_ids(r#"{"data":[{"id":"k-exaone_v2.5"}]}"#).len() == 1,
        "A model list is read from the data array",
    )?;
    check(
        shape::chat_text(r#"{"choices":[{"message":{"content":" ok "}}]}"#).as_deref()
            == Some("ok"),
        "A chat reply is read from the first choice",
    )?;

    check(
        envelope::seal("not-a-key", &serde_json::json!({})).is_err(),
        "A broken public key is refused in words",
    )?;
    check(
        envelope::seal_raw("not-a-key", b"x").is_err(),
        "A bare secret is refused too",
    )?;

    check(
        endpoint::id("") == "https://id.exaone.ai"
            && endpoint::portal("") == "https://docs.exaone.ai"
            && endpoint::api("") == "https://api.exaone.ai",
        "Each host keeps its own name",
    )?;
    check(
        crate::step::portal::login_params(
            "https://id.exaone.ai/login?client_id=abc&oauth_login_challenge=xyz",
        )
        .map(|(c, o)| c == "abc" && o == "xyz")
        .unwrap_or(false),
        "The portal login parameters are read off the bounce",
    )?;

    let proxy = Proxy::parse("socks5://user:secret@192.0.2.10:10043")?;
    check(
        proxy.masked() == "http://user:pass@192*****10:10043",
        "A proxy is masked before it is printed",
    )?;
    check(
        proxy.url() == "socks5://user:secret@192.0.2.10:10043",
        "A proxy keeps its credentials in the url",
    )?;

    check(
        mask_key("ab-cdefgZZZZhijklmnopq") == "ab-cdefg****hijklmnopq",
        "A key keeps a head and a tail only",
    )?;
    check(
        clean_text("###", "fallback") == "fallback",
        "Empty text falls back",
    )?;
    check(
        clock(3_723) == "01:02:03",
        "A countdown is written as a clock",
    )?;
    check(num(1.23456) == "1.2346", "A number keeps four places")?;

    let passed = PASSED.load(Ordering::Relaxed);
    let failed = FAILED.load(Ordering::Relaxed);

    gap();
    if failed > 0 {
        return Err(AppError::api(format!(
            "{} of {} checks failed",
            failed,
            passed + failed
        )));
    }
    lg(&format!("All {} checks passed", passed));
    Ok(())
}

static PASSED: AtomicUsize = AtomicUsize::new(0);
static FAILED: AtomicUsize = AtomicUsize::new(0);

fn check(flag: bool, msg: &str) -> AppResult<()> {
    if flag {
        PASSED.fetch_add(1, Ordering::Relaxed);
        lg(&format!("The check passed for {}", msg));
        Ok(())
    } else {
        FAILED.fetch_add(1, Ordering::Relaxed);
        Err(AppError::api(msg))
    }
}
