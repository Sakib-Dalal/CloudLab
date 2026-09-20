<script lang="ts">
  import { onMount } from "svelte";
  import {
    Package,
    Search,
    Download,
    Trash2,
    RefreshCw,
    Globe2,
    ArrowLeft,
    Check,
    ExternalLink,
  } from "lucide-svelte";
  import { api } from "./api";
  import type { Workspace } from "./types";
  let {
    workspace,
    connected,
    demo,
    onsettings,
    onback,
  }: {
    workspace: Workspace;
    connected: boolean;
    demo: boolean;
    onsettings: () => void;
    onback: () => void;
  } = $props();
  type Installed = { name: string; version: string; removable: boolean };
  type Result = {
    name: string;
    version: string;
    summary: string;
    requires_python: string;
    url: string;
  };
  let packages = $state<Installed[]>([]),
    results = $state<Result[]>([]);
  let query = $state(""),
    filter = $state(""),
    error = $state(""),
    notice = $state(""),
    output = $state("");
  let busy = $state(false),
    searching = $state(false),
    loaded = $state(false),
    searched = $state(false);
  let python = $state(""),
    progress = $state(""),
    remove = $state<Installed | null>(null);
  let disposed = false;
  const available = $derived(connected && workspace.status === "running");
  const visible = $derived(
    packages.filter((p) => p.name.toLowerCase().includes(filter.toLowerCase())),
  );
  const normalized = (name: string) =>
    name.toLowerCase().replace(/[-_.]+/g, "-");
  const installed = (name: string) =>
    packages.find((p) => normalized(p.name) === normalized(name));
  onMount(() => {
    if (available) load();
    return () => {
      disposed = true;
    };
  });
  async function job(action: string, name = "", version = "") {
    const submitted = await api<{ id: string }>(
      `/workspaces/${workspace.id}/packages`,
      { action, name, version },
    );
    for (let i = 0; i < 700 && !disposed; i++) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      if (disposed) return null;
      const result = await api<{
        status: string;
        output: string;
        error: string;
      }>(`/jobs/${submitted.id}`);
      if (result.status === "failed")
        throw new Error(result.error || "Package operation failed.");
      if (result.status === "done") return result.output;
    }
    if (!disposed)
      throw new Error(
        "This operation is still pending. Check Activity before retrying, then refresh the installed list.",
      );
    return null;
  }
  async function list() {
    if (demo) {
      if (!loaded)
        packages = [
          { name: "ipykernel", version: "6.30.1", removable: false },
          { name: "numpy", version: "2.3.3", removable: true },
        ];
      python = "3.12 (preview)";
      loaded = true;
      return;
    }
    const response = await job("list");
    if (response === null) return;
    const value = JSON.parse(response);
    packages = value.packages;
    python = value.python;
    loaded = true;
  }
  async function load() {
    busy = true;
    error = "";
    progress = "Reading installed packages…";
    try {
      await list();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
      progress = "";
    }
  }
  async function search() {
    searching = true;
    error = "";
    searched = true;
    try {
      if (demo)
        results = [
          {
            name: query.trim() || "requests",
            version: "2.32.5",
            summary:
              "Sample package result. Connect a real workspace to search PyPI and manage packages.",
            requires_python: ">=3.9",
            url: "",
          },
        ];
      else
        results = (
          await api<{ packages: Result[] }>(
            `/packages/search?q=${encodeURIComponent(query.trim())}`,
          )
        ).packages;
    } catch (e) {
      error = (e as Error).message;
      results = [];
    } finally {
      searching = false;
    }
  }
  async function change(
    action: "install" | "remove",
    name: string,
    version = "",
  ) {
    busy = true;
    error = "";
    notice = "";
    output = "";
    remove = null;
    progress = `${action === "install" ? "Installing" : "Removing"} ${name}…`;
    try {
      if (demo) {
        packages = packages.filter(
          (p) => normalized(p.name) !== normalized(name),
        );
        if (action === "install")
          packages = [...packages, { name, version, removable: true }];
        loaded = true;
      } else {
        const result = await job(action, name, version);
        if (result === null) return;
        output = result;
        await list();
      }
      notice = `${name} ${action === "install" ? "installed" : "removed"}. Restart an active notebook kernel to use the new environment.`;
    } catch (e) {
      error = (e as Error).message;
    } finally {
      busy = false;
      progress = "";
    }
  }
</script>

<div class="package-manager">
  <div class="modal-symbol"><Package size={25} /></div>
  <div class="eyebrow">{workspace.name}</div>
  <h2>Python packages</h2>
  <p class="modal-intro">
    Manage your workspace environment with uv. Packages stay with your saved
    files.
  </p>
  <div class="package-environment">
    <span
      ><strong>uv</strong> · {python
        ? `Python ${python}`
        : "Python environment"}<small>/home/lab/.venv</small></span
    ><span class:offline={!workspace.network}
      ><Globe2 size={15} />Internet {workspace.network ? "on" : "off"}</span
    >
  </div>
  {#if !available}<div class="form-note">
      {connected
        ? "Resume this workspace to view, install, or remove packages."
        : "Reconnect the compute node to manage packages."}
    </div>{/if}
  {#if !workspace.network}<p class="hint">
      Installation needs internet access. You can still browse the catalog and
      remove installed packages.
    </p>{/if}
  <button class="text-button" disabled={busy} onclick={onsettings}
    >Workspace settings · internet and compute</button
  >
  {#if workspace.status === "stopped" || workspace.status === "error"}<p
      class="hint"
    >
      Using an older workspace image? Build the current images on its node, then
      choose Update environment in workspace controls.
    </p>{/if}
  <form
    class="package-search"
    onsubmit={(event) => {
      event.preventDefault();
      search();
    }}
  >
    <label for="package-query">Find a PyPI package</label>
    <div>
      <input
        id="package-query"
        bind:value={query}
        placeholder="Package name, e.g. numpy"
        required
        maxlength="128"
      /><button class="primary" disabled={searching || busy || !query.trim()}
        ><Search size={16} />{searching ? "Searching…" : "Search"}</button
      >
    </div>
    <p class="hint">
      Look up any exact package name, or start typing a popular package name.
    </p>
  </form>
  {#if error}<div class="inline-error" role="alert">{error}</div>{/if}
  {#if progress}<div class="package-progress" role="status">
      <RefreshCw size={16} />{progress}<small
        >You can close this panel; the operation will continue.</small
      >
    </div>{/if}
  {#if notice}<p class="package-success" role="status">
      <Check size={16} />{notice}
    </p>{/if}
  {#if searched && !searching}<section
      class="package-results"
      aria-label="Package search results"
    >
      {#each results as result}<article class="package-result">
          <div>
            <strong>{result.name}</strong><span class="package-version"
              >{result.version}</span
            >{#if result.url}<a
                href={result.url}
                target="_blank"
                rel="noreferrer"
                aria-label={`View ${result.name} on PyPI`}
                ><ExternalLink size={14} /></a
              >{/if}
            <p>{result.summary}</p>
            {#if result.requires_python}<small
                >Python {result.requires_python}</small
              >{/if}
          </div>
          <button
            class="secondary"
            disabled={busy ||
              !available ||
              !workspace.network ||
              installed(result.name)?.version === result.version}
            onclick={() => change("install", result.name, result.version)}
            ><Download size={15} />{installed(result.name)?.version ===
            result.version
              ? "Installed"
              : installed(result.name)
                ? "Update"
                : "Install"}</button
          >
        </article>{:else}<p class="hint">
          No match found. Try the package’s full PyPI name.
        </p>{/each}
    </section>{/if}
  <section class="installed-packages" aria-label="Installed packages">
    <div class="section-heading">
      <h3>
        Installed {#if loaded}<span class="count">{packages.length}</span>{/if}
      </h3>
      <button class="text-button" onclick={load} disabled={busy || !available}
        ><RefreshCw size={15} />Refresh</button
      >
    </div>
    <label class="sr-only" for="installed-filter"
      >Filter installed packages</label
    ><input
      id="installed-filter"
      bind:value={filter}
      placeholder="Filter installed packages…"
    />
    <div class="package-list">
      {#each visible as pkg}<div class="package-row">
          <span
            ><strong>{pkg.name}</strong><small
              >{pkg.version}{!pkg.removable
                ? " · Included in image"
                : ""}</small
            ></span
          ><button
            class="icon-button"
            aria-label={`Remove ${pkg.name}`}
            title={pkg.removable
              ? `Remove ${pkg.name}`
              : "Image packages are read-only"}
            disabled={busy || !available || !pkg.removable}
            onclick={() => (remove = pkg)}><Trash2 size={16} /></button
          >
        </div>{:else}<p class="hint">
          {loaded
            ? "No matching packages."
            : "Installed packages will appear here."}
        </p>{/each}
    </div>
  </section>
  {#if remove}<div class="package-confirm" role="alert">
      <p>
        Remove <strong>{remove.name}</strong>? Code that uses it may need it
        installed again.
      </p>
      <button class="secondary" onclick={() => (remove = null)}
        >Keep package</button
      ><button class="primary" onclick={() => change("remove", remove!.name)}
        >Remove package</button
      >
    </div>{/if}
  {#if output}<details>
      <summary>Operation output</summary>
      <pre class="package-output">{output}</pre>
    </details>{/if}
  <button class="secondary full" onclick={onback}
    ><ArrowLeft size={16} />Back to workspace</button
  >
</div>
