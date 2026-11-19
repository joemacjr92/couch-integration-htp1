use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Connection settings. The HTP-1 WebSocket controller normally uses port 80;
/// HTTPS deployments can use port 443.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub use_tls: bool,
}

const fn default_port() -> u16 {
    80
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.host.is_empty()
            || self.host.len() > 253
            || self
                .host
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '/' | '@' | '\\'))
            || self.port == 0
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }

    pub fn websocket_url(&self) -> Result<String> {
        self.validate()?;
        let scheme = if self.use_tls { "wss" } else { "ws" };
        Ok(format!(
            "{scheme}://{}:{}/ws/controller",
            self.host, self.port
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_websocket_urls() {
        let plain = Settings {
            host: "htp1.local".into(),
            port: 80,
            use_tls: false,
        };
        let tls = Settings {
            host: "10.0.0.8".into(),
            port: 443,
            use_tls: true,
        };
        assert_eq!(
            plain.websocket_url().unwrap(),
            "ws://htp1.local:80/ws/controller"
        );
        assert_eq!(
            tls.websocket_url().unwrap(),
            "wss://10.0.0.8:443/ws/controller"
        );
    }
}
