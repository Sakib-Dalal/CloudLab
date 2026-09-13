import { duckdnsGuide } from "../../../shared/duckdns-guide";
import type { Block, Guide } from "../../../shared/guide-types";
export type { Block, Guide } from "../../../shared/guide-types";

const text = (text: string): Block => ({ type: "text", text });
const list = (...items: string[]): Block => ({ type: "list", items });
const note = (title: string, text: string): Block => ({
  type: "note",
  title,
  text,
});
const code = (code: string, label = "Terminal"): Block => ({
  type: "code",
  code,
  label,
});
const links = (...items: { label: string; href: string }[]): Block => ({
  type: "links",
  items,
});
const repo = "https://github.com/Sakib-Dalal/CloudLab";

export const categories = [
  "Start here",
  "Build your lab",
  "Operate & protect",
  "Project",
];

export const guides: Guide[] = [
  {
    slug: "introduction",
    title: "Meet CloudLab",
    category: "Start here",
    minutes: 4,
    description:
      "What CloudLab does, who it is for, and how the pieces fit together.",
    sections: [
      {
        id: "your-personal-lab",
        title: "A home for your compute",
        blocks: [
          text(
            "CloudLab brings your computers into one personal compute lab. Pair a Linux server, a Mac running Docker Desktop, or a compatible single-board computer, then create Linux workspaces on those devices from one dashboard.",
          ),
          text(
            "Open a notebook in JupyterLab, write code in a browser with code-server, or run a Linux command. Each workspace lives in a Docker container with its own persistent project volume and a CPU and memory budget.",
          ),
          note(
            "Your machines do the work",
            "CloudLab runs on hardware you control. This public website is a guide; it does not host your lab, connect to your devices, or store your credentials.",
          ),
        ],
      },
      {
        id: "who-it-is-for",
        title: "Made for curious builders",
        blocks: [
          list(
            "Home lab owners who want one place to organize several computers.",
            "Developers who want reproducible, isolated coding environments.",
            "Students and researchers working with trusted collaborators.",
            "Tinkerers turning spare machines into useful compute nodes.",
          ),
          text(
            "CloudLab is designed for a personal lab or a small trusted group. It has not been audited as a service for running hostile workloads from strangers. Containers share the host kernel; use an additional dedicated VM boundary for untrusted work.",
          ),
        ],
      },
      {
        id: "the-vocabulary",
        title: "Four ideas to know",
        blocks: [
          {
            type: "table",
            columns: ["Term", "What it means"],
            rows: [
              [
                "Coordinator",
                "The Rust service that serves the dashboard, manages access, stores metadata, and coordinates work.",
              ],
              [
                "Lab",
                "A collection of nodes, workspaces, activity, and scoped collaborator access.",
              ],
              [
                "Node",
                "An enrolled computer running a CloudLab agent and Docker. One agent normally represents one physical device.",
              ],
              [
                "Workspace",
                "A container on one chosen node, with a tool, resource limits, and persistent project storage.",
              ],
            ],
          },
          text(
            "A workspace runs on the node you select. CloudLab does not pool RAM across devices, split a job automatically between machines, or move a running container to another node.",
          ),
        ],
      },
      {
        id: "what-you-can-do",
        title: "A lab, from your browser or desktop",
        blocks: [
          list(
            "Create multiple labs and pair nodes with expiring, one-time enrollment keys.",
            "Inspect node health and CPU/RAM telemetry.",
            "Create, stop, resume, and remove JupyterLab, code-server, or Linux console workspaces.",
            "Allocate CPU and memory and configure idle shutdown.",
            "Invite operators or viewers using lab-scoped access keys.",
            "Use a responsive Svelte web app or build the Tauri desktop client.",
          ),
          text(
            "The project takes inspiration from RustDesk’s self-hosted connectivity and Raspberry Pi Connect’s approachable device access. CloudLab uses its own container workspace protocol; it does not provide host screen sharing, host SSH, or either product’s remote-desktop protocol.",
          ),
          links(
            { label: "Build your first lab", href: "/docs/quickstart/" },
            { label: "See the architecture", href: "/docs/architecture/" },
          ),
        ],
      },
    ],
  },
  {
    slug: "quickstart",
    title: "Your first lab",
    category: "Start here",
    minutes: 10,
    description:
      "Build CloudLab, pair your first node, and open a container workspace on one computer.",
    sections: [
      {
        id: "before-you-begin",
        title: "01 / Get the essentials",
        blocks: [
          text(
            "Start on one computer first. You will run both the coordinator and one compute agent on it. Once that works, connect other devices using the remote access guide. Initial builds and image downloads can take several minutes.",
          ),
          list(
            "Git to clone the project.",
            "Node.js 22.12 or newer and npm to build the interface.",
            "A current stable Rust toolchain with Cargo.",
            "Docker Engine or Docker Desktop in Linux-container mode, running on your compute device.",
            "Internet access during installation to download build dependencies and workspace images.",
          ),
          text(
            "The full integration suite has been exercised on macOS with Docker Desktop. Linux and Windows source builds require their platform tooling. On Windows, use Git Bash for the image-building shell script; PowerShell commands are provided below for the coordinator.",
          ),
          links(
            {
              label: "Node.js downloads",
              href: "https://nodejs.org/en/download",
            },
            {
              label: "Install Rust",
              href: "https://www.rust-lang.org/tools/install",
            },
            {
              label: "Install Docker",
              href: "https://docs.docker.com/engine/install/",
            },
          ),
          code("node --version\nrustc --version\ndocker info"),
        ],
      },
      {
        id: "build-the-coordinator",
        title: "02 / Build and start CloudLab",
        blocks: [
          code(
            "git clone https://github.com/Sakib-Dalal/CloudLab.git\ncd CloudLab\nnpm ci\nnpm run build\ncargo build --release --locked -p cloudlab",
          ),
          {
            type: "tabs",
            options: [
              {
                label: "macOS / Linux",
                code: "./target/release/cloudlab serve",
              },
              {
                label: "Windows · PowerShell",
                code: ".\\target\\release\\cloudlab.exe serve",
              },
            ],
          },
          text(
            "Leave this process running and open http://127.0.0.1:8088 in your browser. The coordinator serves the compiled dashboard on port 8088 and its workspace gateway on port 8089. Both bind to loopback by default.",
          ),
          note(
            "Keep the repository as your working directory",
            "The default web directory is dist/ and the default data directory is .cloudlab/. Starting from a different folder without explicit paths can make CloudLab use different assets or create a separate lab.",
          ),
        ],
      },
      {
        id: "sign-in",
        title: "03 / Sign in as the owner",
        blocks: [
          text(
            "On first start, CloudLab generates an owner access key on the coordinator computer. In a second terminal, from the repository directory, read that file and paste its value into the dashboard sign-in form.",
          ),
          {
            type: "tabs",
            options: [
              { label: "macOS / Linux", code: "cat .cloudlab/admin-token" },
              {
                label: "Windows · PowerShell",
                code: "Get-Content .cloudlab/admin-token",
              },
            ],
          },
          note(
            "Treat this key like a password",
            "The owner key grants administration over every lab. Keep it private, out of Git, screenshots, chat, and public build logs. Use scoped access keys for collaborators.",
          ),
        ],
      },
      {
        id: "connect-your-node",
        title: "04 / Prepare and pair a node",
        blocks: [
          text(
            "In another terminal, in the same repository, build the trusted workspace images and install the Rust agent. Keep Docker running. On Windows, run the shell script in Git Bash.",
          ),
          code(
            "./scripts/build-images.sh\ncargo install --locked --path crates/cloudlab",
          ),
          text(
            "In the dashboard, choose Compute nodes → Connect a node. Generate a one-time pairing command and run the exact command shown. For the same computer as the coordinator, it has this shape:",
          ),
          code(
            "cloudlab agent --coordinator http://127.0.0.1:8088 --enrollment YOUR_ONE_TIME_KEY",
          ),
          text(
            "Replace YOUR_ONE_TIME_KEY with the generated value. Enrollment expires after 10 minutes and can only be used once. Leave the agent running and wait for the node to appear online.",
          ),
          note(
            "Pairing a different computer?",
            "127.0.0.1 always refers to the computer running the command. Set up an HTTPS coordinator address before pairing another physical device; see Reach your lab remotely.",
          ),
        ],
      },
      {
        id: "open-a-workspace",
        title: "05 / Make something",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "Open Workspaces and choose New workspace.",
              "Give it a name, select your online node, and pick Linux console, JupyterLab, or code-server.",
              "Set a CPU and memory budget that fits the node, then create the workspace.",
              "Wait for it to be running, then open the console or application.",
              "Save project files inside /home/lab so they persist when the workspace stops.",
            ],
          },
          text(
            "For a simple first check, use a Linux console workspace and run the commands below. The user ID should be 1000. These commands execute inside the container.",
          ),
          code(
            'id\npwd\nprintf "hello from my lab\\n" > /home/lab/projects/hello.txt\ncat /home/lab/projects/hello.txt',
            "Workspace console",
          ),
          links(
            {
              label: "Learn the workspace lifecycle",
              href: "/docs/workspaces/",
            },
            {
              label: "Fix common setup issues",
              href: "/docs/troubleshooting/",
            },
          ),
        ],
      },
    ],
  },
  {
    slug: "nodes",
    title: "Connect your computers",
    category: "Build your lab",
    minutes: 6,
    description:
      "Prepare compute nodes, enroll devices, and keep agents connected across restarts.",
    sections: [
      {
        id: "prepare-each-device",
        title: "Prepare each compute device",
        blocks: [
          text(
            "A node needs Docker in Linux-container mode, the CloudLab Rust binary, and the images for the tools you want to run. A coordinator can also be a node, but it needs its own agent process to contribute compute.",
          ),
          code(
            "git clone https://github.com/Sakib-Dalal/CloudLab.git\ncd CloudLab\ncargo install --locked --path crates/cloudlab\n./scripts/build-images.sh",
          ),
          text(
            "The agent’s operating-system account must be able to use Docker. Docker daemon access is powerful: use a dedicated account or rootless Docker where practical. Do not expose the Docker daemon on a public socket.",
          ),
          text(
            "For ARM devices, build the images on the target device and confirm the upstream image supports its architecture. Raspberry Pi-class devices need a compatible 64-bit Linux system and enough memory for the selected tool. Every Pi model and image combination has not been validated.",
          ),
        ],
      },
      {
        id: "enroll",
        title: "Pair a node with a lab",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "Choose the destination lab in the dashboard.",
              "Open Compute nodes → Connect a node.",
              "Generate a pairing command. Enrollment belongs to that lab.",
              "Run it on the target computer before the 10-minute expiry.",
              "Check that the node appears online before creating a workspace.",
            ],
          },
          code(
            "cloudlab agent \\\n  --coordinator https://lab.example.com \\\n  --enrollment YOUR_ONE_TIME_KEY",
          ),
          text(
            "Remote pairing requires a reachable HTTPS coordinator with a valid certificate. Agents make outbound HTTPS/WSS connections, so individual nodes do not require inbound port forwarding.",
          ),
          links({
            label: "Configure remote access first",
            href: "/docs/remote-access/",
          }),
        ],
      },
      {
        id: "restart",
        title: "Restart without pairing again",
        blocks: [
          text(
            "Enrollment exchanges the short-lived key for a device credential saved in the agent data directory. Restart with the same coordinator URL and the same data directory, without the enrollment flag.",
          ),
          code(
            "cloudlab agent \\\n  --coordinator https://lab.example.com \\\n  --data-dir /var/lib/cloudlab-agent",
          ),
          note(
            "Use the same path on the first run",
            "If you plan to use /var/lib/cloudlab-agent, also pass that --data-dir during enrollment and make it writable by the agent account. Omitting the flag uses .cloudlab/agent relative to the working directory. Never copy an enrolled agent directory to another device.",
          ),
          text(
            "For an always-on Linux node, adapt deploy/cloudlab-agent.service to your binary path, user, coordinator URL, and data directory. Run one agent per physical device in normal operation.",
          ),
        ],
      },
      {
        id: "manage",
        title: "Understand health and revocation",
        blocks: [
          text(
            "Node health and CPU/RAM readings arrive through agent heartbeats. A sleeping computer, stopped agent, unavailable Docker daemon, or interrupted network can make it unavailable. Existing containers remain on that node; CloudLab does not move them automatically.",
          ),
          text(
            "Revoking a node removes its coordinator access. If the device is offline, revocation cannot kill its running containers. Stop those containers locally if needed, and deliberately re-enroll the node before using it again.",
          ),
          links({
            label: "Resource and workspace rules",
            href: "/docs/workspaces/",
          }),
        ],
      },
    ],
  },
  {
    slug: "workspaces",
    title: "Work inside containers",
    category: "Build your lab",
    minutes: 7,
    description:
      "Choose a tool, budget resources, save projects, and understand stop, resume, and removal.",
    sections: [
      {
        id: "choose-a-tool",
        title: "Three ways to work",
        blocks: [
          {
            type: "table",
            columns: ["Template", "Use it for", "Trusted image"],
            rows: [
              [
                "Linux console",
                "Short shell commands, scripts, and quick experiments.",
                "cloudlab/terminal:2",
              ],
              [
                "JupyterLab",
                "Python notebooks, interactive exploration, and project files.",
                "cloudlab/jupyter:2",
              ],
              [
                "code-server",
                "Editing projects with a VS Code-style browser interface.",
                "cloudlab/code:2",
              ],
            ],
          },
          text(
            "Build the selected image on the node before creating a workspace. CloudLab accepts only these fixed templates; it does not let a collaborator submit arbitrary Docker arguments or images.",
          ),
          code(
            "./scripts/build-images.sh terminal\n# Or build every supported template:\n./scripts/build-images.sh",
          ),
        ],
      },
      {
        id: "resources",
        title: "Give each workspace a budget",
        blocks: [
          text(
            "Choose the node, CPU allowance, and memory limit when creating a workspace. Defaults start at 2 CPUs and 2048 MB RAM and can be changed in Lab settings. A node must have enough unreserved capacity for the requested allocation.",
          ),
          text(
            "CPU and memory reservations include stopped workspaces. Stopping a workspace ends its running process, but keeps its reservation so it can resume. Remove a workspace to release that reservation. Resource policies are not per-user quotas.",
          ),
          note(
            "One workspace, one node",
            "A resource budget applies to one container. CloudLab is not a distributed job scheduler and cannot combine memory from multiple computers.",
          ),
        ],
      },
      {
        id: "files",
        title: "Keep files in your project volume",
        blocks: [
          text(
            "Each workspace has a dedicated Docker volume mounted at /home/lab. Store your projects there, for example in /home/lab/projects. Files in this volume survive stop/start cycles. The container’s root filesystem is read-only; temporary paths are not persistent.",
          ),
          text(
            "Use JupyterLab or code-server to upload and edit files inside the workspace. There is no host-directory picker or automatic access to the user’s home folder. The API does not mount host paths or the Docker socket.",
          ),
          text(
            "Removing a workspace removes its container but retains its project volume for deliberate recovery or cleanup. CloudLab does not yet provide a volume restore UI or disk quotas. Back up and manage retained volumes as the host administrator.",
          ),
        ],
      },
      {
        id: "lifecycle",
        title: "Create, stop, resume, remove",
        blocks: [
          list(
            "Create queues work on the selected node; the agent starts the container and waits for the application to be ready.",
            "Open launches the application on a workspace-specific origin using a short-lived ticket. Do not share that ticket URL.",
            "Stop ends compute while preserving files and the resource reservation.",
            "Resume starts the same workspace on the same node.",
            "Remove releases the workspace reservation and removes its container. Its volume stays on the node.",
          ),
          text(
            "If a node is disconnected, a queued operation needs that agent to return. Command execution is at most once; CloudLab does not automatically replay an uncertain shell command after a reconnect.",
          ),
        ],
      },
      {
        id: "network-and-idle",
        title: "Network access and long-running work",
        blocks: [
          text(
            "New workspaces have no external network by default. Their only network interface for applications is private loopback. To allow outbound network access for a newly created workspace, enable the coordinator policy and start the node agent with --allow-network.",
          ),
          note(
            "Outbound access changes the boundary",
            "A Docker bridge can reach services on your LAN. This option is not an Internet-only firewall. Use it only for trusted work and apply host/network controls for stricter egress policies.",
          ),
          text(
            "The default idle stop is 60 minutes. Console/application access resets activity, and open app WebSockets keep a workspace active. Background computation alone does not reset the timer. Disable idle stopping for unattended jobs; the coordinator and agent must be connected for idle stopping to run.",
          ),
          text(
            "The built-in Linux console is a buffered command runner, not an interactive PTY. Commands have a 30-second timeout and 64 KiB output limit. Use the terminal inside JupyterLab or code-server for interactive sessions. The HTTP gateway buffers responses up to 16 MiB; WebSockets are supported, but indefinite HTTP streams/SSE are not.",
          ),
        ],
      },
    ],
  },
  {
    slug: "desktop",
    title: "Use the desktop app",
    category: "Build your lab",
    minutes: 5,
    description:
      "Build the Tauri client and choose a local embedded coordinator or a remote lab.",
    sections: [
      {
        id: "build",
        title: "Build for your platform",
        blocks: [
          text(
            "CloudLab’s desktop app wraps the Svelte interface in Tauri 2. Install the platform prerequisites, then build from the repository root. A verified macOS build exists locally; this website does not offer a signed, prebuilt installer.",
          ),
          links({
            label: "Tauri platform prerequisites",
            href: "https://v2.tauri.app/start/prerequisites/",
          }),
          code("npm ci\nnpm run desktop", "Run in development"),
          code("npm run desktop:build", "Package the application"),
          text(
            "Build outputs are placed under target/release/bundle/. Installers are unsigned unless you configure your own signing and notarization credentials. Linux and Windows desktop packaging still needs platform-specific validation.",
          ),
        ],
      },
      {
        id: "local-mode",
        title: "Start a local lab",
        blocks: [
          text(
            "By default, the desktop app starts an embedded coordinator and workspace gateway on loopback ports 8088 and 8089. A single-use, expiring bootstrap signs in the local owner. You still need Docker, trusted images, and a separately running agent to create workspaces.",
          ),
          text(
            "Desktop-owned lab data lives in the operating system’s application-data directory for dev.cloudlab.desktop. On macOS this is ~/Library/Application Support/dev.cloudlab.desktop. This is separate from the CLI’s default .cloudlab/ directory.",
          ),
          note(
            "Keep the coordinator available",
            "Closing the desktop application stops a coordinator it owns. Containers remain on their nodes. Run a standalone coordinator as a service if your lab should stay reachable when you close the app.",
          ),
        ],
      },
      {
        id: "remote-mode",
        title: "Connect to an always-on lab",
        blocks: [
          text(
            "If a CloudLab coordinator already occupies local port 8088, the app attaches to it and uses normal sign-in. You can also choose Connect to another coordinator on the sign-in screen.",
          ),
          {
            type: "tabs",
            options: [
              {
                label: "macOS / Linux",
                code: "CLOUDLAB_DESKTOP_URL=https://lab.example.com npm run desktop",
              },
              {
                label: "Windows · PowerShell",
                code: '$env:CLOUDLAB_DESKTOP_URL = "https://lab.example.com"\nnpm run desktop',
              },
            ],
          },
          text(
            "Remote mode uses the coordinator’s HTTPS address. The web client and desktop client work with the same labs, access rules, and container workspace gateway.",
          ),
          links({
            label: "Set up your HTTPS gateway",
            href: "/docs/remote-access/",
          }),
        ],
      },
    ],
  },
  {
    slug: "remote-access",
    title: "Reach your lab remotely",
    category: "Build your lab",
    minutes: 9,
    description:
      "Connect from anywhere through an HTTPS gateway you control, with separate workspace origins.",
    sections: [
      {
        id: "understand-the-route",
        title: "Local compute, a reachable gateway",
        blocks: [
          text(
            "Your compute stays on your nodes. To reach it from another network, clients and agents need a route to an HTTPS coordinator. CloudLab does not require a hosted CloudLab account or control plane.",
          ),
          note(
            "Global access needs network setup",
            "CloudLab does not automatically create a public tunnel or perform RustDesk-style NAT punching. DNS, TLS certificates, and a reachable gateway or VPN are required. DuckDNS is a free option if you do not own a domain. The public documentation website on Vercel is not that gateway.",
          ),
          code(
            "Browser or desktop\n  ├─ https://lab.example.com → coordinator :8088\n  └─ https://w-ID.workspaces.lab.example.com → gateway :8089\n                                     ↑\n                      outbound HTTPS / WSS from node agents\n                                     ↓\n                          isolated Docker workspaces",
            "Connection path",
          ),
        ],
      },
      {
        id: "choose-dns",
        title: "Choose your DNS option",
        blocks: [
          note(
            "Recommended free option: DuckDNS",
            "Use one free DuckDNS name, automatic IP updates, and a single wildcard HTTPS certificate for the dashboard and workspace apps. It is a good fit for a home connection with a changing IPv4 address; it does not bypass CGNAT.",
          ),
          links({
            label: "Follow the ten-step DuckDNS setup",
            href: "/docs/duckdns/",
          }),
          text(
            "Already own a domain? Continue with your current DNS provider and the generic configuration below. For private access, use the VPN route at the end of this guide.",
          ),
        ],
      },
      {
        id: "dns-and-tls",
        title: "01 / Set up DNS and HTTPS",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "Point lab.example.com and *.workspaces.lab.example.com at your gateway’s reachable address. Replace example.com with a domain you control.",
              "Run Caddy on that gateway and adapt deploy/Caddyfile from the repository.",
              "Obtain a wildcard certificate for *.workspaces.lab.example.com using a DNS-01 challenge and configure renewal. The example Caddyfile expects certificate files; a DNS-provider Caddy plugin can automate this.",
              "Forward TCP 443 to the gateway. Allow TCP 80 if you use the coordinator certificate’s HTTP challenge.",
              "Keep ports 8088 and 8089 restricted to loopback or your trusted reverse-proxy network.",
            ],
          },
          links(
            {
              label: "Open the Caddy configuration",
              href: `${repo}/blob/main/deploy/Caddyfile`,
            },
            {
              label: "Caddy HTTPS documentation",
              href: "https://caddyserver.com/docs/automatic-https",
            },
          ),
        ],
      },
      {
        id: "configure-cloudlab",
        title: "02 / Set the workspace origin",
        blocks: [
          text(
            "Start the coordinator with the workspace subdomain template. Replace the paths with your actual installation and writable data directory.",
          ),
          code(
            "CLOUDLAB_APP_URL='https://{workspace}.workspaces.lab.example.com' \\\n  cloudlab serve \\\n  --web-dir /opt/cloudlab/web \\\n  --data-dir /var/lib/cloudlab",
          ),
          text(
            "The {workspace} placeholder is replaced with each workspace’s generated ID. Preserve the original Host and WebSocket upgrade headers through your proxy. Do not log ticket query values in gateway access logs.",
          ),
          note(
            "Separate origins are required",
            "Never proxy workspace applications onto the dashboard’s origin, and never combine different workspaces under one origin. This separation prevents container-hosted code from sharing the dashboard’s browser privileges.",
          ),
        ],
      },
      {
        id: "pair-remotely",
        title: "03 / Sign in and pair your devices",
        blocks: [
          text(
            "Open https://lab.example.com and sign in. In Lab settings, save this address as Public coordinator URL. New pairing commands will use it. The setting does not configure DNS, TLS, port forwarding, or the workspace origin template.",
          ),
          text(
            "Generate enrollment commands and run them on your nodes. Agents connect outward over HTTPS/WSS. Existing agents remember their coordinator URL in local credentials; deliberately update their configuration or re-enroll them when moving to a different address.",
          ),
          text(
            "Check dashboard sign-in, node connectivity, a workspace app, and its WebSocket connection from a genuinely different network before depending on remote access.",
          ),
        ],
      },
      {
        id: "vpn-and-cgnat",
        title: "VPNs, home networks, and CGNAT",
        blocks: [
          text(
            "For private access, use WireGuard between clients and a gateway into your coordinator network. Resolve the coordinator and wildcard workspace hostnames to the VPN address. DNS-01 certificates let you use trusted HTTPS without publicly exposing the dashboard.",
          ),
          text(
            "Agents still require HTTPS for non-loopback addresses, even over a VPN. Certificate validation is not disabled. Use a publicly trusted certificate or deliberately configure a client trust policy for your own certificate authority.",
          ),
          text(
            "If your ISP uses CGNAT and you cannot accept inbound traffic, run a WireGuard gateway on another machine you control with a public address and route it to the local coordinator. This may require rented infrastructure if you do not own a reachable machine.",
          ),
          links({
            label: "Troubleshoot connectivity",
            href: "/docs/troubleshooting/#remote-connection-fails",
          }),
        ],
      },
    ],
  },
  duckdnsGuide,
  {
    slug: "access-and-settings",
    title: "Access & settings",
    category: "Operate & protect",
    minutes: 6,
    description:
      "Share the right lab, choose access roles, and tune resource and session policies.",
    sections: [
      {
        id: "roles",
        title: "Give people the access they need",
        blocks: [
          {
            type: "table",
            columns: ["Role", "Scope", "Can do"],
            rows: [
              [
                "Owner",
                "Coordinator-wide",
                "Administer all labs, nodes, access keys, and settings.",
              ],
              [
                "Operator",
                "Assigned lab",
                "Create and operate workspaces and run code inside that lab’s containers.",
              ],
              [
                "Viewer",
                "Assigned lab",
                "Inspect status and activity without operating containers.",
              ],
            ],
          },
          note(
            "Operators share a lab",
            "Operators can access all workspaces and persistent project data in their assigned lab. There are no per-workspace private ACLs. Create separate labs for separate trust groups.",
          ),
        ],
      },
      {
        id: "invite",
        title: "Create and revoke access",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "As owner, select the lab and open Access & security.",
              "Create an access key with a descriptive name and the appropriate role.",
              "Share the key privately with the intended collaborator, along with the coordinator’s HTTPS address.",
              "Revoke the key when access is no longer needed.",
            ],
          },
          text(
            "Collaborator keys expire after 7 days. Browser sessions have their own configurable expiry, 12 hours by default, and are revoked with their access key. Device enrollment keys are a separate credential: one use, 10-minute expiry.",
          ),
          text(
            "CloudLab currently uses access keys. SSO, MFA, email invitations, and identity-provider integration are not implemented.",
          ),
        ],
      },
      {
        id: "settings",
        title: "Choose practical defaults",
        blocks: [
          {
            type: "table",
            columns: ["Setting", "Default", "Effect"],
            rows: [
              ["Coordinator name", "My CloudLab", "Names your installation."],
              [
                "Public coordinator URL",
                "Empty",
                "Sets the address used in newly generated pairing commands.",
              ],
              [
                "Default CPU / memory",
                "2 CPUs / 2048 MB",
                "Pre-fills new workspace allocations.",
              ],
              [
                "Lab workspace cap",
                "12",
                "Limits the number of workspaces in a lab.",
              ],
              [
                "Idle shutdown",
                "60 minutes",
                "Stops workspaces without tracked activity; can be disabled.",
              ],
              [
                "Session duration",
                "12 hours",
                "Controls the lifetime of new dashboard sessions.",
              ],
              [
                "Outbound network",
                "Disabled",
                "Allows requested workspace networking only when the agent also opts in.",
              ],
            ],
          },
          text(
            "These are coordinator-wide policies and defaults, not per-user quotas. The workspace origin template is configured at server launch, separately from the Public coordinator URL.",
          ),
          links(
            { label: "Understand isolation", href: "/docs/security/" },
            { label: "Configure remote origins", href: "/docs/remote-access/" },
          ),
        ],
      },
    ],
  },
  {
    slug: "security",
    title: "Isolation & security",
    category: "Operate & protect",
    minutes: 7,
    description:
      "Understand the safeguards, trust boundaries, and limits before inviting collaborators.",
    sections: [
      {
        id: "trust-model",
        title: "Start with the trust model",
        blocks: [
          text(
            "CloudLab is intended for your personal lab and small groups of trusted collaborators. It has not received an independent security audit and is not a hardened public service for anonymous or hostile workloads.",
          ),
          note(
            "A container is not a virtual machine",
            "Docker containers share the host kernel. An application escape or kernel vulnerability can still put a node at risk. Use rootless Docker where practical and a dedicated Linux VM or separate machine for untrusted workloads.",
          ),
          text(
            "CloudLab gives collaborators a container workspace. It does not grant a host shell, host desktop session, home-directory mount, or access to the Docker daemon through its API.",
          ),
        ],
      },
      {
        id: "container-controls",
        title: "What each workspace enforces",
        blocks: [
          list(
            "Non-root UID/GID 1000 and a read-only root filesystem.",
            "All Linux capabilities dropped and no-new-privileges enabled.",
            "CPU, hard memory/swap, and process-count limits.",
            "Restricted temporary filesystems and a dedicated project volume.",
            "A private network namespace with loopback only by default.",
            "No host folders, Docker socket, host PID/network namespace, GPU, or USB mounts exposed through the API.",
            "A fixed allowlist of workspace templates and verified Docker ownership labels.",
          ),
          text(
            "Workspace app ports are not published on the host. The node relays to a fixed port inside the container through an unprivileged Docker exec stream. Agent accounts themselves need Docker access, which is a host-level trust boundary.",
          ),
        ],
      },
      {
        id: "browser-boundaries",
        title: "Keep browser privileges separate",
        blocks: [
          text(
            "Every browser workspace has its own origin, separate from the dashboard and other workspaces. Opening an app exchanges a single-use, 60-second ticket for a scoped HttpOnly cookie. Coordinator and node credentials are stripped before requests reach the container.",
          ),
          text(
            "The gateway authenticates HTTP and WebSocket traffic, validates workspace hosts and WebSocket origins, and checks session revocation. Preserve this topology when configuring a reverse proxy. Never put untrusted workspace pages on the coordinator’s origin.",
          ),
        ],
      },
      {
        id: "operations",
        title: "Operate within the boundaries",
        blocks: [
          list(
            "Keep the owner key and agent credential directories private. Never commit .cloudlab/ or backup files to Git.",
            "Use HTTPS for all non-loopback connections and keep internal ports off the public network.",
            "Use separate labs for different trust groups; operators share a lab’s workspaces.",
            "Keep Docker, the host OS, trusted base images, and CloudLab updated.",
            "Apply a disk-space policy yourself: persistent volumes have no built-in disk quota.",
            "Enable outbound networking only deliberately; bridge networking can reach the LAN.",
            "Remember that revoking an offline node does not stop containers running on it.",
          ),
          text(
            "On Unix, CloudLab writes private data directories and files with restrictive permissions. Windows installations require appropriate ACLs on the data directory. Backups contain sensitive credentials or credential verifiers and should be encrypted and access-controlled.",
          ),
        ],
      },
      {
        id: "limitations",
        title: "Current limits and review status",
        blocks: [
          text(
            "MFA, SSO, per-workspace ACLs, automatic certificate management, automatic NAT traversal, disk quotas, and arbitrary TCP tunnels are not included. The command console is buffered. HTTP app responses are limited to 16 MiB; WebSocket traffic is supported, but SSE is not.",
          ),
          links(
            {
              label: "Full security notes in the repository",
              href: `${repo}/blob/main/docs/security.md`,
            },
            {
              label: "What has been validated",
              href: "/docs/project/#validation",
            },
            {
              label: "Docker security documentation",
              href: "https://docs.docker.com/engine/security/",
            },
          ),
        ],
      },
    ],
  },
  {
    slug: "architecture",
    title: "How CloudLab works",
    category: "Operate & protect",
    minutes: 5,
    description:
      "Follow a request from the Svelte client through the Rust coordinator to a Docker workspace.",
    sections: [
      {
        id: "components",
        title: "One control point, many compute nodes",
        blocks: [
          {
            type: "table",
            columns: ["Component", "Responsibility", "Runs on"],
            rows: [
              [
                "Svelte 5 client",
                "Dashboard, lab switching, workspace operations, and settings.",
                "Your browser or Tauri webview",
              ],
              [
                "Rust coordinator",
                "Access control, local metadata, node enrollment, and job coordination.",
                "A computer you control",
              ],
              [
                "Rust workspace gateway",
                "Authenticated HTTP/WebSocket relay on separate workspace origins.",
                "Alongside the coordinator",
              ],
              [
                "Rust agent",
                "Heartbeats, resource checks, container lifecycle, and outbound relay.",
                "Each compute device",
              ],
              [
                "Docker workspace",
                "A fixed tool image and dedicated persistent project volume.",
                "The selected compute device",
              ],
            ],
          },
          text(
            "The coordinator does not need Docker and never needs a Docker socket mount. Only compute agents interact with their local Docker daemon. The desktop client can start an embedded coordinator or connect to an existing one.",
          ),
        ],
      },
      {
        id: "request-path",
        title: "What happens when you open a notebook",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "The signed-in client requests a workspace operation from the coordinator.",
              "The coordinator checks the access role and lab scope, then queues the operation for the chosen node.",
              "The node agent verifies the request and starts a constrained Docker container from the trusted Jupyter image.",
              "The app becomes available on its workspace-specific origin after a readiness check.",
              "Your browser exchanges a one-time ticket for a workspace-scoped cookie.",
              "The gateway relays authenticated HTTP and WebSockets through the node’s outbound connection to the container.",
            ],
          },
          text(
            "Files stay in the Docker volume on the node. Traffic passes through your gateway and coordinator; this is not a peer-to-peer protocol that automatically bypasses them.",
          ),
        ],
      },
      {
        id: "state",
        title: "Local state and recovery",
        blocks: [
          text(
            "Coordinator metadata is stored locally in an atomic JSON state file. Access sessions, enrollments, activity, and queued jobs persist across restarts. Agents keep their own credentials and operation receipts so uncertain commands are not blindly replayed.",
          ),
          text(
            "A stopped coordinator cannot enforce idle stopping until it is available again. Containers do not depend on the browser staying open. A disconnected node keeps its containers locally and reconnects when its agent can reach the coordinator.",
          ),
          links({
            label: "Back up and maintain a lab",
            href: "/docs/operations/",
          }),
        ],
      },
      {
        id: "website",
        title: "The public website is separate",
        blocks: [
          text(
            "This site is a prerendered SvelteKit app deployed as static files on Vercel. It has no coordinator, Docker daemon, account database, or access to your lab. GitHub commits can rebuild the site without moving your compute or lab data to Vercel.",
          ),
          links({
            label: "Website deployment guide",
            href: "/docs/website-deployment/",
          }),
        ],
      },
    ],
  },
  {
    slug: "operations",
    title: "Run & maintain a lab",
    category: "Operate & protect",
    minutes: 7,
    description:
      "Run the coordinator as a service, use Docker Compose, and protect your local data.",
    sections: [
      {
        id: "always-on",
        title: "Keep the coordinator running",
        blocks: [
          text(
            "For access while your desktop app is closed, run the Rust coordinator as a service on an always-on machine. Adapt deploy/cloudlab.service and deploy/cloudlab-agent.service for a Linux installation. Set explicit binary, web, and data paths and use a dedicated service account.",
          ),
          code(
            "cloudlab serve \\\n  --web-dir /opt/cloudlab/web \\\n  --data-dir /var/lib/cloudlab",
          ),
          text(
            "The standalone web client is the compiled dist/ directory served by the Rust coordinator. No Vite development server or Node runtime is required after building it.",
          ),
          links(
            {
              label: "Coordinator service template",
              href: `${repo}/blob/main/deploy/cloudlab.service`,
            },
            {
              label: "Agent service template",
              href: `${repo}/blob/main/deploy/cloudlab-agent.service`,
            },
          ),
        ],
      },
      {
        id: "docker-compose",
        title: "Run the coordinator in Docker",
        blocks: [
          code(
            "docker compose up --build -d\ndocker compose exec coordinator cat /data/admin-token",
          ),
          text(
            "The Compose service packages the coordinator and compiled dashboard, persists metadata in a named volume, and binds ports 8088/8089 to loopback. Compute agents run separately on their devices. The coordinator container has no Docker socket.",
          ),
          text(
            "Set CLOUDLAB_APP_URL for remote workspace origins and place your HTTPS proxy in front of both ports. The coordinator data volume must remain writable. The Compose configuration has been validated; the coordinator container image has not yet been exercised by the full integration suite.",
          ),
        ],
      },
      {
        id: "backups",
        title: "Back up metadata and projects separately",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "Stop the coordinator for a consistent metadata backup.",
              "Back up its entire data directory, including the private owner key and state.json. For the CLI default, this is .cloudlab/.",
              "Back up each node’s workspace volumes separately using Docker’s volume backup procedure. Stop or quiesce writing applications first.",
              "Keep agent credential directories private and bound to their original device.",
              "Test restoration to an isolated environment before relying on a backup.",
            ],
          },
          text(
            "A metadata backup alone does not include project files. Deleting a workspace retains its Docker volume, so old volumes can continue using disk space. CloudLab has no automatic backup or volume-restore interface.",
          ),
          links({
            label: "Docker volume backup and restore",
            href: "https://docs.docker.com/engine/storage/volumes/#back-up-restore-or-migrate-data-volumes",
          }),
        ],
      },
      {
        id: "updates",
        title: "Update deliberately",
        blocks: [
          text(
            "Back up first. Review the incoming changes, stop services, rebuild the Svelte dashboard and Rust binary, then restart the coordinator and agents with their existing data paths. Keep coordinator and agents on the same revision.",
          ),
          code(
            "git pull --ff-only\nnpm ci\nnpm run check\nnpm run build\ncargo build --release --locked -p cloudlab",
          ),
          text(
            "Rebuild trusted workspace images when their definitions change. Existing containers keep their original image; replacing a workspace is a deliberate migration. Do not remove project volumes as part of a routine update.",
          ),
        ],
      },
      {
        id: "migration",
        title: "Moving from CloudLab 1.x",
        blocks: [
          text(
            "The previous Go/Python implementation is archived under legacy/ for reference and is not part of the new build. Stop old host services and public tunnels before using 2.0. Host SSH, host Python management, credential emails, and automatic public tunnels are replaced by container-scoped tools and explicit authenticated connectivity.",
          ),
          text(
            "There is no automatic migration of old notebooks, environments, or configuration. Copy only the project files you need into a new workspace’s project volume.",
          ),
          links({
            label: "Migration notice",
            href: `${repo}/blob/main/legacy/NOTICE.md`,
          }),
        ],
      },
    ],
  },
  {
    slug: "troubleshooting",
    title: "Troubleshooting",
    category: "Operate & protect",
    minutes: 6,
    description:
      "Resolve offline nodes, missing images, sign-in problems, and remote workspace failures.",
    sections: [
      {
        id: "dashboard-wont-open",
        title: "The dashboard will not open",
        blocks: [
          list(
            "Confirm the coordinator process is running and read its terminal output.",
            "Open http://127.0.0.1:8088 on the coordinator computer; localhost on another computer points somewhere else.",
            "Build the frontend with npm ci && npm run build and check that --web-dir points to dist/.",
            "Check whether another process already uses port 8088 or 8089. The desktop app uses these stable ports too.",
          ),
          code("curl http://127.0.0.1:8088/api/health"),
        ],
      },
      {
        id: "sign-in-fails",
        title: "My key is not accepted",
        blocks: [
          text(
            "Use the key from the data directory of the coordinator you are actually connecting to. The CLI’s .cloudlab/ directory and desktop application-data directory are separate. A collaborator key may be expired or revoked. Ask the owner to create another scoped key rather than sharing the owner credential.",
          ),
          text(
            "If a browser session expires, sign in again. Do not publish access keys in issue reports, screenshots, URLs, or logs.",
          ),
        ],
      },
      {
        id: "node-offline",
        title: "A node is offline or will not pair",
        blocks: [
          list(
            "Check that the node is awake, Docker is running, and the agent process is still active.",
            "Run docker info as the agent’s user to confirm daemon access.",
            "A new enrollment key expires in 10 minutes and works once. Generate another if needed.",
            "For an enrolled node, restart without --enrollment using its original coordinator and data directory.",
            "Confirm the HTTPS address and certificate are valid from the node. An HTTP LAN IP is not accepted.",
            "Check proxy support for WebSocket upgrades and whether a firewall blocks outbound HTTPS.",
          ),
        ],
      },
      {
        id: "workspace-fails",
        title: "A workspace fails to start",
        blocks: [
          text(
            "Read the operation entry in Activity. Common causes are a missing trusted image, insufficient unreserved CPU/RAM, unavailable Docker, or an offline node. Build the image on the selected node, not only on the coordinator.",
          ),
          code(
            "docker image ls cloudlab/terminal\ndocker image ls cloudlab/jupyter\ndocker image ls cloudlab/code",
          ),
          text(
            "Stopped workspaces still reserve CPU and memory. Remove a workspace you no longer need to release its reservation, remembering that its project volume remains on the node.",
          ),
        ],
      },
      {
        id: "remote-connection-fails",
        title: "The dashboard loads, but the workspace does not",
        blocks: [
          list(
            "Check DNS for the exact workspace hostname, not only the dashboard: w-ID.NAME.duckdns.org for DuckDNS, or your configured custom-domain template.",
            "Verify the wildcard certificate covers *.NAME.duckdns.org for DuckDNS, or *.workspaces.lab.example.com for the generic example.",
            "Set CLOUDLAB_APP_URL at coordinator startup and point the wildcard site to gateway port 8089.",
            "Preserve Host and WebSocket upgrade headers in the proxy.",
            "Reopen the app from the dashboard. A launch ticket is single-use and expires after 60 seconds.",
            "For local use, your browser must resolve *.localhost to loopback. Otherwise configure local DNS/hosts or your own wildcard HTTPS domain.",
          ),
          text(
            "The HTTP gateway buffers up to 16 MiB per response. Larger downloads and indefinite HTTP streams are outside the current gateway limits.",
          ),
          links(
            {
              label: "Review the remote access checklist",
              href: "/docs/remote-access/",
            },
            {
              label: "DuckDNS connection checks",
              href: "/docs/duckdns/#troubleshooting",
            },
          ),
        ],
      },
      {
        id: "network-and-jobs",
        title: "A package install or background job fails",
        blocks: [
          text(
            "Workspaces have no external network by default. If you need package downloads, opt into outbound networking in coordinator settings and at agent startup before creating that workspace. This grants bridge networking that may reach your LAN.",
          ),
          text(
            "The root filesystem is read-only. Install user tools under /home/lab when supported, or maintain the trusted image as the host administrator. Disable idle stopping for unattended computation; background CPU activity alone does not keep a workspace active.",
          ),
          text(
            "The basic console has a 30-second timeout and 64 KiB output limit. Use a JupyterLab or code-server terminal for interactive work.",
          ),
        ],
      },
      {
        id: "report",
        title: "Still stuck?",
        blocks: [
          text(
            "Open a GitHub issue with your OS, CPU architecture, CloudLab revision, Docker version, the failing step, and a sanitized error message. Remove owner keys, enrollment keys, agent credentials, app tickets, and private project content first.",
          ),
          links({ label: "Report an issue on GitHub", href: `${repo}/issues` }),
        ],
      },
    ],
  },
  {
    slug: "website-deployment",
    title: "Deploy this website",
    category: "Project",
    minutes: 4,
    description:
      "Publish the public SvelteKit site on Vercel and deploy changes automatically from GitHub.",
    sections: [
      {
        id: "separate-builds",
        title: "Two apps in one repository",
        blocks: [
          text(
            "The public documentation site lives in website/. The private lab dashboard lives in src/ and is served by the Rust coordinator. Root vercel.json builds only website/ into website/build/; it does not publish the coordinator, dashboard, credentials, or workspace data.",
          ),
          code("npm ci --prefix website\nnpm run dev --prefix website"),
          text(
            "The website development server opens on http://127.0.0.1:4173. It does not need a running CloudLab coordinator, Rust, or Docker.",
          ),
        ],
      },
      {
        id: "connect-vercel",
        title: "Connect GitHub to Vercel once",
        blocks: [
          {
            type: "list",
            ordered: true,
            items: [
              "In Vercel, import the Sakib-Dalal/CloudLab GitHub repository, or open the existing project linked to it.",
              "Keep Root Directory at the repository root, not website/. The root vercel.json contains the build configuration.",
              "Use Node.js 22.x or a newer version compatible with the project.",
              "Set the production branch to main and leave Git deployments enabled.",
              "Optionally set PUBLIC_SITE_URL to your final HTTPS website origin, without a path. It defaults to https://cloudlab-alpha.vercel.app.",
              "Deploy. Vercel runs the site type check, static build, and generated-page validation before publishing.",
            ],
          },
          note(
            "No deployment tokens in the repository",
            "Use Vercel’s GitHub integration. It handles deployment permissions without committing a Vercel token or adding a custom deployment workflow.",
          ),
          links({
            label: "Vercel GitHub integration docs",
            href: "https://vercel.com/docs/git/vercel-for-github",
          }),
        ],
      },
      {
        id: "automatic-deployments",
        title: "Every push has a path to production",
        blocks: [
          text(
            "With the repository connected, a push to main creates a production deployment. Branch pushes and pull requests get Vercel preview deployments according to project settings. GitHub Actions separately validates the website’s build and generated links.",
          ),
          code(
            'git add website vercel.json\ngit commit -m "Update CloudLab website"\ngit push origin main',
            "Publish a website update",
          ),
          text(
            "A failed site build does not replace the last successful production site. Use Vercel deployment history to inspect build logs or roll back. Branch protection and any required GitHub checks are configured in GitHub; the Vercel build itself runs the website checks.",
          ),
        ],
      },
      {
        id: "custom-domain",
        title: "Use your own domain",
        blocks: [
          text(
            "Add the domain in the Vercel project’s Domains settings, follow the DNS instructions shown there, and set PUBLIC_SITE_URL to that HTTPS origin. Redeploy so canonical URLs and the sitemap use the new address.",
          ),
          text(
            "Keep this public website domain separate from your coordinator and workspace origins. Vercel serves the guide; your own HTTPS gateway serves the actual lab.",
          ),
          links({
            label: "Vercel custom domains",
            href: "https://vercel.com/docs/domains/working-with-domains/add-a-domain",
          }),
        ],
      },
    ],
  },
  {
    slug: "project",
    title: "Project & roadmap",
    category: "Project",
    minutes: 5,
    description:
      "Explore the source, current release, validation status, and ways to contribute.",
    sections: [
      {
        id: "open-source",
        title: "Built in the open",
        blocks: [
          text(
            "CloudLab is an MIT-licensed project by Sakib Dalal. Version 2.0 moves the backend from Go to Rust and introduces a Svelte 5 interface, a Tauri 2 desktop client, multi-node labs, and container-based workspaces.",
          ),
          text(
            "You can inspect the implementation, build it yourself, suggest improvements, and contribute fixes. No subscription or CloudLab account is needed to run it. Your own hardware, electricity, domain, or optional reachable gateway may have costs.",
          ),
          links(
            { label: "Browse the source", href: repo },
            {
              label: "Read the MIT license",
              href: `${repo}/blob/main/LICENSE`,
            },
            { label: "Open issues", href: `${repo}/issues` },
          ),
        ],
      },
      {
        id: "validation",
        title: "What has been checked",
        blocks: [
          list(
            "Svelte type and accessibility checks and a production frontend build.",
            "Rust formatting, Clippy, and seven unit tests.",
            "A real-Docker integration suite with a coordinator and two agents on one macOS machine.",
            "Enrollment, lab scoping, revocation, resource limits, non-root execution, stop/start persistence, and restart recovery.",
            "JupyterLab HTTP/WebSocket access and code-server through the authenticated gateway.",
            "A macOS Tauri build, application launch, and local owner bootstrap.",
            "The public website’s static build, internal routes, anchors, metadata, and output boundary.",
          ),
          note(
            "Validation has a scope",
            "Two agents on one host are not the same as two physical machines. Public DNS/TLS and Internet connectivity, Linux/Windows desktop packaging, the coordinator Docker image, and hostile multi-tenant use still need independent validation. No full browser interaction audit is claimed.",
          ),
          links({
            label: "Detailed validation record",
            href: `${repo}/blob/main/docs/validation.md`,
          }),
        ],
      },
      {
        id: "future-work",
        title: "Useful next steps",
        blocks: [
          text(
            "These are contribution opportunities, not shipped features or promised release dates:",
          ),
          list(
            "Signed, reproducible installers and broader platform coverage.",
            "Independent security review and stronger untrusted-workload isolation.",
            "MFA, SSO, and finer workspace permissions.",
            "Disk quotas, volume recovery, and managed backup flows.",
            "Simpler self-hosted remote connectivity and certificate setup.",
            "Broader streaming support and richer observability.",
          ),
          text(
            "Discuss larger changes in an issue before building them so the design fits the project’s local-first model.",
          ),
        ],
      },
      {
        id: "contribute",
        title: "Contribute a focused change",
        blocks: [
          text(
            "Fork the repository, create a branch, make your change, and include a clear before/after description and relevant validation in a pull request. Keep credentials and generated build artifacts out of commits.",
          ),
          code(
            "npm run check\nnpm run build\ncargo fmt --check\ncargo clippy --locked -p cloudlab --all-targets -- -D warnings\ncargo test --locked -p cloudlab",
            "Application checks",
          ),
          code(
            "npm ci --prefix website\nnpm run check --prefix website\nnpm run build --prefix website\nnpm test --prefix website",
            "Website checks",
          ),
          text(
            "The real-Docker integration suite and platform build commands are documented in the repository README. Run checks appropriate to the part you changed.",
          ),
        ],
      },
    ],
  },
];

export function searchGuides(query: string): Guide[] {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) return guides;
  return guides
    .map((guide) => {
      const title = `${guide.title} ${guide.description}`.toLowerCase();
      const body = JSON.stringify(guide.sections).toLowerCase();
      return {
        guide,
        score: terms.every((term) => `${title} ${body}`.includes(term))
          ? terms.reduce((n, term) => n + (title.includes(term) ? 3 : 1), 0)
          : 0,
      };
    })
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((item) => item.guide);
}
