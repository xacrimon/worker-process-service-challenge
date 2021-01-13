mod routes;

use crate::auth;
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

/// For authorization we'd ideally use some sort of proper middleware setup here.
/// The only good way to integrate that into the Rust stack would be with a Tower service
/// but that takes significant amounts of boilerplate code. Therefore I'll be doing in the
/// improper way and verifying authorization in each route to save some time here.
#[tonic::async_trait]
impl Api for ApiCore {
    type StreamLogStream = UnboundedReceiver<Result<StreamLogResponse, Status>>;

    async fn spawn(
        &self,
        request: Request<SpawnRequest>,
    ) -> Result<Response<SpawnResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.spawn {
            return Err(Status::permission_denied("claims.spawn not true"));
        }

        todo!()
    }

    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.stop {
            return Err(Status::permission_denied("claims.stop not true"));
        }

        todo!()
    }

    async fn stream_log(
        &self,
        request: Request<StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.stream_log {
            return Err(Status::permission_denied("claims.stream_log not true"));
        }

        todo!()
    }

    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.status {
            return Err(Status::permission_denied("claims.status not true"));
        }

        todo!()
    }

    async fn issue_jwt(
        &self,
        request: Request<IssueJwtRequest>,
    ) -> Result<Response<IssueJwtResponse>, Status> {
        let request = request.get_ref();
        routes::issue_jwt::issue_jwt(request)
            .await
            .map(Response::new)
    }
}
