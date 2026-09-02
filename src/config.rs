use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub pluggy_id: String,
    pub pluggy_secret: String,
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub webhook_base_url: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            pluggy_id: env::var("PLUGGY_ID").expect("PLUGGY_ID must be set"),
            pluggy_secret: env::var("PLUGGY_SECRET").expect("PLUGGY_SECRET must be set"),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://orfin:orfin@localhost:5432/orfin".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3000),
            webhook_base_url: env::var("WEBHOOK_BASE_URL").ok(),
        }
    }
}
