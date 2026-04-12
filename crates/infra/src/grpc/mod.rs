mod workout_data_service;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use fitness_tracker_proto::workout_data::workout_data_service_server::WorkoutDataServiceServer;
use fitness_tracker_proto::workout_generator::workout_generator_service_client::WorkoutGeneratorServiceClient;
use fitness_tracker_proto::workout_generator::{GenerateWorkoutRequest, GenerateWorkoutResponse};
use tonic::transport::{Endpoint, Server};

use crate::web::Databases;

pub async fn serve_workout_data(addr: SocketAddr, databases: Arc<Databases>) -> anyhow::Result<()> {
    let service = workout_data_service::WorkoutDataGrpcService::new(databases);

    Server::builder()
        .add_service(WorkoutDataServiceServer::new(service))
        .serve(addr)
        .await
        .with_context(|| format!("gRPC server on {addr}"))
}

pub async fn request_generated_workout(
    grpc_addr: &str,
    request: GenerateWorkoutRequest,
) -> anyhow::Result<GenerateWorkoutResponse> {
    let timeout = Duration::from_secs(20);
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

    client
        .generate_workout(grpc_request)
        .await
        .map(|response| response.into_inner())
        .with_context(|| "call WorkoutGeneratorService/GenerateWorkout")
}
