use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
#[sqlx(type_name = "wallet_transaction", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletTransactionType {
    Topup,
    AwbCreation,
    EmailNotification,
    WaNotification,
    Refund,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub type_: Option<WalletTransactionType>,
    pub amount: i64,
    pub reference_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, Eq, PartialEq)]
pub struct PricingTier {
    pub id: Uuid,
    pub tier_level: i16,
    pub min_volume: i32,
    pub max_volume: Option<i32>,
    pub price: Option<i64>,
}
