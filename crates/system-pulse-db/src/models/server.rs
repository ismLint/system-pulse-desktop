use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Server {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub host: String,
    pub ssh_user: String,
    #[serde(skip_serializing)]
    pub password_encrypted: String,
    pub description: Option<String>,
    pub is_active: bool,
    /// "local" or "remote"
    pub server_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ServerPublic {
    pub id: String,
    pub name: String,
    pub host: String,
    pub ssh_user: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub server_type: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Server> for ServerPublic {
    fn from(s: Server) -> Self {
        ServerPublic {
            id: s.id,
            name: s.name,
            host: s.host,
            ssh_user: s.ssh_user,
            description: s.description,
            is_active: s.is_active,
            server_type: s.server_type,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateServerInput {
    pub name: String,
    pub host: String,
    pub ssh_user: String,
    pub password_encrypted: String,
    pub description: Option<String>,
    pub server_type: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateServerInput {
    pub name: Option<String>,
    pub host: Option<String>,
    pub ssh_user: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}
