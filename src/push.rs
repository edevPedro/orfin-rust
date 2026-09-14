use reqwest::Client;
use serde_json::json;

use crate::models::PaymentAskPayload;

#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("fcm error: {0}")]
    Api(String),
}

pub struct FcmClient {
    http: Client,
    server_key: Option<String>,
}

impl FcmClient {
    pub fn new(server_key: Option<String>) -> Self {
        Self {
            http: Client::new(),
            server_key,
        }
    }

    pub async fn send_payment_ask(
        &self,
        fcm_token: &str,
        payload: &PaymentAskPayload,
    ) -> Result<(), PushError> {
        let Some(server_key) = &self.server_key else {
            tracing::debug!("FCM_SERVER_KEY not set; skipping push");
            return Ok(());
        };

        let merchant = payload
            .merchant
            .clone()
            .unwrap_or_else(|| "estabelecimento".to_string());

        let body = json!({
            "to": fcm_token,
            "priority": "high",
            "notification": {
                "title": "O que foi essa compra?",
                "body": format!("R$ {} em {}", payload.amount, merchant),
            },
            "data": payload,
        });

        let response = self
            .http
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={server_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PushError::Api(response.text().await.unwrap_or_default()));
        }

        Ok(())
    }
}
