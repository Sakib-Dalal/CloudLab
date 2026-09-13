# Remote access with DuckDNS

DuckDNS is CloudLab’s recommended free DNS option for a home lab with a changing public IPv4 address. You do not need to buy a domain or manage records at a registrar. DuckDNS supplies the name; your own Caddy gateway handles HTTPS and routes traffic to CloudLab. Your normal internet connection is still required.

This guide uses one registered name, **my-cloudlab**. Replace it everywhere with a name you actually register:

| Purpose | Address |
| --- | --- |
| Registered DuckDNS name | `my-cloudlab.duckdns.org` |
| Dashboard and node enrollment | `https://lab.my-cloudlab.duckdns.org` |
| Workspace origin template | `https://{workspace}.my-cloudlab.duckdns.org` |
| Example workspace | `https://w-ID.my-cloudlab.duckdns.org` |
| Certificate | `*.my-cloudlab.duckdns.org` |

Use **lab.my-cloudlab.duckdns.org** for the dashboard, not the bare registered name. Keeping the dashboard and workspaces one level below the registered name lets one wildcard certificate cover both. The repository includes [`deploy/Caddyfile.duckdns`](../deploy/Caddyfile.duckdns), a [config template](../deploy/duckdns.env.example), and a [small IP updater](../scripts/duckdns-update.py).

## 1. Check your connection and prepare the gateway

Start with a working [local CloudLab](../README.md#start-a-local-lab). Run these commands from the CloudLab repository on the computer that runs the coordinator. Caddy must run on that same computer for this configuration; it proxies to `127.0.0.1:8088` and `127.0.0.1:8089`.

Install Python 3 for the updater and a current Go toolchain if you build Caddy below. Alternatively, download a custom Caddy build with the `github.com/caddy-dns/duckdns` plugin from [Caddy’s download page](https://caddyserver.com/download).

In your router, reserve a fixed LAN address for this gateway. Compare the router’s WAN IPv4 address with your public IPv4 address. A private WAN address, an address in `100.64.0.0/10`, or a different upstream public address may indicate double NAT or CGNAT. Ask your ISP for a reachable public IPv4 address, forward through both routers if you control both, or use the [VPN/gateway route](remote-access.md#nat-and-cgnat).

**DuckDNS does not bypass CGNAT, open ports, or create a tunnel.** If your gateway cannot receive inbound connections, resolve that before using the public-access steps below.

## 2. Register a name

1. Open [DuckDNS](https://www.duckdns.org/) and sign in using an available sign-in provider.
2. Add an available name, for example `my-cloudlab`. Register just this name; do not register `lab`, `w-ID`, or `*` separately.
3. Find your DuckDNS account token at the top of its account page. You will put it in a private local file in the next step. It is separate from every CloudLab owner, collaborator, and enrollment key.
4. Check the IPv4 shown for your name. The updater will keep it aligned with the gateway’s public address. DuckDNS sub-subdomains resolve under the registered name; verify both the dashboard and a sample workspace in step 4.

## 3. Save your DuckDNS settings privately

On Linux or macOS:

```sh
mkdir -p .cloudlab
chmod 700 .cloudlab
cp -n deploy/duckdns.env.example .cloudlab/duckdns.env
chmod 600 .cloudlab/duckdns.env
```

On Windows PowerShell:

```powershell
New-Item -ItemType Directory -Force .cloudlab | Out-Null
if (-not (Test-Path .cloudlab/duckdns.env)) {
  Copy-Item deploy/duckdns.env.example .cloudlab/duckdns.env
}
notepad .cloudlab/duckdns.env
```

Open `.cloudlab/duckdns.env` in your text editor and replace both values:

```text
DUCKDNS_SUBDOMAIN=my-cloudlab
DUCKDNS_API_TOKEN=REPLACE_WITH_YOUR_DUCKDNS_TOKEN
```

Use plain, unquoted values. `DUCKDNS_SUBDOMAIN` is only the registered name, without `.duckdns.org`. On Windows keep the file in your private user profile and use Properties → Security to restrict access to your account and necessary administrators. The `.cloudlab` directory is excluded from Git. Do not paste the token into the website, a CloudLab settings field, shell history, or a support message.

## 4. Update the address and verify DNS

Run the updater on the gateway’s network, not on a traveling laptop, compute node elsewhere, or a machine routed through an unrelated VPN:

```sh
python3 scripts/duckdns-update.py --config .cloudlab/duckdns.env
```

On Windows, use `py -3` in place of `python3`. Success prints `OK: DuckDNS IPv4 address updated.` Failure returns a nonzero exit code with a short explanation, without printing the token. The update uses certificate-verified HTTPS and leaves certificate TXT records alone.

Check the dashboard and an arbitrary workspace name:

```sh
nslookup lab.my-cloudlab.duckdns.org
nslookup w-test.my-cloudlab.duckdns.org
```

Both should resolve to the gateway’s public IPv4 address. `w-test` is only a DNS check; it is not a real workspace. Wait for cached DNS answers to expire after a change.

This helper updates IPv4 only. If DuckDNS already has an IPv6/AAAA value, remove it in your DuckDNS settings unless that IPv6 route and firewall also reach this gateway. An old AAAA record can make the site fail even when IPv4 is correct. See the [DuckDNS API specification](https://www.duckdns.org/spec.jsp).

## 5. Keep the address up to date

Choose **one** updater:

- If your router supports DuckDNS over HTTPS, its built-in updater is the option with the least extra software. Enter the registered name and account token in its DDNS settings, enable updates, and check its status. Use [DuckDNS’s router instructions](https://www.duckdns.org/install.jsp) for your model.
- Otherwise, keep the included helper running. It updates immediately and every five minutes, waits between attempts, and rereads the private file so a changed token takes effect on the next cycle:

  ```sh
  python3 scripts/duckdns-update.py --config .cloudlab/duckdns.env --watch
  ```

For updates after reboot, schedule the **one-shot** command, without `--watch`:

**Linux/macOS:** run `command -v python3` to find the interpreter. Run `crontab -e` and add the line below, replacing all three absolute paths with yours. Do not delete existing entries.

```cron
*/5 * * * * /ABS/PATH/python3 /ABS/PATH/CloudLab/scripts/duckdns-update.py --config /ABS/PATH/CloudLab/.cloudlab/duckdns.env >/dev/null
```

**Windows:** open Task Scheduler → Create Task. Use your gateway account, add an “At startup” trigger, and enable repetition every 5 minutes indefinitely. Under Actions choose the full path to `python.exe`; Arguments are `"C:\PATH\CloudLab\scripts\duckdns-update.py" --config "C:\PATH\CloudLab\.cloudlab\duckdns.env"`. Run the task once and check Last Run Result is `0x0`. Configure it to run when you are signed out if the machine must stay reachable without a logged-in session.

Keep the gateway awake. Do not run several competing updaters for the same name from different networks.

## 6. Build Caddy with DuckDNS support

A standard Caddy binary does not include every DNS provider. On Linux/macOS:

```sh
go run github.com/caddyserver/xcaddy/cmd/xcaddy@latest build --with github.com/caddy-dns/duckdns --output .cloudlab/caddy
.cloudlab/caddy list-modules
```

On Windows PowerShell:

```powershell
go run github.com/caddyserver/xcaddy/cmd/xcaddy@latest build --with github.com/caddy-dns/duckdns --output .cloudlab/caddy.exe
.cloudlab/caddy.exe list-modules
```

Confirm `dns.providers.duckdns` appears. This is a one-time build; do not rebuild every five minutes. You can also use the custom download mentioned in step 1. See [Caddy’s build instructions](https://caddyserver.com/docs/build) and the [DuckDNS module](https://github.com/caddy-dns/duckdns).

## 7. Validate the HTTPS gateway

The supplied Caddyfile reads the registered name and token from the private environment file. It obtains and renews **one wildcard certificate** with DNS-01, sends the `lab.` hostname to the coordinator, and sends workspace hosts to the gateway. It preserves Host and WebSocket upgrades and does not enable request access logs.

```sh
.cloudlab/caddy validate --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env
```

On Windows use `.cloudlab/caddy.exe`. Validation should report `Valid configuration`; it does not prove that public DNS, port forwarding, or certificate issuance works yet.

DuckDNS shares one TXT value across the registered name’s subdomains. Keep the single wildcard certificate in the example; do not add a separate certificate for each workspace or run another ACME client against the same DuckDNS name concurrently. The wildcard does not cover the bare `my-cloudlab.duckdns.org` address or extra levels such as `w-ID.workspaces.my-cloudlab.duckdns.org`.

## 8. Set the workspace address and start HTTPS

Stop the existing coordinator process or service cleanly, then restart it with **the same data directory** and the DuckDNS workspace origin. From the repository, a local CLI installation uses:

```sh
cloudlab serve --web-dir dist --data-dir .cloudlab --app-url 'https://{workspace}.my-cloudlab.duckdns.org'
```

Keep `{workspace}` literally in the template. CloudLab substitutes `w-` plus the workspace ID. If you already run CloudLab as a service, update its `CLOUDLAB_APP_URL` or `--app-url` setting instead, preserving its web and data paths. Do not start a second coordinator on the same ports or reuse another deployment’s metadata directory.

On Linux, allow your newly built Caddy binary to bind port 443 before running it as your normal user:

```sh
sudo setcap cap_net_bind_service=+ep .cloudlab/caddy
```

This requires your distribution’s `setcap` utility and must be repeated after replacing the binary. If you use the packaged Caddy service, follow its [custom-binary instructions](https://caddyserver.com/docs/build#package-support-files-for-custom-builds-for-debianubunturaspbian) instead.

In another terminal, start the proxy:

```sh
.cloudlab/caddy run --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env
```

On Windows use `.cloudlab/caddy.exe`. Keep this process running; Caddy manages certificate renewal while running, and its certificate storage must remain persistent and writable. For unattended operation, configure Caddy and CloudLab as [services that start on boot](https://caddyserver.com/docs/running). On Linux a user service can use the exact Caddy command above with absolute paths and `WorkingDirectory` set to your repository; enable lingering for that user if it must run after logout. Updating your IP and running the HTTPS gateway are separate jobs.

## 9. Forward the router and firewall

1. Forward **WAN TCP 443 → the gateway’s reserved LAN address, TCP 443**.
2. Allow inbound TCP 443 through that computer’s firewall.
3. Leave CloudLab ports 8088/8089 on loopback. Never forward the Docker socket or Docker API.
4. The example disables automatic HTTP redirects and uses DNS-01, so you do not need inbound TCP 80. Always type the `https://` address.
5. Let Caddy reach DuckDNS and the certificate authority over outbound HTTPS and perform DNS lookups.

If another reverse proxy already owns port 443, merge this site into that proxy’s Caddy configuration instead of starting a second listener.

## 10. Sign in, pair nodes, and test outside your Wi-Fi

1. Open `https://lab.my-cloudlab.duckdns.org` using mobile data or a different internet connection. Sign in with your existing CloudLab key.
2. In **Lab settings → Public coordinator URL**, save that exact HTTPS address. This affects new pairing commands; it does not change `CLOUDLAB_APP_URL`, DNS, or Caddy.
3. Generate a pairing command in **Compute nodes → Connect a node** and run it on the target device. Existing nodes can continue using an unchanged reachable address. When changing an enrolled node’s URL, update its local configuration deliberately and keep its own credentials/data directory.
4. Open a Jupyter workspace and run a notebook cell. Open VS Code and its terminal. Confirm the browser uses `w-ID.my-cloudlab.duckdns.org`, a valid certificate, and the CloudLab workspace header.
5. If it works over mobile data but not home Wi-Fi, check the router’s NAT loopback/hairpin support. A local DNS override is an option only if it resolves both dashboard and workspace names to the gateway.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Updater says rejected / KO | Use the DuckDNS account token, correct registered name, and plain unquoted config values. |
| Names resolve to the wrong IP | Run the updater on the gateway’s network; disable a competing updater or unrelated VPN. |
| DNS works but HTTPS times out | Check WAN/public-IP reachability, CGNAT, TCP 443 forwarding, gateway sleep, and firewalls. |
| Caddy says module not registered | Run the custom binary from step 6, not an older standard `caddy` on PATH. |
| Caddy says permission denied on 443 | Apply the Linux capability or use the correctly configured Caddy service. |
| Certificate challenge fails | Check the token, DNS propagation, Caddy logs, and another ACME client overwriting DuckDNS’s shared TXT record. |
| Dashboard works but workspace fails | Keep the exact one-level workspace template and gateway route to 8089; reopen from the dashboard for a fresh ticket. |
| Certificate warning on the bare name | Use `https://lab.NAME.duckdns.org`; the wildcard covers that name, not `NAME.duckdns.org`. |

[DuckDNS service](https://www.duckdns.org/) · [API specification](https://www.duckdns.org/spec.jsp) · [Caddy DuckDNS provider](https://github.com/caddy-dns/duckdns) · [Automatic HTTPS](https://caddyserver.com/docs/automatic-https)
