use std::path::Path;

use crate::cli::{enroll, probe, selftest};
use crate::job::r#loop;
use crate::support::error::{AppError, AppResult};
use crate::support::log::format::ly;

const USAGE: &str = "\
Usage: exaone-bot <mode>

  run         open one cycle, sign up when data.txt holds no key
  enroll <n>  sign up n accounts and keep every one of them
  probe       check the hosts and every proxy
  selftest    run the offline checks
";

pub async fn execute(root: &Path) -> AppResult<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args
        .first()
        .map(|row| row.trim().to_ascii_lowercase())
        .unwrap_or_default();

    match mode.as_str() {
        "" | "run" => r#loop::run(root, 1).await,
        "enroll" => enroll::execute(root, args.get(1)).await,
        "probe" => probe::execute(root).await,
        "selftest" => selftest::execute(),
        other => {
            ly(&format!("Unknown mode {}", other));
            ly(USAGE.trim_end());
            Err(AppError::config(format!("{} is not a known mode", other)))
        }
    }
}
