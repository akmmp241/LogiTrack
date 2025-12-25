#[tokio::main]
async fn main() {
    observability::init("tracking-service");

    println!("Hello, world!");
}
