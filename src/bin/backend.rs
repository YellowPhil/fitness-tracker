use std::env;
use std::net::SocketAddr;

use anyhow::Context;
use fitness_tracker::init_tracing;
use infra::{Databases, http_router};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::instrument;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "backend exited with error");
        std::process::exit(1);
    }
}

#[instrument(skip_all)]
async fn run() -> anyhow::Result<()> {
    let postgres_url = env::var("POSTGRES_URL").context("read POSTGRES_URL")?;
    let dbs = Arc::new(Mutex::new(
        Databases::connect(&postgres_url).context("connect POSTGRES_URL")?,
    ));

    let frontend_url = env::var("FRONTEND_URL").ok();
    if let Some(ref url) = frontend_url {
        tracing::info!(%url, "CORS enabled for FRONTEND_URL");
    }

    let dev_skip_auth = env::var("DEV_SKIP_AUTH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let bot_token = match env::var("TELOXIDE_TOKEN") {
        Ok(t) if !t.trim().is_empty() => Some(t),
        _ => {
            if dev_skip_auth {
                tracing::warn!(
                    "DEV_SKIP_AUTH is set: API accepts x-user-id without Telegram initData validation — use only for local development"
                );
                None
            } else {
                anyhow::bail!(
                    "TELOXIDE_TOKEN must be set for Telegram Mini App auth (or set DEV_SKIP_AUTH=1 for local dev only)"
                );
            }
        }
    };

    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()
        .context("parse BIND_ADDR")?;

    let app = http_router(
        Arc::clone(&dbs),
        frontend_url.as_deref(),
        bot_token,
        dev_skip_auth,
    );
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;

    tracing::info!(%addr, "HTTP server listening (API + static UI when web/dist exists)");

    axum::serve(listener, app).await.context("HTTP server")?;
    Ok(())
}
