# CouchOS Monoprice HTP-1 integration

Initial Rust project scaffold for a CouchOS integration targeting the
Monoprice HTP-1. This is a development starting point, **not an installable
or working integration yet**.

The original Unfolded Circle integration by Meir Miyara
([mase1981/uc-intg-monoprice-htp1](https://github.com/mase1981/uc-intg-monoprice-htp1))
is the protocol and feature reference for this adaptation. Preserve attribution
to Meir Miyara in future documentation and release materials. CouchOS and its
SDK are authored/maintained by dangerouslaser.

## Included structure

- `Cargo.toml` — Rust package, Couch SDK dependencies pinned to one Couch commit,
  async WebSocket dependencies, release profile.
- `plugin.json` — deliberately empty capability/presentation contract while
  the Couch model support is checked. Do not install this manifest.
- `src/transport.rs` — persistent WebSocket connection and text-frame handling.
- `src/protocol.rs` — initial `mso` / `msoupdate` message recognition and
  `getmso` / `changemso` helpers.
- `src/state.rs` — initial typed fields plus retained raw MSO JSON.
- `src/client.rs` — client ownership boundary and implementation TODOs.
- `src/settings.rs` — host, port and TLS settings.
- `src/sdk.rs` — reserved for the Couch `DeviceClient` adapter.
- `tests/` — reserved for mock WebSocket and SDK contract tests.

## Build

From WSL:

```bash
cd couch-integration-htp1
cargo test
cargo check
```

The first Cargo build will fetch the Couch Git dependency and Rust crates.
The pinned Couch revision is:

`00ab4da2e6336a925e93702507c0b7b233012738`

Keep `couch-plugin` and `couch-sdk` on the same exact revision.

## Current limitations

- The executable is an explicit placeholder and exits with status 2.
- No Couch `DeviceClient` implementation or plugin serving loop exists yet.
- The initial MSO handshake and JSON Patch application are not implemented.
- No device commands are enabled.
- `plugin.json` advertises no capabilities and is not ready for installation.
- No hardware connection or command has been tested.

The next development steps are to verify the Couch model/catalog contract,
complete the WebSocket handshake and state patching, add mock-server tests,
then implement the SDK adapter and only then populate the manifest.

## Scope

Media browsing and BEQ catalogue browsing are intentionally excluded from
this project.
