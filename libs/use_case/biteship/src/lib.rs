use crate::dto::errors::BiteshipError;
use crate::dto::tracking::BiteshipTrackingResponse;
use errors::error::HttpError;
use errors::error::HttpError::{BadRequest, NotFound};
use std::env;

pub mod dto;
pub mod error;

#[derive(Clone)]
pub struct BiteshipUseCase {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl BiteshipUseCase {
    pub fn new(client: reqwest::Client) -> Self {
        let base_url = env::var("BITESHIP_API_URL").expect("BITESHIP_API_URL must be set");
        let api_key = env::var("BITESHIP_API_KEY_TEST").expect("BITESHIP_API_KEY must be set");
        Self {
            client,
            base_url,
            api_key,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    pub async fn fetch_public_tracking(
        &self,
        awb: String,
        courier_code: String,
    ) -> Result<BiteshipTrackingResponse, HttpError> {
        let url = format!(
            "{}/v1/trackings/{}/couriers/{}",
            self.base_url, awb, courier_code
        );

        let resp = self
            .client
            .get(url)
            .header("Authorization", self.api_key.clone())
            .send()
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        self.handle_response(resp).await
    }

    async fn handle_response(
        &self,
        resp: reqwest::Response,
    ) -> Result<BiteshipTrackingResponse, HttpError> {
        if (&resp).status().is_server_error() {
            return Err(HttpError::InternalServerError(anyhow::anyhow!(
                "server error"
            )));
        }

        if (&resp).status().is_client_error() {
            let bs_err = resp
                .json::<BiteshipError>()
                .await
                .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e)))?;

            return Err(self.map_biteship_error(bs_err));
        }

        resp.json::<BiteshipTrackingResponse>()
            .await
            .map_err(|e| HttpError::InternalServerError(anyhow::anyhow!(e)))
    }

    fn map_biteship_error(&self, bs_err: BiteshipError) -> HttpError {
        match bs_err.error.as_str() {
            "40003003" => {
                NotFound("Waybill not found. It's either not activated or expired".to_string())
            }
            _ => BadRequest(bs_err.error),
        }
    }
}
