import assert from "node:assert/strict";
import { readdir, readFile, stat } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

// Validate the artifact Vercel actually publishes, without a running lab or browser.
const root = fileURLToPath(new URL("../build/", import.meta.url));
async function walk(directory) {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await walk(file)));
    else result.push(file);
  }
  return result;
}
const files = await walk(root);
const htmlFiles = files.filter((file) => file.endsWith(".html"));
const pages = new Map();
const decode = (value) =>
  value
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'");
const attributes = (html, attribute) =>
  [...html.matchAll(new RegExp(`\\b${attribute}="([^"]*)"`, "g"))].map(
    (match) => decode(match[1]),
  );
const routeFor = (file) =>
  `/${path.relative(root, file).split(path.sep).join("/")}`
    .replace(/index\.html$/, "")
    .replace(/\.html$/, "");
for (const file of htmlFiles) {
  const html = await readFile(file, "utf8");
  const route = routeFor(file);
  pages.set(route, { html, ids: new Set(attributes(html, "id")) });
  assert.equal(
    (html.match(/<h1(?:\s|>)/g) || []).length,
    1,
    `${route}: expected one rendered H1`,
  );
  assert.match(html, /<title>.+CloudLab<\/title>/, `${route}: missing title`);
  assert.match(
    html,
    /name="description" content="[^"]{30,}"/,
    `${route}: missing description`,
  );
  assert.match(
    html,
    /rel="canonical" href="https?:\/\/[^\"]+"/,
    `${route}: missing canonical URL`,
  );
  assert.match(html, /id="main"/, `${route}: missing skip-link target`);
}
assert.equal(
  [...pages.keys()].filter((route) => /^\/docs\/.+\/$/.test(route)).length,
  15,
  "All 15 guides must be prerendered",
);
assert(
  pages.has("/") &&
    pages.has("/docs/") &&
    pages.has("/privacy/") &&
    pages.has("/404"),
  "Missing top-level route",
);
assert.match(pages.get("/404").html, /name="robots" content="noindex"/);
let linksChecked = 0;
for (const [route, { html }] of pages) {
  for (const href of attributes(html, "href")) {
    const url = new URL(href, `https://site.test${route}`);
    if (url.origin !== "https://site.test") continue;
    const target = pages.get(url.pathname);
    if (target) {
      if (url.hash)
        assert(
          target.ids.has(decodeURIComponent(url.hash.slice(1))),
          `${route}: broken anchor ${href}`,
        );
    } else {
      const file = path.resolve(root, `.${decodeURIComponent(url.pathname)}`);
      assert(file.startsWith(root), `${route}: path escapes build output`);
      assert(
        (await stat(file).catch(() => null))?.isFile(),
        `${route}: broken local link ${href}`,
      );
    }
    linksChecked++;
  }
}
const sitemap = await readFile(path.join(root, "sitemap.xml"), "utf8");
for (const route of pages.keys()) {
  if (route !== "/404")
    assert(sitemap.includes(`${route}</loc>`), `Sitemap omits ${route}`);
}
assert(!sitemap.includes("/404</loc>"), "Do not index the 404 page");
assert.match(
  await readFile(path.join(root, "robots.txt"), "utf8"),
  /Sitemap: https?:\/\//,
);
for (const file of files) {
  const relative = path.relative(root, file);
  assert(
    !/(^|[/\\])(\.cloudlab|\.env|admin-token|state\.json|Cargo\.toml|src-tauri|crates)([/\\]|$)/.test(
      relative,
    ),
    `Private/build-only file in public output: ${relative}`,
  );
}
const deployment = JSON.parse(
  await readFile(new URL("../../vercel.json", import.meta.url), "utf8"),
);
assert.equal(
  deployment.outputDirectory,
  "website/build",
  "Vercel must publish only the public website",
);
assert.equal(deployment.installCommand, "npm ci --prefix website");
assert(
  deployment.buildCommand.includes("npm test --prefix website"),
  "Deployment must validate generated output",
);
console.log(
  `Validated ${pages.size} static pages, 15 guides, ${linksChecked} local links/assets, metadata, sitemap, and deployment boundary.`,
);
