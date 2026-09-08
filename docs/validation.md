# Validation performed

Validated on macOS with Rust 1.96.1, Node.js 22, and Docker Desktop 29.6.1 using Linux containers. Two agents were exercised on this one physical test machine.

- Svelte type/accessibility checks: no errors or warnings.
- Production frontend build: passed; fonts are bundled locally.
- Rust formatting and Clippy with warnings denied: passed.
- Seven Rust tests covering authentication scope, revocation, origin validation, header filtering, one-time desktop sign-in, and sandbox policy: passed.
- Real Docker integration: two enrollments; single-use pairing; owner/operator/viewer permissions; cross-lab denial; resource/template validation; non-root commands; no bind mounts or Docker socket; read-only system; CPU/RAM/PID limits; default network mode `none`; stop/resume; persistent volume data; deletion; revocation; and coordinator restart recovery passed.
- JupyterLab: authenticated HTTP gateway, single-use launch ticket, denied foreign WebSocket origin, and successful WebSocket handshake passed.
- code-server: authenticated gateway and the real workbench JavaScript bundle larger than 8 MiB passed through the relay.
- Tauri: macOS application bundle built, launched, and completed local owner bootstrap sign-in.
- Docker Compose configuration: validated.
- Public website: SvelteKit static build and type/accessibility check passed with no errors or warnings. Generated-output validation checked 17 HTML pages, all 13 guides, 1,453 local links/anchors/assets, page metadata, sitemap, 404 behavior in the artifact, and the Vercel output boundary. Local preview returned HTTP 200. No browser interaction/visual audit was performed.

The Docker integration suite removes only the containers, networks, and volumes belonging to its temporary node IDs. Built workspace images are retained for use.

Not verified in this environment: public internet routing/DNS/TLS, two separate physical machines, Windows/Linux desktop installers, hostile multi-tenant resistance, or full browser interaction/visual QA. The coordinator Dockerfile is supplied; only the workspace images were built and exercised here. See the security guide for current resource and protocol limits.
