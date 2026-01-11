use std::time::Duration;
use reqwest::Client;

pub fn get_reqwest_pool() -> Result<Client, reqwest::Error> {
    let pool = Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(30))
        .build()?;

    Ok(pool)
}
