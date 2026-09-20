<script lang="ts">
  import { Cpu } from "lucide-svelte";
  import type { Workspace, Node } from "./types";
  let { workspace, node }: { workspace: Workspace; node?: Node } = $props();
  const supported = $derived(
    node?.gpus?.filter((g) => ["nvidia", "dri"].includes(g.access)) ?? [],
  );
  const assigned = $derived(
    supported.filter((g) => workspace.gpu_ids?.includes(g.id)),
  );
  const note = $derived(
    !node
      ? "Reconnect the compute node to check GPU availability."
      : assigned.length
        ? `${assigned.map((g) => g.name).join(", ")} assigned. Install a compatible GPU-enabled library in Python packages to use it.`
        : supported.length
          ? "No GPU is assigned. Stop the workspace, then select an available GPU in Edit workspace."
          : node.gpus?.length
            ? node.gpus[0].access_note ||
              "This node’s GPU is available for monitoring only."
            : "No compatible GPU detected. Check the node’s GPU drivers and container runtime.",
  );
</script>

<div class="gpu-access-note">
  <Cpu size={16} /><span
    ><strong
      >GPU {assigned.length
        ? "assigned"
        : "unavailable to this workspace"}</strong
    ><small>{note}</small></span
  >
</div>
