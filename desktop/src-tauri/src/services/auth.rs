use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SECRET: &str = "system_pulse_desktop_local_jwt_secret_2024";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,   // user id
    pub login: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(user_id: &str, login: &str) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        login: login.to_string(),
        exp: (now + Duration::days(30)).timestamp(),
        iat: now.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .context("Failed to encode JWT")
}

pub fn verify_token(token: &str) -> Result<Claims> {
    let mut v = Validation::default();
    v.validate_exp = true;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_bytes()),
        &v,
    )
    .context("Invalid or expired token")?;
    Ok(data.claims)
}

pub fn extract_user_id(token: &str) -> Result<String> {
    Ok(verify_token(token)?.sub)
}
