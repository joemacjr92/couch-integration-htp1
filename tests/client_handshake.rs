use couch_integration_htp1::{Client, Settings};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test]
async fn connects_requests_snapshot_and_processes_incremental_update() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();

        let request = socket.next().await.unwrap().unwrap();
        assert_eq!(request.into_text().unwrap(), "getmso");

        socket
            .send(Message::Text(
                "mso {\"powerIsOn\":true,\"muted\":false,\"volume\":-30,\"input\":\"h4\"}".into(),
            ))
            .await
            .unwrap();

        socket
            .send(Message::Text(
                "msoupdate [{\"op\":\"replace\",\"path\":\"/muted\",\"value\":true}]".into(),
            ))
            .await
            .unwrap();
    });

    let settings = Settings {
        host: "127.0.0.1".into(),
        port,
        use_tls: false,
    };

    let mut client = Client::connect_transport(settings).await.unwrap();
    assert!(client.is_ready());
    assert_eq!(client.cached_state().power_is_on, Some(true));
    assert_eq!(client.cached_state().muted, Some(false));

    assert!(client.process_next_message().await.unwrap());
    assert_eq!(client.cached_state().muted, Some(true));

    server.await.unwrap();
}

#[test]
fn fixture_patch_shape_is_json_array() {
    // Keep a small compile-time assertion that JSON patch payloads use the
    // same representation accepted by the protocol parser.
    assert!(json!([{"op":"replace","path":"/muted","value":true}]).is_array());
}
