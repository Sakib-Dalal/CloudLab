# Security and operational model

CloudLab is intended for a personal lab or a small group of trusted collaborators. It is not a hardened hostile multi-tenant execution service. These boundaries are implemented in code; an independent security audit has not been performed.

## Trust boundaries

- The owner administers the coordinator, enrolls trusted devices, sets policy, and creates access keys. Owner-token access is administrative.
- An operator can execute arbitrary code **inside containers in their assigned lab**, create workspaces, and change their lifecycle. Operators share that lab's workspaces and persistent data.
- A viewer can inspect status and activity but cannot create, open, execute, or delete workspaces.
- A node agent is a trusted host process with access to its Docker daemon. Rootful Docker access is effectively host administrative access; prefer a rootless daemon. Never put an agent credential or Docker socket inside a workspace.
- A compromised node may compromise work and availability assigned to that node. It must not be treated as a trusted place for another user's secrets. Node responses are scoped to the authenticated node and pending requests.

## Container policy

`sandbox.rs` constructs all Docker arguments. API callers cannot choose host paths, privileged mode, capabilities, host namespaces, ports, arbitrary devices, or arbitrary images. The optional GPU selector accepts only GPU identifiers discovered on that node. The agent discovers devices again before creation, allows NVIDIA UUID requests or specific Linux render devices (and AMD’s compute device when present), and adds the required device groups without granting privileged mode. The node independently rejects unknown templates, malformed IDs, invalid budgets, and unauthorized outbound networking. User shell strings are passed as a single argument **after** `docker exec … timeout … sh -lc`; they are never evaluated by a host shell.

By default, each workspace uses Docker’s `none` network driver: its private network namespace contains only loopback, with no route to the host, LAN, or internet. The application relay enters the container through a fixed, non-root exec stream. An outbound-enabled workspace gets its own Docker bridge network. Enabling outbound access requires both the coordinator's setting and `--allow-network` on the node. Outbound mode also makes reachable LAN services accessible: there is no per-destination egress firewall. Do not enable it for untrusted lab members.

Docker containers share the Linux kernel, including on a Linux host. On macOS/Windows, Docker Desktop runs the Linux containers in its VM. For adversarial workloads, use a dedicated VM per trust group, a hardened sandbox runtime, and an external network policy. Keep Docker, the host kernel, and workspace images patched.

## Authentication

The owner, agent, enrollment, and collaborator secrets are randomly generated 64-character tokens. Token verifiers are SHA-256 hashes; this is appropriate for high-entropy generated keys, not human passwords. Enrollment keys expire after 10 minutes and are consumed atomically. Collaborator keys expire after 7 days. User-session expiration is checked on every request. Revoking a collaborator key invalidates its sessions; active workspace sockets recheck authorization every 5 seconds.

Coordinator sessions use HttpOnly, SameSite=Strict cookies. Workspace sessions use HttpOnly, SameSite=Lax cookies so a top-level launch from the coordinator can complete across sites. HTTPS sets Secure, and HTTPS workspace cookies use the host-only `__Host-` prefix. Workspace mutations and WebSocket connections still require the exact workspace Origin. API mutations require a custom header and do not permit cross-origin requests. Each workspace has its own origin and scoped application session. One-time launch tickets expire after 60 seconds, are consumed before redirecting to a clean URL, and are never forwarded to containers. Configure reverse proxies not to log launch-ticket query strings. The CloudLab workspace header and application share only that workspace’s origin; framing is restricted to that same origin. The coordinator remains unframeable, and its cookies are never forwarded to containers.

There is a basic global sign-in limit of 30 attempts per minute. It is intentionally simple and may cause shared lockouts under abuse. For public deployments add gateway rate limiting and, if needed, an external identity provider/MFA. MFA, fine-grained per-workspace ACLs, multi-owner accounts, and account recovery are not implemented.

## Resource and lifecycle limits

- Optional GPUs are reserved to a workspace, including while stopped, until removal. Reservations prevent another CloudLab workspace from choosing the same device; they do not isolate it from host applications, enforce a GPU memory quota, or make GPU drivers a secure boundary against hostile code. GPU telemetry is device-wide.
- CPU and memory are reserved across all workspaces assigned to a node, including stopped workspaces. Removal releases the reservation. The agent applies Docker CPU, RAM, swap, PID, and log limits.
- Docker volume disk use has **no quota**. Monitor free disk space and use filesystem/Docker storage quotas externally. Volume data is retained when a container is removed; deletion is an explicit local administrator operation.
- HTTP requests/responses and WebSocket messages are limited to 16 MiB. Use application chunked uploads or transfer files through a controlled local administrator workflow for larger files. HTTP responses are buffered; SSE and indefinite HTTP streaming are not supported. WebSockets are streamed.
- The dashboard console is buffered, non-interactive, capped at 64 KiB output, with a 30-second in-container timeout. Use JupyterLab or code-server's container terminals for interactive work.
- Idle time is based on console/application access. Open application WebSockets keep workspaces active. Background computation alone does not reset the timer; disable idle stopping for unattended jobs. Idle shutdown requires a running coordinator and a connected agent.
- Container lifecycle jobs are persisted and retried after a 15-minute lease; create/stop/start/delete are idempotent for that workspace. Console execution is never replayed after an expired lease because it may have side effects. After a crash its outcome can be unknown.
- Agent receipts persist command outcomes for coordinator reconnects. They are local files and currently require periodic administrator cleanup. The coordinator retains completed jobs for about a day and the most recent 1,000 activity events.
- Revoking a node prevents new work and closes its relay. It does not stop an unreachable node's already-running containers. Stop containers before revocation or clean them up locally.

## Storage and recovery

The coordinator writes JSON metadata through a synchronized, atomic file replacement. Use a single coordinator process per data directory; this is not a distributed database. Unix data directories use mode 0700 and credential/state files mode 0600. On Windows, place the data directory under a private user profile and restrict its ACLs. Backups contain sensitive session verifiers and should be protected.

To rotate the owner key, stop the coordinator, replace `admin-token` with a new 64-character random key, and clear `sessions` and `app_sessions` from the backed-up state before restarting. Existing collaborator and node tokens are independent and must be revoked separately if exposed.

Persistent work lives in `cloudlab-WORKSPACE_ID-home` volumes. To recover a removed workspace, mount that specific volume in a trusted administrative container. CloudLab never automatically mounts a host home directory or imports old host credentials.

References: [Docker Engine security](https://docs.docker.com/engine/security/), [rootless Docker](https://docs.docker.com/engine/security/rootless/), [Docker run resource and security options](https://docs.docker.com/reference/cli/docker/container/run/), and [code-server proxy guidance](https://coder.com/docs/code-server/guide).

## Cloud VM setup

The opt-in cloud installer uses a host compute service with Docker group access on a dedicated Linux VM. Its coordinator container has no Docker socket. Public IP HTTP traffic only redirects to a configured HTTPS dashboard. A startup-managed public URL controls Caddy’s certificate allowlist; changing a saved Settings URL cannot expand it. The `/api/tls/allow` endpoint performs no network requests, is disabled by default, authorizes only the dashboard and registered browser workspace origins, and is blocked by the generated public gateway. Workspace origins and scoped sessions remain separate from the coordinator. See [cloud deployment](cloud-access.md) for DNS dependencies and operational limits.
