pub mod biteship_handler;
pub mod xendit_handler;

pub trait DefaultHandler {
    fn get_webhook_key(&self) -> &str;
    fn get_webhook_secret(&self) -> &str;
}
