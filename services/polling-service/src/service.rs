use crate::domain::{
    LogisticsProvider, NotificationChannel, ShipmentStatus, ShipmentWithJob, TrackingEventMsg,
    TrackingEventMsgType, TrackingMsgPayload, get_interval_minutes,
};
use crate::repository::PollingRepository;
use chrono::{Duration, Utc};
use lapin::BasicProperties;
use lapin::options::BasicPublishOptions;
use std::sync::Arc;
use uuid::Uuid;

static EXCHANGE_NAME: &str = "notification.events";

const MAX_RETRY_ATTEMPTS: i32 = 3;

pub struct PollingService {
    repo: PollingRepository,
    provider: Arc<dyn LogisticsProvider>,
    rabbitmq_channel: lapin::Channel,
}

impl PollingService {
    pub fn new(
        repo: PollingRepository,
        provider: Arc<dyn LogisticsProvider>,
        rabbitmq_channel: lapin::Channel,
    ) -> Self {
        Self {
            repo,
            provider,
            rabbitmq_channel,
        }
    }

    pub async fn poll_due_shipments(&self) {
        let jobs = match self.repo.fetch_due_jobs().await {
            Ok(jobs) => jobs,
            Err(e) => {
                tracing::error!("failed to fetch due jobs: {}", e);
                return;
            }
        };

        if jobs.is_empty() {
            return;
        }

        tracing::info!("processing {} due shipment(s)", jobs.len());

        for job in jobs {
            let repo = self.repo.clone();
            let provider = self.provider.clone();
            let channel = self.rabbitmq_channel.clone();

            tokio::spawn(async move {
                if let Err(e) = process_single_shipment(&repo, &provider, &channel, &job).await {
                    tracing::error!(
                        shipment_id = %job.shipment_id,
                        waybill = %job.waybill_id,
                        "failed to process shipment: {}",
                        e
                    );
                }
            });
        }
    }
}

async fn process_single_shipment(
    repo: &PollingRepository,
    provider: &Arc<dyn LogisticsProvider>,
    channel: &lapin::Channel,
    job: &ShipmentWithJob,
) -> Result<(), anyhow::Error> {
    let tracking_result = match provider
        .fetch_tracking(&job.waybill_id, &job.courier_code)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return handle_api_failure(repo, job, e).await;
        }
    };

    let status_mapping = repo.get_status_mapping(&tracking_result.raw_status).await?;

    let new_status = match status_mapping {
        Some(ref mapping) => mapping.normalized_status.clone(),
        None => {
            tracing::warn!(
                shipment_id = %job.shipment_id,
                raw_status = %tracking_result.raw_status,
                "no status mapping found, defaulting to Unknown"
            );
            ShipmentStatus::Unknown
        }
    };

    let status_changed = new_status != job.current_status;

    if status_changed {
        repo.insert_tracking_event(
            job.shipment_id,
            &tracking_result.raw_status,
            &new_status,
            &tracking_result.description,
            tracking_result.occurred_at,
        )
        .await?;

        repo.update_shipment_status(job.shipment_id, &new_status)
            .await?;

        match get_interval_minutes(&new_status) {
            Some(interval) => {
                let next_run = Utc::now() + Duration::minutes(interval as i64);

                repo.update_job_schedule(job.shipment_id, next_run, interval, 0)
                    .await?;
            }
            None => {
                repo.deactivate_job(job.shipment_id).await?;
            }
        }

        publish_status_change_events(repo, channel, job, &new_status).await?;
    } else {
        let interval = get_interval_minutes(&new_status).unwrap_or(job.interval_minutes);
        let next_run = Utc::now() + Duration::minutes(interval as i64);

        repo.update_job_schedule(job.shipment_id, next_run, interval, 0)
            .await?;
    }

    Ok(())
}

async fn handle_api_failure(
    repo: &PollingRepository,
    job: &ShipmentWithJob,
    error: anyhow::Error,
) -> Result<(), anyhow::Error> {
    let new_attempt = job.attempt + 1;

    if new_attempt >= MAX_RETRY_ATTEMPTS {
        tracing::warn!(
            shipment_id = %job.shipment_id,
            attempts = new_attempt,
            "max retry attempts reached, skipping until next regular cycle"
        );

        let interval = get_interval_minutes(&job.current_status).unwrap_or(job.interval_minutes);
        let next_run = Utc::now() + Duration::minutes(interval as i64);

        repo.update_job_schedule(job.shipment_id, next_run, interval, 0)
            .await?;
    } else {
        let backoff_minutes = 5i64 * 3i64.pow((new_attempt - 1) as u32);

        tracing::warn!(
            shipment_id = %job.shipment_id,
            attempt = new_attempt,
            backoff_minutes = backoff_minutes,
            error = %error,
            "API call failed, retrying with backoff"
        );

        let next_run = Utc::now() + Duration::minutes(backoff_minutes);

        repo.update_job_schedule(job.shipment_id, next_run, job.interval_minutes, new_attempt)
            .await?;
    }

    Ok(())
}

async fn publish_status_change_events(
    repo: &PollingRepository,
    channel: &lapin::Channel,
    job: &ShipmentWithJob,
    new_status: &ShipmentStatus,
) -> Result<(), anyhow::Error> {
    let subscriptions = repo.get_shipment_subscriptions(job.shipment_id).await?;

    for sub in subscriptions {
        if !sub.subscribed_statuses.contains(new_status) {
            continue;
        }

        for ch in &sub.subscribed_channels {
            let recipient = match ch {
                NotificationChannel::Whatsapp => "6285158824017",
                NotificationChannel::Email => "akmalmp241@gmail.com",
            };

            let msg = TrackingEventMsg {
                message_id: Uuid::new_v4(),
                event_type: TrackingEventMsgType::TrackingStatusUpdated,
                channel: ch.clone(),
                user_id: sub.user_id,
                recipient: recipient.to_string(),
                template_code: "TRACKING_STATUS".to_string(),
                payload: TrackingMsgPayload {
                    waybill_id: job.waybill_id.clone(),
                    status: new_status.to_string().to_lowercase(),
                    courier: job.courier_code.clone(),
                },
            };

            let payload = serde_json::to_vec(&msg)?;

            let confirm = channel
                .basic_publish(
                    EXCHANGE_NAME,
                    &format!(
                        "notification.tracking_status_changed.{}",
                        ch.to_string().to_lowercase()
                    ),
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default().with_delivery_mode(2),
                )
                .await?;

            confirm.await?;
        }
    }

    Ok(())
}
