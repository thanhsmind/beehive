#!/usr/bin/env node
// Guard: every tracked path must be checkout-able on Windows.
// A single file named "... 14 steps : loops ..." aborted `git checkout` on NTFS with
// "invalid path", so install.ps1 cloned an empty tree and reported it as a network
// failure. Windows rejects < > : " | ? * and control characters in filenames, plus a
// trailing dot/space and the reserved DOS device names.

import { spawnSync } from "node:child_process";

const NUL = String.fromCharCode(0);
const ILLEGAL_CHARS = new Set(['<', '>', ':', '"', '|', '?', '*']);
const RESERVED = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\.|$)/i;
const TRAILING = /[ .]$/;

function illegalIn(segment) {
  for (const ch of segment) {
    if (ILLEGAL_CHARS.has(ch)) return ch;
    if (ch.charCodeAt(0) < 32) return "control character";
  }
  return null;
}

const gitResult = spawnSync("git", ["ls-files", "-z"], { encoding: "utf8" });
if (gitResult.status !== 0 || typeof gitResult.stdout !== "string") {
  console.error(
    "FAIL portable-paths: git ls-files did not return a concrete zero status and text output " +
      `(status=${gitResult.status}, stderr=${gitResult.stderr || ""})`,
  );
  process.exit(1);
}

const tracked = gitResult.stdout
  .split(NUL)
  .filter(Boolean);

const bad = [];
for (const path of tracked) {
  for (const segment of path.split("/")) {
    const ch = illegalIn(segment);
    if (ch) bad.push([path, `illegal character on Windows: ${ch}`]);
    else if (RESERVED.test(segment)) bad.push([path, "reserved DOS device name"]);
    else if (TRAILING.test(segment)) bad.push([path, "trailing dot or space"]);
  }
}

if (bad.length) {
  console.error(`FAIL portable-paths: ${bad.length} tracked path(s) cannot be checked out on Windows`);
  for (const [path, why] of bad) console.error(`  ${path}\n    -> ${why}`);
  process.exit(1);
}

console.log(`PASS portable-paths: ${tracked.length} tracked paths are Windows-safe`);
