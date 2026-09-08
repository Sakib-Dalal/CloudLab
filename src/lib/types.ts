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
  labs: Lab[];
  nodes: Node[];
  workspaces: Workspace[];
  events: LabEvent[];
  settings: Settings;
  access: Access[];
  role: string;
};
