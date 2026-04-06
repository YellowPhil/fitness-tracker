//! Shared helpers for the `backend` and `bot` binaries.

use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    // Base filter: suppress verbose third-party crates that produce per-query or
    // per-connection noise.  Application code is kept at `info` by default.
    // Individual targets can be re-enabled via RUST_LOG, e.g.:
    //   RUST_LOG=debug,sqlx=warn
    const BASE: &str = "info,sqlx=warn,hyper_util=warn,reqwest=warn";

    let filter = match std::env::var("RUST_LOG") {
        Ok(env) if !env.is_empty() => {
            // Append the user's directives after the base so that equally-specific
            // overrides (e.g. RUST_LOG=sqlx=debug) take precedence over the base.
            EnvFilter::new(format!("{BASE},{env}"))
        }
        _ => EnvFilter::new(BASE),
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
