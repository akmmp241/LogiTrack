use crate::models::shipment::TrackingJob;

#[derive(Clone)]
pub struct TrackingJobRepository {
    pub pool: sqlx::PgPool,
}

impl TrackingJobRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, tracking_job: &TrackingJob) -> Result<bool, sqlx::Error> {
        let rows = sqlx::query(
            r#"
                INSERT INTO tracking_jobs 
                    (shipment_id, next_run_at, interval_minutes, is_active)
                VALUES ($1, $2, $3, true)
                "#,
        )
        .bind(tracking_job.shipment_id)
        .bind(tracking_job.next_run_at)
        .bind(tracking_job.interval_minutes)
        .execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() == 1)
    }
}
