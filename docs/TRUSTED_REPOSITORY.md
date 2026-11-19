# HTP-1 trusted repository publishing

This document describes the HTP-1 adaptation of the Fire TV trusted-repository
workflow. The package ID remains `monoprice-htp1`, the Cargo binary target is
`couch-plugin-htp1`, the installed executable is
`bin/couch-plugin-monoprice-htp1`, and the package targets ARMv7 with Protocol 3.
The Couch SDK pin remains `e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`.

## Admission checks

`.github/workflows/admission.yml` runs on pull requests and pushes to `main`.
It checks the integration catalog metadata and Protocol 3 manifest, formatting,
and host tests. Its ARMv7 job fetches the exact Couch SDK source, uses the SDK's
pinned Zig and ARM linker configuration, cross-builds the integration and the
real `couch-confd`, then runs `tools/smoke-package.sh` under ARM emulation.

`tests/admission.rs` runs the compiled executable through Couch's package host
to check the executable's embedded manifest handshake. The ignored test runs in
GitHub Actions after the release version is embedded and before the ARM package
is built. The smoke test uses a disposable signing key and verifies successful
sideload admission, rejection with an empty trust directory, and rejection of
an APK whose bytes were changed. It does not use a release key or device.

## Tagged release

`.github/workflows/release.yml` runs only for `v*` tags. It requires the first
published HTP-1 package to be exactly `0.1.1`; later tags must be newer than all
versions already in the feed. Release tags must have three numeric components,
such as `v0.1.1` and `v0.1.2`.

The workflow updates `plugin.json` in the runner before compiling because the
manifest is embedded in the binary. Cargo's crate version stays separate. The
workflow then runs the real admission handshake, cross-compiles ARMv7, uses the
pinned SDK `tools/integrations/build-apk.sh` in Alpine, and cryptographically
verifies the APK with Alpine `apk verify` and the committed repository public
key. It also verifies the packaged manifest and binary against the exact build
inputs. The signed `APKINDEX.tar.gz` is independently checked with `apk verify`
using that same public key, and the OpenSSL feed signature is verified before
publishing. It does not invoke `tools/build-deploy.sh`, SSH, SCP, or the HA100.

The workflow reads these repository Actions secrets:

- `APK_PRIVATE_KEY`: RSA private signing key.
- `APK_PUBLIC_KEY`: matching PEM public key.

Both are written to runner-temporary files, mounted into the package container,
and mapped to Alpine's expected `-6afc809d.rsa` signer name. The private file is
mode 0600 and is never copied into the repository or printed. Before packaging,
the workflow verifies that the private key, public-key secret, and committed
repository public key have the same DER public-key identity. Do not put private
key data in `keys/`, workflow YAML, build logs, or repository files.

The SDK `build-repository.sh` preserves APK signing and creates the signed
`APKINDEX.tar.gz`. The workflow also creates `feed.json`, which includes the
package filename, size, SHA-256, Protocol 3 requirements, index checksum,
sequence, issue time, and 30-day expiry. `openssl dgst -sha256 -sign` signs the
metadata with the same RSA private key. Older HTP-1 APK versions are carried
forward from `gh-pages` so a new release does not remove versions already in the
feed.

The published tree is:

```text
preview/armv7/
  APKINDEX.tar.gz
  feed.json
  feed.json.sig
  couch-integration-monoprice-htp1-<version>-r0.apk
```

The release workflow publishes this tree to the `gh-pages` branch root and
attaches the current APK to a GitHub Release. The repository base URL is
`https://joemacjr92.github.io/couch-integration-htp1/preview`.

## Feed metadata renewal

CouchOS remembers whether signed metadata has been accepted for each custom
repository. Once accepted, missing or expired metadata makes that repository
unusable on refresh. The feed signature expires after 30 days, so
`.github/workflows/renew-feed.yml` renews it weekly without rebuilding the APK
or changing the signed APK index.

`tools/renew-feed-metadata.sh` verifies the old metadata signature, its binding
to the current index, every listed APK's size and SHA-256, and the signing-key
pair. It increments the feed sequence, writes new issue/expiry times, signs the
new metadata, and verifies the result before the workflow republishes the Pages
tree. The scheduled job serializes with release publishing through a shared
Actions concurrency group.

## Public key and fingerprint

Only the public PEM is committed at
`keys/monoprice-htp1-repository-public.pem`. The current key is byte-identical
to the existing HTP-1 APK signer public key and the Fire TV repository public
key. The SHA-256 fingerprint CouchOS computes from normalized PEM text is:

```text
ad5ecdedddfbf48e8597053c2df0679c858af9ff4cdff879df50b9e33eeac79d
```

The SHA-256 of its DER-encoded public key is
`5ff5846b5d3cc1e404bd5a2ac7640414b91baecf20c2b03d0ada812c15b22a14`; this is
a different fingerprint format and is not the string displayed by the reviewed
CouchOS custom-repository flow. Always compare the value on the target CouchOS
UI before trusting a repository.

## GitHub setup and operator flow

One-time repository setup requires the `APK_PRIVATE_KEY` and `APK_PUBLIC_KEY`
Actions secrets, Actions write permission for `contents`, and GitHub Pages
configured to publish the `gh-pages` branch from its root. The Pages branch is
created by the first successful release publication. GitHub settings and the
secrets are outside these workflows and must be configured by a repository
administrator.

After a reviewed `v0.1.1` tag is pushed, verify the Actions run, GitHub Release,
Pages URL, package/index signatures, feed sequence, expiry, and hashes before
using the repository on a device. The HTP-1 is currently reported as installed
from a sideload. CouchOS records repository origin through its managed install
flow; an existing sideload may not show a repository Update action just because
the feed was added. Review that source transition and the retained connection
state before any device-side remove/reinstall. Device installation or changes
require separate approval.
