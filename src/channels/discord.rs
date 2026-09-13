use reqwest::Client;
use serde_json::json;

use crate::models::PaymentAskPayload;

#[derive(Debug, thiserror::Error)]
pub enum DiscordError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("discord error: {0}")]
    Api(String),
}

pub async fn send_payment_ask(
    webhook_url: &str,
    payload: &PaymentAskPayload,
) -> Result<(), DiscordError> {
    let merchant = payload
        .merchant
        .clone()
        .unwrap_or_else(|| "estabelecimento".to_string());
    let suggested = payload
        .suggested_category
        .clone()
        .unwrap_or_else(|| "outros".to_string());

    let body = json!({
        "content": format!(
            "Nova compra: **R$ {} {}** em **{}**\nSugestão: `{}`\nid: `{}`\nResponda no app com a categoria certa.",
            payload.amount,
            payload.currency,
            merchant,
            suggested,
            payload.payment_event_id
        )
    });

    let response = Client::new().post(webhook_url).json(&body).send().await?;
    if !response.status().is_success() {
        return Err(DiscordError::Api(response.text().await.unwrap_or_default()));
    }
    Ok(())
}

pub async fn send_text(webhook_url: &str, content: &str) -> Result<(), DiscordError> {
    let response = Client::new()
        .post(webhook_url)
        .json(&json!({ "content": content }))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(DiscordError::Api(response.text().await.unwrap_or_default()));
    }
    Ok(())
}
