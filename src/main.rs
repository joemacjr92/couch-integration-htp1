//! Plugin executable entry point.
//!
//! The Couch plugin serve/adapter wiring is the next implementation step.
//! Keeping this explicit prevents an installable-looking binary from silently
//! claiming to control hardware before the DeviceClient contract is complete.

fn main() {
    eprintln!(
        "couch-plugin-htp1: initialization scaffold only; DeviceClient adapter is not implemented"
    );
    std::process::exit(2);
}
