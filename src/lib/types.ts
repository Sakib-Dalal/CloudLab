export type Lab = { id: string; name: string; description: string };
export type Node = {
  id: string;
  lab_id: string;
  name: string;
  platform: string;
  arch: string;
  cpus: number;
  memory_mb: number;
  cpu_usage: number;
  memory_used_mb: number;
  last_seen: number;
  docker: boolean;
  revoked: boolean;
  gpus?: GPU[];
  metrics?: Metrics | null;
  history?: Metrics[];
};
export type Workspace = {
  id: string;
  lab_id: string;
  node_id: string;
  name: string;
  template: string;
  cpus: number;
  memory_mb: number;
  status: string;
  created_at: number;
  last_used: number;
  error: string;
  network: boolean;
  gpu_ids?: string[];
  metrics?: Metrics | null;
  history?: Metrics[];
};
export type LabEvent = {
  id: string;
  lab_id: string;
  message: string;
  at: number;
  kind: string;
};
export type Settings = {
  name: string;
  session_hours: number;
  idle_minutes: number;
  max_workspaces: number;
  default_cpus: number;
  default_memory_mb: number;
  allow_network: boolean;
  public_url: string;
};
export type Access = {
  id: string;
  lab_id: string;
  name: string;
  role: string;
  expires_at: number;
};
export type Snapshot = {
  remote_access?: {
    managed: boolean;
    public_url: string;
    workspace_url: string;
  };
  labs: Lab[];
  nodes: Node[];
  workspaces: Workspace[];
  events: LabEvent[];
  settings: Settings;
  access: Access[];
  role: string;
};

export type GPU = {
  id: string;
  name: string;
  vendor: string;
  utilization: number | null;
  memory_used_mb: number | null;
  memory_total_mb: number | null;
  temperature_c: number | null;
  power_watts: number | null;
  shared_memory: boolean;
  access: string;
  access_note: string;
};
export type Metrics = {
  at: number;
  cpu_usage: number;
  memory_used_mb: number;
  memory_total_mb: number;
  network_rx_bytes: number;
  network_tx_bytes: number;
  disk_read_bytes: number | null;
  disk_write_bytes: number | null;
  pids: number | null;
  gpus: GPU[];
};
