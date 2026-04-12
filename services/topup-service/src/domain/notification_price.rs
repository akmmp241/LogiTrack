use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPrice {
    pub id: Uuid,
    pub field: String,
    pub value: i32,
}
