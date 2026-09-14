use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{FromRow, PgPool};

use crate::models::{
    AlertRule, CreateAlertRequest, ALERT_BUDGET_MONTHLY, ALERT_LARGE_PURCHASE,
    ALERT_UNCATEGORIZED_STREAK, STATUS_AWAITING_USER, STATUS_CATEGORIZED,
};

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("invalid alert kind: {0}")]
    InvalidKind(String),
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub total: String,
    pub currency: String,
    pub by_category: Vec<CategoryBucket>,
    pub by_source: Vec<SourceBucket>,
    pub awaiting_count: i64,
    pub uncategorized_count: i64,
}

#[derive(Debug, Serialize)]
pub struct CategoryBucket {
    pub id: String,
    pub label: String,
    pub total: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct SourceBucket {
    pub source: String,
    pub total: String,
    pub count: i64,
}

#[derive(Debug, FromRow)]
struct CategoryRow {
    id: String,
    label: String,
    total: Decimal,
    count: i64,
}

#[derive(Debug, FromRow)]
struct SourceRow {
    source: String,
    total: Decimal,
    count: i64,
}

#[derive(Debug, Serialize)]
pub struct FiredAlert {
    pub kind: String,
    pub message: String,
    pub threshold: Option<Decimal>,
    pub current: Decimal,
}

pub async fn summary(
    pool: &PgPool,
    user_id: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<ReportSummary, ReportError> {
    let total: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount), 0) FROM payment_events \
         WHERE user_id = $1 AND paid_at >= $2 AND paid_at <= $3 \
           AND status IN ('awaiting_user', 'categorized')",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    let by_category = sqlx::query_as::<_, CategoryRow>(
        "SELECT COALESCE(pe.category, 'outros') AS id, \
                COALESCE(c.label, 'Outros') AS label, \
                COALESCE(SUM(pe.amount), 0) AS total, \
                COUNT(*)::bigint AS count \
         FROM payment_events pe \
         LEFT JOIN categories c ON c.id = pe.category \
         WHERE pe.user_id = $1 AND pe.paid_at >= $2 AND pe.paid_at <= $3 \
           AND pe.status IN ('awaiting_user', 'categorized') \
         GROUP BY COALESCE(pe.category, 'outros'), COALESCE(c.label, 'Outros') \
         ORDER BY total DESC",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| CategoryBucket {
        id: row.id,
        label: row.label,
        total: row.total.to_string(),
        count: row.count,
    })
    .collect();

    let by_source = sqlx::query_as::<_, SourceRow>(
        "SELECT source, COALESCE(SUM(amount), 0) AS total, COUNT(*)::bigint AS count \
         FROM payment_events \
         WHERE user_id = $1 AND paid_at >= $2 AND paid_at <= $3 \
           AND status IN ('awaiting_user', 'categorized') \
         GROUP BY source \
         ORDER BY total DESC",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| SourceBucket {
        source: row.source,
        total: row.total.to_string(),
        count: row.count,
    })
    .collect();

    let awaiting_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM payment_events \
         WHERE user_id = $1 AND paid_at >= $2 AND paid_at <= $3 \
           AND status = 'awaiting_user'",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    let uncategorized_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM payment_events \
         WHERE user_id = $1 AND paid_at >= $2 AND paid_at <= $3 \
           AND status IN ('awaiting_user', 'categorized') \
           AND (category IS NULL OR category = '')",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;

    Ok(ReportSummary {
        total: total.to_string(),
        currency: "BRL".into(),
        by_category,
        by_source,
        awaiting_count,
        uncategorized_count,
    })
}

pub async fn list_alerts(pool: &PgPool, user_id: &str) -> Result<Vec<AlertRule>, ReportError> {
    Ok(sqlx::query_as::<_, AlertRule>(
        "SELECT id, user_id, kind, threshold, category_id, enabled, created_at \
         FROM alert_rules WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create_alert(
    pool: &PgPool,
    req: CreateAlertRequest,
) -> Result<AlertRule, ReportError> {
    match req.kind.as_str() {
        ALERT_BUDGET_MONTHLY | ALERT_LARGE_PURCHASE | ALERT_UNCATEGORIZED_STREAK => {}
        other => return Err(ReportError::InvalidKind(other.into())),
    }

    Ok(sqlx::query_as::<_, AlertRule>(
        "INSERT INTO alert_rules (user_id, kind, threshold, category_id, enabled) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, user_id, kind, threshold, category_id, enabled, created_at",
    )
    .bind(&req.user_id)
    .bind(&req.kind)
    .bind(req.threshold)
    .bind(req.category_id.as_deref())
    .bind(req.enabled.unwrap_or(true))
    .fetch_one(pool)
    .await?)
}

pub async fn check_alerts(pool: &PgPool, user_id: &str) -> Result<Vec<FiredAlert>, ReportError> {
    let rules = sqlx::query_as::<_, AlertRule>(
        "SELECT id, user_id, kind, threshold, category_id, enabled, created_at \
         FROM alert_rules WHERE user_id = $1 AND enabled = true",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();
    let month_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .single()
        .unwrap_or(now);
    let week_ago = now - Duration::days(7);

    let mut fired = Vec::new();
    for rule in rules {
        let Some(threshold) = rule.threshold else {
            continue;
        };

        match rule.kind.as_str() {
            ALERT_BUDGET_MONTHLY => {
                let current: Decimal = if let Some(category_id) = rule.category_id.as_deref() {
                    sqlx::query_scalar(
                        "SELECT COALESCE(SUM(amount), 0) FROM payment_events \
                         WHERE user_id = $1 AND status = $2 AND paid_at >= $3 \
                           AND category = $4",
                    )
                    .bind(user_id)
                    .bind(STATUS_CATEGORIZED)
                    .bind(month_start)
                    .bind(category_id)
                    .fetch_one(pool)
                    .await?
                } else {
                    sqlx::query_scalar(
                        "SELECT COALESCE(SUM(amount), 0) FROM payment_events \
                         WHERE user_id = $1 AND status = $2 AND paid_at >= $3",
                    )
                    .bind(user_id)
                    .bind(STATUS_CATEGORIZED)
                    .bind(month_start)
                    .fetch_one(pool)
                    .await?
                };

                if current >= threshold {
                    fired.push(FiredAlert {
                        kind: rule.kind.clone(),
                        message: format!(
                            "Gasto do mês ({current}) atingiu ou passou o limite de {threshold}"
                        ),
                        threshold: Some(threshold),
                        current,
                    });
                }
            }
            ALERT_LARGE_PURCHASE => {
                let current: Option<Decimal> = sqlx::query_scalar(
                    "SELECT MAX(amount) FROM payment_events \
                     WHERE user_id = $1 AND paid_at >= $2 \
                       AND status IN ('awaiting_user', 'categorized') \
                       AND amount >= $3",
                )
                .bind(user_id)
                .bind(week_ago)
                .bind(threshold)
                .fetch_one(pool)
                .await?;

                if let Some(current) = current {
                    fired.push(FiredAlert {
                        kind: rule.kind.clone(),
                        message: format!(
                            "Compra grande detectada: {current} (limite {threshold}) nos últimos 7 dias"
                        ),
                        threshold: Some(threshold),
                        current,
                    });
                }
            }
            ALERT_UNCATEGORIZED_STREAK => {
                let current: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*)::bigint FROM payment_events \
                     WHERE user_id = $1 AND status = $2",
                )
                .bind(user_id)
                .bind(STATUS_AWAITING_USER)
                .fetch_one(pool)
                .await?;
                let current = Decimal::from(current);
                if current >= threshold {
                    fired.push(FiredAlert {
                        kind: rule.kind.clone(),
                        message: format!(
                            "{current} pagamentos aguardando categoria (limite {threshold})"
                        ),
                        threshold: Some(threshold),
                        current,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(fired)
}
