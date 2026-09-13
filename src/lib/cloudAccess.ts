export const cloudProviders = [
  {
    id: "aws",
    name: "AWS EC2",
    address: "Elastic IP",
    firewall: "EC2 security group",
  },
  {
    id: "gcp",
    name: "Google Cloud",
    address: "Static external IP",
    firewall: "VPC firewall rule",
  },
  {
    id: "azure",
    name: "Microsoft Azure",
    address: "Static public IP",
    firewall: "Network security group",
  },
  {
    id: "digitalocean",
    name: "DigitalOcean",
    address: "Reserved IP",
    firewall: "Cloud Firewall",
  },
  {
    id: "other",
    name: "Another provider",
    address: "Static public IPv4",
    firewall: "Cloud firewall",
  },
] as const;

export function publicIp(value: string): string | null {
  const parts = value.trim().split(".");
  if (
    parts.length !== 4 ||
    parts.some((p) => !/^(0|[1-9]\d{0,2})$/.test(p) || +p > 255)
  )
    return null;
  const [a, b, c, d] = parts.map(Number);
  if (
    a === 0 ||
    a === 10 ||
    a === 127 ||
    a >= 224 ||
    (a === 100 && b >= 64 && b <= 127) ||
    (a === 169 && b === 254) ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 &&
      (b === 168 ||
        (b === 0 && (c === 2 || (c === 0 && d !== 9 && d !== 10))))) ||
    (a === 198 && (b === 18 || b === 19 || (b === 51 && c === 100))) ||
    (a === 203 && b === 0 && c === 113)
  )
    return null;
  return parts.join(".");
}

export function cloudCommand(
  provider: string,
  address: string,
  automatic: boolean,
): string | null {
  if (!cloudProviders.some((p) => p.id === provider)) return null;
  if (automatic && provider !== "other")
    return `sudo python3 scripts/cloud-access.py --provider ${provider} --apply`;
  const ip = publicIp(address);
  return ip
    ? `sudo python3 scripts/cloud-access.py --provider ${provider} --public-ip ${ip} --apply`
    : null;
}
