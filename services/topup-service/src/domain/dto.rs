use crate::domain::transaction::{Transaction, TransactionStatus};
use crate::domain::wallet::WalletTransaction;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetWalletResponse {
    pub wallet_balance: i64,
    pub current_month_usage: CurrentMonthUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentMonthUsage {
    pub period: String,
    pub awb_count: i32,
    pub current_tier: i16,
    pub next_tier_threshold: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopupRequest {
    pub amount: i64,
    pub payment_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionType {
    PresentToCustomer,
    RedirectCostumer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionDescriptor {
    QrString,
    VirtualAccountNumber,
    WebUrl,
    DeeplinkUrl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopupResponse {
    pub id: Uuid,
    pub status: TransactionStatus,
    pub actions: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopupAction {
    #[serde(rename = "type")]
    pub type_: String,
    pub descriptor: String,
    pub value: String,
}

pub type GetWalletTransactionsResponse = Vec<WalletTransaction>;

pub type GetTransactionsResponse = Vec<Transaction>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionResponse {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub actions: serde_json::Value,
}
