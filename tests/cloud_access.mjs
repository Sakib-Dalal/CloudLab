import assert from "node:assert/strict";
import { cloudCommand, publicIp } from "../src/lib/cloudAccess.ts";
for (const address of [
  "127.0.0.1",
  "10.1.2.3",
  "172.31.1.2",
  "169.254.169.254",
  "192.168.0.1",
  "192.0.2.1",
  "198.51.100.1",
  "203.0.113.1",
  "224.0.0.1",
  "100.64.0.1",
  "8.8.8.8; echo bad",
  "01.2.3.4",
  "8.8.8.8:443",
])
  assert.equal(publicIp(address), null, address);
assert.equal(publicIp(" 8.8.8.8 "), "8.8.8.8");
assert.equal(
  cloudCommand("aws", "", true),
  "sudo python3 scripts/cloud-access.py --provider aws --apply",
);
assert.equal(cloudCommand("other", "", true), null);
assert.equal(
  cloudCommand("other", "1.1.1.1", true),
  "sudo python3 scripts/cloud-access.py --provider other --public-ip 1.1.1.1 --apply",
);
assert.equal(
  cloudCommand("aws", "1.1.1.1", false),
  "sudo python3 scripts/cloud-access.py --provider aws --public-ip 1.1.1.1 --apply",
);
assert.equal(cloudCommand("aws;bad", "1.1.1.1", false), null);
console.log("Cloud access command and address checks passed.");
