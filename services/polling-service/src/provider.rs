use crate::domain::{LogisticsProvider, TrackingResult};
use biteship::BiteshipUseCase;
use chrono::Utc;

pub struct BiteshipProvider {
    use_case: BiteshipUseCase,
}

impl BiteshipProvider {
    pub fn new(use_case: BiteshipUseCase) -> Self {
        Self { use_case }
    }
}

#[async_trait::async_trait]
impl LogisticsProvider for BiteshipProvider {
    async fn fetch_tracking(
        &self,
        waybill_id: &str,
        courier_code: &str,
    ) -> Result<TrackingResult, anyhow::Error> {
        let resp = self
            .use_case
            .fetch_public_tracking(waybill_id.to_string(), courier_code.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Biteship API error: {:?}", e))?;

        let (description, occurred_at) = match resp.history.last() {
            Some(entry) => (entry.note.clone(), entry.updated_at),
            None => (resp.message.clone(), Utc::now()),
        };

        Ok(TrackingResult {
            raw_status: resp.status,
            description,
            occurred_at,
        })
    }
}
