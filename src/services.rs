pub mod pluggy;
pub mod payments;

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::services::pluggy::PluggyClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub pluggy: Arc<PluggyClient>,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let pluggy = PluggyClient::new(config.clone());
        Self {
            pool,
            pluggy,
            config,
        }
    }
}
