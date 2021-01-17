use anyhow::{anyhow, Result};
use protocol::{
    api_client::ApiClient, status_response, IssueJwtRequest, SpawnRequest, StatusRequest,
    StopRequest, StreamLogRequest, StreamLogResponse,
};
use std::collections::HashMap;
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
    Request, Streaming,
};
use uuid::Uuid;

/// The permissions that the requested JWT should have.
#[derive(Debug, Clone)]
pub struct Claims {
    pub username: String,
    pub spawn: bool,
    pub stop: bool,
    pub stream_log: bool,
    pub status: bool,
}

impl Claims {
    pub fn full_permission(username: String) -> Self {
        Self {
            username,
            spawn: true,
            stop: true,
            stream_log: true,
            status: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JobStatus {
    Running,
    Terminated(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jwt(String);

/// A client that has yet to authorize itself and acquire a JWT.
/// In a production system you'd really just skip this whole step
/// since you'd retrieve your JWT from some other authorization authority
/// service that's mutually trusted. Since we don't have that here,
/// we're essentially pretending that we also are one but in reality
/// it's just an unprotected dummy gRPC call that allows us to create fake JWTs.
///
/// You'd probably also want to cache these tokens in some form of persistent storage
/// but in order for that to work correctly you need a full setup that supports refresh tokens
/// in order to save time, I've skipped implementing JWT refresh tokens which means they're mostly
/// useless to cache locally.
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
            .ca_certificate(server_ca)
            .identity(identity);

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

/// An authorized client with an active JWT.
pub struct Client {
    remote: ApiClient<Channel>,
    token: Jwt,
}

impl Client {
    fn new(remote: ApiClient<Channel>, token: Jwt) -> Self {
        Self { remote, token }
    }

    /// Embed the JWT in the request metadata, authorizing us as the client.
    fn authorize_request<T>(&self, message: T) -> Request<T> {
        let mut request = Request::new(message);
        let header_value = MetadataValue::from_str(&format!("Bearer {}", self.token.0)).unwrap();
        request.metadata_mut().insert("authorization", header_value);
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
