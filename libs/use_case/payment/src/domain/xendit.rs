use crate::errors::{PaymentError, XenditErrorType};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct XenditError {
    pub error_code: XenditErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum XenditAvailableChannel {
    Qris,
    Alfamart,
    Indomaret,
    BCAVirtualAccount,
    BniVirtualAccount,
    BriVirtualAccount,
    MandiriVirtualAccount,
    OVO,
    GOPAY,
    DANA,
    LINKAJA,
    SHOPEEPAY,
}

impl FromStr for XenditAvailableChannel {
    type Err = PaymentError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QRIS" => Ok(Self::Qris),
            "ALFAMART" => Ok(Self::Alfamart),
            "INDOMARET" => Ok(Self::Indomaret),
            "BCA_VIRTUAL_ACCOUNT" => Ok(Self::BCAVirtualAccount),
            "BNI_VIRTUAL_ACCOUNT" => Ok(Self::BniVirtualAccount),
            "BRI_VIRTUAL_ACCOUNT" => Ok(Self::BriVirtualAccount),
            "MANDIRI_VIRTUAL_ACCOUNT" => Ok(Self::MandiriVirtualAccount),
            "OVO" => Ok(Self::OVO),
            "GOPAY" => Ok(Self::GOPAY),
            "DANA" => Ok(Self::DANA),
            "LINKAJA" => Ok(Self::LINKAJA),
            "SHOPEEPAY" => Ok(Self::SHOPEEPAY),
            _ => Err(PaymentError::UnsupportedPaymentMethod(Some(
                "Unsupported payment method".to_string(),
            ))),
        }
    }
}

impl XenditAvailableChannel {
    pub fn default_properties(&self) -> XenditChannelProperties {
        let otc_defaults = || OtcFields {
            payer_name: String::new(),
            expires_at: Some(Utc::now() + Duration::days(1)),
            payment_code: None,
        };

        let va_defaults = || VirtualAccountFields {
            display_name: String::new(),
            expires_at: Some(Utc::now() + Duration::days(1)),
            virtual_account_number: None,
        };

        let return_url_defaults = || ReturnUrls {
            success_return_url: String::new(),
        };

        match self {
            Self::Qris => XenditChannelProperties::Qris {
                expires_at: Some(Utc::now() + Duration::days(1)),
            },
            Self::Alfamart => XenditChannelProperties::Alfamart(otc_defaults()),
            Self::Indomaret => XenditChannelProperties::Indomaret(otc_defaults()),
            Self::BCAVirtualAccount => XenditChannelProperties::BCAVirtualAccount(va_defaults()),
            Self::BniVirtualAccount => XenditChannelProperties::BniVirtualAccount(va_defaults()),
            Self::BriVirtualAccount => XenditChannelProperties::BriVirtualAccount(va_defaults()),
            Self::MandiriVirtualAccount => {
                XenditChannelProperties::MandiriVirtualAccount(va_defaults())
            }
            Self::GOPAY => XenditChannelProperties::GOPAY(GopayFields {
                success_return_url: String::new(),
                failure_return_url: String::new(),
                cancel_return_url: String::new(),
            }),
            Self::DANA => XenditChannelProperties::DANA(return_url_defaults()),
            Self::LINKAJA => XenditChannelProperties::LINKAJA(return_url_defaults()),
            Self::SHOPEEPAY => XenditChannelProperties::SHOPEEPAY(return_url_defaults()),
            Self::OVO => XenditChannelProperties::Ovo {
                account_mobile_number: "".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum XenditActionType {
    PresentToCustomer,
    RedirectCustomer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum XenditActionDescriptor {
    QrString,
    VirtualAccountNumber,
    WebUrl,
    DeeplinkUrl,
    ValidateOtp,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct XenditActions {
    #[serde(rename = "type")]
    pub type_: XenditActionType,
    pub descriptor: XenditActionDescriptor,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReturnUrls {
    pub success_return_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirtualAccountFields {
    pub display_name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub virtual_account_number: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GopayFields {
    pub success_return_url: String,
    pub failure_return_url: String,
    pub cancel_return_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OtcFields {
    pub payer_name: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub payment_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum XenditChannelProperties {
    Qris { expires_at: Option<DateTime<Utc>> },
    Ovo { account_mobile_number: String },
    Alfamart(OtcFields),
    Indomaret(OtcFields),
    BCAVirtualAccount(VirtualAccountFields),
    BniVirtualAccount(VirtualAccountFields),
    BriVirtualAccount(VirtualAccountFields),
    MandiriVirtualAccount(VirtualAccountFields),
    GOPAY(GopayFields),
    DANA(ReturnUrls),
    LINKAJA(ReturnUrls),
    SHOPEEPAY(ReturnUrls),
}

impl XenditChannelProperties {
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn with_expires_at(&mut self, new_expires_at: DateTime<Utc>) -> &mut Self {
        match self {
            Self::Qris { expires_at, .. } => *expires_at = Some(new_expires_at),
            Self::Alfamart(c) => c.expires_at = Some(new_expires_at),
            Self::Indomaret(c) => c.expires_at = Some(new_expires_at),
            Self::BCAVirtualAccount(c) => c.expires_at = Some(new_expires_at),
            Self::BniVirtualAccount(c) => c.expires_at = Some(new_expires_at),
            Self::BriVirtualAccount(c) => c.expires_at = Some(new_expires_at),
            Self::MandiriVirtualAccount(c) => c.expires_at = Some(new_expires_at),
            _ => {}
        }
        self
    }

    pub fn with_user_name(&mut self, name: &str) -> &mut Self {
        match self {
            Self::Alfamart(c) => c.payer_name = name.to_string(),
            Self::Indomaret(c) => c.payer_name = name.to_string(),
            Self::BCAVirtualAccount(c) => c.display_name = name.to_string(),
            Self::BniVirtualAccount(c) => c.display_name = name.to_string(),
            Self::BriVirtualAccount(c) => c.display_name = name.to_string(),
            Self::MandiriVirtualAccount(c) => c.display_name = name.to_string(),
            _ => {}
        }
        self
    }

    pub fn with_success_return_url(&mut self, url: &str) -> &mut Self {
        match self {
            Self::GOPAY(c) => c.success_return_url = url.to_string(),
            Self::DANA(c) => c.success_return_url = url.to_string(),
            Self::LINKAJA(c) => c.success_return_url = url.to_string(),
            Self::SHOPEEPAY(c) => c.success_return_url = url.to_string(),
            _ => {}
        }
        self
    }

    pub fn with_failure_return_url(&mut self, url: &str) -> &mut Self {
        if let Self::GOPAY(c) = self {
            c.failure_return_url = url.to_string()
        }
        self
    }

    pub fn with_cancel_return_url(&mut self, url: &str) -> &mut Self {
        if let Self::GOPAY(c) = self {
            c.failure_return_url = url.to_string()
        }
        self
    }

    pub fn with_mobile_number(&mut self, mobile_number: &str) -> &mut Self {
        if let Self::Ovo {
            account_mobile_number,
            ..
        } = self
        {
            *account_mobile_number = mobile_number.to_string();
        }
        self
    }
}
