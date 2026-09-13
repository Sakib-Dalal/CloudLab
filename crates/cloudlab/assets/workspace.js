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
  $("footer-app").textContent = `${app} on your compute node`;
  frame.title = `${app} · ${info.name} · CloudLab`;
  appPath = info.path;
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
}, 15000);
connect();
