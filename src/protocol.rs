//! HTP-1 WebSocket message parsing and command encoding.
//!
//! Observed protocol message families from the original integration:
//! - `mso <JSON>`: complete state snapshot
//! - `msoupdate <JSON patch>`: incremental state update
//! - `getmso`: request a full snapshot
//! - `changemso <JSON patch operations>`: request a state change
//!
//! TODO: verify the exact JSON Patch representation and patch semantics against
//! the original implementation before enabling command writes.

use serde_json::Value;

use crate::{Error, Result, Htp1State};

#[derive(Clone, Debug)]
pub enum ServerMessage {
    FullState(Htp1State),
    StatePatch(Value),
    Other(String),
}

pub fn parse_server_message(message: &str) -> Result<ServerMessage> {
    if let Some(json) = message.strip_prefix("mso ") {
        return Ok(ServerMessage::FullState(Htp1State::from_mso(
            serde_json::from_str(json)?,
        )));
    }
    if let Some(json) = message.strip_prefix("msoupdate ") {
        return Ok(ServerMessage::StatePatch(serde_json::from_str(json)?));
    }
    if message == "mso" || message == "msoupdate" {
        return Err(Error::Protocol);
    }
    Ok(ServerMessage::Other(message.to_owned()))
}

pub fn get_mso_command() -> &'static str {
    "getmso"
}

/// Build a `changemso` command from already-validated patch operations.
/// Callers must not pass arbitrary user-provided JSON without validation.
pub fn change_mso_command(patch: &Value) -> Result<String> {
    if !patch.is_array() {
        return Err(Error::Invalid);
    }
    Ok(format!("changemso {}", serde_json::to_string(patch)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_full_state_and_patch_frames() {
        let full = parse_server_message(r#"mso {"powerIsOn":true,"muted":false,"volume":-30,"input":"h4"}"#).unwrap();
        assert!(matches!(full, ServerMessage::FullState(_)));
        let patch = parse_server_message(r#"msoupdate [{"op":"replace","path":"/muted","value":true}]"#).unwrap();
        assert!(matches!(patch, ServerMessage::StatePatch(_)));
        assert_eq!(get_mso_command(), "getmso");
    }
}
