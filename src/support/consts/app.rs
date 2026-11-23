pub const CONFIG_FILE: &str = "config.json";
pub const DATA_FILE: &str = "data.txt";
pub const PROXY_FILE: &str = "proxy.txt";
pub const STATE_FILE: &str = "session.json";

pub const STEP_ATTEMPTS: u32 = 3;
pub const STEP_RETRY_MILLIS: u64 = 2000;

pub const MAIL_POLLS: u32 = 40;
pub const MAIL_POLL_SECONDS: u64 = 5;
pub const MAIL_RETRY_SECONDS: u64 = 20;

pub const EMAIL_PREFIX: &str = "ex-";
pub const EMAIL_RANDOM_LEN: usize = 12;
pub const PASSWORD_MIN_LEN: usize = 12;

pub const KEY_NAME: &str = "exaone";
pub const KEY_PREFIX: &str = "sk-";

pub const SESSION_COOKIE_MARK: &str = "EX_SID";

pub const COOKIE_GUARD_LEN: usize = 12;

pub const TERMS_FIELD: &str = "acceptedTermsIds";

pub const COUNTRY: &str = "US";
pub const NICKNAME_PREFIX: &str = "exuser";
