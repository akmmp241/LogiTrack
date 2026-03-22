use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[sqlx(type_name = "transaction_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionStatus {
    Pending,
    Paid,
    Expired,
    Failed,
}
