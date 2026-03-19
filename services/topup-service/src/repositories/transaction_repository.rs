use crate::domain::transaction::Transaction;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        user_id: &Uuid,
        transaction: &Transaction,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(
            r#"
                INSERT INTO transactions (id, user_id, request_amount, charge_amount,
                                          total_amount, payment_method, external_payment_id,
                                          status, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
                "#,
        )
        .bind(transaction.id)
        .bind(user_id)
        .bind(transaction.request_amount)
        .bind(transaction.charge_amount)
        .bind(transaction.total_amount)
        .bind(transaction.payment_method.clone())
        .bind(transaction.external_payment_id.clone())
        .bind(transaction.status.clone())
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected() == 1)
    }

    pub async fn get_transactions(&self, user_id: &Uuid) -> Result<Vec<Transaction>, sqlx::Error> {
        let res = sqlx::query_as::<_, Transaction>(
            r#"
                SELECT id, request_amount, charge_amount,
                       total_amount, status, payment_method,
                       external_payment_id, metadata, paid_at,
                       created_at, external_payment_id
                FROM transactions
                WHERE user_id = $1
                "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn get_transaction(
        &self,
        user_id: &Uuid,
        id: &Uuid,
    ) -> Result<Option<Transaction>, sqlx::Error> {
        let res = sqlx::query_as::<_, Transaction>(
            r#"
                SELECT id, request_amount,
                       charge_amount, total_amount, status,
                       payment_method, external_payment_id
                       metadata, paid_at, created_at,
                       external_payment_id
                FROM transactions
                WHERE user_id = $1 AND id = $2
                "#,
        )
        .bind(user_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }
}
