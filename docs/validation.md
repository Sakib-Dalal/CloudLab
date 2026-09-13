# Validation performed

Validated on macOS with Rust 1.96.1, Node.js 22, and Docker Desktop 29.6.1 using Linux containers. Two agents were exercised on this one physical test machine.

- Svelte type/accessibility checks: no errors or warnings.
- Production frontend build: passed; fonts are bundled locally.
- Rust formatting and Clippy with warnings denied: passed.
- Twelve Rust tests covering authentication scope, revocation, launch ticket replay/expiry, cross-workspace cookie isolation, HTTPS cookie attributes, origin checks, branded error responses, stopped/offline workspace handling, one-time desktop sign-in, and sandbox policy: passed.
- Real Docker integration: two enrollments; single-use pairing; owner/operator/viewer permissions; cross-lab denial; resource/template validation; non-root commands; no bind mounts or Docker socket; read-only system; CPU/RAM/PID limits; default network mode `none`; stop/resume; persistent volume data; deletion; revocation; and coordinator restart recovery passed.
- JupyterLab: authenticated HTTP gateway, single-use launch ticket, denied foreign WebSocket origin, and successful WebSocket handshake passed.
- code-server: authenticated gateway and the real workbench JavaScript bundle larger than 8 MiB passed through the relay.
- Tauri: macOS application bundle built, launched, and completed local owner bootstrap sign-in.
- Docker Compose configuration: validated.
- Public website: SvelteKit static build and type/accessibility check passed with no errors or warnings. Generated-output validation checked 17 HTML pages, all 13 guides, 1,453 local links/anchors/assets, page metadata, sitemap, 404 behavior in the artifact, and the Vercel output boundary. Local preview returned HTTP 200. No browser interaction/visual audit was performed.

The Docker integration suite removes only the containers, networks, and volumes belonging to its temporary node IDs. Built workspace images are retained for use.

Not verified in this environment: public internet routing/DNS/TLS, two separate physical machines, Windows/Linux desktop installers, hostile multi-tenant resistance, or exhaustive browser compatibility. The coordinator Dockerfile is supplied; only the workspace images were built and exercised here. See the security guide for current resource and protocol limits.


## Workspace launch and interface regression — September 13, 2026

- Reproduced the exact Jupyter access-key error in a browser using the original coordinator. The `SameSite=Strict` workspace cookie was withheld on the redirect from the coordinator. The updated `SameSite=Lax` cookie completes the launch; API sessions remain Strict and workspace mutation/WebSocket origin checks remain enforced.
- Real browser: JupyterLab launched in the CloudLab frame, created a notebook, and executed Python with the expected result (`42`). VS Code loaded its real workbench in the branded frame without the access-key error.
- Real browser: the terminal filled the window, executed a command in `/home/lab`, recalled command history, copied/cleared output, and entered/exited full screen. The 390-pixel layout had no console overflow. CloudLab lab/node/resource labels were visually inspected.
- Real Docker integration (both app templates enabled): all checks passed, including single-use tickets, authenticated workspace metadata, HTTP relay, Jupyter WebSocket origin checks, VS Code’s >8 MiB workbench bundle, container isolation/lifecycle, permissions, revocation, and restart persistence.
- Frontend checks: zero errors/warnings; production build passed. Rust formatting, 12 tests, Clippy with warnings denied, release coordinator build, and desktop compilation passed.
- Browser sessions are still finite: expired/revoked links require reopening from the dashboard. Terminal commands remain buffered and non-interactive, with the existing 30-second limit. Browser-specific fullscreen permissions and public HTTPS deployment were not exercised across all supported platforms.

## DuckDNS option and setup guides — September 13, 2026

- Dashboard and public website: type/accessibility checks passed with zero errors or warnings; both production builds passed. The public artifact contains 18 static pages and 14 guides, with 1,630 local links/assets, metadata, sitemap, and the deployment boundary validated.
- The same structured ten-step guide is bundled in the dashboard and public website. The dashboard includes expandable steps, a platform selector, and copyable commands, so the instructions are available without waiting for a public website deployment.
- Browser preview: inspected the homepage DuckDNS section and numbered guide, checked Windows command selection on both surfaces, expanded the dashboard’s embedded guide, and verified the Copy button put the expected command on the clipboard. No browser console warnings/errors were observed in these checks.
- The six offline updater tests passed: HTTPS request fields, IPv4-only updates, rejected responses, token-safe network errors, private config validation, redirect refusal, invalid encoding, and a helpful failure exit. No DuckDNS account token or live update was used.
- Built Caddy 2.11.4 with the DuckDNS provider and successfully validated `deploy/Caddyfile.duckdns` with placeholder settings. This does not issue a certificate or verify a real deployment.
- The Docker frontend stage built successfully with the shared guide included.

Not performed: DuckDNS account registration, changes to public DNS or router/firewall settings, live ACME certificate issuance, public network testing, or website publication. Follow `docs/duckdns.md` with a registered name and a reachable gateway to complete an actual deployment.
