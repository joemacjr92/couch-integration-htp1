# CouchOS Monoprice HTP-1 Integration

A Rust integration for the Monoprice HTP-1 home theater processor. It connects
to the HTP-1 over your local network and provides power, volume, mute, input
selection, status reporting, and supported upmixer commands.

## Installation

### 1. Add the HTP-1 package repository

In CouchOS, open **Integrations** and choose **Add a custom repository**. Enter
these values:

| Field | Value |
| --- | --- |
| Repository ID | `monoprice-htp1` |
| Name | `Monoprice HTP-1 Preview` |
| Base URL | `https://joemacjr92.github.io/couch-integration-htp1/preview` |
| Public signing key | [monoprice-htp1-repository-public.pem](keys/monoprice-htp1-repository-public.pem) |

The public key is also available as a [raw PEM file](https://raw.githubusercontent.com/joemacjr92/couch-integration-htp1/main/keys/monoprice-htp1-repository-public.pem).

### 2. Verify and trust the signing key

1. Paste the contents of the public PEM file into CouchOS.
2. Select **Check public-key fingerprint**.
3. Confirm that CouchOS displays this normalized PEM SHA-256 fingerprint:

   `ad5ecdedddfbf48e8597053c2df0679c858af9ff4cdff879df50b9e33eeac79d`

4. If it matches, select **Trust repository**.
5. Refresh packages and install **Monoprice HTP-1**.

Always compare the fingerprint shown by CouchOS before trusting the repository.
The value above is the normalized PEM fingerprint CouchOS displays; it is not
the DER public-key fingerprint.

If you already have the locally sideloaded HTP-1 integration installed, remove
that local package before installing the repository-managed package with the
same integration ID. Adding this repository does not automatically convert a
sideloaded installation into a repository-managed one.

### 3. Configure the HTP-1 connection

After installation, configure the HTP-1 connection in CouchOS with:

- **Device address** — the HTP-1 IP address or hostname.
- **WebSocket port** — `80` by default.
- **TLS** — optional and disabled by default.

For repository setup details, see the
[trusted-repository guide](docs/TRUSTED_REPOSITORY.md).

## Features

- Power on and power off.
- Volume up and down in 1 dB steps.
- Mute, mute on, and mute off.
- Select HTP-1 inputs that the device marks as visible.
- Report power, mute, current input, and volume status.
- Protocol 3 custom upmixer commands:
  - Auro-3D (`auro`) — `x:upmix-auro`
  - Dolby Surround (`dolby`) — `x:upmix-dolby`
  - DTS Neural:X (`dts`) — `x:upmix-dts`
  - Native (`native`) — `x:upmix-native`
  - Stereo (`stereo`) — `x:upmix-stereo`

All five custom upmixer commands have been tested with an HTP-1. The complete
repository-managed version `0.1.1` installation has also been tested
successfully on the CouchOS remote.

## Compatibility

- **Device:** Monoprice HTP-1.
- **CouchOS protocol:** Protocol 3; minimum core protocol requirement is 3.
- **Package architecture:** ARMv7.

No firmware-version range is specified. Compatibility with other Monoprice
products or other receivers is not claimed.

## Not Currently Supported

- Dirac Live calibration selection.
- Night mode.
- PEQ / parametric equalizer controls.
- Seat-shaker controls.
- A dynamic upmixer selector or dropdown. The five supported modes are exposed
  as custom commands, not as a live selector.

## Security

The integration communicates with the HTP-1 over your local network. By
default, the connection uses port `80` with TLS disabled; TLS can be enabled
in the connection settings. Keep the HTP-1 and CouchOS on a trusted local
network.

The repository publishes only the public signing key so CouchOS users can
verify the package feed. The corresponding private signing key is not included
in the repository.

## Credits and Acknowledgements

- **[dangerouslaser](https://github.com/dangerouslaser)** created CouchOS and
  maintains its SDK and integration ecosystem.
- **[Meir Miyara / Mase](https://github.com/mase1981)** created the original
  [Unfolded Circle HTP-1 integration](https://github.com/mase1981/uc-intg-monoprice-htp1).
  This project is a Rust port and adaptation of that work.
