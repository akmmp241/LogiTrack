use chrono::{Datelike, Local};
use redis::AsyncCommands;
use uuid::Uuid;

pub async fn increment_awb_count(
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
) -> anyhow::Result<i64> {
    let now = Local::now();
    let year = now.year();
    let month = now.month();

    let awb_key = format!("awb_count:{}:{}_{}", user_id, year, month);

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!("failed to get redis connection: {}", e);
        anyhow::anyhow!("redis pool error: {}", e)
    })?;

    let new_count: i64 = conn.incr(&awb_key, 1).await.map_err(|e| {
        tracing::error!("failed to INCR awb count: {}", e);
        anyhow::anyhow!("redis INCR error: {}", e)
    })?;

    // set expire only first creation
    if new_count == 1 {
        // 35 hari
        let _: () = conn.expire(&awb_key, 3024000).await.map_err(|e| {
            tracing::error!("failed to set EXPIRE on awb key: {}", e);
            anyhow::anyhow!("redis EXPIRE error: {}", e)
        })?;
    }

    Ok(new_count)
}
