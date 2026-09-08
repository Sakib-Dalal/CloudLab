<script lang="ts">
  import { page } from "$app/state";
  import { ArrowRight } from "@lucide/svelte";
  import SEO from "$lib/components/SEO.svelte";
</script>

<SEO
  title="Page not found"
  description="Find your way back to the CloudLab documentation."
  noindex
/>
<main id="main" class="container error-page">
  <p class="eyebrow">{page.status} / A SMALL DETOUR</p>
  <h1>This page wandered<br />out of the lab.</h1>
  <p>Try the documentation to find your next step.</p>
  <a class="button primary" href="/docs/"
    >Back to the guides<ArrowRight size={18} /></a
  >
</main>
