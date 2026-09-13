<script lang="ts">
  import { ArrowUpRight, Terminal } from "@lucide/svelte";
  import logo from "../../../../crates/cloudlab/assets/cli-logo.txt?raw";
  import CodeBlock from "./CodeBlock.svelte";

  const commands = [
    {
      label: "Help",
      command: "cloudlab --help",
      description:
        "See every command, its options, and examples to get started.",
    },
    {
      label: "Start",
      command: "cloudlab agent start",
      description:
        "Resume the pairing saved in this folder. The agent remembers your coordinator address.",
    },
    {
      label: "Status",
      command: "cloudlab agent status",
      description:
        "Check whether the local agent is running and whether a pairing is saved.",
    },
    {
      label: "Stop",
      command: "cloudlab agent stop",
      description:
        "Stop the agent and keep its pairing, containers, and saved volumes.",
    },
    {
      label: "Reset pairing",
      command: "cloudlab agent delete",
      description:
        "Stop the agent, revoke its access, and clear its pairing. Docker containers and volumes stay on the node.",
    },
  ];
  let selected = $state(0);
</script>

<section class="cli-preview" aria-label="CloudLab CLI preview">
  <div class="cli-copy">
    <p class="eyebrow">AT YOUR COMMAND</p>
    <h2>Your lab, from the terminal.</h2>
    <p>Choose a command to see what it does, then copy it for your terminal.</p>
    <div class="cli-options" role="group" aria-label="Preview a CLI command">
      {#each commands as command, i}
        <button aria-pressed={selected === i} onclick={() => (selected = i)}
          >{command.label}</button
        >
      {/each}
    </div>
    <p class="cli-description" aria-live="polite">
      {commands[selected].description}
    </p>
    <a href="/docs/cli/">Read the CLI guide <ArrowUpRight size={16} /></a>
  </div>
  <div class="cli-terminal">
    <div class="terminal-title">
      <Terminal size={15} /><span>CloudLab · Terminal</span><span
        class="terminal-version">2.0</span
      >
    </div>
    <pre
      class="cli-logo"
      role="img"
      aria-label="CloudLab ASCII logo">{logo.trimEnd()}</pre>
    <p class="terminal-tagline">Your computers. One lab.</p>
    <CodeBlock
      code={commands[selected].command}
      label={commands[selected].label}
    />
  </div>
</section>

<style>
  .cli-preview {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 36px;
    align-items: center;
    padding: 34px 0;
  }
  .cli-copy,
  .cli-terminal {
    min-width: 0;
  }
  h2 {
    font-size: clamp(24px, 3vw, 34px);
    line-height: 1.2;
    margin: 10px 0 14px;
  }
  .cli-copy > p:not(.eyebrow) {
    color: var(--muted);
    line-height: 1.7;
  }
  .cli-options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 20px 0 14px;
  }
  .cli-options button {
    border: 1px solid var(--line);
    background: var(--surface);
    border-radius: 7px;
    padding: 8px 12px;
    font-size: 13px;
  }
  .cli-options button[aria-pressed="true"] {
    color: white;
    background: var(--red-dark);
    border-color: var(--red-dark);
  }
  .cli-description {
    min-height: 5em;
    font-size: 14px;
  }
  .cli-copy > a {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
    border-bottom: 1px solid var(--red);
    padding-bottom: 4px;
  }
  .cli-terminal {
    background: #292722;
    color: var(--cream);
    border: 1px solid #4b443d;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 16px 38px #302e2a12;
  }
  .terminal-title {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 15px 18px;
    color: #e0cbb5;
    border-bottom: 1px solid #4b443d;
    font-size: 12px;
  }
  .terminal-version {
    margin-left: auto;
  }
  .cli-logo {
    font: clamp(9px, 1.15vw, 13px)/1.3 monospace;
    padding: 24px 16px 0;
    margin: 0;
    overflow-x: auto;
    color: #e5a395;
  }
  .terminal-tagline {
    padding: 8px 20px 16px;
    margin: 0;
    font-size: 12px;
    color: #d6c2ae;
  }
  .cli-terminal :global(.code-block) {
    margin: 0;
    border-radius: 0;
    border-width: 1px 0 0;
  }
  @media (max-width: 1150px) {
    .cli-preview {
      grid-template-columns: 1fr;
      gap: 24px;
    }
    .cli-description {
      min-height: 0;
    }
    .cli-logo {
      font-size: clamp(8px, 1.8vw, 13px);
    }
  }
</style>
