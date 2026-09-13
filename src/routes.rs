use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::categorize::list_categories;
use crate::models::{
    ChannelLinkRequest, ConnectTokenRequest, ExplainPaymentRequest, LinkItemRequest,
    NotificationPaymentRequest, PaymentsQuery, PushTokenRequest, RegisterWebhookRequest,
    CHANNEL_DISCORD, CHANNEL_TELEGRAM,
};
use crate::payments::{
    explain_payment, ingest_notification, link_channel, link_item, list_awaiting, list_payments,
    process_webhook, register_push_token,
};
use crate::pluggy::PluggyWebhookPayload;
use crate::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "status": "ok" })) }),
        )
        .route("/connect/token", post(create_token))
        .route("/connect/items", post(link_item_handler))
        .route("/webhooks/pluggy", post(webhook))
        .route("/webhooks/pluggy/register", post(register_webhooks))
        .route("/payments", get(list))
        .route("/payments/awaiting", get(awaiting))
        .route("/payments/from-notification", post(from_notification))
        .route("/payments/{id}/explain", post(explain))
        .route("/devices/push-token", post(push_token))
        .route("/channels/link", post(channel_link))
        .route("/categories", get(categories))
        .with_state(state)
}

async fn create_token(
    State(state): State<AppState>,
    Json(body): Json<ConnectTokenRequest>,
) -> impl IntoResponse {
    match state.pluggy.create_connect_token(&body.user_id).await {
        Ok(access_token) => ok(
            StatusCode::OK,
            serde_json::json!({ "access_token": access_token }),
        ),
        Err(error) => err(StatusCode::BAD_GATEWAY, error),
    }
}

async fn link_item_handler(
    State(state): State<AppState>,
    Json(body): Json<LinkItemRequest>,
) -> impl IntoResponse {
    match link_item(&state.pool, &body.item_id, &body.user_id).await {
        Ok(()) => ok(
            StatusCode::OK,
            serde_json::json!({ "status": "linked", "item_id": body.item_id, "user_id": body.user_id }),
        ),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn webhook(
    State(state): State<AppState>,
    Json(payload): Json<PluggyWebhookPayload>,
) -> impl IntoResponse {
    match process_webhook(&state.pool, &state.pluggy, &state.config, payload).await {
        Ok(ids) => ok(
            StatusCode::OK,
            serde_json::json!({ "status": "processed", "inserted_payment_ids": ids }),
        ),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn register_webhooks(
    State(state): State<AppState>,
    Json(body): Json<RegisterWebhookRequest>,
) -> impl IntoResponse {
    let Some(base_url) = state.config.webhook_base_url.as_deref() else {
        return err(StatusCode::BAD_REQUEST, "WEBHOOK_BASE_URL is not configured");
    };

    let url = format!("{base_url}/webhooks/pluggy");
    let events = body.events.unwrap_or_else(|| {
        vec![
            "transactions/created".into(),
            "item/created".into(),
            "item/updated".into(),
        ]
    });

    for event in &events {
        if let Err(error) = state.pluggy.register_webhook(&url, event).await {
            return err(
                StatusCode::BAD_GATEWAY,
                format!("failed to register {event}: {error}"),
            );
        }
    }

    ok(
        StatusCode::OK,
        serde_json::json!({ "status": "registered", "url": url, "events": events }),
    )
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<PaymentsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    match list_payments(&state.pool, &query.user_id, query.status.as_deref(), limit).await {
        Ok(payments) => ok(StatusCode::OK, serde_json::json!({ "payments": payments })),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn awaiting(
    State(state): State<AppState>,
    Query(query): Query<PaymentsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    match list_awaiting(&state.pool, &query.user_id, limit).await {
        Ok(payments) => ok(StatusCode::OK, serde_json::json!({ "payments": payments })),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn from_notification(
    State(state): State<AppState>,
    Json(body): Json<NotificationPaymentRequest>,
) -> impl IntoResponse {
    match ingest_notification(&state.pool, &state.config, body).await {
        Ok(Some(id)) => ok(
            StatusCode::CREATED,
            serde_json::json!({ "status": "created", "payment_event_id": id }),
        ),
        Ok(None) => ok(
            StatusCode::OK,
            serde_json::json!({ "status": "duplicate_or_ignored" }),
        ),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn explain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<ExplainPaymentRequest>,
) -> impl IntoResponse {
    match explain_payment(&state.pool, &state.config, id, body).await {
        Ok(payment) => ok(
            StatusCode::OK,
            serde_json::json!({ "status": "categorized", "payment": payment }),
        ),
        Err(crate::payments::PaymentError::NotFound) => {
            err(StatusCode::NOT_FOUND, "payment not found or not awaiting user")
        }
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn push_token(
    State(state): State<AppState>,
    Json(body): Json<PushTokenRequest>,
) -> impl IntoResponse {
    match register_push_token(&state.pool, &body.user_id, &body.platform, &body.fcm_token).await {
        Ok(()) => ok(StatusCode::OK, serde_json::json!({ "status": "registered" })),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn channel_link(
    State(state): State<AppState>,
    Json(body): Json<ChannelLinkRequest>,
) -> impl IntoResponse {
    if body.channel != CHANNEL_DISCORD && body.channel != CHANNEL_TELEGRAM {
        return err(StatusCode::BAD_REQUEST, "channel must be discord or telegram");
    }

    match link_channel(
        &state.pool,
        &body.user_id,
        &body.channel,
        &body.target,
        body.enabled.unwrap_or(true),
    )
    .await
    {
        Ok(()) => ok(
            StatusCode::OK,
            serde_json::json!({
                "status": "linked",
                "channel": body.channel,
                "target": body.target
            }),
        ),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn categories(State(state): State<AppState>) -> impl IntoResponse {
    match list_categories(&state.pool).await {
        Ok(categories) => ok(StatusCode::OK, serde_json::json!({ "categories": categories })),
        Err(error) => err(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

fn ok(status: StatusCode, body: impl Serialize) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::to_value(body).unwrap()))
}

fn err(status: StatusCode, error: impl ToString) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": error.to_string() })),
    )
}
