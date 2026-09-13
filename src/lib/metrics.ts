import type { Metrics } from "./types";
export type Point = { at: number; value: number | null };
export type Series = { name: string; color: string; points: Point[] };
export const palette = [
  "#70d7c7",
  "#ac9cff",
  "#f4b56b",
  "#70b9f1",
  "#ee92b4",
  "#aed581",
];
export const percent = (n: number | null | undefined) =>
  n == null ? "—" : `${n.toFixed(1)}%`;
export const memory = (n: number | null | undefined) =>
  n == null
    ? "—"
    : n < 1024
      ? `${Math.round(n)} MB`
      : `${(n / 1024).toFixed(1)} GB`;
export const bytes = (n: number | null | undefined) =>
  n == null
    ? "—"
    : n >= 1048576
      ? `${(n / 1048576).toFixed(1)} MB`
      : n >= 1024
        ? `${(n / 1024).toFixed(1)} kB`
        : `${Math.round(n)} B`;
export const rate = (n: number | null | undefined) =>
  n == null ? "—" : `${bytes(n)}/s`;
export const fresh = (m: Metrics | null | undefined, clock: number) =>
  !!m && m.at <= clock + 5 && clock - m.at < 30;
export function samples(
  history: Metrics[] = [],
  latest?: Metrics | null,
): Metrics[] {
  return latest && latest.at > (history.at(-1)?.at ?? 0)
    ? [...history, latest]
    : history;
}
export function points(
  history: Metrics[],
  read: (m: Metrics, previous?: Metrics) => number | null,
): Point[] {
  return history.map((m, i) => ({ at: m.at, value: read(m, history[i - 1]) }));
}
export function counterRate(
  m: Metrics,
  previous: Metrics | undefined,
  key:
    | "network_rx_bytes"
    | "network_tx_bytes"
    | "disk_read_bytes"
    | "disk_write_bytes",
) {
  if (!previous || m.at <= previous.at || m.at - previous.at > 30) return null;
  const a = m[key],
    b = previous[key];
  return a == null || b == null || a < b
    ? null
    : (a - b) / (m.at - previous.at);
}
export function pathFor(
  points: Point[],
  start: number,
  end: number,
  max: number,
  width = 600,
  height = 150,
) {
  let previous: Point | undefined;
  return points
    .filter((p) => p.at >= start && p.at <= end)
    .map((p) => {
      if (p.value == null || !Number.isFinite(p.value)) {
        previous = undefined;
        return "";
      }
      const move = !previous || p.at - previous.at > 30;
      previous = p;
      return `${move ? "M" : "L"}${(((p.at - start) / Math.max(1, end - start)) * width).toFixed(1)},${(height - (Math.min(max, Math.max(0, p.value)) / Math.max(1, max)) * height).toFixed(1)}`;
    })
    .join(" ");
}
