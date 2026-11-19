# CouchOS Protocol 3 and custom `x:` commands

**Research snapshot:** 2026-11-19
**Integration revision:** `f98b211b1150e83f43a43cf21ad1094835eea177` (`feature/htp1-state-sync`)
**Exact Couch SDK pin in this project:** `00ab4da2e6336a925e93702507c0b7b233012738`
**Newer Couch source inspected for Protocol 3 implementation:** `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81` (adjacent `../couch-sdk-reference`, branch `main`).

This is an architecture investigation, not an implementation plan approval. It changes no Rust, manifest, dependency, or build files. Source paths and line numbers below refer to the exact commits named in links. The SDK checkout is newer than the integration pin and has `tools/integrations/build-apk.sh` modified in the worktree; that unrelated local modification was left untouched. It is a sparse checkout and does not contain local copies of all core/UI source files, so relevant excluded files were checked through the connected GitHub source at commit `e7fc36c`.

## Findings at a glance

- This integration pins both `couch-plugin` and `couch-sdk` to `00ab4da2e6336a925e93702507c0b7b233012738` (`Cargo.toml:19-20`). That revision implements protocol 2 only. Its plugin protocol has no `Function::Custom` and no phase-bearing command request.
- The later inspected Couch tree has protocol-3 vocabulary, including `Function::Custom`, plus later protocol-4 and protocol-5 features. Its source currently accepts package protocol versions through 5. That does **not** prove the CouchOS runtime installed on an HA100 accepts version 3.
- Protocol 3 `x:<id>` is a package-declared, fixed button name. It has no arbitrary argument/value field. The package implements the ID as a branch in its own command handler; Couch routes the selected ID to that package without interpreting its meaning.
- Newer model code and validation tests support declared package buttons in activity custom pages and several other saved command locations. Some Protocol 3 documentation in the same source tree says custom buttons are refused in scenes and activity steps, which conflicts with those tests. See the UI section below.
- HTP-1 snapshot fields make discrete options plausible, but the repository does not establish the write semantics for Dirac, upmix, night, PEQ, or seat shaker controls. No physical receiver was tested. Treat proposed commands as design candidates only until HTP-1 command behavior is verified.

## Source versions and evidence

### Project pin

`Cargo.toml:19-20` pins both Couch crates to full commit `00ab4da2e6336a925e93702507c0b7b233012738`; the `Cargo.lock` source records this exact Git revision as well. The pinned [`clients/couch-plugin/src/protocol.rs`](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-plugin/src/protocol.rs#L1-L15) defines `PROTOCOL_VERSION = 2`. Its [`Request`](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-plugin/src/protocol.rs#L72-L111) has `Command { function: String }` without a `phase` or `resource`; the pinned model `Function` enum has no `Custom` variant. The pinned [`DeviceClient` trait](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-sdk/src/client.rs#L35-L141) has `execute(&Function)` and the `command()` parse/support/execute gate.

The pinned [`docs/development/protocol.md`](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/docs/development/protocol.md#L1-L14) describes protocol 1 as the installed preview at that source snapshot and protocol 2 as specified but not yet shipped. It documents protocol 2 typed dB support: `volume_db` status and a bounded `set_volume_db` action, with manifest protocol and minimum core version set to 2 ([lines 146-202](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/docs/development/protocol.md#L146-L202)). This historical release-state statement is not evidence of what a current HA100 runtime supports.

### Later Protocol 3 source

The adjacent source checkout is at `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`, not the integration pin. `clients/couch-plugin/src/protocol.rs:6-18` now says `PROTOCOL_VERSION = 5`, `NEXT_PROTOCOL_VERSION = 3`, and `accepted_protocol_version() == 5`. Its manifest validator admits versions 1 through 5 and treats x IDs as protocol 3 features (`clients/couch-plugin/src/manifest.rs:210-240, 394-412`). Thus this is useful evidence for the shape/semantics added at protocol 3, but it is neither the pinned SDK nor proof that this device's installed runtime admits P3 packages.

The docs in that newer tree are not internally uniform about rollout status: [`docs/development/protocol.md:185-203`](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/development/protocol.md#L185-L203) calls P3 the current package contract and says P1/P2 packages retain their wire shape, while other descriptions still call P3 unreleased. The code/docs also disagree about whether custom commands are accepted in scenes/activity sequences. The target runtime and its admission policy must be checked directly before migration.

### What the HTP-1 repository itself proves

The checked-in manifest declares `protocol_version: 2` and `min_core_protocol_version: 2` (`plugin.json:2,69`). The adapter's `SdkClient::capabilities()` and `execute()` in [`src/sdk.rs:67-77,91-147`](../src/sdk.rs) currently handle fixed common capabilities; the wildcard returns `Unsupported`. Dynamic input selection is implemented separately. The typed state parser in [`src/state.rs:17-35`](../src/state.rs) reads `upmix.select`, `night`, and `cal.currentdiracslot`, retaining raw JSON. The recorded snapshot fixture includes upmix mode names, night state, PEQ slot structures, Dirac slot information, and `speakers.seatshaker.present` ([`tests/fixtures/responses.txt`](../tests/fixtures/responses.txt)). Those are observations from a fixture, not proof that any particular JSON Patch write is valid or safe.

## Protocol 2 and Protocol 3

### Protocol 2

Protocol 2 extends the earlier fixed command/status/inputs package contract with typed dB volume reporting and a bounded `set_volume_db` action. This preserves percentage volume compatibility while giving Couch a more precise volume value/action. The project's protocol-2 adapter reports `VolumeDb` in status (`src/sdk.rs:181-200`), but its manifest currently has `actions: []` (`plugin.json:70`), so it does not declare the typed volume action. Its existing power, mute, volume-step, and input functions remain fixed command IDs/capabilities.

### Protocol 3 additions relevant here

The later source documents these additions at [`docs/development/protocol.md:185-244`](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/development/protocol.md#L185-L244):

- `x:<id>` package-named button capability IDs.
- Key command phases: tap, repeat, and long press; tap is omitted from serialized requests. The model explicitly says x buttons never repeat while held (`model/couch-model/src/commands.rs:58-79`).
- Multiple typed action schemas and non-volume typed actions.
- Bridge child enumeration / resource-scoped requests.
- Pairing, host-managed credentials and related error reporting.

Only `x:` commands are needed to expose discrete HTP-1 functions. This is a single-device integration, so children and pairing are not needed unless a separate requirement emerges. Key phase support is optional for this use case; the default `DeviceClient::execute_phased` implementation can ignore phase, and x buttons are specified as non-repeating.

### Backward compatibility

In the newer Protocol 3-capable source, a host is designed to preserve the old P1/P2 wire shapes: new request fields are omitted when absent, tap is not serialized, and the server downgrades phase to tap for older package protocols (`clients/couch-plugin/src/protocol.rs:151-163`, `clients/couch-plugin/src/server.rs:270-280`). It gates `x:` requests on the package protocol and declaration before I/O (`clients/couch-plugin/src/host.rs:1030-1060`; `manifest.rs:394-403`). This supports old **packages on a new host**.

The reverse is not compatible: a P3 manifest requires a host that recognizes protocol 3. A P1/P2-only runtime will reject or fail to admit it. The actual HA100 release/runtime has not been checked in this research, so runtime compatibility is an open deployment gate.

### Migration surface

A migration would need all of the following, after confirming runtime support:

1. Move both Couch crate dependencies to one reviewed exact revision that contains P3 `Function::Custom` support; update and review `Cargo.lock` consistently. The current pin does not contain the enum variant or P3 command framing.
2. Set both manifest version fields to 3: `protocol_version: 3` and `min_core_protocol_version: 3`. Do not set version 5 simply because the newer SDK checkout's current maximum is 5; package protocol describes the minimum feature contract being used.
3. Declare each custom ID/label in `plugin.json.capabilities` and return the exact same ordered `(id, label)` list from `DeviceClient::capabilities()`. The newer `serve()` checks that manifest ID and capability list match the client (`clients/couch-plugin/src/server.rs:64-82`).
4. Add `Function::Custom(id)` dispatch in `SdkClient::execute()` and map each known ID to one explicitly implemented HTP-1 action. The project's existing wildcard currently rejects it (`src/sdk.rs:146`). Keep command authorization exact: the SDK's `command()` parses a command and rejects anything absent from `supports()` before execution; in the newer SDK, `supports()` checks the declared capability ID (`clients/couch-sdk/src/client.rs:163-188`, `clients/couch-plugin/src/manifest.rs:394-412`).
5. Use a P3-compatible Couch runtime/core and confirm package admission on the target. `tools/build-deploy.sh:147-150` runs `couch-confd --supports-integration-protocol=$PROTOCOL_VERSION` only in its optional deployment path; the source-tree version constant alone is insufficient.

No change to the required `DeviceClient` method set is needed for tap-only x commands: `execute(&Function)` handles them. Implement `execute_phased` only if a supported command specifically needs hold/repeat semantics; x functions are documented not to repeat. Manifest schema gains no arbitrary x-command payload definition: IDs are simply ordinary declared capabilities once protocol 3 admits them.

## What an `x:` command is and its full path

### Syntax and payload

The complete command ID is exactly `x:<id>`. The suffix is 1–48 bytes, lowercase ASCII letters, digits, hyphens, or underscores; each package may declare no more than 32 such IDs. The source defines this in `model/couch-model/src/commands.rs:58-65, 86-90, 396-405` and the manifest validator counts/enforces it in `clients/couch-plugin/src/manifest.rs:210-240`.

`x:` does **not** carry an arbitrary JSON object, numeric value, HTP path, or free-form string. It names one fixed command. A tap serializes as a command request containing `function: "x:..."`; in the P3 serializer, phase `tap` and absent resource are omitted (`clients/couch-plugin/src/protocol.rs:151-163`). On the framed socket this is the standard `{id, body}` request envelope. A long-press request can include a phase for commands that support phases, but x commands do not repeat while held and should generally be tap-only.

### Declaration, dispatch, and response

The command must be declared as a manifest capability with a human-facing label and must also appear in `DeviceClient::capabilities()`. The SDK's command boundary parses the persisted ID, checks client support, and only then calls `execute()`. In the adapter, match the parsed `Function::Custom(id)` against a closed set of IDs and issue only the associated validated HTP-1 patch. Unknown or undeclared IDs return `Unsupported`; do not turn an x ID into an arbitrary device path.

The request path for the remote/panel is:

1. A user chooses a function from the device's command list (or a saved Activity page button). Packaged-device choices are derived from the manifest capability IDs and labels in the model's `function_choices()` (`model/couch-model/src/buttons.rs:332-349`).
2. The chosen `Action { device, command }` is stored as an ordinary Couch action. Activity page buttons use the same action dispatcher as device commands; UI code queues the widget's saved action, and the worker executes it (`ui/couch-gui/src/activity_pages.rs` and `activity_buttons.rs` at ref `e7fc36c`).
3. The daemon resolves the connection and sends its ordinary plugin `Request::Command { function }` to that connection's persistent plugin process (`daemon/couch-confd/src/api/plugins.rs:168-180` at ref `e7fc36c`).
4. The P3 host gate requires protocol 3 for `Function::Custom` and verifies that the capability was declared before forwarding; the plugin server calls `DeviceClient::command_phased`, which gates support and dispatches to `execute()` (`clients/couch-plugin/src/host.rs:1030-1060`; `clients/couch-plugin/src/server.rs:270-280`; SDK `DeviceClient::command` and `execute_phased` in `clients/couch-sdk/src/client.rs:128-135,177-203`).
5. The adapter executes the HTP-1 WebSocket operation, then returns success or an SDK error. The server returns `Response::Ok` for success or `Response::Error { code, reason? }` for failure (`clients/couch-plugin/src/protocol.rs:368-422`). The host correlates the response with request ID. Errors include invalid, unsupported, protocol, transport, timeout, rejected, and (P3) unpaired; detailed reason text is optional. There is no custom x-specific response schema.

### Activities and Custom Pages

**Verified in the newer model/UI source:** package capability choices include each declared capability, and the custom Activity Pages data model stores selected device/function actions. Activity page validation routes through normal `Function::parse` and `supports_device` checks (`model/couch-model/src/activity_setup.rs:72-100`; `commands.rs:254-270`). A newer model regression test explicitly verifies declared `x:info` is valid in custom-page widgets, activity setup sequences, activity bindings, and scenes, while undeclared/invalid IDs are rejected (`model/couch-model/src/validate.rs:1012-1065`). The Activities guide describes a function picker and limits custom pages to six buttons each and eight pages, with no arbitrary-value sliders or live-state widgets (`docs/activities.md:12-22, 36-44`).

**Source inconsistency / limitation:** `docs/development/protocol.md:190-197` in this same SDK revision says custom IDs are only offered in the package button picker and “refused everywhere else,” including scenes and activity steps. This conflicts with the newer model validation test that accepts x IDs in those saved locations. The current UI picker content and target release should be verified in the exact CouchOS product build before relying on sequence or scene support. The evidence for custom Activity Pages is stronger in the model and regression test; a generic picker ultimately surfaces the declared package capability, though the guide's phrase “standard typed command catalog” is imprecise here. Treat x in Activities sequences/scenes as implementation-version-dependent until resolved.

## Applying this to the HTP-1

The existing standard Couch capabilities remain the right interface for common receiver controls: power on/off, volume up/down, mute toggle/on/off, plus dynamic `input:<id>` selection. They are already implemented and should not be duplicated as HTP-specific x IDs.

| HTP-1 feature | Couch interface fit | What is verified now | Suggested treatment |
| --- | --- | --- | --- |
| Upmix mode | Discrete HTP-specific option; no standard Couch function represents the receiver's mode list. | Fixture contains `upmix.select` and names including `native`, `dolby`, `auro`, `mono`, `dts`, `stereo`; state parser reads selection. Writable patch path/value semantics not established. | After device/authoritative protocol verification, expose one fixed x ID per safe supported mode, or a bounded typed selector only if Couch adds an appropriate shared schema/UI. |
| Night mode | Discrete HTP-specific toggle/mode. | Fixture shows `night: "off"`; no write path verified. | If verified as a simple on/off, expose `x:night-on` and `x:night-off`; avoid a toggle if explicit setters are possible. |
| Dirac slot/profile | Finite selection; could fit one x button per verified slot. | Fixture contains `/cal/currentdiracslot`, six slot count, and slot metadata; parser reads current slot. The set operation and slot-index validity/write behavior are not verified. | After testing, declare only valid slots (up to six based on this fixture, not a guarantee for all firmware) as explicit x commands. Consider state feedback separately. |
| PEQ presets/enable | Finite slot selection and on/off are plausible fixed commands. | Fixture has `peq.currentpeqslot`, `peq.peqsw`, and slot data; no write semantics or suitable state mapping are verified. | Start with explicit preset/enable commands only after verification. Do not promise editable filters via x. |
| Seat shakers | Presence-gated discrete controls may be useful. | Fixture reports `speakers.seatshaker.present: false`; control fields and command semantics are not established. | Investigate model/firmware first; advertise nothing unless present and controllable. Fixed preset/on/off IDs may fit if confirmed. |
| Input selection | Existing standard dynamic capability. | Dynamic visible input discovery/selection already implemented. | Keep using `supports_inputs` / discovered input IDs; no x commands needed. |

A custom command has no argument payload, so it cannot directly express arbitrary PEQ frequency, Q, gain, channel, shaker level, delay, or other numeric values. Activity custom pages are labeled buttons, not sliders or live-state displays. For arbitrary parameter editing, Couch would need a bounded typed action schema and suitable native UI/control, or another supported structured interface. Do not encode values into ad hoc x IDs without a deliberately bounded and validated command catalog; it still would not provide a general value-entry UI.

The package's current status model has fixed Couch status fields and the manifest presentation selects existing native Couch components; x commands do not create new status fields. Showing current upmix/Dirac/PEQ state in native UI would require an existing compatible field/component or a Couch model/UI extension. Custom page controls likewise are not status tiles per the Activities guide.

## Recommendation

Adopt Protocol 3 only after confirming that the intended CouchOS runtime supports protocol version 3 and selecting a reviewed SDK revision containing its custom function implementation. Keep the one-device WebSocket adapter, persistent Tokio runtime, snapshot refresh-before-command policy, `changemso` patches, and post-command snapshot verification. Use x only as a capability-name dispatch layer for a small, explicit set of verified discrete HTP-1 actions. Retain standard Couch capabilities for common functions and dynamic input selection. Avoid P3 child enumeration, pairing, arbitrary custom payloads, and UI/core changes unless separate requirements justify them.

### Blockers and open questions

1. Does the target HA100 runtime accept protocol 3? The build script has an explicit target support query, but no target was queried for this research.
2. Which exact HTP-1 MSO JSON Patch paths and values safely set each desired control across supported firmware? Current fixture data is read evidence only.
3. Do Activity custom-page pickers and sequence/scene validation in the intended released core include x IDs? Current source has conflicting documentation versus model tests.
4. Which Dirac and PEQ slots are valid/available per receiver, including varying firmware and calibration state? A fixed manifest capability list may not represent device-dependent availability without extra gating.
5. How should the UI communicate current selected mode/slot? x commands report command completion, not resulting HTP-1 status; the current status schema has no general custom map.
6. Source history calls some P3 features preview/unreleased while the current source constant is P5. Deployment compatibility must be checked against the actual CouchOS build, not inferred from the SDK checkout.

## Staged implementation plan

1. **Compatibility gate:** identify the intended HA100 CouchOS release; run its `couch-confd --supports-integration-protocol=3` check or equivalent read-only release verification. Resolve which SDK commit is approved for external P3 use.
2. **Device command verification:** consult the original Python integration and HTP-1 API documentation, then test candidate patches against a receiver. Record exact paths, values, firmware assumptions, and confirmation behavior before exposing commands.
3. **Small P3 migration:** update the two Couch dependency pins and lockfile together; set both manifest versions to 3; add a few fixed x capabilities with exact manifest/DeviceClient parity; dispatch by a closed match; add unit/contract tests for IDs, patch generation, errors, and refresh/verify behavior. Preserve Tokio lifetime and existing command policy.
4. **UI validation:** on the target core, verify device command list, Activity custom page selection/execution, and sequence/scene behavior separately. Confirm error feedback and that unknown/undeclared IDs fail before device I/O.
5. **Incremental controls:** add verified upmix/night/Dirac/PEQ/seat-shaker commands in small groups. Add structured state presentation only if existing schema supports it or after separate CouchOS UI/model work is authorized.
6. **Release verification:** run formatting, complete Rust tests and build/package checks; install only with explicit deployment authorization; test with physical hardware and record results separately from compilation/package success.

## Source index

Local paths are relative to this integration repository unless prefixed `../couch-sdk-reference/`. GitHub links pin exact refs for external source.

| Source | Relevant locations |
| --- | --- |
| Integration `Cargo.toml` | `:19-20`, exact paired Couch Git pin `00ab4da2e6336a925e93702507c0b7b233012738` |
| Integration `plugin.json` | `:2, 69-70`, P2 manifest and no typed actions |
| Integration `src/sdk.rs` | `:67-77, 91-147, 181-200`, declared capabilities, dispatch, wildcard rejection, dB status |
| Integration `src/state.rs` | `:17-35`, fields parsed while retaining raw state |
| Integration `tests/fixtures/responses.txt` | captured MSO fixture (single-line JSON); observed values are not write documentation |
| Pinned Couch `clients/couch-plugin/src/protocol.rs` | [ref 00ab…](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-plugin/src/protocol.rs#L1-L15), [request/response shape](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-plugin/src/protocol.rs#L72-L150) |
| Pinned Couch `clients/couch-sdk/src/client.rs` | [DeviceClient trait and command boundary](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/clients/couch-sdk/src/client.rs#L35-L141) |
| Later Couch `clients/couch-sdk/src/client.rs` | `execute_phased():128-135`; command capability gate `:163-203` |
| Pinned Couch `docs/development/protocol.md` | [protocol status](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/docs/development/protocol.md#L1-L14), [typed dB actions](https://github.com/Couch-OS/couch/blob/00ab4da2e6336a925e93702507c0b7b233012738/docs/development/protocol.md#L146-L202) |
| Later Couch `clients/couch-plugin/src/protocol.rs` | [version constants](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/protocol.rs#L6-L18), [command request](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/protocol.rs#L136-L178), [response/error](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/protocol.rs#L368-L422) |
| Later Couch `model/couch-model/src/commands.rs` | [Custom function and phases](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/model/couch-model/src/commands.rs#L58-L90), [custom grammar](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/model/couch-model/src/commands.rs#L396-L405), [capability check](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/model/couch-model/src/commands.rs#L254-L270) |
| Later Couch `clients/couch-plugin/src/manifest.rs` | [limits/version admission](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/manifest.rs#L210-L240), [x support gate](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/manifest.rs#L394-L412) |
| Later Couch `clients/couch-plugin/src/server.rs` | [manifest/client parity](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/server.rs#L64-L82), [dispatch/response](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/server.rs#L270-L280) |
| Later Couch `clients/couch-plugin/src/host.rs` | [protocol requirement/gate](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/clients/couch-plugin/src/host.rs#L1027-L1066) |
| Later Couch model `buttons.rs`, `activity_setup.rs`, `validate.rs` | `function_choices():332-349`; activity sequence validation `:72-100`; x command placement regression test `validate.rs:1012-1065` |
| Later Couch `docs/development/protocol.md` | [P3 vocabulary and contradictory activity text](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/development/protocol.md#L185-L203), [wire command](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/development/protocol.md#L266-L298), [old-package compatibility](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/development/protocol.md#L403-L424) |
| Later Couch `docs/activities.md` | [custom pages and widget limitations](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/activities.md#L12-L22), [limits/validation](https://github.com/Couch-OS/couch/blob/e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81/docs/activities.md#L36-L44) |
| Integration `tools/build-deploy.sh` | `:147-150`, target-side protocol check performed only in optional deployment path |
