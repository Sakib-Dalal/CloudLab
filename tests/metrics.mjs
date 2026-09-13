import assert from "node:assert/strict";
import { counterRate, fresh, pathFor, samples } from "../src/lib/metrics.ts";
const before = { at: 100, network_rx_bytes: 1000 };
assert.equal(
  counterRate({ at: 110, network_rx_bytes: 1500 }, before, "network_rx_bytes"),
  50,
);
assert.equal(
  counterRate({ at: 110, network_rx_bytes: 10 }, before, "network_rx_bytes"),
  null,
  "A reset is not negative throughput",
);
assert.equal(
  counterRate({ at: 140, network_rx_bytes: 1500 }, before, "network_rx_bytes"),
  null,
  "A collection gap is not a continuous measurement",
);
assert.equal(counterRate(before, before, "network_rx_bytes"), null);
assert.equal(fresh({ at: 100 }, 129), true);
assert.equal(fresh({ at: 100 }, 130), false);
assert.equal(fresh(null, 100), false);
assert.equal(samples([{ at: 100 }], { at: 100 }).length, 1);
assert.equal(samples([{ at: 100 }], { at: 110 }).length, 2);
const path = pathFor(
  [
    { at: 100, value: 10 },
    { at: 110, value: 20 },
    { at: 120, value: null },
    { at: 130, value: 30 },
    { at: 170, value: 20 },
  ],
  90,
  180,
  100,
);
assert.equal(
  (path.match(/M/g) || []).length,
  3,
  "Null samples and time gaps break the chart line",
);
assert.equal((path.match(/L/g) || []).length, 1);
assert.equal(
  pathFor([{ at: 100, value: 10 }], 110, 120, 100),
  "",
  "Samples outside the time window are excluded",
);
console.log(
  "PASS throughput rates, reset/gap handling, freshness, history deduplication, and chart continuity",
);

// A background tab may suspend timers; simulated nodes should resume reporting.
const { demoData, advanceDemo } = await import("../src/lib/demo.ts");
const demo = structuredClone(demoData);
const resumedAt = Math.floor(Date.now() / 1000) + 7200;
advanceDemo(demo, resumedAt);
assert.equal(demo.nodes[0].last_seen, resumedAt);
assert.equal(demo.nodes[0].metrics.at, resumedAt);
assert.ok(demo.nodes.find((n) => n.id === "n4").last_seen < resumedAt - 45);
