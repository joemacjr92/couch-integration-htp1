use couch_integration_htp1::protocol::{
    parse_server_message,
    ServerMessage,
};
use serde_json::json;

const HTP1_RESPONSE: &str = include_str!("fixtures/responses.txt");

#[test]
fn parses_complete_htp1_response_fixture() {
    let message = parse_server_message(HTP1_RESPONSE.trim())
        .expect("HTP-1 response fixture should parse");

    let state = match message {
        ServerMessage::FullState(state) => state,
        other => panic!("expected FullState, got {other:?}"),
    };

    // Verify the typed fields extracted from the real HTP-1 snapshot.
    assert_eq!(state.power_is_on, Some(false));
    assert_eq!(state.muted, Some(false));
    assert_eq!(state.volume, Some(-30.0));
    assert_eq!(state.input.as_deref(), Some("h4"));
    assert_eq!(state.upmix_select.as_deref(), Some("native"));
    assert_eq!(state.night.as_deref(), Some("off"));
    assert_eq!(state.current_dirac_slot, Some(json!(0)));

    // Verify that the complete original JSON remains available.
    assert!(state.raw.get("speakers").is_some());
    assert!(state.raw.get("inputs").is_some());
    assert!(state.raw.get("videostat").is_some());
    assert!(state.raw.get("cal").is_some());
}
