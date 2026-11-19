# HTP-1 deployment preparation and live validation

**Preparation snapshot:** 2026-11-19
**Repository:** `feature/htp1-state-sync` at `5c4b07e13fab40d53bcb1f0060a7ffb3054ddf17` (`Update HTP-1 development status`)
**HA100 address:** `10.0.0.25`
**Prepared APK:** `build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk`
**Expected SHA-256:** `29002fc8ed21b02986e24bc5811a2ec5225c3a47d5354c8e14ad12ac2d1f2460`

## Current local verification

- Branch and HEAD were inspected. The worktree was clean before preparing this document.
- The APK exists (588,792 bytes; timestamp 2026-11-19 10:27 local time) and its SHA-256 matches the expected value above.
- The owner subsequently reported a successful non-interactive key-only SSH command returning `SSH key authentication succeeded`, `root`, and `couch`. No deployment preflight inventory was run for this document. Per the current project owner, this is a first install; the preflight below must verify that no `monoprice-htp1` row exists before transfer. The exact runtime build, saved connection, and target signing trust-key state remain to be confirmed.
- The integration manifest requests Protocol 2. `DEVELOPMENT_STATUS.md` records that this HA100 runtime supports Protocols 1 through 5, including Protocol 3. Recheck the target with the runtime's supported protocol probe immediately before installation.

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
ha100_ssh '/opt/couch/runtime/current/couch-confd integrations \
  --keys-dir /opt/couch/integration-keys/custom/developer \
  install-sideload /tmp/couch-integration-monoprice-htp1-0.1.0-r0.apk'
```

The same procedure is present in this repository's `tools/build-deploy.sh:141-158` (optional path, default HA100 `10.0.0.25`) and CouchOS `docs/development/packaging.md:144-215` / `docs/integration-packages.md:253-274`. Package admission verifies the APK signature, audits its payload, checks its protocol handshake, and only then activates it. **Do not use `apk add`**; CouchOS documentation says that bypasses the package audit, handshake, immutable slots, activation record, saved-settings compatibility check, and rollback mechanism.

The documented SSH example assumes root login into the normal Alpine environment. If the SSH shell is the outer initramfs/recovery shell instead, the CouchOS guide requires the separate `chroot /mnt/alpine` procedure. This plan is for normal SSH only and must stop if `/etc/alpine-release` is absent; do not improvise recovery-shell paths.

## Proposed procedure (not executed)

### 1. Read-only connectivity and runtime preflight

First verify the SSH endpoint and expected shell. Keep normal SSH host-key checking enabled; verify the HA100 host-key fingerprint through the owner's trusted record if it is not already known. The owner has reported a successful key-only remote-command test (`SSH key authentication succeeded`, `root`, `couch`); rerun the test during deployment preflight.

```sh
ha100_ssh 'test -f /etc/alpine-release'
```

Then record the Couch config daemon version, verify protocol support (the project records that 1 through 5 are supported), and inspect installed package IDs/versions. These are read-only commands:

```sh
ha100_ssh '
  set -eu
  CONFD=/opt/couch/runtime/current/couch-confd
  "$CONFD" --version
  for protocol in 1 2 3 4 5; do
    "$CONFD" --supports-integration-protocol="$protocol"
    printf "Protocol %s: supported\n" "$protocol"
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

`couch-confd --version` is implemented by the checked Couch source (`daemon/couch-confd/src/main.rs`, `parse_args`); it identifies that binary's build version. The source and deployment guide document `--supports-integration-protocol=N` as the runtime compatibility probe. Save the version/probe/list output without SSH verbose mode or shell tracing. The probe for Protocol 2 is the required package gate; any unexpected nonzero result stops the procedure.

The CLI list is the pre-install inventory. The preflight command prints the list and exits before transfer if it finds `monoprice-htp1`. If an entry is present, stop: the live state no longer matches the first-install premise. Do not update, overwrite, or bump the APK version to work around that discrepancy.

### 2. First-install and configuration gates

Before transferring anything:

- Confirm `couch-confd integrations list` contains no `monoprice-htp1` row. If it does, stop and review the discrepancy; do not run the install command.
- No existing HTP-1 integration package needs to be preserved. This is a first install of the already-built `0.1.0-r0` APK; no version bump is needed for upgrade handling.
- A saved CouchOS connection/configuration record may still exist even when the package is absent. Review it in the paired UI and do not copy, print, edit, or remove CouchOS configuration or credential files. The CouchOS package manager performs saved-settings checks; its documentation says package removal retains saved connections.
- Do not proceed if the signature trust key, protocol probe, or intended HTP-1 connection/settings cannot be established.

### 3. Verify local artifact and transfer (future, after approval)

The build script defaults to the public key at `$HOME/.abuild/-6afc809d.rsa.pub` and copies that public key to the dedicated developer trust directory. Never transfer or display the private signing key.

```sh
APK=build/integrations/couch-integration-monoprice-htp1-0.1.0-r0.apk
PUBLIC_KEY="$HOME/.abuild/-6afc809d.rsa.pub"
sha256sum "$APK"
test -r "$PUBLIC_KEY"
```

Proceed only if the APK hash equals the expected hash at the top of this document and the public key is readable. After all read-only preflight gates pass, the documented development procedure creates the custom key directory and transfers the public key and package:

```sh
ha100_ssh 'mkdir -p /opt/couch/integration-keys/custom/developer'
ha100_ssh 'cat > /opt/couch/integration-keys/custom/developer/developer.rsa.pub' \
  < "$PUBLIC_KEY"
ha100_ssh 'cat > /tmp/couch-integration-monoprice-htp1-0.1.0-r0.apk' \
  < "$APK"
ha100_ssh 'sha256sum /tmp/couch-integration-monoprice-htp1-0.1.0-r0.apk'
```

Compare the remote checksum with the expected SHA-256 before installation. These commands intentionally do not use `apk`, restart a service, or alter saved device settings.

### 4. Install, verify recognition, and rollback

After an explicit deployment go-ahead and successful preflight/checksum comparison, the supported signed sideload command is:

```sh
ha100_ssh \
  '/opt/couch/runtime/current/couch-confd integrations \
    --keys-dir /opt/couch/integration-keys/custom/developer \
    install-sideload /tmp/couch-integration-monoprice-htp1-0.1.0-r0.apk'
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

Perform these only after deployment approval, package admission, and selection/review of the intended HTP-1 connection. Record pass/fail, observed values, timestamp, and UI/CLI errors without including credentials.

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
- The project record says the HA100 supports Protocols 1–5; the future preflight rechecks that claim on the actual runtime. This package itself declares Protocol 2.
- The prepared APK is an old build from 2026-11-19. Its expected hash matches locally, but no claim is made that it includes changes newer than that build or that it has been tested with physical hardware.
- No production log retrieval mechanism was found in the inspected instructions; avoid guessed `logread`, `journalctl`, service, or file-path commands.
- Packaging previously completed with warnings recorded in `DEVELOPMENT_STATUS.md`. The build script still prints a stale scaffold warning. Neither warning proves a device failure, and neither is resolved by APK signature/admission success.
- Before deployment, the preflight must confirm the package is absent; if it is present, stop. The existing verified APK version is used as-is. Deployment itself requires a separate explicit go-ahead.

## Sources inspected

- Project `tools/build-deploy.sh:11-13, 141-160`: target default, protocol support probe, public-key staging, APK transfer, signed sideload, and installed-version check.
- CouchOS `docs/development/packaging.md:144-215`: supported SSH/sideload procedure, Alpine-vs-initramfs path distinction, and warning against direct `apk add`.
- CouchOS `docs/integration-packages.md:253-274`: CLI command set, immutable slots, protocol handshake, saved-setting check, and rollback semantics.
- CouchOS `docs/integration-management.md:17-25, 90-101`: install/update/previous-version UI and version immutability.
- CouchOS `docs/webui.md:656-697`: connection catalog, saved settings, status/input/command test path through `/plugin/action`.
- Couch source inspected at `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`: `daemon/couch-confd/src/main.rs` (version and protocol probe CLI); `clients/couch-plugin/src/host.rs:242-248` (plugin stderr handling).
