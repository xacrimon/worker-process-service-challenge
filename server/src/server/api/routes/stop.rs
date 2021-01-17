use anyhow::Result;
use engine::{Engine, UniqueJobId};
use protocol::{StopRequest, StopResponse};
use tokio::sync::Mutex;
use tonic::Status;
use uuid::Uuid;

pub async fn stop(
    engine: &Mutex<Engine>,
    request: &StopRequest,
    username: &str,
) -> Result<StopResponse, Status> {
    let uuid =
        Uuid::from_slice(&request.uuid).map_err(|_| Status::invalid_argument("malformed uuid"))?;

    let id = UniqueJobId::new(username.into(), uuid);
    let engine = engine.lock().await;
    engine
        .stop(&id)
        .map_err(|error| Status::internal(error.to_string()))?;

    Ok(StopResponse {})
}
