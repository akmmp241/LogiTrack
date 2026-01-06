pub mod errors {
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    pub struct BiteshipError {
        pub success: bool,
        pub error: String,
        pub code: i32,
    }
}

pub mod tracking {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
    pub struct BiteshipTrackingResponse {
        pub success: bool,
        pub message: String,
        pub order_id: Option<String>,
        pub status: String,
        pub courier: Courier,
        pub destination: Destination,
        pub origin: Origin,
        pub history: Vec<History>,
    }

    #[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Courier {
        pub company: String,
        pub name: String,
        pub phone: String,
        pub driver_name: String,
        pub driver_phone: String,
    }

    #[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
    pub struct Destination {
        pub contact_name: String,
        pub address: String,
    }

    #[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
    pub struct Origin {
        pub contact_name: String,
        pub address: String,
    }

    #[derive(Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct History {
        pub note: String,
        pub service_type: Option<String>,
        pub status: String,
        pub updated_at: DateTime<Utc>,
    }
}
