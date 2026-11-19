use couch_integration_htp1::protocol::apply_state_patch;
use serde_json::json;

#[test]
fn applies_object_and_array_patch_operations() {
    let mut state = json!({
        "muted": false,
        "upmix": {"select": "native"},
        "inputs": ["a", "b"]
    });

    apply_state_patch(
        &mut state,
        &json!([
            {"op": "replace", "path": "/muted", "value": true},
            {"op": "replace", "path": "/upmix/select", "value": "stereo"},
            {"op": "add", "path": "/inputs/1", "value": "new"},
            {"op": "remove", "path": "/inputs/0"}
        ]),
    )
    .unwrap();

    assert_eq!(state["muted"], json!(true));
    assert_eq!(state["upmix"]["select"], json!("stereo"));
    assert_eq!(state["inputs"], json!(["new", "b"]));
}

#[test]
fn decodes_json_pointer_escapes() {
    let mut state = json!({"a/b": {"~key": 1}});
    apply_state_patch(
        &mut state,
        &json!({"op": "replace", "path": "/a~1b/~0key", "value": 2}),
    )
    .unwrap();
    assert_eq!(state["a/b"]["~key"], json!(2));
}

#[test]
fn failed_patch_does_not_partially_modify_state() {
    let mut state = json!({"muted": false, "volume": -30});
    let before = state.clone();

    let result = apply_state_patch(
        &mut state,
        &json!([
            {"op": "replace", "path": "/muted", "value": true},
            {"op": "replace", "path": "/missing", "value": 1}
        ]),
    );

    assert!(result.is_err());
    assert_eq!(state, before);
}

#[test]
fn rejects_invalid_array_indexes() {
    let mut state = json!({"items": [1, 2]});
    assert!(apply_state_patch(
        &mut state,
        &json!({"op": "replace", "path": "/items/01", "value": 3}),
    ).is_err());
    assert_eq!(state["items"], json!([1, 2]));
}
