#!/usr/bin/env node
// skill_budget_fence.mjs — skill-token-diet cell diet-1, D6: the BLOCKING half
// of the skill body budget machinery. Chain-fail fence, okf-fence pattern
// (scripts/okf_specs_fence.mjs / scripts/okf_instructions_fence.mjs).
//
// WHY A SEPARATE SCRIPT, NOT skill_lint.mjs. The 2026-07-27 user law (decision
// 6d9b9afc) is: instruction text gets a lint, never a suite — nothing in
// skill_lint.mjs blocks. skill-token-diet D6 SUPERSEDES that law, narrowly,
// for exactly one check: the user approved spec P2's chain-fail enforcement
// on 2026-07-28. Folding a blocking check into an always-exit-0 script would
// either break the advisory law for every OTHER check in skill_lint.mjs, or
// require a parallel exit-code convention inside one file. A sibling fence
// keeps the narrowing surgical: skill_lint.mjs stays advisory end to end,
// this script is the one thing that can fail scripts/run_verify.mjs.
//
// WHAT IT ENFORCES, all against scripts/skill-body-budget.json as the single
// source of budgets (D6 — no parallel frontmatter mechanism):
//   1. BODY BUDGET (a ratchet). Any skills/*/SKILL.md whose byte size
//      (fs.statSync().size — identical to `wc -c`) exceeds its recorded
//      budget fails: "pay for new text by removing text", not by appending.
//   2. NO SILENT GAPS. A skill directory with a SKILL.md but no baseline
//      entry fails loud (new skills are seeded explicitly, never inferred).
//      A missing baseline FILE fails loud too — never a silent skip.
//   3. EXCEPTION JUSTIFICATION (D1). A baseline entry over 8,192 bytes is a
//      declared exception, not an oversight — it must carry a sibling
//      `notes.<skill>` string ("pending migration" for grandfathers).
//      Without one, the fence fails naming the skill and its budget.
//   4. PROVENANCE EXILE (D8). A migrated skill's body carries no decision
//      IDs / plan names / hardening labels inline — that lives one hop away
//      in the skill's own references/provenance.md. "Migrated" is read
//      STRUCTURALLY from the baseline's top-level `migrated: []` array,
//      never inferred from budget size (validation D2: bee-context-locking
//      is 6,454 bytes — under 8,192 — and unmigrated, carrying 11 pattern
//      matches; a budget<=8192 inference would red the chain immediately on
//      a skill nobody has touched).
//
// --update-baseline MOVES HERE from skill_lint.mjs (this cell, per D6): seeds
// a new skill at its current size, lowers an entry after a genuine trim,
// and never raises one — the whole point of a ratchet. It leaves `migrated`
// and `notes` untouched; those are edited by hand at migration time, the
// same way this cell hand-edited the one-time drift re-seed (validation D1:
// a direct JSON edit, not a raise --update-baseline would have refused).
//
// --selftest proves the fence BITES on temp-dir fixtures, the okf-fence
// precedent: an over-budget body fails; a provenance citation in a skill
// LISTED in `migrated` fails; the same citation in an UNLISTED skill passes
// (D2's own worked example); a grandfathered skill (budget > 8,192, note
// present) passes; a missing baseline file and a skill with no entry both
// fail loud. The plan's full boundary matrix (exact-8192, idempotent
// --update-baseline, etc.) is diet-2's trailing test cell — this selftest
// covers the bites this cell's own must_haves name, not the whole matrix.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');

// The byte ceiling D1 states for a MIGRATED body; also the threshold above
// which a baseline entry is a declared exception requiring a note.
export const BUDGET_CEILING = 8192;

// Provenance grep pattern (D8) — tunable per CONTEXT.md's Agent's Discretion.
// Matches an inline citation of a decision ID, an AO rule, a decision hash,
// a hardening label, or a plan number, each opened by a literal '('.
export const PROVENANCE_RE = /\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)/;

export function skillsDir(root) {
  return path.join(root, 'skills');
}

export function baselinePath(root) {
  return path.join(root, 'scripts', 'skill-body-budget.json');
}

export function skillDirs(root) {
  const dir = skillsDir(root);
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isDirectory() && fs.existsSync(path.join(dir, e.name, 'SKILL.md')))
    .map((e) => e.name)
    .sort();
}

export function bodyBytes(root, skill) {
  return fs.statSync(path.join(skillsDir(root), skill, 'SKILL.md')).size;
}

export function bodyText(root, skill) {
  return fs.readFileSync(path.join(skillsDir(root), skill, 'SKILL.md'), 'utf8');
}

/** Returns the parsed baseline, or null if the file is missing/unparseable — both are the SAME failure to the fence: never silent. */
export function loadBaseline(root) {
  const p = baselinePath(root);
  if (!fs.existsSync(p)) return null;
  try {
    return JSON.parse(fs.readFileSync(p, 'utf8'));
  } catch {
    return null;
  }
}

/**
 * Scan a repo. Returns { root, baselineMissing, entries, findings }.
 *   entries  — one per skill on disk: { skill, size, budget, verdict, reason }.
 *   findings — the subset (plus provenance hits, which are not per-skill)
 *              that fail the chain, each with a `reason` string.
 */
export function fenceFindings(root) {
  const baseline = loadBaseline(root);
  if (!baseline) {
    const reason = `missing or unparseable baseline ${path.relative(root, baselinePath(root))} — seed it with --update-baseline (never a silent skip)`;
    return { root, baselineMissing: true, entries: [], findings: [{ type: 'baseline-missing', skill: null, reason }] };
  }

  const budgets = baseline.budgets || {};
  const notes = baseline.notes || {};
  const migrated = new Set(Array.isArray(baseline.migrated) ? baseline.migrated : []);
  const skills = skillDirs(root);

  const entries = [];
  const findings = [];

  for (const skill of skills) {
    const size = bodyBytes(root, skill);
    const budget = budgets[skill];

    if (budget === undefined) {
      const reason = `${skill}: no recorded budget entry — new skills are seeded explicitly, never inferred (run --update-baseline)`;
      entries.push({ skill, size, budget: null, verdict: 'no-budget-entry', reason });
      findings.push({ type: 'no-budget-entry', skill, reason });
      continue;
    }

    const problems = [];

    if (size > budget) {
      const reason = `${skill}: ${size} bytes exceeds budget ${budget} by ${size - budget} — pay for new text by removing text, not by appending`;
      findings.push({ type: 'over-budget', skill, size, budget, delta: size - budget, reason });
      problems.push('over-budget');
    }

    if (budget > BUDGET_CEILING && !(typeof notes[skill] === 'string' && notes[skill].trim())) {
      const reason = `${skill}: budget ${budget} exceeds ${BUDGET_CEILING} bytes with no notes.${skill} justification — every exception over the ceiling needs a stated reason (D1)`;
      findings.push({ type: 'missing-note', skill, budget, reason });
      problems.push('missing-note');
    }

    entries.push({
      skill,
      size,
      budget,
      verdict: problems.length ? problems.join('+') : 'ok',
      reason: problems.length ? problems.join('; ') : 'within budget',
    });
  }

  // Provenance exile (D8) — ONLY skills explicitly listed in `migrated`,
  // never inferred from budget size (validation D2).
  for (const skill of migrated) {
    if (!skills.includes(skill)) continue; // listed but not on disk — not this fence's job to invent a skill
    const text = bodyText(root, skill);
    text.split('\n').forEach((line, idx) => {
      if (PROVENANCE_RE.test(line)) {
        const reason = `${skill}/SKILL.md:${idx + 1} cites provenance inline — migrated skills exile decision IDs to references/provenance.md (D8): "${line.trim()}"`;
        findings.push({ type: 'provenance-violation', skill, line: idx + 1, text: line.trim(), reason });
      }
    });
  }

  return { root, baselineMissing: false, entries, findings };
}

// ─── --update-baseline: seed/lower recorded budgets only, never raise ──────
function runUpdateBaseline(root) {
  const p = baselinePath(root);
  const current = fs.existsSync(p)
    ? JSON.parse(fs.readFileSync(p, 'utf8'))
    : {
        note: 'Per-skill SKILL.md body budget in bytes. Ratchet: may only go down. See scripts/skill_budget_fence.mjs.',
        budgets: {},
        migrated: [],
        notes: {},
      };
  const budgets = current.budgets || {};
  const changed = [];
  for (const skill of skillDirs(root)) {
    const size = bodyBytes(root, skill);
    const recorded = budgets[skill];
    if (recorded === undefined) {
      budgets[skill] = size;
      changed.push(`${skill}: seeded at ${size}`);
    } else if (size < recorded) {
      budgets[skill] = size;
      changed.push(`${skill}: ${recorded} -> ${size}`);
    } else if (size > recorded) {
      changed.push(`${skill}: ${size} bytes > recorded ${recorded} — NOT raised (ratchet refuses to raise); trim the body or the fence stays red`);
    }
  }
  current.budgets = budgets;
  if (!Array.isArray(current.migrated)) current.migrated = [];
  if (!current.notes || typeof current.notes !== 'object') current.notes = {};
  if (!current.note) {
    current.note = 'Per-skill SKILL.md body budget in bytes. Ratchet: may only go down. See scripts/skill_budget_fence.mjs.';
  }
  fs.writeFileSync(p, `${JSON.stringify(current, null, 2)}\n`, 'utf8');
  console.log(changed.length ? changed.join('\n') : 'nothing to lower');
  return 0;
}

// ─── selftest fixtures ──────────────────────────────────────────────────────

function makeRepo(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `bee-budget-fence-${label}-`));
}

function writeFile(root, rel, text) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

function writeBaseline(root, data) {
  writeFile(root, 'scripts/skill-body-budget.json', `${JSON.stringify(data, null, 2)}\n`);
}

function findingTypes(result) {
  return (result.findings || []).map((f) => `${f.type}:${f.skill ?? ''}`).sort();
}

/** Runs fn with console.log captured (not printed); returns the captured lines. Used to assert on --update-baseline's own report text without polluting selftest output. */
function captureLog(fn) {
  const lines = [];
  const original = console.log;
  console.log = (...args) => lines.push(args.join(' '));
  try {
    fn();
  } finally {
    console.log = original;
  }
  return lines;
}

function runSelftest() {
  let passed = 0;
  let failed = 0;
  const check = (name, fn) => {
    try {
      fn();
      passed += 1;
      console.log(`PASS  ${name}`);
    } catch (error) {
      failed += 1;
      console.log(`FAIL  ${name}`);
      console.log(`      ${error instanceof Error ? error.message : error}`);
    }
  };
  const assert = (condition, message) => {
    if (!condition) throw new Error(message);
  };

  // (A) missing baseline file — never a silent skip.
  check('a repo with no baseline file FAILS loud, never silently', () => {
    const root = makeRepo('nobaseline');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(100));
    const result = fenceFindings(root);
    assert(result.baselineMissing === true, 'reported explicitly');
    assert(result.findings.length === 1 && result.findings[0].type === 'baseline-missing', `got ${JSON.stringify(findingTypes(result))}`);
  });

  // (B) a skill dir with SKILL.md but no baseline entry fails loud.
  check('a skill with no recorded budget entry FAILS', () => {
    const root = makeRepo('noentry');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(100));
    writeBaseline(root, { budgets: {}, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'no-budget-entry:demo', `got ${JSON.stringify(findingTypes(result))}`);
  });

  // (C) the fence BITES: an over-budget body fails, naming size/budget/delta.
  check('an over-budget body FAILS, naming skill, size, budget and delta', () => {
    const root = makeRepo('overbudget');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(150));
    writeBaseline(root, { budgets: { demo: 100 }, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'over-budget:demo', `got ${JSON.stringify(findingTypes(result))}`);
    const finding = result.findings[0];
    assert(finding.size === 150 && finding.budget === 100 && finding.delta === 50, `size/budget/delta recorded, got ${JSON.stringify(finding)}`);
    assert(/pay for new text by removing text/.test(finding.reason), 'names the remedy');
  });

  // (D) exception rule (D1): a budget over the ceiling with no note fails.
  check('a budget over 8192 with NO notes justification FAILS', () => {
    const root = makeRepo('nonote');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(9000));
    writeBaseline(root, { budgets: { demo: 9000 }, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'missing-note:demo', `got ${JSON.stringify(findingTypes(result))}`);
  });

  // (E) grandfathered skill with a note PASSES — the exact D1 exception shape.
  check('a grandfathered skill (budget>8192, note present) PASSES', () => {
    const root = makeRepo('grandfather');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(9000));
    writeBaseline(root, { budgets: { demo: 9000 }, migrated: [], notes: { demo: 'pending migration' } });
    const result = fenceFindings(root);
    assert(result.findings.length === 0, `got ${JSON.stringify(findingTypes(result))}`);
    assert(result.entries[0].verdict === 'ok', 'classified ok');
  });

  // (F) provenance exile (D8): a citation in a MIGRATED skill fails.
  check('a provenance citation in a skill LISTED in migrated[] FAILS', () => {
    const root = makeRepo('migratedbites');
    const body = '# demo\n\nSome rule stated bare, cited here (D3) for audit.\n';
    writeFile(root, 'skills/demo/SKILL.md', body);
    writeBaseline(root, { budgets: { demo: Buffer.byteLength(body, 'utf8') }, migrated: ['demo'], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'provenance-violation:demo', `got ${JSON.stringify(findingTypes(result))}`);
    assert(result.findings[0].line === 3, `the citing line is named, got ${JSON.stringify(result.findings[0])}`);
  });

  // (G) the D2 worked example: the SAME citation in an UNLISTED skill passes
  // — migrated status is never inferred from budget size.
  check('the SAME provenance citation in a skill NOT LISTED in migrated[] PASSES', () => {
    const root = makeRepo('unlisted');
    const body = '# demo\n\nSome rule stated bare, cited here (D3) for audit.\n';
    writeFile(root, 'skills/demo/SKILL.md', body);
    // budget deliberately <= ceiling, mirroring bee-context-locking (6454B,
    // unmigrated, 11 matches) — proves the fence does not infer migrated
    // status from budget size.
    writeBaseline(root, { budgets: { demo: Buffer.byteLength(body, 'utf8') }, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(result.findings.length === 0, `unlisted skills are never provenance-scanned, got ${JSON.stringify(findingTypes(result))}`);
  });

  // (H) --update-baseline: seeds new, lowers on trim, refuses to raise.
  check('--update-baseline seeds a new entry and lowers a trimmed one, never raises', () => {
    const root = makeRepo('updatebaseline');
    writeFile(root, 'skills/newskill/SKILL.md', 'a'.repeat(200));
    writeFile(root, 'skills/trimmed/SKILL.md', 'a'.repeat(50));
    writeFile(root, 'skills/grown/SKILL.md', 'a'.repeat(300));
    writeBaseline(root, { budgets: { trimmed: 100, grown: 250 }, migrated: [], notes: {} });
    runUpdateBaseline(root);
    const after = JSON.parse(fs.readFileSync(baselinePath(root), 'utf8'));
    assert(after.budgets.newskill === 200, `new skill seeded, got ${after.budgets.newskill}`);
    assert(after.budgets.trimmed === 50, `trimmed skill lowered, got ${after.budgets.trimmed}`);
    assert(after.budgets.grown === 250, `grown skill NOT raised, got ${after.budgets.grown}`);
  });

  // (I) the live repo — the actual verify-chain assertion, exercised inside
  // selftest too so a red live run is caught by BOTH invocations.
  check("bee's own re-seeded baseline produces ZERO findings", () => {
    const result = fenceFindings(REPO_ROOT);
    assert(result.baselineMissing === false, 'the live baseline exists');
    assert(result.entries.length > 0, 'the live skill tree is enumerated');
    assert(
      result.findings.length === 0,
      `the live tree must be silent, got:\n${result.findings.map((f) => `  ${f.reason}`).join('\n')}`,
    );
  });

  // diet-2: extends diet-1's bite-proving fixtures (A-I above, untouched) to
  // the plan's full test matrix (docs/history/skill-token-diet/plan.md
  // "Test Matrix") — net slice behavior only, no fence-logic changes.

  // (J) boundary: a body at EXACTLY the recorded budget passes (size > budget
  // is the only over-budget trigger — equality is not a violation).
  check('a body EXACTLY at its recorded budget (8192) PASSES', () => {
    const root = makeRepo('boundary-exact');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(BUDGET_CEILING));
    writeBaseline(root, { budgets: { demo: BUDGET_CEILING }, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(result.findings.length === 0, `exact-budget body must pass, got ${JSON.stringify(findingTypes(result))}`);
    assert(result.entries[0].verdict === 'ok', 'classified ok at the exact boundary');
  });

  // (K) boundary: one byte over the recorded budget fails, with the byte
  // delta named (the state-drift row of the test matrix).
  check('a body ONE BYTE over its recorded budget (8193) FAILS with delta 1', () => {
    const root = makeRepo('boundary-over');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(BUDGET_CEILING + 1));
    writeBaseline(root, { budgets: { demo: BUDGET_CEILING }, migrated: [], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'over-budget:demo', `got ${JSON.stringify(findingTypes(result))}`);
    const finding = result.findings[0];
    assert(finding.delta === 1, `delta must be 1, got ${JSON.stringify(finding)}`);
    assert(finding.size === BUDGET_CEILING + 1 && finding.budget === BUDGET_CEILING, 'size/budget recorded at the boundary');
  });

  // (L) budget entry EXACTLY at the ceiling: not > BUDGET_CEILING, so no
  // missing-note is required — but explicit migrated[] membership still
  // turns the provenance grep on at that same boundary (D8/D2: migrated
  // status is structural, never inferred from the budget number).
  check('a budget of EXACTLY 8192 needs no note, but provenance grep still applies when migrated', () => {
    const root = makeRepo('boundary-migrated');
    const body = '# demo\n\nSome rule stated bare, cited here (D3) for audit.\n';
    const padded = body + 'a'.repeat(BUDGET_CEILING - Buffer.byteLength(body, 'utf8'));
    writeFile(root, 'skills/demo/SKILL.md', padded);
    writeBaseline(root, { budgets: { demo: BUDGET_CEILING }, migrated: ['demo'], notes: {} });
    const result = fenceFindings(root);
    assert(findingTypes(result).join(',') === 'provenance-violation:demo', `got ${JSON.stringify(findingTypes(result))}`);
    assert(!result.findings.some((f) => f.type === 'missing-note'), 'a budget of exactly the ceiling never needs a note');
  });

  // (M) idempotence: once a baseline is stable, a second --update-baseline
  // run reports "nothing to lower" verbatim (no seed, no lower, nothing to
  // print) — the plan's idempotence row.
  check('a second --update-baseline run on a stable baseline reports "nothing to lower"', () => {
    const root = makeRepo('idempotent');
    writeFile(root, 'skills/demo/SKILL.md', 'a'.repeat(200));
    writeBaseline(root, { budgets: {}, migrated: [], notes: {} });
    captureLog(() => runUpdateBaseline(root)); // first run: seeds demo at 200
    const secondRunLines = captureLog(() => runUpdateBaseline(root));
    assert(secondRunLines.join('\n') === 'nothing to lower', `second run must be silent, got ${JSON.stringify(secondRunLines)}`);
    const after = JSON.parse(fs.readFileSync(baselinePath(root), 'utf8'));
    assert(after.budgets.demo === 200, `budget stays at 200, got ${after.budgets.demo}`);
  });

  // (N) ratchet holds across repeated runs: a grown skill's recorded budget
  // is never raised, not on the first --update-baseline run and not on a
  // second one either (the "never raises an entry" row, run twice).
  check('--update-baseline run twice never raises a grown entry either time', () => {
    const root = makeRepo('never-raise-twice');
    writeFile(root, 'skills/grown/SKILL.md', 'a'.repeat(300));
    writeBaseline(root, { budgets: { grown: 250 }, migrated: [], notes: {} });
    captureLog(() => runUpdateBaseline(root));
    const afterFirst = JSON.parse(fs.readFileSync(baselinePath(root), 'utf8'));
    assert(afterFirst.budgets.grown === 250, `first run must not raise, got ${afterFirst.budgets.grown}`);
    captureLog(() => runUpdateBaseline(root));
    const afterSecond = JSON.parse(fs.readFileSync(baselinePath(root), 'utf8'));
    assert(afterSecond.budgets.grown === 250, `second run must not raise either, got ${afterSecond.budgets.grown}`);
  });

  console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} skill_budget_fence --selftest: ${passed} passed, ${failed} failed`);
  return failed === 0 ? 0 : 1;
}

// ─── CLI ─────────────────────────────────────────────────────────────────

function main(argv) {
  const args = argv.slice(2);
  if (args.includes('--update-baseline')) return runUpdateBaseline(REPO_ROOT);
  if (args.includes('--selftest')) return runSelftest();

  const rootIdx = args.indexOf('--root');
  const root = rootIdx === -1 ? REPO_ROOT : path.resolve(args[rootIdx + 1]);
  const result = fenceFindings(root);

  if (args.includes('--json')) {
    console.log(JSON.stringify(result, null, 2));
    return result.findings.length === 0 ? 0 : 1;
  }

  if (result.findings.length === 0) {
    console.log(`PASS skill_budget_fence: ${result.entries.length} skill(s) checked, 0 findings`);
    return 0;
  }

  console.error('FAIL skill_budget_fence: skill body budget/provenance violation(s) (skill-token-diet D6)');
  for (const finding of result.findings) {
    console.error(`  ${finding.reason}`);
  }
  return 1;
}

process.exit(main(process.argv));
