#!/usr/bin/env node
// test_agents_budget.mjs — guards AGENTS.md by content, not size (see
// docs/history/budget-fence-removal/CONTEXT.md D1, D5): exactly one ordered
// BEE:START / BEE:END marker pair in root AGENTS.md, the managed block
// byte-identical to the template it was rendered from, and every doctrine
// section present with no duplicates — a silent hand-edit, a stale render,
// or a dropped section would otherwise go unnoticed.
//
// Reshape (census-debt payoff, 2026-08): the block no longer carries a
// numbered "## Critical rules" roster — the refocus rewrote it as thematic
// `##` sections (workflow boundaries, judgment, session care, delegation,
// communication, guardrails). What this suite pins therefore moved from a
// rule-number roster to a SECTION roster: a diet may compress a section's
// body, never drop the section. The per-anchor content of the always-loaded
// doctrine (fan-out rubric, native-wait pointer, two-kind handoff,
// communication turn shape, cli-gather rider) is pinned by the census checks
// in packages/bee/tests/test_misc.mjs — this file deliberately does not
// duplicate those regexes.

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
// identical managed block) and the section roster further down (every
// section present, none dropped, none duplicated). Those checks catch a
// silent hand-edit, a stale render, or a vanished section — the failure
// modes that matter, independent of length.

// The doctrine sections of the always-loaded block. Adding a section is
// fine — update this roster deliberately so the count stays a decision.
// Dropping or renaming one must fail here by name.
const EXPECTED_SECTIONS = [
  "Bee workflow",
  "Judgment and deviation",
  "Start a session",
  "Prove, then say so",
  "Work in parallel, coordinate through the store",
  "Multi-session etiquette",
  "Capture what settles",
  "Communication",
  "Care for the session",
  "Guardrails",
  "Deep contracts",
];

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

// ─── structural guard: no section may vanish in a future diet ─────────────
// A diet compresses wording; it must never drop a doctrine section outright.
// Nothing else in this suite distinguishes "cut 400 bytes of restated
// elaboration" from "cut the Guardrails section" — these checks are what
// catch that specific failure, whatever the surrounding text's length.

function sectionHeadings(text) {
  return [...text.matchAll(/^## (.+)$/gm)].map((m) => m[1].trim());
}

function assertSectionRoster(text, label) {
  const found = sectionHeadings(text);
  for (const heading of EXPECTED_SECTIONS) {
    assert(
      found.includes(heading),
      `${label}: doctrine section "## ${heading}" is missing — a diet may compress a section's body, never drop the section`,
    );
  }
  const duplicates = found.filter((h, i) => found.indexOf(h) !== i);
  assert(
    duplicates.length === 0,
    `${label}: section heading(s) ${[...new Set(duplicates)].map((h) => `"## ${h}"`).join(", ")} appear more than once`,
  );
}

check("the block still carries every doctrine section, none dropped, none duplicated", () => {
  assertSectionRoster(templateText, "packages/bee/AGENTS.block.md");
  assertSectionRoster(rootText, "AGENTS.md");
});

check("negative control: a fixture missing the Communication section fails the roster check, naming the section", () => {
  const mutated = templateText.replace(/^## Communication$/m, "## Comms");
  assert(mutated !== templateText, "mutation fixture must actually alter the block");
  let message = null;
  try {
    assertSectionRoster(mutated, "mutation");
  } catch (error) {
    message = error.message;
  }
  assert(message !== null, "the roster check must reject a block with the Communication section renamed away");
  assert(
    /"## Communication" is missing/.test(message),
    `the failure must name the missing section, got: ${message}`,
  );
});

check("the Guardrails section survives, and the terminal-home sections state their rules rather than defer out", () => {
  assert(
    /^## Guardrails/m.test(templateText),
    "the Guardrails section is where bee-hive/SKILL.md sends readers for the never-retry rule — its heading must survive any cut",
  );
  // Terminal homes: sections whose operative text lives HERE (the census
  // checks in test_misc.mjs pin their exact anchors). A future diet that
  // replaces one of their bodies with a bare pointer builds a loop in which
  // the rule's full text lives nowhere at all.
  for (const heading of ["Start a session", "Communication", "Guardrails", "Multi-session etiquette"]) {
    const section = new RegExp(`^## ${heading}$\\n\\n([^\\n]+)`, "m").exec(templateText);
    assert(section, `terminal-home section "## ${heading}" is missing entirely`);
    assert(
      !/^\s*(See|Full rule|Full contract|Full mechanics)\b/i.test(section[1]),
      `"## ${heading}" is a terminal home and must state the rule itself, not point outward — ` +
        `found a bare cross-reference: "${section[1].slice(0, 80)}"`,
    );
  }
});

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
