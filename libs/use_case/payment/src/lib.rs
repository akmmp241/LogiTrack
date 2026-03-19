use crate::errors::PaymentError;

pub mod domain;
pub mod errors;
pub mod xendit;

#[async_trait::async_trait]
pub trait PaymentProvider: Send + Sync {
    async fn process_payment(
        &self,
        req: serde_json::Value,
    ) -> Result<serde_json::Value, PaymentError>;
    async fn get_payment_details(
        &self,
        payment_id: String,
    ) -> Result<serde_json::Value, PaymentError>;

    async fn simulate_payment(
        &self,
        payment_id: String,
        amount: Option<i64>,
    ) -> Result<bool, PaymentError>;
}
