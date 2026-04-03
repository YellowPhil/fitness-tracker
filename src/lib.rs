//! Shared helpers for the `backend` and `bot` binaries.

use anyhow::Context;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

/// Ensures parent directories exist for SQLite database paths.
pub fn ensure_db_parent_dirs(paths: &[&str]) -> anyhow::Result<()> {
    for path in paths {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("create {parent:?}"))?;
        }
    }
    Ok(())
}
