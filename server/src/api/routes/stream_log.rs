use anyhow::Result;
use engine::{Engine, OutputEvent, UniqueJobId};
use futures::{Stream, StreamExt};
use protocol::{stream_log_response, StreamLogRequest, StreamLogResponse};
use std::pin::Pin;
use tokio::sync::Mutex;
use tonic::Status;
use uuid::Uuid;

pub type EventStream = Pin<Box<dyn Stream<Item = Result<StreamLogResponse, Status>> + Send + Sync>>;

pub async fn stream_log(
    engine: &Mutex<Engine>,
    request: &StreamLogRequest,
    username: &str,
) -> Result<EventStream, Status> {
    let uuid =
        Uuid::from_slice(&request.uuid).map_err(|_| Status::invalid_argument("malformed uuid"))?;

    let id = UniqueJobId::new(username.into(), uuid);
    let engine = engine.lock().await;
    let stream = engine
        .tail_log(&id, request.from_beginning)
        .map_err(|error| Status::internal(error.to_string()))?;

    Ok(Box::pin(stream.map(transform)))
}

fn transform(event: OutputEvent) -> Result<StreamLogResponse, Status> {
    Ok(StreamLogResponse {
        response: Some(match event {
            OutputEvent::Stdout(output) => {
                stream_log_response::Response::Stdout(stream_log_response::StreamLogStdoutEvent {
                    output,
                })
            }

            OutputEvent::Stderr(output) => {
                stream_log_response::Response::Stderr(stream_log_response::StreamLogStderrEvent {
                    output,
                })
            }

            OutputEvent::Exit(code) => {
                stream_log_response::Response::Exit(stream_log_response::StreamLogExitEvent {
                    code,
                })
            }
        }),
    })
}
