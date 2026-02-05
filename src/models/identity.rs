use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserIdentity {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub provider: String,
    pub uid: String,
    pub slack_id: Option<String>,
}

impl UserIdentity {
    
}
