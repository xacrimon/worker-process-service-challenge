use super::{ENDPOINT, USERNAME};
use crate::client::{Claims, UnauthorizedClient};
use crate::{CLIENT_CERT, CLIENT_KEY};
use anyhow::Result;
use serial_test::serial;
use server::server;
use std::collections::HashMap;
use tonic::transport::{Certificate, Identity};

const CLIENT_CA_CERT: &[u8] = include_bytes!("../../../data/client_ca.pem");

#[tokio::test]
#[should_panic]
#[serial]
async fn wrong_server_cert_issuer() {
    async fn test() -> Result<()> {
        tokio::spawn(server::serve());
        let identity = Identity::from_pem(CLIENT_CERT, CLIENT_KEY);
        let server_ca_cert = Certificate::from_pem(CLIENT_CA_CERT);
        let claims = Claims::full_permission(USERNAME.into());

        let mut client = UnauthorizedClient::connect(ENDPOINT, identity, server_ca_cert)
            .await?
            .authorize(claims)
            .await?;

        client
            .spawn(
                "/usr/bin/echo".into(),
                ".".into(),
                vec!["hi pal".into()],
                HashMap::new(),
            )
            .await?;

        Ok(())
    }

    test().await.unwrap()
}
