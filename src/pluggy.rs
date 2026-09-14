use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::Config;

const BASE_URL: &str = "https://api.pluggy.ai";

#[derive(Debug, thiserror::Error)]
pub enum PluggyError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("pluggy api error: {0}")]
    Api(String),
}

#[derive(Deserialize)]
struct AuthResponse {
    #[serde(rename = "apiKey")]
    api_key: String,
}

#[derive(Deserialize)]
struct ConnectTokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Deserialize)]
pub struct TransactionsPage {
    pub results: Vec<PluggyTransaction>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PluggyTransaction {
    pub id: String,
    pub description: Option<String>,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub merchant: Option<PluggyMerchant>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PluggyMerchant {
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct PluggyWebhookPayload {
    pub event: String,
    #[serde(rename = "itemId")]
    pub item_id: Option<String>,
    #[serde(rename = "clientUserId")]
    pub client_user_id: Option<String>,
    #[serde(rename = "createdTransactionsLink")]
    pub created_transactions_link: Option<String>,
    #[serde(rename = "createdTransactionsLinkV2")]
    pub created_transactions_link_v2: Option<String>,
}

struct CachedKey {
    value: String,
    at: Instant,
}

pub struct PluggyClient {
    http: Client,
    config: Config,
    api_key: RwLock<Option<CachedKey>>,
}

impl PluggyClient {
    pub fn new(config: Config) -> Arc<Self> {
        Arc::new(Self {
            http: Client::new(),
            config,
            api_key: RwLock::new(None),
        })
    }

    pub async fn create_connect_token(&self, user_id: &str) -> Result<String, PluggyError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "clientUserId")]
            client_user_id: &'a str,
        }

        let token: ConnectTokenResponse = self
            .post("/connect_token", &Body { client_user_id: user_id })
            .await?;
        Ok(token.access_token)
    }

    pub async fn fetch_transactions(&self, link: &str) -> Result<TransactionsPage, PluggyError> {
        self.get(link).await
    }

    pub async fn register_webhook(&self, url: &str, event: &str) -> Result<(), PluggyError> {
        #[derive(Serialize)]
        struct Body<'a> {
            url: &'a str,
            event: &'a str,
        }

        let _: serde_json::Value = self.post("/webhooks", &Body { url, event }).await?;
        Ok(())
    }

    async fn api_key(&self) -> Result<String, PluggyError> {
        if let Some(cached) = self.api_key.read().await.as_ref() {
            if cached.at.elapsed() < Duration::from_secs(7000) {
                return Ok(cached.value.clone());
            }
        }

        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "clientId")]
            client_id: &'a str,
            #[serde(rename = "clientSecret")]
            client_secret: &'a str,
        }

        let auth: AuthResponse = self
            .http
            .post(format!("{BASE_URL}/auth"))
            .json(&Body {
                client_id: &self.config.pluggy_id,
                client_secret: &self.config.pluggy_secret,
            })
            .send()
            .await?
            .error_for_status()
            .map_err(|error| PluggyError::Api(error.to_string()))?
            .json()
            .await?;

        *self.api_key.write().await = Some(CachedKey {
            value: auth.api_key.clone(),
            at: Instant::now(),
        });
        Ok(auth.api_key)
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, PluggyError> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{BASE_URL}{path}")
        };

        self.http
            .post(url)
            .header("X-API-KEY", self.api_key().await?)
            .json(body)
            .send()
            .await?
            .error_for_status()
            .map_err(|error| PluggyError::Api(error.to_string()))?
            .json()
            .await
            .map_err(PluggyError::from)
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, PluggyError> {
        self.http
            .get(url)
            .header("X-API-KEY", self.api_key().await?)
            .send()
            .await?
            .error_for_status()
            .map_err(|error| PluggyError::Api(error.to_string()))?
            .json()
            .await
            .map_err(PluggyError::from)
    }
}
