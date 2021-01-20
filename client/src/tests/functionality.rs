use super::{ENDPOINT, USERNAME};
use anyhow::{anyhow, Result};
use futures::StreamExt;
use protocol::stream_log_response;
use serial_test::serial;
use server::server;
use std::collections::HashMap;

#[tokio::test]
#[serial]
async fn spawn_capture_verify_stdout() {
    async fn test() -> Result<()> {
        tokio::spawn(server::serve());
        let mut client = crate::init_client(USERNAME.into(), ENDPOINT).await?;

        let uuid = client
            .spawn(
                "/bin/bash".into(),
                ".".into(),
                vec!["-c".into(), "echo hi".into()],
                HashMap::new(),
            )
            .await?;

        let events: Vec<_> = client.stream_log(uuid, true).await?.take(2).collect().await;
        let response = events[0].as_ref().unwrap().response.as_ref().unwrap();

        if let stream_log_response::Response::Stdout(inner) = response {
            assert_eq!(inner.output, b"hi\n");
            assert!(matches!(
                events[1].as_ref().unwrap().response.as_ref().unwrap(),
                stream_log_response::Response::Exit(_)
            ));

            Ok(())
        } else {
            Err(anyhow!("wrong event type"))
        }
    }

    test().await.unwrap()
}
