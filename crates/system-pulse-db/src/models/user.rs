use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub login: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub login: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub subscription: String,
    pub created_at: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        UserPublic {
            id: u.id,
            login: u.login,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            avatar_url: u.avatar_url,
            subscription: u.subscription,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub login: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UpdateUserInput {
    pub login: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
}
