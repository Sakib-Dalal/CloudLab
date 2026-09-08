import { guides } from "$lib/guides";
import { siteUrl } from "$lib/site";
export const prerender = true;

export function GET() {
  const paths = [
    "/",
    "/docs/",
    "/privacy/",
    ...guides.map((guide) => `/docs/${guide.slug}/`),
  ];
  const escape = (value: string) =>
    value
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;");
  return new Response(
    `<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">${paths.map((path) => `<url><loc>${escape(siteUrl + path)}</loc></url>`).join("")}</urlset>`,
    { headers: { "Content-Type": "application/xml" } },
  );
}
