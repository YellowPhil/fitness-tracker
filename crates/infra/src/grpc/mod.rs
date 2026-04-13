mod health_data_service;
mod workout_data_service;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use fitness_tracker_proto::health_data::health_data_service_server::HealthDataServiceServer;
use fitness_tracker_proto::workout_data::workout_data_service_server::WorkoutDataServiceServer;
use fitness_tracker_proto::workout_generator::workout_generator_service_client::WorkoutGeneratorServiceClient;
use fitness_tracker_proto::workout_generator::{GenerateWorkoutRequest, GenerateWorkoutResponse};
use tonic::transport::{Endpoint, Server};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::web::Databases;

pub async fn serve_workout_data(addr: SocketAddr, databases: Arc<Databases>) -> anyhow::Result<()> {
    let workout_data_service =
        workout_data_service::WorkoutDataGrpcService::new(Arc::clone(&databases));
    let health_data_service = health_data_service::HealthDataGrpcService::new(databases);

    Server::builder()
        .add_service(WorkoutDataServiceServer::new(workout_data_service))
        .add_service(HealthDataServiceServer::new(health_data_service))
        .serve(addr)
        .await
        .with_context(|| format!("gRPC server on {addr}"))
}

#[instrument(skip(request), fields(grpc_addr, timeout_ms = timeout.as_millis(), user_id = request.user_id))]
pub async fn request_generated_workout(
    grpc_addr: &str,
    timeout: std::time::Duration,
    request: GenerateWorkoutRequest,
) -> anyhow::Result<GenerateWorkoutResponse> {
    let request_id = Uuid::new_v4().to_string();
    let user_id = request.user_id;
    let muscle_group_count = request.muscle_groups.len();
    let max_exercise_count = request.max_exercise_count;

    info!(
        grpc_addr,
        timeout_ms = timeout.as_millis(),
        request_id,
        user_id,
        muscle_group_count,
        max_exercise_count,
        "calling WorkoutGeneratorService/GenerateWorkout"
    );

    let endpoint = Endpoint::from_shared(grpc_addr.to_string())
        .with_context(|| format!("parse WorkoutGeneratorService address {grpc_addr}"))?
        .connect_timeout(Duration::from_secs(5))
        .timeout(timeout);
    let channel = endpoint
        .connect()
        .await
        .with_context(|| format!("connect WorkoutGeneratorService at {grpc_addr}"))?;
    let mut client = WorkoutGeneratorServiceClient::new(channel);
    let mut grpc_request = tonic::Request::new(request);
    grpc_request.set_timeout(timeout);
    grpc_request.metadata_mut().insert(
        "x-request-id",
        request_id.parse().context("encode x-request-id metadata")?,
    );

    let response = client.generate_workout(grpc_request).await;

    match response {
        Ok(response) => {
            let generated = response.into_inner();
            info!(
                request_id,
                user_id,
                exercise_count = generated.exercises.len(),
                has_workout_name = generated.workout_name.is_some(),
                "WorkoutGeneratorService/GenerateWorkout completed"
            );
            Ok(generated)
        }
        Err(status) => {
            error!(
                request_id,
                user_id,
                grpc_code = ?status.code(),
                grpc_message = status.message(),
                "WorkoutGeneratorService/GenerateWorkout failed"
            );
            Err(status).with_context(|| "call WorkoutGeneratorService/GenerateWorkout")
        }
    }
}
