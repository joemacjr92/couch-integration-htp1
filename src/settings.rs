use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv6Addr};

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

/// Accept a DNS hostname, IPv4 address, or IPv6 address. IPv6 may be supplied
/// either bare or bracketed; URL formatting adds brackets when required.
fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 253 || host.trim() != host {
        return false;
    }

    if host.starts_with('[') || host.ends_with(']') {
        return host
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
            .and_then(|value| value.parse::<Ipv6Addr>().ok())
            .is_some();
    }

    if host.parse::<IpAddr>().is_ok() {
        return true;
    }

    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !valid_host(&self.host) || self.port == 0 {
            return Err(Error::Invalid);
        }
        Ok(())
    }

    pub fn websocket_url(&self) -> Result<String> {
        self.validate()?;
        let scheme = if self.use_tls { "wss" } else { "ws" };
        let host = if self.host.parse::<Ipv6Addr>().is_ok() {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        Ok(format!("{scheme}://{host}:{}/ws/controller", self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(host: &str) -> Settings {
        Settings {
            host: host.into(),
            port: 80,
            use_tls: false,
        }
    }

    #[test]
    fn builds_expected_websocket_urls() {
        let plain = settings("htp1.local");
        let tls = Settings {
            host: "10.0.0.8".into(),
            port: 443,
            use_tls: true,
        };
        let ipv6 = settings("fe80::1");
        let bracketed_ipv6 = settings("[fe80::1]");

        assert_eq!(
            plain.websocket_url().unwrap(),
            "ws://htp1.local:80/ws/controller"
        );
        assert_eq!(
            tls.websocket_url().unwrap(),
            "wss://10.0.0.8:443/ws/controller"
        );
        assert_eq!(
            ipv6.websocket_url().unwrap(),
            "ws://[fe80::1]:80/ws/controller"
        );
        assert_eq!(
            bracketed_ipv6.websocket_url().unwrap(),
            "ws://[fe80::1]:80/ws/controller"
        );
    }

    #[test]
    fn accepts_dns_ipv4_and_ipv6_hosts() {
        for host in [
            "htp1.local",
            "receiver.example.net",
            "10.0.0.8",
            "fe80::1",
            "[fe80::1]",
        ] {
            assert!(settings(host).validate().is_ok(), "rejected {host:?}");
        }
    }

    #[test]
    fn rejects_invalid_hosts_and_zero_port() {
        for host in [
            "",
            " htp1.local",
            "htp1.local ",
            "https://htp1.local",
            "htp1.local/path",
            "htp1.local?query",
            "htp1.local#fragment",
            "htp1.local\\other",
            "bad..host",
            "-bad.example",
            "bad-.example",
            "bad_host",
            "[not-ipv6]",
            "[fe80::1",
            "fe80::1]",
            "fe80::zz",
        ] {
            assert!(settings(host).validate().is_err(), "accepted {host:?}");
        }

        let mut zero_port = settings("htp1.local");
        zero_port.port = 0;
        assert!(zero_port.validate().is_err());
    }
}
