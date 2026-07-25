#!/usr/bin/env node
// test_agents_budget.mjs — D13 (codex-native-runtime-v2, cell cnr2-15):
// AGENTS.md stays under a hard 20 KiB budget, measured in UTF-8 bytes, for
// both the human-edited template source and this repo's rendered root file.
// Also guards the render contract itself: exactly one ordered BEE:START /
// BEE:END marker pair in root AGENTS.md, and the managed block between those
// markers must be byte-identical to the template it was rendered from — a
// silent hand-edit or a stale render would otherwise drift undetected.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");

const TEMPLATE_PATH = path.join(REPO_ROOT, "skills", "bee-hive", "templates", "AGENTS.block.md");
const ROOT_AGENTS_PATH = path.join(REPO_ROOT, "AGENTS.md");

const MARKER_START = "<!-- BEE:START -->";
const MARKER_END = "<!-- BEE:END -->";

const HARD_FAIL_BYTES = 20480; // 20 KiB
const WARN_BYTES = 18000;

let passed = 0;
let failed = 0;
function check(name, fn) {
  try {
    fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error && error.stack ? error.stack : error}`);
  }
}

function utf8Bytes(text) {
  return Buffer.byteLength(text, "utf8");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const templateText = fs.readFileSync(TEMPLATE_PATH, "utf8");
const rootText = fs.readFileSync(ROOT_AGENTS_PATH, "utf8");

const templateBytes = utf8Bytes(templateText);
const rootBytes = utf8Bytes(rootText);

// ─── dual-target byte budget ───────────────────────────────────────────────

check("template block stays under the 20 KiB hard budget", () => {
  assert(
    templateBytes < HARD_FAIL_BYTES,
    `skills/bee-hive/templates/AGENTS.block.md is ${templateBytes} UTF-8 bytes, ` +
      `at or over the ${HARD_FAIL_BYTES}-byte hard budget`,
  );
  if (templateBytes >= WARN_BYTES) {
    console.warn(
      `WARN template block is ${templateBytes} bytes (warn threshold ${WARN_BYTES}, ` +
        `hard budget ${HARD_FAIL_BYTES})`,
    );
  }
});

check("rendered root AGENTS.md stays under the 20 KiB hard budget", () => {
  assert(
    rootBytes < HARD_FAIL_BYTES,
    `AGENTS.md is ${rootBytes} UTF-8 bytes, at or over the ${HARD_FAIL_BYTES}-byte hard budget`,
  );
  if (rootBytes >= WARN_BYTES) {
    console.warn(
      `WARN rendered AGENTS.md is ${rootBytes} bytes (warn threshold ${WARN_BYTES}, ` +
        `hard budget ${HARD_FAIL_BYTES})`,
    );
  }
});

// ─── marker-pair + byte-identical render ──────────────────────────────────

check("root AGENTS.md has exactly one ordered BEE:START/BEE:END marker pair", () => {
  const starts = rootText.split(MARKER_START).length - 1;
  const ends = rootText.split(MARKER_END).length - 1;
  assert(starts === 1, `expected exactly 1 "${MARKER_START}" marker, found ${starts}`);
  assert(ends === 1, `expected exactly 1 "${MARKER_END}" marker, found ${ends}`);

  const startIdx = rootText.indexOf(MARKER_START);
  const endIdx = rootText.indexOf(MARKER_END);
  assert(startIdx !== -1 && endIdx !== -1, "marker pair not found in AGENTS.md");
  assert(startIdx < endIdx, "BEE:START must appear before BEE:END in AGENTS.md");
});

check("managed block in AGENTS.md renders byte-identically to the template", () => {
  const startIdx = rootText.indexOf(MARKER_START);
  const endIdx = rootText.indexOf(MARKER_END);
  assert(startIdx !== -1 && endIdx !== -1, "marker pair not found in AGENTS.md");

  const renderedBlock = rootText.slice(startIdx + MARKER_START.length, endIdx).replace(/^\n/, "").replace(/\n$/, "");
  const templateBlock = templateText.replace(/\n$/, "");

  assert(
    renderedBlock === templateBlock,
    "the block rendered inside AGENTS.md's BEE:START/BEE:END markers is not byte-identical " +
      "to skills/bee-hive/templates/AGENTS.block.md — re-run the onboarding sync",
  );
});

console.log(`\n${passed} passed, ${failed} failed`);
console.log(
  `sizes: template=${templateBytes}B root=${rootBytes}B (warn>=${WARN_BYTES}B, fail>=${HARD_FAIL_BYTES}B)`,
);
if (failed > 0) process.exit(1);
