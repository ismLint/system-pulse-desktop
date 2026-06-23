use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .context("Failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).context("Invalid hash")?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// XOR encrypt for stored SSH passwords
pub fn encrypt(data: &str, key: &str) -> String {
    let kb = key.as_bytes();
    let enc: Vec<u8> = data
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ kb[i % kb.len()])
        .collect();
    hex::encode(enc)
}

pub fn decrypt(hex_data: &str, key: &str) -> Result<String> {
    let bytes = hex::decode(hex_data).context("Invalid hex")?;
    let kb = key.as_bytes();
    let dec: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ kb[i % kb.len()])
        .collect();
    String::from_utf8(dec).context("Invalid UTF-8")
}
