use crate::domain::dto::{GetTransactionResponse, GetTransactionsResponse};
use crate::domain::transaction::Transaction;
use crate::repositories::transaction_repository::TransactionRepository;
use anyhow::anyhow;
use errors::error::HttpError;
use payment::PaymentProvider;
use payment::domain::dto::GetPaymentRes;
use payment::errors::PaymentError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionService {
    pub transaction_repository: TransactionRepository,
    pub payment_uc: Arc<dyn PaymentProvider>,
}

impl TransactionService {
    pub fn new(
        transaction_repository: TransactionRepository,
        payment_uc: Arc<dyn PaymentProvider>,
    ) -> Self {
        Self {
            transaction_repository,
            payment_uc,
        }
    }

    pub async fn get_transactions(
        &self,
        user_id: Uuid,
    ) -> Result<GetTransactionsResponse, HttpError> {
        let transactions = self
            .transaction_repository
            .get_transactions(&user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        Ok(transactions)
    }

    pub async fn get_transaction(
        &self,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<GetTransactionResponse, HttpError> {
        let transaction = self.find_transaction(&user_id, &id).await?;

        let provider_payment = self
            .payment_uc
            .get_payment_details(transaction.external_payment_id.clone())
            .await
            .map_err(|_e| HttpError::InternalServerError(anyhow!("something went wrong")))?;

        let payment = serde_json::from_value::<GetPaymentRes>(provider_payment).map_err(|e| {
            tracing::error!(
                "failed to deserialize transaction provider payment res: {}",
                e
            );
            HttpError::InternalServerError(anyhow!(e.to_string()))
        })?;

        let actions = serde_json::to_value(&payment.actions).map_err(|e| {
            tracing::error!("failed to serialize transaction actions: {}", e);
            HttpError::InternalServerError(anyhow!(e.to_string()))
        })?;

        Ok(GetTransactionResponse {
            transaction,
            actions,
        })
    }

    pub async fn simulate_transaction(&self, user_id: Uuid, id: Uuid) -> Result<String, HttpError> {
        let transaction = self.find_transaction(&user_id, &id).await?;

        let simulated = self
            .payment_uc
            .simulate_payment(
                transaction.external_payment_id,
                Some(transaction.total_amount),
            )
            .await
            .map_err(|e| match e {
                PaymentError::BadRequest(e) => HttpError::BadRequest(e),
                PaymentError::ProviderError(e) => HttpError::Conflict(e),
                _ => HttpError::InternalServerError(anyhow!("something went wrong")),
            })?;

        if !simulated {
            return Ok("Your payment is not simulated.".to_string());
        }

        Ok("Your payment is successfully simulated.".to_string())
    }

    async fn find_transaction(&self, user_id: &Uuid, id: &Uuid) -> Result<Transaction, HttpError> {
        let transaction = self
            .transaction_repository
            .get_transaction(user_id, id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?
            .ok_or_else(|| HttpError::NotFound("Transaction not found".to_string()))?;

        Ok(transaction)
    }
}
