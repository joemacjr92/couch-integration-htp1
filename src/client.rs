//! High-level HTP-1 client: initial snapshot and cached state updates.

use std::time::Duration;

use tokio::time::timeout;

use crate::protocol::{apply_state_patch, get_mso_command, parse_server_message, ServerMessage};
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
                    ServerMessage::FullState(state) => return Ok(state),
                    // Incremental updates cannot be applied until the initial
                    // snapshot has established the document they modify.
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

    /// True after the initial full-state response has been received.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Receive and process one server message.
    ///
    /// Returns true when the cached state changed. Unknown messages and
    /// patches received before an initial snapshot do not change state.
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
