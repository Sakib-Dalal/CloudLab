<script lang="ts">
  import { onMount, tick } from "svelte";
  import {
    FlaskConical,
    Terminal,
    ArrowLeft,
    ArrowRight,
    Maximize2,
    Minimize2,
    Copy,
    Trash2,
    Server,
    Cpu,
    MemoryStick,
    ShieldCheck,
  } from "lucide-svelte";
  import { api } from "./api";
  import GpuAccess from "./GpuAccess.svelte";
  import WorkspaceMetrics from "./WorkspaceMetrics.svelte";
  import type { Workspace, Node } from "./types";

  let {
    workspace,
    node,
    clock,
    labName,
    nodeName,
    connected,
    demo,
    onclose,
    onbusy,
  }: {
    workspace: Workspace;
    node?: Node;
    clock: number;
    labName: string;
    nodeName: string;
    connected: boolean;
    demo: boolean;
    onclose: () => void;
    onbusy: (running: boolean) => void;
  } = $props();
  let command = $state("");
  let output = $state("");
  let running = $state(false);
  let feedback = $state("");
  let fullscreen = $state(false);
  let supportsFullscreen = $state(false);
  let history: string[] = [];
  let historyIndex = 0;
  let draft = "";
  let input: HTMLInputElement;
  let scrollArea: HTMLDivElement;
  let surface: HTMLElement;
  let disposed = false;
  const available = $derived(connected && workspace.status === "running");
  const memory = $derived(
    `${Number((workspace.memory_mb / 1024).toFixed(1))} GB`,
  );

  onMount(() => {
    supportsFullscreen = document.fullscreenEnabled;
    input?.focus();
    const changed = () => (fullscreen = !!document.fullscreenElement);
    document.addEventListener("fullscreenchange", changed);
    const overflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      disposed = true;
      document.removeEventListener("fullscreenchange", changed);
      document.body.style.overflow = overflow;
      if (document.fullscreenElement === surface)
        document.exitFullscreen().catch(() => {});
    };
  });
  async function append(text: string) {
    output += text;
    await tick();
    scrollArea?.scrollTo({ top: scrollArea.scrollHeight });
  }
  async function run() {
    if (running || !available || !command.trim()) return;
    const text = command;
    command = "";
    history.push(text);
    historyIndex = history.length;
    draft = "";
    running = true;
    onbusy(true);
    feedback = "";
    await append(`\nlab@cloudlab:~ $ ${text}\n`);
    try {
      if (demo) {
        await append(
          "Preview console. Connect a compute node to run real commands.\n",
        );
        return;
      }
      const job = await api<{ id: string }>(
        `/workspaces/${workspace.id}/actions`,
        { action: "exec", command: text },
      );
      for (let i = 0; i < 70 && !disposed; i++) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        if (disposed) return;
        const result = await api<{
          status: string;
          output: string;
          error: string;
        }>(`/jobs/${job.id}`);
        if (result.status === "done" || result.status === "failed") {
          await append((result.output || result.error || "(no output)") + "\n");
          feedback =
            result.status === "failed"
              ? "Command failed. See output above."
              : "Command finished";
          return;
        }
      }
      await append(
        "The command is still pending. Check node connectivity and Activity before running it again.\n",
      );
    } catch (error) {
      await append((error as Error).message + "\n");
      feedback = "Command could not finish";
    } finally {
      running = false;
      onbusy(false);
      await tick();
      input?.focus();
    }
  }
  function recall(event: KeyboardEvent) {
    if (!["ArrowUp", "ArrowDown"].includes(event.key) || !history.length)
      return;
    event.preventDefault();
    if (historyIndex === history.length) draft = command;
    historyIndex = Math.max(
      0,
      Math.min(
        history.length,
        historyIndex + (event.key === "ArrowUp" ? -1 : 1),
      ),
    );
    command = historyIndex === history.length ? draft : history[historyIndex];
  }
  async function copyOutput() {
    try {
      await navigator.clipboard.writeText(output);
      feedback = "Output copied";
    } catch {
      feedback = "Select the output to copy it manually.";
    }
  }
  async function toggleFullscreen() {
    try {
      if (document.fullscreenElement) await document.exitFullscreen();
      else await surface.requestFullscreen();
    } catch {
      feedback =
        "Full screen is unavailable. The console already fills this window.";
    }
  }
</script>

<section
  class="terminal-workspace"
  bind:this={surface}
  aria-label="CloudLab terminal workspace"
>
  <header class="terminal-header">
    <div class="terminal-brand">
      <span><FlaskConical size={24} /></span><strong
        >CloudLab<small>YOUR SPACE TO EXPERIMENT</small></strong
      >
    </div>
    <div class="terminal-heading">
      <h2>{workspace.name}</h2>
      <span class="terminal-label"><Terminal size={13} /> Terminal</span>
    </div>
    <div class="terminal-actions">
      {#if supportsFullscreen}<button
          class="secondary"
          onclick={toggleFullscreen}
          aria-label={fullscreen ? "Exit full screen" : "Enter full screen"}
          >{#if fullscreen}<Minimize2 size={16} />{:else}<Maximize2
              size={16}
            />{/if}<span>{fullscreen ? "Exit full screen" : "Full screen"}</span
          ></button
        >{/if}
      <button
        class="secondary"
        onclick={onclose}
        disabled={running}
        aria-label="Back to lab"
        ><ArrowLeft size={16} /><span>Back to lab</span></button
      >
    </div>
  </header>
  <div class="terminal-info">
    <div>
      <span class="terminal-info-label">LAB</span><strong>{labName}</strong
      ><span class="terminal-separator"></span><Server size={13} /><strong
        >{nodeName}</strong
      >
    </div>
    <div>
      <Cpu size={13} />{workspace.cpus} CPU {workspace.cpus === 1
        ? "core"
        : "cores"}<span class="terminal-separator"></span><MemoryStick
        size={13}
      />{memory} RAM<span class="terminal-separator"></span><ShieldCheck
        size={13}
      />{workspace.network ? "Internet enabled" : "Isolated network"}
    </div>
  </div>
  <WorkspaceMetrics {workspace} {node} {clock} {connected} />
  <GpuAccess {workspace} {node} />
  <div class="terminal-body">
    <div class="terminal-toolbar">
      <span
        ><Terminal size={14} /> Container console
        <span class="terminal-path">/home/lab</span></span
      >
      <div>
        <button
          onclick={copyOutput}
          disabled={!output}
          aria-label="Copy terminal output"
          title="Copy output"><Copy size={15} /></button
        ><button
          onclick={() => {
            output = "";
            feedback = "Console cleared";
            input?.focus();
          }}
          disabled={running || !output}
          aria-label="Clear terminal output"
          title="Clear output"><Trash2 size={15} /></button
        >
      </div>
    </div>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex (keyboard users need to scroll command output) -->
    <div
      class="terminal-scroll"
      bind:this={scrollArea}
      tabindex="0"
      role="region"
      aria-label="Terminal output"
    >
      <div class="terminal-welcome">
        <div class="terminal-welcome-icon"><FlaskConical size={27} /></div>
        <div>
          <p class="terminal-eyebrow">CLOUDLAB / TERMINAL</p>
          <h3>A little room for big ideas.</h3>
          <p>
            Your workspace is ready on <strong>{nodeName}</strong>. Files in
            <code>/home/lab</code> persist between sessions.
          </p>
        </div>
      </div>
      <div class="terminal-start">
        <span>Start exploring</span><button
          onclick={() => {
            command = "pwd";
            input?.focus();
          }}
          disabled={running}>pwd</button
        ><button
          onclick={() => {
            command = "ls -la";
            input?.focus();
          }}
          disabled={running}>ls -la</button
        ><button
          onclick={() => {
            command = "python3 --version";
            input?.focus();
          }}
          disabled={running}>python3 --version</button
        >
      </div>
      <pre class="terminal-transcript" aria-live="polite">{output}</pre>
      {#if running}<p class="terminal-running" role="status">
          Running your command…
        </p>{/if}
    </div>
    <form
      class="terminal-command"
      onsubmit={(event) => {
        event.preventDefault();
        run();
      }}
    >
      <label for="workspace-command"
        >lab<span>@</span>cloudlab <span>~ $</span></label
      >
      <input
        id="workspace-command"
        bind:this={input}
        bind:value={command}
        onkeydown={recall}
        aria-label="Container command"
        placeholder={available ? "Enter a command…" : "Workspace unavailable"}
        disabled={running || !available}
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
      />
      <button
        class="primary"
        disabled={running || !available || !command.trim()}
        >{running ? "Running…" : "Run"}<ArrowRight size={15} /></button
      >
    </form>
    <div class="terminal-notice">
      <span>Buffered commands · 30-second limit · Use ↑ / ↓ for history</span
      ><span role="status">{feedback}</span>
    </div>
  </div>
  <footer class="terminal-statusbar">
    <span class:unavailable={!available}
      ><i></i>{demo
        ? "Preview console"
        : available
          ? "Connected to your container"
          : "Workspace or node offline"}</span
    ><span
      >Non-root user · Interactive apps use the JupyterLab or VS Code terminal</span
    >
  </footer>
</section>
