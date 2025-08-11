pub mod auth;
pub mod items;

use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};

pub fn init_routes() -> Router {
    Router::new()
        .route("/hello", get(hello))
        .nest("/auth", auth::auth())
        .nest("/items", items::items())
}

async fn hello() -> impl IntoResponse {
    Html("Hello, World!")
}

