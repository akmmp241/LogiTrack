use crate::PaymentProvider;
use crate::domain::dto::{
    GetPaymentRes, ProcessPaymentReq, ProcessPaymentRes, XenditGetPaymentRequestResponse,
    XenditPaymentRequestPayRequest, XenditPaymentRequestPayResponse, XenditSimulateResponse,
};
use crate::domain::xendit::{
    XenditActionType, XenditAvailableChannel, XenditChannelProperties, XenditError,
};
use crate::errors::{PaymentError, XenditErrorType};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{Duration, Utc};
use serde::de::DeserializeOwned;
use serde_json::json;
use url::Url;

const QRIS_CHARGE: f64 = 0.007;
const EWALLET_SERVICE_CHARGE: f64 = 0.04;
const VIRTUAL_ACCOUNT_SERVICE_CHARGE: i32 = 4000;
const OTC_SERVICE_CHARGE: i32 = 5500;

#[derive(Clone)]
pub struct XenditProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

#[async_trait::async_trait]
impl PaymentProvider for XenditProvider {
    async fn process_payment(
        &self,
        req_val: serde_json::Value,
    ) -> Result<serde_json::Value, PaymentError> {
        let req = serde_json::from_value::<ProcessPaymentReq>(req_val).unwrap();

        let (channel_code, channel_properties_opt) =
            self.validate_payment_method(&req.channel_code)?;

        let charge = self.get_service_charge(&channel_code, req.request_amount);

        let mut channel_properties = channel_properties_opt
            .ok_or_else(|| PaymentError::BadRequest("Missing channel properties".into()))?;

        channel_properties
            .with_user_name(&req.user_name)
            .with_mobile_number(&req.mobile_number);

        let mut xendit_req = XenditPaymentRequestPayRequest::from_req(req, channel_properties)?;
        xendit_req.request_amount += charge;

        let url = format!("{}/v3/payment_requests", self.base_url);

        let credentials = format!("{}:", &self.api_key);
        let encoded_credentials = BASE64_STANDARD.encode(credentials);

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .header("api-version", "2024-11-11")
            .json(&xendit_req)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "error occurred while call payment-request to xendit: {}",
                    e.to_string()
                );
                PaymentError::Unexpected()
            })?;

        let xendit_res = self
            .handle_response::<XenditPaymentRequestPayResponse>(response)
            .await?;

        let res = {
            let mut temp = ProcessPaymentRes::from(xendit_res);
            temp.charge_amount = charge;
            temp.total_amount = xendit_req.request_amount;
            temp
        };

        let res = serde_json::to_value(&res).map_err(|e| {
            tracing::error!(
                "error occurred while serializing xendit response json: {}",
                e
            );
            PaymentError::Unexpected()
        })?;

        Ok(res)
    }

    async fn get_payment_details(
        &self,
        payment_id: String,
    ) -> Result<serde_json::Value, PaymentError> {
        let url = format!("{}/v3/payment_requests/{}", self.base_url, payment_id);

        let credentials = format!("{}:", &self.api_key);
        let encoded_credentials = BASE64_STANDARD.encode(credentials);

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .header("api-version", "2024-11-11")
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "error occurred while call payment-request to xendit: {}",
                    e.to_string()
                );
                PaymentError::Unexpected()
            })?;

        let xendit_res = self
            .handle_response::<XenditGetPaymentRequestResponse>(response)
            .await?;

        let payment_res = GetPaymentRes {
            reference_id: xendit_res.reference_id,
            external_id: xendit_res.payment_request_id,
            status: xendit_res.status,
            total_amount: xendit_res.request_amount,
            actions: xendit_res.actions,
        };

        let res = serde_json::to_value(&payment_res).map_err(|e| {
            tracing::error!(
                "error occurred while serializing get payment response json: {}",
                e.to_string()
            );
            PaymentError::Unexpected()
        })?;

        Ok(res)
    }

    async fn simulate_payment(
        &self,
        payment_id: String,
        amount: Option<i64>,
    ) -> Result<bool, PaymentError> {
        let pr_val = self.get_payment_details(payment_id).await?;

        let pr = serde_json::from_value::<GetPaymentRes>(pr_val).map_err(|e| {
            tracing::error!("error occurred while deserializing payment details: {}", e);
            PaymentError::Unexpected()
        })?;

        if pr.status == "SUCCEEDED" {
            return Err(PaymentError::BadRequest("Payment already succeeded".into()));
        }

        if pr.status == "FAILED" {
            return Err(PaymentError::BadRequest("Payment already failed".into()));
        }

        if pr.actions.is_empty() {
            return Ok(false);
        }

        match pr.actions.first().unwrap().type_ {
            XenditActionType::PresentToCustomer => {
                self.handle_present(pr.external_id.as_str(), amount.unwrap_or_default())
                    .await?;
            }
            XenditActionType::RedirectCustomer => {
                let url_action = &pr.actions.first().unwrap().value;
                self.handle_redirect(url_action).await?;
            }
        };

        Ok(true)
    }
}

impl Default for XenditProvider {
    fn default() -> Self {
        Self::new(reqwest::Client::new())
    }
}

impl XenditProvider {
    pub fn new(client: reqwest::Client) -> Self {
        let api_key = std::env::var("XENDIT_API_KEY").expect("XENDIT_API_KEY must be set");
        let base_url = std::env::var("XENDIT_BASE_URL").expect("XENDIT_BASE_URL must be set");

        Self {
            client,
            api_key,
            base_url,
        }
    }

    fn map_error(&self, e: XenditError) -> PaymentError {
        match e.error_code {
            XenditErrorType::ChannelUnavailable => PaymentError::ProviderError(e.message),
            XenditErrorType::IssuerUnavailable => PaymentError::ProviderError(e.message),
            XenditErrorType::DataNotFound => PaymentError::DuplicatedPayment(),
            XenditErrorType::InvalidValueError => PaymentError::BadRequest(e.message),
            XenditErrorType::ApiValidationError => PaymentError::BadRequest(e.message),
            XenditErrorType::InvalidPaymentDetails => PaymentError::BadRequest(e.message),
        }
    }

    fn validate_payment_method(
        &self,
        channel_code: &str,
    ) -> Result<(XenditAvailableChannel, Option<XenditChannelProperties>), PaymentError> {
        let channel = channel_code.parse::<XenditAvailableChannel>()?;
        let mut properties = channel.default_properties();

        match channel {
            XenditAvailableChannel::Qris => {
                properties.with_expires_at(Utc::now() + Duration::hours(1));
            }
            XenditAvailableChannel::Alfamart | XenditAvailableChannel::Indomaret => {
                properties.with_expires_at(Utc::now() + Duration::hours(6));
            }
            _ => {
                properties
                    .with_success_return_url("https://xendit.co/success")
                    .with_cancel_return_url("https://xendit.co/cancel")
                    .with_failure_return_url("https://xendit.co/failure");
            }
        }

        Ok((channel, Some(properties)))
    }

    async fn handle_response<T: Sized + DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, PaymentError> {
        if response.status().is_server_error() || response.status().is_client_error() {
            let xendit_err = response.json::<XenditError>().await.map_err(|e| {
                tracing::error!("error while deserializing xendit error: {}", e.to_string());
                PaymentError::Unexpected()
            })?;

            return Err(self.map_error(xendit_err));
        }

        let xendit_res = response.json::<T>().await.map_err(|e| {
            tracing::error!(
                "error while deserializing xendit response: {}",
                e.to_string()
            );
            PaymentError::Unexpected()
        })?;

        Ok(xendit_res)
    }

    async fn handle_redirect(&self, url_action: &str) -> Result<(), PaymentError> {
        let parsed_url = Url::parse(url_action).map_err(|e| {
            tracing::error!("failed to parse url action {}: {}", url_action, e);
            PaymentError::Unexpected()
        })?;

        let mut payment_token_opt: Option<String> = None;

        for (key, value) in parsed_url.query_pairs() {
            if key == "token" {
                payment_token_opt = Some(value.to_string())
            }
        }

        let payment_token = payment_token_opt.ok_or_else(|| {
            tracing::error!("missing token parameter: {}", url_action);
            PaymentError::BadRequest("missing token".to_string())
        })?;

        let url_simulate = format!(
            "https://ewallet-mock-connector.xendit.co/v1/ewallet_connector/payment_callbacks?token={}",
            payment_token
        );

        let response = self
            .client
            .post(url_simulate)
            .timeout(Duration::seconds(15).to_std().unwrap())
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "error occurred while call xendit ewallet simulate payment: {}",
                    e.to_string()
                );
                PaymentError::Unexpected()
            })?;

        if !response.status().is_success() {
            tracing::error!("failed to simulate payment");
            return Err(PaymentError::ProviderError(
                "provider failed to simulate payment".to_string(),
            ));
        }

        let res = response
            .json::<XenditSimulateResponse>()
            .await
            .map_err(|e| {
                tracing::error!(
                    "error while deserializing xendit simulate response: {}",
                    e.to_string()
                );
                PaymentError::Unexpected()
            })?;

        if res.status != "SUCCEEDED" {
            tracing::error!("unexpected status: {}", res.status);
            return Err(PaymentError::Unexpected());
        }

        Ok(())
    }

    async fn handle_present(&self, pr_id: &str, amount: i64) -> Result<(), PaymentError> {
        let url = format!("{}/v3/payment_requests/{}/simulate", self.base_url, pr_id);

        let credentials = format!("{}:", &self.api_key);
        let encoded_credentials = BASE64_STANDARD.encode(credentials);

        let req = json!({"amount": amount});

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Basic {}", encoded_credentials))
            .header("api-version", "2024-11-11")
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    "error occurred while call payment simulate to xendit: {}",
                    e.to_string()
                );
                PaymentError::Unexpected()
            })?;

        if !response.status().is_success() {
            tracing::error!("failed to simulate payment");
            return Err(PaymentError::Unexpected());
        }

        Ok(())
    }

    fn get_service_charge(&self, channel: &XenditAvailableChannel, request_amount: i64) -> i64 {
        let charge: i32;

        match channel {
            XenditAvailableChannel::Qris => {
                charge = (request_amount as f64 * QRIS_CHARGE).ceil() as i32;
            }
            XenditAvailableChannel::Alfamart | XenditAvailableChannel::Indomaret => {
                charge = OTC_SERVICE_CHARGE;
            }
            XenditAvailableChannel::BCAVirtualAccount
            | XenditAvailableChannel::BniVirtualAccount
            | XenditAvailableChannel::BriVirtualAccount
            | XenditAvailableChannel::MandiriVirtualAccount => {
                charge = VIRTUAL_ACCOUNT_SERVICE_CHARGE;
            }
            XenditAvailableChannel::DANA
            | XenditAvailableChannel::GOPAY
            | XenditAvailableChannel::LINKAJA
            | XenditAvailableChannel::OVO
            | XenditAvailableChannel::SHOPEEPAY => {
                charge = (request_amount as f64 * EWALLET_SERVICE_CHARGE).ceil() as i32;
            }
        }

        charge as i64
    }
}
