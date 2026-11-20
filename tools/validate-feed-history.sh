#!/usr/bin/env bash
set -Eeuo pipefail

usage() {
  echo "Usage: $0 FEED_ARCH_DIR REPOSITORY_PUBLIC_KEY NEW_VERSION [--allow-existing-version]" >&2
  exit 64
}

[[ "$#" == 3 || "$#" == 4 ]] || usage
FEED_DIR="$1"
PUBLIC_KEY="$2"
NEW_VERSION="$3"
ALLOW_EXISTING_VERSION=false
if [[ "$#" == 4 ]]; then
  [[ "$4" == "--allow-existing-version" ]] || usage
  ALLOW_EXISTING_VERSION=true
fi

[[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
  echo "ERROR: invalid release version: $NEW_VERSION" >&2
  exit 1
}

FEED="$FEED_DIR/feed.json"
SIGNATURE="$FEED_DIR/feed.json.sig"
INDEX="$FEED_DIR/APKINDEX.tar.gz"
PACKAGE_LIST="${RUNNER_TEMP:-/tmp}/htp1-verified-packages.tsv"

# A missing feed directory is a first-install case. Preserve the HTP-1
# repository's original first trusted release requirement.
if [[ ! -e "$FEED_DIR" ]]; then
  [[ "$NEW_VERSION" == "0.1.1" ]] || {
    echo "ERROR: the first trusted-repository release must be v0.1.1; refusing $NEW_VERSION." >&2
    exit 1
  }
  : > "$PACKAGE_LIST"
  echo "First trusted HTP-1 repository release validated: $NEW_VERSION."
  exit 0
fi

for file in "$FEED" "$SIGNATURE" "$INDEX" "$PUBLIC_KEY"; do
  [[ -s "$file" ]] || {
    echo "ERROR: missing or empty existing feed file: $file" >&2
    exit 1
  }
done
command -v jq >/dev/null
command -v openssl >/dev/null
command -v sha256sum >/dev/null
command -v stat >/dev/null
command -v sort >/dev/null

jq -e '
  .schema == 1 and
  .channel == "preview" and
  (.sequence | type == "number" and . >= 1 and . == floor and . < 2147483647) and
  .index.path == "APKINDEX.tar.gz" and
  (.index.size | type == "number" and . > 0 and . == floor) and
  (.index.sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
  (.packages | type == "array" and length > 0) and
  all(.packages[];
    .id == "monoprice-htp1" and
    .protocol_version == 3 and
    .min_core_protocol_version == 3 and
    (.version | type == "string" and test("^[0-9]+\\.[0-9]+\\.[0-9]+$")) and
    (.apk | type == "string") and
    (.apk == ("couch-integration-monoprice-htp1-" + .version + "-r0.apk")) and
    (.size | type == "number" and . > 0 and . == floor) and
    (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
  ) and
  ([.packages[].apk] | unique | length) == (.packages | length) and
  ([.packages[].version] | unique | length) == (.packages | length)
' "$FEED" >/dev/null || {
  echo "ERROR: existing HTP-1 feed metadata failed validation." >&2
  exit 1
}

openssl dgst -sha256 -verify "$PUBLIC_KEY" \
  -signature "$SIGNATURE" "$FEED" >/dev/null || {
  echo "ERROR: existing HTP-1 feed signature is invalid." >&2
  exit 1
}

INDEX_SIZE="$(stat -c '%s' "$INDEX")"
INDEX_SHA256="$(sha256sum "$INDEX" | awk '{print $1}')"
EXPECTED_INDEX_SIZE="$(jq -er '.index.size | numbers' "$FEED")"
EXPECTED_INDEX_SHA256="$(jq -er '.index.sha256 | strings' "$FEED")"
[[ "$INDEX_SIZE" == "$EXPECTED_INDEX_SIZE" && "$INDEX_SHA256" == "$EXPECTED_INDEX_SHA256" ]] || {
  echo "ERROR: existing APKINDEX does not match the signed feed metadata." >&2
  exit 1
}

jq -er '.packages[] | [.version, .apk, (.size | tostring), .sha256] | @tsv' \
  "$FEED" > "$PACKAGE_LIST"
while IFS=$'\t' read -r METADATA_VERSION APK_NAME EXPECTED_SIZE EXPECTED_SHA256; do
  APK_VERSION="${APK_NAME#couch-integration-monoprice-htp1-}"
  APK_VERSION="${APK_VERSION%-r0.apk}"
  [[ "$APK_VERSION" == "$METADATA_VERSION" ]] || {
    printf 'ERROR: feed version %s does not match APK filename %s.\n' \
      "$METADATA_VERSION" "$APK_NAME" >&2
    exit 1
  }
  APK_PATH="$FEED_DIR/$APK_NAME"
  [[ -s "$APK_PATH" ]] || {
    printf 'ERROR: existing feed APK is missing or empty: %s\n' "$APK_NAME" >&2
    exit 1
  }
  APK_SIZE="$(stat -c '%s' "$APK_PATH")"
  APK_SHA256="$(sha256sum "$APK_PATH" | awk '{print $1}')"
  [[ "$APK_SIZE" == "$EXPECTED_SIZE" && "$APK_SHA256" == "$EXPECTED_SHA256" ]] || {
    printf 'ERROR: existing feed APK does not match signed metadata: %s\n' "$APK_NAME" >&2
    exit 1
  }
done < "$PACKAGE_LIST"

mapfile -t VERSIONS < <(jq -r '.packages[].version' "$FEED" | sort -V)
HIGHEST="${VERSIONS[-1]}"
LATEST="$(printf '%s\n' "$HIGHEST" "$NEW_VERSION" | sort -V | tail -n 1)"
if [[ "$NEW_VERSION" == "$HIGHEST" && "$ALLOW_EXISTING_VERSION" == true ]]; then
  echo "Manual dry-run validation: rebuilding existing HTP-1 version $NEW_VERSION; publishing remains disabled."
elif [[ "$LATEST" != "$NEW_VERSION" || "$NEW_VERSION" == "$HIGHEST" ]]; then
  echo "ERROR: release $NEW_VERSION must be newer than existing HTP-1 version $HIGHEST." >&2
  exit 1
fi

echo "Existing signed HTP-1 feed metadata, APKINDEX, APK history, and version order verified."
