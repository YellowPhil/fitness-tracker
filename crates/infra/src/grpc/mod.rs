mod workout_data_service;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use fitness_tracker_proto::workout_data::workout_data_service_server::WorkoutDataServiceServer;
use tonic::transport::Server;

use crate::web::Databases;

pub async fn serve_workout_data(addr: SocketAddr, databases: Arc<Databases>) -> anyhow::Result<()> {
    let service = workout_data_service::WorkoutDataGrpcService::new(databases);

    Server::builder()
        .add_service(WorkoutDataServiceServer::new(service))
        .serve(addr)
        .await
        .with_context(|| format!("gRPC server on {addr}"))
}
