use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub allowed_origins: Vec<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8090".into())
                .parse()
                .context("PORT must be a number")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set — generate one with `openssl rand -hex 32`")?,
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .context("JWT_EXPIRATION_HOURS must be a number")?,
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }
}
