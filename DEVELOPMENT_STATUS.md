# Development Status

## Repository snapshot

- Branch: `feature/htp1-state-sync`
- HEAD: `045b5b51808957d6f0fbdbf55027910e278ec274` — “Document Protocol 3 and HTP-1 x-command research”
- Main remains at its original scaffold commit: `00b60da88f89800f861062f68bd5cd007973877e` (`Initial HTP-1 integration scaffold`).
- The HA100 runtime supports Protocols 1 through 5; Protocol 3 compatibility is confirmed.
- Snapshot updated 2026-11-19 (16:12 UTC).

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
- Dirac, upmix, night mode, PEQ, and seat-shaker controls remain unimplemented.

## Verification

`cargo fmt --check` passed. `cargo test` passed all 13 tests, including the SDK
adapter and plugin manifest contract test. The tests cover the local handshake,
protocol fixtures, JSON patch behavior, settings, and SDK mapping. Passing tests
do not establish successful operation against physical HTP-1 hardware; no
hardware verification is recorded here.

## Build and packaging

The packaging mismatch was corrected by changing only `plugin.json`'s
`executable` field to `bin/couch-plugin-monoprice-htp1`. This now matches the
packager's ID-derived executable requirement. The existing workflow completed
and produced:

```text
/home/joe/couch-dev/couch-integration-htp1/build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk
SHA-256: 29002fc8ed21b02986e24bc5811a2ec5225c3a47d5354c8e14ad12ac2d1f2460
Size: 575K
```

The Cargo target remains named `couch-plugin-htp1`. The build script compiled
that binary and the packager staged it under the ID-derived filename inside the
APK. The optional HA100 deployment prompt was declined; no deployment was
performed.

The workflow completed with warnings: Alpine `abuild` reported that the
payload is not in `$source/$install/$triggers`, that no maintainer is set, and
that the local APKINDEX was unavailable from its cache before proceeding. The
build script also printed its existing post-package warning that this scaffold
has a placeholder executable and empty manifest; that warning is stale relative
to the current implementation and manifest. These warnings did not prevent APK
creation or signing. Packaging success does not establish device functionality.

## Outstanding work and next steps

1. Review the APK and assess the packaging warnings; package success alone does
   not prove device functionality.
2. Plan and implement advanced HTP-1 controls (Dirac, upmix, night mode, PEQ,
   and seat shakers) if they are in scope.
3. Arrange and record hardware verification before describing the integration
   as functionally complete.

## Uncertainties

- Physical-device behavior has not been verified. The packaging workflow
  completed after the manifest correction, but package creation alone does not
  establish device operation.
- The repository README still describes the original scaffold as incomplete;
  this status file records the later implementation context supplied for this
  branch. Update that older overview only after reviewing its intended role.

## Protocol 3 / custom command research

Completed an architecture-only review of the project's pinned Couch SDK and a
newer Couch source tree. The report is in
[`docs/PROTOCOL3_X_COMMANDS_RESEARCH.md`](docs/PROTOCOL3_X_COMMANDS_RESEARCH.md).
The integration is pinned to Couch revision
`00ab4da2e6336a925e93702507c0b7b233012738` (Protocol 2); the inspected later
source at `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81` contains Protocol 3 `x:`
commands. The HA100 runtime supports Protocols 1 through 5, confirming Protocol
3 compatibility. The integration remains pinned to a Protocol 2 SDK; a future
Protocol 3 migration still requires reviewing and updating that SDK pin. x
commands are fixed declared IDs without arbitrary payloads, so they could map to
verified discrete HTP-1 choices. Snapshot fixtures establish observation only,
not safe write semantics. The newer Couch source has a docs-versus-model-test
inconsistency about x commands in activity sequences/scenes; verify the desired
UI behavior for the target release. No implementation or hardware test was
performed.
