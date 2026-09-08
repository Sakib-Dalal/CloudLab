<script lang="ts">
  import { page } from "$app/state";
  import { siteUrl } from "$lib/site";
  let {
    title,
    description,
    noindex = false,
  }: { title: string; description: string; noindex?: boolean } = $props();
</script>

<svelte:head>
  <title>{title} · CloudLab</title>
  <meta name="description" content={description} />
  <link rel="canonical" href={`${siteUrl}${page.url.pathname}`} />
  <meta property="og:type" content="website" />
  <meta property="og:site_name" content="CloudLab" />
  <meta property="og:title" content={`${title} · CloudLab`} />
  <meta property="og:description" content={description} />
  <meta property="og:url" content={`${siteUrl}${page.url.pathname}`} />
  <meta name="twitter:card" content="summary" />
  {#if noindex}<meta name="robots" content="noindex" />{/if}
</svelte:head>
