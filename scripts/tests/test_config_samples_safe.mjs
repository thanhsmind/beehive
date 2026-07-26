#!/usr/bin/env node
// Guard (ao-2ai-2): the unsafe cli defaults ao-2ai-1's validateModelsConfig
// now flags (--yolo and friends) must never survive in a file bee actually
// ships to onboarded repos — a sample or doc a human copy-pastes is exactly
// where an unsafe default does the most damage, silently, far from the
// validator that would have caught it in a live .bee/config.json.
//
// Two independent checks, plain node asserts, matching the style of
// scripts/tests/test_config_validate.mjs:
//   (a) a targeted text scan for each UNSAFE_CLI_FLAGS alias, restricted to
//       JSON content (whole file for *.json samples; fenced ```json/```jsonc
//       code blocks + inline `"command"` lines for *.md docs) — never a
//       naive whole-file grep, because docs legitimately discuss these flags
//       in prose (e.g. "never use --yolo as the default") without shipping
//       them as a live example;
//   (b) every shipped *.json sample under .bee/ is parsed and run through
//       validateModelsConfig; zero cli-unsafe-flag and zero cli-malformed
//       problems is required. cli-prompt-transport-missing is deliberately
//       NOT asserted zero here — config-sample-cli-executors.json predates
//       promptVia and is out of this cell's scope (ao-2ai-2); fixing it is
//       a separate, un-opened cell.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateModelsConfig, UNSAFE_CLI_FLAGS } from '../../.bee/bin/lib/state.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

// Shipped config samples + docs executor examples a human is expected to
// copy-paste into a real .bee/config.json. "At least" these four (ao-2ai-2
// scope) — a broader sweep found the unsafe --full-auto example that still
// lives in skills/bee-swarming/references/swarming-reference.md (all three
// runtime mirrors) is OUT of this cell's scope (explicitly not-to-touch,
// carried over from ao-2e) and is reported separately, not asserted here.
const JSON_SAMPLES = ['.bee/config-sample.json', '.bee/config-sample-cli-executors.json'];
const DOC_SAMPLES = ['docs/config-reference.md', 'docs/model-presets.md'];

function scannableSegmentsForMarkdown(text) {
  const segments = [];
  // fenced ```json / ```jsonc code blocks
  for (const m of text.matchAll(/```jsonc?\n([\s\S]*?)```/g)) segments.push(m[1]);
  // inline single-line snippets that carry a "command" field (e.g. a
  // markdown table cell) but are not inside a fenced block
  for (const line of text.split('\n')) {
    if (line.includes('"command"')) segments.push(line);
  }
  return segments;
}

function findUnsafeFlags(text) {
  const found = [];
  for (const flag of UNSAFE_CLI_FLAGS) {
    if (text.includes(flag)) found.push(flag);
  }
  return found;
}

// ── (a) targeted scan: *.json samples (whole file is JSON, safe to scan raw) ──

for (const rel of JSON_SAMPLES) {
  const abs = path.join(REPO_ROOT, rel);
  const text = fs.readFileSync(abs, 'utf8');
  const found = findUnsafeFlags(text);
  record(`${rel} contains no UNSAFE_CLI_FLAGS alias`, found.length === 0, `found: ${JSON.stringify(found)}`);
}

// ── (a) targeted scan: *.md docs (only JSON-shaped segments, never raw prose) ──

for (const rel of DOC_SAMPLES) {
  const abs = path.join(REPO_ROOT, rel);
  const text = fs.readFileSync(abs, 'utf8');
  const segments = scannableSegmentsForMarkdown(text);
  const found = new Set();
  for (const seg of segments) {
    for (const flag of findUnsafeFlags(seg)) found.add(flag);
  }
  record(
    `${rel} JSON-shaped executor examples contain no UNSAFE_CLI_FLAGS alias`,
    found.size === 0,
    `found: ${JSON.stringify([...found])}`,
  );
}

// ── (b) every shipped *.json sample validates clean of unsafe/malformed cli ──

for (const rel of JSON_SAMPLES) {
  const abs = path.join(REPO_ROOT, rel);
  const parsed = JSON.parse(fs.readFileSync(abs, 'utf8'));
  const problems = validateModelsConfig(parsed);
  const unsafe = problems.filter((p) => p.code === 'cli-unsafe-flag');
  const malformed = problems.filter((p) => p.code === 'cli-malformed');
  record(
    `${rel} has zero cli-unsafe-flag problems`,
    unsafe.length === 0,
    JSON.stringify(unsafe),
  );
  record(
    `${rel} has zero cli-malformed problems`,
    malformed.length === 0,
    JSON.stringify(malformed),
  );
}

// ── .bee/config-sample.json specifically must validate fully clean (this
//    cell's own fix target) — every cli entry in it now declares promptVia.

{
  const abs = path.join(REPO_ROOT, '.bee/config-sample.json');
  const parsed = JSON.parse(fs.readFileSync(abs, 'utf8'));
  const problems = validateModelsConfig(parsed);
  record('.bee/config-sample.json validates with zero problems of any kind', problems.length === 0, JSON.stringify(problems));
}

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
