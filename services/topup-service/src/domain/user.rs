use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub email: String,
}
