use anyhow::{anyhow, Result};
use rustls::{
    ciphersuite::TLS13_AES_256_GCM_SHA384, internal::pemfile, AllowAnyAuthenticatedClient,
    Certificate, PrivateKey, RootCertStore, ServerConfig,
};

/// Load a x509 cerificate from raw PEM;
pub fn load_pem_cert(mut pem: &[u8]) -> Result<Certificate> {
    let list = pemfile::certs(&mut pem).map_err(|_| anyhow!("could not parse pem"))?;
    list.get(0)
        .cloned()
        .ok_or_else(|| anyhow!("pem did not contain a certificate"))
}

/// Load an RSA private key from a file.
pub fn load_private_key(mut raw: &[u8]) -> Result<PrivateKey> {
    let list =
        pemfile::rsa_private_keys(&mut raw).map_err(|_| anyhow!("could not parse raw key"))?;

    list.get(0)
        .cloned()
        .ok_or_else(|| anyhow!("raw did not contain a private key"))
}

pub fn tls_server_config(
    server_cert: Certificate,
    server_key: PrivateKey,
    mut client_ca: &[u8],
) -> Result<ServerConfig> {
    let mut cert_store = RootCertStore::empty();
    cert_store
        .add_pem_file(&mut client_ca)
        .map_err(|_| anyhow!("failed to parse client ca certificate"))?;

    let authenticator = AllowAnyAuthenticatedClient::new(cert_store);
    let mut config = ServerConfig::with_ciphersuites(authenticator, &[&TLS13_AES_256_GCM_SHA384]);
    config.set_single_cert(vec![server_cert], server_key)?;
    Ok(config)
}
