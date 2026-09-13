# Host CloudLab on a cloud VM

The cloud setup hosts the dashboard **and** compute workspaces on a dedicated VM. Open `http://PUBLIC_IP` and the gateway redirects to `https://lab.PUBLIC-IP.sslip.io`. Sign in with your owner key, select **Cloud workspace**, and create a Terminal, Jupyter, or VS Code workspace. The VM is paired automatically; other nodes can join through the public HTTPS address.

The automatic path uses public IPv4, Docker Engine with its Compose plugin, Python 3, systemd, and glibc 2.36 or later. Use **Ubuntu 24.04+ or Debian 12+**, on x86-64 or ARM64. Have this version of the CloudLab source on the VM. The script builds the dashboard, agent and all three workspace images, so allow time and enough disk/memory for those builds (4 vCPU, 8 GB RAM and 40 GB disk is a useful starting point). Windows VMs and older distributions need a manual installation; this script does not install Docker or GPU drivers.

## Set up the VM

1. Create a VM with a reachable public IPv4. Reserve a stable address so stopping the VM does not change your lab URL.
2. Allow inbound **TCP 80 and 443** in the provider firewall and the VM firewall. Keep SSH restricted to your administrator IP. Do not open 8088, 8089, or container app ports. Outbound DNS and HTTPS must work for image downloads and certificate issuance.
3. Install [Docker Engine and the Compose plugin](https://docs.docker.com/engine/install/) from Docker's instructions for your distribution. Use the local standard Docker socket `/var/run/docker.sock`.
4. Copy/clone the CloudLab source version containing `scripts/cloud-access.py` onto that VM. From its project folder, run one of the commands below.

| Provider | Stable address | Inbound rules | Setup command |
| --- | --- | --- | --- |
| AWS EC2 | Elastic IP | EC2 security group | `sudo python3 scripts/cloud-access.py --provider aws --apply` |
| Google Cloud Compute Engine | Static external IP | VPC firewall | `sudo python3 scripts/cloud-access.py --provider gcp --apply` |
| Azure Virtual Machines | Static public IP | Network security group | `sudo python3 scripts/cloud-access.py --provider azure --apply` |
| DigitalOcean Droplets | Reserved IP | Cloud Firewall | `sudo python3 scripts/cloud-access.py --provider digitalocean --apply` |
| Any other provider | Static public IPv4 | Provider firewall | `sudo python3 scripts/cloud-access.py --provider other --public-ip YOUR_PUBLIC_IP --apply` |

`--provider auto` tries the four supported metadata services. `--public-ip YOUR_PUBLIC_IP` overrides detection for any provider, including when metadata is disabled, the address belongs to a load balancer/NAT mapping, or the VM has multiple interfaces. AWS uses IMDSv2 only. Detection reads only public address metadata on the VM, bypasses proxies, rejects redirects, and never retrieves cloud account credentials or instance user-data. DigitalOcean detection prefers an attached Reserved IP.

Omit `--apply` to generate a plan without building, installing, starting services, or changing firewalls. Plans default to `.cloudlab/cloud-access-plan`, separate from the active deployment. Plans can also be generated on your own computer with an explicit IP. The in-app **Settings → Set up cloud access** and **Access & security → Set up remote access** screens show a provider-specific command and a visual connection map.

## What setup configures

- Persistent coordinator and Caddy containers in the `cloudlab-cloud` Compose project. Only 80/443 are public; the coordinator's 8088 port is bound to loopback for the local agent. The workspace gateway has no published port.
- A host compute service named `cloudlab-cloud-agent.service`, a root-owned executable at `/usr/local/lib/cloudlab-cloud/cloudlab`, and private pairing data in `/var/lib/cloudlab-cloud-agent`. This trusted service has access to the host Docker daemon. The coordinator and workspaces never receive its socket or credentials.
- One-time enrollment through the loopback API. The initial enrollment file is private and removed after the node is online. Owner keys are never embedded in generated commands or printed by setup. Setup prints a separate command you can run privately on the VM to read your owner key.
- Saved dashboard and workspace origins, node pairing instructions, automatic service restarts and certificate renewals. The public URL in Settings is managed by startup configuration; rerun cloud setup on the VM to change it.
- Public HTTPS verification and an online-node check before reporting success. Firewall rules and cloud account resources remain under your control. This creates a new cloud installation; it does not migrate the local lab you are viewing.

The standard installation uses a trusted host agent with rootful Docker access, which is effectively administrative access to the VM. Use a dedicated VM. For a rootless or custom installation, use the existing manual deployment rather than this installer. Workspaces remain non-root, resource-limited containers; see [security and operational limits](security.md).

GPU monitoring and workspace access use the same node capabilities as a local installation. Install the vendor's host drivers/container runtime first. NVIDIA, AMD and Intel support depends on the device and runtime. The setup does not turn a CPU instance into a GPU instance or install drivers automatically. Mac GPU monitoring remains available through separately paired Mac nodes.

## HTTPS without buying a domain

[sslip.io / nip.io](https://nip.io/) provides public DNS hostnames derived from an IP. The generated dashboard uses `lab.<dashed-ip>.sslip.io`; browser workspaces use `w-<workspace-id>.<dashed-ip>.sslip.io`. Raw HTTP IP access only redirects: enter owner keys at the HTTPS dashboard. Direct `https://PUBLIC_IP` is not the supported entry address.

Caddy obtains individual certificates, including one on first opening each browser workspace. Its internal permission endpoint authorizes only the configured dashboard and registered browser workspaces. Unknown names are denied and the endpoint is blocked through the public gateway. Deleted workspaces cannot obtain new certificates. Each app keeps its own browser origin and authenticated app session.

First access can take longer while a certificate is issued. If a launch ticket expires during issuance, reopen the workspace from the dashboard. The DNS and certificate services have availability and issuance limits; certificates are persisted across restarts to avoid repeated issuance. No wildcard certificate or DNS account is required for the IP-based path.

To use your own domain, create `lab.cloud.example.com` and `*.cloud.example.com` A records pointing to the VM, then add `--domain cloud.example.com` to setup. The same per-workspace certificates and restrictions apply. DNS must point directly to this gateway, or an upstream proxy must forward both app traffic and ACME challenges correctly. Existing home installations can still use [DuckDNS](duckdns.md) or a VPN.

## Update, diagnose, and stop

Rerun the same setup command from the same project folder to build an updated version. Existing coordinator volumes and node pairing data are retained. Configuration validation and image builds complete before replacing live deployment files. The setup uses a fixed Compose project and one host compute service; use one cloud installation per VM. If another coordinator owns port 8088, or another web server owns 80/443, resolve that conflict first.

Keep the source folder and generated `.cloudlab/cloud-access` configuration in place. A public IP change also changes IP-based HTTPS addresses. Reserve a static address; after a deliberate change, rerun with `--public-ip NEW_PUBLIC_IP`, sign in at the new address, and pair remote nodes against that address again. The VM's local compute connection stays on loopback. Do not automatically rotate an existing lab URL in the background.

From the project folder:

```sh
# Check services and gateway errors.
sudo docker compose -p cloudlab-cloud -f .cloudlab/cloud-access/compose.yaml ps
sudo docker compose -p cloudlab-cloud -f .cloudlab/cloud-access/compose.yaml logs --tail 80 gateway coordinator
sudo journalctl -u cloudlab-cloud-agent.service -n 80

# Stop hosting and the agent; keep lab data, app storage, and certificates.
sudo systemctl disable --now cloudlab-cloud-agent.service
sudo docker compose -p cloudlab-cloud -f .cloudlab/cloud-access/compose.yaml down
```

Stop your workspaces in CloudLab before stopping the agent if you also want their containers stopped. Workspace containers and volumes are independent of the coordinator stack. Never delete coordinator, certificate or workspace volumes as a troubleshooting shortcut. Back up the coordinator volume and agent pairing directory together. If setup reports that local services are running but public HTTPS is not ready, check IP/DNS, TCP 80/443, and gateway logs, then rerun; do not bypass certificate validation. Verify access from another network before sharing the lab. A revoked local node needs deliberate re-pairing; rerunning setup does not undo revocation.

Provider references: [AWS IMDSv2](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/configuring-instance-metadata-service.html), [EC2 Elastic IPs](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/elastic-ip-addresses-eip.html), [Google Cloud metadata](https://docs.cloud.google.com/compute/docs/metadata/querying-metadata), [Azure instance metadata](https://learn.microsoft.com/en-us/azure/virtual-machines/instance-metadata-service), [DigitalOcean metadata](https://docs.digitalocean.com/products/droplets/how-to/access-metadata/), [Caddy certificate permissions](https://caddyserver.com/docs/caddyfile/options#on-demand-tls).
