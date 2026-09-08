<script lang="ts">
  import CodeBlock from "./CodeBlock.svelte";
  let { options }: { options: { label: string; code: string }[] } = $props();
  let selected = $state(0);
</script>

<div class="command-tabs">
  <div class="segmented" aria-label="Command platform">
    {#each options as option, i}<button
        class:active={selected === i}
        aria-pressed={selected === i}
        onclick={() => (selected = i)}>{option.label}</button
      >{/each}
  </div>
  <CodeBlock code={options[selected].code} label={options[selected].label} />
</div>
