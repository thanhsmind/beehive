#!/usr/bin/env node
// test_always_loaded_rules — BLOCKING. Exits non-zero when a rule that applies
// EVERY TURN is reachable only from an on-demand reference file and never named
// by the always-loaded operating block.
//
// ── WHAT THIS PROVES, AND WHAT IT DOES NOT (tick-contract-inline T6) ──────────
// It proves REACHABILITY: an every-turn rule stated in the on-demand reference
// tier is also named by the always-loaded block, so an agent that loads nothing
// else still arrives at the rule. It proves NOTHING about whether the rule was
// FOLLOWED. Nothing in this repo observes agent chat output — no hook parses it,
// no suite asserts on it — so progress-tick EMISSION is NOT enforced here, and
// this file must never be cited as if it were. A check that overstates its
// coverage is how the next gap gets missed.
//
// ── WHY IT EXISTS ────────────────────────────────────────────────────────────
// The progress-tick contract was specified as a mandatory per-step rule, lived
// only in skills/bee-hive/references/routing-and-contracts.md, and the one check
// that could have flagged the gap (scripts/skill_lint.mjs) both mismatched on
// the pointer shape AND always exits 0 — so its warning fired the entire time
// and blocked nobody. Three stacked causes, one survivable class of failure.
// This suite is the class-level gate: it exits 1.
//
// ── HOW THE RULE SET IS DERIVED (tick-contract-inline T5) ────────────────────
// The Deferred-To-Planning question — explicit every-turn marker in the doctrine
// source vs. derivation from the rule's own wording — is settled here as
// DERIVATION FROM WORDING.
//   An authored marker would have to be applied by the same author who just made
//   the mistake of filing an every-turn rule in a reference. Its failure mode is
//   SILENCE — precisely the defect this suite exists to catch, relocated one
//   level up. Wording derivation needs no bookkeeping: the moment doctrine says
//   a rule applies every turn, this suite reacts. Its failure mode is a FALSE
//   POSITIVE — a red build a human clears by moving the rule or rewording the
//   claim — which is the safe direction for a gate to be wrong in.
//
//   THE ONE SEED is EVERY_TURN_PHRASES below: a small vocabulary of how English
//   states per-turn scope. It is a domain fact about wording, not a location and
//   not a roster. No rule name, no heading, no file path, and no rule count is
//   hardcoded anywhere in this file — the rule set, the reference corpus, and
//   the pointer set are all read off the tree at run time. (scripts/skill_lint.mjs
//   takes the opposite bet with a literal three-rule list; that list is why this
//   file does not have one.)
//
// ── SCOPE LIMITS, stated so nothing here reads as broader than it is ─────────
//   * On-demand tier scanned = `*.md` under any `references/` directory inside
//     `skills/`. That is the tier the failure came from.
//   * `skills/*/SKILL.md` bodies are OUT of scope: they are a different tier
//     with a different pointer convention (they cite `AGENTS.md` rule numbers
//     rather than being pointed AT). An every-turn rule authored only in a
//     SKILL.md body is not caught here.
//   * Provenance ledgers are skipped — see isProvenanceLedger(); a file is
//     skipped only when it SELF-DECLARES as a rule→decision map, so a ledger
//     that drops that declaration gets scanned. The exclusion fails toward
//     flagging, never toward silence.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");

const SKILLS_DIR = path.join(REPO_ROOT, "skills");

// The always-loaded layer: the rendered root body an agent gets every session,
// and the template it ships from. Whichever exist are read together — a host
// repo has the former, this repo has both, and test_agents_budget.mjs already
// pins them byte-identical inside the BEE markers.
const ALWAYS_LOADED_CANDIDATES = ["AGENTS.md", path.join("packages", "bee", "AGENTS.block.md")];

// ── THE SEED ────────────────────────────────────────────────────────────────
// Phrases that assert a rule applies on every turn / every step / by default,
// rather than inside some phase, lane, or skill. Deliberately narrow: phrases
// that merely assert strength ("never", "mandatory") or immunity ("no bypass
// level lifts it") are NOT here — a phase-scoped rule can be absolute within
// its phase and still belong in a reference.
const EVERY_TURN_PHRASES = [
  /\bevery turn\b/i,
  /\beach turn\b/i,
  /\bper[-\s]turn\b/i,
  /\bevery message\b/i,
  /\bon by default\b/i,
  /\bevery phase and lane\b/i,
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
    console.error(`FAIL ${name}: ${error && error.message ? error.message : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const squash = (s) => s.replace(/\s+/g, " ").trim();
const norm = (s) => squash(s).toLowerCase();

// ── corpus discovery ────────────────────────────────────────────────────────

// Every `*.md` under any directory named `references` inside skills/. Walked,
// never listed — a new skill or a new reference file joins the corpus with no
// edit here.
function referenceFiles(dir, inReferences = false, out = []) {
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      referenceFiles(full, inReferences || entry.name === "references", out);
    } else if (inReferences && entry.isFile() && entry.name.endsWith(".md")) {
      out.push(full);
    }
  }
  return out;
}

// A provenance ledger maps body rules to the decisions that authorize them; it
// is a citation index, not a rule's home, so a per-turn phrase inside one is a
// reference TO a rule that already lives in a body. Detected from the file's own
// self-declaration, not from its filename — a ledger that stops declaring itself
// is scanned like any other reference.
function isProvenanceLedger(text) {
  return /maps each body rule to the decision/i.test(squash(text.slice(0, 2000)));
}

// ── sectioning ──────────────────────────────────────────────────────────────

// Split a markdown file into sections at `##`/`###`/`####` headings. Text above
// the first such heading belongs to a synthetic preamble section carrying the
// document title, so a claim in an intro paragraph is never silently dropped.
function sections(text) {
  const lines = text.split("\n");
  const result = [];
  let current = { heading: null, headingLine: 0, lines: [] };
  let inFence = false;
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (/^\s*```/.test(line)) inFence = !inFence;
    const m = inFence ? null : /^#{2,4}\s+(.*\S)\s*$/.exec(line);
    if (m) {
      result.push(current);
      current = { heading: m[1], headingLine: i + 1, lines: [] };
    } else {
      current.lines.push({ text: line, lineNumber: i + 1 });
    }
  }
  result.push(current);
  return result.filter((s) => s.lines.length > 0 || s.heading);
}

function matchedPhrase(line) {
  for (const re of EVERY_TURN_PHRASES) {
    const m = re.exec(line);
    if (m) return m[0];
  }
  return null;
}

// The derived rule set: every reference-tier section whose own wording claims
// every-turn application, with the line that makes the claim.
function deriveEveryTurnRules() {
  const rules = [];
  for (const file of referenceFiles(SKILLS_DIR)) {
    const text = fs.readFileSync(file, "utf8");
    if (isProvenanceLedger(text)) continue;
    const rel = path.relative(REPO_ROOT, file).split(path.sep).join("/");
    for (const section of sections(text)) {
      const claims = [];
      let inFence = false;
      for (const { text: line, lineNumber } of section.lines) {
        if (/^\s*```/.test(line)) inFence = !inFence;
        if (inFence) continue;
        const phrase = matchedPhrase(line);
        if (phrase) claims.push({ phrase, line: squash(line), lineNumber });
      }
      if (claims.length > 0) {
        rules.push({
          file: rel,
          heading: section.heading,
          headingLine: section.headingLine,
          claims,
        });
      }
    }
  }
  return rules;
}

// ── the always-loaded layer, and the pointers it carries ────────────────────

function alwaysLoadedSources() {
  const found = [];
  for (const rel of ALWAYS_LOADED_CANDIDATES) {
    const full = path.join(REPO_ROOT, rel);
    if (fs.existsSync(full)) {
      found.push({ rel: rel.split(path.sep).join("/"), text: fs.readFileSync(full, "utf8") });
    }
  }
  return found;
}

// Paragraph-scoped, depth-tracked parenthetical contents — one unbalanced `(`
// in prose cannot swallow the rest of the document and turn a bare mention into
// a pointer. Same shape scripts/skill_lint.mjs settled on in cell tci-2.
function* parentheticals(text) {
  for (const para of text.split(/\n[ \t]*\n/)) {
    const open = [];
    for (let i = 0; i < para.length; i += 1) {
      const c = para[i];
      if (c === "(") open.push(i);
      else if (c === ")" && open.length > 0) yield para.slice(open.pop() + 1, i);
    }
  }
}

// The two pointer conventions the always-loaded body actually uses:
//   ("Some Heading")  — one parenthetical may name several
//   § Some Heading    — runs to end of line
function headingPointers(text) {
  const pointers = new Set();
  for (const inner of parentheticals(text)) {
    for (const m of inner.matchAll(/"([^"]+)"/g)) pointers.add(squash(m[1]));
  }
  for (const m of text.matchAll(/§\s*([^\n.;]+)/g)) pointers.add(squash(m[1]));
  return [...pointers].filter((p) => p.length >= 3);
}

// A heading is reachable when the always-loaded body names it. Substring, not
// equality: the body cites "Progress ticks" for a heading that reads "Progress
// ticks — worked examples (spec #81 P4, …)", and demanding the decorated form
// would report a live pointer as missing — the exact false positive tci-2 had
// to clear out of skill_lint.
function reachableFrom(pointers, heading) {
  if (!heading) return false;
  const h = norm(heading);
  return pointers.some((p) => h.includes(norm(p)));
}

// ── checks ──────────────────────────────────────────────────────────────────

const alwaysLoaded = alwaysLoadedSources();
const alwaysLoadedText = alwaysLoaded.map((s) => s.text).join("\n");
const rules = deriveEveryTurnRules();
const pointers = alwaysLoaded.flatMap((s) => headingPointers(s.text));

check("the always-loaded operating block is present and readable", () => {
  assert(
    alwaysLoaded.length > 0,
    `none of the always-loaded sources exist: ${ALWAYS_LOADED_CANDIDATES.join(", ")}`,
  );
});

// A derivation that silently matches nothing is a gate that cannot fail, which
// is worse than no gate: it teaches its readers that green means checked. If a
// doctrine rewrite ever moves past this vocabulary, this check goes red and the
// seed gets revisited — the detector is never allowed to go blind quietly.
check("the every-turn detector is not blind — it finds claims in the block AND in the reference tier", () => {
  const inBlock = alwaysLoadedText
    .split("\n")
    .filter((line) => matchedPhrase(line))
    .map((line) => squash(line).slice(0, 70));
  assert(
    inBlock.length > 0,
    "the always-loaded block states no every-turn rule in any wording this suite recognizes — " +
      `either the block lost its per-turn rules or EVERY_TURN_PHRASES has drifted from the doctrine's language (phrases: ${EVERY_TURN_PHRASES.map(String).join(", ")})`,
  );
  assert(
    rules.length > 0,
    "no every-turn claim found anywhere in the reference tier — the corpus walk or the seed vocabulary has gone blind, and this suite would pass vacuously",
  );
  console.log(
    `     derived: ${rules.length} every-turn section(s) in the reference tier, ${inBlock.length} in the always-loaded block, ${pointers.length} heading pointer(s)`,
  );
});

// THE GATE. This is the check that exists to make the tick-contract failure
// class unrepeatable: a rule the doctrine itself says applies every turn must be
// named by the layer the agent always has, never left behind an on-demand load.
check(
  "every rule the reference tier says applies EVERY TURN is named by the always-loaded block",
  () => {
    const unreachable = [];
    for (const rule of rules) {
      if (reachableFrom(pointers, rule.heading)) continue;
      const where = rule.heading
        ? `section "${rule.heading}" (${rule.file}:${rule.headingLine})`
        : `${rule.file} (before its first section heading)`;
      const claim = rule.claims[0];
      unreachable.push(
        `${rule.file}:${claim.lineNumber} — ${where} states an every-turn rule ` +
          `(matched "${claim.phrase}") but the always-loaded block never names it\n` +
          `      claim: ${claim.line.slice(0, 160)}`,
      );
    }
    assert(
      unreachable.length === 0,
      "an every-turn rule is reachable ONLY from an on-demand reference. Move the rule's operative " +
        "clauses into the always-loaded block, or have the block name the section:\n  " +
        unreachable.join("\n  "),
    );
  },
);

console.log(`\n${passed} passed, ${failed} failed`);
console.log(
  "scope: reachability of every-turn rules only — this suite does NOT observe or enforce tick emission",
);
if (failed > 0) process.exit(1);
