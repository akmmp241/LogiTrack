use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingDeductionEvent {
    pub user_id: Uuid,
    pub shipment_id: Uuid,
    pub amount: i64,
    pub deducted_at: DateTime<Utc>,
}
