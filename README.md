# CloudLab 2.0

A personal compute lab, built with **Rust, Svelte 5, and Tauri 2**. Connect multiple computers to a lab and create isolated JupyterLab, code-server, or Linux console workspaces on them. The coordinator, agents, credentials, project files, and browser assets run on hardware you control.

The interface uses `#BF4646`, `#EDDCC6`, `#FFF4EA`, and `#7EACB5`, with locally bundled fonts. It includes lab switching, live node health, workspace search, resource allocation, scoped collaborator access, activity history, and configurable session/idle limits. `?demo=1` opens a clearly labeled, in-memory preview; demo devices never claim to be connected hardware.

## Public website and documentation

The public SvelteKit website lives in [`website/`](website/README.md), separately from the private lab dashboard. It includes the product overview, an interactive lab diagram, searchable documentation, 15 step-by-step guides, copyable commands, FAQs, security details, and project information.

```sh
npm ci --prefix website
npm run website:dev
```

Open **http://127.0.0.1:4173**. Root `vercel.json` publishes only `website/build/`, with the repository root selected in Vercel. Connect the GitHub repository to Vercel once: pushes to `main` deploy production and branch changes receive previews according to project settings. Website checks run both during deployment and in GitHub Actions. See the [deployment instructions](website/README.md#vercel--github). Vercel hosts the public guide; your Rust coordinator and compute nodes remain self-hosted.

## Host on AWS EC2 or another cloud VM

Open **Settings → Set up cloud access** for a visual guide. CloudLab can detect a VM’s public IP, configure an HTTPS dashboard and isolated workspace addresses, and pair the VM as a compute node automatically. AWS, Google Cloud, Azure, and DigitalOcean support metadata detection; other providers work with a supplied public/static IPv4.

See [cloud VM setup](docs/cloud-access.md) for prerequisites, a provider-specific setup command, firewall rules, and updates. The VM hosts both the dashboard and workspaces. No cloud account keys or domain purchase are needed for the IP-based path.

## Start a local lab

Requirements: Node.js 22.12+, a current stable Rust toolchain, and Docker Engine or Docker Desktop in Linux-container mode on each compute node. A coordinator does not need Docker.

```sh
npm ci
npm run build
cargo build --release --locked -p cloudlab
./target/release/cloudlab serve
```

Open **http://127.0.0.1:8088**. Read the owner access key from `.cloudlab/admin-token` on that computer and enter it in the sign-in form. The key is generated on first start and is never printed in service logs. Data is stored in `.cloudlab/state.json`; back up this directory while the coordinator is stopped.

To stop the standalone server, run this in another terminal from the same directory:

```sh
./target/release/cloudlab stop
```

If installed on your PATH, use `cloudlab stop`. Pass the same `--bind` and `--data-dir` as `serve` when using custom values (or set `CLOUDLAB_BIND` and `CLOUDLAB_DATA_DIR`). The command reads the local owner key and shuts down the coordinator and workspace gateway within five seconds; agents and compute containers remain running. For a service-managed installation, use its service manager, such as `sudo systemctl stop cloudlab` or `docker compose stop coordinator`, so it stays stopped.

When upgrading from a version without `stop`, stop the existing `serve` process once with Ctrl+C in its terminal, then start `serve` with the updated binary. Rebuilding alone does not update an already-running server; it will return `405 Method Not Allowed` until restarted. Close the desktop app to stop a coordinator it owns.

On each compute device, build the trusted workspace images and install the agent:

```sh
./scripts/build-images.sh
cargo install --locked --path crates/cloudlab
```

In **Compute nodes → Connect a node**, generate a one-time pairing command. Run it on the target device. For a device on the same computer as the coordinator it looks like:

```sh
cloudlab agent --coordinator http://127.0.0.1:8088 --enrollment YOUR_ONE_TIME_KEY
```

For other computers, configure an HTTPS coordinator address first (see [remote access](docs/remote-access.md)). Enrollment expires in 10 minutes and can only be used once. Afterwards, use `cloudlab agent start` with the same data directory to resume the saved pairing. The original `cloudlab agent --coordinator URL` syntax also works **without** `--enrollment`. One agent process represents one enrolled compute node; use one agent per physical device in normal operation.

Create a workspace, choose its node and resource budget, and open it. The node must already have the selected `cloudlab/terminal:2`, `cloudlab/jupyter:2`, or `cloudlab/code:2` image. There is no automatic arbitrary-image pull and no host shell endpoint.

### CLI commands

Run `cloudlab` or `cloudlab --help` for the CloudLab ASCII logo, command overview, and examples. Running `cloudlab` with no arguments shows help without starting services. The logo also appears when starting a coordinator or agent in an interactive terminal; redirected service logs stay plain. The [CLI guide](https://cloudlab-alpha.vercel.app/docs/cli/) includes command examples, data folders, environment variables, and pairing recovery.

| Command | What it does |
| --- | --- |
| `cloudlab serve` | Start the dashboard and workspace gateway |
| `cloudlab stop` | Stop the coordinator; agents and containers keep running |
| `cloudlab agent start` | Start using the saved pairing and coordinator address |
| `cloudlab agent status` | Show local process and pairing status without displaying keys |
| `cloudlab agent status --json` | Return local status as JSON for scripts, without credentials |
| `cloudlab agent stop` | Stop the agent; keep its pairing, containers, and volumes |
| `cloudlab agent delete` | Stop the agent, revoke its node access, and remove local pairing and operation receipts |
| `cloudlab agent delete --local-only` | Clear local pairing without contacting the coordinator; revoke the old node in the dashboard separately |

Agent commands use `.cloudlab/agent` relative to the current directory. Use the original `--data-dir` (or `CLOUDLAB_AGENT_DIR`) for every command, for example `cloudlab agent stop --data-dir /var/lib/cloudlab-agent`. Deleting agent enrollment never deletes Docker containers or volumes. Remove workspaces before deleting an active node; afterward, workspaces on that revoked node offer **Remove workspace record** in the dashboard, with Docker cleanup performed locally.

If you see **This agent is already enrolled**, omit `--enrollment` to resume it. To replace the pairing, run `cloudlab agent delete`, generate a fresh key in **Compute nodes → Connect a node**, and run that new pairing command. If the coordinator is unreachable, normal deletion keeps the credentials so you can retry; `--local-only` deliberately skips remote revocation. The delete command also works when the old node has already been revoked.

Install the updated CLI with `cargo install --locked --path crates/cloudlab --force`, and restart the coordinator with the updated binary for agent deletion support. An agent started with an older binary must be stopped once in its original terminal with Ctrl+C or through its service manager. For a service-managed agent, stop its service first so an automatic restart does not undo your stop.

### Development

Run `cargo run -p cloudlab -- serve` in one terminal and `npm run dev` in another. Vite listens at **http://127.0.0.1:5173** and proxies API calls to port 8088. Production uses Rust to serve the compiled `dist/` directory.

### Desktop app

```sh
npm run desktop           # build frontend and run Tauri in development
npm run desktop:build     # package for the current platform
```

Tauri starts an embedded local coordinator and workspace gateway on loopback ports 8088/8089 and signs the local owner in using an expiring, single-use bootstrap credential. It stores data in the platform application-data directory for `dev.cloudlab.desktop`. If a standalone CloudLab coordinator already runs on port 8088, the desktop app connects to it instead and uses its normal sign-in. Closing the desktop application stops a coordinator it owns; compute containers remain on their nodes. Stable ports let agents reconnect after a desktop restart. For an always-on lab, run the coordinator as a service and use the desktop app as a remote client:

```sh
CLOUDLAB_DESKTOP_URL=https://lab.example.com npm run desktop
```

You can also choose **Connect to another coordinator** on the sign-in screen. Native builds need the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/). There are no shell, filesystem, or process IPC permissions granted to workspace webviews. Platform installers are unsigned until you configure your own signing/notarization credentials.

### Coordinator in Docker

```sh
docker compose up --build -d
docker compose exec coordinator cat /data/admin-token
```

The coordinator has no Docker socket. Compute agents run separately on their devices and are the only component that talks to Docker. The Compose ports are bound to loopback, ready for a local HTTPS reverse proxy. Set `CLOUDLAB_APP_URL` for remote use. The coordinator's data volume must remain writable.

## What is included

| Area | Behavior |
| --- | --- |
| Labs | Multiple labs with separate nodes, workspaces, activity, and access keys |
| Nodes | One-time enrollment, independent credentials, heartbeats, CPU/RAM/network and GPU telemetry, revocation, reconnecting outbound relay |
| Workspaces | Fixed templates, CPU/RAM/GPU reservations, create/stop/resume/remove, dedicated persistent volumes |
| Tools | JupyterLab and code-server over authenticated HTTP/WebSockets; buffered Linux console commands |
| Access | Owner administration, lab-scoped operator/viewer keys, 7-day key expiry, session expiry and revocation |
| Settings | Coordinator name/URL, default CPU/RAM, lab workspace cap, idle shutdown, session duration, network policy |
| Persistence | Atomic local metadata writes, persisted operation receipts, bounded activity history, restart recovery |
| Clients | Responsive Svelte web app and Tauri desktop shell |

**Open workspace** launches JupyterLab and VS Code in a CloudLab window with lab/node details, CPU and memory allocations, connection status, and a full-screen control. The Linux console fills the dashboard window and includes command history, copy/clear output, and full-screen controls. Browser sessions reconnect through the dashboard; an expired link never requires entering the lab key into the container. Existing workspace images do not need rebuilding for the workspace interface.

**Operators share all workspaces in their assigned lab.** Viewers can only inspect status and activity. Use separate labs for separate trust groups. Lab settings are coordinator-wide defaults and policies, not per-user quotas.

## Analytics and GPU workspaces

**Compute nodes** and **Workspaces** include Grafana-style visual dashboards: CPU and memory trends, network throughput, container disk I/O, status summaries, node reservations, GPU utilization, and available GPU memory/temperature/power sensors. Filter by node or workspace, select the last 5 minutes, 15 minutes, or hour, pause the display, inspect points with the time slider, and toggle GPU charts. Open consoles, JupyterLab, and VS Code also show live usage with expandable charts.

Agents sample in the background; the coordinator holds up to one hour of history in memory, resetting on restart. Missing or stale readings are shown as unavailable. New workspace creation offers detected GPUs that support container access: NVIDIA on Linux/WSL2 with a configured GPU runtime, and AMD/Intel render devices on a local Linux Docker engine. Apple and other Mac GPUs are detected for monitoring; ordinary Docker Desktop Linux containers cannot use Metal GPUs. Driver and image requirements, sensor limitations, and reservation behavior are described in [analytics and GPU support](docs/analytics.md).

## Isolation and remote connectivity

Each workspace runs as UID/GID 1000 with a read-only root filesystem, all Linux capabilities dropped, no-new-privileges, a PID cap, CPU and hard memory/swap limits, restricted temporary filesystems, a private network namespace with only loopback by default, and a dedicated Docker volume. There are no host directories, host PID/network namespaces, USB devices, or Docker socket mounts exposed through the API. Optional GPU access is limited to devices discovered by the node; see [analytics and GPUs](docs/analytics.md).

Each browser workspace has its **own origin**, such as `http://w-ID.localhost:8089`, with a separate scoped HttpOnly cookie. The dashboard remains on another origin. Workspace credentials never reach containers. The node verifies Docker ownership labels, then relays to a fixed application port inside the container over an unprivileged Docker exec stream. No workspace ports are published on the host. The gateway supports HTTP and WebSocket traffic; it does not provide arbitrary TCP or host desktop access.

Global access can use a free DuckDNS name with automatic IP updates, your own HTTPS gateway/domain or an HTTPS gateway reachable over WireGuard. All nodes connect outward. There is no third-party rendezvous account, automatic public tunnel, or hosted control plane. Internet access still requires a routable coordinator, port forwarding, or a self-hosted gateway when behind CGNAT. Follow [the remote access guide](docs/remote-access.md).

Read [security and operational limits](docs/security.md) before inviting people. Docker containers share the host kernel; they are not virtual machines or a complete boundary against hostile code. Prefer rootless Docker and a dedicated Linux VM for untrusted workloads.


### Free remote address with DuckDNS

Use DuckDNS as an alternative to a purchased domain and registrar DNS. One registered name gives you `lab.NAME.duckdns.org` for the dashboard and `w-ID.NAME.duckdns.org` for workspace apps. Caddy handles a single wildcard HTTPS certificate and renewal; the included updater keeps the gateway’s public IPv4 current.

Follow the **[ten-step DuckDNS setup](docs/duckdns.md)** for registration, private token storage, automatic IP updates, the Caddy plugin, HTTPS, router forwarding, node pairing, and troubleshooting. Use [`deploy/Caddyfile.duckdns`](deploy/Caddyfile.duckdns) with [`deploy/duckdns.env.example`](deploy/duckdns.env.example). DuckDNS does not bypass CGNAT; a reachable public gateway or VPN route is still required.

## Checks

```sh
npm run check
npm run test:metrics
npm run build
python3 tests/duckdns.py
cargo fmt --check
cargo clippy --locked -p cloudlab --all-targets -- -D warnings
cargo test --locked -p cloudlab
cargo build --locked -p cloudlab
./scripts/build-images.sh
CLOUDLAB_TEST_APPS=1 python3 tests/integration.py
cargo check --locked -p cloudlab-desktop
```

The integration suite runs a real coordinator and two agents on the test machine, creates real Docker containers, checks authorization and sandbox flags, executes non-root commands, verifies storage across restarts, and cleans up only its own labeled resources. The optional app checks exercise Jupyter and code-server through the authenticated gateway, including WebSockets. They do not simulate two physical machines or certify access through a public network.

## Migration from 1.x

The old Go/Python implementation and launch scripts are archived under [`legacy/`](legacy/NOTICE.md). They are not included in the new build. Stop old host services and public tunnels before migration. Existing notebooks and project files are not moved automatically: copy only the files you want into a new workspace. Host Python environment/kernel management, host SSH, email delivery of credentials, and automatic public tunnels are replaced by container-scoped tools and explicit authenticated connectivity.

The project takes interaction cues from [RustDesk's self-hosted connectivity](https://rustdesk.com/docs/en/self-host/) and [Raspberry Pi Connect's device access](https://www.raspberrypi.com/documentation/services/connect.html). It implements its own container workspace protocol, not either product's remote-desktop protocol.

MIT License. Original project by Sakib Dalal.
