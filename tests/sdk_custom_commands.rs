use couch_integration_htp1::{sdk::SdkClient, Settings};
use couch_sdk::{DeviceClient, Error as SdkError};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{sync::mpsc, thread, time::Duration};
use tokio::{net::TcpListener, runtime::Builder, time::timeout};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type TestSocket = WebSocketStream<tokio::net::TcpStream>;

async fn next_text(socket: &mut TestSocket) -> String {
    socket
        .next()
        .await
        .expect("websocket message")
        .expect("websocket frame")
        .into_text()
        .expect("text frame")
        .to_string()
}

async fn send_snapshot(socket: &mut TestSocket, state: &Value) {
    socket
        .send(Message::Text(format!("mso {state}")))
        .await
        .unwrap();
}

fn test_state() -> Value {
    json!({
        "powerIsOn": true,
        "muted": false,
        "volume": -30,
        "input": "h4",
        "cal": {"zeroPoint": 0, "vpl": -80, "vph": 12},
        "upmix": {"select": "native"},
        "inputs": {}
    })
}

fn start_server<F, Fut>(handler: F) -> (u16, thread::JoinHandle<()>)
where
    F: FnOnce(TestSocket) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (port_tx, port_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let runtime = Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async move {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            port_tx.send(port).unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let socket = accept_async(stream).await.unwrap();
            handler(socket).await;
        });
    });
    (port_rx.recv().unwrap(), server)
}

fn settings(port: u16) -> Settings {
    Settings {
        host: "127.0.0.1".into(),
        port,
        use_tls: false,
    }
}

#[test]
fn custom_upmix_commands_send_fixed_patches_and_confirm_fresh_state() {
    let commands = [
        ("x:upmix-auro", "auro"),
        ("x:upmix-dolby", "dolby"),
        ("x:upmix-dts", "dts"),
        ("x:upmix-native", "native"),
        ("x:upmix-stereo", "stereo"),
    ];
    let expected = commands;
    let (port, server) = start_server(move |mut socket| async move {
        assert_eq!(next_text(&mut socket).await, "getmso");
        let mut state = test_state();
        send_snapshot(&mut socket, &state).await;

        for (id, mode) in expected {
            // The adapter observes current state before it issues a command.
            assert_eq!(next_text(&mut socket).await, "getmso");
            send_snapshot(&mut socket, &state).await;

            let command = next_text(&mut socket).await;
            let patch: Value = serde_json::from_str(
                command
                    .strip_prefix("changemso ")
                    .expect("changemso command"),
            )
            .unwrap();
            assert_eq!(
                patch,
                json!([{
                    "op": "replace",
                    "path": "/upmix/select",
                    "value": mode
                }]),
                "patch for {id}"
            );
            state["upmix"]["select"] = json!(mode);

            // The adapter requests and checks a fresh snapshot after the patch.
            assert_eq!(next_text(&mut socket).await, "getmso");
            send_snapshot(&mut socket, &state).await;
        }
    });

    let mut client = SdkClient::connect(&settings(port)).unwrap();
    for (id, _) in commands {
        client.command(id).unwrap();
    }
    drop(client);
    server.join().unwrap();
}

#[test]
fn unknown_custom_id_is_rejected_without_sending_a_websocket_command() {
    let (port, server) = start_server(|mut socket| async move {
        assert_eq!(next_text(&mut socket).await, "getmso");
        send_snapshot(&mut socket, &test_state()).await;
        assert!(timeout(Duration::from_millis(250), socket.next())
            .await
            .is_err());
    });

    let mut client = SdkClient::connect(&settings(port)).unwrap();
    assert!(matches!(
        client.command("x:upmix-unknown"),
        Err(SdkError::Unsupported)
    ));
    server.join().unwrap();
}
