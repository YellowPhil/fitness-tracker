//! Validate Telegram Mini App [`initData`](https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app).

use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use tracing::{debug, warn};
use urlencoding::decode;

type HmacSha256 = Hmac<Sha256>;

/// Parsed Telegram user from validated `initData`.
#[derive(Debug, Clone)]
pub struct TelegramUser {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
struct TelegramUserJson {
    id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing hash in initData")]
    MissingHash,
    #[error("invalid hash format")]
    InvalidHashHex,
    #[error("invalid initData encoding")]
    InvalidEncoding,
    #[error("invalid hash (signature mismatch)")]
    InvalidSignature,
    #[error("missing user field")]
    MissingUser,
    #[error("invalid user JSON")]
    InvalidUserJson,
    #[error("missing auth_date")]
    MissingAuthDate,
    #[error("invalid auth_date")]
    InvalidAuthDate,
    #[error("initData expired (auth_date too old)")]
    Expired,
}

/// Parse `application/x-www-form-urlencoded` style query string into key-value pairs.
/// `initData` from the client is typically raw; keys are URL-encoded.
fn parse_init_data(init_data: &str) -> Result<BTreeMap<String, String>, AuthError> {
    let mut pairs = BTreeMap::new();
    for pair in init_data.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').ok_or(AuthError::InvalidEncoding)?;
        let key = decode(k).map_err(|_| AuthError::InvalidEncoding)?;
        let key = key.into_owned();
        let value = decode(v).map_err(|_| AuthError::InvalidEncoding)?;
        pairs.insert(key, value.into_owned());
    }
    Ok(pairs)
}

/// Maximum age of `auth_date` in seconds (default 1 hour).
const DEFAULT_MAX_AGE_SECS: u64 = 3600;

/// Validate `initData` and return the Telegram user id.
pub fn validate_init_data(
    init_data: &str,
    bot_token: &str,
    max_age: Duration,
) -> Result<TelegramUser, AuthError> {
    let params = parse_init_data(init_data)?;
    let received_hash = params.get("hash").ok_or(AuthError::MissingHash)?;
    let received_bytes = hex::decode(received_hash.trim()).map_err(|_| AuthError::InvalidHashHex)?;

    // Keys sorted alphabetically (BTreeMap iteration order).
    let data_check_string: String = params
        .iter()
        .filter(|(k, _)| k.as_str() != "hash")
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut mac = HmacSha256::new_from_slice(b"WebAppData")
        .map_err(|_| AuthError::InvalidEncoding)?;
    mac.update(bot_token.as_bytes());
    let secret_key = mac.finalize().into_bytes();

    let mut mac = HmacSha256::new_from_slice(&secret_key).map_err(|_| AuthError::InvalidEncoding)?;
    mac.update(data_check_string.as_bytes());
    let expected = mac.finalize().into_bytes();

    if !constant_time_eq(&expected, &received_bytes) {
        debug!("invalid initData signature");
        return Err(AuthError::InvalidSignature);
    }

    check_auth_date(&params, max_age)?;

    let user_json = params.get("user").ok_or(AuthError::MissingUser)?;
    let user: TelegramUserJson =
        serde_json::from_str(user_json).map_err(|_| AuthError::InvalidUserJson)?;

    Ok(TelegramUser { id: user.id })
}

/// Validate `initData` with default max age (1 hour).
pub fn validate_init_data_default(init_data: &str, bot_token: &str) -> Result<TelegramUser, AuthError> {
    validate_init_data(
        init_data,
        bot_token,
        Duration::from_secs(DEFAULT_MAX_AGE_SECS),
    )
}

fn check_auth_date(params: &BTreeMap<String, String>, max_age: Duration) -> Result<(), AuthError> {
    let auth_date_str = params.get("auth_date").ok_or(AuthError::MissingAuthDate)?;
    let auth_ts: u64 = auth_date_str
        .parse()
        .map_err(|_| AuthError::InvalidAuthDate)?;
    let auth_time = UNIX_EPOCH + Duration::from_secs(auth_ts);
    let now = SystemTime::now();
    if now.duration_since(auth_time).is_err() {
        warn!("auth_date is in the future");
        return Err(AuthError::InvalidAuthDate);
    }
    let elapsed = now.duration_since(auth_time).unwrap_or_else(|_| Duration::ZERO);
    if elapsed > max_age {
        warn!(?elapsed, "initData expired");
        return Err(AuthError::Expired);
    }
    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}