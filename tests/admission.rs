//! Exercise the production executable through Couch's package host.
//!
//! The Hello handshake checks that the manifest embedded in the executable
//! matches the manifest supplied by the package.

use couch_plugin::testing::{Adapter, Package};
use std::path::Path;

#[test]
#[ignore = "runs in GitHub Actions; local WSL permissions may prevent execution"]
fn executable_manifest_matches_the_package_manifest() {
    let package = Package::new(Adapter {
        binary: Path::new(env!("CARGO_BIN_EXE_couch-plugin-htp1")),
        manifest_json: include_str!("../plugin.json"),
    });

    let _host = package.host();
}
