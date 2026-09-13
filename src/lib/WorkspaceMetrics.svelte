<script lang="ts">
  import {
    Activity,
    ChevronDown,
    Cpu,
    MemoryStick,
    Microchip,
  } from "lucide-svelte";
  import MetricChart from "./MetricChart.svelte";
  import {
    fresh,
    memory,
    percent,
    samples,
    points,
    counterRate,
    rate,
    palette,
  } from "./metrics";
  import type { Workspace, Node } from "./types";
  let {
    workspace,
    node,
    clock,
    connected = true,
  }: {
    workspace: Workspace;
    node?: Node;
    clock: number;
    connected?: boolean;
  } = $props();
  let expanded = $state(false),
    includeGpu = $state(true);
  const available = $derived(
    connected &&
      workspace.status === "running" &&
      fresh(workspace.metrics, clock),
  );
  const metrics = $derived(available ? workspace.metrics : null);
  const history = $derived(samples(workspace.history, workspace.metrics));
  const gpus = $derived(
    (node?.gpus ?? []).filter((g) => workspace.gpu_ids?.includes(g.id)),
  );
</script>

<section class="workspace-monitor" aria-label="Workspace live metrics">
  <div class="workspace-monitor-bar">
    <span class="monitor-status" class:stale={!available}
      ><Activity size={14} />{!connected
        ? "Node offline"
        : workspace.status !== "running"
          ? workspace.status
          : available
            ? "Live metrics"
            : "Awaiting telemetry"}</span
    ><span
      ><Cpu size={14} />CPU <strong>{percent(metrics?.cpu_usage)}</strong></span
    ><span
      ><MemoryStick size={14} />RAM
      <strong>{memory(metrics?.memory_used_mb)}</strong><small
        >/ {memory(workspace.memory_mb)}</small
      ></span
    >{#each gpus as g}<span title={`${g.name} · device-wide utilization`}
        ><Microchip size={14} />GPU
        <strong
          >{fresh(node?.metrics, clock) && connected
            ? percent(g.utilization)
            : "—"}</strong
        ><small>device-wide</small></span
      >{/each}<button
      aria-expanded={expanded}
      onclick={() => (expanded = !expanded)}
      >{expanded ? "Hide charts" : "Show charts"}<ChevronDown
        size={14}
        class={expanded ? "rotated" : ""}
      /></button
    >
  </div>
  {#if expanded}{#if gpus.length}<label
        class="gpu-chart-toggle workspace-gpu-toggle"
        ><input type="checkbox" bind:checked={includeGpu} />Include assigned GPU
        charts</label
      >{/if}
    <div class="workspace-monitor-charts analytics-chart-grid">
      <MetricChart
        title="Workspace CPU"
        subtitle={`${workspace.cpus} allocated cores · last 5 minutes`}
        series={[
          {
            name: "CPU",
            color: palette[0],
            points: points(history, (m) => m.cpu_usage),
          },
        ]}
        end={clock}
        range={300}
        ceiling={100}
      /><MetricChart
        title="Workspace memory"
        subtitle="Container usage · last 5 minutes"
        series={[
          {
            name: "RAM",
            color: palette[1],
            points: points(history, (m) => m.memory_used_mb),
          },
        ]}
        end={clock}
        range={300}
        format={memory}
      /><MetricChart
        title="Workspace network"
        subtitle="Receive + transmit · last 5 minutes"
        series={[
          {
            name: "Network",
            color: palette[2],
            points: points(history, (m, p) => {
              const a = counterRate(m, p, "network_rx_bytes"),
                b = counterRate(m, p, "network_tx_bytes");
              return a == null || b == null ? null : a + b;
            }),
          },
        ]}
        end={clock}
        range={300}
        format={rate}
      />{#if includeGpu && gpus.length}{#each gpus as g}<MetricChart
            title={g.name}
            subtitle="Assigned GPU · device-wide · last 5 minutes"
            series={[
              {
                name: "GPU",
                color: palette[2],
                points: points(
                  samples(node?.history, node?.metrics),
                  (m) => m.gpus.find((v) => v.id === g.id)?.utilization ?? null,
                ),
              },
            ]}
            end={clock}
            range={300}
            ceiling={100}
          />{/each}{/if}
    </div>{/if}
</section>
