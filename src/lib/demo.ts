import type { Snapshot } from "./types";
const now = Math.floor(Date.now() / 1000);
export const demoData: Snapshot = {
  role: "owner",
  labs: [
    {
      id: "lab-home",
      name: "Home lab",
      description: "A little space for big ideas.",
    },
    {
      id: "lab-edge",
      name: "Edge experiments",
      description: "Small devices. Interesting possibilities.",
    },
  ],
  nodes: [
    {
      id: "n1",
      lab_id: "lab-home",
      name: "Atlas workstation",
      platform: "Linux",
      arch: "x86_64",
      cpus: 16,
      memory_mb: 65536,
      cpu_usage: 32,
      memory_used_mb: 19456,
      last_seen: now,
      docker: true,
      revoked: false,
    },
    {
      id: "n2",
      lab_id: "lab-home",
      name: "Mac mini",
      platform: "macOS",
      arch: "aarch64",
      cpus: 10,
      memory_mb: 16384,
      cpu_usage: 18,
      memory_used_mb: 6144,
      last_seen: now,
      docker: true,
      revoked: false,
    },
    {
      id: "n3",
      lab_id: "lab-home",
      name: "Raspberry Pi 5",
      platform: "Linux",
      arch: "aarch64",
      cpus: 4,
      memory_mb: 8192,
      cpu_usage: 8,
      memory_used_mb: 1024,
      last_seen: now,
      docker: true,
      revoked: false,
    },
    {
      id: "n4",
      lab_id: "lab-home",
      name: "ThinkPad",
      platform: "Linux",
      arch: "x86_64",
      cpus: 8,
      memory_mb: 16384,
      cpu_usage: 0,
      memory_used_mb: 0,
      last_seen: now - 86400,
      docker: true,
      revoked: false,
    },
  ],
  workspaces: [
    {
      id: "w1",
      lab_id: "lab-home",
      node_id: "n1",
      name: "Deep learning playground",
      template: "jupyter",
      cpus: 4,
      memory_mb: 8192,
      status: "running",
      created_at: now - 14400,
      last_used: now,
      error: "",
      network: false,
    },
    {
      id: "w2",
      lab_id: "lab-home",
      node_id: "n2",
      name: "Weekend project",
      template: "code",
      cpus: 2,
      memory_mb: 4096,
      status: "running",
      created_at: now - 9000,
      last_used: now,
      error: "",
      network: false,
    },
    {
      id: "w3",
      lab_id: "lab-home",
      node_id: "n3",
      name: "Sensor experiments",
      template: "terminal",
      cpus: 1,
      memory_mb: 1024,
      status: "stopped",
      created_at: now - 86400,
      last_used: now - 4000,
      error: "",
      network: false,
    },
  ],
  events: [
    {
      id: "e1",
      lab_id: "lab-home",
      message: "Deep learning playground started",
      at: now - 180,
      kind: "workspace",
    },
    {
      id: "e2",
      lab_id: "lab-home",
      message: "Mac mini connected to Home lab",
      at: now - 780,
      kind: "node",
    },
    {
      id: "e3",
      lab_id: "lab-home",
      message: "Sensor experiments stopped",
      at: now - 3600,
      kind: "workspace",
    },
  ],
  settings: {
    name: "My CloudLab",
    session_hours: 12,
    idle_minutes: 60,
    max_workspaces: 12,
    default_cpus: 2,
    default_memory_mb: 2048,
    allow_network: false,
    public_url: "",
  },
  access: [],
};

// Clearly marked preview data follows the same telemetry contract as real agents.
const previewGpu = (
  id: string,
  name: string,
  vendor: string,
  access: string,
) => ({
  id,
  name,
  vendor,
  access,
  utilization: 0,
  memory_used_mb: 0,
  memory_total_mb: vendor === "Apple" ? null : 24576,
  temperature_c: vendor === "Apple" ? null : 54,
  power_watts: vendor === "Apple" ? null : 112,
  shared_memory: vendor === "Apple",
  access_note:
    vendor === "Apple"
      ? "Mac GPU detected. Docker Desktop does not expose Metal GPUs to these Linux workspaces."
      : "NVIDIA compute access. The workspace image also needs compatible CUDA libraries.",
});
demoData.nodes[0].gpus = [
  previewGpu("GPU-demo-atlas", "NVIDIA GeForce RTX 4090", "NVIDIA", "nvidia"),
];
demoData.nodes[1].gpus = [
  previewGpu("mac-apple-m2", "Apple M2 GPU", "Apple", "none"),
];
demoData.workspaces[0].gpu_ids = ["GPU-demo-atlas"];
const offlineDemoNodes = new Set(
  demoData.nodes
    .filter((node) => now - node.last_seen >= 45)
    .map((node) => node.id),
);
export function advanceDemo(data: Snapshot, at: number) {
  for (const [index, resource] of [
    ...data.nodes,
    ...data.workspaces,
  ].entries()) {
    const active =
      "last_seen" in resource
        ? !resource.revoked && !offlineDemoNodes.has(resource.id)
        : resource.status === "running";
    if (!active) continue;
    const phase = at / 45 + index * 2;
    const cpu = Math.max(
      2,
      Math.min(
        98,
        30 + index * 4 + Math.sin(phase) * 13 + Math.sin(phase * 2.7) * 6,
      ),
    );
    const gpus =
      "gpus" in resource
        ? (resource.gpus ?? []).map((g) => ({
            ...g,
            utilization: Math.max(0, 43 + Math.sin(phase / 2) * 31),
            memory_used_mb:
              g.vendor === "Apple"
                ? 840
                : 6700 + Math.round(Math.sin(phase) * 450),
          }))
        : [];
    const m = {
      at,
      cpu_usage: cpu,
      memory_used_mb: Math.round(
        resource.memory_mb * (0.29 + Math.sin(phase / 3) * 0.055),
      ),
      memory_total_mb: resource.memory_mb,
      network_rx_bytes: Math.floor(
        (at % 1e7) * (170000 + index * 37000) + Math.sin(phase) * 1200000,
      ),
      network_tx_bytes: Math.floor((at % 1e7) * (60000 + index * 7000)),
      disk_read_bytes: Math.floor(at * 24000),
      disk_write_bytes: Math.floor(at * 14000),
      pids: 14 + index * 2,
      gpus,
    };
    resource.metrics = m;
    resource.history ??= [];
    if (at - (resource.history.at(-1)?.at ?? 0) >= 10) resource.history.push(m);
    resource.history = resource.history
      .filter((m) => m.at > at - 3600)
      .slice(-360);
    if ("last_seen" in resource) {
      resource.last_seen = at;
      resource.gpus = gpus;
      resource.cpu_usage = cpu;
      resource.memory_used_mb = m.memory_used_mb;
    }
  }
}
for (let offset = 3590; offset >= 0; offset -= 10)
  advanceDemo(demoData, now - offset);
