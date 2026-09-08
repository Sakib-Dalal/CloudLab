# CloudLab 2.0

A personal compute lab, built with **Rust, Svelte 5, and Tauri 2**. Connect multiple computers to a lab and create isolated JupyterLab, code-server, or Linux console workspaces on them. The coordinator, agents, credentials, project files, and browser assets run on hardware you control.

The interface uses `#BF4646`, `#EDDCC6`, `#FFF4EA`, and `#7EACB5`, with locally bundled fonts. It includes lab switching, live node health, workspace search, resource allocation, scoped collaborator access, activity history, and configurable session/idle limits. `?demo=1` opens a clearly labeled, in-memory preview; demo devices never claim to be connected hardware.

## Public website and documentation

The public SvelteKit website lives in [`website/`](website/README.md), separately from the private lab dashboard. It includes the product overview, an interactive lab diagram, searchable documentation, 13 step-by-step guides, copyable commands, FAQs, security details, and project information.

```sh
npm ci --prefix website
npm run website:dev
```

Open **http://127.0.0.1:4173**. Root `vercel.json` publishes only `website/build/`, with the repository root selected in Vercel. Connect the GitHub repository to Vercel once: pushes to `main` deploy production and branch changes receive previews according to project settings. Website checks run both during deployment and in GitHub Actions. See the [deployment instructions](website/README.md#vercel--github). Vercel hosts the public guide; your Rust coordinator and compute nodes remain self-hosted.

## Start a local lab

Requirements: Node.js 22.12+, a current stable Rust toolchain, and Docker Engine or Docker Desktop in Linux-container mode on each compute node. A coordinator does not need Docker.

```sh
npm ci
npm run build
cargo build --release --locked -p cloudlab
./target/release/cloudlab serve
```

Open **http://127.0.0.1:8088**. Read the owner access key from `.cloudlab/admin-token` on that computer and enter it in the sign-in form. The key is generated on first start and is never printed in service logs. Data is stored in `.cloudlab/state.json`; back up this directory while the coordinator is stopped.

On each compute device, build the trusted workspace images and install the agent:

```sh
./scripts/build-images.sh
cargo install --locked --path crates/cloudlab
```

In **Compute nodes → Connect a node**, generate a one-time pairing command. Run it on the target device. For a device on the same computer as the coordinator it looks like:

```sh
cloudlab agent --coordinator http://127.0.0.1:8088 --enrollment YOUR_ONE_TIME_KEY
```

For other computers, configure an HTTPS coordinator address first (see [remote access](docs/remote-access.md)). Enrollment expires in 10 minutes and can only be used once. Afterwards, restart the agent with the same coordinator and data directory, **without** `--enrollment`. One agent process represents one enrolled compute node; use one agent per physical device in normal operation.

Create a workspace, choose its node and resource budget, and open it. The node must already have the selected `cloudlab/terminal:2`, `cloudlab/jupyter:2`, or `cloudlab/code:2` image. There is no automatic arbitrary-image pull and no host shell endpoint.

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
| Nodes | One-time enrollment, independent credentials, heartbeats, CPU/RAM telemetry, revocation, reconnecting outbound relay |
| Workspaces | Fixed templates, CPU/RAM reservations, create/stop/resume/remove, dedicated persistent volumes |
| Tools | JupyterLab and code-server over authenticated HTTP/WebSockets; buffered Linux console commands |
| Access | Owner administration, lab-scoped operator/viewer keys, 7-day key expiry, session expiry and revocation |
| Settings | Coordinator name/URL, default CPU/RAM, lab workspace cap, idle shutdown, session duration, network policy |
| Persistence | Atomic local metadata writes, persisted operation receipts, bounded activity history, restart recovery |
| Clients | Responsive Svelte web app and Tauri desktop shell |

**Operators share all workspaces in their assigned lab.** Viewers can only inspect status and activity. Use separate labs for separate trust groups. Lab settings are coordinator-wide defaults and policies, not per-user quotas.

## Isolation and remote connectivity

Each workspace runs as UID/GID 1000 with a read-only root filesystem, all Linux capabilities dropped, no-new-privileges, a PID cap, CPU and hard memory/swap limits, restricted temporary filesystems, a private network namespace with only loopback by default, and a dedicated Docker volume. There are no host directories, host PID/network namespaces, GPUs, USB devices, or Docker socket mounts exposed through the API.

Each browser workspace has its **own origin**, such as `http://w-ID.localhost:8089`, with a separate scoped HttpOnly cookie. The dashboard remains on another origin. Workspace credentials never reach containers. The node verifies Docker ownership labels, then relays to a fixed application port inside the container over an unprivileged Docker exec stream. No workspace ports are published on the host. The gateway supports HTTP and WebSocket traffic; it does not provide arbitrary TCP or host desktop access.

Global access uses your own HTTPS gateway/domain or an HTTPS gateway reachable over WireGuard. All nodes connect outward. There is no third-party rendezvous account, automatic public tunnel, or hosted control plane. Internet access still requires a routable coordinator, port forwarding, or a self-hosted gateway when behind CGNAT. Follow [the remote access guide](docs/remote-access.md).

Read [security and operational limits](docs/security.md) before inviting people. Docker containers share the host kernel; they are not virtual machines or a complete boundary against hostile code. Prefer rootless Docker and a dedicated Linux VM for untrusted workloads.

## Checks

```sh
npm run check
npm run build
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
