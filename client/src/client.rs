use anyhow::Result;
use protocol::{
    api_client::ApiClient, IssueJwtRequest, IssueJwtResponse, SpawnRequest, StreamLogResponse,
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
    pub async fn new(
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

    pub fn spawn(
        &mut self,
        program_path: String,
        working_directory: String,
        args: Vec<String>,
        envs: HashMap<String, String>,
    ) -> Result<Uuid> {
        todo!()
    }

    pub fn stop(&mut self, job: Uuid) -> Result<()> {
        todo!()
    }

    pub fn stream_log(&mut self, job: Uuid, from_beginning: bool) -> Streaming<StreamLogResponse> {
        todo!()
    }

    pub fn status(&mut self, job: Uuid) -> Result<JobStatus> {
        todo!()
    }
}
