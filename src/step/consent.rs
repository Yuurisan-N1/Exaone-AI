use serde_json::{json, Value};

use crate::net::client::Site;
use crate::support::consts::app::TERMS_FIELD;
use crate::support::consts::net::{TERMS_ACCEPT, TERMS_REQUIRED};
use crate::support::error::{AppError, AppResult};
use crate::wire::endpoint;
use crate::wire::shape;

pub async fn accept(site: &Site) -> AppResult<(Vec<String>, Option<String>)> {
    let reply = site.get(TERMS_REQUIRED).await?;
    if !reply.is_success() {
        return Err(AppError::api(format!(
            "the consent list was refused with HTTP {}",
            reply.status
        )));
    }
    let pending = shape::terms(&reply.text)
        .ok_or_else(|| AppError::api("the server listed no consent to give"))?;
    if pending.ids.is_empty() {
        return Err(AppError::api("the server listed no consent to give"));
    }

    let mut body = serde_json::Map::new();
    body.insert("pendingSessionId".to_string(), json!(pending.session_id));
    body.insert("csrfToken".to_string(), json!(pending.csrf_token));
    body.insert(TERMS_FIELD.to_string(), json!(pending.ids));

    let origin = endpoint::id("");
    let reply = site
        .post_json_with(
            TERMS_ACCEPT,
            &Value::Object(body),
            &[
                ("origin", origin.clone()),
                ("referer", format!("{}/", origin)),
            ],
        )
        .await?;
    if !reply.is_success() {
        let reason =
            shape::refusal(&reply.text).unwrap_or_else(|| format!("HTTP {}", reply.status));
        return Err(AppError::api(crate::support::error::refusal(&reason)));
    }
    Ok((pending.ids, shape::redirect_url(&reply.text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_accept_body_uses_the_field_the_form_uses() {
        assert_eq!(TERMS_FIELD, "acceptedTermsIds");
        let mut body = serde_json::Map::new();
        body.insert(TERMS_FIELD.to_string(), json!(["37", "39"]));
        let text = serde_json::to_string(&Value::Object(body)).unwrap();
        assert!(text.contains("\"acceptedTermsIds\":[\"37\",\"39\"]"));
        assert!(!text.contains("\"ids\""));
    }

    #[test]
    fn the_origin_header_is_the_identity_host() {
        assert_eq!(endpoint::id(""), "https://id.exaone.ai");
    }
}
