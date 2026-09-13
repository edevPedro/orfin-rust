use reqwest::Client;
use serde_json::json;

use crate::models::PaymentAskPayload;

#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("telegram error: {0}")]
    Api(String),
    #[error("TELEGRAM_BOT_TOKEN not configured")]
    NotConfigured,
}

pub async fn send_payment_ask(
    bot_token: Option<&str>,
    chat_id: &str,
    payload: &PaymentAskPayload,
) -> Result<(), TelegramError> {
    let token = bot_token
        .filter(|value| !value.is_empty())
        .ok_or(TelegramError::NotConfigured)?;
    let merchant = payload
        .merchant
        .clone()
        .unwrap_or_else(|| "estabelecimento".to_string());
    let suggested = payload
        .suggested_category
        .clone()
        .unwrap_or_else(|| "outros".to_string());

    let text = format!(
        "Nova compra: R$ {} {} em {}\nSugestão: {}\nid: {}\nResponda no app com a categoria certa.",
        payload.amount, payload.currency, merchant, suggested, payload.payment_event_id
    );
    send_message(token, chat_id, &text).await
}

pub async fn send_text(
    bot_token: Option<&str>,
    chat_id: &str,
    text: &str,
) -> Result<(), TelegramError> {
    let token = bot_token
        .filter(|value| !value.is_empty())
        .ok_or(TelegramError::NotConfigured)?;
    send_message(token, chat_id, text).await
}

async fn send_message(token: &str, chat_id: &str, text: &str) -> Result<(), TelegramError> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let response = Client::new()
        .post(url)
        .json(&json!({ "chat_id": chat_id, "text": text }))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(TelegramError::Api(response.text().await.unwrap_or_default()));
    }
    Ok(())
}
