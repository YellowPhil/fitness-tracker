//! Minimal bot handlers: open the Telegram Mini App from /start.

use anyhow::Context;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, WebAppInfo};
use teloxide::utils::command::BotCommands;
use url::Url;

pub type HandlerResult = anyhow::Result<()>;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Open the fitness web app")]
    Start,
    #[command(description = "Help")]
    Help,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    web_app_url: WebAppPublicUrl,
) -> HandlerResult {
    match cmd {
        Command::Start => {
            let url: Url = web_app_url.0.parse().context("FITNESS_WEB_APP_URL")?;
            let markup = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::web_app(
                "Open Fitness Tracker",
                WebAppInfo { url },
            )]]);
            bot.send_message(
                msg.chat.id,
                "🏋 <b>Fitness tracker</b>\n\n\
                 Tap the button below to open the app in Telegram (calendar, workouts, sets, custom exercises).",
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(markup)
            .await?;
        }
        Command::Help => {
            bot.send_message(
                msg.chat.id,
                "Open the Mini App with /start. Your data is tied to your Telegram account.",
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn handle_generic_message(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Use /start to open the fitness web app.",
    )
    .await?;
    Ok(())
}

/// Injected dependency: public `https://…` URL where the web UI is served.
#[derive(Clone)]
pub struct WebAppPublicUrl(pub String);
