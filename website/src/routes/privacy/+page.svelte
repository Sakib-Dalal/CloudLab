<script lang="ts">
  import SEO from "$lib/components/SEO.svelte";
</script>

<SEO
  title="Website privacy"
  description="How this public CloudLab website handles search, fonts, analytics, and hosting."
/>
<main id="main" class="container standalone-page guide-article">
  <p class="eyebrow">A SMALL WEBSITE FOOTPRINT</p>
  <h1>Website privacy</h1>
  <p class="docs-lead">The guide is public. Your lab stays yours.</p>
  <section class="article-section">
    <h2>This website</h2>
    <p>
      CloudLab’s public website is a static SvelteKit site. It has no account
      system, contact form, analytics integration, advertising scripts, or
      connections to your lab. Documentation search runs locally in your
      browser. Fonts are served with the website, without contacting a
      third-party font service.
    </p>
    <p>
      The site does not intentionally set application cookies or store your
      search queries. Vercel, the hosting provider, can process normal request
      information such as IP addresses and user-agent headers to deliver and
      protect the website. Its handling is described in the <a
        href="https://vercel.com/legal/privacy-policy">Vercel privacy policy</a
      >.
    </p>
  </section>
  <section class="article-section">
    <h2>Links to other services</h2>
    <p>
      Links to GitHub, Vercel, and dependency documentation take you to those
      services, whose own privacy policies apply. The copy buttons use your
      browser’s clipboard API only when you press them.
    </p>
  </section>
  <section class="article-section">
    <h2>Your CloudLab installation</h2>
    <p>
      Your self-hosted coordinator is separate from this site. It uses
      authentication cookies and stores lab metadata on your hardware. Node
      agents and Docker volumes store their own credentials and project data
      locally. Read the <a href="/docs/security/">security model</a> and
      <a href="/docs/operations/">operations guide</a> for details.
    </p>
  </section>
</main>
