mod config;
mod db;
mod models;
mod payments;
mod pluggy;
mod routes;

use std::sync::Arc;

use dotenvy::dotenv;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::pluggy::PluggyClient;
use crate::routes::router;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub pluggy: Arc<PluggyClient>,
    pub config: Config,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "orfin_backend=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let pool = create_pool(&config).await.expect("database connection failed");
    run_migrations(&pool).await.expect("migration failed");

    let state = AppState {
        pool,
        pluggy: PluggyClient::new(config.clone()),
        config: config.clone(),
    };

    let address = format!("{}:{}", config.host, config.port);
    tracing::info!(%address, "starting orfin backend");
    axum::serve(TcpListener::bind(&address).await.unwrap(), router(state))
        .await
        .unwrap();
}
