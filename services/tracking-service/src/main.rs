use crate::app::App;
use dotenvy::dotenv;

mod app;
mod handlers;
mod helper;
mod middlewares;
mod models;
mod repository;
mod routes;
mod service;

#[tokio::main]
async fn main() {
    dotenv().ok();
    observability::init("tracking-service");

    App::new().await.run().await;
}
