use crate::domain::xendit::{XenditActions, XenditChannelProperties};
use crate::errors::PaymentError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPaymentReq {
    pub reference_id: Uuid,
    pub request_amount: i64,
    pub channel_code: String,
    pub user_name: String,
    pub mobile_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPaymentRes {
    pub reference_id: Uuid,
    pub external_id: String,
    pub status: String,
    pub charge_amount: i64,
    pub total_amount: i64,
    pub failure_code: Option<String>,
    pub actions: Vec<XenditActions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPaymentRes {
    pub reference_id: Uuid,
    pub external_id: String,
    pub status: String,
    pub total_amount: i64,
    pub actions: Vec<XenditActions>,
}

impl From<XenditPaymentRequestPayResponse> for ProcessPaymentRes {
    fn from(x: XenditPaymentRequestPayResponse) -> Self {
        Self {
            reference_id: x.reference_id,
            external_id: x.payment_request_id,
            status: x.status,
            failure_code: x.failure_code,
            charge_amount: 0,
            total_amount: 0,
            actions: x.actions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentRequestPayRequest {
    pub reference_id: Uuid,
    #[serde(rename = "type")]
    type_: String,
    country: String,
    currency: String,
    pub channel_code: String,
    pub channel_properties: serde_json::Value,
    pub request_amount: i64,
    capture_method: String,
}

impl XenditPaymentRequestPayRequest {
    pub fn from_req(
        x: ProcessPaymentReq,
        properties: XenditChannelProperties,
    ) -> Result<XenditPaymentRequestPayRequest, PaymentError> {
        Ok(Self {
            reference_id: x.reference_id,
            type_: "PAY".into(),
            country: "ID".to_string(),
            currency: "IDR".to_string(),
            channel_code: x.channel_code,
            channel_properties: properties.to_value(),
            request_amount: x.request_amount,
            capture_method: "AUTOMATIC".to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentRequestPayResponse {
    pub reference_id: Uuid,
    pub payment_request_id: String,
    pub actions: Vec<XenditActions>,
    pub failure_code: Option<String>,
    pub status: String,
    #[serde(rename = "updated")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditGetPaymentRequestResponse {
    pub reference_id: Uuid,
    pub payment_request_id: String,
    pub actions: Vec<XenditActions>,
    pub request_amount: i64,
    pub status: String,
    #[serde(rename = "updated")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditGetPaymentResponse {
    pub payment_request_id: String,
    pub reference_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditSimulateResponse {
    pub status: String,
}
