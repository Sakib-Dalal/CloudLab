<script lang="ts">
  import { page } from "$app/state";
  import { BookOpen, ArrowUpRight, List } from "@lucide/svelte";
  import { guides, categories } from "$lib/guides";
  let { children } = $props();
  let expanded = $state(false);
  const isActive = (slug: string) => page.url.pathname === `/docs/${slug}/`;
</script>

<div class="docs-shell container">
  <aside class="docs-sidebar">
    <button
      class="docs-menu-button"
      onclick={() => (expanded = !expanded)}
      aria-expanded={expanded}
      aria-controls="docs-navigation"
      ><List size={17} />Guide navigation<span>{expanded ? "−" : "+"}</span
      ></button
    >
    <nav id="docs-navigation" class:expanded aria-label="Documentation">
      <a
        class="docs-index-link"
        class:active={page.url.pathname === "/docs/"}
        href="/docs/"
        onclick={() => (expanded = false)}
        ><BookOpen size={17} />Documentation</a
      >{#each categories as category}<div class="docs-nav-group">
          <h2>{category}</h2>
          {#each guides.filter((guide) => guide.category === category) as guide}<a
              href={`/docs/${guide.slug}/`}
              class:active={isActive(guide.slug)}
              aria-current={isActive(guide.slug) ? "page" : undefined}
              onclick={() => (expanded = false)}>{guide.title}</a
            >{/each}
        </div>{/each}<a
        class="sidebar-github"
        href="https://github.com/Sakib-Dalal/CloudLab"
        >View source on GitHub<ArrowUpRight size={13} /></a
      >
    </nav>
  </aside>
  <main id="main" class="docs-main">{@render children()}</main>
</div>
