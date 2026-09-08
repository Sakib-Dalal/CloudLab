import { siteUrl } from "$lib/site";
export const prerender = true;
export function GET() {
  return new Response(
    `User-agent: *\nAllow: /\nSitemap: ${siteUrl}/sitemap.xml\n`,
    { headers: { "Content-Type": "text/plain" } },
  );
}
