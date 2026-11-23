use std::path::Path;

use crate::net::client::Site;
use crate::net::egress::Proxy;
use crate::support::consts::net::{API_BASE, BASE_URL, MODELS, PORTAL_URL, SHOWROOM_URL};
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::{gap, lg, ly};
use crate::wire::endpoint;

pub async fn execute(root: &Path) -> AppResult<()> {
    ly(&format!("Talking to {}", BASE_URL));

    for (label, base) in [
        ("The identity host", BASE_URL),
        ("The portal host", PORTAL_URL),
        ("The callback host", SHOWROOM_URL),
    ] {
        match Site::open(base, None)?.get("/").await {
            Ok(reply) if reply.is_success() => lg(&format!("{} answers", label)),
            Ok(reply) => ly(&format!("{} answers HTTP {}", label, reply.status)),
            Err(err) => ly(&format!("{} failed: {}", label, err)),
        }
    }

    let api = Site::open(API_BASE, None)?;
    match api.get(MODELS).await {
        Ok(reply) if reply.status == 401 => lg("The model list asks for a key"),
        Ok(reply) if reply.is_success() => ly("The model list answers without a key"),
        Ok(reply) => ly(&format!("The model list answers HTTP {}", reply.status)),
        Err(err) => ly(&format!("The model list failed: {}", err)),
    }

    let lines = read_proxy_lines(root);
    if lines.is_empty() {
        ly("No proxy is configured; the identity host refuses this region");
    }
    for (index, proxy) in lines.iter().enumerate() {
        match proxy {
            Ok(proxy) => match egress(proxy).await {
                Ok((country, ip)) => {
                    if country == "US" || country == "KR" {
                        lg(&format!(
                            "Proxy {} leaves from {} via {}",
                            index + 1,
                            country,
                            ip
                        ));
                    } else {
                        ly(&format!(
                            "Proxy {} leaves from {} and the identity host refuses it",
                            index + 1,
                            country
                        ));
                    }
                }
                Err(err) => ly(&format!("Proxy {} failed: {}", index + 1, err)),
            },
            Err(err) => ly(&format!("Proxy {} is not usable: {}", index + 1, err)),
        }
    }

    gap();
    lg("The probe is done");
    Ok(())
}

fn read_proxy_lines(root: &Path) -> Vec<AppResult<Proxy>> {
    let path = root.join(crate::support::consts::app::PROXY_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(Proxy::parse)
        .collect()
}

async fn egress(proxy: &Proxy) -> AppResult<(String, String)> {
    let site = Site::open("https://ipinfo.io", Some(proxy))?;
    let reply = site.get("/json").await?;
    if !reply.is_success() {
        return Err(AppError::network(format!("HTTP {}", reply.status)));
    }
    let value: serde_json::Value = serde_json::from_str(&reply.text)
        .map_err(|_| AppError::network("the answer was not readable"))?;
    let country = value
        .get("country")
        .and_then(|row| row.as_str())
        .unwrap_or("an unknown country")
        .to_string();
    let ip = value
        .get("ip")
        .and_then(|row| row.as_str())
        .unwrap_or("an unknown address")
        .to_string();
    Ok((country, ip))
}

#[allow(dead_code)]
fn _hosts() {
    let _ = endpoint::id("");
}
