use super::{DOMAIN, ENDPOINT, USERNAME};
use crate::client::{Claims, Jwt, UnauthorizedClient};
use crate::{CLIENT_CERT, CLIENT_KEY, SERVER_CA_CERT};
use anyhow::Result;
use serial_test::serial;
use server::server;
use std::collections::HashMap;
use tonic::transport::{Certificate, Identity};

#[tokio::test]
#[serial]
async fn success() {
    async fn test() -> Result<()> {
        tokio::spawn(server::serve());
        let mut client = crate::init_client(USERNAME.into(), ENDPOINT, DOMAIN).await?;

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

#[tokio::test]
#[should_panic]
#[serial]
async fn invalid_jwt_signature() {
    async fn test() -> Result<()> {
        const FAKE_JWT: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

        let token = Jwt(FAKE_JWT.into());
        tokio::spawn(server::serve());
        let identity = Identity::from_pem(CLIENT_CERT, CLIENT_KEY);
        let server_ca_cert = Certificate::from_pem(SERVER_CA_CERT);

        let mut client = UnauthorizedClient::connect(ENDPOINT, DOMAIN, identity, server_ca_cert)
            .await?
            .with_token(token)?;

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

#[tokio::test]
#[should_panic]
#[serial]
async fn missing_privileges() {
    async fn test() -> Result<()> {
        tokio::spawn(server::serve());
        let identity = Identity::from_pem(CLIENT_CERT, CLIENT_KEY);
        let server_ca_cert = Certificate::from_pem(SERVER_CA_CERT);

        let mut unauthorized_client =
            UnauthorizedClient::connect(ENDPOINT, DOMAIN, identity, server_ca_cert).await?;

        let claims = Claims {
            username: USERNAME.into(),
            spawn: false,
            stop: true,
            status: true,
            stream_log: true,
        };

        let token = unauthorized_client.issue_jwt(claims).await?;
        let mut client = unauthorized_client.with_token(token)?;

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
