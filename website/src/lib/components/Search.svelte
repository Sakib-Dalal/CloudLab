<script lang="ts">
  import { Search, X, ArrowUpRight, BookOpen } from "@lucide/svelte";
  import { searchGuides } from "$lib/guides";
  let dialog: HTMLDialogElement;
  let query = $state("");
  let input: HTMLInputElement;
  const matches = $derived(searchGuides(query));
  function open() {
    query = "";
    dialog.showModal();
    input.focus();
  }
  function shortcut(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      if (dialog.open) dialog.close();
      else open();
    }
  }
</script>

<svelte:window onkeydown={shortcut} />
<button class="search-trigger" onclick={open} aria-label="Search documentation"
  ><Search size={16} /><span>Search docs</span><kbd>⌘ K</kbd></button
>
<dialog class="search-dialog" bind:this={dialog} aria-labelledby="search-title">
  <div class="search-dialog-header">
    <h2 id="search-title">Find your next step</h2>
    <button
      class="icon-button"
      onclick={() => dialog.close()}
      aria-label="Close search"><X size={21} /></button
    >
  </div>
  <label class="search-field"
    ><Search size={20} /><input
      bind:this={input}
      bind:value={query}
      placeholder="Try “Docker”, “remote access”, or “backups”"
      aria-label="Search all documentation"
      autocomplete="off"
    /></label
  >
  <p class="search-count" role="status">
    {query
      ? `${matches.length} matching ${matches.length === 1 ? "guide" : "guides"}`
      : "Explore the documentation"}
  </p>
  <div class="search-results">
    {#each matches as guide}
      <a href={`/docs/${guide.slug}/`} onclick={() => dialog.close()}
        ><BookOpen size={18} /><span
          ><strong>{guide.title}</strong><small>{guide.description}</small
          ></span
        ><ArrowUpRight size={17} /></a
      >
    {:else}
      <div class="search-empty">
        <p>No guides found for “{query}”.</p>
        <span>Try a tool name, a shorter phrase, or browse all guides.</span
        ><button class="button secondary" onclick={() => (query = "")}
          >Show all guides</button
        >
      </div>
    {/each}
  </div>
  <div class="search-hint">
    Local search. No data leaves this website.<span
      ><kbd>esc</kbd> to close</span
    >
  </div>
</dialog>
