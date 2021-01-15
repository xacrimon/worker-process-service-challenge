use anyhow::{anyhow, Result};
use protocol::{
    api_client::ApiClient, status_response, IssueJwtRequest, IssueJwtResponse, SpawnRequest,
    StatusRequest, StopRequest, StreamLogRequest, StreamLogResponse,
};
use std::collections::HashMap;
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
    Request, Streaming,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Claims {
    pub username: String,
    pub spawn: bool,
    pub stop: bool,
    pub stream_log: bool,
    pub status: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum JobStatus {
    Running,
    Terminated(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jwt(String);

pub struct UnauthorizedClient {
    remote: ApiClient<Channel>,
}

impl UnauthorizedClient {
    pub async fn connect(
        endpoint: &str,
        domain: &str,
        identity: Identity,
        server_ca: Certificate,
    ) -> Result<Self> {
        let tls = ClientTlsConfig::new()
            .domain_name(domain)
            .identity(identity)
            .ca_certificate(server_ca);

        let channel = Endpoint::from_shared(endpoint.to_string())?
            .tls_config(tls)?
            .connect()
            .await?;

        Ok(Self {
            remote: ApiClient::new(channel),
        })
    }

    pub async fn issue_jwt(&mut self, claims: Claims) -> Result<Jwt> {
        let request = IssueJwtRequest {
            user_name: claims.username,
            allow_spawn: claims.spawn,
            allow_stop: claims.stop,
            allow_stream_log: claims.stream_log,
            allow_status: claims.status,
        };

        let response = self.remote.issue_jwt(request).await?.into_inner();
        let jwt = Jwt(response.jwt);
        Ok(jwt)
    }

    pub async fn authorize(mut self, claims: Claims) -> Result<Client> {
        let token = self.issue_jwt(claims).await?;
        Ok(Client::new(self.remote, token))
    }
}

pub struct Client {
    remote: ApiClient<Channel>,
    token: Jwt,
}

impl Client {
    fn new(remote: ApiClient<Channel>, token: Jwt) -> Self {
        Self { remote, token }
    }

    fn authorize_request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        let header_value = MetadataValue::from_str(&format!("Bearer: {}", self.token.0)).unwrap();
        request.metadata_mut().insert("Authorization", header_value);
        request
    }

    pub async fn spawn(
        &mut self,
        program_path: String,
        working_directory: String,
        args: Vec<String>,
        envs: HashMap<String, String>,
    ) -> Result<Uuid> {
        let request = self.authorize_request(SpawnRequest {
            program: program_path,
            working_directory,
            args,
            envs,
        });

        let response = self.remote.spawn(request).await?.into_inner();
        let uuid = Uuid::from_slice(&response.uuid)?;
        Ok(uuid)
    }

    pub async fn stop(&mut self, job: Uuid) -> Result<()> {
        let request = self.authorize_request(StopRequest {
            uuid: job.as_bytes()[..].into(),
        });

        self.remote.stop(request).await?;
        Ok(())
    }

    pub async fn stream_log(
        &mut self,
        job: Uuid,
        from_beginning: bool,
    ) -> Result<Streaming<StreamLogResponse>> {
        let request = self.authorize_request(StreamLogRequest {
            uuid: job.as_bytes()[..].into(),
            from_beginning,
        });

        let response = self.remote.stream_log(request).await?.into_inner();
        Ok(response)
    }

    pub async fn status(&mut self, job: Uuid) -> Result<JobStatus> {
        let request = self.authorize_request(StatusRequest {
            uuid: job.as_bytes()[..].into(),
        });

        let response = self
            .remote
            .status(request)
            .await?
            .into_inner()
            .response
            .ok_or_else(|| anyhow!("no status response received"))?;

        Ok(match response {
            status_response::Response::Running(_) => JobStatus::Running,
            status_response::Response::Terminated(terminated) => {
                JobStatus::Terminated(terminated.code)
            }
        })
    }
}
