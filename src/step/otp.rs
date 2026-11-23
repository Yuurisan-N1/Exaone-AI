use serde_json::json;

use crate::net::client::Site;
use crate::support::consts::net::{OTP_SEND, OTP_VERIFY};
use crate::support::error::{AppError, AppResult};
use crate::wire::shape;

pub async fn send(site: &Site, challenge_id: &str, email: &str) -> AppResult<()> {
    let body = json!({
        "challengeId": challenge_id,
        "purpose": "SIGNUP",
        "email": email,
    });
    let reply = site.post_json(OTP_SEND, &body).await?;
    if !reply.is_success() {
        return Err(AppError::api(format!(
            "the code could not be requested with HTTP {}",
            reply.status
        )));
    }
    Ok(())
}

pub async fn verify(site: &Site, challenge_id: &str, email: &str, code: &str) -> AppResult<()> {
    if code.chars().count() != 6 {
        return Err(AppError::config("the code is not six digits long"));
    }
    let body = json!({
        "challengeId": challenge_id,
        "purpose": "SIGNUP",
        "email": email,
        "code": code,
    });
    let reply = site.post_json(OTP_VERIFY, &body).await?;
    if !reply.is_success() {
        let reason =
            shape::refusal(&reply.text).unwrap_or_else(|| format!("HTTP {}", reply.status));
        return Err(AppError::api(crate::support::error::refusal(&reason)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_code_is_refused_before_the_request() {
        let site = Site::open(crate::support::consts::net::BASE_URL, None).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let outcome = runtime.block_on(verify(&site, "c", "a@b.test", "123"));
        let text = outcome.unwrap_err().to_string();
        assert!(text.contains("six digits"));
    }

    #[test]
    fn the_send_body_names_the_signup_purpose() {
        let body = json!({ "purpose": "SIGNUP" });
        assert_eq!(body["purpose"], "SIGNUP");
    }
}
