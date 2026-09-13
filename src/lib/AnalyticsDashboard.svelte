<script lang="ts">
  import {
    Activity,
    Cpu,
    MemoryStick,
    Microchip,
    Pause,
    Play,
    Radio,
    Clock3,
    Box,
  } from "lucide-svelte";
  import MetricChart from "./MetricChart.svelte";
  import {
    counterRate,
    fresh,
    memory,
    palette,
    percent,
    points,
    rate,
    samples,
    type Series,
  } from "./metrics";
  import type { Node, Workspace, Metrics } from "./types";
  let {
    nodes,
    workspaces,
    clock,
    mode = "nodes",
    demo = false,
    selected = $bindable(""),
  }: {
    nodes: Node[];
    workspaces: Workspace[];
    clock: number;
    mode?: "nodes" | "workspaces";
    demo?: boolean;
    selected?: string;
  } = $props();
  let range = $state(900),
    includeGpu = $state(true),
    paused = $state(false);
  let frozen = $state<{
    nodes: Node[];
    workspaces: Workspace[];
    clock: number;
  } | null>(null);
  const currentNodes = $derived(paused && frozen ? frozen.nodes : nodes);
  const currentWorkspaces = $derived(
    paused && frozen ? frozen.workspaces : workspaces,
  );
  const end = $derived(paused && frozen ? frozen.clock : clock);
  const all = $derived(mode === "nodes" ? currentNodes : currentWorkspaces);
  const resources = $derived(all.filter((r) => !selected || r.id === selected));
  const selectedNodes = $derived(
    currentNodes.filter((n) =>
      mode === "nodes"
        ? !selected || n.id === selected
        : resources.some((w) => "node_id" in w && w.node_id === n.id),
    ),
  );
  const devices = $derived(
    selectedNodes.flatMap((n) =>
      (n.gpus ?? [])
        .filter(
          (g) =>
            mode === "nodes" ||
            resources.some(
              (r) =>
                "gpu_ids" in r &&
                r.node_id === n.id &&
                r.gpu_ids?.includes(g.id),
            ),
        )
        .map((g) => ({ ...g, node: n, key: `${n.id}/${g.id}` })),
    ),
  );
  const live = $derived(
    resources.filter(
      (r) =>
        fresh(r.metrics, end) &&
        ("last_seen" in r
          ? end - r.last_seen < 45
          : r.status === "running" &&
            currentNodes.some(
              (n) => n.id === r.node_id && end - n.last_seen < 45,
            )),
    ),
  );
  const totalCpus = $derived(live.reduce((a, r) => a + r.cpus, 0));
  const cpu = $derived(
    totalCpus
      ? live.reduce((a, r) => a + r.metrics!.cpu_usage * r.cpus, 0) / totalCpus
      : null,
  );
  const usedMemory = $derived(
    live.length
      ? live.reduce((a, r) => a + r.metrics!.memory_used_mb, 0)
      : null,
  );
  const totalMemory = $derived(live.reduce((a, r) => a + r.memory_mb, 0));
  const running = $derived(
    currentWorkspaces.filter(
      (w) =>
        w.status === "running" &&
        currentNodes.some((n) => n.id === w.node_id && end - n.last_seen < 45),
    ),
  );
  const statusCounts = $derived(
    mode === "nodes"
      ? [
          {
            name: "Online",
            count: resources.filter(
              (r) => "last_seen" in r && end - r.last_seen < 45,
            ).length,
            color: palette[0],
          },
          {
            name: "Offline",
            count: resources.filter(
              (r) => "last_seen" in r && end - r.last_seen >= 45,
            ).length,
            color: "#77828e",
          },
        ]
      : [
          {
            name: "Running",
            count: resources.filter((r) => running.some((w) => w.id === r.id))
              .length,
            color: palette[0],
          },
          {
            name: "Stopped",
            count: resources.filter(
              (r) => "status" in r && r.status === "stopped",
            ).length,
            color: "#77828e",
          },
          {
            name: "Attention / pending",
            count: resources.filter(
              (r) =>
                "status" in r &&
                r.status !== "stopped" &&
                !running.some((w) => w.id === r.id),
            ).length,
            color: palette[2],
          },
        ],
  );
  function metricSeries(
    read: (m: Metrics, previous?: Metrics) => number | null,
  ): Series[] {
    return resources.map((r, i) => ({
      name: r.name,
      color: palette[i % palette.length],
      points: points(samples(r.history, r.metrics), read),
    }));
  }
  function togglePause() {
    if (!paused)
      frozen = {
        nodes: structuredClone($state.snapshot(nodes)),
        workspaces: structuredClone($state.snapshot(workspaces)),
        clock,
      };
    paused = !paused;
  }
</script>

<section
  class="analytics-dashboard"
  aria-label={mode === "nodes" ? "Node analytics" : "Workspace analytics"}
>
  <div class="analytics-topline">
    <span
      ><Activity size={15} /> OBSERVABILITY <i>/</i>
      {mode === "nodes" ? "COMPUTE" : "WORKSPACES"}</span
    ><span class="analytics-live" class:paused
      >{demo
        ? paused
          ? "PREVIEW · PAUSED"
          : "SIMULATED PREVIEW"
        : paused
          ? "DISPLAY PAUSED"
          : "LIVE"}<Radio size={12} /></span
    >
  </div>
  <div class="analytics-heading">
    <div>
      <h2>{mode === "nodes" ? "Node analytics" : "Workspace analytics"}</h2>
      <p>
        {mode === "nodes"
          ? "A real-time view of your compute, memory, and accelerators."
          : "Follow resource use and container health across your workspaces."}
      </p>
    </div>
    <div class="analytics-controls">
      <label class="sr-only" for={`resource-${mode}`}>Filter {mode}</label
      ><select id={`resource-${mode}`} bind:value={selected}
        ><option value="">All {mode}</option>{#each all as r}<option
            value={r.id}>{r.name}</option
          >{/each}</select
      ><label class="range-select"
        ><Clock3 size={13} /><select
          aria-label="Chart time range"
          bind:value={range}
          ><option value={300}>Last 5 minutes</option><option value={900}
            >Last 15 minutes</option
          ><option value={3600}>Last hour</option></select
        ></label
      ><button
        class="analytics-pause"
        onclick={togglePause}
        aria-label={paused
          ? "Resume live analytics"
          : "Pause analytics display"}
        title={paused
          ? "Resume live display"
          : "Pause display; collection continues"}
        >{#if paused}<Play size={15} />{:else}<Pause size={15} />{/if}</button
      >
    </div>
  </div>
  <div class="analytics-kpis">
    <article>
      <span><Cpu size={15} /> CPU utilization</span><strong
        >{percent(cpu)}</strong
      ><small
        >{totalCpus}
        {mode === "nodes" ? "online" : "allocated"} cores reporting</small
      >
      <div class="kpi-track">
        <i style={`width:${cpu ?? 0}%;background:${palette[0]}`}></i>
      </div>
    </article>
    <article>
      <span><MemoryStick size={15} /> Memory in use</span><strong
        >{memory(usedMemory)}</strong
      ><small>of {memory(totalMemory)} reporting capacity</small>
      <div class="kpi-track">
        <i
          style={`width:${totalMemory ? ((usedMemory ?? 0) / totalMemory) * 100 : 0}%;background:${palette[1]}`}
        ></i>
      </div>
    </article>
    <article>
      <span
        ><Box size={15} />
        {mode === "nodes" ? "Node availability" : "Workspace health"}</span
      ><strong>{statusCounts[0].count}<em>/ {resources.length}</em></strong
      ><small
        >{statusCounts
          .map((s) => `${s.count} ${s.name.toLowerCase()}`)
          .join(" · ")}</small
      >
      <div class="status-distribution">
        {#each statusCounts as s}<i
            style={`flex:${s.count};background:${s.color}`}
            title={`${s.name}: ${s.count}`}
          ></i>{/each}
      </div>
    </article>
    <article>
      <span><Microchip size={15} /> GPU devices</span><strong
        >{devices.length.toString().padStart(2, "0")}<em
          >{mode === "nodes" ? "detected" : "assigned"}</em
        ></strong
      ><small
        >{mode === "nodes"
          ? `${devices.filter((g) => ["nvidia", "dri"].includes(g.access)).length} support workspace access`
          : "Reserved to selected workspaces"}</small
      >{#if devices.length}<label class="gpu-chart-toggle"
          ><input type="checkbox" bind:checked={includeGpu} />Include GPU charts</label
        >{:else}<small
          >{mode === "nodes"
            ? "Waiting for GPU-capable hardware"
            : "No GPUs assigned to these workspaces"}</small
        >{/if}
    </article>
  </div>
  <div class="analytics-chart-grid">
    <MetricChart
      title="CPU utilization"
      subtitle={mode === "nodes"
        ? "Percentage of each node’s CPU capacity"
        : "Percentage of each workspace’s allocated CPU budget"}
      series={metricSeries((m) => m.cpu_usage)}
      {end}
      {range}
      ceiling={100}
    />
    <MetricChart
      title="Memory utilization"
      subtitle={mode === "nodes"
        ? "Used memory as a percentage of node capacity"
        : "Container memory as a percentage of its limit"}
      series={metricSeries((m) =>
        m.memory_total_mb ? (m.memory_used_mb / m.memory_total_mb) * 100 : null,
      )}
      {end}
      {range}
      ceiling={100}
    />
    <MetricChart
      title="Network throughput"
      subtitle="Receive + transmit · rates between measured samples"
      series={metricSeries((m, p) => {
        const rx = counterRate(m, p, "network_rx_bytes"),
          tx = counterRate(m, p, "network_tx_bytes");
        return rx == null || tx == null ? null : rx + tx;
      })}
      {end}
      {range}
      format={rate}
    />
    {#if mode === "workspaces"}<MetricChart
        title="Disk I/O"
        subtitle="Container block reads + writes · bytes per second"
        series={metricSeries((m, p) => {
          const r = counterRate(m, p, "disk_read_bytes"),
            w = counterRate(m, p, "disk_write_bytes");
          return r == null || w == null ? null : r + w;
        })}
        {end}
        {range}
        format={rate}
      />
    {:else}<article class="allocation-panel">
        <div class="chart-heading">
          <div>
            <h3>Workspace reservations</h3>
            <p>
              CPU capacity assigned to containers, including stopped workspaces
            </p>
          </div>
        </div>
        <div class="allocation-list">
          {#each selectedNodes as n}{@const reserved = currentWorkspaces
              .filter((w) => w.node_id === n.id && w.status !== "deleting")
              .reduce((a, w) => a + w.cpus, 0)}
            <div>
              <span
                ><i class:offline={end - n.last_seen >= 45}></i>{n.name}<strong
                  >{reserved} / {n.cpus} cores</strong
                ></span
              >
              <div class="reservation-track">
                <i
                  style={`width:${Math.min(100, (reserved / Math.max(1, n.cpus)) * 100)}%`}
                ></i>
              </div>
            </div>{:else}<p class="analytics-empty">
              Connect a node to see resource reservations.
            </p>{/each}
        </div>
      </article>{/if}
  </div>
  {#if includeGpu && devices.length}<div class="gpu-section-heading">
      <h3><Microchip size={17} /> GPU telemetry</h3>
      <span>Device-wide readings · shared with host applications</span>
    </div>
    <div class="analytics-chart-grid gpu-grid">
      {#each devices as g}{@const reporting =
          fresh(g.node.metrics, end) && end - g.node.last_seen < 45}
        <div class="gpu-panel">
          <MetricChart
            title={g.name}
            subtitle={`${g.node.name} · ${g.vendor}${g.shared_memory ? " · unified memory" : ""}`}
            series={[
              {
                name: "GPU utilization",
                color: palette[2],
                points: points(
                  samples(g.node.history, g.node.metrics),
                  (m) => m.gpus.find((v) => v.id === g.id)?.utilization ?? null,
                ),
              },
            ]}
            {end}
            {range}
            ceiling={100}
          />
          <div class="gpu-sensors">
            <div>
              <span>{g.shared_memory ? "Shared memory used" : "VRAM used"}</span
              ><strong>{reporting ? memory(g.memory_used_mb) : "—"}</strong>
            </div>
            <div>
              <span>Temperature</span><strong
                >{reporting && g.temperature_c != null
                  ? `${g.temperature_c.toFixed(0)} °C`
                  : "—"}</strong
              >
            </div>
            <div>
              <span>Power</span><strong
                >{reporting && g.power_watts != null
                  ? `${g.power_watts.toFixed(0)} W`
                  : "—"}</strong
              >
            </div>
          </div>
          <p class="gpu-access-note">
            <span class:available={["nvidia", "dri"].includes(g.access)}
              >{["nvidia", "dri"].includes(g.access)
                ? "Workspace capable"
                : "Monitoring only"}</span
            >{g.access_note}
          </p>
        </div>{/each}
    </div>{/if}
  <div class="analytics-footnote">
    <span
      ><i></i>{live.length} / {resources.length}
      {mode} reporting{demo ? " · sample data" : " · refresh every 5s"}</span
    ><span
      >1-hour history in coordinator memory · gaps indicate unavailable samples</span
    >
  </div>
</section>
