#!/usr/bin/env node
// test_agents_budget.mjs — D13 (codex-native-runtime-v2, cell cnr2-15):
// AGENTS.md stays under a ratcheted byte budget, measured in UTF-8 bytes, for
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

const TEMPLATE_PATH = path.join(REPO_ROOT, "packages", "bee", "AGENTS.block.md");
const ROOT_AGENTS_PATH = path.join(REPO_ROOT, "AGENTS.md");

const MARKER_START = "<!-- BEE:START -->";
const MARKER_END = "<!-- BEE:END -->";

// agents-block-diet abd-2 — the fence is ratcheted onto the achieved size.
//
// It used to be 20 KiB / 18 KiB warn, set when the block was ~16.2 KB. The
// diet cut the template to 12,573 B and the render to 12,692 B, which left
// the old fence ~8 KB above reality: a budget that far above the real file
// protects nothing, because the file can double before it ever bites, and
// the whole point of the cut was to stop paying those bytes every session
// and after every compaction.
//
// New numbers, measured from the achieved render (the larger of the two, so
// one pair of thresholds governs both targets):
//   hard fail 15,000 B — ~2.3 KB of headroom, room for several ordinary rule
//                        additions before anyone has to think about size.
//   warn       14,000 B — ~1.3 KB, the early signal that the next addition
//                        should come with a corresponding cut.
// The fence guards REGROWTH, not growth: it is deliberately loose enough
// that adding a genuine rule is never blocked, and tight enough that drifting
// back to 16 KB is impossible without someone editing this constant on
// purpose — which is exactly the moment the decision deserves to be visible.
const HARD_FAIL_BYTES = 15000;
const WARN_BYTES = 14000;

// The 17 numbered critical rules, and the four whose FULL text is terminal
// here (bee-hive/SKILL.md line "Rules 2-4, 13 appear in full in AGENTS.md"
// and its rule 13 pointing at "AGENTS.md Guardrails"). A future diet that
// answers one of those defer-backs with a defer-out builds a pointer loop in
// which the rule's full text lives nowhere at all.
//
// validation-diet vd-10 — rule 16 (evidence doctrine, D9) was appended after
// rule 15 rather than inserted mid-list, so TERMINAL_HOME_RULES' indices
// [1, 5, 6, 11] are unaffected by the count moving from 15 to 16.
//
// tick-contract-inline tci-1 (T1/T7) — rule 17 (the progress-tick contract,
// moved out of routing-and-contracts.md so a rule that applies every turn is
// not stored behind an on-demand reference) was likewise appended, not
// inserted, so TERMINAL_HOME_RULES is again unaffected by 16 -> 17.
const EXPECTED_RULE_COUNT = 17;
const TERMINAL_HOME_RULES = [1, 5, 6, 11];

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

check("template block stays under the ratcheted hard budget", () => {
  assert(
    templateBytes < HARD_FAIL_BYTES,
    `packages/bee/AGENTS.block.md is ${templateBytes} UTF-8 bytes, ` +
      `at or over the ${HARD_FAIL_BYTES}-byte hard budget`,
  );
  if (templateBytes >= WARN_BYTES) {
    console.warn(
      `WARN template block is ${templateBytes} bytes (warn threshold ${WARN_BYTES}, ` +
        `hard budget ${HARD_FAIL_BYTES})`,
    );
  }
});

check("rendered root AGENTS.md stays under the ratcheted hard budget", () => {
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
      "to packages/bee/AGENTS.block.md — re-run the onboarding sync",
  );
});

// ─── structural guard: no rule may vanish in a future diet ────────────────
// The byte fence above rewards cutting. Nothing in it distinguishes "cut 400
// bytes of restated elaboration" from "cut critical rule 7", so the fence on
// its own quietly incentivises the wrong cut. These two checks are what make
// the budget safe to enforce.

// The rules live under `## Critical rules`, up to the next `##` heading —
// scoped so that numbered lists elsewhere in the block (Startup, Session
// finish) can never be miscounted as rules.
function criticalRulesSection(text) {
  const start = text.indexOf("\n## Critical rules");
  assert(start !== -1, "AGENTS.block.md must keep its `## Critical rules` heading");
  const rest = text.slice(start + 1);
  const nextHeading = rest.indexOf("\n## ");
  return nextHeading === -1 ? rest : rest.slice(0, nextHeading);
}

function ruleNumbers(text) {
  return [...criticalRulesSection(text).matchAll(/^(\d+)\. /gm)].map((m) => Number(m[1]));
}

function assertRuleRoster(text, label) {
  const found = ruleNumbers(text);
  for (let n = 1; n <= EXPECTED_RULE_COUNT; n += 1) {
    assert(
      found.includes(n),
      `${label}: critical rule ${n} is missing from \`## Critical rules\` — a diet may compress a rule's body, never drop the rule`,
    );
  }
  const duplicates = found.filter((n, i) => found.indexOf(n) !== i);
  assert(
    duplicates.length === 0,
    `${label}: critical rule number(s) ${[...new Set(duplicates)].join(", ")} appear more than once`,
  );
  assert(
    found.length === EXPECTED_RULE_COUNT,
    `${label}: expected exactly ${EXPECTED_RULE_COUNT} numbered critical rules, found ${found.length} (${found.join(", ")}) — ` +
      `adding a rule is fine, but update EXPECTED_RULE_COUNT deliberately so the count stays a decision`,
  );
}

check("the block still carries all 17 numbered critical rules, no gaps, no duplicates", () => {
  assertRuleRoster(templateText, "packages/bee/AGENTS.block.md");
  assertRuleRoster(rootText, "AGENTS.md");
});

check("negative control: a fixture missing rule 7 fails the roster check, naming the number", () => {
  // The mutation must be applied INSIDE the Critical rules section: `## Startup`
  // also has a numbered step 7, and blanking that one changes nothing here —
  // which is precisely the scoping this check exists to prove.
  const section = criticalRulesSection(templateText);
  const withoutRule7 = section.replace(/^7\. .*$/m, "");
  assert(withoutRule7 !== section, "mutation fixture must actually remove critical rule 7");
  const mutated = templateText.replace(section, withoutRule7);
  assert(mutated !== templateText, "mutation fixture must actually alter the block");
  let message = null;
  try {
    assertRuleRoster(mutated, "mutation");
  } catch (error) {
    message = error.message;
  }
  assert(message !== null, "the roster check must reject a block with rule 7 removed");
  assert(
    /critical rule 7 is missing/.test(message),
    `the failure must name the missing number, got: ${message}`,
  );
});

check("the terminal-home rules and the Guardrails heading survive — a defer-out here would be a pointer loop", () => {
  const section = criticalRulesSection(templateText);
  for (const n of TERMINAL_HOME_RULES) {
    const rule = new RegExp(`^${n}\\. (.*)$`, "m").exec(section);
    assert(rule, `critical rule ${n} is missing entirely`);
    assert(
      !/^\s*(See|Full rule|Full contract|Full mechanics)\b/i.test(rule[1]),
      `critical rule ${n} is a terminal home (bee-hive/SKILL.md defers back to it) and must state the rule itself, ` +
        `not point outward — found a bare cross-reference: "${rule[1].slice(0, 80)}"`,
    );
  }
  assert(
    /^## Guardrails/m.test(templateText),
    "the Guardrails section is where bee-hive/SKILL.md sends readers for the never-retry rule — its heading must survive any cut",
  );
});

console.log(`\n${passed} passed, ${failed} failed`);
console.log(
  `sizes: template=${templateBytes}B root=${rootBytes}B (warn>=${WARN_BYTES}B, fail>=${HARD_FAIL_BYTES}B)`,
);
if (failed > 0) process.exit(1);
