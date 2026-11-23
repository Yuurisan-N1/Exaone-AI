use std::path::Path;
use std::time::Duration;

use crate::job::onboard;
use crate::support::cfg::loader;
use crate::support::consts::app::STEP_RETRY_MILLIS;
use crate::support::error::AppResult;
use crate::support::log::format::{countdown, gap, lg, ly};

pub async fn run(root: &Path, target: usize) -> AppResult<()> {
    cycle(root, target, false).await
}

pub async fn enroll(root: &Path, target: usize) -> AppResult<()> {
    cycle(root, target, true).await
}

async fn cycle(root: &Path, target: usize, fresh: bool) -> AppResult<()> {
    let setup = loader::load(root)?;
    if setup.proxies.is_empty() {
        ly("proxy.txt is empty so no proxy line is used");
        return Err(crate::support::error::AppError::config(
            "the identity host refuses this region without a proxy",
        ));
    }
    let target = target.max(1);
    let mut made = 0usize;
    let mut attempt = 0usize;

    while made < target {
        lg(&format!("Starting automation cycle number {}", attempt + 1));
        let mut slot = attempt;
        let mut rounds = 0usize;
        loop {
            rounds += 1;
            match once(&setup, slot, fresh).await {
                Ok(()) => {
                    made += 1;
                    lg(&format!(
                        "Automation cycle number {} is complete",
                        attempt + 1
                    ));
                    break;
                }
                Err(err) => {
                    let blocked = matches!(err, crate::support::error::AppError::Blocked(_));
                    if blocked && rounds < setup.proxies.len().max(1) {
                        slot += 1;
                        continue;
                    }
                    ly(&format!(
                        "Automation cycle number {} stopped: {}",
                        attempt + 1,
                        err
                    ));
                    let wait = if blocked { 0 } else { STEP_RETRY_MILLIS };
                    if wait > 0 {
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                    }
                    break;
                }
            }
        }
        attempt += 1;
        gap();
    }

    countdown(setup.config.sleep_seconds(), "Next cycle starts in").await;
    Ok(())
}

async fn once(
    setup: &crate::support::cfg::loader::Setup,
    index: usize,
    fresh: bool,
) -> AppResult<()> {
    if fresh {
        onboard::signup(setup, index).await?;
        return Ok(());
    }
    let stored =
        onboard::stored_email(setup)?.and_then(|email| onboard::stored_account(setup, &email));
    match stored {
        Some(account) => onboard::drive(setup, &account, index).await?,
        None => {
            onboard::signup(setup, index).await?;
        }
    }
    Ok(())
}
