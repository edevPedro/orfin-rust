use sqlx::PgPool;

use crate::channels::{discord, telegram};
use crate::config::Config;
use crate::models::{
    PaymentAskPayload, PaymentEvent, CHANNEL_DISCORD, CHANNEL_TELEGRAM, STATUS_AWAITING_USER,
};
use crate::push::FcmClient;

#[derive(Debug, sqlx::FromRow)]
struct DeviceTokenRow {
    fcm_token: String,
}

#[derive(Debug, sqlx::FromRow)]
struct ChannelLinkRow {
    channel: String,
    target: String,
}

pub async fn on_new_payment(pool: &PgPool, config: &Config, payment: &PaymentEvent) {
    if payment.status != STATUS_AWAITING_USER {
        return;
    }

    let payload = PaymentAskPayload {
        kind: "payment_ask",
        payment_event_id: payment.id,
        amount: payment.amount.normalize().to_string(),
        currency: payment.currency.clone(),
        merchant: payment.merchant.clone(),
        suggested_category: payment.suggested_category.clone(),
        paid_at: payment.paid_at,
    };

    let fcm = FcmClient::new(config.fcm_server_key.clone());
    if let Ok(tokens) = sqlx::query_as::<_, DeviceTokenRow>(
        "SELECT fcm_token FROM device_tokens WHERE user_id = $1",
    )
    .bind(&payment.user_id)
    .fetch_all(pool)
    .await
    {
        for token in tokens {
            if let Err(error) = fcm.send_payment_ask(&token.fcm_token, &payload).await {
                tracing::warn!(error = %error, "failed to send fcm push");
            }
        }
    }

    if let Ok(links) = sqlx::query_as::<_, ChannelLinkRow>(
        "SELECT channel, target FROM channel_links WHERE user_id = $1 AND enabled = true",
    )
    .bind(&payment.user_id)
    .fetch_all(pool)
    .await
    {
        for link in links {
            let result = match link.channel.as_str() {
                CHANNEL_DISCORD => discord::send_payment_ask(&link.target, &payload)
                    .await
                    .map_err(|e| e.to_string()),
                CHANNEL_TELEGRAM => telegram::send_payment_ask(
                    config.telegram_bot_token.as_deref(),
                    &link.target,
                    &payload,
                )
                .await
                .map_err(|e| e.to_string()),
                other => {
                    tracing::debug!(channel = other, "skipping unsupported channel");
                    Ok(())
                }
            };

            if let Err(error) = result {
                tracing::warn!(channel = %link.channel, error = %error, "failed to notify channel");
            } else {
                let _ = save_explanation(
                    pool,
                    payment.id,
                    &format!(
                        "payment_ask {} R$ {} {}",
                        payload.payment_event_id, payload.amount, payload.currency
                    ),
                )
                .await;
            }
        }
    }
}

pub async fn on_payment_categorized(pool: &PgPool, config: &Config, payment: &PaymentEvent) {
    let merchant = payment
        .merchant
        .clone()
        .unwrap_or_else(|| "estabelecimento".to_string());
    let category = payment
        .category
        .clone()
        .unwrap_or_else(|| "outros".to_string());
    let note = payment
        .user_note
        .as_deref()
        .map(|value| format!("\nNota: {value}"))
        .unwrap_or_default();
    let message = format!(
        "Organizado: R$ {} {} em {} → categoria `{}`{note}",
        payment.amount.normalize(),
        payment.currency,
        merchant,
        category
    );

    let Ok(links) = sqlx::query_as::<_, ChannelLinkRow>(
        "SELECT channel, target FROM channel_links WHERE user_id = $1 AND enabled = true",
    )
    .bind(&payment.user_id)
    .fetch_all(pool)
    .await
    else {
        return;
    };

    for link in links {
        let result = match link.channel.as_str() {
            CHANNEL_DISCORD => discord::send_text(&link.target, &message)
                .await
                .map_err(|e| e.to_string()),
            CHANNEL_TELEGRAM => {
                telegram::send_text(config.telegram_bot_token.as_deref(), &link.target, &message)
                    .await
                    .map_err(|e| e.to_string())
            }
            _ => Ok(()),
        };

        if let Err(error) = result {
            tracing::warn!(channel = %link.channel, error = %error, "failed to send confirmation");
        }
    }
}

async fn save_explanation(
    pool: &PgPool,
    payment_id: uuid::Uuid,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO payment_explanations (payment_event_id, message_text) VALUES ($1, $2)",
    )
    .bind(payment_id)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(())
}
