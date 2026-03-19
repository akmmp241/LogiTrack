use serde::Deserialize;

pub enum PaymentError {
    UnsupportedPaymentMethod(Option<String>),
    ProviderError(String),
    Unexpected(),
    DuplicatedPayment(),
    BadRequest(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum XenditErrorType {
    ChannelUnavailable,
    IssuerUnavailable,
    DataNotFound,
    InvalidValueError,
    ApiValidationError,
    InvalidPaymentDetails,
}
