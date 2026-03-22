use crate::domain::xendit::{XenditPaymentRequestWebhookPayload, XenditPaymentWebhookPayload};
use crate::handlers::DefaultHandler;
use crate::services::xendit_service::XenditService;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use errors::error::HttpError;
use std::env;
use std::sync::Arc;

#[derive(Clone)]
pub struct XenditHandler {
    webhook_key: String,
    webhook_secret: String,
    service: Arc<XenditService>,
}

impl DefaultHandler for XenditHandler {
    fn get_webhook_key(&self) -> &str {
        &self.webhook_key
    }

    fn get_webhook_secret(&self) -> &str {
        &self.webhook_secret
    }
}

impl XenditHandler {
    pub fn new(service: Arc<XenditService>) -> Self {
        let secret_key = env::var("XENDIT_WEBHOOK_KEY").expect("XENDIT_WEBHOOK_KEY must be set");
        let secret = env::var("XENDIT_WEBHOOK_TOKEN").expect("XENDIT_WEBHOOK_TOKEN must be set");

        Self {
            webhook_key: secret_key,
            webhook_secret: secret,
            service,
        }
    }

    pub async fn payment_capture(
        State(handler): State<Arc<XenditHandler>>,
        req: Result<Json<XenditPaymentWebhookPayload>, JsonRejection>,
    ) -> Result<impl IntoResponse, HttpError> {
        let Json(data) = req?;

        match handler.service.handle_payment_capture(&data.data).await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn pr_expiry(
        State(handler): State<Arc<XenditHandler>>,
        req: Result<Json<XenditPaymentRequestWebhookPayload>, JsonRejection>,
    ) -> Result<impl IntoResponse, HttpError> {
        let Json(data) = req?;

        match handler.service.handle_pr_expiry(&data.data).await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(e.into()),
        }
    }
}
