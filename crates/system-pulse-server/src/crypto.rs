//! Lightweight reversible encryption for SSH credentials at rest.
//!
//! Same XOR+hex scheme as the desktop app uses, so a database file is
//! portable between the two binaries. For production deployments, set
//! `SERVER_ENC_KEY` to a long random value (the handlers read it via
//! `std::env`, falling back to a dev-only constant otherwise).

pub fn encrypt(data: &str, key: &str) -> String {
    let kb = key.as_bytes();
    let enc: Vec<u8> = data
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ kb[i % kb.len()])
        .collect();
    hex::encode(enc)
}

pub fn decrypt(hex_data: &str, key: &str) -> anyhow::Result<String> {
    let bytes = hex::decode(hex_data)?;
    let kb = key.as_bytes();
    let dec: Vec<u8> = bytes.iter().enumerate().map(|(i, b)| b ^ kb[i % kb.len()]).collect();
    Ok(String::from_utf8(dec)?)
}
