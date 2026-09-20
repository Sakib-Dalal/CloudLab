const $ = (id) => document.getElementById(id);
const frame = $("application");
let appPath = "/";
let connecting = false;
let loaded = false;
let loadTimer;

function connected() {
  $("connection-alert").hidden = true;
  $("connection").textContent = "Connected";
  $("connection").className = "connection ready";
}
function failure(message, expired = false) {
  clearMetrics();
  clearTimeout(loadTimer);
  // A temporary relay outage must not unload an editor with unsaved work.
  if (loaded && !expired) {
    $("connection-alert").textContent =
      `${message} Your open editor is preserved; CloudLab will check the connection again.`;
    $("connection-alert").hidden = false;
    $("connection").textContent = "Reconnecting…";
    $("connection").className = "connection failed";
    return;
  }
  loaded = false;
  frame.hidden = true;
  frame.removeAttribute("src");
  $("loading").hidden = false;
  $("loading-title").textContent = expired
    ? "Reconnect from CloudLab"
    : "Connection interrupted";
  $("loading-detail").textContent = message;
  $("retry").hidden = expired;
  $("connection").textContent = "Disconnected";
  $("connection").className = "connection failed";
}
function connectionMessage(error) {
  if (error.name === "TimeoutError")
    return "Your node took too long to respond. Check its connection, then try again.";
  if (error instanceof TypeError)
    return "CloudLab could not reach your workspace. Check the coordinator and node connection.";
  return error.message;
}
async function workspaceInfo() {
  const response = await fetch("/_cloudlab/workspace.json", {
    cache: "no-store",
    signal: AbortSignal.timeout(12000),
  });
  const info = await response.json();
  if (!response.ok) {
    const expired = response.status === 401 || response.status === 403;
    const error = new Error(
      expired
        ? "Return to your CloudLab dashboard and choose Open workspace to start a new session."
        : info.error || "Check your node connection, then try again.",
    );
    error.expired = expired;
    throw error;
  }
  return info;
}
function renderInfo(info) {
  const app = info.template === "jupyter" ? "JupyterLab" : "VS Code";
  document.title = `${info.name} · ${app} · CloudLab`;
  $("workspace-name").textContent = info.name;
  $("app-name").textContent = app;
  $("app-name").className = `app-label ${info.template}`;
  $("lab-name").textContent = info.lab;
  $("node-name").textContent = info.node;
  $("resources").textContent =
    `${info.cpus} CPU ${info.cpus === 1 ? "core" : "cores"} · ${Number((info.memory_mb / 1024).toFixed(1))} GB RAM`;
  $("network").textContent = info.network
    ? "Internet enabled"
    : "Isolated network";
  $("gpu-access-note").textContent = info.gpu_note || "";
  $("footer-app").textContent = `${app} on your compute node`;
  frame.title = `${app} · ${info.name} · CloudLab`;
  appPath = info.path;
  latestInfo = info;
  renderMetrics(info);
}
async function connect() {
  if (connecting) return;
  connecting = true;
  loaded = false;
  clearTimeout(loadTimer);
  frame.hidden = true;
  frame.removeAttribute("src");
  $("loading").hidden = false;
  $("connection-alert").hidden = true;
  $("retry").hidden = true;
  $("reload").disabled = true;
  $("loading-title").textContent = "Connecting to your workspace";
  $("loading-detail").textContent =
    "Preparing your tools on your compute node…";
  $("connection").textContent = "Connecting…";
  $("connection").className = "connection";
  try {
    renderInfo(await workspaceInfo());
    const probe = await fetch(appPath, {
      cache: "no-store",
      signal: AbortSignal.timeout(65000),
    });
    if (!probe.ok)
      throw new Error(
        "The workspace application is not ready. Check your node connection, then try again.",
      );
    await probe.body?.cancel();
    frame.src = appPath;
    loadTimer = setTimeout(
      () =>
        failure(
          "The application is taking longer than expected. Check your compute node and try again.",
        ),
      90000,
    );
  } catch (error) {
    failure(connectionMessage(error), error.expired);
  } finally {
    connecting = false;
    $("reload").disabled = false;
  }
}
frame.addEventListener("load", () => {
  if (!frame.hasAttribute("src")) return;
  try {
    const doc = frame.contentDocument;
    if (!doc || doc.documentElement.hasAttribute("data-cloudlab-error")) {
      failure(
        "Return to CloudLab and reopen this workspace, or check your compute node.",
      );
      return;
    }
  } catch {
    failure("The application left its workspace address. Reload to reconnect.");
    return;
  }
  clearTimeout(loadTimer);
  loaded = true;
  frame.hidden = false;
  $("loading").hidden = true;
  connected();
});
$("retry").addEventListener("click", connect);
$("reload").addEventListener("click", connect);
$("fullscreen").hidden = !document.fullscreenEnabled;
$("fullscreen").addEventListener("click", async () => {
  try {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await document.documentElement.requestFullscreen();
  } catch {
    $("fullscreen").title =
      "Full screen is unavailable in this browser. The workspace already fills this window.";
  }
});
document.addEventListener("fullscreenchange", () => {
  const label = document.fullscreenElement
    ? "Exit full screen"
    : "Enter full screen";
  $("fullscreen").setAttribute("aria-label", label);
  $("fullscreen").querySelector("span").textContent = document.fullscreenElement
    ? "Exit full screen"
    : "Full screen";
});
setInterval(async () => {
  if (!loaded || connecting || document.hidden) return;
  try {
    renderInfo(await workspaceInfo());
    connected();
  } catch (error) {
    failure(connectionMessage(error), error.expired);
  }
}, 5000);

let latestInfo;
const pct = (n) => (n == null ? "—" : `${n.toFixed(1)}%`);
const mem = (n) =>
  n == null
    ? "—"
    : n >= 1024
      ? `${(n / 1024).toFixed(1)} GB`
      : `${Math.round(n)} MB`;
const throughput = (n) =>
  n == null
    ? "—"
    : n >= 1048576
      ? `${(n / 1048576).toFixed(1)} MB/s`
      : `${(n / 1024).toFixed(1)} kB/s`;
function clearMetrics() {
  $("metric-status").textContent = "Telemetry unavailable";
  $("metric-status").classList.add("stale");
  for (const id of [
    "metric-cpu",
    "metric-memory",
    "metric-network",
    "metric-gpu",
    "chart-cpu-value",
    "chart-memory-value",
    "chart-network-value",
    "chart-gpu-value",
  ])
    $(id).textContent = "—";
}
function networkRate(m, previous) {
  if (
    !previous ||
    m.at <= previous.at ||
    m.at - previous.at > 30 ||
    m.network_rx_bytes < previous.network_rx_bytes ||
    m.network_tx_bytes < previous.network_tx_bytes
  )
    return null;
  return (
    (m.network_rx_bytes -
      previous.network_rx_bytes +
      m.network_tx_bytes -
      previous.network_tx_bytes) /
    (m.at - previous.at)
  );
}
function plot(id, series, max, end, range) {
  const svg = $(id);
  svg.replaceChildren();
  const ns = "http://www.w3.org/2000/svg";
  for (const y of [0, 25, 50, 75, 100]) {
    const line = document.createElementNS(ns, "line");
    for (const [k, v] of Object.entries({
      x1: 0,
      x2: 600,
      y1: y,
      y2: y,
      stroke: "#34414c",
      "stroke-width": 0.6,
    }))
      line.setAttribute(k, v);
    svg.append(line);
  }
  for (const [index, data] of series.entries()) {
    let previous;
    const d = data
      .filter((m) => m.at >= end - range && m.at <= end)
      .map((m) => {
        if (m.value == null || !Number.isFinite(m.value)) {
          previous = undefined;
          return "";
        }
        const move = !previous || m.at - previous.at > 30;
        previous = m;
        return `${move ? "M" : "L"}${(((m.at - end + range) / range) * 600).toFixed(1)},${(100 - (Math.max(0, Math.min(max, m.value)) / max) * 100).toFixed(1)}`;
      })
      .join(" ");
    const path = document.createElementNS(ns, "path");
    for (const [k, v] of Object.entries({
      d,
      fill: "none",
      stroke: ["#70d7c7", "#ac9cff", "#f4b56b", "#70b9f1"][index % 4],
      "stroke-width": 2,
      "vector-effect": "non-scaling-stroke",
    }))
      path.setAttribute(k, v);
    svg.append(path);
  }
}
function renderMetrics(info) {
  const end = Date.now() / 1000,
    range = +$("metric-range").value;
  const available =
    info.status === "running" &&
    info.node_online &&
    info.metrics &&
    end - info.metrics.at < 30;
  const m = available ? info.metrics : null;
  const history = [...(info.history || [])];
  if (info.metrics && info.metrics.at > (history.at(-1)?.at ?? 0))
    history.push(info.metrics);
  const network = history.map((m, i) => ({
    at: m.at,
    value: networkRate(m, history[i - 1]),
  }));
  $("metric-status").textContent = !info.node_online
    ? "Node offline"
    : info.status !== "running"
      ? info.status
      : available
        ? "Live metrics"
        : "Awaiting telemetry";
  $("metric-status").classList.toggle("stale", !available);
  $("metric-cpu").textContent = pct(m?.cpu_usage);
  $("metric-memory").textContent =
    `${mem(m?.memory_used_mb)} / ${mem(info.memory_mb)}`;
  $("metric-network").textContent = available
    ? throughput(network.at(-1)?.value)
    : "—";
  $("chart-cpu-value").textContent = pct(m?.cpu_usage);
  $("chart-memory-value").textContent = m
    ? pct((m.memory_used_mb / Math.max(1, info.memory_mb)) * 100)
    : "—";
  $("chart-network-value").textContent = $("metric-network").textContent;
  const gpus = info.gpus || [],
    gpuFresh =
      info.node_online &&
      info.node_metrics_at &&
      end - info.node_metrics_at < 30;
  $("metric-gpu-wrap").hidden = !gpus.length;
  $("include-gpu-label").hidden = !gpus.length;
  $("metric-gpu").textContent = gpuFresh
    ? gpus.map((g) => pct(g.utilization)).join(" / ")
    : "—";
  $("metric-gpu-wrap").title = gpus.map((g) => g.name).join(" / ");
  $("gpu-chart-panel").hidden = !gpus.length || !$("include-gpu").checked;
  $("chart-gpu-value").textContent = $("metric-gpu").textContent;
  $("gpu-legend").textContent = gpus.map((g) => g.name).join(" · ");
  if (!$("metric-charts").hidden) {
    plot(
      "chart-cpu",
      [history.map((m) => ({ at: m.at, value: m.cpu_usage }))],
      100,
      end,
      range,
    );
    plot(
      "chart-memory",
      [
        history.map((m) => ({
          at: m.at,
          value: m.memory_total_mb
            ? (m.memory_used_mb / m.memory_total_mb) * 100
            : null,
        })),
      ],
      100,
      end,
      range,
    );
    plot(
      "chart-network",
      [network],
      Math.max(
        1,
        ...network.filter((m) => m.at >= end - range).map((m) => m.value ?? 0),
      ) * 1.1,
      end,
      range,
    );
    plot(
      "chart-gpu",
      gpus.map((g) =>
        (info.gpu_history || []).map((m) => ({
          at: m.at,
          value: m.gpus.find((v) => v.id === g.id)?.utilization ?? null,
        })),
      ),
      100,
      end,
      range,
    );
    document
      .querySelectorAll(".chart-start")
      .forEach(
        (label) =>
          (label.textContent = new Date(
            (end - range) * 1000,
          ).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })),
      );
  }
}
$("toggle-metrics").addEventListener("click", () => {
  const open = $("metric-charts").hidden;
  $("metric-charts").hidden = !open;
  $("toggle-metrics").setAttribute("aria-expanded", String(open));
  $("toggle-metrics").textContent = open ? "Hide charts ▴" : "Show charts ▾";
  if (latestInfo) renderMetrics(latestInfo);
});
for (const id of ["metric-range", "include-gpu"])
  $(id).addEventListener("change", () => {
    if (latestInfo) renderMetrics(latestInfo);
  });
connect();
