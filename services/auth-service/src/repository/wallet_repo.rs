use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct WalletRepository {
    pool: PgPool,
}

impl WalletRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, user_id: &Uuid) -> Result<bool, sqlx::Error> {
        let res = sqlx::query(r#"INSERT INTO wallets (user_id) VALUES ($1)"#)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected() == 1)
    }
}
