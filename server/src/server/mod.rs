mod api;
mod auth;
mod tls;

use anyhow::Result;
use api::ApiCore;
use protocol::api_server::ApiServer;
use tonic::transport::{Server, ServerTlsConfig};

const ADDR: &str = "0.0.0.0:7005";

// We include the certs and keys in the binary for simplicity.
const CERT: &[u8] = include_bytes!("../../../data/server.pem");
const KEY: &[u8] = include_bytes!("../../../data/server.key");
const CLIENT_CA_CERT: &[u8] = include_bytes!("../../../data/client_ca.pem");

pub async fn serve() -> Result<()> {
    let server_cert = tls::load_pem_cert(CERT)?;
    let server_key = tls::load_private_key(KEY)?;
    let addr = ADDR.parse().unwrap();
    let service = ApiServer::new(ApiCore::new());
    let base_tls_config = tls::tls_server_config(server_cert, server_key, CLIENT_CA_CERT)?;
    let mut tls = ServerTlsConfig::new();
    tls.rustls_server_config(base_tls_config);
    println!("serving gRPC endpoint at {}", ADDR);

    Server::builder()
        .tls_config(tls)?
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}
