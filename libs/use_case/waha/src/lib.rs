mod dto;
mod error;

use crate::dto::{SendTextReq, WahaClientErrorResponse};
use crate::error::WahaError;
use reqwest::header;
use std::env;

pub struct WahaUseCase {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    session: String,
}

impl WahaUseCase {
    pub fn new(client: reqwest::Client) -> Self {
        let base_url = env::var("WAHA_API_URL").expect("BITESHIP_API_URL must be set");
        let api_key = env::var("WAHA_API_KEY").expect("BITESHIP_API_KEY must be set");
        let session = env::var("WAHA_SESSION").expect("WAHA_SESSION must be set");
        Self {
            client,
            base_url,
            api_key,
            session,
        }
    }

    pub async fn send_text(&self, id: String, text: String) -> Result<(), WahaError> {
        let url = format!("{}/api/sendText", self.base_url);

        let payload = SendTextReq {
            chat_id: format!("{}@c.us", id),
            text,
            session: self.session.clone(),
        };

        let resp = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Api-Key", self.api_key.as_str())
            .header(header::ACCEPT, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| WahaError::InternalServerError(e.into()))?;

        self.handle_response(resp).await
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<(), WahaError> {
        if resp.status().is_server_error() {
            let waha_err = resp
                .json::<WahaClientErrorResponse>()
                .await
                .map_err(|e| WahaError::InternalServerError(anyhow::anyhow!(e)))?;

            return Err(WahaError::InternalServerError(anyhow::anyhow!(
                waha_err.message
            )));
        }

        if resp.status().is_client_error() {
            let waha_err = resp
                .json::<WahaClientErrorResponse>()
                .await
                .map_err(|e| WahaError::InternalServerError(anyhow::anyhow!(e)))?;

            if waha_err.status_code == 401 {
                return Err(WahaError::Unauthorized);
            }

            return Err(WahaError::ClientError(waha_err.message));
        }

        Ok(())
    }
}
