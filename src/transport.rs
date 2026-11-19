//! Persistent WebSocket transport.
//!
//! This module owns the socket. It deliberately does not interpret MSO JSON;
//! that belongs in `protocol.rs` and `client.rs`.

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{Error, Result, Settings};

pub struct Transport {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Transport {
    pub async fn connect(settings: &Settings) -> Result<Self> {
        let url = settings.websocket_url()?;
        let (socket, _) = connect_async(url)
            .await
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(Self { socket })
    }

    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<()> {
        self.socket
            .send(Message::Text(text.into().into()))
            .await
            .map_err(|e| Error::Transport(e.to_string()))
    }

    pub async fn receive_text(&mut self) -> Result<String> {
        loop {
            match self.socket.next().await {
                Some(Ok(Message::Text(text))) => return Ok(text.to_string()),
                Some(Ok(Message::Ping(payload))) => {
                    self.socket
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|e| Error::Transport(e.to_string()))?;
                }
                Some(Ok(Message::Close(_))) | None => return Err(Error::Disconnected),
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(Error::Transport(e.to_string())),
            }
        }
    }
}
