use crate::domain::user::User;
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

    pub async fn get_by_id(&self, user_id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let result = sqlx::query_as::<_, User>(
            r#"
                SELECT u.id id , u.name, u.phone_number, u.email
                FROM users u
                WHERE u.id = $1;
                "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }
}
