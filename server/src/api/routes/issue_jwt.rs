use crate::auth::{self, Claims};
use protocol::{IssueJwtRequest, IssueJwtResponse};
use std::time::UNIX_EPOCH;
use tonic::Status;

/// 15 minute JWT expiration.
const JWT_EXPIRATION: usize = 15 * 60;

/// Figure out the unix timestamp 15 minutes from now.
fn generate_exp_timestamp() -> usize {
    UNIX_EPOCH.elapsed().unwrap().as_secs() as usize + JWT_EXPIRATION
}

pub async fn issue_jwt(request: &IssueJwtRequest) -> Result<IssueJwtResponse, Status> {
    let claims = Claims {
        exp: generate_exp_timestamp(),
        username: request.user_name.clone(),
        spawn: request.allow_spawn,
        stop: request.allow_stop,
        stream_log: request.allow_stream_log,
        status: request.allow_status,
    };

    let jwt = auth::issue_jwt(claims).map_err(|error| Status::internal(error.to_string()))?;
    Ok(IssueJwtResponse { jwt })
}
