//! High-level HTP-1 client: initial snapshot, cached state, patches and actions.
//!
//! TODO:
//! - connect and send `getmso`
//! - wait for initial `mso`
//! - apply `msoupdate` JSON patches to cached raw state
//! - serialize command writes and reconcile state from server notifications
//! - reconnect with bounded backoff and request a fresh snapshot
//! - implement power, volume, mute, input, upmix and Dirac operations

use crate::{Htp1State, Result, Settings};
use crate::transport::Transport;

pub struct Client {
    #[allow(dead_code)]
    settings: Settings,
    #[allow(dead_code)]
    transport: Transport,
    state: Htp1State,
}

impl Client {
    /// Connects the transport. The initial MSO handshake is intentionally not
    /// implemented yet; callers must not treat this as a ready device client.
    pub async fn connect_transport(settings: Settings) -> Result<Self> {
        let transport = Transport::connect(&settings).await?;
        Ok(Self {
            settings,
            transport,
            state: Htp1State::default(),
        })
    }

    pub fn cached_state(&self) -> &Htp1State {
        &self.state
    }
}
