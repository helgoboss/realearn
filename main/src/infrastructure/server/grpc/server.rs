use crate::infrastructure::server::grpc::handlers::MyClipEngine;
use crate::infrastructure::server::grpc::proto::clip_engine_server::ClipEngineServer;
use crate::infrastructure::server::grpc::proto::ClipPositionUpdate;
use crate::infrastructure::server::layers::MainThreadLayer;
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tonic::transport::Server;

pub async fn start_grpc_server(
    address: SocketAddr,
    mut shutdown_receiver: broadcast::Receiver<()>,
) -> Result<(), tonic::transport::Error> {
    let clip_engine = MyClipEngine::default();
    Server::builder()
        .layer(MainThreadLayer)
        .add_service(ClipEngineServer::new(clip_engine))
        .serve_with_shutdown(
            address,
            async move { shutdown_receiver.recv().await.unwrap() },
        )
        .await
}

#[derive(Clone)]
pub struct GrpcClipPositionsUpdateEvent {
    pub session_id: String,
    pub updates: Vec<ClipPositionUpdate>,
}
