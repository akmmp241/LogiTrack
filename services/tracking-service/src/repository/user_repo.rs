use crate::models::user::UserWithNotifPref;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pub pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<UserWithNotifPref>, sqlx::Error> {
        let result = sqlx::query_as::<_, UserWithNotifPref>(
            r#"
                SELECT u.id, u.name, u.phone_number, u.email, unp.default_channels
                FROM users u
                    INNER JOIN user_notification_preferences unp ON u.id = unp.user_id
                WHERE u.id = $1;
                "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }
}
