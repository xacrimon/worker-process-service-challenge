use anyhow::Result;
use engine::{Engine, OutputEvent, UniqueJobId};
use protocol::{status_response, StatusRequest, StatusResponse};
use tokio::sync::Mutex;
use tonic::Status;
use uuid::Uuid;

pub async fn status(
    engine: &Mutex<Engine>,
    request: &StatusRequest,
    username: &str,
) -> Result<StatusResponse, Status> {
    let uuid =
        Uuid::from_slice(&request.uuid).map_err(|_| Status::invalid_argument("malformed uuid"))?;

    let id = UniqueJobId::new(username.into(), uuid);
    let engine = engine.lock().await;
    let events = engine
        .get_past_events(&id)
        .map_err(|error| Status::internal(error.to_string()))?;

    let response = if let Some(code) = events.iter().find_map(|event| {
        if let OutputEvent::Exit(code) = event {
            Some(*code)
        } else {
            None
        }
    }) {
        status_response::Response::Terminated(status_response::StatusResponseTerminated { code })
    } else {
        status_response::Response::Running(status_response::StatusResponseRunning {})
    };

    Ok(StatusResponse {
        response: Some(response),
    })
}
