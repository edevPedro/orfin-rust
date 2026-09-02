use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::services::payments::process_pluggy_webhook;
use crate::services::pluggy::PluggyWebhookPayload;
use crate::services::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn handle_pluggy_webhook(
    State(state): State<AppState>,
    Json(payload): Json<PluggyWebhookPayload>,
) -> impl IntoResponse {
    tracing::info!(event = %payload.event, "received pluggy webhook");

    match process_pluggy_webhook(&state.pool, &state.pluggy, payload).await {
        Ok(inserted_ids) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "processed",
                "inserted_payment_ids": inserted_ids
            })),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to process pluggy webhook");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: error.to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RegisterWebhookRequest {
    pub events: Option<Vec<String>>,
}

pub async fn register_webhooks(
    State(state): State<AppState>,
    Json(request): Json<RegisterWebhookRequest>,
) -> impl IntoResponse {
    let base_url = match state.config.webhook_base_url.as_deref() {
        Some(url) => url,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "WEBHOOK_BASE_URL is not configured".to_string(),
                }),
            )
                .into_response();
        }
    };

    let webhook_url = format!("{base_url}/webhooks/pluggy");
    let events = request.events.unwrap_or_else(|| {
        vec![
            "transactions/created".to_string(),
            "item/created".to_string(),
            "item/updated".to_string(),
        ]
    });

    let mut registered = Vec::new();
    for event in events {
        match state.pluggy.register_webhook(&webhook_url, &event).await {
            Ok(()) => registered.push(event),
            Err(error) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: format!("failed to register {event}: {error}"),
                    }),
                )
                    .into_response();
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "registered",
            "url": webhook_url,
            "events": registered
        })),
    )
        .into_response()
}
