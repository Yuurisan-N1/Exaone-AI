use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Challenge {
    pub challenge_id: String,
    pub csrf_token: String,
    pub public_key: String,
}

pub fn challenge(body: &str) -> Option<Challenge> {
    let value: Value = serde_json::from_str(body).ok()?;
    let id = field(&value, "challengeId")?;
    let csrf = field(&value, "csrfToken")?;
    let key = field(&value, "publicKey")?;
    Some(Challenge {
        challenge_id: id,
        csrf_token: csrf,
        public_key: key,
    })
}

pub fn account_id(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    field(&value, "userId")
}

pub struct Terms {
    pub session_id: String,
    pub csrf_token: String,
    pub ids: Vec<String>,
}

pub fn terms(body: &str) -> Option<Terms> {
    let value: Value = serde_json::from_str(body).ok()?;
    let session_id = field(&value, "pendingSessionId")?;
    let csrf_token = field(&value, "csrfToken")?;
    let ids = value
        .get("terms")?
        .as_array()?
        .iter()
        .filter_map(|row| row.get("id"))
        .filter_map(|id| match id {
            Value::String(text) => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
        .collect();
    Some(Terms {
        session_id,
        csrf_token,
        ids,
    })
}

pub fn redirect_url(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    let text = value.get("redirectUrl")?.as_str()?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

pub fn one_time_code(blob: &str) -> Option<String> {
    let digits: Vec<char> = blob.chars().collect();
    let mut run: Vec<char> = Vec::new();
    for c in digits {
        if c.is_ascii_digit() {
            run.push(c);
        } else {
            if run.len() == 6 {
                return Some(run.iter().collect());
            }
            run.clear();
        }
    }
    if run.len() == 6 {
        return Some(run.iter().collect());
    }
    None
}

pub fn fresh_key(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    let text = value.pointer("/api_key/api_key")?.as_str()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(text.trim().to_string())
    }
}

fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    if let Some(list) = value.get(key).and_then(Value::as_array) {
        return list;
    }
    if let Some(list) = value.as_array() {
        return list;
    }
    &[]
}

pub fn message_ids(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    rows(&value, "hydra:member")
        .iter()
        .filter_map(|row| row.get("id").and_then(Value::as_str))
        .map(|id| id.to_string())
        .collect()
}

pub fn domain_list(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    rows(&value, "hydra:member")
        .iter()
        .filter(|row| row.get("isActive").and_then(Value::as_bool).unwrap_or(true))
        .filter_map(|row| row.get("domain").and_then(Value::as_str))
        .map(|domain| domain.to_string())
        .collect()
}

fn joined(row: &Value, keys: &[&str]) -> String {
    let mut out = String::new();
    for key in keys {
        if let Some(text) = row.get(*key).and_then(Value::as_str) {
            out.push_str(text);
            out.push('\n');
        }
    }
    out
}

pub fn mail_tm_bodies(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    let text = value.get("text").and_then(Value::as_str).unwrap_or("");
    if !text.is_empty() {
        out.push(text.to_string());
    }
    if let Some(html) = value.get("html").and_then(Value::as_str) {
        if !html.is_empty() {
            out.push(html.to_string());
        }
    }
    out
}

pub fn mail_io_bodies(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    match value.as_array() {
        Some(rows) => rows
            .iter()
            .map(|row| joined(row, &["body_text", "body_html", "subject"]))
            .filter(|blob| !blob.trim().is_empty())
            .collect(),
        None => Vec::new(),
    }
}

pub fn mail_lol_bodies(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    match value.get("emails").and_then(Value::as_array) {
        Some(rows) => rows
            .iter()
            .map(|row| joined(row, &["body", "body_html", "subject"]))
            .filter(|blob| !blob.trim().is_empty())
            .collect(),
        None => Vec::new(),
    }
}

pub fn guerrilla_ids(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    match value.get("list").and_then(Value::as_array) {
        Some(rows) => rows
            .iter()
            .filter_map(|row| row.get("mail_id"))
            .filter_map(|id| match id {
                Value::String(text) => Some(text.clone()),
                Value::Number(number) => Some(number.to_string()),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
    }
}

pub fn guerrilla_body(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    let text = value.get("mail_body")?.as_str()?.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn model_ids(body: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("id").and_then(Value::as_str))
                .filter(|id| !id.trim().is_empty())
                .map(|id| id.to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn chat_text(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    let content = value.pointer("/choices/0/message/content")?.as_str()?;
    Some(content.trim().to_string())
}

pub fn refusal(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    for pointer in ["/detail", "/message", "/error/message", "/reason"] {
        if let Some(text) = value.pointer(pointer).and_then(Value::as_str) {
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }
    }
    None
}

fn field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(|text| text.to_string())
        .filter(|text| !text.is_empty())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub agent: String,
    pub platform: String,
    pub language: String,
}

const AGENTS: &[(&str, &str)] = &[
    (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        "\"Windows\"",
    ),
    (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "\"Windows\"",
    ),
    (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
        "\"Windows\"",
    ),
    (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        "\"macOS\"",
    ),
    (
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
        "\"macOS\"",
    ),
];

const LANGUAGES: &[&str] = &["en-US,en;q=0.9", "en-GB,en;q=0.9", "en-US,en;q=0.8"];

impl Device {
    pub fn fresh() -> Device {
        use rand::Rng;
        let mut rng = rand::rngs::OsRng;
        let (agent, platform) = AGENTS[rng.gen_range(0..AGENTS.len())];
        let language = LANGUAGES[rng.gen_range(0..LANGUAGES.len())];
        Device {
            agent: agent.to_string(),
            platform: platform.to_string(),
            language: language.to_string(),
        }
    }

    pub fn plain() -> Device {
        Device {
            agent: crate::support::consts::net::BROWSER_AGENT.to_string(),
            platform: "\"Windows\"".to_string(),
            language: "en-US,en;q=0.9".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub password: String,
    pub api_key: String,
    pub created_at: String,
    #[serde(default)]
    pub device: Option<Device>,

    #[serde(default)]
    pub terms: String,

    #[serde(default)]
    pub step_up: String,

    #[serde(default)]
    pub portal_at: String,
    #[serde(default)]
    pub portal_rt: String,
    #[serde(default)]
    pub session_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_challenge_carries_its_three_fields() {
        let body = r#"{"challengeId":"c1","csrfToken":"s1","publicKey":"k1","expiresAt":"x"}"#;
        let got = challenge(body).unwrap();
        assert_eq!(got.challenge_id, "c1");
        assert_eq!(got.csrf_token, "s1");
        assert_eq!(got.public_key, "k1");
        assert!(challenge("{}").is_none());
    }

    #[test]
    fn the_terms_list_reads_numeric_ids() {
        let body = r#"{"pendingSessionId":"p","csrfToken":"s","terms":[{"id":"37"},{"id":39}]}"#;
        let got = terms(body).unwrap();
        assert_eq!(got.ids, vec!["37", "39"]);
        assert_eq!(got.session_id, "p");
    }

    #[test]
    fn the_consent_redirect_is_read_back() {
        let body =
            r#"{"success":true,"redirectUrl":"/oauth2/authorize?oauth_login_challenge=abc"}"#;
        assert_eq!(
            redirect_url(body).as_deref(),
            Some("/oauth2/authorize?oauth_login_challenge=abc")
        );
        assert!(redirect_url(r#"{"success":true}"#).is_none());
    }

    #[test]
    fn the_code_is_never_taken_from_the_msgid() {
        let blob = r#"{"id":"6b00fa56d531ef4b037312c7","msgid":"<010c01a1fbd1d8c5-13b5af9b-44ff-495a-b930-dc27d5f99ef9-000000@ap-northeast-2.amazonses.com>","retentionDate":"2026-11-28T01:56:38+00:00","text":"Verification Code 752587 valid for 30 minutes","html":"<p>752587</p>"}"#;
        let bodies = mail_tm_bodies(blob);
        assert_eq!(bodies.len(), 2);
        assert_eq!(one_time_code(&bodies[0]).as_deref(), Some("752587"));

        assert_eq!(one_time_code(blob).as_deref(), Some("037312"));
        assert_ne!(one_time_code(blob).as_deref(), Some("752587"));
    }

    #[test]
    fn the_code_is_exactly_six_digits() {
        assert_eq!(
            one_time_code("code 524177 thanks").as_deref(),
            Some("524177")
        );
        assert!(one_time_code("code 12345").is_none());
        assert!(one_time_code("2026-11-20 17:53").is_none());
        assert_eq!(one_time_code("ends 998877").as_deref(), Some("998877"));
    }

    #[test]
    fn only_a_fresh_key_is_readable() {
        let fresh = r#"{"result_code":200,"api_key":{"api_key_id":118783,"api_key":"sk-abc"}}"#;
        assert_eq!(fresh_key(fresh).as_deref(), Some("sk-abc"));
        let stored = r#"{"api_keys":[{"api_key_id":1,"masked_key":"sk-****sveI"}]}"#;
        assert!(fresh_key(stored).is_none());
    }

    #[test]
    fn the_account_id_is_read_from_the_completion() {
        assert_eq!(
            account_id(r#"{"userId":"01a1fa49-2eff"}"#).as_deref(),
            Some("01a1fa49-2eff")
        );
        assert!(account_id(r#"{"ok":true}"#).is_none());
    }

    #[test]
    fn the_refusal_is_found_under_any_of_its_names() {
        assert_eq!(
            refusal(r#"{"detail":"Malformed request body"}"#).as_deref(),
            Some("Malformed request body")
        );
        assert_eq!(
            refusal(r#"{"code":"X","message":"Authentication required."}"#).as_deref(),
            Some("Authentication required.")
        );
        assert_eq!(
            refusal(r#"{"error":{"message":"invalid api key format"}}"#).as_deref(),
            Some("invalid api key format")
        );
        assert!(refusal(r#"{"ok":true}"#).is_none());
    }

    #[test]
    fn the_domain_list_skips_inactive_rows() {
        let body = r#"{"hydra:member":[{"domain":"a.test","isActive":true},{"domain":"b.test","isActive":false}]}"#;
        assert_eq!(domain_list(body), vec!["a.test"]);
        assert_eq!(
            domain_list(r#"[{"domain":"euploading.com"}]"#),
            vec!["euploading.com"]
        );
    }

    #[test]
    fn the_message_list_reads_both_shapes() {
        assert_eq!(
            message_ids(r#"{"hydra:member":[{"id":"abc"}]}"#),
            vec!["abc"]
        );
        assert_eq!(message_ids(r#"[{"id":"a"},{"id":"b"}]"#), vec!["a", "b"]);
    }

    #[test]
    fn the_guerrilla_list_reads_numeric_ids() {
        assert_eq!(
            guerrilla_ids(r#"{"list":[{"mail_id":42},{"mail_id":"7"}]}"#),
            vec!["42", "7"]
        );
        assert!(guerrilla_body(r#"{"mail_body":"  "}"#).is_none());
        assert_eq!(
            guerrilla_body(r#"{"mail_body":"code 123456"}"#).as_deref(),
            Some("code 123456")
        );
    }

    #[test]
    fn a_provider_body_joins_its_fields() {
        let body = r#"[{"body_text":"hello","subject":"x"}]"#;
        assert!(mail_io_bodies(body)[0].contains("hello"));
        assert!(mail_lol_bodies(r#"{"emails":[{"body":"hi"}]}"#)[0].contains("hi"));
        assert!(mail_lol_bodies(r#"{}"#).is_empty());
    }

    #[test]
    fn the_chat_text_is_the_first_choice() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":" ok "}}]}"#;
        assert_eq!(chat_text(body).as_deref(), Some("ok"));
    }

    #[test]
    fn the_model_list_drops_the_empty_ids() {
        let body = r#"{"object":"list","data":[{"id":"k-exaone_v2.5"},{"id":""}]}"#;
        assert_eq!(model_ids(body), vec!["k-exaone_v2.5"]);
    }

    #[test]
    fn a_fresh_entry_carries_the_device_and_the_credentials() {
        let entry = Entry {
            password: "p".to_string(),
            api_key: "sk-abc".to_string(),
            created_at: "2026-11-20 10:00:00".to_string(),
            device: Some(Device::plain()),
            terms: "REQUIRED_TERMS,MARKETING".to_string(),
            step_up: String::new(),
            portal_at: "at-token".to_string(),
            portal_rt: "rt-token".to_string(),
            session_id: "ex-sid".to_string(),
        };
        let value = serde_json::to_value(&entry).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 9);
        assert_eq!(value["api_key"], "sk-abc");
        assert_eq!(value["terms"], "REQUIRED_TERMS,MARKETING");
        assert!(value["device"]["agent"]
            .as_str()
            .unwrap()
            .contains("Mozilla/5.0"));
    }

    #[test]
    fn an_old_entry_without_a_device_still_reads() {
        let text = r#"{"a@b.test":{"password":"p","api_key":"k","created_at":"x"}}"#;
        let book: serde_json::Value = serde_json::from_str(text).unwrap();
        let row: Entry = serde_json::from_value(book["a@b.test"].clone()).unwrap();
        assert!(row.device.is_none());
    }

    #[test]
    fn a_fresh_device_is_not_one_hardcoded_string() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..24 {
            seen.insert(Device::fresh().agent);
        }
        assert!(seen.len() > 1);
        assert_eq!(
            Device::plain().agent,
            crate::support::consts::net::BROWSER_AGENT
        );
    }
}
