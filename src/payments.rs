use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::categorize::suggest_category;
use crate::config::Config;
use crate::models::{
    ExplainPaymentRequest, NotificationPaymentRequest, PaymentEvent, SOURCE_ANDROID, SOURCE_PLUGGY,
    STATUS_AWAITING_USER, STATUS_CATEGORIZED, STATUS_DUPLICATE,
};
use crate::notify::{on_new_payment, on_payment_categorized};
use crate::pluggy::{PluggyClient, PluggyTransaction, PluggyWebhookPayload};

const PAYMENT_COLS: &str = "id, user_id, source, external_id, amount, currency, description, merchant, category, suggested_category, user_note, paid_at, raw_payload, status, explained_at, created_at";

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("pluggy error: {0}")]
    Pluggy(#[from] crate::pluggy::PluggyError),
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("payment not found")]
    NotFound,
}

pub async fn process_webhook(
    pool: &PgPool,
    pluggy: &PluggyClient,
    config: &Config,
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
        if let Some(payment) =
            save_pluggy_tx(pool, config, &user_id, &tx, payload.item_id.as_deref()).await?
        {
            ids.push(payment.id.to_string());
        }
    }
    Ok(ids)
}

pub async fn ingest_notification(
    pool: &PgPool,
    config: &Config,
    req: NotificationPaymentRequest,
) -> Result<Option<Uuid>, PaymentError> {
    let status = if find_duplicate(pool, &req.user_id, req.amount, req.paid_at, SOURCE_ANDROID)
        .await?
        .is_some()
    {
        STATUS_DUPLICATE
    } else {
        STATUS_AWAITING_USER
    };

    let suggested = if status == STATUS_DUPLICATE {
        None
    } else {
        Some(
            suggest_category(
                pool,
                &req.user_id,
                req.merchant.as_deref(),
                req.description.as_deref(),
                None,
            )
            .await?,
        )
    };

    let payment = insert_payment(
        pool,
        &req.user_id,
        SOURCE_ANDROID,
        &req.external_id,
        req.amount,
        req.currency.as_deref().unwrap_or("BRL"),
        req.description.as_deref(),
        req.merchant.as_deref(),
        None,
        suggested.as_deref(),
        req.paid_at,
        req.raw_payload,
        status,
    )
    .await?;

    if let Some(payment) = payment.as_ref() {
        if payment.status == STATUS_AWAITING_USER {
            on_new_payment(pool, config, payment).await;
        }
    }

    Ok(payment.map(|row| row.id))
}

pub async fn list_payments(
    pool: &PgPool,
    user_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<PaymentEvent>, PaymentError> {
    let query = format!(
        "SELECT {PAYMENT_COLS} FROM payment_events \
         WHERE user_id = $1 AND ($2::text IS NULL OR status = $2) \
         ORDER BY paid_at DESC LIMIT $3"
    );
    Ok(sqlx::query_as::<_, PaymentEvent>(&query)
        .bind(user_id)
        .bind(status)
        .bind(limit)
        .fetch_all(pool)
        .await?)
}

pub async fn list_awaiting(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<PaymentEvent>, PaymentError> {
    list_payments(pool, user_id, Some(STATUS_AWAITING_USER), limit).await
}

pub async fn explain_payment(
    pool: &PgPool,
    config: &Config,
    payment_id: Uuid,
    req: ExplainPaymentRequest,
) -> Result<PaymentEvent, PaymentError> {
    let query = format!(
        "UPDATE payment_events \
         SET category = $2, user_note = $3, status = $4, explained_at = now() \
         WHERE id = $1 AND status = $5 \
         RETURNING {PAYMENT_COLS}"
    );

    let payment = sqlx::query_as::<_, PaymentEvent>(&query)
        .bind(payment_id)
        .bind(&req.category)
        .bind(req.note.as_deref())
        .bind(STATUS_CATEGORIZED)
        .bind(STATUS_AWAITING_USER)
        .fetch_optional(pool)
        .await?
        .ok_or(PaymentError::NotFound)?;

    on_payment_categorized(pool, config, &payment).await;
    Ok(payment)
}

pub async fn register_push_token(
    pool: &PgPool,
    user_id: &str,
    platform: &str,
    fcm_token: &str,
) -> Result<(), PaymentError> {
    sqlx::query(
        "INSERT INTO device_tokens (user_id, platform, fcm_token) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (fcm_token) DO UPDATE \
         SET user_id = EXCLUDED.user_id, platform = EXCLUDED.platform",
    )
    .bind(user_id)
    .bind(platform)
    .bind(fcm_token)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn link_channel(
    pool: &PgPool,
    user_id: &str,
    channel: &str,
    target: &str,
    enabled: bool,
) -> Result<(), PaymentError> {
    sqlx::query(
        "INSERT INTO channel_links (user_id, channel, target, enabled) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id, channel, target) DO UPDATE SET enabled = EXCLUDED.enabled",
    )
    .bind(user_id)
    .bind(channel)
    .bind(target)
    .bind(enabled)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn link_item(pool: &PgPool, item_id: &str, user_id: &str) -> Result<(), PaymentError> {
    sqlx::query(
        "INSERT INTO pluggy_item_users (item_id, user_id) VALUES ($1, $2) \
         ON CONFLICT (item_id) DO UPDATE SET user_id = EXCLUDED.user_id",
    )
    .bind(item_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn save_pluggy_tx(
    pool: &PgPool,
    config: &Config,
    user_id: &str,
    tx: &PluggyTransaction,
    item_id: Option<&str>,
) -> Result<Option<PaymentEvent>, PaymentError> {
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

    let suggested = suggest_category(
        pool,
        user_id,
        merchant.as_deref(),
        tx.description.as_deref(),
        tx.category.as_deref(),
    )
    .await?;

    let payment = insert_payment(
        pool,
        user_id,
        SOURCE_PLUGGY,
        &tx.id,
        amount,
        "BRL",
        tx.description.as_deref(),
        merchant.as_deref(),
        tx.category.as_deref(),
        Some(&suggested),
        paid_at,
        raw,
        STATUS_AWAITING_USER,
    )
    .await?;

    if let Some(payment) = payment.as_ref() {
        on_new_payment(pool, config, payment).await;
    }

    Ok(payment)
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
    suggested_category: Option<&str>,
    paid_at: DateTime<Utc>,
    raw_payload: Option<serde_json::Value>,
    status: &str,
) -> Result<Option<PaymentEvent>, PaymentError> {
    let query = format!(
        "INSERT INTO payment_events \
            (user_id, source, external_id, amount, currency, description, merchant, category, suggested_category, paid_at, raw_payload, status) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) \
         ON CONFLICT (source, external_id) DO NOTHING \
         RETURNING {PAYMENT_COLS}"
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
        .bind(suggested_category)
        .bind(paid_at)
        .bind(raw_payload)
        .bind(status)
        .fetch_optional(pool)
        .await?)
}

async fn find_duplicate(
    pool: &PgPool,
    user_id: &str,
    amount: Decimal,
    paid_at: DateTime<Utc>,
    exclude_source: &str,
) -> Result<Option<PaymentEvent>, PaymentError> {
    let query = format!(
        "SELECT {PAYMENT_COLS} FROM payment_events \
         WHERE user_id = $1 AND amount = $2 AND paid_at BETWEEN $3 AND $4 \
           AND source <> $5 AND status <> 'duplicate' \
         ORDER BY CASE source WHEN 'pluggy' THEN 0 ELSE 1 END, created_at ASC \
         LIMIT 1"
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
    Ok(
        sqlx::query_scalar("SELECT user_id FROM pluggy_item_users WHERE item_id = $1")
            .bind(item_id)
            .fetch_optional(pool)
            .await?,
    )
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
