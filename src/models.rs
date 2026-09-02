use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
pub enum PaymentSource {
    #[serde(rename = "pluggy")]
    Pluggy,
    #[serde(rename = "android_notification")]
    AndroidNotification,
}

impl PaymentSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pluggy => "pluggy",
            Self::AndroidNotification => "android_notification",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text")]
pub enum PaymentStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "explained")]
    Explained,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "duplicate")]
    Duplicate,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Explained => "explained",
            Self::Failed => "failed",
            Self::Duplicate => "duplicate",
        }
    }
}

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
pub struct PaymentsQuery {
    pub user_id: String,
    pub status: Option<String>,
    pub limit: Option<i64>,
}
