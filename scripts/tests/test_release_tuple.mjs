#!/usr/bin/env node
// Guard: the release-version tuple must never desync. DIST-05 (SPEC), decision
// ed0b2920: BEE_VERSION is duplicated across the packages/bee/lib mirror and the
// live .bee/bin/lib copy, and the two plugin manifests carry their own
// top-level `.version`. All four must always read the same value — this is
// the mechanical guard that catches a release that bumped one and missed
// another.
//
// WHERE the four components live, and how each is read, is defined once in
// scripts/lib/release-tuple.mjs (decision cba8b832). This file only runs the
// check; scripts/bump_version.mjs writes them. Both import the same registry.

import { COMPONENTS, readComponentVersion, checkTupleEquality } from "../lib/release-tuple.mjs";

// ─── internal self-test: prove the checker actually bites ─────────────────
// Feed checkTupleEquality() a synthetic desynced tuple built in-memory only —
// no real file is touched — and assert it is flagged. This proves the
// equality check is not a rubber stamp before trusting it on the real tuple.
{
  const syntheticEntries = [
    { name: "synthetic-a", version: "0.1.44" },
    { name: "synthetic-b", version: "0.1.44" },
    { name: "synthetic-c (desynced)", version: "0.1.43" },
    { name: "synthetic-d", version: "0.1.44" },
  ];

  const selfTestResult = checkTupleEquality(syntheticEntries);

  if (selfTestResult.ok || !selfTestResult.desynced.some((e) => e.name === "synthetic-c (desynced)")) {
    console.error("FAIL test_release_tuple: internal self-test did not catch a synthetic desynced tuple");
    console.error(`      synthetic entries: ${JSON.stringify(syntheticEntries)}`);
    console.error(`      checker result: ${JSON.stringify(selfTestResult)}`);
    process.exit(1);
  }

  console.log("PASS test_release_tuple: internal self-test — checker correctly flags a synthetic desynced tuple");
}

// ─── real check: THIS repo's real release tuple ────────────────────────────

const entries = [];
for (const component of COMPONENTS) {
  try {
    const version = readComponentVersion(component);
    entries.push({ name: component.name, version });
  } catch (error) {
    console.error(`FAIL test_release_tuple: ${error.message}`);
    process.exit(1);
  }
}

const result = checkTupleEquality(entries);
if (!result.ok) {
  console.error(`FAIL test_release_tuple: release-version tuple is desynced across ${result.desynced.length} component(s):`);
  for (const entry of entries) {
    const marker = result.desynced.includes(entry) ? " <-- DESYNCED" : "";
    console.error(`      ${entry.name}: ${entry.version}${marker}`);
  }
  console.error("      FIX: run `node scripts/bump_version.mjs <version>` to set every component from one command.");
  process.exit(1);
}

console.log(`PASS test_release_tuple: all ${entries.length} components agree on version ${entries[0].version} (${entries.map((e) => e.name).join(", ")})`);
