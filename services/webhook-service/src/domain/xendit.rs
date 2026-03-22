use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentWebhookPayload {
    pub event: String,
    pub data: XenditPaymentData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentData {
    pub payment_id: String,
    pub status: String,
    pub payment_request_id: String,
    pub request_amount: i64,
    pub channel_code: String,
    pub reference_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentRequestWebhookPayload {
    pub event: String,
    pub data: XenditPaymentRequestData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditPaymentRequestData {
    pub payment_request_id: String,
    pub reference_id: String,
    pub status: String,
}
