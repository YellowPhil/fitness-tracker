use std::sync::Arc;

use domain::types::UserId;
use fitness_tracker_proto::health_data::health_data_service_server::HealthDataService;
use fitness_tracker_proto::health_data::{
    GetCurrentHealthProfileRequest, GetCurrentHealthProfileResponse, HealthAttribute,
};
use tonic::{Request, Response, Status};
use tracing::instrument;

use crate::web::Databases;

pub struct HealthDataGrpcService {
    databases: Arc<Databases>,
}

impl HealthDataGrpcService {
    pub fn new(databases: Arc<Databases>) -> Self {
        Self { databases }
    }
}

#[tonic::async_trait]
impl HealthDataService for HealthDataGrpcService {
    #[instrument(skip(self, request), err)]
    async fn get_current_health_profile(
        &self,
        request: Request<GetCurrentHealthProfileRequest>,
    ) -> Result<Response<GetCurrentHealthProfileResponse>, Status> {
        let payload = request.into_inner();
        let user_id = UserId::new(payload.user_id);
        let profile = self
            .databases
            .health_app(user_id)
            .get_profile()
            .await
            .map_err(internal_status)?;

        let attributes = vec![
            HealthAttribute {
                key: "weight".to_string(),
                value: profile.weight.value.to_string(),
                unit: Some(profile.weight.units.to_string()),
            },
            HealthAttribute {
                key: "height".to_string(),
                value: profile.height.value.to_string(),
                unit: Some(profile.height.units.to_string()),
            },
            HealthAttribute {
                key: "age".to_string(),
                value: profile.age.to_string(),
                unit: None,
            },
        ];

        Ok(Response::new(GetCurrentHealthProfileResponse {
            attributes,
        }))
    }
}

fn internal_status(error: impl std::fmt::Display) -> Status {
    Status::internal(error.to_string())
}
