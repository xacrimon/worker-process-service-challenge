use anyhow::{anyhow, Result};
use rustls::{
    ciphersuite::TLS13_AES_256_GCM_SHA384, internal::pemfile, AllowAnyAuthenticatedClient,
    Certificate, PrivateKey, RootCertStore, ServerConfig,
};
use std::io::{BufReader, Cursor};

pub fn load_pem_cert(pem: &[u8]) -> Result<Certificate> {
    let mut pem_reader = BufReader::new(Cursor::new(pem));
    let list = pemfile::certs(&mut pem_reader).map_err(|_| anyhow!("could not parse pem"))?;
    list.get(0)
        .map(|cert| cert.clone())
        .ok_or_else(|| anyhow!("pem did not contain a certificate"))
}

pub fn load_private_key(raw: &[u8]) -> Result<PrivateKey> {
    let mut raw_reader = BufReader::new(Cursor::new(raw));
    let list = pemfile::rsa_private_keys(&mut raw_reader)
        .map_err(|_| anyhow!("could not parse raw key"))?;
    list.get(0)
        .map(|cert| cert.clone())
        .ok_or_else(|| anyhow!("raw did not contain a private key"))
}

pub fn tls_server_config(
    server_cert: Certificate,
    server_key: PrivateKey,
    client_ca: &[u8],
) -> Result<ServerConfig> {
    let mut client_ca_reader = BufReader::new(Cursor::new(client_ca));
    let mut cert_store = RootCertStore::empty();
    cert_store
        .add_pem_file(&mut client_ca_reader)
        .map_err(|_| anyhow!("failed to parse client ca certificate"))?;

    let authenticator = AllowAnyAuthenticatedClient::new(cert_store);
    let mut config = ServerConfig::with_ciphersuites(authenticator, &[&TLS13_AES_256_GCM_SHA384]);
    config.set_single_cert(vec![server_cert], server_key)?;
    Ok(config)
}
