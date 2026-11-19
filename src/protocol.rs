//! HTP-1 WebSocket message parsing, state patches and command encoding.
//!
//! Observed protocol message families:
//! - `mso <JSON>`: complete state snapshot
//! - `msoupdate <JSON patch>`: incremental state update
//! - `getmso`: request a full snapshot
//! - `changemso <JSON patch operations>`: request a state change

use serde_json::Value;

use crate::{Error, Htp1State, Result};

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

/// Apply one HTP-1 `msoupdate` payload to a raw MSO state document.
///
/// The HTP-1 sends either a single operation or an array of operations. The
/// supported operations observed in the original integration are add,
/// replace and remove. Updates are applied to a clone first, so a malformed
/// operation cannot leave the cached state half-updated.
pub fn apply_state_patch(target: &mut Value, patch: &Value) -> Result<()> {
    let operations: Vec<&Value> = match patch {
        Value::Array(items) => items.iter().collect(),
        Value::Object(_) => vec![patch],
        _ => return Err(Error::Protocol),
    };

    let mut updated = target.clone();
    for operation in operations {
        apply_patch_operation(&mut updated, operation)?;
    }
    *target = updated;
    Ok(())
}

fn apply_patch_operation(target: &mut Value, operation: &Value) -> Result<()> {
    let op = operation
        .get("op")
        .and_then(Value::as_str)
        .ok_or(Error::Protocol)?;
    let path = operation
        .get("path")
        .and_then(Value::as_str)
        .ok_or(Error::Protocol)?;
    let segments = parse_json_pointer(path)?;
    let value = operation.get("value");

    if segments.is_empty() {
        return match op {
            "add" | "replace" => {
                *target = value.cloned().ok_or(Error::Protocol)?;
                Ok(())
            }
            "remove" => {
                *target = Value::Null;
                Ok(())
            }
            _ => Err(Error::Protocol),
        };
    }

    let (last, parents) = segments.split_last().ok_or(Error::Protocol)?;
    let mut parent = target;
    for segment in parents {
        parent = match parent {
            Value::Object(object) => object.get_mut(segment).ok_or(Error::Protocol)?,
            Value::Array(array) => {
                let index = parse_array_index(segment, array.len(), false)?;
                array.get_mut(index).ok_or(Error::Protocol)?
            }
            _ => return Err(Error::Protocol),
        };
    }

    match parent {
        Value::Object(object) => match op {
            "add" => {
                object.insert(last.clone(), value.cloned().ok_or(Error::Protocol)?);
                Ok(())
            }
            "replace" => {
                let entry = object.get_mut(last).ok_or(Error::Protocol)?;
                *entry = value.cloned().ok_or(Error::Protocol)?;
                Ok(())
            }
            "remove" => {
                object.remove(last).ok_or(Error::Protocol)?;
                Ok(())
            }
            _ => Err(Error::Protocol),
        },
        Value::Array(array) => match op {
            "add" => {
                let new_value = value.cloned().ok_or(Error::Protocol)?;
                if last == "-" {
                    array.push(new_value);
                } else {
                    let index = parse_array_index(last, array.len(), true)?;
                    array.insert(index, new_value);
                }
                Ok(())
            }
            "replace" => {
                let index = parse_array_index(last, array.len(), false)?;
                let entry = array.get_mut(index).ok_or(Error::Protocol)?;
                *entry = value.cloned().ok_or(Error::Protocol)?;
                Ok(())
            }
            "remove" => {
                let index = parse_array_index(last, array.len(), false)?;
                if index >= array.len() {
                    return Err(Error::Protocol);
                }
                array.remove(index);
                Ok(())
            }
            _ => Err(Error::Protocol),
        },
        _ => Err(Error::Protocol),
    }
}

fn parse_json_pointer(path: &str) -> Result<Vec<String>> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    if !path.starts_with('/') {
        return Err(Error::Protocol);
    }

    path[1..]
        .split('/')
        .map(|segment| {
            let mut decoded = String::with_capacity(segment.len());
            let mut chars = segment.chars();
            while let Some(ch) = chars.next() {
                if ch == '~' {
                    match chars.next() {
                        Some('0') => decoded.push('~'),
                        Some('1') => decoded.push('/'),
                        _ => return Err(Error::Protocol),
                    }
                } else {
                    decoded.push(ch);
                }
            }
            Ok(decoded)
        })
        .collect()
}

fn parse_array_index(segment: &str, len: usize, allow_end: bool) -> Result<usize> {
    if segment.is_empty()
        || (segment.len() > 1 && segment.starts_with('0'))
        || !segment.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(Error::Protocol);
    }
    let index = segment.parse::<usize>().map_err(|_| Error::Protocol)?;
    let max = if allow_end {
        len
    } else {
        len.saturating_sub(1)
    };
    if index > max || (!allow_end && index >= len) {
        return Err(Error::Protocol);
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_full_state_and_patch_frames() {
        let full = parse_server_message(
            r#"mso {"powerIsOn":true,"muted":false,"volume":-30,"input":"h4"}"#,
        )
        .unwrap();
        assert!(matches!(full, ServerMessage::FullState(_)));
        let patch =
            parse_server_message(r#"msoupdate [{"op":"replace","path":"/muted","value":true}]"#)
                .unwrap();
        assert!(matches!(patch, ServerMessage::StatePatch(_)));
        assert_eq!(get_mso_command(), "getmso");
    }
}
