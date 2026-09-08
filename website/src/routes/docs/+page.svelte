<script lang="ts">
  import { ArrowUpRight, Search, Clock, ArrowRight } from "@lucide/svelte";
  import SEO from "$lib/components/SEO.svelte";
  import { categories, searchGuides } from "$lib/guides";
  let query = $state("");
  const matches = $derived(searchGuides(query));
</script>

<SEO
  title="Documentation"
  description="Everything you need to build, connect, and maintain a personal CloudLab. Quick start, nodes, container workspaces, security, remote access, and troubleshooting."
/>
<div class="docs-landing">
  <p class="eyebrow">THE CLOUDLAB FIELD GUIDE</p>
  <h1>Let’s make it work.</h1>
  <p class="docs-lead">
    From your first node to a lab you can reach anywhere.<br />Practical guides
    for every step along the way.
  </p>
  <a class="docs-start-card" href="/docs/quickstart/"
    ><div>
      <span class="eyebrow">NEW TO CLOUDLAB?</span>
      <h2>Your first lab, step by step.</h2>
      <p>
        Start on one computer. Build the coordinator, pair a node, and open your
        first workspace.
      </p>
      <span>Start the guide<ArrowRight size={17} /></span>
    </div>
    <span class="start-card-number">01<span>START HERE</span></span></a
  >
  <label class="search-field docs-filter"
    ><Search size={19} /><input
      type="search"
      placeholder="Find a guide, tool, or answer…"
      aria-label="Filter documentation guides"
      bind:value={query}
    /><kbd>/ docs</kbd></label
  >
  {#if query}<p class="search-count" role="status">
      {matches.length}
      {matches.length === 1 ? "guide" : "guides"} matching “{query}”
    </p>{/if}
  {#each categories as category}
    {@const items = matches.filter((guide) => guide.category === category)}
    {#if items.length}<section class="docs-category">
        <h2>{category}<span>{String(items.length).padStart(2, "0")}</span></h2>
        <div class="docs-guide-grid">
          {#each items as guide}<a
              class="docs-guide-card"
              href={`/docs/${guide.slug}/`}
              ><h3>{guide.title}<ArrowUpRight size={17} /></h3>
              <p>{guide.description}</p>
              <small><Clock size={12} />{guide.minutes} min read</small></a
            >{/each}
        </div>
      </section>{/if}
  {/each}
  {#if !matches.length}<div class="search-empty">
      <h2>No guide found yet.</h2>
      <p>Try “Docker”, “access”, “backup”, or a shorter phrase.</p>
      <button class="button secondary" onclick={() => (query = "")}
        >Show all guides</button
      >
    </div>{/if}
</div>
