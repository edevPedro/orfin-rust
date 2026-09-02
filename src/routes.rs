pub mod connect;
pub mod payments;
pub mod webhooks;

use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

use crate::services::AppState;

pub fn init_routes(state: AppState) -> Router {
    Router::new()
        .route("/hello", get(hello))
        .route("/health", get(health))
        .nest("/connect", connect_routes())
        .nest("/webhooks", webhooks_routes())
        .nest("/payments", payments_routes())
        .with_state(state)
}

fn connect_routes() -> Router<AppState> {
    Router::new()
        .route("/token", post(connect::create_token))
        .route("/items", post(connect::link_item))
}

fn webhooks_routes() -> Router<AppState> {
    Router::new()
        .route("/pluggy", post(webhooks::handle_pluggy_webhook))
        .route("/pluggy/register", post(webhooks::register_webhooks))
}

fn payments_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(payments::list_payments))
        .route("/from-notification", post(payments::from_notification))
}

async fn hello() -> impl IntoResponse {
    Html("Orfin backend is running")
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}
