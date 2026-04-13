use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use fitness_tracker::init_tracing;
use infra::{Databases, grpc, http_router};
use tokio::net::TcpListener;
use tracing::instrument;

/// Default gRPC timeout for the workout generator service. Since it's a long-running operation, we set a higher timeout than the default.
const DEFAULT_GRPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

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

    let dbs = Arc::new(
        Databases::connect(&postgres_url)
            .await
            .context("connect POSTGRES_URL")?,
    );

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

    let grpc_addr: SocketAddr = env::var("GRPC_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".into())
        .parse()
        .context("parse GRPC_BIND_ADDR")?;

    let workout_generator_grpc_addr =
        env::var("WORKOUT_GENERATOR_GRPC_URL").unwrap_or_else(|_| "http://127.0.0.1:50052".into());

    let allowed_user_ids = env::var("ALLOWED_USER_IDS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            s.split(',')
                .filter_map(|id| id.trim().parse::<i64>().ok())
                .collect::<Vec<_>>()
        })
        .filter(|ids| !ids.is_empty());

    let grpc_timeout = env::var("GRPC_TIMEOUT_SECONDS")
        .map(|timeout_str| {
            timeout_str
                .parse::<u64>()
                .map(std::time::Duration::from_secs)
                .unwrap_or(DEFAULT_GRPC_TIMEOUT)
        })
        .unwrap_or(DEFAULT_GRPC_TIMEOUT);

    let app = http_router(
        Arc::clone(&dbs),
        frontend_url.as_deref(),
        bot_token,
        dev_skip_auth,
        workout_generator_grpc_addr,
        allowed_user_ids,
        grpc_timeout,
    );
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;

    tracing::info!(%addr, "HTTP server listening (API + static UI when web/dist exists)");
    tracing::info!(%grpc_addr, "gRPC server listening (WorkoutDataService)");

    let http_server = async move { axum::serve(listener, app).await.context("HTTP server") };
    let grpc_server = grpc::serve_workout_data(grpc_addr, dbs);

    tokio::try_join!(http_server, grpc_server)?;
    Ok(())
}
