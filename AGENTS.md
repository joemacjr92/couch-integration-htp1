# Project Instructions

## Purpose and attribution

This repository is a Rust adaptation of Meir Miyara's original Python
Unfolded Circle Monoprice HTP-1 integration:
[mase1981/uc-intg-monoprice-htp1](https://github.com/mase1981/uc-intg-monoprice-htp1).
Preserve attribution to Meir Miyara in project and release materials. CouchOS
and its SDK are authored/maintained by dangerouslaser; preserve that attribution
as well.

Media browsing and BEQ catalogue browsing are out of scope.

## Architecture and protocol

- The HTP-1 controller is reached over WebSocket at `/ws/controller`. Settings
  provide host, port (default 80), and TLS selection; the URL uses `ws` or
  `wss` accordingly.
- On connection, send `getmso` and wait up to five seconds for a complete
  `mso <JSON>` state snapshot. The initial snapshot establishes the cached
  state.
- A later full `mso` snapshot replaces the cached state. `msoupdate <JSON
  patch>` carries JSON Patch operations that are applied to the existing raw
  MSO document. Keep the raw JSON because the HTP-1 schema is broad and may
  vary with firmware.
- State-changing commands are encoded as `changemso <JSON patch array>`.
  The client handles text frames and WebSocket ping/pong; connection and
  protocol errors are surfaced through the integration error types.

## CouchOS SDK integration

- `SdkClient` implements the synchronous CouchOS `DeviceClient` interface and
  owns both the HTP-1 client and a persistent Tokio current-thread runtime.
  Keep that runtime alive for the WebSocket lifetime; do not create and drop a
  runtime for each SDK call.
- `couch-plugin` and `couch-sdk` are pinned to the same exact Git revision in
  `Cargo.toml`. The current reviewed Protocol 3 pin is
  `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`; keep both pins and `Cargo.lock`
  synchronized. The SDK source also contains later protocol APIs, but this
  package declares and uses Protocol 3 only.
- Commands refresh device state before acting. A command sends its
  `changemso` patch, requests a fresh snapshot, and verifies the requested
  value before reporting success. Relative volume and mute-toggle decisions
  must use freshly observed state.
- Command tests use simulated HTP-1 snapshots. Their refresh, patch, and
  confirmation checks validate the software path; they do not verify behavior
  on physical HTP-1 hardware.
- Refresh status and visible inputs from device state. Do not claim successful
  compilation or APK packaging proves device functionality.

## Implemented capabilities

- Power on and off.
- Volume up and down by 1 dB, bounded by the device calibration limits.
- Mute toggle, mute on, and mute off.
- Dynamic selection of visible HTP-1 inputs.
- Status reporting for power, mute, input, and volume.
- Protocol 3 custom commands for the explicit upmix modes Auro, Dolby, DTS,
  Native, and Stereo. Each ID maps to a fixed `/upmix/select` replacement and
  is verified against a refreshed snapshot; the tests use simulated snapshots
  and do not verify behavior on physical HTP-1 hardware.

Dirac, night mode, PEQ, and seat-shaker controls are not implemented. The
`off` and `mono` upmix values remain unresolved and are not advertised. Do not
add arbitrary `x:` payloads or Protocol 4/5 features in the integration.

## Development and verification

Run from the repository root:

```sh
cargo fmt --check
cargo test
cargo check
```

After Rust source changes, run `cargo fmt` and `cargo test`. The tests include
SDK/manifest contract checks, protocol and patch cases, and a local WebSocket
handshake test. The handshake test needs local socket permissions.

The packaging/deployment workflow is `tools/build-deploy.sh`. It cross-compiles
for `armv7-unknown-linux-musleabihf` using the adjacent
`../couch-sdk-reference` checkout and pinned Zig toolchain, then invokes that
SDK checkout's `tools/integrations/build-apk.sh` in Alpine ARMv7 to package and
sign. It can offer an optional HA100 sideload after packaging. Inspect the
script and its prompts before using it; deployment is a separate action and
requires explicit authorization.

For all HA100 deployment and validation commands, use non-interactive SSH only:
`ssh -F /dev/null -T`, an explicit remote command, the dedicated
`~/.ssh/id_ed25519_ha100` identity, explicit `~/.ssh/known_hosts`, and
`StrictHostKeyChecking=yes`. Use batch public-key-only authentication
(`PasswordAuthentication=no`, `PreferredAuthentications=publickey`). The HA100
has a PTY allocation issue; never use `ssh -t`, `ssh -tt`, an interactive shell,
or PTY-dependent commands. The deployment script transfers files with SCP in
batch mode using the dedicated identity, explicit known-hosts file, strict
host-key checking, and public-key-only authentication. Non-interactive SSH
streaming is also available for explicit remote commands. Do not change system
SSH configuration to work around the PTY issue. The owner reported a
successful key-only remote-command test returning
`SSH key authentication succeeded`, `root`, and `couch`. This does not authorize
deployment; installation or any other HA100 change still requires explicit
approval.

Successful tests, compilation, or APK creation alone do not establish
functional hardware support. Report hardware testing separately and only when
it has actually been performed.

## Development workflow and Git safety

- Codex is the primary implementation agent for this repository.
- Inspect the current branch, status, and relevant files before editing. Keep
  changes small and focused.
- After each meaningful task, run the appropriate formatting, tests, and build
  checks for the changes made.
- Update `DEVELOPMENT_STATUS.md` with changes, test results, build/package
  results, and outstanding issues.
- Before committing, inspect `git status` and the complete diff.
- Never commit or push without explicit user approval.
- Never modify or merge into `main` without explicit user approval.
- Keep commits focused and use descriptive commit messages.
- After an approved push, report the branch, commit hash, and summary.
- Do not repeatedly ask the user to paste files or command output Codex can
  inspect directly.
- Do not deploy, delete files, or change the build/signing workflow without
  explicit user authorization.
- Preserve existing files and documentation. Do not claim work or verification
  that was not actually completed.
