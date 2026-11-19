//! High-level HTP-1 client: initial snapshot and cached state updates.

use std::time::Duration;

use serde_json::Value;
use tokio::time::timeout;

use crate::protocol::{
    apply_state_patch, change_mso_command, get_mso_command, parse_server_message, ServerMessage,
};
use crate::transport::Transport;
use crate::{Error, Htp1State, Result, Settings};

const INITIAL_STATE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Client {
    transport: Transport,
    state: Htp1State,
    ready: bool,
}

impl Client {
    /// Connect, request the full MSO snapshot and wait until the device is ready.
    pub async fn connect_transport(settings: Settings) -> Result<Self> {
        let mut transport = Transport::connect(&settings).await?;
        transport.send_text(get_mso_command()).await?;

        let initial_state = timeout(INITIAL_STATE_TIMEOUT, async {
            loop {
                let message = transport.receive_text().await?;
                match parse_server_message(&message)? {
                    ServerMessage::FullState(state) => return Ok::<Htp1State, Error>(state),
                    ServerMessage::StatePatch(_) | ServerMessage::Other(_) => {}
                }
            }
        })
        .await
        .map_err(|_| Error::Timeout)??;

        Ok(Self {
            transport,
            state: initial_state,
            ready: true,
        })
    }

    pub fn cached_state(&self) -> &Htp1State {
        &self.state
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Send one already-validated MSO patch transaction.
    pub async fn send_patch(&mut self, patch: &Value) -> Result<()> {
        let command = change_mso_command(patch)?;
        self.transport.send_text(command).await
    }

    /// Request and cache a fresh full state snapshot. Incremental updates
    /// received while waiting are applied to the existing snapshot.
    pub async fn refresh_state(&mut self) -> Result<()> {
        self.transport.send_text(get_mso_command()).await?;
        timeout(INITIAL_STATE_TIMEOUT, async {
            loop {
                let message = self.transport.receive_text().await?;
                match parse_server_message(&message)? {
                    ServerMessage::FullState(state) => {
                        self.state = state;
                        self.ready = true;
                        return Ok::<(), Error>(());
                    }
                    ServerMessage::StatePatch(patch) if self.ready => {
                        apply_state_patch(&mut self.state.raw, &patch)?;
                        self.state = Htp1State::from_mso(self.state.raw.clone());
                    }
                    ServerMessage::StatePatch(_) | ServerMessage::Other(_) => {}
                }
            }
        })
        .await
        .map_err(|_| Error::Timeout)?
    }

    /// Receive and process one server message.
    pub async fn process_next_message(&mut self) -> Result<bool> {
        let message = self.transport.receive_text().await?;
        match parse_server_message(&message)? {
            ServerMessage::FullState(state) => {
                self.state = state;
                self.ready = true;
                Ok(true)
            }
            ServerMessage::StatePatch(patch) => {
                if !self.ready {
                    return Ok(false);
                }
                apply_state_patch(&mut self.state.raw, &patch)?;
                self.state = Htp1State::from_mso(self.state.raw.clone());
                Ok(true)
            }
            ServerMessage::Other(_) => Ok(false),
        }
    }
}
