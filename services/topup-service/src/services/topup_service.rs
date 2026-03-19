use crate::domain::dto::{
    CurrentMonthUsage, GetWalletResponse, GetWalletTransactionsResponse, TopupAction, TopupRequest,
    TopupResponse,
};
use crate::domain::transaction::{Transaction, TransactionStatus};
use crate::domain::wallet::Wallet;
use crate::repositories::billing_repository::BillingRepository;
use crate::repositories::transaction_repository::TransactionRepository;
use crate::repositories::user_repo::UserRepository;
use anyhow::anyhow;
use chrono::{Datelike, Local, Utc};
use errors::error::HttpError;
use payment::PaymentProvider;
use payment::domain::dto::{ProcessPaymentReq, ProcessPaymentRes};
use payment::errors::PaymentError;
use redis::AsyncCommands;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TopupService {
    pub payment_uc: Arc<dyn PaymentProvider>,
    pub billing_repository: BillingRepository,
    pub transaction_repository: TransactionRepository,
    pub user_repository: UserRepository,
    pub redis: deadpool_redis::Pool,
}

impl TopupService {
    pub fn new(
        billing_repository: BillingRepository,
        redis: deadpool_redis::Pool,
        user_repository: UserRepository,
        transaction_repository: TransactionRepository,
        provider: Arc<dyn PaymentProvider>,
    ) -> Self {
        Self {
            payment_uc: provider,
            billing_repository,
            transaction_repository,
            user_repository,
            redis,
        }
    }

    pub async fn get_wallet(&self, user_id: Uuid) -> Result<GetWalletResponse, HttpError> {
        let wallet = self.get_wallet_by_user(user_id).await?;

        let (year, month) = {
            let current_period = Local::now();
            (current_period.year(), current_period.month())
        };

        let awb_count = {
            let mut conn = self.redis.get().await.map_err(|e| {
                tracing::error!("Failed to get redis connection pool: {}", e);
                HttpError::InternalServerError(e.into())
            })?;

            let key = format!("awb_count:{}:{}_{}", user_id, year, month);

            let result = conn.get::<&str, Option<i64>>(&key).await.map_err(|e| {
                tracing::error!("Failed to get topup result from redis: {}", e);
                HttpError::InternalServerError(e.into())
            })?;

            result.unwrap_or(0)
        };

        let (current_tier, next_tier_threshold) = {
            let current_tier = self
                .billing_repository
                .get_tier_by_volume(&self.billing_repository.pool, &(awb_count as i32))
                .await
                .map_err(|e| HttpError::InternalServerError(e.into()))?
                .ok_or_else(|| HttpError::NotFound("Tier threshold not found".into()))?;

            let next_tier_threshold: Option<i32> = current_tier.max_volume.map(|max| max + 1);

            (current_tier.tier_level, next_tier_threshold)
        };

        Ok(GetWalletResponse {
            wallet_balance: wallet.balance,
            current_month_usage: CurrentMonthUsage {
                period: format!("{}-{}", year, month),
                awb_count: awb_count as i32,
                current_tier,
                next_tier_threshold,
            },
        })
    }

    pub async fn topup(
        &self,
        user_id: Uuid,
        req: TopupRequest,
    ) -> Result<TopupResponse, HttpError> {
        let user = self
            .user_repository
            .get_by_id(&user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow!(e)))?
            .ok_or_else(|| HttpError::NotFound("User not found".into()))?;

        let xendit_req = ProcessPaymentReq {
            reference_id: Uuid::new_v4(),
            request_amount: req.amount,
            channel_code: req.payment_method,
            user_name: user.name,
            mobile_number: user.phone_number,
        };

        let payload = serde_json::to_value(&xendit_req).map_err(|e| {
            tracing::error!("Failed to serialize XenditPaymentReq: {}", e);
            HttpError::InternalServerError(anyhow!(e))
        })?;

        let topup_res = self
            .payment_uc
            .process_payment(payload)
            .await
            .map_err(|e| match e {
                PaymentError::BadRequest(e) => HttpError::BadRequest(e),
                PaymentError::UnsupportedPaymentMethod(e) => {
                    HttpError::BadRequest(e.unwrap_or_default())
                }
                PaymentError::DuplicatedPayment() => {
                    HttpError::Conflict("Duplicated payment".into())
                }
                PaymentError::ProviderError(e) => {
                    tracing::error!("error from payment provider");
                    HttpError::InternalServerError(anyhow!(e))
                }
                PaymentError::Unexpected() => {
                    tracing::error!("unexpected error from provider");
                    HttpError::InternalServerError(anyhow!("unexpected error from provider"))
                }
            })?;

        let res = serde_json::from_value::<ProcessPaymentRes>(topup_res).map_err(|e| {
            tracing::error!("Failed to deserialize payment response: {}", e);
            HttpError::InternalServerError(anyhow!(e))
        })?;

        let transaction = Transaction {
            id: Uuid::new_v4(),
            external_payment_id: res.external_id,
            request_amount: req.amount,
            charge_amount: res.charge_amount,
            total_amount: res.total_amount,
            status: TransactionStatus::Pending,
            payment_method: xendit_req.channel_code,
            metadata: None,
            paid_at: None,
            created_at: Utc::now(),
        };

        let pass = self
            .transaction_repository
            .save(&user_id, &transaction)
            .await
            .map_err(|e| {
                tracing::error!("Failed to save transaction: {}", e);
                HttpError::InternalServerError(anyhow!(e))
            })?;

        let actions = serde_json::to_value(&res.actions).map_err(|e| {
            tracing::error!("Failed to serialize actions: {}", e);
            HttpError::InternalServerError(anyhow!(e))
        })?;

        let topup_res = TopupResponse {
            id: transaction.id,
            status: transaction.status,
            actions,
        };

        Ok(topup_res)
    }

    pub async fn get_transactions(
        &self,
        user_id: Uuid,
    ) -> Result<GetWalletTransactionsResponse, HttpError> {
        let wallet = self.get_wallet_by_user(user_id).await?;

        let res = self
            .billing_repository
            .get_wallet_transactions(&self.billing_repository.pool, &wallet.id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        Ok(res)
    }

    async fn get_wallet_by_user(&self, user_id: Uuid) -> Result<Wallet, HttpError> {
        let wallet = self
            .billing_repository
            .get_wallet_by_user_id(&self.billing_repository.pool, &user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?
            .ok_or_else(|| HttpError::NotFound("Wallet not found".into()))?;

        Ok(wallet)
    }
}
