use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::services::UserKey;

pub async fn create(Extension(key): Extension<Arc<UserKey>>) -> impl IntoResponse{
  println!("Requisição recebida!");
  Json(json!({"status": "success"}))
}