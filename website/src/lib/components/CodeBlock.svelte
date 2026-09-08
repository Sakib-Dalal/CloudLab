<script lang="ts">
  import { Check, Copy } from "@lucide/svelte";
  import { onDestroy } from "svelte";
  let { code, label = "Terminal" }: { code: string; label?: string } = $props();
  let status = $state("");
  let timer: ReturnType<typeof setTimeout>;
  async function copy() {
    try {
      await navigator.clipboard.writeText(code);
      status = "Copied";
    } catch {
      status = "Select the code to copy it manually";
    }
    clearTimeout(timer);
    timer = setTimeout(() => (status = ""), 3000);
  }
  onDestroy(() => clearTimeout(timer));
</script>

<div class="code-block">
  <div class="code-bar">
    <span>{label}</span><button
      onclick={copy}
      aria-label={`Copy ${label} commands`}
      >{#if status === "Copied"}<Check size={14} />{:else}<Copy
          size={14}
        />{/if}<span>{status === "Copied" ? "Copied" : "Copy"}</span></button
    >
  </div>
  <!-- svelte-ignore a11y_no_noninteractive_tabindex (Keyboard users need to scroll long commands horizontally.) -->
  <pre
    tabindex="0"
    aria-label={`${label} code, scroll horizontally if needed`}><code
      >{code}</code
    ></pre>
  <span class="sr-only" role="status">{status}</span>
</div>
