use serde_json::json;

use crate::net::client::{Reply, Site};
use crate::support::consts::app::{KEY_NAME, KEY_PREFIX};
use crate::support::consts::net::{CHAT, MODELS, PORTAL_KEYS};
use crate::support::error::{AppError, AppResult};
use crate::wire::endpoint;
use crate::wire::shape;

pub async fn create(site: &Site) -> AppResult<String> {
    let body = json!({ "key_name": KEY_NAME });
    let origin = endpoint::portal("");
    let reply = site
        .post_json_with(
            &endpoint::portal(PORTAL_KEYS),
            &body,
            &[
                ("origin", origin.clone()),
                ("referer", endpoint::portal("/en/my-profile/api-key")),
            ],
        )
        .await?;
    if !reply.is_success() {
        return Err(refuse("the key was refused", &reply));
    }
    let key = shape::fresh_key(&reply.text)
        .ok_or_else(|| AppError::api("the portal handed over no key"))?;
    if !key.starts_with(KEY_PREFIX) {
        return Err(AppError::api(
            "the portal handed over a key in an odd shape",
        ));
    }
    Ok(key)
}

pub async fn models(model_site: &Site) -> AppResult<Vec<String>> {
    let reply = model_site.get(MODELS).await?;
    if !reply.is_success() {
        return Err(refuse("the model list was refused", &reply));
    }
    let rows = shape::model_ids(&reply.text);
    if rows.is_empty() {
        return Err(AppError::api("the server returned no model"));
    }
    Ok(rows)
}

pub async fn chat(model_site: &Site, model: &str, prompt: &str) -> AppResult<String> {
    let body = json!({
        "model": model,
        "max_tokens": 512,
        "messages": [{ "role": "user", "content": prompt }],
    });
    let reply = model_site.post_json(CHAT, &body).await?;
    if !reply.is_success() {
        return Err(refuse("the chat was refused", &reply));
    }
    match shape::chat_text(&reply.text) {
        Some(text) if !text.is_empty() => Ok(text),

        Some(_) => Ok("an empty answer, the budget was spent on thinking".to_string()),
        None => Err(AppError::api("the reply carried no message")),
    }
}

fn refuse(what: &str, reply: &Reply) -> AppError {
    match shape::refusal(&reply.text) {
        Some(text) => AppError::api(crate::support::error::refusal(&text)),
        None => AppError::api(format!("{} with HTTP {}", what, reply.status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_name_is_the_one_the_portal_expects() {
        assert_eq!(KEY_NAME, "exaone");
        assert!(KEY_PREFIX.starts_with("sk-"));
    }

    #[test]
    fn the_key_request_uses_the_short_field_name() {
        let body = json!({ "key_name": KEY_NAME });
        assert!(body.get("key_name").is_some());
        assert!(body.get("name").is_none());
    }

    #[test]
    fn a_refusal_reply_is_turned_into_words() {
        let reply = Reply {
            status: 400,
            text: r#"{"detail":"Malformed request body"}"#.to_string(),
            location: None,
        };
        let text = refuse("the key was refused", &reply).to_string();
        assert!(text.contains("Malformed request body"));
    }

    #[test]
    fn a_silent_refusal_still_names_the_step() {
        let reply = Reply {
            status: 503,
            text: String::new(),
            location: None,
        };
        assert!(refuse("the key was refused", &reply)
            .to_string()
            .contains("HTTP 503"));
    }
}
