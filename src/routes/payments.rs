use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::models::{NotificationPaymentRequest, PaymentsQuery};
use crate::services::payments::{get_payments, ingest_notification_payment};
use crate::services::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn list_payments(
    State(state): State<AppState>,
    Query(query): Query<PaymentsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    match get_payments(&state.pool, &query.user_id, query.status.as_deref(), limit).await {
        Ok(payments) => (StatusCode::OK, Json(serde_json::json!({ "payments": payments }))).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn from_notification(
    State(state): State<AppState>,
    Json(request): Json<NotificationPaymentRequest>,
) -> impl IntoResponse {
    match ingest_notification_payment(&state.pool, request).await {
        Ok(Some(id)) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "created",
                "payment_event_id": id
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "duplicate_or_ignored" })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
            .into_response(),
    }
}
