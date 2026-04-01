use std::env;
use std::net::SocketAddr;

use anyhow::Context;
use fitness_tracker::{ensure_db_parent_dirs, init_tracing};
use infra::{Databases, SqliteExcerciseDb, SqliteHealthDb, SqliteWorkoutDb, http_router};
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
    let exercise_path =
        env::var("FITNESS_EXERCISE_DB").unwrap_or_else(|_| "data/exercises.db".into());
    let workout_path = env::var("FITNESS_WORKOUT_DB").unwrap_or_else(|_| "data/workouts.db".into());
    let health_path =
        env::var("FITNESS_HEALTH_DB").unwrap_or_else(|_| "data/health.db".into());

    ensure_db_parent_dirs(&[&exercise_path, &workout_path, &health_path])?;

    let exercise_db =
        SqliteExcerciseDb::open(&exercise_path).context("open FITNESS_EXERCISE_DB")?;
    let workout_db = SqliteWorkoutDb::open(&workout_path).context("open FITNESS_WORKOUT_DB")?;
    let health_db = SqliteHealthDb::open(&health_path).context("open FITNESS_HEALTH_DB")?;
    let dbs = Databases::new(exercise_db, workout_db, health_db);

    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()
        .context("parse BIND_ADDR")?;

    let app = http_router(dbs);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;

    tracing::info!(%addr, "HTTP server listening (API + static UI when web/dist exists)");

    axum::serve(listener, app)
        .await
        .context("HTTP server")?;
    Ok(())
}
