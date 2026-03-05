use crate::models::notification_log::NotificationLog;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationLogRepository {
    pub pool: Pool<Postgres>,
}

impl NotificationLogRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_by_shipment_id(
        &self,
        shipment_id: Uuid,
    ) -> Result<Vec<NotificationLog>, sqlx::Error> {
        let res: Vec<NotificationLog> = sqlx::query_as(
            r#"
            SELECT id, shipment_id, event_id, channel, recipient_to,
                    message_content, status, error_message, sent_at, created_at
             FROM notification_logs
             WHERE shipment_id = $1
             ORDER BY created_at DESC
             "#,
        )
        .bind(shipment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<NotificationLog>, sqlx::Error> {
        let res: Option<NotificationLog> = sqlx::query_as(
            r#"
                SELECT id, shipment_id, event_id, channel, recipient_to,
                    message_content, status, error_message, sent_at, created_at
                FROM notification_logs
                WHERE id = $1
                "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(res)
    }
}
