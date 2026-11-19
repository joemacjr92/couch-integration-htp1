//! Synchronous CouchOS SDK adapter around the persistent async HTP-1 client.
//!
//! Keep one Tokio runtime alive for the lifetime of the WebSocket. Recreating
//! the runtime for each SDK call would also tear down its socket reactor.

use couch_sdk::{
    couch_model::commands::{valid_input_id, Function},
    Capability, ClientSettings, DeviceClient, Selectable, Status, VolumeDb,
};
use serde_json::{json, Value};
use tokio::runtime::{Builder, Runtime};

use crate::{Client, Error as Htp1Error, Htp1State, Settings};

pub struct SdkClient {
    runtime: Runtime,
    client: Client,
}

fn sdk_error(error: Htp1Error) -> couch_sdk::Error {
    match error {
        Htp1Error::Invalid => couch_sdk::Error::Invalid,
        Htp1Error::Transport(_) | Htp1Error::Disconnected => couch_sdk::Error::Transport,
        Htp1Error::Timeout => couch_sdk::Error::Timeout,
        Htp1Error::Protocol | Htp1Error::Json(_) => couch_sdk::Error::Protocol,
    }
}

impl ClientSettings for Settings {
    const FILE_PREFIX: &'static str = "monoprice-htp1";

    fn validate(&self) -> couch_sdk::Result<()> {
        Settings::validate(self).map_err(|_| couch_sdk::Error::Invalid)
    }
}

impl SdkClient {
    fn refresh(&mut self) -> couch_sdk::Result<()> {
        let runtime = &self.runtime;
        let client = &mut self.client;
        runtime.block_on(client.refresh_state()).map_err(sdk_error)
    }

    fn patch_and_confirm(&mut self, path: &str, value: Value) -> couch_sdk::Result<()> {
        let patch = json!([{"op": "replace", "path": path, "value": value}]);
        let runtime = &self.runtime;
        let client = &mut self.client;
        runtime
            .block_on(async move {
                client.send_patch(&patch).await?;
                client.refresh_state().await?;
                if client.cached_state().raw.pointer(path) != Some(&value) {
                    return Err(Htp1Error::Protocol);
                }
                Ok(())
            })
            .map_err(sdk_error)
    }
}

impl DeviceClient for SdkClient {
    type Settings = Settings;

    const KIND: &'static str = "monoprice-htp1";
    const LABEL: &'static str = "Monoprice HTP-1";

    fn capabilities() -> &'static [Capability] {
        &[
            ("power-on", "Power on"),
            ("power-off", "Power off"),
            ("volume-up", "Volume up (1 dB)"),
            ("volume-down", "Volume down (1 dB)"),
            ("mute", "Toggle mute"),
            ("mute-on", "Mute"),
            ("mute-off", "Unmute"),
        ]
    }

    fn connect(settings: &Settings) -> couch_sdk::Result<Self> {
        settings.validate().map_err(|_| couch_sdk::Error::Invalid)?;
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| couch_sdk::Error::Transport)?;
        let client = runtime
            .block_on(Client::connect_transport(settings.clone()))
            .map_err(sdk_error)?;
        Ok(Self { runtime, client })
    }

    fn execute(&mut self, function: &Function) -> couch_sdk::Result<()> {
        // Relative and toggle commands must use a fresh observation.
        self.refresh()?;
        match function {
            Function::PowerOn => self.patch_and_confirm("/powerIsOn", json!(true)),
            Function::PowerOff => self.patch_and_confirm("/powerIsOn", json!(false)),
            Function::MuteOn => self.patch_and_confirm("/muted", json!(true)),
            Function::MuteOff => self.patch_and_confirm("/muted", json!(false)),
            Function::Mute => {
                let muted = self
                    .client
                    .cached_state()
                    .muted
                    .ok_or(couch_sdk::Error::Protocol)?;
                self.patch_and_confirm("/muted", json!(!muted))
            }
            Function::VolumeUp | Function::VolumeDown => {
                let raw = &self.client.cached_state().raw;
                let current = raw
                    .get("volume")
                    .and_then(Value::as_i64)
                    .ok_or(couch_sdk::Error::Protocol)?;
                let cal = raw.get("cal").ok_or(couch_sdk::Error::Protocol)?;
                let minimum = cal
                    .get("vpl")
                    .and_then(Value::as_i64)
                    .ok_or(couch_sdk::Error::Protocol)?;
                let maximum = cal
                    .get("vph")
                    .and_then(Value::as_i64)
                    .ok_or(couch_sdk::Error::Protocol)?;
                let next = match function {
                    Function::VolumeUp if current < maximum => current + 1,
                    Function::VolumeDown if current > minimum => current - 1,
                    _ => current,
                };
                if next == current {
                    Ok(())
                } else {
                    self.patch_and_confirm("/volume", json!(next))
                }
            }
            Function::Input(id) => {
                let inputs = self
                    .client
                    .cached_state()
                    .raw
                    .get("inputs")
                    .and_then(Value::as_object)
                    .ok_or(couch_sdk::Error::Protocol)?;
                if !inputs.contains_key(id) {
                    return Err(couch_sdk::Error::Unsupported);
                }
                self.patch_and_confirm("/input", json!(id))
            }
            _ => Err(couch_sdk::Error::Unsupported),
        }
    }

    fn status(&mut self) -> couch_sdk::Result<Status> {
        self.refresh()?;
        Ok(status_from_state(self.client.cached_state()))
    }

    fn inputs(&mut self) -> couch_sdk::Result<Vec<Selectable>> {
        self.refresh()?;
        let inputs = self
            .client
            .cached_state()
            .raw
            .get("inputs")
            .and_then(Value::as_object)
            .ok_or(couch_sdk::Error::Protocol)?;
        Ok(inputs
            .iter()
            .filter_map(|(id, details)| {
                if details.get("visible").and_then(Value::as_bool) != Some(true) {
                    return None;
                }
                let name = details.get("label").and_then(Value::as_str).unwrap_or(id);
                Some(Selectable::new(id.clone(), name))
            })
            .collect())
    }

    fn supports_input(id: &str) -> bool {
        valid_input_id(id)
    }
}

fn status_from_state(state: &Htp1State) -> Status {
    let mut status = Status::default();
    status.on = state.power_is_on;
    status.muted = state.muted;
    status.input = state.input.clone();

    let raw = &state.raw;
    if let (Some(volume), Some(cal)) = (raw.get("volume").and_then(Value::as_f64), raw.get("cal")) {
        let zero_point = cal.get("zeroPoint").and_then(Value::as_f64).unwrap_or(0.0);
        let minimum = cal.get("vpl").and_then(Value::as_f64);
        if minimum.is_some_and(|minimum| volume <= minimum) {
            status.volume_db = Some(VolumeDb::Minimum);
        } else {
            let tenths = ((volume - zero_point) * 10.0).round();
            if (i16::MIN as f64..=i16::MAX as f64).contains(&tenths) {
                status.volume_db = Some(VolumeDb::Reading {
                    tenths: tenths as i16,
                });
            }
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_ids_use_the_model_token_grammar() {
        assert!(SdkClient::supports_input("h4"));
        assert!(SdkClient::supports_input("SAT/CBL"));
        assert!(!SdkClient::supports_input(""));
        assert!(!SdkClient::supports_input("bad\rinput"));
    }

    #[test]
    fn status_reports_observed_htp1_values() {
        let state = Htp1State::from_mso(json!({
            "powerIsOn": true,
            "muted": false,
            "input": "h4",
            "volume": -30,
            "cal": {"zeroPoint": 0, "vpl": -80, "vph": 12}
        }));
        let status = status_from_state(&state);
        assert_eq!(status.on, Some(true));
        assert_eq!(status.muted, Some(false));
        assert_eq!(status.input.as_deref(), Some("h4"));
        assert_eq!(status.volume_db, Some(VolumeDb::Reading { tenths: -300 }));
    }

    #[test]
    fn minimum_volume_is_reported_as_the_explicit_minimum_sentinel() {
        let state = Htp1State::from_mso(json!({
            "volume": -80,
            "cal": {"zeroPoint": 0, "vpl": -80}
        }));
        assert_eq!(status_from_state(&state).volume_db, Some(VolumeDb::Minimum));
    }
    #[test]
    fn embedded_manifest_matches_the_sdk_contract() {
        let manifest: couch_plugin::Manifest =
            serde_json::from_str(include_str!("../plugin.json")).unwrap();
        manifest.validate().unwrap();
        assert_eq!(manifest.id, <SdkClient as DeviceClient>::KIND);
        let manifest_capabilities: Vec<_> = manifest
            .capabilities
            .iter()
            .map(|capability| (capability.id.as_str(), capability.label.as_str()))
            .collect();
        let sdk_capabilities: Vec<_> = <SdkClient as DeviceClient>::capabilities()
            .iter()
            .map(|(id, label)| (*id, *label))
            .collect();
        assert_eq!(manifest_capabilities, sdk_capabilities);
        assert!(manifest.supports_inputs);
    }
}
