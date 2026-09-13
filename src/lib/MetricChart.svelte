<script lang="ts">
  import { pathFor, type Series } from "./metrics";
  let {
    title,
    subtitle = "",
    series,
    end,
    range = 900,
    ceiling,
    format = (n: number) => `${n.toFixed(1)}%`,
  }: {
    title: string;
    subtitle?: string;
    series: Series[];
    end: number;
    range?: number;
    ceiling?: number;
    format?: (n: number) => string;
  } = $props();
  let hidden = $state<string[]>([]);
  let cursor = $state<number | null>(null);
  const start = $derived(end - range);
  const visible = $derived(series.filter((s) => !hidden.includes(s.name)));
  const maximum = $derived(
    ceiling ??
      Math.max(
        1,
        ...visible.flatMap((s) =>
          s.points
            .filter((p) => p.at >= start && p.at <= end)
            .map((p) => p.value ?? 0),
        ),
      ) * 1.15,
  );
  const hasData = $derived(
    visible.some((s) =>
      s.points.some((p) => p.at >= start && p.at <= end && p.value != null),
    ),
  );
  const selectedAt = $derived(
    cursor == null ? end : start + (range * cursor) / 100,
  );
  const time = (at: number) =>
    new Date(at * 1000).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  function nearest(s: Series) {
    const all = s.points.filter(
      (p) => p.at >= start && p.at <= end && p.value != null,
    );
    if (!all.length) return null;
    const p = all.reduce((a, b) =>
      Math.abs(b.at - selectedAt) < Math.abs(a.at - selectedAt) ? b : a,
    );
    return Math.abs(p.at - selectedAt) <= 30 ? p.value : null;
  }
</script>

<article class="metric-chart">
  <div class="chart-heading">
    <div>
      <h3>{title}</h3>
      <p>{subtitle}</p>
    </div>
    <span class="chart-unit"
      >{cursor == null ? "USAGE TREND" : time(selectedAt)}</span
    >
  </div>
  <div class="chart-plot">
    <div class="chart-y">
      <span>{format(maximum)}</span><span>{format(maximum / 2)}</span><span
        >{format(0)}</span
      >
    </div>
    <div
      class="plot-inner"
      onpointermove={(event) => {
        const r = event.currentTarget.getBoundingClientRect();
        cursor = Math.max(
          0,
          Math.min(100, ((event.clientX - r.left) / r.width) * 100),
        );
      }}
      onpointerleave={() => (cursor = null)}
      role="group"
      aria-label={`${title}. ${hasData ? "Use the time slider to inspect measurements." : "Waiting for metric samples."}`}
    >
      <svg viewBox="0 0 600 150" preserveAspectRatio="none" aria-hidden="true">
        {#each [0, 37.5, 75, 112.5, 150] as y}<line
            x1="0"
            x2="600"
            y1={y}
            y2={y}
            class="gridline"
          />{/each}
        {#each [0, 150, 300, 450, 600] as x}<line
            x1={x}
            x2={x}
            y1="0"
            y2="150"
            class="gridline vertical"
          />{/each}
        {#each visible as s}<path
            d={pathFor(s.points, start, end, maximum)}
            stroke={s.color}
            fill="none"
            stroke-width="2"
            vector-effect="non-scaling-stroke"
          />{/each}
        {#if cursor != null}<line
            x1={cursor * 6}
            x2={cursor * 6}
            y1="0"
            y2="150"
            class="cursor-line"
          />{/if}
      </svg>
      {#if !hasData}<div class="chart-empty">
          Waiting for samples<span
            >Charts appear as your node reports usage</span
          >
        </div>{/if}
      <input
        class="chart-scrubber"
        type="range"
        min="0"
        max="100"
        step="1"
        value={cursor ?? 100}
        oninput={(event) => (cursor = +event.currentTarget.value)}
        onblur={() => (cursor = null)}
        aria-label={`Inspect ${title} by time`}
      />
    </div>
  </div>
  <div class="chart-x">
    <span>{time(start)}</span><span>{time(start + range / 2)}</span><span
      >{time(end)}</span
    >
  </div>
  <div class="chart-legend">
    {#each series as s}<button
        class:series-hidden={hidden.includes(s.name)}
        aria-pressed={!hidden.includes(s.name)}
        onclick={() =>
          (hidden = hidden.includes(s.name)
            ? hidden.filter((n) => n !== s.name)
            : [...hidden, s.name])}
        ><i style={`background:${s.color}`}></i><span>{s.name}</span><strong
          >{nearest(s) == null ? "—" : format(nearest(s)!)}</strong
        ></button
      >{/each}
  </div>
</article>
