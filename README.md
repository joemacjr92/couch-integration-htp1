# CouchOS Monoprice HTP-1 integration

A Rust CouchOS integration for the Monoprice HTP-1. It connects to the receiver
at `/ws/controller` over WebSocket and provides power, volume, mute, visible
input selection, status reporting, and five tested Protocol 3 upmixer commands.

## Trusted repository installation

After the first approved tagged release publishes it, the preview feed will be
available at:

```text
https://joemacjr92.github.io/couch-integration-htp1/preview
```

After publication, add the feed in CouchOS **Integrations → Add a custom
repository** using:

| Field | Value |
| --- | --- |
| Repository ID | `monoprice-htp1` |
| Name | `Monoprice HTP-1 Preview` |
| Base URL | `https://joemacjr92.github.io/couch-integration-htp1/preview` |
| Public signing key | [monoprice-htp1-repository-public.pem](keys/monoprice-htp1-repository-public.pem) |

Before trusting it, select **Check public-key fingerprint** and compare the
displayed SHA-256 with the normalized PEM fingerprint published here:

```text
ad5ecdedddfbf48e8597053c2df0679c858af9ff4cdff879df50b9e33eeac79d
```

This is SHA-256 of the normalized PEM text CouchOS fingerprints, not the DER
public-key fingerprint. If the values match, select **Trust repository**, then
**Refresh packages**. Install **Monoprice HTP-1** from the preview catalog.
Updates are offered by the Integrations page after refresh and require an
explicit **Update** action; they are not automatic.

The first repository package is version `0.1.1` (`v0.1.1` release tag). Do not
rebuild or republish version `0.1.0` with different bytes. An integration
installed by sideload may not have a recorded repository origin. Adding this
feed alone may therefore not create an Update action for that installation;
review the package-source migration before removing or reinstalling a working
integration. CouchOS removal preserves saved connections, but no migration is
performed automatically by this project.

## Capabilities

- Power on/off; volume up/down in 1 dB steps; mute toggle/on/off.
- Dynamic selection of visible HTP-1 inputs.
- Power, mute, input, and volume status.
- Protocol 3 custom commands: Auro, Dolby, DTS, Native, and Stereo upmix.
- Dirac, night mode, PEQ, and seat-shaker controls are not implemented.

The owner reports successful hardware validation of the functional 0.1.0
baseline, including the five upmixer commands. The published repository release
is a separately versioned package; see [development status](DEVELOPMENT_STATUS.md)
and the [trusted-repository workflow notes](docs/TRUSTED_REPOSITORY.md).

## Development

Both Couch crates remain pinned to
`e7fc36cd90faa6d6b2e3e8a733c28a3fed833d81`. Run locally:

```sh
cargo fmt --check
cargo test
cargo check
```

The tagged GitHub Actions workflow builds the Protocol 3 ARMv7 APK, checks its
package admission, and publishes the signed preview feed. A weekly workflow
renews signed feed metadata without rebuilding packages. The local
`tools/build-deploy.sh` remains the separate build/sideload path; the GitHub
workflows do not connect to or deploy to an HA100.

## Attribution

- Meir Miyara created the original [Unfolded Circle HTP-1 integration](https://github.com/mase1981/uc-intg-monoprice-htp1), which is the reference for this Rust adaptation.
- dangerouslaser created CouchOS and maintains its SDK and integration ecosystem.
