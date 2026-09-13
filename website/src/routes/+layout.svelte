<script lang="ts">
  import "@fontsource/dm-sans/latin-400.css";
  import "@fontsource/dm-sans/latin-500.css";
  import "@fontsource/dm-sans/latin-600.css";
  import "@fontsource/manrope/latin-400.css";
  import "@fontsource/manrope/latin-500.css";
  import "@fontsource/manrope/latin-600.css";
  import "@fontsource/manrope/latin-700.css";
  import "@fontsource/manrope/latin-800.css";
  import "../app.css";
  import { page } from "$app/state";
  import { afterNavigate } from "$app/navigation";
  import { FlaskConical, ArrowUpRight, GitFork, Menu, X } from "@lucide/svelte";
  import Search from "$lib/components/Search.svelte";
  import { repository } from "$lib/site";
  let { children } = $props();
  let menuOpen = $state(false);
  afterNavigate(() => (menuOpen = false));
</script>

<a class="skip-link" href="#main">Skip to content</a>
<header class="site-header">
  <div class="header-inner">
    <a class="brand" href="/" aria-label="CloudLab home"
      ><span class="brand-mark"
        ><FlaskConical size={23} strokeWidth={1.8} /></span
      ><span>CloudLab<span class="brand-dot">.</span></span></a
    >
    <nav class="desktop-nav" aria-label="Main navigation">
      <a href="/#possibilities">Why CloudLab</a>
      <a
        href="/docs/"
        class:current={page.url.pathname.startsWith("/docs/")}
        aria-current={page.url.pathname === "/docs/" ? "page" : undefined}
        >Documentation</a
      >
      <a href="/docs/project/">Open source<ArrowUpRight size={12} /></a>
    </nav>
    <div class="header-actions">
      <Search /><a class="button primary header-cta" href="/docs/quickstart/"
        >Get started<ArrowUpRight size={15} /></a
      ><button
        class="icon-button menu-toggle"
        aria-label={menuOpen ? "Close menu" : "Open menu"}
        aria-expanded={menuOpen}
        aria-controls="mobile-nav"
        onclick={() => (menuOpen = !menuOpen)}
        >{#if menuOpen}<X size={22} />{:else}<Menu size={22} />{/if}</button
      >
    </div>
  </div>
  {#if menuOpen}<nav
      id="mobile-nav"
      class="mobile-nav"
      aria-label="Mobile navigation"
    >
      <a href="/#possibilities">Why CloudLab</a><a href="/docs/"
        >Documentation</a
      ><a href="/docs/project/">Open source</a><a href="/docs/quickstart/"
        >Get started →</a
      >
    </nav>{/if}
</header>

{@render children()}

<footer class="site-footer">
  <div class="footer-main container">
    <div class="footer-about">
      <a class="brand" href="/"
        ><span class="brand-mark"
          ><FlaskConical size={23} strokeWidth={1.8} /></span
        ><span>CloudLab<span class="brand-dot">.</span></span></a
      >
      <p>A little less setup.<br />A lot more possibility.</p>
      <span class="footer-meta">Local compute. Open source. Yours.</span>
    </div>
    <div>
      <h2>Get building</h2>
      <a href="/docs/quickstart/">Quick start</a><a href="/docs/nodes/"
        >Connect a node</a
      ><a href="/docs/workspaces/">Workspaces</a><a href="/docs/desktop/"
        >Desktop app</a
      >
    </div>
    <div>
      <h2>Go deeper</h2>
      <a href="/docs/architecture/">Architecture</a><a href="/docs/security/"
        >Security model</a
      ><a href="/docs/remote-access/">Remote access</a><a href="/docs/duckdns/"
        >DuckDNS setup</a
      ><a href="/docs/troubleshooting/">Troubleshooting</a>
    </div>
    <div>
      <h2>In the open</h2>
      <a href={repository}>GitHub<ArrowUpRight size={13} /></a><a
        href={`${repository}/issues`}
        >Issues & ideas<ArrowUpRight size={13} /></a
      ><a href="/docs/project/">Project & roadmap</a><a
        href="/docs/website-deployment/">Website deployment</a
      >
    </div>
  </div>
  <div class="footer-bottom container">
    <span>© {new Date().getFullYear()} CloudLab · By Sakib Dalal</span><span
      ><a href={`${repository}/blob/main/LICENSE`}>MIT licensed</a><span
        aria-hidden="true">/</span
      ><a href="/privacy/">Privacy</a><a
        href={repository}
        aria-label="CloudLab on GitHub"><GitFork size={17} /></a
      ></span
    >
  </div>
</footer>
