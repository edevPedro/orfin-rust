use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::models::{PaymentEvent, PaymentSource, PaymentStatus};

pub struct NewPaymentEvent<'a> {
    pub user_id: &'a str,
    pub source: PaymentSource,
    pub external_id: Option<&'a str>,
    pub amount: Decimal,
    pub currency: &'a str,
    pub description: Option<&'a str>,
    pub merchant: Option<&'a str>,
    pub category: Option<&'a str>,
    pub paid_at: DateTime<Utc>,
    pub raw_payload: Option<serde_json::Value>,
    pub status: PaymentStatus,
}

pub async fn insert_payment_event(
    pool: &PgPool,
    event: NewPaymentEvent<'_>,
) -> Result<PaymentEvent, sqlx::Error> {
    sqlx::query_as::<_, PaymentEvent>(
        r#"
        INSERT INTO payment_events (
            user_id, source, external_id, amount, currency, description,
            merchant, category, paid_at, raw_payload, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (source, external_id) DO NOTHING
        RETURNING
            id, user_id, source, external_id, amount, currency, description,
            merchant, category, paid_at, raw_payload, status, explained_at, created_at
        "#,
    )
    .bind(event.user_id)
    .bind(event.source.as_str())
    .bind(event.external_id)
    .bind(event.amount)
    .bind(event.currency)
    .bind(event.description)
    .bind(event.merchant)
    .bind(event.category)
    .bind(event.paid_at)
    .bind(event.raw_payload)
    .bind(event.status.as_str())
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn find_existing_by_external_id(
    pool: &PgPool,
    source: &str,
    external_id: &str,
) -> Result<Option<PaymentEvent>, sqlx::Error> {
    sqlx::query_as::<_, PaymentEvent>(
        r#"
        SELECT
            id, user_id, source, external_id, amount, currency, description,
            merchant, category, paid_at, raw_payload, status, explained_at, created_at
        FROM payment_events
        WHERE source = $1 AND external_id = $2
        "#,
    )
    .bind(source)
    .bind(external_id)
    .fetch_optional(pool)
    .await
}

pub async fn find_duplicate_within_window(
    pool: &PgPool,
    user_id: &str,
    amount: Decimal,
    paid_at: DateTime<Utc>,
    exclude_source: &str,
) -> Result<Option<PaymentEvent>, sqlx::Error> {
    let window_start = paid_at - Duration::minutes(5);
    let window_end = paid_at + Duration::minutes(5);

    sqlx::query_as::<_, PaymentEvent>(
        r#"
        SELECT
            id, user_id, source, external_id, amount, currency, description,
            merchant, category, paid_at, raw_payload, status, explained_at, created_at
        FROM payment_events
        WHERE user_id = $1
          AND amount = $2
          AND paid_at BETWEEN $3 AND $4
          AND source <> $5
          AND status <> 'duplicate'
        ORDER BY
            CASE source
                WHEN 'pluggy' THEN 0
                ELSE 1
            END,
            created_at ASC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(amount)
    .bind(window_start)
    .bind(window_end)
    .bind(exclude_source)
    .fetch_optional(pool)
    .await
}

pub async fn list_payments(
    pool: &PgPool,
    user_id: &str,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<PaymentEvent>, sqlx::Error> {
    sqlx::query_as::<_, PaymentEvent>(
        r#"
        SELECT
            id, user_id, source, external_id, amount, currency, description,
            merchant, category, paid_at, raw_payload, status, explained_at, created_at
        FROM payment_events
        WHERE user_id = $1
          AND ($2::text IS NULL OR status = $2)
        ORDER BY paid_at DESC
        LIMIT $3
        "#,
    )
    .bind(user_id)
    .bind(status)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn upsert_item_user_mapping(
    pool: &PgPool,
    item_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO pluggy_item_users (item_id, user_id)
        VALUES ($1, $2)
        ON CONFLICT (item_id) DO UPDATE SET user_id = EXCLUDED.user_id
        "#,
    )
    .bind(item_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_id_for_pluggy_item(
    pool: &PgPool,
    item_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT user_id
        FROM pluggy_item_users
        WHERE item_id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await
}
