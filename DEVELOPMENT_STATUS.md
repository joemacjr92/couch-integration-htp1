# Development Status

## Repository snapshot

- Branch: `main`
- HEAD: `bf622891a830358db55e0d3dbf3f5e1fd35f7db7` — “Merge HTP-1 functional baseline”.
- Functional baseline tag: `htp1-functional-0.1.0-r0` targets `7c1cf0baff71635ee1c57f435b0331d29d21eb04`; the Protocol 3 implementation is merged into `main`.
- Protocol 3 runtime support is confirmed by the reviewed SDK/source, and the
  HA100 runtime is reported to support Protocols 1 through 5. The owner reports
  successful P3 installation and functional validation on the HA100.
- Integration package version remains `0.1.0`; manifest `protocol_version` and
  `min_core_protocol_version` are both 3.
- Both Couch dependencies are pinned to
  `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`.
- Snapshot updated 2026-11-19. Trusted-repository publishing work below is local and uncommitted.

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
- The owner reports that basic controls work and all five Activity-page `x:`
  upmix commands were tested successfully on the HA100 with the HTP-1.
- Dirac, night mode, PEQ, and seat-shaker controls remain unimplemented. The
  `off` and `mono` upmix values are intentionally omitted pending resolution
  of the documented/original integration discrepancy.

## Verification

`cargo fmt --check`, `cargo check`, and `cargo build --release` passed. A
historical complete `cargo test` run passed all 17 tests, including a local
WebSocket test of all five fixed patches and post-command state confirmation, an
unknown-ID rejection test that observes no WebSocket command, and the Protocol 3
manifest/capability contract test. A later `cargo test --locked` attempt passed
the eight library tests, then stopped when the client-handshake test could not
create its local socket (`PermissionDenied`); the admission test was ignored and
later test binaries were not run in that attempt. This is an environment-limited
failure, not evidence of a test regression. `cargo clippy --all-targets`
completed with three warnings in the existing status initialization
(`src/sdk.rs`), state initialization (`src/state.rs`), and WebSocket text
conversion (`src/transport.rs`); `-D warnings` fails on those warnings. These
local simulated-device tests do not establish operation against physical HTP-1
hardware.

## Build and packaging

The packaging mismatch was corrected earlier by changing only `plugin.json`'s
`executable` field to `bin/couch-plugin-monoprice-htp1`. The owner reports that
the following P3 package was installed and functionally validated on the HA100:

```text
Filename: couch-integration-monoprice-htp1-0.1.0-r0.apk
Version: 0.1.0-r0
Protocol: 3
Source commit reported for the installed build: 096d8d9cf83c54b27858ac9260f9a2ce91a8af69
User-reported installed APK SHA-256: f688ef28bf6e1cc2931a5536c709edb29cef15b2e0d79a6f8f2ae21c04951089
```

The user-reported installed APK hash has **not** been matched to a local APK
artifact. A local file with the same filename and version is present at:

```text
/home/joe/couch-dev/couch-integration-htp1/build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk
Local APK SHA-256: eb2b41dfbdd467df6076145d93f38af2057a90102ac9fb6b0cbafdbf38285ac2
Size: 654,482 bytes
```

The local APK hash differs from the user-reported installed APK hash; the two
artifacts must not be treated as identical. The installed artifact's source
commit has not been independently verified against its APK bytes.

The Cargo target remains named `couch-plugin-htp1`; the APK contains the
ID-derived executable at `usr/lib/couch/integrations/monoprice-htp1/bin/couch-plugin-monoprice-htp1`.
The extracted executable is a statically linked ARM EABI5 32-bit ELF. The
package metadata identifies the local artifact as `armv7`.

Alpine `abuild` warned that the payload is not in `$source/$install/$triggers`,
that no maintainer is set, and that APKINDEX files were unavailable from cache
before proceeding. These warnings did not prevent package creation or signing.
The build script correctly warns that packaging alone does not establish
physical HTP-1 functionality or validate device command responses. Those
behaviors are reported separately above based on the owner's hardware test.

## Outstanding work and next steps

1. Keep the reviewed `0.1.0-r0` Protocol 3 baseline commit and annotated tag
   intact while preparing the separate `0.1.1` repository release.
2. Consider Dirac, night mode, PEQ, and seat-shaker features separately if they
   remain in scope; do not infer `off`/`mono` behavior.

## Uncertainties

- The user-reported installed APK SHA-256 has not been matched to the local APK
  with the same filename and version. Its source commit has not been
  independently established from package bytes.

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
report's source inconsistency about activity/scene picker behavior is resolved
for this installed build by the owner's successful Activity-page tests. The
owner reports successful
installation and hardware testing of this implementation on the HTP-1; those
results are recorded above.

## Deployment preparation

Prepared the reviewable deployment procedure and separate live-validation
checklist in
[`docs/DEPLOYMENT_AND_LIVE_VALIDATION.md`](docs/DEPLOYMENT_AND_LIVE_VALIDATION.md).
The P3 APK filename, reported installed hash, source commit, and distinct local
APK hash are recorded above. The supported deployment path is
non-interactive SSH plus CouchOS
`couch-confd integrations install-sideload`, which verifies signature, package
contents, and protocol handshake; package-manager rollback is documented. The
owner reports successful dedicated Ed25519 key authentication using the
non-interactive remote command (`SSH key authentication succeeded`, `root`,
`couch`). The HA100 has a PTY allocation issue, so deployment instructions
require `ssh -F
/dev/null -T`, an explicit command, that identity, strict host-key checking,
and non-interactive key-only transfers. The build script uses SCP in batch mode
with the dedicated identity and the same host-key safeguards. The owner reports
public-key enrollment and successful key-only access. The owner now reports
successful first installation and live validation of basic
controls and all five Activity-page upmix commands. The documented first-install
rollback remains package removal, preserving saved connections. Production log
retrieval has not been verified.

The dedicated client key is at `~/.ssh/id_ed25519_ha100`. Its public half was
enrolled through CouchOS Remote settings → SSH, as confirmed by the reported
successful key-only login. The private key remains local.

## Trusted repository publishing preparation

- Adapted the Fire TV Actions workflow for this HTP-1 package identity and the
  pinned Protocol 3 SDK. The first tagged repository release is constrained to
  `v0.1.1`; the workflow rejects republishing an existing version.
- Added the package metadata contract, package-host admission test, disposable
  signed-package smoke test, ARMv7 CI job, tagged APK/repository publication,
  and weekly signed-feed metadata renewal. The release workflow uses the
  existing `APK_PRIVATE_KEY` and `APK_PUBLIC_KEY` Actions secrets and checks
  their public identity against the committed repository public key.
- Added only the public PEM at `keys/monoprice-htp1-repository-public.pem`.
  Its normalized PEM fingerprint is
  `ad5ecdedddfbf48e8597053c2df0679c858af9ff4cdff879df50b9e33eeac79d`; the
  private key remains outside the repository.
- Updated the README with repository onboarding and linked publishing notes.
  GitHub Actions have not been run, no `v0.1.1` package has been built, and
  nothing has been published to Pages or a GitHub Release. No HA100 command or
  deployment was performed.
- Rust runtime behavior and the SDK pin are unchanged; `tools/build-deploy.sh`
  is unchanged.
- Hardened trusted-release handling: `git ls-remote` distinguishes a missing
  `gh-pages` branch from transport/authentication failures, package and
  APKINDEX signatures are cryptographically verified with the committed public
  key, and feed renewal rejects metadata versions that disagree with APK
  filenames before updating signed metadata.
- Static workflow/shell checks and temporary signed-feed tests passed,
  including successful renewal, version mismatch rejection without mutation,
  and rejection of invalid feed signatures, index/APK hashes, or signing-key
  pairs. The Alpine `apk verify` commands were not executed locally; no APK was
  built and no publishing or device operation was performed.
