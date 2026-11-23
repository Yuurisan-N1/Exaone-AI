use std::path::Path;

use crate::job::r#loop;
use crate::support::error::{AppError, AppResult};

pub async fn execute(root: &Path, raw: Option<&String>) -> AppResult<()> {
    let target = match raw {
        None => 1usize,
        Some(value) => value
            .trim()
            .parse::<usize>()
            .map_err(|_| AppError::config(format!("{} is not a number of accounts", value)))?,
    };
    if target == 0 {
        return Err(AppError::config("enroll needs at least one account"));
    }
    r#loop::enroll(root, target).await
}
