use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Network(String),
    #[error("{0}")]
    Api(String),
    #[error("{0}")]
    Blocked(String),
}

impl AppError {
    pub fn config(message: impl Into<String>) -> AppError {
        AppError::Config(message.into())
    }

    pub fn network(message: impl Into<String>) -> AppError {
        AppError::Network(message.into())
    }

    pub fn api(message: impl Into<String>) -> AppError {
        AppError::Api(message.into())
    }

    pub fn blocked(message: impl Into<String>) -> AppError {
        AppError::Blocked(message.into())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> AppError {
        if err.is_timeout() {
            AppError::Network("the request timed out".to_string())
        } else if err.is_connect() {
            AppError::Network("the connection could not be established".to_string())
        } else if err.is_decode() {
            AppError::Network("the reply could not be read".to_string())
        } else {
            AppError::Network("the request failed".to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(_: serde_json::Error) -> AppError {
        AppError::Network("the reply could not be read".to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub fn refusal(message: &str) -> String {
    let text = message.to_ascii_lowercase();
    let known = if text.contains("bot check failed") || text.contains("needcaptcha") {
        "the human check was refused"
    } else if text.contains("at least 18") {
        "the age confirmation was missing"
    } else if text.contains("verify your email") {
        "the mailbox has not been verified yet"
    } else if text.contains("free_models_disabled") {
        "the free models are still switched off for this account"
    } else if text.contains("rate limit") || text.contains("too many") {
        "the server is rate limiting the account"
    } else if text.contains("invalid server action") {
        "the server action envelope was refused"
    } else if text.contains("already registered") || text.contains("already exists") {
        "the mailbox is already used by an account"
    } else if text.contains("unauthorized") || text.contains("sign in first") {
        "the session was refused"
    } else {
        return short(message);
    };
    known.to_string()
}

fn short(message: &str) -> String {
    let clean: String = message
        .chars()
        .filter(|c| !c.is_control())
        .take(120)
        .collect();
    let clean = clean.trim();
    if clean.is_empty() {
        "the server refused the request without a reason".to_string()
    } else {
        format!("the server said {}", clean)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_refusal_is_written_for_a_human() {
        assert_eq!(
            refusal("Bot check failed. Please refresh and try again."),
            "the human check was refused"
        );
        assert_eq!(
            refusal("Please confirm that you are at least 18 years old."),
            "the age confirmation was missing"
        );
        assert_eq!(
            refusal("Verify your email address to use the API."),
            "the mailbox has not been verified yet"
        );
    }

    #[test]
    fn an_unknown_refusal_keeps_the_server_words() {
        let text = refusal("SOMETHING_NEW");
        assert!(text.starts_with("the server said "));
        assert!(text.contains("SOMETHING_NEW"));
    }

    #[test]
    fn an_empty_refusal_still_reads() {
        assert_eq!(
            refusal("   "),
            "the server refused the request without a reason"
        );
    }

    #[test]
    fn the_kind_names_the_layer() {
        assert_eq!(AppError::api("x").to_string(), "x");
        assert_eq!(AppError::config("x").to_string(), "x");
    }
}
