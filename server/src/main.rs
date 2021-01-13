mod api;
mod auth;

use anyhow::Result;
use api::ApiCore;
use protocol::api_server::ApiServer;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};

const CERT: &[u8] = include_bytes!("../../data/server.pem");
const KEY: &[u8] = include_bytes!("../../data/server.key");
const CLIENT_CA_CERT: &[u8] = include_bytes!("../../data/client_ca.pem");

#[tokio::main]
async fn main() -> Result<()> {
    let server_identity = Identity::from_pem(CERT, KEY);
    let client_ca_cert = Certificate::from_pem(CLIENT_CA_CERT);
    let addr = "0.0.0.0:7005".parse().unwrap();
    let service = ApiServer::new(ApiCore::new());

    let tls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_cert);

    Server::builder()
        .tls_config(tls)?
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
