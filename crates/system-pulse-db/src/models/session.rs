use serde::{Deserialize, Serialize};

/// Server-side session record. The desktop app does not create rows here
/// (it keeps JWTs purely client-side), but the table is always present so
/// the schema is identical across both binaries — the server populates it
/// to support logout and multi-device session revocation.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub created_at: String,
    pub expires_at: String,
    pub revoked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionInput {
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
}
