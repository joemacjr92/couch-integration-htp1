#!/usr/bin/env bash
# Build, sign, and optionally sideload the CouchOS Monoprice HTP-1 integration.
# Cross-compilation runs on WSL/Linux x86_64; APK packaging runs in Alpine ARMv7.
set -Eeuo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
SDK_ROOT="$(cd "$PLUGIN_ROOT/../couch-sdk-reference" 2>/dev/null && pwd -P)" || {
  printf 'ERROR: SDK checkout not found at %s\n' "$PLUGIN_ROOT/../couch-sdk-reference" >&2
  exit 1
}
TARGET="armv7-unknown-linux-musleabihf"
ID="monoprice-htp1"
DEVICE_DEFAULT="10.0.0.25"

CONTAINER_ENGINE="${CONTAINER_ENGINE:-podman}"
ALPINE_IMAGE="${ALPINE_IMAGE:-alpine:3.24}"
CONTAINER_PLATFORM="${CONTAINER_PLATFORM:-linux/arm/v7}"

PRIVATE_KEY_DEFAULT="$HOME/.abuild/-6afc809d.rsa"
PUBLIC_KEY_DEFAULT="$HOME/.abuild/-6afc809d.rsa.pub"
OUTPUT_DIR="$PLUGIN_ROOT/build/integrations"

ZIG_VERSION="0.15.2"
ZIG_HOST="x86_64-linux"
ZIG_SHA256="02aa270f183da276e5b5920b1dac44a63f1a49e55050ebde3aecc9eb82f93239"
ZIG_HOME="$HOME/toolchains"
ZIG_DIR="$ZIG_HOME/zig-$ZIG_HOST-$ZIG_VERSION"
ZIG_BIN="$ZIG_DIR/zig"
ZIG_ARCHIVE="$ZIG_HOME/zig-$ZIG_HOST-$ZIG_VERSION.tar.xz"

log() { printf '\n==> %s\n' "$*"; }
die() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "Required command not found: $1"; }

[[ -f "$PLUGIN_ROOT/Cargo.toml" && -f "$PLUGIN_ROOT/plugin.json" ]] || die "HTP-1 Cargo.toml or plugin.json not found."
[[ -f "$SDK_ROOT/tools/arm-cc-env.sh" && -f "$SDK_ROOT/tools/fetch-zig.sh" ]] || die "SDK checkout/tools not found at $SDK_ROOT"
[[ -f "$SDK_ROOT/tools/integrations/build-apk.sh" ]] || die "SDK APK builder not found."

for c in cargo rustup curl tar sha256sum file stat sed grep head "$CONTAINER_ENGINE"; do need "$c"; done

MANIFEST_ID="$(sed -nE 's/^[[:space:]]*"id"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$PLUGIN_ROOT/plugin.json" | head -n1)"
[[ "$MANIFEST_ID" == "$ID" ]] || die "Expected manifest id '$ID', found '${MANIFEST_ID:-<missing>}'"
VERSION="$(sed -nE 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' "$PLUGIN_ROOT/plugin.json" | head -n1)"
[[ -n "$VERSION" ]] || die "Could not read version from plugin.json."
[[ "$VERSION" =~ ^[A-Za-z0-9._+-]+$ ]] || die "Invalid package version: $VERSION"

printf 'Building HTP-1 integration version: %s\n' "$VERSION"
APK_NAME="couch-integration-$ID-$VERSION-r0.apk"
mkdir -p "$OUTPUT_DIR"
APK_PATH="$OUTPUT_DIR/$APK_NAME"
[[ ! -e "$APK_PATH" ]] || die "Refusing to overwrite existing artifact: $APK_PATH"

read -r -p "Private signing key [$PRIVATE_KEY_DEFAULT]: " PRIVATE_KEY
PRIVATE_KEY="${PRIVATE_KEY:-$PRIVATE_KEY_DEFAULT}"
read -r -p "Public signing key [$PUBLIC_KEY_DEFAULT]: " PUBLIC_KEY
PUBLIC_KEY="${PUBLIC_KEY:-$PUBLIC_KEY_DEFAULT}"
[[ -f "$PRIVATE_KEY" ]] || die "Private signing key not found: $PRIVATE_KEY"
[[ -f "$PUBLIC_KEY" ]] || die "Public signing key not found: $PUBLIC_KEY"
[[ "$(stat -c '%a' "$PRIVATE_KEY")" == "600" ]] || printf 'WARNING: private key permissions are not 600.\n' >&2

log "Checking $CONTAINER_ENGINE ARMv7 execution ($ALPINE_IMAGE)"
if ! "$CONTAINER_ENGINE" run --rm --platform "$CONTAINER_PLATFORM" "$ALPINE_IMAGE" sh -c 'uname -m'; then
  log "ARMv7 emulation is unavailable. Attempting to register QEMU with binfmt_misc."
  need sudo
  sudo "$CONTAINER_ENGINE" run --rm --privileged docker.io/tonistiigi/binfmt --install arm ||
    die "Could not register ARM emulation. No build or deployment was attempted."
  log "Rechecking ARMv7 execution"
  "$CONTAINER_ENGINE" run --rm --platform "$CONTAINER_PLATFORM" "$ALPINE_IMAGE" sh -c 'uname -m' ||
    die "ARMv7 emulation is still unavailable. No build or deployment was attempted."
fi

log "Preparing pinned Zig $ZIG_VERSION toolchain in $ZIG_HOME"
[[ "$(uname -s)" == "Linux" && "$(uname -m)" == "x86_64" ]] || die "This script expects WSL/Linux x86_64 for the pinned Zig toolchain."
mkdir -p "$ZIG_HOME"
if [[ -x "$ZIG_BIN" ]]; then
  log "Using existing Zig: $ZIG_BIN"
else
  log "Downloading pinned Zig archive"
  curl -fL --retry 3 "https://ziglang.org/download/$ZIG_VERSION/zig-$ZIG_HOST-$ZIG_VERSION.tar.xz" -o "$ZIG_ARCHIVE"
  printf '%s  %s\n' "$ZIG_SHA256" "$ZIG_ARCHIVE" | sha256sum -c - || die "Zig archive checksum verification failed."
  tar -xJf "$ZIG_ARCHIVE" -C "$ZIG_HOME"
  rm -f "$ZIG_ARCHIVE"
fi
[[ -x "$ZIG_BIN" ]] || die "Zig executable not found: $ZIG_BIN"
"$ZIG_BIN" version | grep -Fxq "$ZIG_VERSION" || die "Unexpected Zig version."

log "Compiling plugin for $TARGET"
(
  cd "$SDK_ROOT"
  export ZIG="$ZIG_BIN"
  # shellcheck disable=SC1091
  . tools/arm-cc-env.sh
  cd clients
  cargo build --release --locked --target "$TARGET" --manifest-path "$PLUGIN_ROOT/Cargo.toml"
)
BINARY="$PLUGIN_ROOT/target/$TARGET/release/couch-plugin-htp1"
[[ -f "$BINARY" ]] || die "Cargo did not produce $BINARY"
file "$BINARY" | tee /dev/stderr | grep -q 'ARM' || die "Build output is not an ARM executable."

WORK="$(mktemp -d "${TMPDIR:-/tmp}/couch-htp1-build.XXXXXX")"
cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
TEMP_MANIFEST="$WORK/plugin.json"

log "Building and signing $APK_NAME in $ALPINE_IMAGE ($CONTAINER_PLATFORM)"
"$CONTAINER_ENGINE" run --rm --platform "$CONTAINER_PLATFORM" \
  -v "$SDK_ROOT:/work/sdk:ro" \
  -v "$PLUGIN_ROOT:/work/plugin:ro" \
  -v "$WORK:/work/tmp:rw" \
  -v "$OUTPUT_DIR:/work/out:rw" \
  -v "$PRIVATE_KEY:/root/.abuild/-6afc809d.rsa:ro" \
  -v "$PUBLIC_KEY:/root/.abuild/-6afc809d.rsa.pub:ro" \
  --env "BUILD_VERSION=$VERSION" \
  "$ALPINE_IMAGE" sh -eu -c '
    apk add --no-cache alpine-sdk jq binutils
    cp /root/.abuild/-6afc809d.rsa.pub /etc/apk/keys/
    cp /work/plugin/plugin.json /work/tmp/plugin.json
    sh /work/sdk/tools/integrations/build-apk.sh \
      monoprice-htp1 \
      "$BUILD_VERSION" \
      /work/plugin/target/armv7-unknown-linux-musleabihf/release/couch-plugin-htp1 \
      /work/tmp/plugin.json \
      /root/.abuild/-6afc809d.rsa \
      /work/out
  '

[[ -f "$APK_PATH" ]] || die "Container completed without producing $APK_PATH"
APK_ENTRIES="$(tar -tzf "$APK_PATH" 2>/dev/null)" || die "Could not read APK archive."
printf '%s\n' "$APK_ENTRIES" | grep '^\.SIGN\.RSA\.' >/dev/null || die "APK signature entry not found."
sha256sum "$APK_PATH"
ls -lh "$APK_PATH"

log "Build complete: $APK_PATH"

printf '\nWARNING: A successful build and package operation does not establish physical HTP-1 functionality or validate the device response to integration commands.\n'
read -r -p "Deploy this package to the HA100 now? [y/N]: " DEPLOY
if [[ "$DEPLOY" =~ ^[Yy]$ ]]; then
  need ssh
  need scp
  read -r -p "HA100 IP/hostname [$DEVICE_DEFAULT]: " DEVICE
  DEVICE="${DEVICE:-$DEVICE_DEFAULT}"
  SSH_ARGS=(
    -F /dev/null -T
    -i "$HOME/.ssh/id_ed25519_ha100"
    -o "UserKnownHostsFile=$HOME/.ssh/known_hosts"
    -o StrictHostKeyChecking=yes
    -o IdentitiesOnly=yes
    -o BatchMode=yes
    -o PreferredAuthentications=publickey
    -o PasswordAuthentication=no
  )
  SCP_ARGS=(
    -B
    -F /dev/null
    -i "$HOME/.ssh/id_ed25519_ha100"
    -o "UserKnownHostsFile=$HOME/.ssh/known_hosts"
    -o StrictHostKeyChecking=yes
    -o IdentitiesOnly=yes
    -o BatchMode=yes
    -o PreferredAuthentications=publickey
    -o PasswordAuthentication=no
  )
  [[ -r "$HOME/.ssh/id_ed25519_ha100" ]] || die "Dedicated HA100 SSH identity is not readable."
  [[ -r "$HOME/.ssh/known_hosts" ]] || die "Explicit known_hosts file is not readable."
  ha100_ssh() { ssh -T "${SSH_ARGS[@]}" "root@$DEVICE" "$@"; }
  PROTOCOL_VERSION="$(sed -nE 's/^[[:space:]]*"protocol_version"[[:space:]]*:[[:space:]]*([0-9]+).*/\1/p' "$PLUGIN_ROOT/plugin.json" | head -n1)"
  [[ -n "$PROTOCOL_VERSION" ]] || die "Could not read protocol_version from plugin.json."
  log "Checking protocol $PROTOCOL_VERSION support on root@$DEVICE"
  ha100_ssh "/opt/couch/runtime/current/couch-confd --supports-integration-protocol=$PROTOCOL_VERSION" || die "Protocol support check failed. Package remains built locally."
  INSTALLED_LIST="$(ha100_ssh '/opt/couch/runtime/current/couch-confd integrations list')" || die "Could not inspect installed integrations. Package remains built locally."
  printf '%s\n' "$INSTALLED_LIST"
  if grep -Eq "(^|[[:space:]])$ID([[:space:]]|$)" <<< "$INSTALLED_LIST"; then
    die "$ID is already installed; first-install preflight failed. No files were transferred."
  fi
  log "Confirmed $ID is absent before first-install staging."
  ha100_ssh 'mkdir -p /opt/couch/integration-keys/custom/developer'
  scp "${SCP_ARGS[@]}" "$PUBLIC_KEY" "root@$DEVICE:/opt/couch/integration-keys/custom/developer/-6afc809d.rsa.pub"
  scp "${SCP_ARGS[@]}" "$APK_PATH" "root@$DEVICE:/tmp/$APK_NAME"
  LOCAL_APK_CHECKSUM="$(sha256sum "$APK_PATH")" || die "Could not calculate local APK SHA-256."
  LOCAL_APK_SHA256="${LOCAL_APK_CHECKSUM%%[[:space:]]*}"
  [[ "$LOCAL_APK_SHA256" =~ ^[[:xdigit:]]{64}$ ]] || die "Local APK checksum did not produce a valid SHA-256 hash."
  REMOTE_APK_CHECKSUM="$(ha100_ssh "sha256sum /tmp/$APK_NAME")" || die "Could not calculate remote APK SHA-256; refusing installation."
  if [[ "$REMOTE_APK_CHECKSUM" =~ ^([[:xdigit:]]{64})[[:space:]]+(/tmp/[^[:space:]]+)$ ]]; then
    REMOTE_APK_SHA256="${BASH_REMATCH[1]}"
    REMOTE_APK_PATH="${BASH_REMATCH[2]}"
  else
    die "Remote APK checksum output was empty or invalid; refusing installation."
  fi
  [[ "$REMOTE_APK_PATH" == "/tmp/$APK_NAME" ]] || die "Remote checksum was for an unexpected file; refusing installation."
  [[ "$REMOTE_APK_SHA256" == "$LOCAL_APK_SHA256" ]] || die "Local and remote APK SHA-256 hashes differ; refusing installation."
  printf 'Local APK SHA-256:  %s\n' "$LOCAL_APK_SHA256"
  printf 'Remote APK SHA-256: %s\n' "$REMOTE_APK_SHA256"
  ha100_ssh "/opt/couch/runtime/current/couch-confd integrations --keys-dir /opt/couch/integration-keys/custom/developer install-sideload /tmp/$APK_NAME"
  INSTALLED_LIST="$(ha100_ssh '/opt/couch/runtime/current/couch-confd integrations list')"
  printf '%s\n' "$INSTALLED_LIST"
  grep -Fxq "$ID $VERSION" <<< "$INSTALLED_LIST" || die "Installed integration version was not confirmed."
  log "Deployment verified: $ID $VERSION is installed on $DEVICE."
else
  log "Deployment skipped. APK is ready at $APK_PATH"
fi
