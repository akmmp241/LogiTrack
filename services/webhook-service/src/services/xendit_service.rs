use crate::domain::transaction::TransactionStatus;
use crate::domain::xendit::{XenditPaymentData, XenditPaymentRequestData};
use crate::errors::ShipmentServiceError;
use chrono::Utc;
use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
struct TransactionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub request_amount: i64,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct WalletRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: i64,
}

pub struct XenditService {
    db: Arc<PgPool>,
    redis_pool: RedisPool,
}

impl XenditService {
    pub fn new(db: Arc<PgPool>, redis_pool: RedisPool) -> Self {
        Self { db, redis_pool }
    }

    pub async fn handle_payment_capture(
        &self,
        payload: &XenditPaymentData,
    ) -> Result<(), ShipmentServiceError> {
        let mut tx = self.db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        let txn = self
            .get_transaction_by_external_id(&mut tx, &payload.payment_request_id)
            .await?
            .ok_or_else(|| {
                ShipmentServiceError::NotFound(format!(
                    "Transaction not found for payment_request_id: {}",
                    payload.payment_request_id
                ))
            })?;

        self.update_transaction_status(&mut tx, txn.id, "PAID")
            .await?;

        let wallet = self
            .get_wallet_by_user_id(&mut tx, &txn.user_id)
            .await?
            .ok_or_else(|| {
                ShipmentServiceError::NotFound(format!(
                    "Wallet not found for user_id: {}",
                    txn.user_id
                ))
            })?;

        self.insert_wallet_transaction(
            &mut tx,
            &wallet.id,
            txn.request_amount,
            &txn.id.to_string(),
        )
        .await?;

        self.update_wallet_balance(&mut tx, &wallet.id, txn.request_amount)
            .await?;

        self.log_webhook_event(&mut tx, payload).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        if let Err(e) = self
            .sync_redis_balance(&txn.user_id, txn.request_amount)
            .await
        {
            tracing::error!("Failed update wallet cache {}: {}.", txn.user_id, e);
        }

        Ok(())
    }

    pub async fn handle_pr_expiry(
        &self,
        payload: &XenditPaymentRequestData,
    ) -> Result<(), ShipmentServiceError> {
        let mut tx = self.db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        let txn = self
            .get_transaction_by_external_id(&mut tx, &payload.payment_request_id)
            .await?
            .ok_or_else(|| {
                ShipmentServiceError::NotFound(format!(
                    "Transaction not found for payment_request id: {}",
                    payload.payment_request_id
                ))
            })?;

        self.update_transaction_status(&mut tx, txn.id, "EXPIRED")
            .await?;

        self.log_webhook_event(&mut tx, payload).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(())
    }

    async fn get_transaction_by_external_id(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        external_id: &str,
    ) -> Result<Option<TransactionRow>, ShipmentServiceError> {
        let row = sqlx::query_as::<_, TransactionRow>(
            r#"SELECT id, user_id, request_amount, status
               FROM transactions
               WHERE external_payment_id = $1"#,
        )
        .bind(external_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(row)
    }

    async fn update_transaction_status(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        status: &str,
    ) -> Result<(), ShipmentServiceError> {
        let paid_at = if status == "PAID" {
            Some(Utc::now())
        } else {
            None
        };

        let res = sqlx::query(
            r#"UPDATE transactions
               SET status = $1::transaction_status, paid_at = COALESCE($2, paid_at)
               WHERE id = $3"#,
        )
        .bind(status)
        .bind(paid_at)
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update transaction status: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        if res.rows_affected() == 0 {
            return Err(ShipmentServiceError::NotFound(format!(
                "Transaction {} not found during update",
                id
            )));
        }

        Ok(())
    }

    async fn get_wallet_by_user_id(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: &Uuid,
    ) -> Result<Option<WalletRow>, ShipmentServiceError> {
        let row = sqlx::query_as::<_, WalletRow>(
            "SELECT id, user_id, balance FROM wallets WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query wallet: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(row)
    }

    async fn insert_wallet_transaction(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        wallet_id: &Uuid,
        amount: i64,
        reference_id: &str,
    ) -> Result<(), ShipmentServiceError> {
        sqlx::query(
            r#"INSERT INTO wallet_transactions (wallet_id, type, amount, reference_id, created_at)
               VALUES ($1, 'TOPUP'::wallet_transaction, $2, $3, now())"#,
        )
        .bind(wallet_id)
        .bind(amount)
        .bind(reference_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert wallet_transaction: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(())
    }

    async fn update_wallet_balance(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        wallet_id: &Uuid,
        amount: i64,
    ) -> Result<(), ShipmentServiceError> {
        let res = sqlx::query(
            "UPDATE wallets SET balance = balance + $1, updated_at = now() WHERE id = $2",
        )
        .bind(amount)
        .bind(wallet_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update wallet balance: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        if res.rows_affected() == 0 {
            return Err(ShipmentServiceError::NotFound(format!(
                "Wallet {} not found during balance update",
                wallet_id
            )));
        }

        Ok(())
    }

    async fn log_webhook_event<T: serde::Serialize>(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &T,
    ) -> Result<(), ShipmentServiceError> {
        sqlx::query(
            "INSERT INTO webhook_logs (payload, processed_at, created_at) VALUES ($1, $2, $3)",
        )
        .bind(Json(payload))
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to log webhook event: {}", e);
            ShipmentServiceError::Unexpected(e.into())
        })?;

        Ok(())
    }

    async fn sync_redis_balance(
        &self,
        user_id: &Uuid,
        amount: i64,
    ) -> Result<(), ShipmentServiceError> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            ShipmentServiceError::Unexpected(anyhow::anyhow!("Redis pool error: {}", e))
        })?;

        let key = format!("wallet_balance:{}", user_id);
        conn.incr::<_, _, i64>(&key, amount).await.map_err(|e| {
            ShipmentServiceError::Unexpected(anyhow::anyhow!("Redis INCRBY error: {}", e))
        })?;

        Ok(())
    }
}
