#!/usr/bin/env node
// test_agents_budget.mjs — guards AGENTS.md by content, not size (see
// docs/history/budget-fence-removal/CONTEXT.md D1, D5): exactly one ordered
// BEE:START / BEE:END marker pair in root AGENTS.md, the managed block
// byte-identical to the template it was rendered from, and every numbered
// critical rule present with no gaps or duplicates — a silent hand-edit, a
// stale render, or a dropped rule would otherwise go unnoticed.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");

const TEMPLATE_PATH = path.join(REPO_ROOT, "packages", "bee", "AGENTS.block.md");
const ROOT_AGENTS_PATH = path.join(REPO_ROOT, "AGENTS.md");

const MARKER_START = "<!-- BEE:START -->";
const MARKER_END = "<!-- BEE:END -->";

// budget-fence-removal D1/D5 — AGENTS.md carries no enforced size ceiling.
// A size number is never a standing law here (D1): a diet is a deliberate
// one-off pass, not a permanent gate. What this suite still enforces is
// meaning: the render contract below (one ordered marker pair, byte-
// identical managed block) and the critical-rule roster further down (every
// rule present, none dropped, the terminal-home rules keep their full
// text). Those checks catch a silent hand-edit, a stale render, or a
// vanished rule — the failure modes that matter, independent of length.

// The 17 numbered critical rules, and the four whose FULL text is terminal
// here (bee-hive/SKILL.md line "Rules 2-4, 12 are in `AGENTS.md`
// (auto-loaded)" and its rule 12 pointing at "AGENTS.md Guardrails"). A
// future diet that answers one of those defer-backs with a defer-out builds a
// pointer loop in which the rule's full text lives nowhere at all.
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

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const templateText = fs.readFileSync(TEMPLATE_PATH, "utf8");
const rootText = fs.readFileSync(ROOT_AGENTS_PATH, "utf8");

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
// A diet compresses wording; it must never drop a rule outright. Nothing
// else in this suite distinguishes "cut 400 bytes of restated elaboration"
// from "cut critical rule 7" — these two checks are what catch that
// specific failure, whatever the surrounding text's length.

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
if (failed > 0) process.exit(1);
