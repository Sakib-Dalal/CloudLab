import { error } from "@sveltejs/kit";
import { guides } from "$lib/guides";
import type { EntryGenerator, PageLoad } from "./$types";

export const entries: EntryGenerator = () =>
  guides.map((guide) => ({ slug: guide.slug }));
export const load: PageLoad = ({ params }) => {
  const index = guides.findIndex((guide) => guide.slug === params.slug);
  if (index < 0) error(404, "This guide could not be found");
  return {
    guide: guides[index],
    previous: guides[index - 1] ?? null,
    next: guides[index + 1] ?? null,
  };
};
