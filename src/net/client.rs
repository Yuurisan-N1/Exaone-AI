use std::sync::Arc;
use std::time::Duration;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Method, Url};
use serde_json::Value;

use crate::net::egress::Proxy;
use crate::support::consts::app::{COOKIE_GUARD_LEN, SESSION_COOKIE_MARK};
use crate::support::consts::net::{BASE_URL, CONNECT_TIMEOUT_SECS, READ_TIMEOUT_SECS};
use crate::support::error::{AppError, AppResult};
use crate::wire::shape::Device;

pub struct Reply {
    pub status: u16,
    pub text: String,

    pub location: Option<String>,
}

impl Reply {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

pub struct Site {
    http: Client,
    jar: Arc<Jar>,
    base: String,
    device: Device,

    bearer: Option<String>,
}

impl Site {
    pub fn open(base: &str, proxy: Option<&Proxy>) -> AppResult<Site> {
        Site::open_as(base, proxy, Device::plain())
    }

    pub fn open_as(base: &str, proxy: Option<&Proxy>, device: Device) -> AppResult<Site> {
        let jar = Arc::new(Jar::default());
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(READ_TIMEOUT_SECS))
            .user_agent(device.agent.clone())
            .cookie_provider(jar.clone())
            .gzip(true)
            .redirect(reqwest::redirect::Policy::none());
        if let Some(proxy) = proxy {
            let route = reqwest::Proxy::all(proxy.url())
                .map_err(|_| AppError::config("the proxy could not be used"))?;
            builder = builder.proxy(route);
        }
        let http = builder
            .build()
            .map_err(|_| AppError::network("the client could not be built"))?;
        Ok(Site {
            http,
            jar,
            base: base.trim_end_matches('/').to_string(),
            device,
            bearer: None,
        })
    }

    pub fn with_bearer(mut self, key: &str) -> Site {
        self.bearer = Some(key.to_string());
        self
    }

    fn url(&self, target: &str) -> String {
        if target.starts_with("http") {
            target.to_string()
        } else {
            format!("{}{}", self.base, target)
        }
    }

    pub fn has_cookie(&self, target: &str, mark: &str) -> bool {
        let url = match Url::parse(&self.url(target)) {
            Ok(url) => url,
            Err(_) => return false,
        };
        match self.jar.cookies(&url) {
            Some(header) => header
                .to_str()
                .map(|text| text.contains(mark))
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn cookie_value(&self, target: &str, name: &str) -> Option<String> {
        let url = Url::parse(&self.url(target)).ok()?;
        let header = self.jar.cookies(&url)?;
        let text = header.to_str().ok()?;
        for pair in text.split(';') {
            let pair = pair.trim();
            if let Some(rest) = pair.strip_prefix(name) {
                if let Some(value) = rest.strip_prefix('=') {
                    if value.len() >= COOKIE_GUARD_LEN {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn cookie_header(&self, target: &str) -> String {
        Url::parse(&self.url(target))
            .ok()
            .and_then(|url| self.jar.cookies(&url))
            .and_then(|header| header.to_str().ok().map(|text| text.to_string()))
            .unwrap_or_default()
    }

    pub fn has_session(&self) -> bool {
        self.has_cookie(BASE_URL, SESSION_COOKIE_MARK)
    }

    pub async fn get(&self, target: &str) -> AppResult<Reply> {
        self.send(Method::GET, target, None, None, &[]).await
    }

    pub async fn get_with(&self, target: &str, extra: &[(&str, String)]) -> AppResult<Reply> {
        self.send(Method::GET, target, None, None, extra).await
    }

    pub async fn get_hop(&self, target: &str) -> AppResult<Reply> {
        self.send(Method::GET, target, None, Some("text/html"), &[])
            .await
    }

    pub async fn post_json(&self, target: &str, body: &Value) -> AppResult<Reply> {
        self.post_json_with(target, body, &[]).await
    }

    pub async fn post_json_with(
        &self,
        target: &str,
        body: &Value,
        extra: &[(&str, String)],
    ) -> AppResult<Reply> {
        let text = serde_json::to_string(body)
            .map_err(|_| AppError::config("the request body could not be written"))?;
        self.send(
            Method::POST,
            target,
            Some(("application/json", text.into_bytes())),
            None,
            extra,
        )
        .await
    }

    async fn send(
        &self,
        method: Method,
        target: &str,
        body: Option<(&str, Vec<u8>)>,
        accept: Option<&str>,
        extra: &[(&str, String)],
    ) -> AppResult<Reply> {
        let url = self.url(target);
        let mut request = self
            .http
            .request(method, &url)
            .header("accept-language", self.device.language.as_str());

        if let Some(accept) = accept {
            request = request.header("accept", accept);
        }
        if let Some((content_type, payload)) = body {
            request = request.header("content-type", content_type).body(payload);
        }
        if let Some(key) = &self.bearer {
            request = request.header("authorization", format!("Bearer {}", key));
        }
        for (name, value) in extra {
            request = request.header(*name, value.as_str());
        }

        let reply = request.send().await?;
        let status = reply.status().as_u16();
        let location = reply
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let text = reply.text().await?;
        Ok(Reply {
            status,
            text,
            location,
        })
    }
}

#[allow(dead_code)]
pub fn bearer_base(base: &str) -> String {
    base.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::endpoint;

    #[test]
    fn only_a_two_hundred_reply_counts_as_success() {
        let ok = Reply {
            status: 200,
            text: String::new(),
            location: None,
        };
        assert!(ok.is_success());
        let bad = Reply {
            status: 403,
            text: String::new(),
            location: None,
        };
        assert!(!bad.is_success());
    }

    #[test]
    fn a_path_joins_the_base_and_a_url_is_left_alone() {
        let site = Site::open("https://id.exaone.ai/", None).unwrap();
        assert_eq!(site.url("/signup/init"), "https://id.exaone.ai/signup/init");
        assert_eq!(
            site.url(&endpoint::portal("/api/api-keys")),
            "https://docs.exaone.ai/api/api-keys"
        );
    }

    #[test]
    fn the_base_never_keeps_a_trailing_slash() {
        assert_eq!(bearer_base("https://a.test/"), "https://a.test");
    }

    #[test]
    fn a_fresh_site_has_no_session() {
        let site = Site::open(BASE_URL, None).unwrap();
        assert!(!site.has_session());
        assert!(site.cookie_value(BASE_URL, SESSION_COOKIE_MARK).is_none());
    }

    #[test]
    fn the_bearer_is_only_present_when_it_is_set() {
        let plain = Site::open(BASE_URL, None).unwrap();
        assert!(plain.bearer.is_none());
        let keyed = Site::open(BASE_URL, None).unwrap().with_bearer("sk-abc");
        assert_eq!(keyed.bearer.as_deref(), Some("sk-abc"));
    }
}
