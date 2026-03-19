mod app;
mod domain;
mod handlers;
mod middleware;
pub mod repositories;
mod router;
pub mod services;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    observability::init("topup-service");

    app::App::new().await.run().await
}
