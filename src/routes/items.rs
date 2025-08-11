use axum::{routing::post, Router};
use crate::services::items;

pub fn items() -> Router {
    Router::new()
        .route("/create", post(items::create))
}