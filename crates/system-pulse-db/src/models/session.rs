use serde::{Deserialize, Serialize};

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
