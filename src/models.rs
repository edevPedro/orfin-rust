use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const SOURCE_PLUGGY: &str = "pluggy";
pub const SOURCE_ANDROID: &str = "android_notification";
pub const SOURCE_OCR: &str = "receipt_ocr";

pub const STATUS_PENDING: &str = "pending";
pub const STATUS_AWAITING_USER: &str = "awaiting_user";
pub const STATUS_CATEGORIZED: &str = "categorized";
pub const STATUS_DUPLICATE: &str = "duplicate";
pub const STATUS_FAILED: &str = "failed";

pub const CHANNEL_DISCORD: &str = "discord";
pub const CHANNEL_TELEGRAM: &str = "telegram";

pub const ALERT_BUDGET_MONTHLY: &str = "budget_monthly";
pub const ALERT_LARGE_PURCHASE: &str = "large_purchase";
pub const ALERT_UNCATEGORIZED_STREAK: &str = "uncategorized_streak";

pub const RECONCILE_UNMATCHED: &str = "unmatched";
pub const RECONCILE_MATCHED: &str = "matched";

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PaymentEvent {
    pub id: Uuid,
    pub user_id: String,
    pub source: String,
    pub external_id: Option<String>,
    pub amount: Decimal,
    pub currency: String,
    pub description: Option<String>,
    pub merchant: Option<String>,
    pub category: Option<String>,
    pub suggested_category: Option<String>,
    pub user_note: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub raw_payload: Option<serde_json::Value>,
    pub status: String,
    pub explained_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Category {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationPaymentRequest {
    pub user_id: String,
    pub external_id: String,
    pub amount: Decimal,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub merchant: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub raw_payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct OcrPaymentRequest {
    pub user_id: String,
    pub external_id: String,
    pub amount: Decimal,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub merchant: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub raw_payload: Option<serde_json::Value>,
    pub ocr_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReportSummaryQuery {
    pub user_id: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UserIdQuery {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRequest {
    pub user_id: String,
    pub kind: String,
    pub threshold: Option<Decimal>,
    pub category_id: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReconcileRunRequest {
    pub user_id: String,
    pub window_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub user_id: String,
    pub kind: String,
    pub threshold: Option<Decimal>,
    pub category_id: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectTokenRequest {
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LinkItemRequest {
    pub user_id: String,
    pub item_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PaymentsQuery {
    pub user_id: String,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterWebhookRequest {
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct PushTokenRequest {
    pub user_id: String,
    pub platform: String,
    pub fcm_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChannelLinkRequest {
    pub user_id: String,
    pub channel: String,
    pub target: String,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ExplainPaymentRequest {
    pub category: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentAskPayload {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub payment_event_id: Uuid,
    pub amount: String,
    pub currency: String,
    pub merchant: Option<String>,
    pub suggested_category: Option<String>,
    pub paid_at: DateTime<Utc>,
}
