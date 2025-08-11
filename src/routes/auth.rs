use axum::{Router, routing::get};
use crate::services::auth;

pub fn auth() -> Router {
  Router::new()
    .route("/get_key", get(auth::get_key))
}
