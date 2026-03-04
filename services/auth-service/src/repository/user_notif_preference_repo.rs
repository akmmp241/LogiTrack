use crate::models::notif_preferences::{NotificationChannel, UserNotificationPreferences};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserNotifPreferenceRepository {
    pool: PgPool,
}

impl UserNotifPreferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// use default value from database while creating
    pub async fn create_notif_preferences(&self, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let rows = sqlx::query(
            r#"
                INSERT INTO user_notification_preferences (user_id) VALUES ($1)
                "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() == 1)
    }

    pub async fn get_current_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<Option<UserNotificationPreferences>, sqlx::Error> {
        let result = sqlx::query_as::<_, UserNotificationPreferences>(
            r#"
                SELECT user_id, default_channels, updated_at
                FROM user_notification_preferences WHERE user_id = $1
                "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn update_notif_pref(
        &self,
        user_id: Uuid,
        channels: Vec<NotificationChannel>,
    ) -> Result<bool, sqlx::Error> {
        let rows = sqlx::query(
            r#"
                UPDATE user_notification_preferences SET default_channels = $1, 
                updated_at = NOW() WHERE user_id = $2
                "#,
        )
        .bind(channels)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() == 1)
    }
}
