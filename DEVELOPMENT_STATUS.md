# Development Status

## Repository snapshot

- Branch: `feature/htp1-state-sync`
- Latest reviewed baseline: P3 migration commit `096d8d9cf83c54b27858ac9260f9a2ce91a8af69` — “Add Protocol 3 custom upmix commands”.
- Main remains at its original scaffold commit: `00b60da88f89800f861062f68bd5cd007973877e` (`Initial HTP-1 integration scaffold`).
- Protocol 3 runtime support is confirmed by the reviewed SDK/source, and the
  HA100 runtime is reported to support Protocols 1 through 5. P3 package
  admission on the HA100 and physical HTP-1 behavior remain unverified.
- Integration package version remains `0.1.0`; manifest `protocol_version` and
  `min_core_protocol_version` are both 3.
- Both Couch dependencies are pinned to
  `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`.
- Snapshot updated 2026-11-19.

## Implemented

- Rust HTP-1 WebSocket client connects at `/ws/controller`, requests an initial
  snapshot with `getmso`, replaces cached state on full `mso` messages, and
  applies JSON Patch operations from `msoupdate` messages.
- Commands use `changemso` patch arrays. SDK commands refresh state before
  acting, then refresh and verify the requested value.
- The synchronous CouchOS SDK adapter retains a Tokio current-thread runtime
  alongside the asynchronous WebSocket client.
- Power on/off, volume up/down by 1 dB, mute toggle/on/off, dynamic visible
  input selection, and power/mute/input/volume status reporting are present.
- Protocol 3 custom commands `x:upmix-auro`, `x:upmix-dolby`,
  `x:upmix-dts`, `x:upmix-native`, and `x:upmix-stereo`. Each maps to a fixed
  replace operation on `/upmix/select`; command handling refreshes before the
  patch, then requests and verifies a fresh snapshot.
- Dirac, night mode, PEQ, and seat-shaker controls remain unimplemented. The
  `off` and `mono` upmix values are intentionally omitted pending resolution
  of the documented/original integration discrepancy.

## Verification

`cargo fmt --check`, `cargo check`, and `cargo build --release` passed. `cargo
test` passed all 17 tests, including a local WebSocket test of all five fixed
patches and post-command state confirmation, an unknown-ID rejection test that
observes no WebSocket command, and the Protocol 3 manifest/capability contract
test. `cargo clippy --all-targets` completed with three warnings in the existing
status initialization (`src/sdk.rs`), state initialization (`src/state.rs`),
and WebSocket text conversion (`src/transport.rs`); `-D warnings` fails on those
warnings. These local simulated-device tests do not establish operation against
physical HTP-1 hardware.

## Build and packaging

The packaging mismatch was corrected earlier by changing only `plugin.json`'s
`executable` field to `bin/couch-plugin-monoprice-htp1`. The last recorded APK
was built before this Protocol 3 migration, when the manifest declared
Protocol 2. It is stale and must not be used to validate or deploy this P3
implementation. No P3 APK has been built; its SHA-256 is pending. The build
script checksum gate and deployment documentation safeguards must be reviewed
before creating a fresh P3 APK.

The previous APK record was:

```text
/home/joe/couch-dev/couch-integration-htp1/build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk
SHA-256: 29002fc8ed21b02986e24bc5811a2ec5225c3a47d5354c8e14ad12ac2d1f2460
Size: 575K
```

The Cargo target remains named `couch-plugin-htp1`. The build script compiles
that binary and the packager stages it under the ID-derived filename inside the
APK. No Protocol 3 package has been deployed.

The workflow completed with warnings: Alpine `abuild` reported that the
payload is not in `$source/$install/$triggers`, that no maintainer is set, and
that the local APKINDEX was unavailable from its cache before proceeding. The
earlier build script output also included an obsolete post-package warning about
a placeholder executable and empty manifest. The deployment-preparation change
replaces that warning with an accurate statement that packaging does not
establish physical HTP-1 functionality or validate device command responses.
These warnings did not prevent the prior APK creation or signing. Packaging
success does not establish device functionality.

## Outstanding work and next steps

1. Complete and review the deployment-script checksum safeguard and deployment
   documentation updates before building a fresh signed P3 APK.
2. Build the P3 APK, record its SHA-256, and review package warnings and
   manifest/protocol contents.
3. After separate deployment approval, validate P3 package admission on the
   HA100 without assuming packaging proves runtime behavior.
4. After separate physical-validation approval, verify each upmix patch/value
   and observed result on the HTP-1.
5. Consider Dirac, night mode, PEQ, and seat-shaker features separately if they
   remain in scope; do not infer `off`/`mono` behavior.

## Uncertainties

- No Protocol 3 APK has been packaged or admitted on the HA100, and the HTP-1
  upmix patch semantics have not been verified on physical hardware. The
  pre-migration Protocol 2 packaging result does not validate this P3 build.
- The repository README still describes the original scaffold as incomplete;
  this status file records the later implementation context supplied for this
  branch. Update that older overview only after reviewing its intended role.

## Protocol 3 / custom command research

Completed an architecture-only review of the project's pinned Couch SDK and a
newer Couch source tree. The report is in
[`docs/PROTOCOL3_X_COMMANDS_RESEARCH.md`](docs/PROTOCOL3_X_COMMANDS_RESEARCH.md).
That report documents the investigation before migration. This branch now pins
both `couch-plugin` and `couch-sdk` to reviewed revision
`e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81` and declares Protocol 3. The selected
source provides `Function::Custom` and accepts P3 manifests; although it
contains APIs for later protocols, this package does not use P4/P5 features.
The five requested upmix IDs are implemented as a closed capability set. The
report's source inconsistency about activity/scene picker behavior remains
relevant to validating the target CouchOS UI. The target HA100 has previously
been reported to support Protocols 1–5; this implementation has not been
deployed or tested against an HTP-1.

## Deployment preparation

Prepared the reviewable deployment procedure and separate live-validation
checklist in
[`docs/DEPLOYMENT_AND_LIVE_VALIDATION.md`](docs/DEPLOYMENT_AND_LIVE_VALIDATION.md).
The pre-migration APK exists and its SHA-256 matches the recorded
`29002fc8ed21b02986e24bc5811a2ec5225c3a47d5354c8e14ad12ac2d1f2460`. The
supported deployment path is non-interactive SSH plus CouchOS
`couch-confd integrations install-sideload`, which verifies signature, package
contents, and protocol handshake; package-manager rollback is documented. The
owner reports that
dedicated Ed25519 key authentication was verified using a non-interactive
remote command (`SSH key authentication succeeded`, `root`, `couch`). The HA100
has a PTY allocation issue, so deployment instructions require `ssh -F
/dev/null -T`, an explicit command, that identity, strict host-key checking,
and non-interactive SSH streaming for file transfer; they avoid SCP and PTYs.
The owner reports public-key enrollment and successful key-only access; no HTP-1
package installation or HTP-1 control was performed. Per the owner, this is a
first installation and no HTP-1 integration package is installed; the SSH
preflight must verify its absence before transfer. The existing verified APK
version is used without a version bump. First-install
recovery is package removal (which preserves saved connections); no prior
package version exists for rollback. Production log retrieval remains
unverified. Live HTP-1 WebSocket and control behavior still requires hardware
validation.

The dedicated client key is at `~/.ssh/id_ed25519_ha100`. Its public half was
enrolled through CouchOS Remote settings → SSH, as confirmed by the reported
successful key-only login. The private key remains local.
