use lapin::{Channel, Connection, ConnectionProperties};
use std::sync::Arc;

pub async fn get_connection() -> Result<Arc<Connection>, lapin::Error> {
    let user = std::env::var("RABBITMQ_USER").expect("RABBITMQ_USER not set");
    let pass = std::env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD not set");
    let host = std::env::var("RABBITMQ_HOST").expect("RABBITMQ_HOST not set");
    let port = std::env::var("RABBITMQ_PORT").expect("RABBITMQ_PORT not set");

    let dsn = format!("amqp://{}:{}@{}:{}/%2f", user, pass, host, port);

    let conn = Connection::connect(dsn.as_str(), ConnectionProperties::default()).await?;

    Ok(Arc::new(conn))
}

pub async fn create_channel() -> Result<Channel, lapin::Error> {
    let conn = get_connection().await?;

    let channel = (&conn).create_channel().await?;

    Ok(channel)
}
