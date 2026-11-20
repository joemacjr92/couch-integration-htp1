# Remote HTP-1 test access

## Purpose

This setup allows the CouchOS HA100 development/test unit to exercise an HTP-1 at another site without exposing the HTP-1 or either site's LAN through inbound internet port forwarding.

## Traffic path

The validated test path is:

```text
CouchOS HA100
  -> configured proxy hostname and HTTP port
  -> Synology reverse proxy
  -> Tailscale private network
  -> reverse proxy on the remote-site NAS
  -> HTP-1 local HTTP service
```

The Synology reverse proxy must route the configured hostname to the remote-site proxy. The remote-site proxy forwards to the HTP-1 on its local network. The proxy chain must support WebSocket upgrades as well as ordinary HTTP requests; the HTP-1 controller WebSocket endpoint is `/ws/controller`.

No public port forwarding is required for this path. Keep the proxy listeners restricted to the intended LAN/private-network interfaces and retain the private overlay-network access controls.

## CouchOS hostname resolution requirement

The hostname configured in the HTP-1 integration must resolve on the CouchOS device itself, not merely on the developer's PC. In the validated setup, CouchOS used the local router as DNS, but that DNS service did not resolve the custom reverse-proxy hostname.

Adding a hosts-file mapping on CouchOS from the proxy hostname to the Synology's LAN address resolved the issue:

```text
<synology-lan-ip> htp1-remote
```

The corresponding one-shot command used for the lab test was:

```sh
echo '<synology-lan-ip> htp1-remote' >> /etc/hosts
```

Use the actual LAN address for the Synology in the test environment; do not copy an address from another installation. Verify resolution from CouchOS (for example, `ping -c 1 htp1-remote`) before diagnosing the integration or WebSocket protocol.

The mapping is specific to this lab's reverse-proxy hostname and network. It is not an integration code requirement for users whose DNS already resolves their configured HTP-1 host.

## Validation performed

- The HTP-1 was reachable from the remote-site NAS through its local HTTP service.
- The Synology reverse proxy reached the remote-site NAS proxy across Tailscale.
- A WebSocket upgrade to `/ws/controller` succeeded through the reverse-proxy chain.
- CouchOS could reach the Synology IP, but a request without the expected hostname was routed to the wrong reverse-proxy virtual host.
- Supplying the expected HTTP Host header returned the HTP-1 UI through the proxy.
- CouchOS initially had no resolution for the proxy hostname. Adding the hosts-file mapping made `htp1-remote` resolve to the Synology and a ping to that address succeeded.
- The owner then confirmed that the HTP-1 integration connection worked.

## Remaining caveat

Persistence of the manual `/etc/hosts` change across a CouchOS reboot has not been verified. The file was a regular file, and a search of the inspected startup/network configuration did not find a script explicitly rebuilding it. Treat reboot persistence as unconfirmed until checked after a planned reboot or until a supported persistent configuration mechanism is identified.
