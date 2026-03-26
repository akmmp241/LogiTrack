use crate::domain::NotificationChannel;
use chrono::{Datelike, Local};
use redis::Script;
use uuid::Uuid;

#[derive(Debug)]
pub enum BillingResult {
    Success {
        price_deducted: i64,
        remaining_balance: i64,
    },
    InsufficientBalance {
        required_price: i64,
        current_balance: i64,
    },
    PriceNotFound,
}

/// Thanks to AI for this implementation :D
///
/// KEYS[1] = wallet_balance:{user_id}
/// KEYS[2] = awb_count:{user_id}:{year}_{month}
/// KEYS[3] = NotificationChannel
///
/// Returns array: [status_code, price, balance]
///   status_code  1 = success, -1 = insufficient balance
const BILLING_LUA_SCRIPT: &str = r#"
local current_awb = tonumber(redis.call('GET', KEYS[2]) or 0)

local price = tonumber(redis.call('HGET', 'pricing:notifications', KEYS[3]))

if not price then
    return { -2, 0, 0 }
end

local current_balance = tonumber(redis.call('GET', KEYS[1]) or 0)

if current_balance < price then
    return { -1, price, current_balance }
end

redis.call('DECRBY', KEYS[1], price)

return { 1, price, current_balance - price }
"#;

pub async fn check_and_deduct_balance(
    redis_pool: &deadpool_redis::Pool,
    user_id: &Uuid,
    channel: &NotificationChannel,
) -> anyhow::Result<BillingResult> {
    let now = Local::now();
    let year = now.year();
    let month = now.month();

    let balance_key = format!("wallet_balance:{}", user_id);
    let awb_key = format!("awb_count:{}:{}_{}", user_id, year, month);

    let mut conn = redis_pool.get().await.map_err(|e| {
        tracing::error!("failed to get redis connection: {}", e);
        anyhow::anyhow!("redis pool error: {}", e)
    })?;

    let script = Script::new(BILLING_LUA_SCRIPT);

    let result: Vec<i64> = script
        .key(&balance_key)
        .key(&awb_key)
        .key(channel)
        .invoke_async(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute billing lua script: {}", e);
            anyhow::anyhow!("lua script error: {}", e)
        })?;

    if result.len() != 3 {
        return Err(anyhow::anyhow!(
            "unexpected lua script result length: {}",
            result.len()
        ));
    }

    let status_code = result[0];
    let price = result[1];
    let balance = result[2];

    match status_code {
        1 => Ok(BillingResult::Success {
            price_deducted: price,
            remaining_balance: balance,
        }),
        -1 => Ok(BillingResult::InsufficientBalance {
            required_price: price,
            current_balance: balance,
        }),
        -2 => Ok(BillingResult::PriceNotFound),
        _ => Err(anyhow::anyhow!(
            "unexpected billing status code: {}",
            status_code
        )),
    }
}
