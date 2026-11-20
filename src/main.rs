//! Standalone CouchOS plugin entry point.
//!
//! stdout is reserved for the framed plugin protocol; do not print or log here.

fn main() {
    let manifest = serde_json::from_str(include_str!("../plugin.json"))
        .expect("embedded HTP-1 plugin manifest");
    if couch_plugin::serve::<couch_integration_htp1::sdk::SdkClient>(manifest).is_err() {
        std::process::exit(1);
    }
}
