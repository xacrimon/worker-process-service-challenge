use anyhow::Result;
use engine::Engine;
use protocol::{SpawnRequest, SpawnResponse};
use tokio::sync::Mutex;
use tonic::Status;

pub async fn spawn(
    engine: &Mutex<Engine>,
    request: &SpawnRequest,
    username: &str,
) -> Result<SpawnResponse, Status> {
    let mut engine = engine.lock().await;
    let uuid = engine
        .spawn(
            username.into(),
            &request.program,
            &request.working_directory,
            &request.args,
            &request.envs,
        )
        .map_err(|error| Status::internal(error.to_string()))?;

    Ok(SpawnResponse {
        uuid: uuid.as_bytes()[..].into(),
    })
}
