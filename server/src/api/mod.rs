use engine::Engine;
use protocol::{
    api_server::Api, IssueJwtRequest, IssueJwtResponse, SpawnRequest, SpawnResponse, StatusRequest,
    StatusResponse, StopRequest, StopResponse, StreamLogRequest, StreamLogResponse,
};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub struct ApiCore {
    engine: Mutex<Engine>,
}

impl ApiCore {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(Engine::new()),
        }
    }
}

#[tonic::async_trait]
impl Api for ApiCore {
    type StreamLogStream = UnboundedReceiver<Result<StreamLogResponse, Status>>;

    async fn spawn(
        &self,
        request: Request<SpawnRequest>,
    ) -> Result<Response<SpawnResponse>, Status> {
        todo!()
    }

    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        todo!()
    }

    async fn stream_log(
        &self,
        request: Request<StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        todo!()
    }

    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        todo!()
    }

    async fn issue_jwt(
        &self,
        request: Request<IssueJwtRequest>,
    ) -> Result<Response<IssueJwtResponse>, Status> {
        todo!()
    }
}
