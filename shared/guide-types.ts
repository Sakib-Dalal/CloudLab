export type Block =
  | { type: "text"; text: string }
  | { type: "list"; items: string[]; ordered?: boolean }
  | { type: "note"; title: string; text: string }
  | { type: "code"; code: string; label?: string }
  | { type: "tabs"; options: { label: string; code: string }[] }
  | { type: "links"; items: { label: string; href: string }[] }
  | { type: "table"; columns: string[]; rows: string[][] };

export type Guide = {
  slug: string;
  title: string;
  description: string;
  category: string;
  minutes: number;
  sections: { id: string; title: string; blocks: Block[] }[];
};
