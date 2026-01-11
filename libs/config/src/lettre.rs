use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, Tokio1Executor};

pub async fn create_smtp_transport()
-> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
    let smtp_host = std::env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let smtp_port = std::env::var("SMTP_PORT").expect("SMTP_PORT must be set");
    let smtp_username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = std::env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");

    let credentials = Credentials::new(smtp_username, smtp_password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
        .credentials(credentials)
        .port(
            smtp_port
                .parse::<u16>()
                .expect("SMTP_PORT must be a number"),
        )
        .build();

    Ok(mailer)
}
