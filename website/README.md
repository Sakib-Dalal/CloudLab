# CloudLab public website

A standalone, prerendered SvelteKit site for the CloudLab product and documentation. Uses the same burgundy, sand, cream, and blue palette as the lab app, with local fonts, responsive layouts, local documentation search, an interactive explanatory lab diagram, copyable commands, OS command selectors, FAQs, and 15 guides.

```sh
npm ci --prefix website
npm run dev --prefix website
```

Open http://127.0.0.1:4173. Commands run from the repository root. This site does not need Rust, Docker, or a running lab.

## Production checks

```sh
npm run check --prefix website
npm run build --prefix website
npm test --prefix website
npm run preview --prefix website
```

The static adapter writes HTML, fonts, CSS, and JavaScript to `website/build/`. The artifact checker validates every internal link/anchor, rendered page metadata, all guide routes, sitemap, 404, and the public output boundary. It is not a browser interaction or visual audit.

## Vercel + GitHub

1. Import `Sakib-Dalal/CloudLab` into Vercel, or use its existing linked project.
2. Keep **Root Directory** at the **repository root**. Do not select `website/`.
3. Choose Node.js **22.x** or a compatible newer version, and production branch **main**.
4. Root `vercel.json` overrides the framework, install command, build command, and output directory. No Rust compilation or Docker service runs on Vercel.
5. If using another domain, set **PUBLIC_SITE_URL** to its HTTPS origin (no path), then rebuild. The default is `https://cloudlab-alpha.vercel.app`.
6. Enable Vercel Git deployments. Pushes to `main` deploy production; branches and pull requests receive previews according to project settings. Vercel runs the website check/build/artifact check before publishing. GitHub Actions also validates the site independently.

No Vercel API token or deployment secret belongs in this repository. Build artifacts and `.vercel/` are ignored. A Vercel project/account link is required once; configuration files alone cannot authorize access to an account. See [Vercel's GitHub documentation](https://vercel.com/docs/git/vercel-for-github).

The root app's `npm run build` still builds the **private lab dashboard** into `dist/`. Vercel publishes **only** `website/build/`. Host the Rust coordinator and workspace gateway on your own infrastructure. Never point a lab API or workspace-origin proxy at this documentation deployment.

## Editing

- `src/routes/+page.svelte`: product landing page.
- `../shared/duckdns-guide.ts`: ten-step DuckDNS setup shared with the dashboard, including IP updates, wildcard HTTPS, and troubleshooting.
- `src/lib/guides.ts`: structured guide content, navigation, and local search.
- `src/routes/docs/[slug]/`: prerendered article renderer and routing.
- `src/app.css`: shared responsive visual system.
- `src/lib/components/`: search dialog, copy controls, OS selector, and explanatory diagram.
- `src/routes/privacy/`: website privacy explanation.
- `src/routes/sitemap.xml/` and `robots.txt/`: build-time SEO files.
- `scripts/check-site.mjs`: generated-artifact validation.

Keep claims aligned with the implementation and `docs/security.md`, `docs/remote-access.md`, and `docs/validation.md`. Clearly distinguish shipped capabilities from future work and examples from connected hardware.

No analytics or third-party font requests are included. Search runs in the browser. Vercel can process normal hosting request logs; see the website privacy page.

The dependency override pins the compatible `cookie` 0.7 line because SvelteKit currently requests 0.6, which has a known cookie-attribute validation advisory. The published site is static and does not run SvelteKit's cookie server; keep the override under review when updating the framework.
