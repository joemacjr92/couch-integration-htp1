use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parsed HTP-1 state snapshot.
///
/// Keep the raw JSON alongside typed fields during early protocol development:
/// the HTP-1 MSO schema is broad and firmware-dependent. Promote fields into
/// typed members as their shape and semantics are verified.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Htp1State {
    #[serde(default)]
    pub raw: Value,
    pub power_is_on: Option<bool>,
    pub muted: Option<bool>,
    pub volume: Option<f64>,
    pub input: Option<String>,
    pub upmix_select: Option<String>,
    pub night: Option<String>,
    pub current_dirac_slot: Option<Value>,
}

impl Htp1State {
    pub fn from_mso(raw: Value) -> Self {
        let mut state = Self::default();
        state.power_is_on = raw.get("powerIsOn").and_then(Value::as_bool);
        state.muted = raw.get("muted").and_then(Value::as_bool);
        state.volume = raw.get("volume").and_then(Value::as_f64);
        state.input = raw.get("input").and_then(Value::as_str).map(str::to_owned);
        state.upmix_select = raw.pointer("/upmix/select").and_then(Value::as_str).map(str::to_owned);
        state.night = raw.get("night").and_then(Value::as_str).map(str::to_owned);
        state.current_dirac_slot = raw.pointer("/cal/currentdiracslot").cloned();
        state.raw = raw;
        state
    }
}
