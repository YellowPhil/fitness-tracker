//! Telegram bot: opens the Mini App via /start.
//!
//! Run as the `bot` binary (`cargo run --bin bot`); the HTTP API is served by the `backend` binary.

mod handlers;

use teloxide::dispatching::Dispatcher;
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use tracing::instrument;

use self::handlers::{Command, WebAppPublicUrl, handle_command, handle_generic_message};

/// Run the bot until shutdown. Requires `TELOXIDE_TOKEN` (see [`Bot::from_env`]).
#[instrument(fields(web_app_url = %web_app_url))]
pub async fn run_bot(web_app_url: String) {
    let bot = Bot::from_env();
    let schema = bot_schema();

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![WebAppPublicUrl(web_app_url)])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn bot_schema() -> UpdateHandler<anyhow::Error> {
    dptree::entry().branch(
        Update::filter_message()
            .branch(teloxide::filter_command::<Command, _>().endpoint(handle_command))
            .branch(dptree::endpoint(handle_generic_message)),
    )
}
