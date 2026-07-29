#!/usr/bin/env node
// Guard: every tracked path must be checkout-able on Windows.
// A single file named "... 14 steps : loops ..." aborted `git checkout` on NTFS with
// "invalid path", so install.ps1 cloned an empty tree and reported it as a network
// failure. Windows rejects < > : " | ? * and control characters in filenames, plus a
// trailing dot/space and the reserved DOS device names.

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";

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

function runGit(args) {
  const result = spawnSync("git", args, { encoding: "utf8" });
  if (result.status !== 0 || typeof result.stdout !== "string") {
    console.error(
      `FAIL portable-paths: git ${args.join(" ")} did not return a concrete zero status and text output ` +
        `(status=${result.status}, stderr=${result.stderr || ""})`,
    );
    process.exit(1);
  }
  return result.stdout;
}

// `git ls-files` reflects the index, not the working tree: a staged-but-
// unindexed file is invisible to it. This loop only analyses path strings
// (no filesystem read), so an index-only view does not crash the guard the
// way it crashed test_doctrine_parity.mjs -- it silently under-scans
// instead: a file present on disk but absent from the index (an illegal
// Windows name included) sails through green.
const tracked = runGit(["ls-files", "-z"])
  .split(NUL)
  .filter(Boolean);

// `git status --porcelain` covers exactly that gap by reporting the working
// tree's own view of reality, untracked files included. Mirrors
// statusPorcelainFiles() at scripts/run_verify.mjs:875 -- same parse shape
// (skip the 2-char status code, resolve a rename's destination, unquote),
// not a second git-listing approach.
function unquoteGitPath(raw) {
  const s = raw.trim();
  if (s.length >= 2 && s.startsWith('"') && s.endsWith('"')) {
    try {
      return JSON.parse(s);
    } catch {
      return s.slice(1, -1);
    }
  }
  return s;
}

function statusPorcelainPaths() {
  const files = [];
  for (const line of runGit(["status", "--porcelain"]).split("\n")) {
    if (!line) continue;
    const rest = line.slice(3);
    const arrowIdx = rest.indexOf(" -> ");
    const rawPath = arrowIdx === -1 ? rest : rest.slice(arrowIdx + 4);
    const cleaned = unquoteGitPath(rawPath);
    if (cleaned) files.push(cleaned);
  }
  return files;
}

// Union index + working-tree reality, then filter to what actually exists on
// disk: an index entry for a path deleted from disk (not yet staged) is
// excluded, and an untracked-but-real path is included. Mirrors
// deriveScanSet()'s existence filter at
// scripts/tests/test_doctrine_parity.mjs:136.
const scanSet = [...new Set([...tracked, ...statusPorcelainPaths()])]
  .filter((p) => existsSync(p))
  .sort();

const bad = [];
for (const path of scanSet) {
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

console.log(`PASS portable-paths: ${scanSet.length} paths are Windows-safe`);
