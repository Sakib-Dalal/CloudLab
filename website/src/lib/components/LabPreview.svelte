<script lang="ts">
  import {
    Cpu,
    Monitor,
    Server,
    CircuitBoard,
    ArrowUpRight,
    Box,
    LockKeyhole,
    FlaskConical,
    Terminal,
    Code2,
    BookOpen,
  } from "@lucide/svelte";
  let selected = $state(0);
  const nodes = [
    {
      name: "Studio Mac",
      kind: "Desktop",
      icon: Monitor,
      cores: "8 CPU",
      memory: "16 GB",
      tool: "code-server",
      project: "Build your next idea",
      toolIcon: Code2,
      detail: "An editor, terminal, and project files in one container.",
      command: "projects / next-big-thing",
      image: "cloudlab/code:2",
    },
    {
      name: "Home server",
      kind: "Server",
      icon: Server,
      cores: "12 CPU",
      memory: "32 GB",
      tool: "JupyterLab",
      project: "Follow your curiosity",
      toolIcon: BookOpen,
      detail: "A notebook workspace for exploration and experiments.",
      command: "projects / experiment.ipynb",
      image: "cloudlab/jupyter:2",
    },
    {
      name: "Little Pi",
      kind: "Single-board computer",
      icon: CircuitBoard,
      cores: "4 CPU",
      memory: "8 GB",
      tool: "Linux console",
      project: "Start with a command",
      toolIcon: Terminal,
      detail: "A focused Linux environment for scripts and small jobs.",
      command: '$ echo "hello, possibilities"',
      image: "cloudlab/terminal:2",
    },
  ];
  const node = $derived(nodes[selected]);
</script>

<div class="lab-preview">
  <div class="preview-top">
    <span><span class="live-dot"></span> THE WEEKEND LAB</span><span
      class="example-label">Interactive example</span
    >
  </div>
  <div class="preview-intro">
    <div>
      <h2>Your compute, connected.</h2>
      <p>Select a node to explore its workspace.</p>
    </div>
    <span class="node-total">03<small>NODES</small></span>
  </div>
  <div class="preview-nodes" aria-label="Example compute nodes">
    {#each nodes as item, i}
      <button
        class:chosen={selected === i}
        aria-pressed={selected === i}
        onclick={() => (selected = i)}
      >
        <span class="node-symbol"
          ><item.icon size={24} strokeWidth={1.6} /></span
        ><strong>{item.name}</strong><small
          >{item.cores}<span>·</span>{item.memory}</small
        ><span class="node-selector" aria-hidden="true"></span>
      </button>
    {/each}
  </div>
  <div class="preview-connector" aria-hidden="true">
    <span></span><span></span><span></span>
  </div>
  <div class="preview-coordinator">
    <span class="coordinator-icon"><FlaskConical size={18} /></span><strong
      >CloudLab coordinator</strong
    ><span>On your hardware</span><LockKeyhole size={14} />
  </div>
  <div class="preview-path">
    <span></span><small>ISOLATED WORKSPACE ON {node.name.toUpperCase()}</small
    ><span></span>
  </div>
  <div class="preview-workspace" aria-live="polite">
    <div class="workspace-preview-heading">
      <span class="workspace-symbol"
        ><node.toolIcon size={24} strokeWidth={1.6} /></span
      >
      <div>
        <small>{node.tool}</small>
        <h3>{node.project}</h3>
      </div>
      <Box size={20} />
    </div>
    <p>{node.detail}</p>
    <div class="preview-code">
      <span class="code-file-dot"></span><code>{node.command}</code><span
        class="cursor"
        aria-hidden="true">▍</span
      >
    </div>
    <div class="preview-workspace-footer">
      <span><Cpu size={13} />2 CPU<span>·</span>2 GB RAM</span><span
        ><LockKeyhole size={12} />Container only</span
      >
    </div>
  </div>
  <a href="/docs/architecture/" class="preview-bottom"
    ><span>Small pieces. One connected lab.</span><ArrowUpRight size={16} /></a
  >
</div>
