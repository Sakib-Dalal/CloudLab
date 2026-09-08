<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    FlaskConical,
    LayoutDashboard,
    Server,
    Box,
    ShieldCheck,
    Settings2,
    Plus,
    ChevronDown,
    ArrowUpRight,
    ArrowRight,
    Search,
    CircleHelp,
    Wifi,
    Command,
    Cpu,
    MemoryStick,
    MoreHorizontal,
    Play,
    Square,
    Terminal,
    Code2,
    BookOpen,
    X,
    Check,
    Copy,
    Activity,
    Globe2,
    LockKeyhole,
    LogOut,
    Menu,
    RefreshCw,
    Trash2,
    KeyRound,
    Network,
    CheckCheck,
    AlertCircle,
    ChevronRight,
    Cable,
    SlidersHorizontal,
    ExternalLink,
    Laptop,
  } from "lucide-svelte";
  import { api, APIError } from "./lib/api";
  import { demoData } from "./lib/demo";
  import type { Snapshot, Workspace } from "./lib/types";

  const initialDemo = new URLSearchParams(location.search).has("demo");
  let demo = $state(initialDemo);
  let data = $state<Snapshot | null>(
    initialDemo ? structuredClone(demoData) : null,
  );
  let loading = $state(!initialDemo),
    loginToken = $state(""),
    error = $state(""),
    toast = $state(""),
    busy = $state(false);
  let page = $state("Overview"),
    selectedLab = $state(""),
    query = $state(""),
    filter = $state("all"),
    mobileNav = $state(false);
  let modal = $state(""),
    modalError = $state(""),
    name = $state(""),
    description = $state(""),
    nodeId = $state(""),
    template = $state("jupyter"),
    cpus = $state(2),
    memory = $state(2048),
    network = $state(false);
  let coordinatorUrl = $state(""),
    showCoordinator = $state(false);
  let dialogElement: HTMLDivElement | undefined = $state(),
    returnFocus: HTMLElement | null = null;
  $effect(() => {
    if (modal) {
      returnFocus = document.activeElement as HTMLElement;
      tick().then(() =>
        dialogElement
          ?.querySelector<HTMLElement>("input, select, button")
          ?.focus(),
      );
    } else {
      returnFocus?.focus();
    }
  });
  let secret = $state(""),
    role = $state("operator"),
    activeWorkspace = $state<Workspace | null>(null),
    command = $state(""),
    output = $state(""),
    settingsDraft = $state<Snapshot["settings"] | null>(null);
  let clock = $state(Date.now() / 1000),
    showLabMenu = $state(false);
  const nav = [
    { label: "Overview", icon: LayoutDashboard },
    { label: "Compute nodes", icon: Server },
    { label: "Workspaces", icon: Box },
    { label: "Activity", icon: Activity },
  ];
  const templates = [
    {
      id: "jupyter",
      name: "JupyterLab",
      detail: "Notebooks & Python",
      icon: BookOpen,
      class: "ochre",
    },
    {
      id: "code",
      name: "Code editor",
      detail: "VS Code in your browser",
      icon: Code2,
      class: "teal",
    },
    {
      id: "terminal",
      name: "Linux console",
      detail: "A clean, isolated shell",
      icon: Terminal,
      class: "rose",
    },
  ];
  const lab = $derived(
    data?.labs.find((l) => l.id === selectedLab) || data?.labs[0],
  );
  const nodes = $derived(
    data?.nodes.filter((n) => n.lab_id === lab?.id && !n.revoked) || [],
  );
  const workspaces = $derived(
    data?.workspaces.filter((w) => w.lab_id === lab?.id) || [],
  );
  const events = $derived(
    data?.events.filter((e) => e.lab_id === lab?.id).reverse() || [],
  );
  const online = $derived(nodes.filter((n) => clock - n.last_seen < 45));
  const running = $derived(
    workspaces.filter(
      (w) => w.status === "running" && online.some((n) => n.id === w.node_id),
    ),
  );
  const availableCpus = $derived(
    online.reduce(
      (sum, n) =>
        sum +
        Math.max(
          0,
          n.cpus -
            workspaces
              .filter((w) => w.node_id === n.id)
              .reduce((a, w) => a + w.cpus, 0),
        ),
      0,
    ),
  );
  const visibleWorkspaces = $derived(
    workspaces.filter(
      (w) =>
        w.name.toLowerCase().includes(query.toLowerCase()) &&
        (filter === "all" || w.status === filter),
    ),
  );
  const owner = $derived(data?.role === "owner");
  const canOperate = $derived(data?.role !== "viewer");
  const mem = (mb: number) => `${Number((mb / 1024).toFixed(1))} GB`;
  const ago = (at: number) => {
    const seconds = Math.max(0, clock - at);
    return seconds < 60
      ? "Just now"
      : seconds < 3600
        ? `${Math.floor(seconds / 60)}m ago`
        : seconds < 86400
          ? `${Math.floor(seconds / 3600)}h ago`
          : `${Math.floor(seconds / 86400)}d ago`;
  };
  const nodeName = (id: string) =>
    nodes.find((n) => n.id === id)?.name || "Unavailable node";
  const templateName = (id: string) =>
    templates.find((t) => t.id === id)?.name || id;
  function notify(message: string) {
    toast = message;
    setTimeout(() => (toast = ""), 4500);
  }
  async function refresh() {
    if (demo) return;
    try {
      data = await api<Snapshot>("/state");
      if (activeWorkspace)
        activeWorkspace =
          data.workspaces.find((w) => w.id === activeWorkspace?.id) || null;
      error = "";
    } catch (e) {
      if (e instanceof APIError && e.status === 401) data = null;
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }
  onMount(() => {
    const bootstrap = new URLSearchParams(location.hash.slice(1)).get(
      "desktop",
    );
    if (bootstrap) {
      history.replaceState(null, "", location.pathname);
      api("/desktop-login", { token: bootstrap })
        .then(refresh)
        .catch((e) => {
          error = e.message;
          loading = false;
        });
    } else refresh();
    const timer = setInterval(() => {
      clock = Date.now() / 1000;
      if (data && demo) {
        for (const node of data.nodes)
          if (clock - node.last_seen < 45) node.last_seen = clock;
      }
      if (data && !demo) refresh();
    }, 5000);
    return () => clearInterval(timer);
  });
  async function login() {
    busy = true;
    error = "";
    try {
      await api("/login", { token: loginToken });
      loginToken = "";
      await refresh();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
    }
  }
  function navigate(value: string) {
    page = value;
    query = "";
    filter = "all";
    mobileNav = false;
    if (value === "Settings" && data) settingsDraft = { ...data.settings };
  }
  function open(kind: string, w?: Workspace) {
    modal = kind;
    modalError = "";
    name = "";
    description = "";
    secret = "";
    activeWorkspace = w || null;
    nodeId = online.find((n) => n.docker)?.id || "";
    cpus = data?.settings.default_cpus || 2;
    memory = data?.settings.default_memory_mb || 2048;
    template = "jupyter";
    network = false;
    output = "";
    command = "";
  }
  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      notify("Copied to clipboard");
    } catch {
      notify("Clipboard unavailable. Select and copy the text.");
    }
  }
  async function submit() {
    busy = true;
    modalError = "";
    try {
      if (demo) {
        if (modal === "workspace") {
          data!.workspaces.push({
            id: crypto.randomUUID(),
            lab_id: lab!.id,
            node_id: nodeId,
            name,
            template,
            cpus,
            memory_mb: memory,
            status: "running",
            created_at: clock,
            last_used: clock,
            error: "",
            network,
          });
          modal = "";
          notify("Demo workspace created");
        } else if (modal === "lab") {
          data!.labs.push({ id: crypto.randomUUID(), name, description });
          selectedLab = data!.labs.at(-1)!.id;
          modal = "";
        } else if (modal === "node" || modal === "access") {
          secret = "DEMO-PREVIEW-ONLY";
        }
        return;
      }
      if (modal === "workspace") {
        await api("/workspaces", {
          lab_id: lab!.id,
          node_id: nodeId,
          name,
          template,
          cpus,
          memory_mb: memory,
          network,
        });
        modal = "";
        notify("Workspace queued. Your node will prepare the container.");
      } else if (modal === "lab") {
        const result = await api("/labs", { name, description });
        selectedLab = result.id;
        modal = "";
      } else if (modal === "node") {
        const result = await api("/enrollments", { lab_id: lab!.id, name });
        secret = result.token;
      } else if (modal === "access") {
        const result = await api("/access", { lab_id: lab!.id, name, role });
        secret = result.token;
      }
      await refresh();
    } catch (e) {
      modalError = (e as Error).message;
    } finally {
      busy = false;
    }
  }
  async function action(w: Workspace, action: string) {
    busy = true;
    try {
      if (demo) {
        if (action === "delete")
          data!.workspaces = data!.workspaces.filter((x) => x.id !== w.id);
        else w.status = action === "start" ? "running" : "stopped";
      } else {
        await api(`/workspaces/${w.id}/actions`, { action });
        await refresh();
      }
      if (modal === "delete") modal = "";
      notify(
        action === "delete"
          ? "Container removal queued. Volume data is retained."
          : `Workspace ${action} requested`,
      );
    } catch (e) {
      notify((e as Error).message);
    } finally {
      busy = false;
    }
  }
  async function consoleRun() {
    if (!command.trim() || !activeWorkspace) return;
    busy = true;
    const text = command;
    command = "";
    output += `\n$ ${text}\n`;
    try {
      if (demo) {
        output +=
          "This is a preview console. Connect a node to run real commands.\n";
        return;
      }
      const job = await api(`/workspaces/${activeWorkspace.id}/actions`, {
        action: "exec",
        command: text,
      });
      for (let i = 0; i < 70; i++) {
        await new Promise((r) => setTimeout(r, 1000));
        const state = await api(`/jobs/${job.id}`);
        if (state.status === "done" || state.status === "failed") {
          output += (state.output || state.error || "(no output)") + "\n";
          return;
        }
      }
      output +=
        "The command is still pending. Check node connectivity and Activity.\n";
    } catch (e) {
      output += (e as Error).message + "\n";
    } finally {
      busy = false;
    }
  }
  async function openApp(w: Workspace) {
    if (w.template === "terminal") {
      open("console", w);
      return;
    }
    if (demo) {
      notify("Preview only. Connect a real node to open this workspace.");
      return;
    }
    const tab = window.open("about:blank", "_blank");
    if (tab) tab.opener = null;
    try {
      const result = await api(`/workspaces/${w.id}/open`, {});
      if (tab) tab.location.replace(result.url);
      else location.assign(result.url);
    } catch (e) {
      tab?.close();
      notify((e as Error).message);
    }
  }
  async function saveSettings() {
    if (!settingsDraft) return;
    busy = true;
    try {
      if (demo) data!.settings = { ...settingsDraft };
      else {
        await api("/settings", settingsDraft, "PUT");
        await refresh();
      }
      notify("Settings saved");
    } catch (e) {
      notify((e as Error).message);
    } finally {
      busy = false;
    }
  }
  async function revoke(id: string, kind: string) {
    try {
      if (demo) {
        notify("Preview only");
        return;
      }
      await api(`/${kind}/${id}`, {}, "DELETE");
      await refresh();
      notify("Access revoked");
      modal = "";
    } catch (e) {
      notify((e as Error).message);
    }
  }
  async function logout() {
    try {
      if (!demo) await api("/logout", {});
    } finally {
      location.href = "/";
    }
  }
</script>

<svelte:head
  ><title>{data ? `${page} · CloudLab` : "Welcome · CloudLab"}</title
  ></svelte:head
>

{#if loading}
  <main class="auth-screen">
    <div class="brand-mark"><FlaskConical size={30} /></div>
    <p>Connecting to your lab…</p>
  </main>
{:else if !data}
  <main class="auth-screen">
    <div class="auth-card">
      <div class="wordmark">
        <span class="brand-mark"><FlaskConical size={25} /></span>CloudLab<span
          class="version">2.0</span
        >
      </div>
      <div class="eyebrow">YOUR SPACE TO EXPERIMENT</div>
      <h1>A whole lab.<br />Anywhere you are.</h1>
      <p>
        Connect your computers. Create isolated workspaces. Keep your work on
        your own hardware.
      </p>
      <form
        onsubmit={(e) => {
          e.preventDefault();
          login();
        }}
      >
        <label for="token">Lab access key</label><input
          id="token"
          type="password"
          bind:value={loginToken}
          placeholder="Paste your access key"
          required
          autocomplete="current-password"
        />{#if error}<div class="inline-error" role="alert">
            {error}
          </div>{/if}<button class="primary full" disabled={busy}
          >{busy ? "Connecting…" : "Enter your lab"}<ArrowRight
            size={18}
          /></button
        >
      </form>
      <p class="hint">
        First time? Start the coordinator and use the key saved in <code
          >.cloudlab/admin-token</code
        >.
      </p>
      <button
        class="text-button"
        onclick={() => {
          demo = true;
          data = structuredClone(demoData);
          error = "";
        }}>Explore the demo lab <ArrowUpRight size={16} /></button
      >
      <button
        class="text-button connect-other"
        onclick={() => (showCoordinator = !showCoordinator)}
        ><Globe2 size={15} />Connect to another coordinator</button
      >
      {#if showCoordinator}<form
          onsubmit={(e) => {
            e.preventDefault();
            try {
              const url = new URL(coordinatorUrl);
              if (
                url.protocol !== "https:" &&
                !(
                  url.protocol === "http:" &&
                  ["localhost", "127.0.0.1"].includes(url.hostname)
                )
              )
                throw new Error("Use HTTPS for a remote coordinator.");
              location.assign(url.origin);
            } catch (e) {
              error = (e as Error).message;
            }
          }}
        >
          <label for="coordinator">Coordinator address</label><input
            id="coordinator"
            type="url"
            bind:value={coordinatorUrl}
            placeholder="https://lab.example.com"
            required
          /><button class="secondary full"
            >Connect <ArrowRight size={15} /></button
          >
        </form>{/if}
    </div>
    <div class="auth-note">
      <ShieldCheck size={18} /> Local by design. Isolated by default.
    </div>
  </main>
{:else}
  <div class="app-shell" inert={!!modal}>
    <aside class:mobile-open={mobileNav}>
      <a
        class="wordmark"
        href="/"
        onclick={(e) => {
          e.preventDefault();
          navigate("Overview");
        }}
        ><span class="brand-mark"><FlaskConical size={24} /></span>CloudLab<span
          class="version">2.0</span
        ></a
      >
      <div class="lab-picker">
        <button class="lab-button" onclick={() => (showLabMenu = !showLabMenu)}
          ><span class="lab-avatar">{lab?.name.slice(0, 1) || "L"}</span><span
            ><strong>{lab?.name || "Your first lab"}</strong><small
              >Personal lab</small
            ></span
          ><ChevronDown size={16} /></button
        >
        {#if showLabMenu}<div class="lab-menu">
            {#each data.labs as item}<button
                onclick={() => {
                  selectedLab = item.id;
                  showLabMenu = false;
                }}
                >{item.name}{#if lab?.id === item.id}<Check
                    size={14}
                  />{/if}</button
              >{/each}{#if owner}<button
                onclick={() => {
                  open("lab");
                  showLabMenu = false;
                }}><Plus size={15} /> Create a lab</button
              >{/if}
          </div>{/if}
      </div>
      <div class="nav-label">WORKBENCH</div>
      <nav>
        {#each nav as item}<button
            class:active={page === item.label}
            onclick={() => navigate(item.label)}
            ><item.icon
              size={19}
            />{item.label}{#if item.label === "Workspaces"}<span
                class="nav-count">{workspaces.length}</span
              >{/if}</button
          >{/each}
      </nav>
      <div class="nav-label management-label">MANAGE</div>
      <nav>
        <button
          class:active={page === "Access & security"}
          onclick={() => navigate("Access & security")}
          ><ShieldCheck size={19} />Access & security</button
        ><button
          class:active={page === "Settings"}
          onclick={() => navigate("Settings")}
          ><Settings2 size={19} />Lab settings</button
        >
      </nav>
      <div class="sidebar-bottom">
        <div class="local-card">
          <span class="status-dot"></span><strong
            >Your hardware. Your lab.</strong
          >
          <p>Hosted by you, for you.</p>
          <span><LockKeyhole size={13} /> Container access only</span>
        </div>
        <button class="help-link" onclick={() => open("help")}
          ><CircleHelp size={18} /> A little help <ArrowUpRight
            size={15}
          /></button
        ><button class="profile" onclick={logout}
          ><span class="profile-avatar">{demo ? "D" : owner ? "O" : "M"}</span
          ><span
            ><strong
              >{demo
                ? "Demo explorer"
                : owner
                  ? "Lab owner"
                  : "Lab member"}</strong
            ><small>{demo ? "Preview environment" : data.role}</small></span
          ><LogOut size={16} /></button
        >
      </div>
    </aside>
    <div class="main-shell">
      {#if demo}<div class="demo-banner">
          <FlaskConical size={14} /><span
            >Demo lab · Sample devices and workspaces</span
          ><button
            onclick={() => {
              location.href = "/";
            }}>Connect to your lab <ArrowRight size={14} /></button
          >
        </div>{/if}
      <header>
        <div class="breadcrumbs">
          <button
            class="mobile-toggle icon-button"
            aria-label="Toggle navigation"
            onclick={() => (mobileNav = !mobileNav)}><Menu size={21} /></button
          ><span>{lab?.name || "CloudLab"}</span><ChevronRight
            size={14}
          /><strong>{page}</strong>
        </div>
        <div class="header-right">
          <span class="connection-pill" class:disconnected={!!error}
            ><span class="status-dot"></span>{demo
              ? "Preview"
              : error
                ? "Reconnecting"
                : "Coordinator connected"}</span
          ><button
            class="icon-button"
            aria-label="Help"
            onclick={() => open("help")}><CircleHelp size={19} /></button
          >
        </div>
      </header>
      <main class="content">
        {#if error}<div class="inline-error" role="alert">
            {error}
            <button class="text-button" onclick={refresh}
              >Retry connection</button
            >
          </div>{/if}
        <div class="page-heading">
          <div>
            <div class="eyebrow">
              {page === "Overview"
                ? "MAKE ROOM FOR YOUR NEXT IDEA"
                : "YOUR PERSONAL COMPUTE LAB"}
            </div>
            <h1>
              {page === "Overview"
                ? "Your lab, at a glance."
                : page === "Compute nodes"
                  ? "A place for every device."
                  : page === "Workspaces"
                    ? "Pick up where you left off."
                    : page === "Access & security"
                      ? "Open doors. Clear boundaries."
                      : page === "Settings"
                        ? "Make this lab yours."
                        : "What’s happening in your lab."}
            </h1>
            <p>
              {page === "Overview"
                ? lab?.description ||
                  "Bring your hardware together. Start something new."
                : page === "Compute nodes"
                  ? "Bring your computers together in one connected lab."
                  : page === "Workspaces"
                    ? "Your tools and projects, each in their own container."
                    : page === "Access & security"
                      ? "Decide who can enter your lab and what they can do."
                      : page === "Settings"
                        ? "Set the defaults for your workspaces, resources, and connections."
                        : "A record of connections, workspace changes, and access."}
            </p>
          </div>
          {#if (page === "Overview" || page === "Workspaces") && canOperate}<button
              class="primary"
              onclick={() => open("workspace")}
              disabled={!online.some((n) => n.docker)}
              ><Plus size={18} />New workspace</button
            >{:else if page === "Compute nodes" && owner}<button
              class="primary"
              onclick={() => open("node")}
              ><Plus size={18} />Connect a node</button
            >{/if}
        </div>

        {#if page === "Overview"}
          <section class="stats-grid" aria-label="Lab overview">
            <div class="stat">
              <span>Connected nodes <Server size={18} /></span>
              <div>
                <strong>{online.length.toString().padStart(2, "0")}</strong
                ><span class="stat-detail">/ {nodes.length} total</span>
              </div>
              <small
                ><span class="status-dot"></span>{online.length
                  ? "Ready for your next experiment"
                  : "Waiting for a connection"}</small
              >
            </div>
            <div class="stat">
              <span>Active workspaces <Box size={18} /></span>
              <div>
                <strong>{running.length.toString().padStart(2, "0")}</strong
                ><span class="stat-detail">/ {workspaces.length} total</span>
              </div>
              <small
                >{workspaces.filter((w) => w.status === "stopped").length} paused
                and ready to resume</small
              >
            </div>
            <div class="stat">
              <span>Available compute <Cpu size={18} /></span>
              <div>
                <strong>{availableCpus}</strong><span class="stat-detail"
                  >CPU cores</span
                >
              </div>
              <small>Across your connected devices</small>
            </div>
            <div class="stat stat-tint">
              <span>Total memory <MemoryStick size={18} /></span>
              <div>
                <strong
                  >{Number(
                    (
                      online.reduce((a, n) => a + n.memory_mb, 0) / 1024
                    ).toFixed(0),
                  )}</strong
                ><span class="stat-detail">GB RAM</span>
              </div>
              <small>On your own hardware</small>
            </div>
          </section>
          <div class="overview-grid">
            <section class="panel nodes-panel">
              <div class="section-heading">
                <div>
                  <h2>
                    Your compute nodes <span class="count">{nodes.length}</span>
                  </h2>
                  <p>Different devices. One connected lab.</p>
                </div>
                <button
                  class="text-button"
                  onclick={() => navigate("Compute nodes")}
                  >View all <ArrowRight size={16} /></button
                >
              </div>
              <div class="node-list">
                {#each nodes.slice(0, 4) as n}<button
                    class="node-row"
                    onclick={() => {
                      nodeId = n.id;
                      activeWorkspace = null;
                      modal = "node-detail";
                    }}
                    ><span
                      class="device-icon"
                      class:offline={clock - n.last_seen >= 45}
                      >{#if n.arch === "aarch64"}<Cpu size={23} />{:else}<Server
                          size={23}
                        />{/if}</span
                    ><span class="node-info"
                      ><strong>{n.name}</strong><small
                        >{n.platform} <span>·</span>
                        {n.cpus} cores <span>·</span>
                        {mem(n.memory_mb)}</small
                      ></span
                    ><span class="node-load"
                      ><span class="load-track"
                        ><i
                          style={`width:${clock - n.last_seen < 45 ? n.cpu_usage : 0}%`}
                        ></i></span
                      ><small
                        >{clock - n.last_seen < 45
                          ? `${Math.round(n.cpu_usage)}%`
                          : "—"}</small
                      ></span
                    ><span class="badge" class:muted={clock - n.last_seen >= 45}
                      ><span class="status-dot"></span>{clock - n.last_seen < 45
                        ? "Online"
                        : "Offline"}</span
                    ><ChevronRight size={16} /></button
                  >{:else}<div class="empty">
                    <Server size={32} />
                    <h3>Your lab starts with a node.</h3>
                    <p>
                      Connect a computer with Docker to create your first
                      workspace.
                    </p>
                    {#if owner}<button
                        class="secondary"
                        onclick={() => open("node")}
                        ><Plus size={16} />Connect a node</button
                      >{/if}
                  </div>{/each}
              </div>
              {#if owner && nodes.length}<button
                  class="add-node"
                  onclick={() => open("node")}
                  ><Plus size={16} />Connect another device<span
                    >Linux, macOS, Windows & Raspberry Pi <ArrowUpRight
                      size={14}
                    /></span
                  ></button
                >{/if}
            </section>
            <section class="isolation-card">
              <div class="shield-box"><ShieldCheck size={27} /></div>
              <span class="mini-label">A SAFER SPACE TO EXPLORE</span>
              <h2>All the possibilities.<br />A little more peace of mind.</h2>
              <p>
                Your workspaces run inside isolated containers. Your personal
                files stay outside the lab.
              </p>
              <div class="isolation-tags">
                <span><Check size={14} />No host folders</span><span
                  ><Check size={14} />Limited resources</span
                ><span><Check size={14} />No privileged containers</span>
              </div>
              <button
                class="text-button"
                onclick={() => navigate("Access & security")}
                >Explore your security settings <ArrowUpRight
                  size={16}
                /></button
              >
            </section>
          </div>
        {/if}

        {#if page === "Overview" || page === "Workspaces"}
          <section class="workspace-section">
            <div class="section-heading">
              <div>
                <h2>
                  {page === "Overview" ? "Your workspaces" : "All workspaces"}
                  <span class="count">{workspaces.length}</span>
                </h2>
                {#if page === "Overview"}<p>
                    A familiar place to get back to work.
                  </p>{/if}
              </div>
              <div class="section-actions">
                <div class="search-input">
                  <Search size={16} /><input
                    aria-label="Search workspaces"
                    placeholder="Find a workspace…"
                    bind:value={query}
                  />
                </div>
                {#if page === "Workspaces"}<select
                    aria-label="Filter workspace status"
                    bind:value={filter}
                    ><option value="all">All statuses</option><option
                      value="running">Running</option
                    ><option value="stopped">Stopped</option><option
                      value="error">Needs attention</option
                    ></select
                  >{/if}
              </div>
            </div>
            <div class="workspace-grid">
              {#each visibleWorkspaces as w}<article class="workspace-card">
                  <div class="workspace-top">
                    <span
                      class={`template-icon ${templates.find((t) => t.id === w.template)?.class}`}
                      >{#if w.template === "jupyter"}<BookOpen
                          size={23}
                        />{:else if w.template === "code"}<Code2
                          size={23}
                        />{:else}<Terminal size={23} />{/if}</span
                    ><span
                      class="badge"
                      class:muted={w.status !== "running"}
                      class:warning={w.status === "error"}
                      ><span class="status-dot"></span>{w.status === "stopped"
                        ? "Stopped"
                        : w.status === "running"
                          ? "Running"
                          : w.status}</span
                    ><button
                      class="icon-button"
                      aria-label={`Manage ${w.name}`}
                      onclick={() => open("workspace-detail", w)}
                      ><MoreHorizontal size={19} /></button
                    >
                  </div>
                  <h3>{w.name}</h3>
                  <p class="workspace-type">
                    {templateName(w.template)}<span>·</span><LockKeyhole
                      size={12}
                    />Isolated container
                  </p>
                  <div class="workspace-node">
                    <Server size={14} />{nodeName(w.node_id)}
                  </div>
                  <div class="resource-pills">
                    <span><Cpu size={14} />{w.cpus} vCPU</span><span
                      ><MemoryStick size={14} />{mem(w.memory_mb)}</span
                    >
                  </div>
                  {#if w.error}<p class="workspace-error">{w.error}</p>{/if}
                  <div class="workspace-footer">
                    <span>{ago(w.last_used)}</span
                    >{#if w.status === "running" && canOperate}<button
                        class="text-button accent"
                        onclick={() => openApp(w)}
                        >Open workspace <ArrowUpRight size={16} /></button
                      >{:else if w.status === "stopped" && canOperate}<button
                        class="text-button"
                        disabled={busy}
                        onclick={() => action(w, "start")}
                        >Resume workspace <Play size={14} /></button
                      >{:else}<button
                        class="text-button"
                        onclick={() => open("workspace-detail", w)}
                        >View details <ArrowRight size={14} /></button
                      >{/if}
                  </div>
                </article>{:else}<div class="empty workspace-empty">
                  <Box size={34} />
                  <h3>
                    {query
                      ? "No matching workspaces"
                      : "Your next idea starts here."}
                  </h3>
                  <p>
                    {query
                      ? "Try another name."
                      : "Connect a node, choose a tool, and give your project a home."}
                  </p>
                  {#if !query && canOperate}<button
                      class="secondary"
                      disabled={!online.some((n) => n.docker)}
                      onclick={() => open("workspace")}
                      ><Plus size={16} />Create a workspace</button
                    >{/if}
                </div>{/each}
            </div>
          </section>
        {/if}

        {#if page === "Overview"}<section class="activity-strip">
            <span class="activity-icon"><Activity size={18} /></span>
            <div>
              <strong>Latest in your lab</strong><span
                >{events[0]?.message ||
                  "Your lab is ready for its first experiment."}</span
              >
            </div>
            <small>{events[0] ? ago(events[0].at) : ""}</small><button
              class="text-button"
              onclick={() => navigate("Activity")}
              >All activity <ArrowRight size={15} /></button
            >
          </section>{/if}

        {#if page === "Compute nodes"}<div class="node-card-grid">
            {#each nodes as n}<article class="panel compute-card">
                <div class="section-heading">
                  <span class="device-icon"><Server size={26} /></span><span
                    class="badge"
                    class:muted={clock - n.last_seen >= 45}
                    ><span class="status-dot"></span>{clock - n.last_seen < 45
                      ? "Online"
                      : "Offline"}</span
                  >
                </div>
                <h2>{n.name}</h2>
                <p>{n.platform} · {n.arch}</p>
                <div class="resource-pills">
                  <span><Cpu size={15} />{n.cpus} cores</span><span
                    ><MemoryStick size={15} />{mem(n.memory_mb)}</span
                  >
                </div>
                <div class="meter-label">
                  <span>CPU usage</span><strong
                    >{clock - n.last_seen < 45
                      ? `${Math.round(n.cpu_usage)}%`
                      : "—"}</strong
                  >
                </div>
                <div class="meter"><i style={`width:${n.cpu_usage}%`}></i></div>
                <div class="meter-label">
                  <span>Memory</span><strong
                    >{clock - n.last_seen < 45
                      ? `${mem(n.memory_used_mb)} / ${mem(n.memory_mb)}`
                      : "—"}</strong
                  >
                </div>
                <div class="meter teal-meter">
                  <i
                    style={`width:${n.memory_mb ? (n.memory_used_mb / n.memory_mb) * 100 : 0}%`}
                  ></i>
                </div>
                <div class="compute-footer">
                  <span
                    ><Box size={14} />{n.docker
                      ? "Docker available"
                      : "Docker unavailable"}</span
                  ><button
                    class="text-button"
                    onclick={() => {
                      nodeId = n.id;
                      modal = "node-detail";
                    }}>Details <ArrowRight size={15} /></button
                  >
                </div>
              </article>{:else}<div class="empty panel">
                <Network size={35} />
                <h3>Bring your hardware together.</h3>
                <p>Each node contributes its compute to this lab.</p>
                {#if owner}<button class="primary" onclick={() => open("node")}
                    ><Plus size={17} />Connect your first node</button
                  >{/if}
              </div>{/each}
          </div>{/if}

        {#if page === "Activity"}<section class="panel activity-panel">
            {#each events as event}<div class="event-row">
                <span class="event-icon"
                  >{#if event.kind === "node"}<Server
                      size={19}
                    />{:else if event.kind === "access"}<KeyRound
                      size={19}
                    />{:else}<Box size={19} />{/if}</span
                >
                <div>
                  <strong>{event.message}</strong><small
                    >{new Date(event.at * 1000).toLocaleString()}</small
                  >
                </div>
                <span>{ago(event.at)}</span>
              </div>{:else}<div class="empty">
                <Activity size={30} />
                <h3>A fresh start.</h3>
                <p>Your lab’s activity will appear here.</p>
              </div>{/each}
          </section>{/if}

        {#if page === "Access & security"}<div class="security-banner">
            <ShieldCheck size={30} />
            <div>
              <h2>Container boundaries are always on.</h2>
              <p>
                No host shell, no host folder mounts, no privileged mode. Every
                workspace has a fixed CPU and memory budget.
              </p>
            </div>
            <span class="badge"><LockKeyhole size={14} />Enforced</span>
          </div>
          <section class="panel">
            <div class="section-heading">
              <div>
                <h2>Lab access</h2>
                <p>
                  Scoped keys for people you trust. Keys expire after 7 days.
                </p>
              </div>
              {#if owner}<button
                  class="secondary"
                  onclick={() => open("access")}
                  ><Plus size={16} />Invite someone</button
                >{/if}
            </div>
            <div class="access-row">
              <span class="profile-avatar">O</span>
              <div>
                <strong>Lab owner</strong><small>Full lab administration</small>
              </div>
              <span class="badge muted">Owner</span>
            </div>
            {#each data.access.filter((a) => a.lab_id === lab?.id) as access}<div
                class="access-row"
              >
                <span class="profile-avatar">{access.name.slice(0, 1)}</span>
                <div>
                  <strong>{access.name}</strong><small
                    >Expires {new Date(
                      access.expires_at * 1000,
                    ).toLocaleDateString()}</small
                  >
                </div>
                <span class="badge muted">{access.role}</span>{#if owner}<button
                    class="icon-button danger"
                    aria-label={`Revoke ${access.name}`}
                    onclick={() => {
                      secret = access.id;
                      modal = "revoke-access";
                    }}><Trash2 size={16} /></button
                  >{/if}
              </div>{/each}{#if !data.access.length}<p class="panel-hint">
                Your lab is private. Invite a viewer to observe, or an operator
                to create and use containers.
              </p>{/if}
          </section>
          <div class="security-grid">
            <section class="panel">
              <LockKeyhole size={23} />
              <h3>Restricted by default</h3>
              <p>
                Non-root containers, a read-only system, dropped Linux
                capabilities, and isolated networks. Container storage stays in
                dedicated Docker volumes.
              </p>
            </section>
            <section class="panel">
              <Globe2 size={23} />
              <h3>Your connection, your choice</h3>
              <p>
                Use a self-hosted HTTPS gateway or WireGuard VPN to reach this
                coordinator. Nodes only need an outbound connection.
              </p>
              <button class="text-button" onclick={() => open("remote")}
                >Set up remote access <ArrowUpRight size={15} /></button
              >
            </section>
          </div>{/if}

        {#if page === "Settings" && settingsDraft}<form
            class="settings-form"
            onsubmit={(e) => {
              e.preventDefault();
              saveSettings();
            }}
          >
            <section class="panel">
              <h2>The essentials</h2>
              <div class="setting-row">
                <div>
                  <label for="labname">Coordinator name</label>
                  <p>A familiar name for your CloudLab installation.</p>
                </div>
                <input
                  id="labname"
                  bind:value={settingsDraft.name}
                  required
                  maxlength="60"
                  disabled={!owner}
                />
              </div>
              <div class="setting-row">
                <div>
                  <label for="publicurl">Public coordinator URL</label>
                  <p>
                    Your HTTPS gateway address, used in pairing instructions.
                  </p>
                </div>
                <input
                  id="publicurl"
                  type="url"
                  bind:value={settingsDraft.public_url}
                  placeholder="https://lab.example.com"
                  disabled={!owner}
                />
              </div>
            </section>
            <section class="panel">
              <h2>Workspace defaults</h2>
              <div class="setting-row">
                <div>
                  <label for="cores">CPU cores per workspace</label>
                  <p>Nodes enforce the limit when creating a container.</p>
                </div>
                <select
                  id="cores"
                  bind:value={settingsDraft.default_cpus}
                  disabled={!owner}
                  >{#each [1, 2, 4, 8, 16] as n}<option value={n}
                      >{n} vCPU</option
                    >{/each}</select
                >
              </div>
              <div class="setting-row">
                <div>
                  <label for="ram">Memory per workspace</label>
                  <p>A hard memory cap, with no additional swap.</p>
                </div>
                <select
                  id="ram"
                  bind:value={settingsDraft.default_memory_mb}
                  disabled={!owner}
                  >{#each [512, 1024, 2048, 4096, 8192, 16384, 32768] as n}<option
                      value={n}>{mem(n)}</option
                    >{/each}</select
                >
              </div>
              <div class="setting-row">
                <div>
                  <label for="max">Maximum workspaces per lab</label>
                  <p>Includes running and stopped workspaces.</p>
                </div>
                <input
                  id="max"
                  type="number"
                  min="1"
                  max="100"
                  bind:value={settingsDraft.max_workspaces}
                  disabled={!owner}
                />
              </div>
              <div class="setting-row">
                <div>
                  <label for="idle">Stop idle workspaces after</label>
                  <p>
                    Based on console and proxied app activity. Long jobs do not
                    count as activity; choose Never for unattended work.
                  </p>
                </div>
                <select
                  id="idle"
                  bind:value={settingsDraft.idle_minutes}
                  disabled={!owner}
                  ><option value={0}>Never</option><option value={30}
                    >30 minutes</option
                  ><option value={60}>1 hour</option><option value={120}
                    >2 hours</option
                  ><option value={480}>8 hours</option></select
                >
              </div>
            </section>
            <section class="panel">
              <h2>Access & connections</h2>
              <div class="setting-row">
                <div>
                  <label for="session">Session duration</label>
                  <p>Applies to new browser sign-ins.</p>
                </div>
                <select
                  id="session"
                  bind:value={settingsDraft.session_hours}
                  disabled={!owner}
                  >{#each [1, 4, 12, 24, 72] as n}<option value={n}
                      >{n} hours</option
                    >{/each}</select
                >
              </div>
              <div class="setting-row">
                <div>
                  <label for="network">Allow workspace internet access</label>
                  <p>
                    Lets new workspaces opt into outbound networking. This also
                    permits routes to your LAN; keep it off for untrusted users.
                  </p>
                </div>
                <input
                  class="switch"
                  id="network"
                  type="checkbox"
                  role="switch"
                  bind:checked={settingsDraft.allow_network}
                  disabled={!owner}
                />
              </div>
            </section>
            <div class="settings-save">
              <span
                >Isolation controls are enforced by the node and cannot be
                disabled here.</span
              ><button class="primary" disabled={!owner || busy}
                ><Check size={17} />{busy ? "Saving…" : "Save changes"}</button
              >
            </div>
          </form>{/if}
        <footer>
          <span
            ><FlaskConical size={14} /> A little lab. A lot of possibilities.</span
          ><span
            >CloudLab 2.0 <span class="footer-dot">·</span> Self-hosted</span
          >
        </footer>
      </main>
    </div>
  </div>
{/if}

{#if modal}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget && !busy) modal = "";
    }}
  >
    <div
      class="modal"
      class:console-modal={modal === "console"}
      bind:this={dialogElement}
      role="dialog"
      aria-modal="true"
      aria-label={modal === "workspace"
        ? "New workspace"
        : modal === "node"
          ? "Connect a node"
          : "Lab dialog"}
      tabindex="-1"
      onkeydown={(e) => {
        if (e.key === "Escape" && !busy) modal = "";
        if (e.key === "Tab") {
          const els = [
            ...e.currentTarget.querySelectorAll<HTMLElement>(
              "button:not(:disabled),input:not(:disabled),select:not(:disabled),textarea,a[href]",
            ),
          ];
          const first = els[0],
            last = els.at(-1);
          if (e.shiftKey && document.activeElement === first) {
            e.preventDefault();
            last?.focus();
          } else if (!e.shiftKey && document.activeElement === last) {
            e.preventDefault();
            first?.focus();
          }
        }
      }}
    >
      <button
        class="modal-close icon-button"
        aria-label="Close dialog"
        onclick={() => (modal = "")}
        disabled={busy}><X size={21} /></button
      >
      {#if ["workspace", "lab", "node", "access"].includes(modal)}
        <div class="modal-symbol"><FlaskConical size={25} /></div>
        <div class="eyebrow">{lab?.name || "CLOUDLAB"}</div>
        <h2>
          {modal === "workspace"
            ? "A fresh space to work."
            : modal === "lab"
              ? "Create a new lab."
              : modal === "node"
                ? "Welcome another device."
                : "Make room for a collaborator."}
        </h2>
        <p class="modal-intro">
          {modal === "workspace"
            ? "Choose your tools and a node. We’ll take care of the container."
            : modal === "lab"
              ? "Group your devices and workspaces around a shared purpose."
              : modal === "node"
                ? "Pair a trusted computer with Docker installed. The key can be used once and expires in 10 minutes."
                : "An access key is limited to this lab. Share it privately with someone you trust."}
        </p>
        {#if secret}<div class="secret-block">
            <strong
              >{modal === "node"
                ? "Run on the device you want to connect"
                : "Copy this access key now"}</strong
            >
            <pre>{modal === "node"
                ? `cloudlab agent --coordinator ${data?.settings.public_url || location.origin.replace("5173", "8088")} --enrollment ${secret}`
                : secret}</pre>
            <button
              class="secondary"
              onclick={() =>
                copy(
                  modal === "node"
                    ? `cloudlab agent --coordinator ${data?.settings.public_url || location.origin.replace("5173", "8088")} --enrollment ${secret}`
                    : secret,
                )}
              ><Copy size={15} />Copy {modal === "node"
                ? "command"
                : "key"}</button
            >
            <p>
              {modal === "node"
                ? "Build and install the Rust agent on the target device first. The device’s credentials stay on that device."
                : "This key is shown once. You can revoke it at any time."}
            </p>
          </div>
          <button class="primary full" onclick={() => (modal = "")}
            >Done <Check size={17} /></button
          >
        {:else}<form
            onsubmit={(e) => {
              e.preventDefault();
              submit();
            }}
          >
            <label for="name"
              >{modal === "node"
                ? "Device name"
                : modal === "access"
                  ? "Person or key name"
                  : modal === "lab"
                    ? "Lab name"
                    : "Workspace name"}</label
            ><input
              id="name"
              bind:value={name}
              placeholder={modal === "node"
                ? "e.g. Studio workstation"
                : modal === "access"
                  ? "e.g. Alex"
                  : modal === "lab"
                    ? "e.g. Robotics lab"
                    : "e.g. My next experiment"}
              required
              minlength="2"
              maxlength="60"
            />
            {#if modal === "lab"}<label for="description">Description</label
              ><input
                id="description"
                bind:value={description}
                maxlength="160"
                placeholder="What will you explore here?"
              />{/if}
            {#if modal === "workspace"}<label for="template-picker"
                >Start with a tool</label
              >
              <div id="template-picker" class="template-picker">
                {#each templates as t}<button
                    type="button"
                    class:selected={template === t.id}
                    onclick={() => (template = t.id)}
                    ><t.icon size={22} /><strong>{t.name}</strong><small
                      >{t.detail}</small
                    >{#if template === t.id}<Check size={14} />{/if}</button
                  >{/each}
              </div>
              <label for="node">Compute node</label><select
                id="node"
                bind:value={nodeId}
                required
                ><option value="" disabled>Select an online node</option
                >{#each online.filter((n) => n.docker) as n}<option value={n.id}
                    >{n.name} · {n.cpus} cores · {mem(n.memory_mb)}</option
                  >{/each}</select
              >
              <div class="form-columns">
                <div>
                  <label for="cpus">CPU budget</label><input
                    id="cpus"
                    type="number"
                    min="1"
                    max={nodes.find((n) => n.id === nodeId)?.cpus || 16}
                    bind:value={cpus}
                    required
                  />
                </div>
                <div>
                  <label for="memory">Memory budget</label><select
                    id="memory"
                    bind:value={memory}
                    >{#each [512, 1024, 2048, 4096, 8192, 16384, 32768] as m}<option
                        value={m}>{mem(m)}</option
                      >{/each}</select
                  >
                </div>
              </div>
              {#if data?.settings.allow_network}<label class="checkbox-label"
                  ><input type="checkbox" bind:checked={network} />Allow
                  outbound network access</label
                >{/if}
              <div class="form-note">
                <ShieldCheck size={17} />Non-root · No host mounts · Resource
                limited
              </div>{/if}
            {#if modal === "access"}<label for="role">Permission</label><select
                id="role"
                bind:value={role}
                ><option value="operator"
                  >Operator — create and use workspaces</option
                ><option value="viewer"
                  >Viewer — view status and activity only</option
                ></select
              >{/if}
            {#if modalError}<div class="inline-error" role="alert">
                {modalError}
              </div>{/if}<button
              class="primary full"
              disabled={busy || (modal === "workspace" && !nodeId)}
              >{busy
                ? "Working…"
                : modal === "workspace"
                  ? "Create workspace"
                  : modal === "lab"
                    ? "Create lab"
                    : modal === "node"
                      ? "Generate pairing command"
                      : "Create access key"}<ArrowRight size={17} /></button
            >
          </form>{/if}
      {:else if modal === "workspace-detail" && activeWorkspace}<div
          class="modal-symbol"
        >
          <Box size={25} />
        </div>
        <h2>{activeWorkspace.name}</h2>
        <p class="modal-intro">
          {templateName(activeWorkspace.template)} on {nodeName(
            activeWorkspace.node_id,
          )}
        </p>
        <div class="detail-grid">
          <span>Status</span><strong>{activeWorkspace.status}</strong><span
            >Resources</span
          ><strong
            >{activeWorkspace.cpus} vCPU · {mem(
              activeWorkspace.memory_mb,
            )}</strong
          ><span>Network</span><strong
            >{activeWorkspace.network ? "Outbound allowed" : "Isolated"}</strong
          ><span>Storage</span><strong>Persistent Docker volume</strong>
        </div>
        {#if activeWorkspace.error}<div class="inline-error">
            {activeWorkspace.error}
          </div>{/if}{#if canOperate}<div class="detail-actions">
            {#if activeWorkspace.status === "running"}<button
                class="primary"
                onclick={() => openApp(activeWorkspace!)}
                ><ArrowUpRight size={17} />Open workspace</button
              ><button
                class="secondary"
                disabled={busy}
                onclick={() => {
                  action(activeWorkspace!, "stop");
                  modal = "";
                }}><Square size={15} />Stop</button
              ><button
                class="secondary"
                onclick={() => open("console", activeWorkspace!)}
                ><Terminal size={17} />Console</button
              >{:else if activeWorkspace.status === "stopped"}<button
                class="primary"
                disabled={busy}
                onclick={() => {
                  action(activeWorkspace!, "start");
                  modal = "";
                }}><Play size={16} />Resume</button
              >{/if}<button
              class="text-button danger"
              onclick={() => (modal = "delete")}
              ><Trash2 size={16} />Remove container</button
            >
          </div>{/if}
      {:else if modal === "delete" && activeWorkspace}<h2>
          Remove this workspace?
        </h2>
        <p class="modal-intro">
          The container for <strong>{activeWorkspace.name}</strong> will be removed.
          Its Docker volume is retained on the node for manual recovery.
        </p>
        <div class="detail-actions">
          <button class="secondary" onclick={() => (modal = "workspace-detail")}
            >Keep workspace</button
          ><button
            class="primary"
            disabled={busy}
            onclick={() => action(activeWorkspace!, "delete")}
            ><Trash2 size={16} />Remove container</button
          >
        </div>
      {:else if modal === "node-detail"}{@const n = nodes.find(
          (n) => n.id === nodeId,
        )}{#if n}<div class="modal-symbol"><Server size={25} /></div>
          <h2>{n.name}</h2>
          <p class="modal-intro">{n.platform} · {n.arch}</p>
          <div class="detail-grid">
            <span>Last heartbeat</span><strong>{ago(n.last_seen)}</strong><span
              >Capacity</span
            ><strong>{n.cpus} cores · {mem(n.memory_mb)}</strong><span
              >Container runtime</span
            ><strong>{n.docker ? "Docker ready" : "Docker unavailable"}</strong
            ><span>Workspaces</span><strong
              >{workspaces.filter((w) => w.node_id === n.id).length}</strong
            ><span>Node ID</span><code>{n.id}</code>
          </div>
          {#if owner}<p class="hint">
              Revoking disconnects this node from the coordinator. Stop its
              workspaces first; revocation does not stop containers on an
              unreachable device.
            </p>
            <button
              class="text-button danger"
              onclick={() => (modal = "revoke-node")}
              ><LockKeyhole size={16} />Revoke node access</button
            >{/if}{/if}
      {:else if modal === "revoke-node" || modal === "revoke-access"}<h2>
          Revoke this {modal === "revoke-node" ? "node" : "access key"}?
        </h2>
        <p class="modal-intro">
          {modal === "revoke-node"
            ? "The node will no longer receive commands. Its existing containers and data remain on the device."
            : "This key and its browser sessions will immediately lose access to the lab."}
        </p>
        <button
          class="primary"
          onclick={() =>
            revoke(
              modal === "revoke-node" ? nodeId : secret,
              modal === "revoke-node" ? "nodes" : "access",
            )}>Revoke access</button
        >
      {:else if modal === "console" && activeWorkspace}<div
          class="console-title"
        >
          <Terminal size={21} />
          <h2>{activeWorkspace.name}</h2>
          <span class="badge">Container console</span>
        </div>
        <p class="hint">
          Commands run inside the container as a non-root user. Output is
          buffered; interactive programs are not supported here. Each command
          has a 30-second limit.
        </p>
        <pre class="terminal-output" aria-live="polite">{output ||
            "CloudLab container console\nYour host machine is outside this workspace.\n"}</pre>
        <form
          class="console-input"
          onsubmit={(e) => {
            e.preventDefault();
            consoleRun();
          }}
        >
          <span>$</span><input
            aria-label="Container command"
            bind:value={command}
            placeholder="pwd"
            disabled={busy}
            autocomplete="off"
          /><button class="primary" disabled={busy || !command.trim()}
            >{busy ? "Running…" : "Run"}<ArrowRight size={15} /></button
          >
        </form>
      {:else if modal === "remote"}<div class="modal-symbol">
          <Globe2 size={25} />
        </div>
        <h2>Your lab, from anywhere.</h2>
        <p class="modal-intro">
          Keep the coordinator and nodes on your hardware. Give the coordinator
          a reachable, encrypted address.
        </p>
        <ol class="help-steps">
          <li>
            <strong>Choose a private VPN or HTTPS gateway.</strong>
            <p>
              Use WireGuard for private access, or the included Caddy
              configuration with your own domain.
            </p>
          </li>
          <li>
            <strong>Make the coordinator reachable.</strong>
            <p>
              Forward HTTPS port 443 to Caddy, or route through a self-hosted
              gateway if your ISP uses CGNAT.
            </p>
          </li>
          <li>
            <strong>Pair nodes with that address.</strong>
            <p>
              Save your HTTPS address in Lab settings. Nodes connect outward; no
              inbound node ports are needed.
            </p>
          </li>
        </ol>
        <div class="form-note">
          <LockKeyhole size={17} />The coordinator authenticates every workspace
          connection.
        </div>
      {:else}<div class="modal-symbol"><FlaskConical size={25} /></div>
        <h2>A quick tour of your lab.</h2>
        <ol class="help-steps">
          <li>
            <strong>Connect a compute node.</strong>
            <p>
              Install Docker and the CloudLab Rust agent on each device.
              Generate a pairing command in Compute nodes.
            </p>
          </li>
          <li>
            <strong>Create your first workspace.</strong>
            <p>
              Choose JupyterLab, a code editor, or a Linux console. Assign a
              node, CPU budget, and memory limit.
            </p>
          </li>
          <li>
            <strong>Make yourself at home.</strong>
            <p>
              Open a workspace, invite collaborators, and adjust your lab
              defaults. Your projects stay on your nodes.
            </p>
          </li>
        </ol>
        <button class="secondary full" onclick={() => (modal = "remote")}
          ><Globe2 size={17} />Learn about global access <ArrowRight
            size={16}
          /></button
        >{/if}
    </div>
  </div>
{/if}
{#if toast}<div class="toast" role="status">
    <CheckCheck size={18} />{toast}<button
      aria-label="Dismiss notification"
      onclick={() => (toast = "")}><X size={15} /></button
    >
  </div>{/if}
