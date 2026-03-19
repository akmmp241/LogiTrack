use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
#[sqlx(type_name = "transaction_status", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionStatus {
    Pending,
    Paid,
    Expired,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub external_payment_id: String,
    pub request_amount: i64,
    pub charge_amount: i64,
    pub total_amount: i64,
    pub status: TransactionStatus,
    pub payment_method: String,
    pub metadata: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
