// skill_lint — ADVISORY skill-tree lint. Not a test, deliberately (user law,
// 2026-07-27: tests are for code; instruction text gets a lint, not a suite).
// Always exits 0: anchor/numbering problems are warnings. A human decides;
// nothing here blocks, nothing here runs in the verify estate.
//
// BODY BUDGET moved OUT to scripts/skill_budget_fence.mjs (skill-token-diet
// D6, cell diet-1): the byte-budget ratchet plus the D8 provenance grep are
// now a BLOCKING chain-fail fence, narrowly superseding the advisory-lint law
// for that one check. `--update-baseline` moved there too. This file keeps
// only the two properties below, both still advisory.
//
// The slice's net behavior is instruction text: three rules (progress ticks,
// the re-lane checkpoint, the merged review wave) moved into references with
// routing lines left in the bodies. Prose has no runtime to assert, but the two
// mechanical properties that make the thin-body doctrine real do — and both were
// violated at least once while authoring this very slice, which is why they are
// worth a suite rather than a promise:
//
//   1. ANCHOR INTEGRITY. Moving text to a reference is only safe if the pointer
//      resolves. A body line reading `references/x.md` ("Some Heading") must
//      find that file AND that heading. During this slice a body pointed at a
//      "merged reviewer prompt" section that did not exist yet — a dangling
//      pointer costs the reader the whole rule, silently.
//
//   2. ORDERED-LIST INTEGRITY. Inserting a numbered step without renumbering
//      its successors produces two steps with the same number; this slice did
//      exactly that in bee-exploring before it was caught by eye.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..');
const SKILLS = path.join(REPO, 'skills');

let warnings = 0;
function check(name, fn) {
  try {
    fn();
    console.log(`ok    ${name}`);
  } catch (error) {
    warnings += 1;
    console.log(`WARN  ${name}`);
    console.log(`      ${error && error.message ? error.message : error}`);
  }
}
function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

function skillDirs() {
  return fs
    .readdirSync(SKILLS, { withFileTypes: true })
    .filter((e) => e.isDirectory() && fs.existsSync(path.join(SKILLS, e.name, 'SKILL.md')))
    .map((e) => e.name)
    .sort();
}

// Body budget + --update-baseline live in scripts/skill_budget_fence.mjs now
// (skill-token-diet D6, cell diet-1) — that script is the blocking half and
// the single source of the ratchet; this file no longer touches it.

// ─── 1. anchor integrity ───────────────────────────────────────────────────
// A pointer may be same-skill (`references/x.md`) or cross-skill
// (`bee-hive/references/x.md`) — bee-exploring, bee-planning and bee-swarming
// all route into bee-hive's reference, so resolving only the same-skill shape
// would report three false dangles.
const ANCHOR_RE = /(?:([a-z0-9-]+)\/)?references\/([a-z0-9-]+\.md)`?\s*\("([^"]+)"\)/g;

check('every references/<file>.md ("Heading") pointer in a body resolves to a real file and a real heading', () => {
  const broken = [];
  for (const skill of skillDirs()) {
    const bodyPath = path.join(SKILLS, skill, 'SKILL.md');
    const body = fs.readFileSync(bodyPath, 'utf8');
    for (const m of body.matchAll(ANCHOR_RE)) {
      const [, ownerSkill, file, heading] = m;
      const owner = ownerSkill || skill;
      const refPath = path.join(SKILLS, owner, 'references', file);
      if (!fs.existsSync(refPath)) {
        broken.push(`${skill}/SKILL.md -> ${owner}/references/${file} (file missing)`);
        continue;
      }
      const ref = fs.readFileSync(refPath, 'utf8');
      const found = ref
        .split('\n')
        .filter((line) => /^#{2,4}\s/.test(line))
        .some((line) => line.toLowerCase().includes(heading.toLowerCase()));
      if (!found) broken.push(`${skill}/SKILL.md -> ${owner}/references/${file} ("${heading}" — no such heading)`);
    }
  }
  assert(
    broken.length === 0,
    `dangling reference pointer — the reader loses the whole rule, silently:\n  ${broken.join('\n  ')}`,
  );
});

// A pointer is a quoted heading inside a parenthetical. One parenthetical
// routinely names several — `("Backlog flip", "Brief check", "Command
// detection")` — and the house style wraps the long ones across lines. Testing
// for the literal `("<heading>")`, as this check did until cell tci-2, sees
// only the lone-heading shape: every pointer that grew a second heading read
// back as *missing*, and two workers in one session dismissed the resulting
// warning as pre-existing noise. That is the cost of a false positive in an
// advisory check — it teaches its readers to skip the output.
//
// Reachability is still the bar, not mention: a heading quoted outside every
// parenthetical is prose about the rule, not a pointer to it, and does not
// count. The parenthetical need not carry the reference path — bee-hive's body
// says up front that its bare quoted headings resolve in
// references/routing-and-contracts.md.
function* parentheticals(text) {
  // Paragraph-scoped: a parenthetical wraps across lines but never across a
  // blank line, so one unbalanced `(` in prose cannot swallow the rest of the
  // file and turn a bare mention into a pointer. Depth-tracked, so a nested
  // `(...)` does not truncate its parent.
  for (const para of text.split(/\n[ \t]*\n/)) {
    const open = [];
    for (let i = 0; i < para.length; i += 1) {
      const c = para[i];
      if (c === '(') open.push(i);
      else if (c === ')' && open.length > 0) yield para.slice(open.pop() + 1, i);
    }
  }
}

const norm = (s) => s.replace(/\s+/g, ' ').trim().toLowerCase();

function pointsTo(body, heading) {
  const want = norm(heading);
  for (const inner of parentheticals(body)) {
    for (const m of inner.matchAll(/"([^"]+)"/g)) if (norm(m[1]) === want) return true;
  }
  return false;
}

check('the three rules this slice moved to references are reachable from a body pointer', () => {
  const required = [
    ['bee-hive', 'Progress ticks'],
    ['bee-hive', 'Re-lane checkpoint'],
    ['bee-exploring', 'Re-lane checkpoint'],
  ];
  const missing = [];
  for (const [skill, heading] of required) {
    const body = fs.readFileSync(path.join(SKILLS, skill, 'SKILL.md'), 'utf8');
    if (!pointsTo(body, heading)) missing.push(`${skill}/SKILL.md has no pointer to "${heading}"`);
  }
  assert(missing.length === 0, missing.join('\n  '));
});

// ─── 2. ordered-list integrity ─────────────────────────────────────────────
check('no SKILL.md body repeats a number within one ordered list — inserting a step without renumbering its successors is silent', () => {
  const dupes = [];
  for (const skill of skillDirs()) {
    const lines = fs.readFileSync(path.join(SKILLS, skill, 'SKILL.md'), 'utf8').split('\n');
    let seen = new Set();
    let inFence = false;
    for (let i = 0; i < lines.length; i += 1) {
      const line = lines[i];
      if (/^\s*```/.test(line)) inFence = !inFence;
      if (inFence) continue;
      const m = /^(\d+)\.\s+\S/.exec(line);
      if (!m) {
        // A blank line does not end a list; a non-list, non-blank line does.
        if (line.trim() === '' || /^\s+/.test(line)) continue;
        seen = new Set();
        continue;
      }
      const n = m[1];
      if (seen.has(n)) dupes.push(`${skill}/SKILL.md:${i + 1} repeats step ${n}. — "${line.slice(0, 60)}"`);
      seen.add(n);
    }
  }
  assert(dupes.length === 0, `duplicate step number:\n  ${dupes.join('\n  ')}`);
});

console.log(warnings === 0 ? `\nOK — skill tree clean` : `\n${warnings} advisory warning(s) — nothing blocks`);
process.exit(0);
