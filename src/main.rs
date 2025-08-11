use std::sync::Arc;
use axum::Extension;
use tokio::net::TcpListener;

use crate::services::UserKey;
mod routes;
mod services;

#[tokio::main]
async fn main() {
	let user_key = Arc::new(UserKey::new());
	let app = routes::init_routes()
			.layer(Extension(user_key));
  let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

	axum::serve(listener, app).await.unwrap();
}

