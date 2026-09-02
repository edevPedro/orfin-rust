use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    NotificationPaymentRequest, PaymentEvent, SOURCE_ANDROID, SOURCE_PLUGGY, STATUS_DUPLICATE,
    STATUS_PENDING,
};
use crate::pluggy::{PluggyClient, PluggyTransaction, PluggyWebhookPayload};

const PAYMENT_COLS: &str = "id, user_id, source, external_id, amount, currency, description, merchant, category, paid_at, raw_payload, status, explained_at, created_at";

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("pluggy error: {0}")]
    Pluggy(#[from] crate::pluggy::PluggyError),
    #[error("invalid date: {0}")]
    InvalidDate(String),
}

pub async fn process_webhook(
    pool: &PgPool,
    pluggy: &PluggyClient,
    payload: PluggyWebhookPayload,
) -> Result<Vec<String>, PaymentError> {
    if payload.event != "transactions/created" {
        return Ok(Vec::new());
    }

    let link = payload
        .created_transactions_link_v2
        .or(payload.created_transactions_link)
        .ok_or_else(|| PaymentError::InvalidDate("missing createdTransactionsLink".into()))?;

    let fallback = payload
        .client_user_id
        .or(payload.item_id.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let user_id = match &payload.item_id {
        Some(item_id) => get_user_for_item(pool, item_id).await?.unwrap_or(fallback),
        None => fallback,
    };

    let mut ids = Vec::new();
    for tx in pluggy.fetch_transactions(&link).await?.results {
        if let Some(id) = save_pluggy_tx(pool, &user_id, &tx, payload.item_id.as_deref()).await? {
            ids.push(id.to_string());
        }
    }
    Ok(ids)
}

pub async fn ingest_notification(
    pool: &PgPool,
    req: NotificationPaymentRequest,
) -> Result<Option<Uuid>, PaymentError> {
    let status = if find_duplicate(pool, &req.user_id, req.amount, req.paid_at, SOURCE_ANDROID)
        .await?
        .is_some()
    {
        STATUS_DUPLICATE
    } else {
        STATUS_PENDING
    };

    insert_payment(
        pool,
        &req.user_id,
        SOURCE_ANDROID,
        &req.external_id,
        req.amount,
        req.currency.as_deref().unwrap_or("BRL"),
        req.description.as_deref(),
        req.merchant.as_deref(),
        None,
        req.paid_at,
        req.raw_payload,
        status,
    )
    .await
}

pub async fn list_payments(
    pool: &PgPool,
    user_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<PaymentEvent>, PaymentError> {
    let query = format!(
        "SELECT {PAYMENT_COLS} FROM payment_events WHERE user_id = $1 AND ($2::text IS NULL OR status = $2) ORDER BY paid_at DESC LIMIT $3"
    );
    Ok(sqlx::query_as::<_, PaymentEvent>(&query)
        .bind(user_id)
        .bind(status)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

pub async fn link_item(pool: &PgPool, item_id: &str, user_id: &str) -> Result<(), PaymentError> {
    sqlx::query(
        "INSERT INTO pluggy_item_users (item_id, user_id) VALUES ($1, $2) ON CONFLICT (item_id) DO UPDATE SET user_id = EXCLUDED.user_id",
    )
    .bind(item_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn save_pluggy_tx(
    pool: &PgPool,
    user_id: &str,
    tx: &PluggyTransaction,
    item_id: Option<&str>,
) -> Result<Option<Uuid>, PaymentError> {
    if tx.amount >= 0.0 {
        return Ok(None);
    }

    let paid_at = parse_date(&tx.date)?;
    let amount = Decimal::try_from(tx.amount.abs()).unwrap_or_else(|_| Decimal::from(0));
    let merchant = tx.merchant.as_ref().and_then(|m| m.name.clone());
    let mut raw = serde_json::to_value(tx).ok();
    if let (Some(value), Some(item_id)) = (&mut raw, item_id) {
        if let Some(obj) = value.as_object_mut() {
            obj.insert("itemId".into(), item_id.into());
        }
    }

    insert_payment(
        pool,
        user_id,
        SOURCE_PLUGGY,
        &tx.id,
        amount,
        "BRL",
        tx.description.as_deref(),
        merchant.as_deref(),
        tx.category.as_deref(),
        paid_at,
        raw,
        STATUS_PENDING,
    )
    .await
}

async fn insert_payment(
    pool: &PgPool,
    user_id: &str,
    source: &str,
    external_id: &str,
    amount: Decimal,
    currency: &str,
    description: Option<&str>,
    merchant: Option<&str>,
    category: Option<&str>,
    paid_at: DateTime<Utc>,
    raw_payload: Option<serde_json::Value>,
    status: &str,
) -> Result<Option<Uuid>, PaymentError> {
    let query = format!(
        "INSERT INTO payment_events (user_id, source, external_id, amount, currency, description, merchant, category, paid_at, raw_payload, status) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT (source, external_id) DO NOTHING RETURNING {PAYMENT_COLS}"
    );

    Ok(sqlx::query_as::<_, PaymentEvent>(&query)
        .bind(user_id)
        .bind(source)
        .bind(external_id)
        .bind(amount)
        .bind(currency)
        .bind(description)
        .bind(merchant)
        .bind(category)
        .bind(paid_at)
        .bind(raw_payload)
        .bind(status)
        .fetch_optional(pool)
        .await?
        .map(|row| row.id))
}

async fn find_duplicate(
    pool: &PgPool,
    user_id: &str,
    amount: Decimal,
    paid_at: DateTime<Utc>,
    exclude_source: &str,
) -> Result<Option<PaymentEvent>, PaymentError> {
    let query = format!(
        "SELECT {PAYMENT_COLS} FROM payment_events WHERE user_id = $1 AND amount = $2 AND paid_at BETWEEN $3 AND $4 AND source <> $5 AND status <> 'duplicate' ORDER BY CASE source WHEN 'pluggy' THEN 0 ELSE 1 END, created_at ASC LIMIT 1"
    );
    Ok(sqlx::query_as::<_, PaymentEvent>(&query)
        .bind(user_id)
        .bind(amount)
        .bind(paid_at - Duration::minutes(5))
        .bind(paid_at + Duration::minutes(5))
        .bind(exclude_source)
        .fetch_optional(pool)
        .await?)
}

async fn get_user_for_item(pool: &PgPool, item_id: &str) -> Result<Option<String>, PaymentError> {
    Ok(sqlx::query_scalar("SELECT user_id FROM pluggy_item_users WHERE item_id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await?)
}

fn parse_date(value: &str) -> Result<DateTime<Utc>, PaymentError> {
    DateTime::parse_from_rfc3339(value)
        .map(|d| d.with_timezone(&Utc))
        .or_else(|_| {
            value
                .parse::<DateTime<Utc>>()
                .map_err(|e| PaymentError::InvalidDate(e.to_string()))
        })
}
