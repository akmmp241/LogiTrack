use crate::models::api_key::ApiKey;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ApiKeyRepository {
    pool: PgPool,
}

impl ApiKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        name: &str,
        hashed_key: &str,
    ) -> Result<ApiKey, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (id, user_id, name, hashed_key)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, hashed_key, active, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(name)
        .bind(hashed_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(key)
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKey>, sqlx::Error> {
        let keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, hashed_key, active, created_at
            FROM api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(keys)
    }

    pub async fn find_active_by_hash(
        &self,
        hashed_key: &str,
    ) -> Result<Option<ApiKey>, sqlx::Error> {
        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, hashed_key, active, created_at
            FROM api_keys
            WHERE hashed_key = $1 AND active = true
            "#,
        )
        .bind(hashed_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(key)
    }

    pub async fn revoke(&self, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET active = false
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
