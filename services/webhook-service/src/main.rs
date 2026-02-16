mod app;
mod domain;
mod errors;
mod handlers;
mod middlewares;
mod routes;
mod services;

use crate::app::App;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    observability::init("webhook-service");

    App::new().await.run().await;
}
