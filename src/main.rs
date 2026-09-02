mod config;
mod db;
mod models;
mod repositories;
mod routes;
mod services;

use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::routes::init_routes;
use crate::services::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orfin_backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let pool = create_pool(&config)
        .await
        .expect("failed to connect to database");
    run_migrations(&pool)
        .await
        .expect("failed to run database migrations");

    let state = AppState::new(pool, config.clone());
    let app = init_routes(state);
    let address = format!("{}:{}", config.host, config.port);

    tracing::info!(%address, "starting orfin backend");
    let listener = TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
