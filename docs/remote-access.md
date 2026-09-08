# Reach your own lab from anywhere

CloudLab separates the coordinator from workspace applications. The coordinator needs one HTTPS origin; each workspace needs a different subdomain. All compute nodes dial the coordinator over HTTPS/WSS. There are no inbound node ports to forward, and no Docker daemon should be exposed to the network.

```text
Web / Tauri client ── HTTPS ──┬── lab.example.com → coordinator :8088
                            └── w-ID.workspaces.lab.example.com → gateway :8089
                                                                │
                                  authenticated outbound WSS    │
                            ┌───────────────────────────────────┘
                       Node agents → isolated Docker workspaces
```

## HTTPS on your own hardware

1. Point `lab.example.com` and `*.workspaces.lab.example.com` DNS at your gateway's public address. The `*` is required because workspace IDs are generated dynamically.
2. Put Caddy on the gateway. Start with [`deploy/Caddyfile`](../deploy/Caddyfile), replacing both domain names. The coordinator site can use automatic TLS. Obtain a wildcard certificate for `*.workspaces.lab.example.com` with your DNS provider's DNS-01 challenge integration and install it at the paths in the example. Arrange certificate renewal. A Caddy build with your DNS-provider plugin can manage the wildcard certificate instead.
3. Forward TCP 443 to Caddy. Allow TCP 80 as needed for the coordinator certificate's ACME HTTP challenge. Keep 8088 and 8089 restricted to loopback or the trusted proxy network. Keep the gateway's access logging from recording `ticket` query values; the provided configuration does not enable access logging.
4. Start the coordinator with the workspace origin template:

   ```sh
   CLOUDLAB_APP_URL='https://{workspace}.workspaces.lab.example.com' \
     cloudlab serve --web-dir /opt/cloudlab/web --data-dir /var/lib/cloudlab
   ```

5. Sign in through `https://lab.example.com`. Save that address as **Public coordinator URL** in Lab settings. It updates newly generated pairing commands; it does not create DNS, TLS certificates, or port forwarding.
6. Enroll nodes using the HTTPS address. Existing nodes have a pinned coordinator URL in their credential file; changing that URL requires updating their local configuration/re-enrollment deliberately. Do not copy the same agent credential directory across devices.

The origin template is a server launch setting, separate from the public coordinator URL. **Never reverse-proxy workspace apps onto the dashboard's origin or combine different workspace subdomains.** Caddy must preserve the original `Host` and WebSocket upgrade headers. Both HTTP and WebSocket traffic are authenticated by CloudLab.

## A private VPN

Run WireGuard between your clients, gateway, and coordinator network. Resolve the same coordinator and wildcard workspace hostnames to the gateway's VPN address. Use HTTPS certificates issued with DNS-01; neither the coordinator nor its gateway needs to accept public browser traffic in this topology. Agents may connect over that VPN to the same HTTPS origin.

CloudLab requires HTTPS for non-loopback agent connections even on a VPN. It does not disable certificate validation or accept self-signed certificates by default. Use a publicly trusted certificate or add your own trust root to the operating system and build a client configured for that trust policy.

## NAT and CGNAT

A reachable route is still necessary. A public address with port forwarding is enough for a home-hosted gateway. If your ISP uses CGNAT, place a WireGuard gateway on another machine you control that has a public address and route to your local coordinator through it. You may need rented infrastructure if you do not own any reachable machine. CloudLab does not yet implement RustDesk-style NAT punching or an arbitrary TCP relay.

## Local workspace addresses

The default gateway template is `http://{workspace}.localhost:8089`. Browsers that implement the `.localhost` special-use domain resolve these addresses to loopback. If your browser or system does not, add the exact workspace hostnames to local DNS/hosts, or use your own wildcard DNS and HTTPS gateway. Access from a different computer requires the HTTPS setup above; `localhost` always refers to the client computer.
