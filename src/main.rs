//! Standalone CouchOS plugin entry point.

fn main() {
    let manifest = serde_json::from_str(include_str!("../plugin.json"))
        .expect("embedded HTP-1 plugin manifest");
    if couch_plugin::serve::<couch_integration_htp1::sdk::SdkClient>(manifest).is_err() {
        std::process::exit(1);
    }
}
