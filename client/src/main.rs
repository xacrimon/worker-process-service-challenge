mod cli;
mod client;

use anyhow::Result;
use cli::{CommandOpts, Opts, StreamType};
use client::{Claims, Client, JobStatus, UnauthorizedClient};
use futures::StreamExt;
use std::collections::HashMap;
use structopt::StructOpt;
use tonic::transport::{Certificate, Identity};
use uuid::Uuid;

const CLIENT_CERT: &[u8] = include_bytes!("../../data/client1.pem");
const CLIENT_KEY: &[u8] = include_bytes!("../../data/client1.key");
const SERVER_CA_CERT: &[u8] = include_bytes!("../../data/ca.pem");

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();
    let identity = Identity::from_pem(CLIENT_CERT, CLIENT_KEY);
    let server_ca_certificate = Certificate::from_pem(SERVER_CA_CERT);
    let claims = Claims::full_permission(opts.username);

    let mut client = UnauthorizedClient::connect(
        &opts.endpoint,
        &opts.domain,
        identity,
        server_ca_certificate,
    )
    .await?
    .authorize(claims)
    .await?;

    match opts.command {
        CommandOpts::Spawn {
            program_path,
            working_directory,
            args,
            envs,
        } => spawn(&mut client, program_path, working_directory, args.0, envs.0).await?,
        CommandOpts::Stop { uuid } => stop(&mut client, uuid).await?,
        CommandOpts::StreamLog {
            uuid,
            past_events,
            stream_type,
        } => stream_log(&mut client, uuid, past_events, stream_type).await?,
        CommandOpts::Status { uuid } => status(&mut client, uuid).await?,
    }

    Ok(())
}

async fn spawn(
    client: &mut Client,
    program_path: String,
    working_directory: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
) -> Result<()> {
    let uuid = client
        .spawn(program_path, working_directory, args, envs)
        .await?;
    println!("spawned job with id {}", uuid);
    Ok(())
}

async fn stop(client: &mut Client, uuid: Uuid) -> Result<()> {
    client.stop(uuid).await?;
    println!("stopped job with id {} if it was running", uuid);
    Ok(())
}

async fn stream_log(
    client: &mut Client,
    uuid: Uuid,
    past_events: bool,
    stream_type: StreamType,
) -> Result<()> {
    let mut stream = client.stream_log(uuid, past_events).await?;
    let mut writer = stream_type.writer();
    writer.start()?;

    while let Some(Ok(event)) = stream.next().await {
        writer.write(event)?;
    }

    Ok(())
}

async fn status(client: &mut Client, uuid: Uuid) -> Result<()> {
    let status = client.status(uuid).await?;

    match status {
        JobStatus::Running => println!("job with id {} is running", uuid),
        JobStatus::Terminated(code) => {
            println!("job with id {} has terminated with code {}", uuid, code)
        }
    }

    Ok(())
}
