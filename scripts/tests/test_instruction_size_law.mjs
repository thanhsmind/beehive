#!/usr/bin/env node
// test_instruction_size_law.mjs — regression guard for budget-fence-removal
// D1/D5 (docs/history/budget-fence-removal/CONTEXT.md, decision 8f63adb4): a
// size ceiling on instruction text is never a standing law in this repo.
// Slice 1 deleted scripts/skill_budget_fence.mjs and
// scripts/skill-body-budget.json and stripped the byte thresholds out of
// scripts/tests/test_agents_budget.mjs. A removal is verified by its
// invariants, not by the names it deletes (critical pattern 20260711) — an
// `rg` for "skill_budget_fence" would pass forever while someone reintroduced
// the same law under a new name. So this suite asserts the DEFECT CLASS:
//
//   1. No byte-or-line ceiling on instruction text survives anywhere in
//      scripts/, under any identifier or filename.
//   2. The meaning guards that replaced the fence in
//      scripts/tests/test_agents_budget.mjs still bite: both its critical-
//      rule-roster guard and its byte-identical render guard exit the real
//      suite non-zero when independently violated — a guard exercised in
//      only one state is a law with a hole (critical pattern 20260713).
//
// Never mutates the live AGENTS.md or packages/bee/AGENTS.block.md (critical
// rule 16): assertion 2 copies both into a throwaway fixture directory and
// spawns the real, unmodified test_agents_budget.mjs against the copies.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.join(__dirname, "..", "..");
const SCRIPTS_ROOT = path.join(REPO_ROOT, "scripts");
const AGENTS_BUDGET_SUITE_PATH = path.join(__dirname, "test_agents_budget.mjs");
const REAL_TEMPLATE_PATH = path.join(REPO_ROOT, "packages", "bee", "AGENTS.block.md");

const MARKER_START = "<!-- BEE:START -->";
const MARKER_END = "<!-- BEE:END -->";

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

// ═══════════════════════════════════════════════════════════════════════════
// Invariant 1 — no byte/line ceiling on instruction text survives anywhere
// in scripts/, under any name. Structural detectors, not filename greps.
// ═══════════════════════════════════════════════════════════════════════════

// Shape A: an identifier that names BOTH a size unit (byte/line) AND a
// limiting concept (hard-fail/warn/max/limit/ceiling/budget/threshold),
// assigned a numeric literal — as a `const`/`let`/`var` declaration or as an
// object/class field (`identifier: N` / `identifier = N`). Order-independent
// on purpose: catches `HARD_FAIL_BYTES = 15000` exactly as it would catch a
// future `skillTextCeilingBytes = 9001` or `LINE_LIMIT = 200` — the two
// deleted constant names are two instances of this shape, not the shape
// itself.
const UNIT_WORD_RE = /byte|line/i;
const LIMIT_WORD_RE = /hard.?fail|warn|max|limit|ceil|budget|threshold/i;
const ASSIGNMENT_RE = /\b(?:const|let|var)?\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*[:=]\s*\d+/g;

// Shape B: a runtime comparison of a computed size (`.length`, `byteLength(`,
// `.size`) against a numeric literal, on the same line as a reference to
// skill/instruction text — the shape of "is this SKILL.md too big", inline,
// with no named constant at all.
const INSTRUCTION_TEXT_REF_RE = /skills?[\\/]|SKILL\.md/;
const SIZE_COMPARISON_RE = /(?:\.length|byteLength\(|\.size)\s*[><]=?\s*\d{2,}/;

function sizeLawShapeHits(text) {
  const hits = [];
  const lines = text.split("\n");
  lines.forEach((line, i) => {
    ASSIGNMENT_RE.lastIndex = 0;
    let m;
    while ((m = ASSIGNMENT_RE.exec(line))) {
      const name = m[1];
      if (UNIT_WORD_RE.test(name) && LIMIT_WORD_RE.test(name)) {
        hits.push({ line: i + 1, text: line.trim(), rule: `ceiling-shaped identifier "${name}"` });
      }
    }
    if (INSTRUCTION_TEXT_REF_RE.test(line) && SIZE_COMPARISON_RE.test(line)) {
      hits.push({ line: i + 1, text: line.trim(), rule: "inline size comparison against instruction text" });
    }
  });
  return hits;
}

// Shape C: a JSON table whose values are ALL numbers, keyed at least in part
// by something skill-shaped — the shape scripts/skill-body-budget.json had
// (a per-skill byte baseline), under any filename.
function baselineShapeHits(value, keyPath = "") {
  const hits = [];
  if (value && typeof value === "object" && !Array.isArray(value)) {
    const entries = Object.entries(value);
    const numericEntries = entries.filter(([, v]) => typeof v === "number");
    // At least half the entries numeric (tolerates a "note"/metadata field,
    // the way scripts/verify-cache-inputs.json carries one today), and at
    // least one of those numeric entries keyed by something skill-shaped.
    const skillKeyedNumeric = entries.some(
      ([k, v]) => typeof v === "number" && (/skill/i.test(k) || /SKILL\.md$/i.test(k)),
    );
    if (numericEntries.length >= 2 && numericEntries.length / entries.length >= 0.5 && skillKeyedNumeric) {
      hits.push({ path: keyPath || "<root>" });
    }
    for (const [k, v] of entries) {
      hits.push(...baselineShapeHits(v, keyPath ? `${keyPath}.${k}` : k));
    }
  }
  return hits;
}

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(full, out);
    } else {
      out.push(full);
    }
  }
  return out;
}

// ── Vacuity guard ─────────────────────────────────────────────────────────
// A law that scans a set must FAIL when that set is unexpectedly empty. A
// green check over an empty scan set is worse than no check: it reports
// "the law holds" when in fact its SUBJECT vanished. That is precisely what
// happens to this suite at the R6 Node-runtime cutover, when `scripts/**`
// is deleted (plans/rust-port.md § "Hard blockers for deleting the Node
// runtime"; plans/cutover-readiness.md). Every scan in this file goes
// through here, and the refusal names what it expected to find and where.
//
// The post-cutover home of invariant 1 is the Rust suite —
// `packages/bee-rs/crates/bee/tests/instruction_laws.rs` — which carries the
// same detectors re-pointed at `packages/bee-rs/crates/**/*.rs`. This guard
// is what makes the handover safe in either order: if `scripts/**` goes away
// while this file is still wired up, it goes RED here, loudly, instead of
// green over nothing.
function requireNonEmptyScanSet({ label, roots, expectation, minimum, files }) {
  const rel = (p) => path.relative(REPO_ROOT, p) || ".";
  const missing = roots.filter((r) => !fs.existsSync(r));
  assert(
    missing.length === 0,
    `${label}: scan root(s) missing — ${missing.map(rel).join(", ")}. Expected ${expectation}. ` +
      `A law whose scan root no longer exists must be re-pointed or explicitly retired ` +
      `(see plans/cutover-readiness.md), never left to pass over nothing.`,
  );
  assert(
    files.length >= minimum,
    `${label}: scanned ${files.length} file(s) under [${roots.map(rel).join(", ")}], expected at least ` +
      `${minimum}. Expected ${expectation}. An empty (or implausibly small) scan set means this law is ` +
      `asserting nothing — re-point it or retire it (see plans/cutover-readiness.md).`,
  );
  return files;
}

check("no byte/line-ceiling-shaped construct on instruction text survives anywhere in scripts/", () => {
  const files = requireNonEmptyScanSet({
    label: "invariant 1 — ceiling-shaped constructs",
    roots: [SCRIPTS_ROOT],
    expectation: "the repo's Node script tree (scripts/**/*.{mjs,js,cjs})",
    minimum: 10,
    files: walk(SCRIPTS_ROOT).filter((f) => /\.(mjs|js|cjs)$/.test(f) && f !== __filename),
  });
  const offenders = [];
  for (const file of files) {
    const text = fs.readFileSync(file, "utf8");
    for (const hit of sizeLawShapeHits(text)) {
      offenders.push(`${path.relative(REPO_ROOT, file)}:${hit.line} (${hit.rule}): ${hit.text}`);
    }
  }
  assert(
    offenders.length === 0,
    `a size ceiling on instruction text is never a standing law here (D1) — found ` +
      `${offenders.length} offender(s):\n${offenders.join("\n")}`,
  );
});

check("no per-skill byte-baseline table survives anywhere in scripts/**/*.json", () => {
  const files = requireNonEmptyScanSet({
    label: "invariant 1 — per-skill byte-baseline tables",
    roots: [SCRIPTS_ROOT],
    expectation: "at least one JSON data file under scripts/ (the genre scripts/skill-body-budget.json belonged to)",
    minimum: 1,
    files: walk(SCRIPTS_ROOT).filter((f) => f.endsWith(".json")),
  });
  const offenders = [];
  for (const file of files) {
    let parsed;
    try {
      parsed = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch {
      continue; // not JSON's job to be valid here; other suites own that
    }
    for (const hit of baselineShapeHits(parsed)) {
      offenders.push(`${path.relative(REPO_ROOT, file)} @ ${hit.path}`);
    }
  }
  assert(
    offenders.length === 0,
    `found ${offenders.length} per-skill byte-baseline-shaped table under scripts/:\n${offenders.join("\n")}`,
  );
});

check("negative control: the ceiling-identifier detector catches a reintroduction under a brand-new name", () => {
  const fixture = [
    "// pretend this file was never named anything like the deleted fence",
    "const skillTextCeilingBytes = 9001;",
    "export function tooBig(n) { return n > skillTextCeilingBytes; }",
  ].join("\n");
  const hits = sizeLawShapeHits(fixture);
  assert(
    hits.some((h) => h.rule.includes("ceiling-shaped identifier")),
    "the detector must flag a renamed byte-ceiling constant, not just the two deleted names",
  );
});

check("negative control: the inline-comparison detector catches a ceiling with no named constant at all", () => {
  const fixture = "if (fs.readFileSync('skills/renamed-skill/SKILL.md').length > 9001) { throw new Error('too big'); }";
  const hits = sizeLawShapeHits(fixture);
  assert(
    hits.some((h) => h.rule.includes("inline size comparison")),
    "the detector must flag an inline size-vs-instruction-text comparison with no named ceiling constant",
  );
});

// ── Negative controls for the vacuity guard itself ────────────────────────
// "Prove it fails correctly, naming what it expected" is a permanent part of
// this suite, not a probe run once and deleted (the house convention set by
// test_scan_set_hygiene.mjs's --selftest). Both arms of the guard are
// exercised: an ABSENT root (what deleting scripts/ produces) and a PRESENT
// but EMPTY root (what emptying it produces).

check("negative control: the vacuity guard refuses an ABSENT scan root, naming what it expected", () => {
  const gone = path.join(REPO_ROOT, ".bee", "tmp", `no-such-scan-root-${process.pid}-${crypto.randomUUID()}`);
  assert(!fs.existsSync(gone), "fixture precondition: the absent-root path must really not exist");
  let error = null;
  try {
    requireNonEmptyScanSet({
      label: "fixture law",
      roots: [gone],
      expectation: "the repo's Node script tree (scripts/**/*.{mjs,js,cjs})",
      minimum: 10,
      files: [],
    });
  } catch (e) {
    error = e;
  }
  assert(error, "an absent scan root must be a hard failure — this is the exact shape deleting scripts/ produces");
  assert(
    /scan root\(s\) missing/.test(error.message) && /Node script tree/.test(error.message),
    `the refusal must name the missing root AND what it expected to find there, got: ${error.message}`,
  );
});

check("negative control: the vacuity guard refuses a PRESENT but empty scan root, naming what it expected", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-size-law-vacuity-"));
  try {
    const files = walk(dir).filter((f) => /\.(mjs|js|cjs)$/.test(f));
    assert(files.length === 0, "fixture precondition: the temp scan root must be empty");
    let error = null;
    try {
      requireNonEmptyScanSet({
        label: "fixture law",
        roots: [dir],
        expectation: "the repo's Node script tree (scripts/**/*.{mjs,js,cjs})",
        minimum: 10,
        files,
      });
    } catch (e) {
      error = e;
    }
    assert(error, "an empty scan set must be a hard failure, never a silent pass");
    assert(
      /scanned 0 file\(s\)/.test(error.message) && /expected at least 10/.test(error.message),
      `the refusal must name the count it got and the minimum it expected, got: ${error.message}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

check("negative control: the JSON-baseline detector catches a renamed per-skill byte table", () => {
  const fixture = {
    note: "a renamed baseline, not scripts/skill-body-budget.json",
    "skills/renamed-skill/SKILL.md": 4096,
    "skills/other-skill/SKILL.md": 2048,
  };
  const hits = baselineShapeHits(fixture);
  assert(hits.length > 0, "the detector must flag a renamed per-skill byte-baseline table");
});

// ═══════════════════════════════════════════════════════════════════════════
// Invariant 2 — the meaning guards in test_agents_budget.mjs still bite.
// Never mutates the live AGENTS.md / AGENTS.block.md: both are copied into a
// throwaway fixture directory, alongside a byte-copy of the real suite file,
// so REPO_ROOT (computed from the copied suite's own __dirname) resolves
// inside the fixture. The real, unmodified suite is then spawned against the
// copies and its process exit code is the proof.
// ═══════════════════════════════════════════════════════════════════════════

const realSuiteSource = fs.readFileSync(AGENTS_BUDGET_SUITE_PATH, "utf8");
const realTemplateText = fs.readFileSync(REAL_TEMPLATE_PATH, "utf8");

// Census-debt payoff (2026-08): test_agents_budget.mjs's roster moved from a
// numbered `## Critical rules` list to a doctrine-SECTION roster (the block
// was rewritten as thematic sections in the refocus). The mutation built here
// mirrors the reshaped suite's own section-roster guard: dropping a section
// heading must make the real suite exit non-zero, naming the section.
function removeSectionHeading(templateText, heading) {
  const mutated = templateText.replace(new RegExp(`^## ${heading}$\\n`, "m"), "");
  assert(mutated !== templateText, `fixture mutation must actually remove the "## ${heading}" heading`);
  return mutated;
}

// Builds `<rootDir>/AGENTS.md` whose managed block, sliced the same way the
// real render-identical check slices it, is exactly `templateText`.
function renderRootAgents(templateText) {
  return (
    "# Fixture AGENTS.md — not the live repo file\n\n" +
    `${MARKER_START}\n${templateText.replace(/\n$/, "")}\n${MARKER_END}\n`
  );
}

function makeFixtureDir() {
  const dir = path.join(REPO_ROOT, ".bee", "tmp", `test-instruction-size-law-${process.pid}-${crypto.randomUUID()}`);
  fs.mkdirSync(path.join(dir, "scripts", "tests"), { recursive: true });
  fs.mkdirSync(path.join(dir, "packages", "bee"), { recursive: true });
  fs.writeFileSync(path.join(dir, "scripts", "tests", "test_agents_budget.mjs"), realSuiteSource);
  return dir;
}

function writeFixtureAgents(dir, templateText, rootText) {
  fs.writeFileSync(path.join(dir, "packages", "bee", "AGENTS.block.md"), templateText);
  fs.writeFileSync(path.join(dir, "AGENTS.md"), rootText);
}

function runFixtureSuite(dir) {
  const suitePath = path.join(dir, "scripts", "tests", "test_agents_budget.mjs");
  const result = spawnSync(process.execPath, [suitePath], { cwd: dir, encoding: "utf8" });
  return { status: result.status, output: `${result.stdout || ""}\n${result.stderr || ""}` };
}

check("fixture harness sanity: the real suite, byte-copied and pointed at a matching fixture, exits 0", () => {
  const dir = makeFixtureDir();
  try {
    writeFixtureAgents(dir, realTemplateText, renderRootAgents(realTemplateText));
    const { status, output } = runFixtureSuite(dir);
    assert(
      status === 0,
      `expected the fixture harness to reproduce a green run before seeding any violation, got exit ${status}:\n${output}`,
    );
    assert(
      output.includes("PASS negative control: a fixture missing the Communication section fails the roster check, naming the section"),
      "test_agents_budget.mjs's own built-in negative control must still exist and pass after budget-fence-removal's edits — " +
        `got:\n${output}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

check("the roster guard still bites: removing a doctrine section makes the real suite exit non-zero", () => {
  // "Multi-session etiquette" (not "Communication", which the suite's own
  // built-in negative control already uses as its fixture target) so this
  // failure is isolated to exactly the main roster check, not entangled with
  // that control's own assertions.
  const mutatedTemplate = removeSectionHeading(realTemplateText, "Multi-session etiquette");
  const dir = makeFixtureDir();
  try {
    // Root's rendered block is derived from the SAME mutated template, so
    // the render-identical check still passes — only the roster is broken.
    writeFixtureAgents(dir, mutatedTemplate, renderRootAgents(mutatedTemplate));
    const { status, output } = runFixtureSuite(dir);
    assert(status !== 0, `expected a non-zero exit with the Multi-session etiquette section missing, got exit ${status}:\n${output}`);
    assert(
      /"## Multi-session etiquette" is missing/.test(output),
      `expected the failure to name the missing section, got:\n${output}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

check("the byte-identical render guard still bites: a drifted managed block makes the real suite exit non-zero", () => {
  // Template is untouched (full section roster, matches the real one) — only the
  // block rendered inside AGENTS.md's markers drifts from it, isolating the
  // failure to exactly the render-identical check.
  const driftedBlock = `${realTemplateText.replace(/\n$/, "")} render-drift-marker\n`;
  const dir = makeFixtureDir();
  try {
    writeFixtureAgents(dir, realTemplateText, renderRootAgents(driftedBlock));
    const { status, output } = runFixtureSuite(dir);
    assert(status !== 0, `expected a non-zero exit with a drifted rendered block, got exit ${status}:\n${output}`);
    assert(
      /not byte-identical/.test(output),
      `expected the failure to name the render mismatch, got:\n${output}`,
    );
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
