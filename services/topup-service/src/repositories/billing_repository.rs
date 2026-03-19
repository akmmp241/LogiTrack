use crate::domain::wallet::{PricingTier, Wallet, WalletTransaction, WalletTransactionType};
use sqlx::{Error, PgExecutor, PgPool};
use uuid::Uuid;

#[derive(Clone)]
pub struct BillingRepository {
    pub pool: PgPool,
}

impl BillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn update_balance(
        &self,
        executor: impl PgExecutor<'_>,
        new_balance: &i64,
    ) -> Result<bool, Error> {
        let res = sqlx::query(
            r#"
            UPDATE wallets SET balance = balance + $1
            "#,
        )
        .bind(new_balance)
        .execute(executor)
        .await?;

        Ok(res.rows_affected() == 1)
    }

    pub async fn get_wallet_by_user_id(
        &self,
        executor_opt: impl PgExecutor<'_>,
        user_id: &Uuid,
    ) -> Result<Option<Wallet>, Error> {
        let res = sqlx::query_as::<_, Wallet>(
            r#"
                SELECT id, user_id, balance, updated_at FROM wallets WHERE user_id = $1
                "#,
        )
        .bind(user_id)
        .fetch_optional(executor_opt)
        .await?;

        Ok(res)
    }

    pub async fn save_wallet_transaction(
        &self,
        executor: impl PgExecutor<'_>,
        wallet_id: &Uuid,
        type_: &Option<WalletTransactionType>,
        amount: &i64,
        reference_id: &Option<String>,
    ) -> Result<bool, Error> {
        let res = sqlx::query(
            r#"
            INSERT INTO wallet_transactions (wallet_id, type, amount, reference_id, created_at)
            VALUES ($1, $2, $3, $4, now())
            "#,
        )
        .bind(wallet_id)
        .bind(type_)
        .bind(amount)
        .bind(reference_id)
        .execute(executor)
        .await?;

        Ok(res.rows_affected() == 1)
    }

    pub async fn get_wallet_transactions(
        &self,
        executor: impl PgExecutor<'_>,
        wallet_id: &Uuid,
    ) -> Result<Vec<WalletTransaction>, Error> {
        let res = sqlx::query_as::<_, WalletTransaction>(
            r#"
            SELECT id, wallet_id, type, amount, reference_id, created_at
            FROM wallet_transactions
            WHERE wallet_id = $1
            "#,
        )
        .bind(wallet_id)
        .fetch_all(executor)
        .await?;

        Ok(res)
    }

    pub async fn get_tier_by_volume(
        &self,
        executor: impl PgExecutor<'_>,
        min: &i32,
    ) -> Result<Option<PricingTier>, Error> {
        let res = sqlx::query_as::<_, PricingTier>(
            r#"
                SELECT id, tier_level, min_volume, max_volume, price
                FROM pricing_tiers
                WHERE min_volume <= $1
                  AND (max_volume >= $1 OR max_volume IS NULL)
                LIMIT 1;
                "#,
        )
        .bind(min)
        .fetch_optional(executor)
        .await?;

        Ok(res)
    }
}
