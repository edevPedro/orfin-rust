use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::Config;

const PLUGGY_BASE_URL: &str = "https://api.pluggy.ai";

#[derive(Debug, thiserror::Error)]
pub enum PluggyError {
    #[error("missing api key")]
    MissingApiKey,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("pluggy api error: {0}")]
    Api(String),
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "apiKey")]
    api_key: String,
}

#[derive(Debug, Serialize)]
struct AuthRequest<'a> {
    #[serde(rename = "clientId")]
    client_id: &'a str,
    #[serde(rename = "clientSecret")]
    client_secret: &'a str,
}

#[derive(Debug, Serialize)]
struct ConnectTokenRequest<'a> {
    #[serde(rename = "clientUserId")]
    client_user_id: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ConnectTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct TransactionsPage {
    pub results: Vec<PluggyTransaction>,
    #[serde(rename = "totalPages")]
    pub total_pages: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluggyMerchant {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PluggyTransaction {
    pub id: String,
    pub description: Option<String>,
    pub amount: f64,
    pub date: String,
    #[serde(rename = "category")]
    pub category: Option<String>,
    #[serde(rename = "merchant")]
    pub merchant: Option<PluggyMerchant>,
    #[serde(rename = "paymentData")]
    pub payment_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PluggyWebhookPayload {
    pub event: String,
    #[serde(rename = "itemId")]
    pub item_id: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "clientUserId")]
    pub client_user_id: Option<String>,
    #[serde(rename = "createdTransactionsLink")]
    pub created_transactions_link: Option<String>,
    #[serde(rename = "createdTransactionsLinkV2")]
    pub created_transactions_link_v2: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterWebhookRequest<'a> {
    url: &'a str,
    event: &'a str,
}

struct CachedApiKey {
    value: String,
    fetched_at: Instant,
}

pub struct PluggyClient {
    http: Client,
    config: Config,
    api_key: RwLock<Option<CachedApiKey>>,
}

impl PluggyClient {
    pub fn new(config: Config) -> Arc<Self> {
        Arc::new(Self {
            http: Client::new(),
            config,
            api_key: RwLock::new(None),
        })
    }

    pub async fn authenticate(&self) -> Result<String, PluggyError> {
        {
            let cache = self.api_key.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < Duration::from_secs(7000) {
                    return Ok(cached.value.clone());
                }
            }
        }

        let response = self
            .http
            .post(format!("{PLUGGY_BASE_URL}/auth"))
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .json(&AuthRequest {
                client_id: &self.config.pluggy_id,
                client_secret: &self.config.pluggy_secret,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PluggyError::Api(body));
        }

        let auth: AuthResponse = response.json().await?;
        let api_key = auth.api_key;

        let mut cache = self.api_key.write().await;
        *cache = Some(CachedApiKey {
            value: api_key.clone(),
            fetched_at: Instant::now(),
        });

        Ok(api_key)
    }

    pub async fn create_connect_token(&self, client_user_id: &str) -> Result<String, PluggyError> {
        let api_key = self.authenticate().await?;
        let response = self
            .http
            .post(format!("{PLUGGY_BASE_URL}/connect_token"))
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("X-API-KEY", api_key)
            .json(&ConnectTokenRequest { client_user_id })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PluggyError::Api(body));
        }

        let token: ConnectTokenResponse = response.json().await?;
        Ok(token.access_token)
    }

    pub async fn fetch_transactions_page(
        &self,
        link: &str,
    ) -> Result<TransactionsPage, PluggyError> {
        let api_key = self.authenticate().await?;
        let response = self
            .http
            .get(link)
            .header("accept", "application/json")
            .header("X-API-KEY", api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PluggyError::Api(body));
        }

        Ok(response.json().await?)
    }

    pub async fn register_webhook(&self, url: &str, event: &str) -> Result<(), PluggyError> {
        let api_key = self.authenticate().await?;
        let response = self
            .http
            .post(format!("{PLUGGY_BASE_URL}/webhooks"))
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("X-API-KEY", api_key)
            .json(&RegisterWebhookRequest { url, event })
            .send()
            .await?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PluggyError::Api(body));
        }

        Ok(())
    }
}
