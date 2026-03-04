use crate::models::notif_preferences::{GetCurrentPreferencesResponse, NotificationChannel};
use crate::repository::user_notif_preference_repo::UserNotifPreferenceRepository;
use anyhow::anyhow;
use axum::Json;
use errors::error::HttpError;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserNotificationPreferencesService {
    repo: UserNotifPreferenceRepository,
}

impl UserNotificationPreferencesService {
    pub fn new(repo: UserNotifPreferenceRepository) -> Self {
        Self { repo }
    }

    pub async fn get_current_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<GetCurrentPreferencesResponse, HttpError> {
        let notif_pref = self
            .repo
            .get_current_preferences(user_id)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?
            .ok_or_else(|| {
                tracing::warn!(
                    "user with id {} does not have notification preferences",
                    user_id
                );
                HttpError::InternalServerError(anyhow!("Missing notification preferences"))
            })?;

        Ok(notif_pref.default_channels)
    }

    pub async fn update_notif_pref(
        &self,
        user_id: Uuid,
        channels: Vec<NotificationChannel>,
    ) -> Result<(), HttpError> {
        let updated = self
            .repo
            .update_notif_pref(user_id, channels)
            .await
            .map_err(|e| HttpError::InternalServerError(e.into()))?;

        if !updated {
            return Err(HttpError::NotFound(
                "User Notification Preferences not found".to_string(),
            ));
        }

        Ok(())
    }
}
