use crate::domain::notification_price::NotificationPrice;
use sqlx::PgPool;

#[derive(Clone)]
pub struct NotificationPriceRepository {
    pool: PgPool,
}

impl NotificationPriceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all(&self) -> Result<Vec<NotificationPrice>, sqlx::Error> {
        let prices = sqlx::query_as!(
            NotificationPrice,
            r#"
            SELECT id, field, value
            FROM notification_prices
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(prices)
    }
}
