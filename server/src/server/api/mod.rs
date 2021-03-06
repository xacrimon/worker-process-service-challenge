mod routes;

use crate::server::auth;
use engine::Engine;
use protocol::{
    api_server::Api, IssueJwtRequest, IssueJwtResponse, SpawnRequest, SpawnResponse, StatusRequest,
    StatusResponse, StopRequest, StopResponse, StreamLogRequest,
};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

/// Our service handler.
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
    type StreamLogStream = routes::stream_log::EventStream;

    async fn spawn(
        &self,
        request: Request<SpawnRequest>,
    ) -> Result<Response<SpawnResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.spawn {
            return Err(Status::permission_denied("claims.spawn not true"));
        }

        let request = request.get_ref();
        routes::spawn::spawn(&self.engine, request, &claims.username)
            .await
            .map(Response::new)
    }

    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.stop {
            return Err(Status::permission_denied("claims.stop not true"));
        }

        let request = request.get_ref();
        routes::stop::stop(&self.engine, request, &claims.username)
            .await
            .map(Response::new)
    }

    async fn stream_log(
        &self,
        request: Request<StreamLogRequest>,
    ) -> Result<Response<Self::StreamLogStream>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.stream_log {
            return Err(Status::permission_denied("claims.stream_log not true"));
        }

        let request = request.get_ref();
        routes::stream_log::stream_log(&self.engine, request, &claims.username)
            .await
            .map(Response::new)
    }

    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let claims = auth::validate_claims(&request)?;

        if !claims.status {
            return Err(Status::permission_denied("claims.status not true"));
        }

        let request = request.get_ref();
        routes::status::status(&self.engine, request, &claims.username)
            .await
            .map(Response::new)
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
