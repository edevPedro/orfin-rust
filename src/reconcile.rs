use chrono::Duration;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    RECONCILE_MATCHED, RECONCILE_UNMATCHED, SOURCE_ANDROID, SOURCE_OCR, SOURCE_PLUGGY,
};

#[derive(Debug, thiserror::Error)]
pub enum ReconcileError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Serialize)]
pub struct ReconcileResult {
    pub matched: i64,
    pub candidates_left: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct Candidate {
    id: Uuid,
    amount: rust_decimal::Decimal,
    paid_at: chrono::DateTime<chrono::Utc>,
}

/// Match Open Finance (Pluggy) rows with Android notification / OCR rows.
///
/// Algorithm (greedy, deterministic):
/// 1. Load unmatched Pluggy payments for the user.
/// 2. For each, find the oldest unmatched android_notification|receipt_ocr
///    with the same amount and paid_at within ±window_minutes.
/// 3. Link both sides (reconcile_status=matched, reconciled_with=peer id).
///
/// Pluggy is treated as the ledger source of truth; mobile/OCR is the
/// real-time signal. Soft duplicates at ingest already catch some overlap;
/// this pass links survivors for reporting.
pub async fn run(
    pool: &PgPool,
    user_id: &str,
    window_minutes: i64,
) -> Result<ReconcileResult, ReconcileError> {
    let window = Duration::minutes(window_minutes.clamp(1, 24 * 60));

    let pluggy_rows = sqlx::query_as::<_, Candidate>(
        "SELECT id, amount, paid_at FROM payment_events \
         WHERE user_id = $1 AND source = $2 \
           AND reconcile_status = $3 \
           AND status IN ('awaiting_user', 'categorized') \
         ORDER BY paid_at ASC",
    )
    .bind(user_id)
    .bind(SOURCE_PLUGGY)
    .bind(RECONCILE_UNMATCHED)
    .fetch_all(pool)
    .await?;

    let mut matched = 0i64;

    for pluggy in pluggy_rows {
        let peer: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM payment_events \
             WHERE user_id = $1 \
               AND source IN ($2, $3) \
               AND reconcile_status = $4 \
               AND amount = $5 \
               AND paid_at BETWEEN $6 AND $7 \
               AND status IN ('awaiting_user', 'categorized', 'duplicate') \
             ORDER BY paid_at ASC \
             LIMIT 1",
        )
        .bind(user_id)
        .bind(SOURCE_ANDROID)
        .bind(SOURCE_OCR)
        .bind(RECONCILE_UNMATCHED)
        .bind(pluggy.amount)
        .bind(pluggy.paid_at - window)
        .bind(pluggy.paid_at + window)
        .fetch_optional(pool)
        .await?;

        let Some(peer_id) = peer else {
            continue;
        };

        let mut tx = pool.begin().await?;
        sqlx::query(
            "UPDATE payment_events \
             SET reconcile_status = $1, reconciled_with = $2 \
             WHERE id = $3",
        )
        .bind(RECONCILE_MATCHED)
        .bind(peer_id)
        .bind(pluggy.id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE payment_events \
             SET reconcile_status = $1, reconciled_with = $2 \
             WHERE id = $3",
        )
        .bind(RECONCILE_MATCHED)
        .bind(pluggy.id)
        .bind(peer_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        matched += 1;
    }

    let candidates_left: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM payment_events \
         WHERE user_id = $1 AND reconcile_status = $2 \
           AND source IN ($3, $4) \
           AND status IN ('awaiting_user', 'categorized')",
    )
    .bind(user_id)
    .bind(RECONCILE_UNMATCHED)
    .bind(SOURCE_ANDROID)
    .bind(SOURCE_OCR)
    .fetch_one(pool)
    .await?;

    Ok(ReconcileResult {
        matched,
        candidates_left,
    })
}
