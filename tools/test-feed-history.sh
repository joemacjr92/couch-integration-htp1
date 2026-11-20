#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd -P)"
VALIDATOR="$ROOT/tools/validate-feed-history.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
command -v jq >/dev/null
command -v openssl >/dev/null
command -v sha256sum >/dev/null

openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "$WORK/private.pem" >/dev/null 2>&1
openssl pkey -in "$WORK/private.pem" -pubout -out "$WORK/public.pem" >/dev/null
export RUNNER_TEMP="$WORK"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }
make_fixture() {
  local dir="$WORK/feed"
  rm -rf "$dir"
  mkdir -p "$dir"
  printf 'test APKINDEX bytes\\n' > "$dir/APKINDEX.tar.gz"
  printf 'test HTP-1 APK bytes\\n' > "$dir/couch-integration-monoprice-htp1-0.1.1-r0.apk"
  local index_size index_sha apk_size apk_sha
  index_size="$(stat -c '%s' "$dir/APKINDEX.tar.gz")"
  index_sha="$(sha256sum "$dir/APKINDEX.tar.gz" | awk '{print $1}')"
  apk_size="$(stat -c '%s' "$dir/couch-integration-monoprice-htp1-0.1.1-r0.apk")"
  apk_sha="$(sha256sum "$dir/couch-integration-monoprice-htp1-0.1.1-r0.apk" | awk '{print $1}')"
  jq -n --argjson is "$index_size" --arg ih "$index_sha" --argjson as "$apk_size" --arg ah "$apk_sha" '
    {schema:1,channel:"preview",sequence:2,index:{path:"APKINDEX.tar.gz",size:$is,sha256:$ih},
     packages:[{id:"monoprice-htp1",version:"0.1.1",apk:"couch-integration-monoprice-htp1-0.1.1-r0.apk",
     size:$as,sha256:$ah,protocol_version:3,min_core_protocol_version:3}]}
  ' > "$dir/feed.json"
  openssl dgst -sha256 -sign "$WORK/private.pem" -out "$dir/feed.json.sig" "$dir/feed.json"
}
sign_feed() {
  openssl dgst -sha256 -sign "$WORK/private.pem" -out "$WORK/feed/feed.json.sig" "$WORK/feed/feed.json"
}
run_ok() {
  if ! bash "$VALIDATOR" "$WORK/feed" "$WORK/public.pem" "$1" "${@:2}" >/dev/null; then
    fail "expected validator success for version $1"
  fi
}
run_bad() {
  if bash "$VALIDATOR" "$WORK/feed" "$WORK/public.pem" "$1" "${@:2}" >/dev/null 2>&1; then
    fail "expected validator rejection for version $1"
  fi
}

if bash "$VALIDATOR" "$WORK/missing-feed" "$WORK/public.pem" 0.1.1 >/dev/null; then
  pass "first trusted HTP-1 version accepted"
else
  fail "first trusted HTP-1 version rejected"
fi
if bash "$VALIDATOR" "$WORK/missing-feed" "$WORK/public.pem" 0.1.2 >/dev/null 2>&1; then
  fail "wrong first trusted version accepted"
else
  pass "wrong first trusted version rejected"
fi

make_fixture
run_ok 0.1.2
pass "valid signed feed and newer version"
run_ok 0.1.1 --allow-existing-version
pass "existing version allowed only with explicit dry-run flag"
run_bad 0.1.1
pass "existing version rejected for publication"
run_bad 0.1.0
pass "older version rejected"

make_fixture
printf 'tamper' >> "$WORK/feed/feed.json"
run_bad 0.1.2
pass "invalid feed signature rejected"

make_fixture
printf 'tamper' >> "$WORK/feed/APKINDEX.tar.gz"
run_bad 0.1.2
pass "APKINDEX size/hash mismatch rejected"

make_fixture
printf 'tamper' >> "$WORK/feed/couch-integration-monoprice-htp1-0.1.1-r0.apk"
run_bad 0.1.2
pass "historical APK size/hash mismatch rejected"

make_fixture
rm "$WORK/feed/couch-integration-monoprice-htp1-0.1.1-r0.apk"
run_bad 0.1.2
pass "missing historical APK rejected"

make_fixture
jq '.packages += .packages' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"
mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"
sign_feed
run_bad 0.1.2
pass "duplicate package/version rejected"

make_fixture
jq '.packages[0].apk = "couch-integration-monoprice-htp1-0.1.0-r0.apk"' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"
mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"
sign_feed
run_bad 0.1.2
pass "filename/version mismatch rejected"

make_fixture
jq '.sequence = 1.5' "$WORK/feed/feed.json" > "$WORK/feed/modified.json"
mv "$WORK/feed/modified.json" "$WORK/feed/feed.json"
sign_feed
run_bad 0.1.2
pass "non-integer sequence rejected"

echo "All HTP-1 feed-history validation tests passed."
