use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPrice {
    pub id: Uuid,
    pub field: String,
    pub value: i32,
}
