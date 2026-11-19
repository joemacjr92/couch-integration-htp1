#!/bin/sh
# Exercise signed HTP-1 APK admission with a disposable key and ARMv7 Couch host.
# This never uses the release key or contacts a device.
set -eu

usage() {
    echo "Usage: $0 HTP1_BINARY COUCH_CONFD SDK_ROOT PLUGIN_MANIFEST OUTPUT_DIR" >&2
    exit 64
}
[ "$#" = 5 ] || usage

binary=$1
confd=$2
sdk=$3
manifest=$4
output=$5

for path in "$binary" "$confd" "$sdk" "$manifest"; do
    case "$path" in /*) ;; *) echo "paths must be absolute: $path" >&2; exit 64;; esac
done
[ -x "$binary" ] || { echo "HTP-1 binary is not executable: $binary" >&2; exit 1; }
[ -x "$confd" ] || { echo "couch-confd is not executable: $confd" >&2; exit 1; }
[ -f "$sdk/tools/integrations/build-apk.sh" ] || { echo "pinned SDK package tools are missing" >&2; exit 1; }
[ -f "$manifest" ] || { echo "plugin manifest is missing: $manifest" >&2; exit 1; }
[ ! -e "$output" ] || { echo "output directory must not already exist: $output" >&2; exit 1; }

mkdir -p "$output"

docker run --rm --platform linux/arm/v7 \
    -v "$binary:/work/input/couch-plugin-htp1:ro" \
    -v "$confd:/work/input/couch-confd:ro" \
    -v "$manifest:/work/input/plugin.json:ro" \
    -v "$sdk:/work/sdk:ro" \
    -v "$output:/out:rw" \
    alpine:3.24 sh -eu -c '
        apk add --no-cache alpine-sdk binutils jq >/dev/null

        # Generate a key that exists only inside this disposable container.
        abuild-keygen -a -n >/dev/null
        key=$(find /root/.abuild -type f -name "*.rsa" ! -name "*.rsa.pub" -print -quit)
        test -n "$key"
        test -f "$key.pub"
        cp "$key.pub" /etc/apk/keys/

        version=$(jq -er ".version" /work/input/plugin.json)
        mkdir -p /out/packages

        apk_file="/out/packages/couch-integration-monoprice-htp1-$version-r0.apk"
        sh /work/sdk/tools/integrations/build-apk.sh \
            monoprice-htp1 "$version" /work/input/couch-plugin-htp1 \
            /work/input/plugin.json "$key" /out/packages
        test -s "$apk_file"

        # Verify the APK signature independently before passing it to Couch.
        apk --keys-dir /root/.abuild verify "$apk_file"

        # Exercise signed sideload admission with the production Couch installer.
        /work/input/couch-confd integrations \
            --root /tmp/htp1-sideload \
            --keys-dir /root/.abuild \
            install-sideload "$apk_file" >/out/sideload.log 2>&1
        /work/input/couch-confd integrations --root /tmp/htp1-sideload list \
            | grep -Fx "monoprice-htp1 $version"

        # An empty trust directory must reject the otherwise valid signed APK.
        mkdir -m 700 /tmp/untrusted-keys
        if /work/input/couch-confd integrations \
            --root /tmp/htp1-untrusted \
            --keys-dir /tmp/untrusted-keys \
            install-sideload "$apk_file" >/out/untrusted.log 2>&1; then
            echo "untrusted APK was unexpectedly admitted" >&2
            exit 1
        fi
        test ! -e /tmp/htp1-untrusted/state/monoprice-htp1

        # Appending bytes invalidates the signed APK; admission must reject it.
        cp "$apk_file" /tmp/htp1-tampered.apk
        printf "\001" >>/tmp/htp1-tampered.apk
        if /work/input/couch-confd integrations \
            --root /tmp/htp1-tampered \
            --keys-dir /root/.abuild \
            install-sideload /tmp/htp1-tampered.apk >/out/tampered.log 2>&1; then
            echo "tampered APK was unexpectedly admitted" >&2
            exit 1
        fi
        test ! -e /tmp/htp1-tampered/state/monoprice-htp1

        sha256sum "$apk_file" > /out/checksums.txt
        printf "%s\n" "signed sideload: passed" \
            "untrusted signature rejection: passed" \
            "tampered APK rejection: passed" > /out/receipt.txt
    '

echo "HTP-1 package admission smoke test passed: $output"
