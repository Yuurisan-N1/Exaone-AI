use serde_json::json;

use crate::net::client::Site;
use crate::support::consts::app::{COUNTRY, NICKNAME_PREFIX, PASSWORD_MIN_LEN};
use crate::support::consts::net::{SIGNUP_CHECK, SIGNUP_DONE, SIGNUP_INIT, SIGNUP_SUBMIT};
use crate::support::error::{AppError, AppResult};
use crate::wire::envelope::seal;
use crate::wire::shape::{self, Challenge, Device};

pub struct Identity {
    pub password: String,
    pub nickname: String,
    pub device: Device,
}

pub fn fresh_identity() -> Identity {
    Identity {
        password: format!("Ex-{}-{}!", hex(4), hex(4)),
        nickname: format!("{}{}", NICKNAME_PREFIX, digits(4)),
        device: Device::fresh(),
    }
}

pub async fn begin(site: &Site, email: &str) -> AppResult<Challenge> {
    let reply = site.post_json(SIGNUP_INIT, &json!({})).await?;
    if !reply.is_success() {
        if reply.status == 302 || reply.status == 429 || reply.status == 403 {
            return Err(AppError::blocked(
                "the exit address is refused in this region",
            ));
        }
        return Err(refuse("the signup could not be started", &reply));
    }
    let challenge = shape::challenge(&reply.text)
        .ok_or_else(|| AppError::api("the server handed over no signup challenge"))?;

    let body = json!({
        "challengeId": challenge.challenge_id,
        "csrfToken": challenge.csrf_token,
        "encryptedEmail": seal(&challenge.public_key, &json!({ "email": email }))?,
    });
    let reply = site.post_json(SIGNUP_CHECK, &body).await?;
    if !reply.is_success() {
        return Err(refuse("the mailbox was refused", &reply));
    }
    Ok(challenge)
}

pub async fn submit(
    site: &Site,
    challenge: &Challenge,
    email: &str,
    identity: &Identity,
) -> AppResult<()> {
    if identity.password.chars().count() < PASSWORD_MIN_LEN {
        return Err(AppError::config("the generated password is too short"));
    }
    let body = json!({
        "challengeId": challenge.challenge_id,
        "csrfToken": challenge.csrf_token,
        "encryptedCredentials": seal(
            &challenge.public_key,
            &json!({ "email": email, "password": identity.password }),
        )?,
        "nickname": identity.nickname,
        "country": COUNTRY,
        "acceptedTerms": ["REQUIRED_TERMS"],
    });
    let reply = site.post_json(SIGNUP_SUBMIT, &body).await?;
    if !reply.is_success() {
        return Err(refuse("the password was refused", &reply));
    }
    Ok(())
}

pub async fn complete(site: &Site, challenge: &Challenge) -> AppResult<()> {
    let body = json!({
        "challengeId": challenge.challenge_id,
        "csrfToken": challenge.csrf_token,
    });
    let reply = site.post_json(SIGNUP_DONE, &body).await?;
    if !reply.is_success() {
        return Err(refuse("the signup could not be closed", &reply));
    }
    if shape::account_id(&reply.text).is_none() {
        return Err(AppError::api("the server did not close the signup"));
    }
    Ok(())
}

fn refuse(what: &str, reply: &crate::net::client::Reply) -> AppError {
    match shape::refusal(&reply.text) {
        Some(text) => AppError::api(crate::support::error::refusal(&text)),
        None => AppError::api(format!("{} with HTTP {}", what, reply.status)),
    }
}

fn hex(len: usize) -> String {
    use rand::RngCore;
    let mut raw = vec![0u8; len];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    raw.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn digits(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;
    (0..len)
        .map(|_| char::from(b'0' + rng.gen_range(0..10)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_password_has_the_expected_shape() {
        let identity = fresh_identity();
        assert!(identity.password.starts_with("Ex-"));
        assert!(identity.password.ends_with('!'));
        assert!(identity.password.chars().count() >= PASSWORD_MIN_LEN);
        assert_ne!(identity.password, fresh_identity().password);
    }

    #[test]
    fn the_nickname_is_prefixed_and_padded() {
        let identity = fresh_identity();
        assert!(identity.nickname.starts_with(NICKNAME_PREFIX));
        assert_eq!(identity.nickname.chars().count(), NICKNAME_PREFIX.len() + 4);
        assert!(identity
            .nickname
            .chars()
            .skip(NICKNAME_PREFIX.len())
            .all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn two_accounts_never_share_a_device() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..24 {
            seen.insert(fresh_identity().device.agent);
        }
        assert!(seen.len() > 1);
    }

    #[test]
    fn the_digits_helper_stays_numeric() {
        assert_eq!(digits(4).chars().count(), 4);
        assert!(digits(4).chars().all(|c| c.is_ascii_digit()));
    }
}
