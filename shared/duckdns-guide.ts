import type { Guide } from "./guide-types";

export const duckdnsGuide: Guide = {
  slug: "duckdns",
  title: "Set up remote access with DuckDNS",
  category: "Build your lab",
  minutes: 15,
  description:
    "Give your home lab a free address, keep a changing IP up to date, and connect JupyterLab and VS Code through automatic HTTPS. Follow all ten setup steps.",
  sections: [
    {
      id: "plan-your-addresses",
      title: "One name for your whole lab",
      blocks: [
        {
          type: "text",
          text: "DuckDNS is our recommended free DNS option for a home lab with a changing public IPv4 address. You can use it without buying a domain or configuring a registrar’s DNS. DuckDNS provides the name; Caddy and CloudLab stay on your hardware. Your internet connection is still required.",
        },
        {
          type: "table",
          columns: ["Purpose", "Use this address"],
          rows: [
            ["Register this name", "my-cloudlab.duckdns.org"],
            [
              "Dashboard and node enrollment",
              "https://lab.my-cloudlab.duckdns.org",
            ],
            [
              "Workspace origin template",
              "https://{workspace}.my-cloudlab.duckdns.org",
            ],
            ["Example workspace", "https://w-ID.my-cloudlab.duckdns.org"],
            ["One certificate for both", "*.my-cloudlab.duckdns.org"],
          ],
        },
        {
          type: "note",
          title: "Replace my-cloudlab with your own registered name",
          text: "Use lab.NAME.duckdns.org for the dashboard, not the bare NAME.duckdns.org. Keep workspaces at w-ID.NAME.duckdns.org, with no extra workspaces. level. This lets one wildcard certificate cover both while CloudLab keeps their origins separate.",
        },
        {
          type: "links",
          items: [
            {
              label: "What DuckDNS does",
              href: "https://www.duckdns.org/why.jsp",
            },
            {
              label: "Repository setup guide",
              href: "https://github.com/Sakib-Dalal/CloudLab/blob/main/docs/duckdns.md",
            },
          ],
        },
      ],
    },
    {
      id: "check-your-network",
      title: "01 / Prepare a reachable gateway",
      blocks: [
        {
          type: "text",
          text: "Start with a working local CloudLab. Run these commands from its repository on the coordinator computer. The supplied Caddy configuration runs on that same computer and proxies to 127.0.0.1:8088 and :8089. Install Python 3 for the updater and a current Go toolchain for the Caddy build.",
        },
        {
          type: "list",
          ordered: true,
          items: [
            "Reserve a fixed LAN address for this computer in your router.",
            "Check the router’s WAN IPv4 address against your public IPv4. A private WAN address, a 100.64.0.0/10 address, or an upstream mismatch can indicate double NAT or CGNAT.",
            "If necessary, ask your ISP for a reachable public IPv4, forward through both routers you control, or choose the VPN/gateway route below.",
          ],
        },
        {
          type: "note",
          title: "DNS does not create a tunnel",
          text: "DuckDNS does not bypass CGNAT, forward ports, or replace your internet provider. A reachable public gateway or VPN route is still needed. This guide uses public IPv4 and TCP 443.",
        },
        {
          type: "links",
          items: [
            { label: "Build a local lab first", href: "/docs/quickstart/" },
            {
              label: "VPNs and CGNAT",
              href: "/docs/remote-access/#vpn-and-cgnat",
            },
            { label: "Install Go", href: "https://go.dev/doc/install" },
          ],
        },
      ],
    },
    {
      id: "register",
      title: "02 / Register your DuckDNS name",
      blocks: [
        {
          type: "list",
          ordered: true,
          items: [
            "Visit DuckDNS and sign in with an available provider.",
            "Add an available name such as my-cloudlab. Register just one name; lab, individual workspace IDs, and * do not need separate registrations.",
            "Find the DuckDNS account token at the top of the account page. Save it privately in the next step.",
            "Check the name’s IPv4 address. The updater will keep it aligned with your gateway’s public address.",
          ],
        },
        {
          type: "note",
          title: "Use the DuckDNS token here",
          text: "This account token is separate from CloudLab owner, collaborator, and node enrollment keys. Do not enter it into the public website or CloudLab’s Public coordinator URL field.",
        },
        {
          type: "links",
          items: [
            { label: "Register at DuckDNS", href: "https://www.duckdns.org/" },
          ],
        },
      ],
    },
    {
      id: "save-settings",
      title: "03 / Save two private settings",
      blocks: [
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: "mkdir -p .cloudlab\nchmod 700 .cloudlab\ncp -n deploy/duckdns.env.example .cloudlab/duckdns.env\nchmod 600 .cloudlab/duckdns.env",
            },
            {
              label: "Windows PowerShell",
              code: "New-Item -ItemType Directory -Force .cloudlab | Out-Null\nif (-not (Test-Path .cloudlab/duckdns.env)) {\n  Copy-Item deploy/duckdns.env.example .cloudlab/duckdns.env\n}\nnotepad .cloudlab/duckdns.env",
            },
          ],
        },
        {
          type: "text",
          text: "Open .cloudlab/duckdns.env in your editor and replace both values below. Use plain, unquoted values. The subdomain is only your registered name, without .duckdns.org. On Windows keep the file in your private user profile and restrict its Properties → Security permissions to your account and necessary administrators.",
        },
        {
          type: "code",
          label: ".cloudlab/duckdns.env",
          code: "DUCKDNS_SUBDOMAIN=my-cloudlab\nDUCKDNS_API_TOKEN=REPLACE_WITH_YOUR_DUCKDNS_TOKEN",
        },
        {
          type: "text",
          text: "The .cloudlab directory is ignored by Git. Keep this file private; never paste the token into shell commands, issue reports, screenshots, or website settings.",
        },
      ],
    },
    {
      id: "update-address",
      title: "04 / Update and check your address",
      blocks: [
        {
          type: "text",
          text: "Run this on the gateway’s network, not a traveling laptop, a remote compute node, or a machine using an unrelated VPN. The helper asks DuckDNS to detect the gateway’s public IPv4 over certificate-verified HTTPS; it does not modify certificate TXT records.",
        },
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: "python3 scripts/duckdns-update.py --config .cloudlab/duckdns.env",
            },
            {
              label: "Windows PowerShell",
              code: "py -3 scripts/duckdns-update.py --config .cloudlab/duckdns.env",
            },
          ],
        },
        {
          type: "text",
          text: "Expect: OK: DuckDNS IPv4 address updated. Errors exit with a failure status and never print the token. Check both names below resolve to your public IPv4. w-test is only a DNS probe, not a real workspace. Allow cached DNS answers time to expire after an address change.",
        },
        {
          type: "code",
          label: "Check both DNS names",
          code: "nslookup lab.my-cloudlab.duckdns.org\nnslookup w-test.my-cloudlab.duckdns.org",
        },
        {
          type: "note",
          title: "Check IPv6 too",
          text: "This helper updates IPv4 only. If your DuckDNS name has an IPv6/AAAA record, remove it in DuckDNS settings unless that IPv6 route and firewall also reach the gateway. A stale AAAA address can break an otherwise working IPv4 setup.",
        },
        {
          type: "links",
          items: [
            {
              label: "DuckDNS update API",
              href: "https://www.duckdns.org/spec.jsp",
            },
          ],
        },
      ],
    },
    {
      id: "automatic-updates",
      title: "05 / Keep a changing IP up to date",
      blocks: [
        {
          type: "text",
          text: "Choose one updater. A router with built-in DuckDNS over HTTPS uses the least extra software: enter your registered name and token in its DDNS settings, enable updates, and check its status. Otherwise run the included helper, which updates immediately and every five minutes.",
        },
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: "python3 scripts/duckdns-update.py --config .cloudlab/duckdns.env --watch",
            },
            {
              label: "Windows PowerShell",
              code: "py -3 scripts/duckdns-update.py --config .cloudlab/duckdns.env --watch",
            },
          ],
        },
        {
          type: "text",
          text: "For Linux/macOS reboot persistence, find Python with command -v python3, open crontab -e, and append the one-shot job below with your actual absolute paths. Keep existing entries. Do not use --watch in a scheduled job.",
        },
        {
          type: "code",
          label: "Linux/macOS crontab — replace absolute paths",
          code: "*/5 * * * * /ABS/PATH/python3 /ABS/PATH/CloudLab/scripts/duckdns-update.py --config /ABS/PATH/CloudLab/.cloudlab/duckdns.env >/dev/null",
        },
        {
          type: "text",
          text: "On Windows, open Task Scheduler → Create Task. Choose your gateway account, an At startup trigger, and repetition every 5 minutes indefinitely. The action runs your full python.exe path, with arguments shown below. Run once and check Last Run Result is 0x0. If needed, allow it to run while signed out. Keep the gateway awake.",
        },
        {
          type: "code",
          label: "Windows Task Scheduler arguments",
          code: '"C:\\PATH\\CloudLab\\scripts\\duckdns-update.py" --config "C:\\PATH\\CloudLab\\.cloudlab\\duckdns.env"',
        },
        {
          type: "note",
          title: "One name, one updater",
          text: "Do not run competing updaters from different networks. Choose your router, the foreground watcher, or a scheduled one-shot job. The helper rereads the config each cycle so token changes take effect without a restart.",
        },
        {
          type: "links",
          items: [
            {
              label: "DuckDNS router and OS instructions",
              href: "https://www.duckdns.org/install.jsp",
            },
          ],
        },
      ],
    },
    {
      id: "build-caddy",
      title: "06 / Add DuckDNS support to Caddy",
      blocks: [
        {
          type: "text",
          text: "Build Caddy once with the DuckDNS module. A standard Caddy binary may not contain it. You can alternatively download a custom build from Caddy’s website with github.com/caddy-dns/duckdns selected.",
        },
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: "go run github.com/caddyserver/xcaddy/cmd/xcaddy@latest build --with github.com/caddy-dns/duckdns --output .cloudlab/caddy\n.cloudlab/caddy list-modules",
            },
            {
              label: "Windows PowerShell",
              code: "go run github.com/caddyserver/xcaddy/cmd/xcaddy@latest build --with github.com/caddy-dns/duckdns --output .cloudlab/caddy.exe\n.cloudlab/caddy.exe list-modules",
            },
          ],
        },
        {
          type: "text",
          text: "Confirm dns.providers.duckdns appears. Rebuild only when installing or updating Caddy, not on every IP update.",
        },
        {
          type: "links",
          items: [
            {
              label: "Custom Caddy download",
              href: "https://caddyserver.com/download",
            },
            {
              label: "Caddy build instructions",
              href: "https://caddyserver.com/docs/build",
            },
            {
              label: "DuckDNS module",
              href: "https://github.com/caddy-dns/duckdns",
            },
          ],
        },
      ],
    },
    {
      id: "validate-https",
      title: "07 / Validate the wildcard HTTPS config",
      blocks: [
        {
          type: "text",
          text: "The included Caddyfile reads your private settings, obtains and renews one wildcard certificate with DNS-01, sends lab.NAME to port 8088, and sends workspace hosts to 8089. It preserves Host and WebSocket upgrades and leaves request access logging off.",
        },
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: ".cloudlab/caddy validate --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env",
            },
            {
              label: "Windows PowerShell",
              code: ".cloudlab/caddy.exe validate --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env",
            },
          ],
        },
        {
          type: "note",
          title: "Use just the wildcard certificate",
          text: "DuckDNS shares one TXT value across this registered name’s subdomains. Do not request separate certificates for each workspace, add the bare registered name, or run a competing ACME client against that name. Validation should say Valid configuration; real certificate issuance still needs working DNS and internet access.",
        },
        {
          type: "links",
          items: [
            {
              label: "Included Caddyfile",
              href: "https://github.com/Sakib-Dalal/CloudLab/blob/main/deploy/Caddyfile.duckdns",
            },
            {
              label: "How Caddy manages HTTPS",
              href: "https://caddyserver.com/docs/automatic-https",
            },
          ],
        },
      ],
    },
    {
      id: "start-services",
      title: "08 / Configure CloudLab and start HTTPS",
      blocks: [
        {
          type: "text",
          text: "Stop the existing coordinator cleanly and restart it with the same data directory and the new workspace template. The command below is for a local CLI installation in this repository. Preserve your existing installation paths if different. Keep {workspace} literally; CloudLab inserts w- plus the generated workspace ID.",
        },
        {
          type: "code",
          label: "Restart the coordinator — replace my-cloudlab",
          code: "cloudlab serve --web-dir dist --data-dir .cloudlab --app-url 'https://{workspace}.my-cloudlab.duckdns.org'",
        },
        {
          type: "text",
          text: "For an existing service, change its CLOUDLAB_APP_URL or --app-url setting instead. Do not start a second coordinator on the same ports or move to an empty data directory. On Linux grant the custom Caddy binary permission to bind port 443 with the command below; your distribution needs the setcap utility. Repeat after replacing the binary, or use Caddy’s packaged-service custom-binary instructions.",
        },
        {
          type: "code",
          label: "Linux only",
          code: "sudo setcap cap_net_bind_service=+ep .cloudlab/caddy",
        },
        {
          type: "text",
          text: "In another terminal start Caddy. Keep it running and keep its certificate storage persistent and writable so renewal works.",
        },
        {
          type: "tabs",
          options: [
            {
              label: "Linux / macOS",
              code: ".cloudlab/caddy run --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env",
            },
            {
              label: "Windows PowerShell",
              code: ".cloudlab/caddy.exe run --config deploy/Caddyfile.duckdns --envfile .cloudlab/duckdns.env",
            },
          ],
        },
        {
          type: "text",
          text: "For unattended operation, run Caddy and CloudLab as startup services using your actual absolute paths and the repository as the working directory. The IP updater is a separate job. On Linux a user service can run this Caddy command; enable lingering for its user if it must survive logout. Use the platform service instructions below rather than relying on an open terminal.",
        },
        {
          type: "links",
          items: [
            {
              label: "Keep Caddy running on your OS",
              href: "https://caddyserver.com/docs/running",
            },
            {
              label: "Custom binary with a Linux package",
              href: "https://caddyserver.com/docs/build#package-support-files-for-custom-builds-for-debianubunturaspbian",
            },
            { label: "CloudLab service operations", href: "/docs/operations/" },
          ],
        },
      ],
    },
    {
      id: "router",
      title: "09 / Forward HTTPS through your router",
      blocks: [
        {
          type: "list",
          ordered: true,
          items: [
            "Forward WAN TCP 443 to the gateway’s reserved LAN address on TCP 443.",
            "Allow inbound TCP 443 through the gateway computer’s firewall.",
            "Keep CloudLab 8088/8089 on loopback; do not expose the Docker socket or Docker API.",
            "Use the https:// address explicitly. This Caddyfile disables HTTP redirects and uses DNS-01, so inbound TCP 80 is not required.",
            "Allow outbound HTTPS to DuckDNS and the certificate authority, plus DNS lookups from Caddy.",
          ],
        },
        {
          type: "note",
          title: "Already running a reverse proxy?",
          text: "If another proxy owns port 443, merge this site into that Caddy deployment instead of starting another listener. Only one process can bind the same address and port.",
        },
      ],
    },
    {
      id: "connect-and-test",
      title: "10 / Sign in, pair devices, and test",
      blocks: [
        {
          type: "list",
          ordered: true,
          items: [
            "From mobile data or another network, open https://lab.my-cloudlab.duckdns.org and use your existing CloudLab access key.",
            "In Lab settings, save that exact HTTPS address as Public coordinator URL. This changes new pairing commands, not DNS, TLS, or the workspace origin setting.",
            "Generate a pairing command in Compute nodes → Connect a node and run it on the target device. Existing nodes can keep an unchanged reachable coordinator address; deliberately update their own configuration when changing it.",
            "Open JupyterLab, run a notebook cell, and open VS Code and its terminal. Confirm each uses w-ID.my-cloudlab.duckdns.org, a trusted HTTPS certificate, and the CloudLab header.",
            "If mobile data works but home Wi-Fi does not, check router NAT loopback/hairpin support. A local DNS override must resolve both dashboard and workspace hosts to the gateway.",
          ],
        },
        {
          type: "links",
          items: [
            { label: "Pair a compute node", href: "/docs/nodes/" },
            { label: "Open your first workspace", href: "/docs/workspaces/" },
          ],
        },
      ],
    },
    {
      id: "troubleshooting",
      title: "If something does not connect",
      blocks: [
        {
          type: "table",
          columns: ["Symptom", "What to check"],
          rows: [
            [
              "Updater rejected / KO",
              "Correct DuckDNS account token, registered name, and plain unquoted values.",
            ],
            [
              "Wrong IP",
              "Updater runs on the gateway network; no competing updater or unrelated VPN.",
            ],
            [
              "HTTPS timeout",
              "CGNAT, public WAN address, port 443 forwarding, firewall, and gateway sleep.",
            ],
            [
              "Module not registered",
              "Use the custom Caddy binary, not an older standard one on PATH.",
            ],
            [
              "Permission denied on 443",
              "Apply the Linux capability or use the configured Caddy service.",
            ],
            [
              "Certificate challenge fails",
              "Token, DNS propagation, logs, and a competing ACME client changing the shared TXT value.",
            ],
            [
              "Dashboard works; workspace fails",
              "Exact one-level origin template, proxy to 8089, Host/WebSocket forwarding, and a fresh launch from the dashboard.",
            ],
            [
              "Certificate warning on bare hostname",
              "Use lab.NAME.duckdns.org; the wildcard does not cover NAME.duckdns.org.",
            ],
          ],
        },
        {
          type: "links",
          items: [
            {
              label: "Full troubleshooting guide",
              href: "/docs/troubleshooting/",
            },
            { label: "DuckDNS FAQ", href: "https://www.duckdns.org/faqs.jsp" },
          ],
        },
      ],
    },
  ],
};
