use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub const SOURCE_PLUGGY: &str = "pluggy";
pub const SOURCE_ANDROID: &str = "android_notification";
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_DUPLICATE: &str = "duplicate";

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
    pub paid_at: DateTime<Utc>,
    pub raw_payload: Option<serde_json::Value>,
    pub status: String,
    pub explained_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
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
