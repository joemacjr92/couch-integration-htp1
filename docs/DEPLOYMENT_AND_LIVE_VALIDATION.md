# HTP-1 deployment preparation and live validation

**Preparation snapshot:** 2026-11-19
**Repository:** `feature/htp1-state-sync` at deployment safeguards commit `e39c149701f261c9e9f716fc0c4bc6b96c0e945e` (`Add Protocol 3 deployment checksum safeguards`)
**HA100 address:** `10.0.0.25`
**P3 APK:** `build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk`

**SHA-256:** `07ccc743192ce34fecabc640a239a69d09e0a0bd7ce8a178ec338c0927ab84d2` (654,484 bytes)

## Current local verification

- Branch and HEAD were inspected; the worktree was clean before building. The fresh package is from the current committed P3 source.
- The APK was built and signed by `tools/build-deploy.sh`. Its manifest declares Protocol 3, ID `monoprice-htp1`, version `0.1.0`, and executable `bin/couch-plugin-monoprice-htp1`; package metadata says `armv7`, and the extracted executable is a statically linked ARM EABI5 32-bit ELF. Local `apk verify` with the matching public key returned `OK`.
- The prior Protocol 2 artifact was preserved as `build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk.pre-p3-29002fc8`; it is not the deployment candidate.
- The owner subsequently reported a successful non-interactive key-only SSH command returning `SSH key authentication succeeded`, `root`, and `couch`. No deployment preflight inventory was run for this document. Per the current project owner, this is a first install; the preflight below must verify that no `monoprice-htp1` row exists before transfer. The exact runtime build, saved connection, and target signing trust-key state remain to be confirmed.
- The integration manifest requests Protocol 3. The target has been reported to support Protocols 1 through 5; Protocol 3 is the required package compatibility gate. Recheck Protocol 3 support on the target runtime immediately before installation.

All commands below are proposals for a separately approved deployment window. No SSH command, transfer, install, restart, stop, remove, or configuration edit was performed by Codex while preparing this document. The owner separately reports that key-only SSH authentication works.

Define this helper in the local shell before using the command examples. Each invocation runs one explicit remote command without allocating a PTY; `BatchMode`, `PasswordAuthentication=no`, and `PreferredAuthentications=publickey` ensure these operations use the verified key only. The explicit known-hosts file and strict checking remain enabled.

```sh
HA100=10.0.0.25
ha100_ssh() {
  ssh -F /dev/null -T \
    -i "$HOME/.ssh/id_ed25519_ha100" \
    -o UserKnownHostsFile="$HOME/.ssh/known_hosts" \
    -o StrictHostKeyChecking=yes \
    -o IdentitiesOnly=yes \
    -o BatchMode=yes \
    -o PasswordAuthentication=no \
    -o PreferredAuthentications=publickey \
    "root@$HA100" "$@"
}
```

## Supported deployment mechanism

CouchOS's development packaging guidance documents signed APK sideload over SSH. The HA100 has a PTY allocation issue: interactive sessions fail with `PTY allocation request failed on channel 0`, while non-interactive key-authenticated remote commands work. All HA100 SSH operations in this procedure therefore use `ssh -F /dev/null -T`, the dedicated `~/.ssh/id_ed25519_ha100` identity, explicit `~/.ssh/known_hosts`, strict host-key checking, and public-key-only batch authentication. Do not use an interactive shell, `ssh -t`/`-tt`, or SCP. Transfer files by streaming them over non-interactive SSH to an explicit remote `cat` command.

In the normal HA100 Alpine environment, invoke the signed package installation as this explicit remote command:

```sh
ha100_ssh "/opt/couch/runtime/current/couch-confd integrations \
  --keys-dir /opt/couch/integration-keys/custom/developer \
  install-sideload /tmp/$APK_NAME"
```

`APK_NAME` is set from the freshly built P3 artifact in the transfer step below.

The same procedure is present in this repository's `tools/build-deploy.sh` (optional path, default HA100 `10.0.0.25`) and CouchOS `docs/development/packaging.md:144-215` / `docs/integration-packages.md:253-274`. Package admission verifies the APK signature, audits its payload, checks its protocol handshake, and only then activates it. **Do not use `apk add`**; CouchOS documentation says that bypasses the package audit, handshake, immutable slots, activation record, saved-settings compatibility check, and rollback mechanism.

The documented SSH example assumes root login into the normal Alpine environment. If the SSH shell is the outer initramfs/recovery shell instead, the CouchOS guide requires the separate `chroot /mnt/alpine` procedure. This plan is for normal SSH only and must stop if `/etc/alpine-release` is absent; do not improvise recovery-shell paths.

## Proposed procedure (not executed)

### 1. Read-only connectivity and runtime preflight

First verify the SSH endpoint and expected shell. Keep normal SSH host-key checking enabled; verify the HA100 host-key fingerprint through the owner's trusted record if it is not already known. The owner has reported a successful key-only remote-command test (`SSH key authentication succeeded`, `root`, `couch`); rerun the test during deployment preflight.

```sh
ha100_ssh 'test -f /etc/alpine-release'
```

Then record the Couch config daemon version, verify protocol support (the project records that 1 through 5 are supported), and inspect installed package IDs/versions. These are read-only commands:

```bash
ha100_ssh '
  set -eu
  CONFD=/opt/couch/runtime/current/couch-confd
  "$CONFD" --version
  for protocol in 1 2 3 4 5; do
    if "$CONFD" --supports-integration-protocol="$protocol"; then
      printf "Protocol %s: supported\n" "$protocol"
    elif [ "$protocol" = 3 ]; then
      printf "ERROR: Protocol 3 is required but unsupported.\n" >&2
      exit 1
    else
      printf "Protocol %s: unsupported\n" "$protocol"
    fi
  done
  installed="$("$CONFD" integrations list)"
  printf "%s\n" "$installed"
  if printf "%s\n" "$installed" | grep -Eq "(^|[[:space:]])monoprice-htp1([[:space:]]|$)"; then
    printf "ERROR: monoprice-htp1 is already installed; stop before transfer.\n" >&2
    exit 1
  fi
  printf "Confirmed: monoprice-htp1 is not installed.\n"
'
```

`couch-confd --version` is implemented by the checked Couch source (`daemon/couch-confd/src/main.rs`, `parse_args`); it identifies that binary's build version. The source and deployment guide document `--supports-integration-protocol=N` as the runtime compatibility probe. Save the version/probe/list output without SSH verbose mode or shell tracing. Protocol 3 is the required package gate; a failed Protocol 3 probe stops the procedure. Other protocol probes are informational.

The CLI list is the pre-install inventory. The preflight command prints the list and exits before transfer if it finds `monoprice-htp1`. If an entry is present, stop: the live state no longer matches the first-install premise. Protocol 3 support is the required protocol gate. Do not update, overwrite, or bump the APK version to work around an installation-state discrepancy.

### 2. First-install and configuration gates

Before transferring anything:

- Confirm `couch-confd integrations list` contains no `monoprice-htp1` row. If it does, stop and review the discrepancy; do not run the install command.
- No existing HTP-1 integration package needs to be preserved. This is a first install of the freshly built P3 APK; no version bump is needed for upgrade handling.
- A saved CouchOS connection/configuration record may still exist even when the package is absent. Review it in the paired UI and do not copy, print, edit, or remove CouchOS configuration or credential files. The CouchOS package manager performs saved-settings checks; its documentation says package removal retains saved connections.
- Do not proceed if the signature trust key, protocol probe, or intended HTP-1 connection/settings cannot be established.

### 3. Verify local artifact and transfer (future, after approval)

The build script defaults to the public key at `$HOME/.abuild/-6afc809d.rsa.pub` and copies that public key to the dedicated developer trust directory. Never transfer or display the private signing key.

```bash
APK=build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk
APK_NAME="$(basename "$APK")"
PUBLIC_KEY="$HOME/.abuild/-6afc809d.rsa.pub"
sha256sum "$APK"
test -r "$PUBLIC_KEY"
```

The fresh P3 APK hash is recorded at the top of this document. Recalculate the local hash immediately before transfer and compare it with the remote hash after streaming. Proceed only if the public key is readable and both hashes match. After all read-only preflight gates pass, the documented development procedure creates the custom key directory and transfers the public key and package:

```bash
ha100_ssh 'mkdir -p /opt/couch/integration-keys/custom/developer'
ha100_ssh 'cat > /opt/couch/integration-keys/custom/developer/developer.rsa.pub' \
  < "$PUBLIC_KEY"
ha100_ssh "cat > /tmp/$APK_NAME" \
  < "$APK"
REMOTE_CHECKSUM="$(ha100_ssh "sha256sum /tmp/$APK_NAME")" || {
  printf 'ERROR: remote checksum command failed; do not install.\n' >&2
  exit 1
}
LOCAL_CHECKSUM="$(sha256sum "$APK")" || {
  printf 'ERROR: local checksum command failed; do not install.\n' >&2
  exit 1
}
LOCAL_SHA256="${LOCAL_CHECKSUM%%[[:space:]]*}"
if [[ ! "$LOCAL_SHA256" =~ ^[[:xdigit:]]{64}$ ]]; then
  printf 'ERROR: APK SHA-256 comparison failed; do not install.\n' >&2
  exit 1
fi
if [[ "$REMOTE_CHECKSUM" =~ ^([[:xdigit:]]{64})[[:space:]]+(/tmp/[^[:space:]]+)$ ]]; then
  REMOTE_SHA256="${BASH_REMATCH[1]}"
  REMOTE_PATH="${BASH_REMATCH[2]}"
else
  printf 'ERROR: remote checksum output is empty or invalid; do not install.\n' >&2
  exit 1
fi
if [[ "$REMOTE_PATH" != "/tmp/$APK_NAME" || "$REMOTE_SHA256" != "$LOCAL_SHA256" ]]; then
  printf 'ERROR: APK SHA-256 comparison failed; do not install.\n' >&2
  exit 1
fi
printf 'Local APK SHA-256:  %s\nRemote APK SHA-256: %s\n' "$LOCAL_SHA256" "$REMOTE_SHA256"
```

The comparison fails closed if the remote command fails, its output does not contain a valid SHA-256 and the expected path, or the hashes differ. Do not run `install-sideload` unless both hashes match. These commands intentionally do not use `apk`, restart a service, or alter saved device settings.

### 4. Install, verify recognition, and rollback

After an explicit deployment go-ahead and successful preflight/checksum comparison, the supported signed sideload command is:

```sh
ha100_ssh \
  "/opt/couch/runtime/current/couch-confd integrations --keys-dir /opt/couch/integration-keys/custom/developer install-sideload /tmp/$APK_NAME"
```

Capture its stdout/stderr locally. On success, verify package recognition/version:

```sh
ha100_ssh '/opt/couch/runtime/current/couch-confd integrations list'
```

CouchOS admission performs the package protocol handshake, but the HTP-1 WebSocket client connects when Couch first asks the integration for status or inputs. Use the paired CouchOS UI's saved integration connection and read-only **Refresh status** / input enumeration to exercise startup and device connection. Do not create duplicate connections or change a saved HTP-1 address without reviewing the current connection first.

This is a first install, so there is no prior integration version for `integrations rollback` to restore. If live validation fails after package activation, the supported first-install recovery is to remove the newly installed package, then verify it is absent and the saved connection remains. This is a future recovery action and requires explicit authorization; it was not performed during preparation:

```sh
ha100_ssh \
  '/opt/couch/runtime/current/couch-confd integrations remove monoprice-htp1'
ha100_ssh '/opt/couch/runtime/current/couch-confd integrations list'
```

The package guide states removal retains saved connection records, while integration commands become unavailable until the package is installed again. The documented `integrations rollback monoprice-htp1` is only applicable if an actual verified previous slot exists; that would contradict this first-install preflight and must trigger a stop/review rather than an automatic action. Package rollback/removal does not restore HTP-1 audio settings; this integration only changes receiver state when a user command is issued.

### 5. Diagnostics and error collection

- Preserve the terminal output from the read-only preflight, install, post-install list, and any rollback command. Do not enable `ssh -v`, `set -x`, or print environment variables/credentials.
- For startup/device errors, use the paired CouchOS connection page's status, input enumeration, and declared-command result/error display; record the exact visible error and time. The CouchOS guide says these operations go through the package API.
- The inspected Couch plugin host connects the plugin child's stderr to `/dev/null` (`clients/couch-plugin/src/host.rs:242-248`). Therefore, do not expect plugin stderr to be a readable log stream.
- The available documentation does not identify a supported production HA100 command/path for retrieving couch-confd or plugin-host logs. `/tmp/confd.log` appears in a development web UI launch example, not as a production HA100 log contract. If the CLI and UI diagnostics are insufficient, stop and obtain the supported log-collection method for the exact HA100 runtime before trying any log command or service manipulation.

## Separate live validation checklist

Physical HTP-1 validation is a future action requiring separate explicit approval; deployment approval alone does not authorize device controls. Perform this checklist only after separate deployment and physical-validation approval, package admission, and selection/review of the intended HTP-1 connection. Record pass/fail, observed values, timestamp, and UI/CLI errors without including credentials.

### Connection and state (read-only)

- [ ] `ha100_ssh '/opt/couch/runtime/current/couch-confd integrations list'` shows `monoprice-htp1` at the expected manifest version.
- [ ] CouchOS recognizes the package in its Integrations/Connections UI.
- [ ] A status request starts the package successfully; no package startup/protocol error is shown.
- [ ] The configured HTP-1 host/port/TLS values are reviewed in the UI (do not confuse the HA100 address with the HTP-1 address).
- [ ] HTP-1 WebSocket connection to `/ws/controller` succeeds.
- [ ] Initial `getmso` completes and a full `mso` snapshot is received.
- [ ] Status reports power, mute, current input, and volume from that snapshot.
- [ ] Visible inputs enumerate and match the HTP-1's current input set.
- [ ] Record any disconnect, timeout, protocol, missing-field, or invalid-state messages before trying a control.

### Basic controls (after read-only connection checks pass)

- [ ] Mute on/off works and is confirmed by refreshed status.
- [ ] If safe for the current listening conditions, change volume by one 1 dB step and confirm status; restore the starting level if the owner wants it restored.
- [ ] Change input only after the owner selects a safe, known input and accepts the playback interruption; verify the resulting input by refreshed status.
- [ ] Test power only if the owner explicitly approves the interruption and the receiver's current state makes the test safe. Confirm state after the command.
- [ ] Do not test Dirac, upmix, night mode, PEQ, seat shakers, or any Protocol 3 custom commands in this first validation.

## Unknowns and review gates

- The owner reports successful key-only SSH access, but Codex has not run the deployment preflight. The exact installed CouchOS build, package absence, signing trust-key state, and saved connection settings remain to be confirmed. Per the owner, this is a first install and there is no prior HTP-1 package version to roll back to.
- The HA100 has been reported to support Protocols 1–5. The future preflight must verify Protocol 3 specifically on the actual runtime; Protocol 3 is the required package compatibility gate.
- The P3 APK is built and locally signature-verified, but package admission and physical HTP-1 behavior remain unverified.
- No production log retrieval mechanism was found in the inspected instructions; avoid guessed `logread`, `journalctl`, service, or file-path commands.
- Packaging previously completed with warnings recorded in `DEVELOPMENT_STATUS.md`. A successful build/package does not establish physical HTP-1 functionality or validate the device response to commands.
- Before deployment, the preflight must confirm the package is absent; if it is present, stop. The existing verified APK version is used as-is. Deployment itself requires a separate explicit go-ahead.

## Sources inspected

- Project `tools/build-deploy.sh`: target default, Protocol 3 support probe, public-key staging, APK transfer with local/remote SHA-256 gate, signed sideload, and installed-version check.
- CouchOS `docs/development/packaging.md:144-215`: supported SSH/sideload procedure, Alpine-vs-initramfs path distinction, and warning against direct `apk add`.
- CouchOS `docs/integration-packages.md:253-274`: CLI command set, immutable slots, protocol handshake, saved-setting check, and rollback semantics.
- CouchOS `docs/integration-management.md:17-25, 90-101`: install/update/previous-version UI and version immutability.
- CouchOS `docs/webui.md:656-697`: connection catalog, saved settings, status/input/command test path through `/plugin/action`.
- Couch source inspected at `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`: `daemon/couch-confd/src/main.rs` (version and protocol probe CLI); `clients/couch-plugin/src/host.rs:242-248` (plugin stderr handling).
