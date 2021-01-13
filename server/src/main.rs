mod api;
mod auth;
mod tls;

use anyhow::Result;
use api::ApiCore;
use protocol::api_server::ApiServer;
use tonic::transport::{Server, ServerTlsConfig};

const CERT: &[u8] = include_bytes!("../../data/server.pem");
const KEY: &[u8] = include_bytes!("../../data/server.key");
const CLIENT_CA_CERT: &[u8] = include_bytes!("../../data/client_ca.pem");

#[tokio::main]
async fn main() -> Result<()> {
    let server_cert = tls::load_pem_cert(CERT)?;
    let server_key = tls::load_private_key(KEY)?;
    let addr = "0.0.0.0:7005".parse().unwrap();
    let service = ApiServer::new(ApiCore::new());
    let base_tls_config = tls::tls_server_config(server_cert, server_key, CLIENT_CA_CERT)?;
    let mut tls = ServerTlsConfig::new();
    tls.rustls_server_config(base_tls_config);

    Server::builder()
        .tls_config(tls)?
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
