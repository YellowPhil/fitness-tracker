use std::env;

use anyhow::Context;
use fitness_tracker::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(err) = run().await {
        tracing::error!(error = %err, "bot exited with error");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    env::var("TELOXIDE_TOKEN")
        .context("TELOXIDE_TOKEN must be set (Telegram bot token from @BotFather)")?;

    let web_app_url = env::var("FRONTEND_URL")
        .or_else(|_| env::var("FITNESS_WEB_APP_URL"))
        .unwrap_or_else(|_| {
            tracing::warn!(
                "FRONTEND_URL not set; defaulting to http://127.0.0.1:3001/ — \
                 set this to the public HTTPS URL where the Mini App is served"
            );
            "http://127.0.0.1:3001/".to_string()
        });

    tracing::info!(%web_app_url, "starting Telegram bot");
    infra::bot::run_bot(web_app_url).await;
    Ok(())
}
