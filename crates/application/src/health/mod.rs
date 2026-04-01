use domain::{
    health::HealthParams,
    traits::HealthRepo,
    types::Weight,
};
use tracing::instrument;

pub struct HealthApp<H: HealthRepo> {
    health_repo: H,
}

impl<H: HealthRepo> HealthApp<H> {
    pub fn new(health_repo: H) -> Self {
        Self { health_repo }
    }

    #[instrument(skip(self), err)]
    pub fn get_profile(&self) -> Result<HealthParams, H::RepoError> {
        self.health_repo.get_health()
    }

    #[instrument(skip(self, params), err)]
    pub fn update_profile(&self, params: HealthParams) -> Result<HealthParams, H::RepoError> {
        self.health_repo.save(&params)?;
        Ok(params)
    }

    #[instrument(skip(self, weight), err)]
    pub fn update_weight(&self, weight: Weight) -> Result<HealthParams, H::RepoError> {
        let mut current = self.health_repo.get_health()?;
        current.weight = weight;
        self.health_repo.save(&current)?;
        Ok(current)
    }
}
