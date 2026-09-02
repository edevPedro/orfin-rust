use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::models::ConnectTokenRequest;
use crate::services::payments::{create_connect_token, link_pluggy_item_to_user};
use crate::services::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LinkItemRequest {
    pub user_id: String,
    pub item_id: String,
}

pub async fn create_token(
    State(state): State<AppState>,
    Json(request): Json<ConnectTokenRequest>,
) -> impl IntoResponse {
    match create_connect_token(&state.pluggy, &request.user_id).await {
        Ok(access_token) => (
            StatusCode::OK,
            Json(serde_json::json!({ "access_token": access_token })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse {
                error: error.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn link_item(
    State(state): State<AppState>,
    Json(request): Json<LinkItemRequest>,
) -> impl IntoResponse {
    match link_pluggy_item_to_user(&state.pool, &request.item_id, &request.user_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "linked",
                "item_id": request.item_id,
                "user_id": request.user_id
            })),
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
