use crate::domain::biteship::BiteshipChangeStatusEvent;
use crate::services::biteship_service::BiteshipService;
use Result;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use errors::error::HttpError;
use std::env;
use std::sync::Arc;

pub struct BiteshipHandler {
    webhook_key: String,
    webhook_secret: String,
    service: Arc<BiteshipService>,
}

impl BiteshipHandler {
    pub fn new(service: Arc<BiteshipService>) -> Self {
        let secret_key = env::var("BITESHIP_WEBHOOK_KEY").expect("BITESHIP_SECRET_KEY must be set");
        let secret = env::var("BITESHIP_WEBHOOK_SECRET").expect("BITESHIP_SECRET must be set");

        Self {
            webhook_key: secret_key,
            webhook_secret: secret,
            service,
        }
    }

    pub fn get_webhook_key(&self) -> &str {
        &self.webhook_key
    }

    pub fn get_webhook_secret(&self) -> &str {
        &self.webhook_secret
    }

    pub async fn status_change(
        State(handler): State<Arc<BiteshipHandler>>,
        payload: Result<Json<BiteshipChangeStatusEvent>, JsonRejection>,
    ) -> Result<impl IntoResponse, HttpError> {
        let Json(data) = payload?;

        let event_type = &data.event.to_string();

        match handler.service.handle_event(event_type, &data).await {
            Ok(_) => Ok(StatusCode::OK),
            Err(e) => Err(e.into()),
        }
    }
}
