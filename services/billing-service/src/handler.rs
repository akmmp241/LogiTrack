use crate::domain::BillingDeductionEvent;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
struct WalletRow {
    pub id: Uuid,
}

pub struct BillingHandler {
    pool: PgPool,
}

impl BillingHandler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn handle(&self, event: &BillingDeductionEvent) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!("failed to begin transaction: {}", e);
            anyhow::anyhow!("db transaction error: {}", e)
        })?;

        let wallet = sqlx::query_as::<_, WalletRow>("SELECT id FROM wallets WHERE user_id = $1")
            .bind(event.user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("failed to query wallet: {}", e);
                anyhow::anyhow!("wallet query error: {}", e)
            })?
            .ok_or_else(|| {
                tracing::error!(user_id = %event.user_id, "wallet not found");
                anyhow::anyhow!("wallet not found for user_id: {}", event.user_id)
            })?;

        sqlx::query(
            r#"INSERT INTO wallet_transactions (wallet_id, type, amount, reference_id, created_at)
               VALUES ($1, 'AWB_CREATION'::wallet_transaction, $2, $3, $4)"#,
        )
        .bind(wallet.id)
        .bind(-event.amount) // negative because it's a deduction
        .bind(event.shipment_id.to_string())
        .bind(event.deducted_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("failed to insert wallet_transaction: {}", e);
            anyhow::anyhow!("wallet_transaction insert error: {}", e)
        })?;

        let res = sqlx::query(
            "UPDATE wallets SET balance = balance - $1, updated_at = now() WHERE id = $2",
        )
        .bind(event.amount)
        .bind(wallet.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("failed to update wallet balance: {}", e);
            anyhow::anyhow!("wallet balance update error: {}", e)
        })?;

        if res.rows_affected() == 0 {
            return Err(anyhow::anyhow!(
                "wallet {} not found during balance update",
                wallet.id
            ));
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("failed to commit transaction: {}", e);
            anyhow::anyhow!("db commit error: {}", e)
        })?;

        tracing::info!(
            user_id = %event.user_id,
            shipment_id = %event.shipment_id,
            amount = event.amount,
            "billing deduction persisted to database"
        );

        Ok(())
    }
}
