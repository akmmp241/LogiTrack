use crate::domain::notification_price::NotificationPrice;
use crate::repositories::notification_price_repo::NotificationPriceRepository;
use deadpool_redis::Pool;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct NotificationPriceService {
    repo: NotificationPriceRepository,
    redis_pool: Pool,
}

impl NotificationPriceService {
    pub fn new(repo: NotificationPriceRepository, redis_pool: Pool) -> Self {
        Self { repo, redis_pool }
    }

    pub async fn rehydrate_cache(&self) -> anyhow::Result<Vec<NotificationPrice>> {
        let prices = self
            .repo
            .get_all()
            .await
            .map_err(|e| anyhow::anyhow!("db error: {}", e))?;

        if prices.is_empty() {
            return Ok(prices);
        }

        let mut conn = self.redis_pool.get().await?;

        let hash_args: Vec<(String, i32)> =
            prices.iter().map(|p| (p.field.clone(), p.value)).collect();

        let _: () = conn
            .hset_multiple("pricing:notifications", &hash_args)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to set cache: {}", e))?;

        Ok(prices)
    }
}
