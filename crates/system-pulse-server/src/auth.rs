use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub login: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(user_id: &str, login: &str, secret: &str, hours: i64) -> Result<(String, String)> {
    let now = Utc::now();
    let exp = now + Duration::hours(hours);

    let claims = Claims {
        sub: user_id.to_string(),
        login: login.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .context("failed to encode JWT")?;

    Ok((token, exp.to_rfc3339()))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .context("invalid or expired token")?;
    Ok(data.claims)
}

pub fn extract_bearer(header: &str) -> Option<&str> {
    header.strip_prefix("Bearer ")
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .context("failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).context("invalid password hash")?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}
