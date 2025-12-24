use tracing_subscriber::fmt;

pub fn init(service_name: &str) {
    fmt()
        .init();

    tracing::info!("{} is initialized", service_name);
}