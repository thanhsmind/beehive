// skill_lint — ADVISORY skill-tree lint. Not a test, deliberately (user law,
// 2026-07-27: tests are for code; instruction text gets a lint, not a suite).
// Always exits 0: budget deltas are information, anchor/numbering problems are
// warnings. A human decides; nothing blocks, nothing runs in the verify estate.
//
// The slice's net behavior is instruction text: three rules (progress ticks,
// the re-lane checkpoint, the merged review wave) moved into references with
// routing lines left in the bodies. Prose has no runtime to assert, but the two
// mechanical properties that make the thin-body doctrine real do — and both were
// violated at least once while authoring this very slice, which is why they are
// worth a suite rather than a promise:
//
//   1. BODY BUDGET (a ratchet). A SKILL.md body is injected whole on every
//      invoke, so a body that grows taxes every future session forever. The
//      baseline is recorded per skill and may only shrink. Adding a rule means
//      paying for it by removing text, not by appending. Spec #78 P2.
//
//   2. ANCHOR INTEGRITY. Moving text to a reference is only safe if the pointer
//      resolves. A body line reading `references/x.md` ("Some Heading") must
//      find that file AND that heading. During this slice a body pointed at a
//      "merged reviewer prompt" section that did not exist yet — a dangling
//      pointer costs the reader the whole rule, silently.
//
//   3. ORDERED-LIST INTEGRITY. Inserting a numbered step without renumbering
//      its successors produces two steps with the same number; this slice did
//      exactly that in bee-exploring before it was caught by eye.
//
// The baseline lives beside this file. To lower a budget after a genuine trim,
// run this suite with --update-baseline; it refuses to RAISE one, which is the
// whole point.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, '..');
const SKILLS = path.join(REPO, 'skills');
const BASELINE = path.join(HERE, 'skill-body-budget.json');

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

function bodyBytes(skill) {
  return fs.statSync(path.join(SKILLS, skill, 'SKILL.md')).size;
}

// ─── --update-baseline: lower recorded budgets only, never raise ────────────
if (process.argv.includes('--update-baseline')) {
  const current = fs.existsSync(BASELINE) ? JSON.parse(fs.readFileSync(BASELINE, 'utf8')) : { budgets: {} };
  const budgets = current.budgets || {};
  const lowered = [];
  for (const skill of skillDirs()) {
    const size = bodyBytes(skill);
    const recorded = budgets[skill];
    if (recorded === undefined) {
      budgets[skill] = size;
      lowered.push(`${skill}: seeded at ${size}`);
    } else if (size < recorded) {
      budgets[skill] = size;
      lowered.push(`${skill}: ${recorded} -> ${size}`);
    }
  }
  fs.writeFileSync(
    BASELINE,
    `${JSON.stringify({ note: 'Per-skill SKILL.md body budget in bytes. Ratchet: may only go down. See scripts/skill_lint.mjs.', budgets }, null, 2)}\n`,
    'utf8',
  );
  console.log(lowered.length ? lowered.join('\n') : 'nothing to lower');
  process.exit(0);
}

assert(fs.existsSync(BASELINE), `missing baseline ${BASELINE} — seed it with --update-baseline`);
const { budgets } = JSON.parse(fs.readFileSync(BASELINE, 'utf8'));

// ─── 1. body budget ────────────────────────────────────────────────────────
check('every SKILL.md body is within its recorded budget — bodies load whole on every invoke, so they may shrink but never grow', () => {
  const over = [];
  for (const skill of skillDirs()) {
    const budget = budgets[skill];
    if (budget === undefined) {
      over.push(`${skill}: no recorded budget (new skill — seed it with --update-baseline)`);
      continue;
    }
    const size = bodyBytes(skill);
    if (size > budget) over.push(`${skill}: ${size} bytes exceeds budget ${budget} by ${size - budget}`);
  }
  assert(
    over.length === 0,
    `body budget exceeded — pay for new text by removing text, do not append:\n  ${over.join('\n  ')}`,
  );
});

check('the flow-batch-2 slice left every body it touched SMALLER than before, which is what made its three new rules free', () => {
  // The five bodies this slice edited, with the sizes they must not exceed.
  // These are the pre-slice sizes, so the row is a regression net for the
  // specific claim the slice made rather than a restatement of the budget file.
  const preSlice = {
    'bee-hive': 30076,
    'bee-swarming': 24965,
    'bee-exploring': 16209,
    'bee-validating': 17260,
    'bee-planning': 23698,
  };
  const grew = [];
  for (const [skill, before] of Object.entries(preSlice)) {
    const now = bodyBytes(skill);
    if (now > before) grew.push(`${skill}: ${now} > pre-slice ${before}`);
  }
  assert(grew.length === 0, `a flow-batch-2 body grew back:\n  ${grew.join('\n  ')}`);
});

// ─── 2. anchor integrity ───────────────────────────────────────────────────
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

check('the three rules this slice moved to references are reachable from a body pointer', () => {
  const required = [
    ['bee-hive', 'Progress ticks'],
    ['bee-hive', 'Re-lane checkpoint'],
    ['bee-exploring', 'Re-lane checkpoint'],
  ];
  const missing = [];
  for (const [skill, heading] of required) {
    const body = fs.readFileSync(path.join(SKILLS, skill, 'SKILL.md'), 'utf8');
    if (!body.includes(`("${heading}")`)) missing.push(`${skill}/SKILL.md has no pointer to "${heading}"`);
  }
  assert(missing.length === 0, missing.join('\n  '));
});

check('bee-validating routes to the MERGED reviewer prompt and no longer names the retired pair as separate dispatches', () => {
  const body = fs.readFileSync(path.join(SKILLS, 'bee-validating', 'SKILL.md'), 'utf8');
  const ref = fs.readFileSync(path.join(SKILLS, 'bee-validating', 'references', 'validation-reference.md'), 'utf8');
  assert(/merged reviewer prompt/i.test(body), 'the body no longer points at the merged reviewer prompt');
  assert(/^##\s+Merged Reviewer Subagent Prompt/m.test(ref), 'the reference has no merged reviewer prompt section');
  assert(
    !/^##\s+Plan-Checker Subagent Prompt/m.test(ref) && !/^##\s+Cell-Reviewer Subagent Prompt/m.test(ref),
    'the reference still carries the retired split prompts — two sources of truth for one dispatch',
  );
  for (const vocab of ['BLOCKER', 'WARNING', 'CRITICAL', 'MINOR']) {
    assert(ref.includes(vocab), `the merged prompt dropped the ${vocab} finding class`);
  }
});

// ─── 3. ordered-list integrity ─────────────────────────────────────────────
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
