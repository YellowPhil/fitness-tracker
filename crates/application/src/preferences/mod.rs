use domain::{preferences::WorkoutPreferences, traits::PreferencesRepo};
use tracing::instrument;

pub struct PreferencesApp<P: PreferencesRepo> {
    preferences_repo: P,
}

impl<P: PreferencesRepo> PreferencesApp<P> {
    pub fn new(preferences_repo: P) -> Self {
        Self { preferences_repo }
    }

    #[instrument(skip(self), err)]
    pub async fn get_preferences(&self) -> Result<WorkoutPreferences, P::RepoError> {
        self.preferences_repo.get_preferences().await
    }

    #[instrument(skip(self, preferences), err)]
    pub async fn update_preferences(
        &self,
        preferences: WorkoutPreferences,
    ) -> Result<WorkoutPreferences, P::RepoError> {
        self.preferences_repo.save(&preferences).await?;
        Ok(preferences)
    }
}
