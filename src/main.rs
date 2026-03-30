use std::env;
use std::net::SocketAddr;

use anyhow::Context;
use infra::{Databases, SqliteExcerciseDb, SqliteWorkoutDb, http_router};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exercise_path =
        env::var("FITNESS_EXERCISE_DB").unwrap_or_else(|_| "data/exercises.db".into());
    let workout_path = env::var("FITNESS_WORKOUT_DB").unwrap_or_else(|_| "data/workouts.db".into());

    for path in [&exercise_path, &workout_path] {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("create {parent:?}"))?;
        }
    }

    let exercise_db =
        SqliteExcerciseDb::open(&exercise_path).context("open FITNESS_EXERCISE_DB")?;
    let workout_db = SqliteWorkoutDb::open(&workout_path).context("open FITNESS_WORKOUT_DB")?;
    let dbs = Databases::new(exercise_db, workout_db);

    let addr: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3001".into())
        .parse()
        .context("parse BIND_ADDR")?;

    let web_app_url = env::var("FITNESS_WEB_APP_URL").unwrap_or_else(|_| {
        eprintln!(
            "FITNESS_WEB_APP_URL not set; defaulting to http://127.0.0.1:{}/ (Telegram Mini App requires HTTPS in production)",
            addr.port()
        );
        format!("http://127.0.0.1:{}/", addr.port())
    });

    if web_app_url.starts_with("https://") {
        eprintln!(
            "Note: this process serves plain HTTP on {addr}. If you use an HTTPS tunnel (ngrok, cloudflared, etc.), \
             point it at http://127.0.0.1:{} — not https:// — or you may see \"Client sent an HTTP request to an HTTPS server\".",
            addr.port()
        );
    }

    let app = http_router(dbs);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;

    eprintln!("HTTP server listening on http://{addr}");

    let server = axum::serve(listener, app);

    if env::var("TELOXIDE_TOKEN").is_ok() {
        eprintln!("Starting Telegram bot (Mini App URL: {web_app_url})");
        let url = web_app_url;
        tokio::spawn(async move {
            infra::bot::run_bot(url).await;
        });
    } else {
        eprintln!("TELOXIDE_TOKEN not set; Telegram bot disabled (API and static UI only)");
    }

    server.await.context("HTTP server")?;
    Ok(())
}
