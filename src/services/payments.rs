use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::models::{NotificationPaymentRequest, PaymentSource, PaymentStatus};
use crate::repositories::payment_events::{
    find_duplicate_within_window, find_existing_by_external_id, insert_payment_event, list_payments,
    NewPaymentEvent,
};
use crate::services::pluggy::{PluggyClient, PluggyTransaction, PluggyWebhookPayload};

#[derive(Debug, thiserror::Error)]
pub enum PaymentServiceError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("pluggy error: {0}")]
    Pluggy(#[from] crate::services::pluggy::PluggyError),
    #[error("invalid transaction date: {0}")]
    InvalidDate(String),
}

pub async fn create_connect_token(
    pluggy: &PluggyClient,
    user_id: &str,
) -> Result<String, PaymentServiceError> {
    Ok(pluggy.create_connect_token(user_id).await?)
}

pub async fn process_pluggy_webhook(
    pool: &PgPool,
    pluggy: &PluggyClient,
    payload: PluggyWebhookPayload,
) -> Result<Vec<String>, PaymentServiceError> {
    match payload.event.as_str() {
        "transactions/created" => {
            let link = payload
                .created_transactions_link_v2
                .or(payload.created_transactions_link)
                .ok_or_else(|| {
                    PaymentServiceError::InvalidDate("missing createdTransactionsLink".into())
                })?;

            let fallback_user_id = payload
                .client_user_id
                .or_else(|| payload.item_id.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let resolved_user_id = if let Some(item_id) = &payload.item_id {
                crate::repositories::payment_events::get_user_id_for_pluggy_item(pool, item_id)
                    .await?
                    .unwrap_or(fallback_user_id)
            } else {
                fallback_user_id
            };

            let page = pluggy.fetch_transactions_page(&link).await?;
            let mut inserted_ids = Vec::new();

            for transaction in page.results {
                if let Some(event_id) = persist_pluggy_transaction(
                    pool,
                    &resolved_user_id,
                    &transaction,
                    payload.item_id.as_deref(),
                )
                .await?
                {
                    inserted_ids.push(event_id.to_string());
                }
            }

            Ok(inserted_ids)
        }
        "item/created" | "item/updated" => {
            tracing::info!(event = %payload.event, item_id = ?payload.item_id, "pluggy item event received");
            Ok(Vec::new())
        }
        other => {
            tracing::debug!(event = %other, "ignoring unsupported pluggy webhook");
            Ok(Vec::new())
        }
    }
}

async fn persist_pluggy_transaction(
    pool: &PgPool,
    user_id: &str,
    transaction: &PluggyTransaction,
    item_id: Option<&str>,
) -> Result<Option<uuid::Uuid>, PaymentServiceError> {
    if transaction.amount >= 0.0 {
        return Ok(None);
    }

    let paid_at = parse_pluggy_date(&transaction.date)?;
    let amount = Decimal::try_from(transaction.amount.abs()).unwrap_or_else(|_| Decimal::from(0));
    let merchant = transaction
        .merchant
        .as_ref()
        .and_then(|value| value.name.clone());
    let mut raw_payload = serde_json::to_value(transaction).ok();
    if let (Some(payload), Some(item_id)) = (&mut raw_payload, item_id) {
        if let Some(object) = payload.as_object_mut() {
            object.insert("itemId".to_string(), serde_json::Value::String(item_id.to_string()));
        }
    }

    if find_existing_by_external_id(pool, PaymentSource::Pluggy.as_str(), &transaction.id)
        .await?
        .is_some()
    {
        return Ok(None);
    }

    let event = NewPaymentEvent {
        user_id,
        source: PaymentSource::Pluggy,
        external_id: Some(&transaction.id),
        amount,
        currency: "BRL",
        description: transaction.description.as_deref(),
        merchant: merchant.as_deref(),
        category: transaction.category.as_deref(),
        paid_at,
        raw_payload,
        status: PaymentStatus::Pending,
    };

    match insert_payment_event(pool, event).await {
        Ok(saved) => Ok(Some(saved.id)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(error) => Err(PaymentServiceError::Database(error)),
    }
}

pub async fn ingest_notification_payment(
    pool: &PgPool,
    request: NotificationPaymentRequest,
) -> Result<Option<uuid::Uuid>, PaymentServiceError> {
    if find_existing_by_external_id(
        pool,
        PaymentSource::AndroidNotification.as_str(),
        &request.external_id,
    )
    .await?
    .is_some()
    {
        return Ok(None);
    }

    let duplicate = find_duplicate_within_window(
        pool,
        &request.user_id,
        request.amount,
        request.paid_at,
        PaymentSource::AndroidNotification.as_str(),
    )
    .await?;

    let status = if duplicate.is_some() {
        PaymentStatus::Duplicate
    } else {
        PaymentStatus::Pending
    };

    let event = NewPaymentEvent {
        user_id: &request.user_id,
        source: PaymentSource::AndroidNotification,
        external_id: Some(&request.external_id),
        amount: request.amount,
        currency: request.currency.as_deref().unwrap_or("BRL"),
        description: request.description.as_deref(),
        merchant: request.merchant.as_deref(),
        category: None,
        paid_at: request.paid_at,
        raw_payload: request.raw_payload,
        status,
    };

    match insert_payment_event(pool, event).await {
        Ok(saved) => Ok(Some(saved.id)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(error) => Err(PaymentServiceError::Database(error)),
    }
}

pub async fn get_payments(
    pool: &PgPool,
    user_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<crate::models::PaymentEvent>, PaymentServiceError> {
    Ok(list_payments(pool, user_id, status, limit).await?)
}

pub async fn link_pluggy_item_to_user(
    pool: &PgPool,
    item_id: &str,
    user_id: &str,
) -> Result<(), PaymentServiceError> {
    crate::repositories::payment_events::upsert_item_user_mapping(pool, item_id, user_id).await?;
    Ok(())
}

fn parse_pluggy_date(value: &str) -> Result<DateTime<Utc>, PaymentServiceError> {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| parsed.with_timezone(&Utc))
        .or_else(|_| {
            value
                .parse::<DateTime<Utc>>()
                .map_err(|error| PaymentServiceError::InvalidDate(error.to_string()))
        })
}
