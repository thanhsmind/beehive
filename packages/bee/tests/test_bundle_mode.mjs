#!/usr/bin/env node
// test_bundle_mode.mjs — the permanent suite for the ONE bundle-mode
// predicate and the scribing decision tree it routes (okf-switchover-f3, cell
// f3-2; G1/G3/G8/G9).
//
// WHY THIS SUITE EXISTS AT ALL (advisor-digest-f3 finding 1, measured): bee's
// own checkout HAS a bundle, and G2 forbids new prose under docs/specs/ here,
// so the home repo structurally CANNOT exercise the compatibility fallback end
// to end. The fallback is the promise that a host repo which never migrated
// keeps working byte-for-byte — a promise that, if left as prose, "ships
// working, rots in one release, no error". It is therefore proven in a
// bundle-LESS fixture repo, permanently, on every chain run.
//
// The three fixtures below are the whole point:
//   - bundle-LESS  -> the docs/specs/ path is selected, NO bundle path is
//                     touched, and NOTHING new is emitted (no deprecation
//                     notice, no nag, not one extra field) — G1.
//   - bundle-ful   -> the concept path is selected — G3.
//   - .gitkeep     -> a docs/knowledge/ directory holding only a .gitkeep is
//                     NOT a bundle. A directory is not a bundle; at least one
//                     concept must actually parse. This exact case is how the
//                     guarantee would rot silently in a host repo: scribing
//                     would write concepts nobody reads while docs/specs/
//                     quietly stopped updating.
//
// RED-FIRST (cell f3-2): written and run red — `bundleMode` and
// `scribingTarget` do not exist in lib/knowledge.mjs yet — BEFORE any
// implementation, per AGENTS.md critical rule 2.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { emitFrontmatter, bundleMode, scribingTarget } from '../lib/knowledge.mjs';
import { buildSessionPreamble } from '../lib/inject.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(TESTS_DIR, '..', '..', '..');

let passed = 0;
let failed = 0;

async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS  ${name}`);
  } catch (error) {
    failed += 1;
    console.log(`FAIL  ${name}`);
    console.log(`      ${error instanceof Error ? error.stack || error.message : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// ─── fixture builders (mkdtempSync roots only — never the real repo) ────────

function makeRepo(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `bee-bundle-mode-${label}-`));
}

function writeFile(root, rel, text) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

/** Canonical concept text through the real emitter (D12). */
function conceptText({
  type = 'bee.area',
  title = 'Demo area — purpose',
  description = 'A canonical fixture concept.',
  id = 'demo-area-overview',
  areas = ['demo-area'],
  authoritativeFor = null,
  body = 'Body.',
} = {}) {
  const bee = { id, lifecycle: 'active', areas };
  if (authoritativeFor) bee.authoritative_for = authoritativeFor;
  const data = { type, title, description, tags: ['demo'], timestamp: '2026-07-22', bee };
  return `${emitFrontmatter(data)}\n# ${title}\n\n${body}\n`;
}

/** A host repo that never migrated: docs/specs/ only, no docs/knowledge/. */
function makeBundlelessRepo() {
  const root = makeRepo('bundleless');
  writeFile(root, 'docs/specs/reading-map.md', '# Reading map\n\n- billing: docs/specs/billing.md\n');
  writeFile(root, 'docs/specs/billing.md', '# Billing\n\n## Purpose\n\nCharges customers.\n');
  return root;
}

/** A migrated repo: a bundle with concepts that actually parse. */
function makeBundlefulRepo() {
  const root = makeRepo('bundleful');
  writeFile(root, 'docs/specs/reading-map.md', '# Reading map\n');
  writeFile(
    root,
    'docs/knowledge/areas/billing/overview.md',
    conceptText({
      id: 'billing-overview',
      title: 'Billing — purpose, vocabulary, and actors',
      areas: ['billing'],
      authoritativeFor: 'billing: purpose, vocabulary, and actors',
    }),
  );
  writeFile(
    root,
    'docs/knowledge/areas/billing/refunds.md',
    conceptText({
      id: 'billing-refunds',
      title: 'Billing — refunds',
      areas: ['billing'],
      authoritativeFor: 'billing: refunds and reversals',
    }),
  );
  return root;
}

/** The rot case: docs/knowledge/ exists but holds no concept at all. */
function makeGitkeepRepo() {
  const root = makeRepo('gitkeep');
  writeFile(root, 'docs/specs/billing.md', '# Billing\n');
  writeFile(root, 'docs/knowledge/.gitkeep', '');
  return root;
}

function listBundleFiles(root) {
  const dir = path.join(root, 'docs', 'knowledge');
  const out = [];
  const walk = (abs, rel) => {
    let entries;
    try {
      entries = fs.readdirSync(abs, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) walk(path.join(abs, entry.name), childRel);
      else out.push(childRel);
    }
  };
  walk(dir, '');
  return out.sort();
}

// Anything that would tell an un-migrated host repo that this release
// happened. G1: the fallback is not a deprecation warning and never nags.
const NAG_RE = /deprecat|obsolete|legacy tree|no longer supported|please migrate|will be removed|superseded by docs\/knowledge|consider migrating/i;

const TARGET_KEYS = ['bundle_mode', 'action', 'area', 'subject', 'path', 'owner', 'regenerate_index'];

// ─── G8: the ONE predicate ─────────────────────────────────────────────────

await check('bundleMode is FALSE when docs/knowledge/ does not exist at all', async () => {
  const root = makeBundlelessRepo();
  assert(bundleMode(root) === false, 'a repo with no docs/knowledge/ is not in bundle mode');
});

await check('bundleMode is FALSE for a docs/knowledge/ containing only a .gitkeep (a directory is not a bundle)', async () => {
  const root = makeGitkeepRepo();
  assert(fs.existsSync(path.join(root, 'docs', 'knowledge')), 'fixture really does have the directory');
  assert(bundleMode(root) === false, 'an empty docs/knowledge/ must NOT flip a host repo into bundle mode');
});

await check('bundleMode is FALSE for a bundle holding only reserved files (index.md / log.md are not concepts)', async () => {
  const root = makeRepo('reserved');
  writeFile(root, 'docs/knowledge/index.md', '---\nokf_version: "0.1"\n---\n\n# Index\n');
  writeFile(root, 'docs/knowledge/log.md', '# Log\n\n## 2026-07-22\n');
  assert(bundleMode(root) === false, 'reserved basenames are never concepts (OKF §3.1)');
});

await check('bundleMode is FALSE when no concept actually parses (frontmatter absent or unparseable)', async () => {
  const root = makeRepo('unparseable');
  writeFile(root, 'docs/knowledge/areas/billing/overview.md', '# No frontmatter here\n');
  writeFile(root, 'docs/knowledge/areas/billing/broken.md', '---\ntype: bee.area\n# never closed\n');
  assert(bundleMode(root) === false, 'files that do not parse as concepts do not make a bundle');
});

await check('bundleMode is TRUE as soon as ONE concept parses', async () => {
  const root = makeBundlefulRepo();
  assert(bundleMode(root) === true, 'a bundle with parsing concepts is bundle mode');
});

await check("bundleMode is TRUE for bee's own checkout (the live bundle)", async () => {
  assert(bundleMode(REPO_ROOT) === true, 'the home repo has a real bundle');
});

await check('bundleMode never throws on a nonexistent root or on a file where the bundle dir should be', async () => {
  assert(bundleMode(path.join(os.tmpdir(), 'bee-does-not-exist-xyz')) === false, 'missing root is false, not a throw');
  const root = makeRepo('filedir');
  writeFile(root, 'docs/knowledge', 'not a directory\n');
  assert(bundleMode(root) === false, 'a FILE at docs/knowledge is not a bundle');
});

// ─── G1: the bundle-LESS fixture — today's behaviour, byte-for-byte ─────────

await check('bundle-less fixture: an existing area routes to docs/specs/<area>.md, in place', async () => {
  const root = makeBundlelessRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(target.bundle_mode === false, 'fallback mode');
  assert(target.action === 'update-spec', `in-place spec update, got ${target.action}`);
  assert(target.path === 'docs/specs/billing.md', `today's path, got ${target.path}`);
  assert(target.regenerate_index === false, 'no index to regenerate without a bundle');
  assert(target.owner === null, 'no concept owners exist');
});

await check('bundle-less fixture: a brand-new area routes to docs/specs/<area>.md — one area, one file, forever', async () => {
  const root = makeBundlelessRepo();
  const target = scribingTarget(root, { area: 'payouts', subject: 'payouts: schedule' });
  assert(target.bundle_mode === false, 'fallback mode');
  assert(target.action === 'create-spec', `first write creates the one spec, got ${target.action}`);
  assert(target.path === 'docs/specs/payouts.md', `today's path, got ${target.path}`);
});

await check('bundle-less fixture: NO bundle path is named, and none is created on disk', async () => {
  const root = makeBundlelessRepo();
  for (const area of ['billing', 'payouts']) {
    const target = scribingTarget(root, { area, subject: `${area}: something` });
    const serialized = JSON.stringify(target);
    assert(!serialized.includes('docs/knowledge'), `no bundle path in the result: ${serialized}`);
    assert(!serialized.includes('areas/'), `no bundle-relative path in the result: ${serialized}`);
  }
  assert(!fs.existsSync(path.join(root, 'docs', 'knowledge')), 'resolving a target creates no bundle directory');
  assert(listBundleFiles(root).length === 0, 'not one bundle file touched');
});

await check('bundle-less fixture: nothing new is emitted — no deprecation notice, no nag, no extra field', async () => {
  const root = makeBundlelessRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  const serialized = JSON.stringify(target);
  assert(!NAG_RE.test(serialized), `the fallback must say nothing new: ${serialized}`);
  const keys = Object.keys(target).sort();
  assert(
    JSON.stringify(keys) === JSON.stringify([...TARGET_KEYS].sort()),
    `the fallback result carries exactly the pinned keys, got ${JSON.stringify(keys)}`,
  );
});

await check('.gitkeep fixture: the empty docs/knowledge/ still routes to docs/specs/ (the silent-rot case)', async () => {
  const root = makeGitkeepRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds' });
  assert(target.bundle_mode === false, 'a .gitkeep is not a bundle');
  assert(target.path === 'docs/specs/billing.md', `still the spec path, got ${target.path}`);
  assert(listBundleFiles(root).join(',') === '.gitkeep', 'the bundle directory is left exactly as found');
});

// ─── G3: the bundle-ful fixture — the concept path ─────────────────────────

await check('bundle-ful fixture (a): a subject already owned updates THAT concept', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(target.bundle_mode === true, 'bundle mode');
  assert(target.action === 'update-concept', `owned subject updates in place, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/refunds.md', `the owning concept, got ${target.path}`);
  assert(target.owner && target.owner.id === 'billing-refunds', 'the owner is named');
  assert(target.regenerate_index === false, 'an in-place update adds no index row');
});

await check('bundle-ful fixture (a): ownership matching ignores case and surrounding whitespace', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'billing', subject: '  Billing:  Refunds and Reversals  ' });
  assert(target.action === 'update-concept', `normalized match, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/refunds.md', `the owning concept, got ${target.path}`);
});

await check('bundle-ful fixture (b): a new subject in an existing area authors a new concept there and regenerates the index', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: dunning and retries' });
  assert(target.bundle_mode === true, 'bundle mode');
  assert(target.action === 'new-concept', `new subject, existing area, got ${target.action}`);
  assert(
    target.path === 'docs/knowledge/areas/billing/dunning-and-retries.md',
    `slug derived from the subject, got ${target.path}`,
  );
  assert(target.owner === null, 'nobody owns it yet');
  assert(target.regenerate_index === true, 'a new file means the index must be regenerated');
});

await check('bundle-ful fixture (c): a brand-new area creates areas/<slug>/ with an overview concept', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'payouts', subject: 'payouts: purpose and actors' });
  assert(target.action === 'new-area', `brand-new area, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/payouts/overview.md', `overview concept, got ${target.path}`);
  assert(target.regenerate_index === true, 'a new area means the index must be regenerated');
});

// ─── G9: the anti-fork gate ────────────────────────────────────────────────

await check('anti-fork: authoring a NEW concept for an already-owned subject is REFUSED, naming the owner', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, {
    area: 'billing',
    subject: 'billing: refunds and reversals',
    intent: 'new-concept',
  });
  assert(target.action === 'fork_denied', `a second concept on one subject is refused, got ${target.action}`);
  assert(target.path === null, 'a refusal hands back NO path to write to');
  assert(target.owner && target.owner.path === 'docs/knowledge/areas/billing/refunds.md', 'the refusal names the owner');
  assert(target.regenerate_index === false, 'nothing to regenerate');
});

await check('anti-fork: an owned subject is never routed to a second file even when its slug differs from the owner filename', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(target.path !== 'docs/knowledge/areas/billing/refunds-and-reversals.md', 'no -v2 by slug drift');
  assert(target.path === 'docs/knowledge/areas/billing/refunds.md', 'the one owning file, always');
});

await check('anti-fork: a derived filename that already exists is an in-place update, never a second file', async () => {
  const root = makeBundlefulRepo();
  // A concept file whose name matches the subject slug but claims no subject.
  writeFile(
    root,
    'docs/knowledge/areas/billing/invoices.md',
    conceptText({ id: 'billing-invoices', title: 'Billing — invoices', areas: ['billing'] }),
  );
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: invoices' });
  assert(target.action === 'update-concept', `an existing path is updated, never duplicated, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/invoices.md', `got ${target.path}`);
});

await check('anti-fork: ownership is bundle-wide — a subject owned in ANOTHER area still blocks a new concept here', async () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, { area: 'payouts', subject: 'billing: refunds and reversals' });
  assert(target.action === 'update-concept', `the owner wins over the requested area, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/refunds.md', 'routed to the one authority');
});

// ─── the prose that must not rot back (advisor finding 1: prose alone rots) ─

await check('bee-capturing names the concept home; the area doctrine survives in its reference', async () => {
  const skill = fs.readFileSync(path.join(REPO_ROOT, 'skills', 'bee-capturing', 'SKILL.md'), 'utf8');
  assert(/docs\/knowledge\//.test(skill), 'the skill names the state layer');
  const areaRef = fs.readFileSync(
    path.join(REPO_ROOT, 'skills', 'bee-capturing', 'references', 'area-spec.md'),
    'utf8',
  );
  assert(/One area =\s+one file/.test(areaRef), 'the one-area-one-file rule survives in the reference');
  assert(/docs\/knowledge\/areas\//.test(areaRef), 'the reference names the concept home');
});

await check('bee-capturing carries NO deprecation notice and no nag anywhere', async () => {
  const skill = fs.readFileSync(path.join(REPO_ROOT, 'skills', 'bee-capturing', 'SKILL.md'), 'utf8');
  assert(!NAG_RE.test(skill), 'an un-migrated host repo must not be told this release happened');
});

await check('the okf-profile area-concept template carries the nine-section skeleton and the rebuild bar', async () => {
  // f3-5/G6: the profile moved INTO the bundle it describes, so the template
  // it ships lives in the concept that owns concept authoring; docs/specs/
  // okf-profile.md is now a pointer stub and carries no template at all.
  const profile = fs.readFileSync(
    path.join(REPO_ROOT, 'docs', 'knowledge', 'areas', 'okf-profile', 'concept-model-and-authoring.md'),
    'utf8',
  );
  const marker = '### `bee.area`';
  const start = profile.indexOf(marker);
  assert(start !== -1, 'the Templates section carries a bee.area template');
  const nextTemplate = profile.indexOf('\n### ', start + marker.length);
  const section = profile.slice(start, nextTemplate === -1 ? profile.length : nextTemplate);
  const NINE = [
    'Purpose',
    'Entry Points & Triggers',
    'Data Dictionary',
    'Behaviors & Operations',
    'Actors & Access',
    'Business Rules',
    'Edge Cases Settled',
    'Open Gaps',
    'Pointers (implementation)',
  ];
  for (const heading of NINE) {
    assert(section.includes(`## ${heading}`), `the template carries "## ${heading}"`);
  }
  assert(/rebuild bar/i.test(section), 'the rebuild bar is stated as the acceptance test, not left to discretion');
  assert(/authoritative_for/.test(section), 'the template shows the subject-ownership field');
});

// ═══════════════════════════════════════════════════════════════════════════
// cell f3-3 — the independent judge's reproductions, verbatim.
//
// f3-2 shipped the gate above and an independent judge broke it four ways and
// found a fifth class it never considered, plus a sixth that all three
// fixtures above are structurally blind to. These are ACCEPTANCE tests, not
// paraphrases: each one is the judge's exact defeat.
//
//   R1  a trailing period          -> a new concept beside the owner
//   R2  a Cyrillic-e homoglyph     -> a new concept beside the owner
//   R3  authoritative_for as ARRAY -> invisible (typeof !== 'string' -> continue)
//   R4  empty/whitespace/null subj -> the gate is skipped entirely and the
//                                      request routes to overview.md EVEN WITH
//                                      intent 'new-concept'
//   R5  two pre-existing owners    -> the FIRST wins by walk order, undetected
//   R6  the divorced topology      -> product_root is ignored, so a migrated
//                                      host is graded bundle-less and its
//                                      fallback points at the empty workshop
//
// RED-FIRST (cell f3-3): every one of the six was written and run RED against
// the f3-2 implementation before a single line of the fix was written.
// ═══════════════════════════════════════════════════════════════════════════

// ─── G14 LAYER 1: the match is hardened (normalization, not exact strings) ──

await check('R1 judge defeat: a TRAILING PERIOD must not buy a second concept beside the owner', () => {
  const root = makeBundlefulRepo();
  const target = scribingTarget(root, {
    area: 'billing',
    subject: 'billing: refunds and reversals.',
    intent: 'new-concept',
  });
  assert(
    target.action === 'fork_denied',
    `"billing: refunds and reversals." is the owned subject with a period; got ${target.action} -> ${target.path}`,
  );
  assert(target.path === null, 'a refusal hands back NO path to write to');
  assert(target.owner && target.owner.path === 'docs/knowledge/areas/billing/refunds.md', 'the refusal names the owner');
});

await check('R1b: leading/trailing punctuation and internal whitespace runs all normalize to the same subject', () => {
  const root = makeBundlefulRepo();
  for (const subject of [
    '  "billing: refunds and reversals"  ',
    'billing:   refunds\tand   reversals!!!',
    '…billing: refunds and reversals…',
    'Billing — Refunds and Reversals',
  ]) {
    const target = scribingTarget(root, { area: 'billing', subject, intent: 'new-concept' });
    assert(target.action === 'fork_denied', `subject ${JSON.stringify(subject)} must resolve to the owner, got ${target.action}`);
  }
});

await check('R2 judge defeat: a CYRILLIC-e HOMOGLYPH must not buy a second concept beside the owner', () => {
  const root = makeBundlefulRepo();
  // U+0435 CYRILLIC SMALL LETTER IE in place of the Latin "e" of "refunds"
  const homoglyph = 'billing: rеfunds and reversals';
  assert(homoglyph !== 'billing: refunds and reversals', 'the fixture really is a different byte string');
  const target = scribingTarget(root, { area: 'billing', subject: homoglyph, intent: 'new-concept' });
  assert(
    target.action === 'fork_denied',
    `a homoglyph is the same subject to every human reader; got ${target.action} -> ${target.path}`,
  );
  assert(target.owner && target.owner.path === 'docs/knowledge/areas/billing/refunds.md', 'the refusal names the owner');
});

await check('R2b: NFKC + confusable folding also catches fullwidth, ligature and Greek look-alikes', () => {
  const root = makeBundlefulRepo();
  for (const subject of [
    'ｂｉｌｌｉｎｇ: refunds and reversals', // fullwidth (NFKC)
    'billing: refunds and reνersals', // Greek nu for v
    'billing: rеfunds and rеversals', // two Cyrillic e's
  ]) {
    const target = scribingTarget(root, { area: 'billing', subject, intent: 'new-concept' });
    assert(target.action === 'fork_denied', `subject ${JSON.stringify(subject)} must resolve to the owner, got ${target.action}`);
  }
});

// ─── G14 LAYER 2: malformed input fails CLOSED, never silently ─────────────

await check('R3 judge defeat: an ARRAY-valued authoritative_for is a VALIDATION ERROR naming the file, never a silent skip', () => {
  const root = makeBundlefulRepo();
  // The judge's exact shape: the claim is a list, so `typeof claim !== 'string'`
  // skipped it and the owner became invisible.
  writeFile(
    root,
    'docs/knowledge/areas/billing/disputes.md',
    conceptText({ id: 'billing-disputes', title: 'Billing — disputes', areas: ['billing'] }).replace(
      'lifecycle: active',
      'lifecycle: active\n  authoritative_for: [billing: disputes, billing: chargebacks]',
    ),
  );
  let thrown = null;
  try {
    scribingTarget(root, { area: 'billing', subject: 'billing: disputes', intent: 'new-concept' });
  } catch (error) {
    thrown = error;
  }
  assert(thrown instanceof Error, 'a malformed authority claim must be an ERROR, not a silently skipped concept');
  assert(
    /disputes\.md/.test(thrown.message),
    `the validation error must NAME the offending file, got ${JSON.stringify(thrown.message)}`,
  );
  assert(
    /authoritative_for/.test(thrown.message),
    `the validation error must name the field, got ${JSON.stringify(thrown.message)}`,
  );
});

await check('R3b: every malformed authority shape a real file can produce fails closed — list, boolean, empty, blank', () => {
  // The reachable set, measured against the D12 parser rather than assumed:
  // `42` and `null` parse as the STRINGS "42"/"null" (odd subjects, but
  // strings — the gate reads them fine), and a mapping is already an OKF
  // `unparseable_frontmatter` error before this code is reached. What is left
  // is exactly this list, and every one of it must fail closed.
  for (const [label, literal] of [
    ['list', '[gates, locks]'],
    ['boolean', 'true'],
    ['empty', '""'],
    ['blank', '"   "'],
  ]) {
    const root = makeBundlefulRepo();
    writeFile(
      root,
      'docs/knowledge/areas/billing/bad.md',
      conceptText({ id: `billing-bad-${label}`, title: 'Billing — bad', areas: ['billing'] }).replace(
        'lifecycle: active',
        `lifecycle: active\n  authoritative_for: ${literal}`,
      ),
    );
    let thrown = null;
    try {
      scribingTarget(root, { area: 'billing', subject: 'billing: anything', intent: 'auto' });
    } catch (error) {
      thrown = error;
    }
    assert(thrown instanceof Error && /bad\.md/.test(thrown.message), `a ${label} authority claim must fail closed naming the file, got ${thrown && thrown.message}`);
  }
});

await check('R4 judge defeat: an EMPTY / whitespace / null / undefined subject with intent new-concept is REFUSED, never routed to overview.md', () => {
  const root = makeBundlefulRepo();
  for (const subject of ['', '   ', '\t\n ', null, undefined, '...', '  --  ']) {
    const target = scribingTarget(root, { area: 'billing', subject, intent: 'new-concept' });
    assert(
      target.action === 'subject_required',
      `subject ${JSON.stringify(subject)} with intent new-concept must be refused, got ${target.action} -> ${target.path}`,
    );
    assert(target.path === null, `a refusal hands back NO path, got ${target.path}`);
    assert(
      !/overview\.md/.test(JSON.stringify(target)),
      `an empty-subject new-concept must never mention overview.md: ${JSON.stringify(target)}`,
    );
  }
});

await check('R4b: the documented intents still fail SAFE — an empty subject on auto keeps today\'s update-concept routing', () => {
  const root = makeBundlefulRepo();
  writeFile(
    root,
    'docs/knowledge/areas/billing/overview.md',
    conceptText({ id: 'billing-overview-2', title: 'Billing — overview', areas: ['billing'] }),
  );
  const target = scribingTarget(root, { area: 'billing', subject: '', intent: 'auto' });
  assert(target.action === 'update-concept', `auto must fail safe to update-concept, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/overview.md', `got ${target.path}`);
});

await check('R5 judge defeat: TWO pre-existing owners of one subject are DETECTED, never resolved first-wins by walk order', () => {
  const root = makeBundlefulRepo();
  // A second concept claiming the subject already owned by refunds.md. Walk
  // order puts "a-fork.md" FIRST, so a first-wins gate answers with the WRONG
  // owner and never says a word.
  writeFile(
    root,
    'docs/knowledge/areas/billing/a-fork.md',
    conceptText({
      id: 'billing-refunds-fork',
      title: 'Billing — refunds (fork)',
      areas: ['billing'],
      authoritativeFor: 'billing: refunds and reversals',
    }),
  );
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(
    target.action === 'duplicate_authority',
    `two claimants is an ambiguity, not a silent first-wins; got ${target.action} -> ${target.path}`,
  );
  assert(target.path === null, 'no reader can tell which file is true, so nothing may be written');
  const conflicts = (target.owner && target.owner.conflicts) || [];
  assert(
    conflicts.includes('docs/knowledge/areas/billing/a-fork.md') &&
      conflicts.includes('docs/knowledge/areas/billing/refunds.md'),
    `BOTH claimants must be named, got ${JSON.stringify(target.owner)}`,
  );
});

await check('R5b: a WORD-ORDER PARAPHRASE is the residual gap layer 1 structurally cannot close — named, not pretended away', () => {
  const root = makeBundlefulRepo();
  writeFile(
    root,
    'docs/knowledge/areas/billing/reversals.md',
    conceptText({
      id: 'billing-reversals',
      title: 'Billing — reversals',
      areas: ['billing'],
      authoritativeFor: 'billing: reversals and refunds', // word-order paraphrase
    }),
  );
  // Layer 1 legitimately does NOT catch this: it is a genuinely DIFFERENT
  // subject string, not a different encoding of the same one. No amount of
  // normalization closes that, which is exactly why layer 3 exists — the
  // moment either concept adopts the other's subject (in any encoding), the
  // chain-failing `duplicate_authoritative_for` finding bites. Proven in
  // test_knowledge.mjs.
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: reversals and refunds' });
  assert(target.action === 'update-concept', `the paraphrase resolves to its own concept, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/reversals.md', `got ${target.path}`);
});

// ─── G13: the DIVORCED topology (config product_root, GitHub #14) ──────────
//
// All three fixtures above are single-root and structurally cannot see this
// class: a workshop root holding `.bee/` with the product an independent repo
// one directory down. Every other product-doc consumer (inject.mjs:71,
// backlog.mjs:19/266, hooks/bee-session-close.mjs:110-118) already routes
// through resolveProductRoot. docs/knowledge/ and docs/specs/ are product doc
// trees exactly like those, so they must too (G13).

/** Workshop root with `.bee/config.json` product_root -> a nested product repo. */
function makeDivorcedRepo(label, { withBundle }) {
  const root = makeRepo(`divorced-${label}`);
  writeFile(root, '.bee/config.json', JSON.stringify({ product_root: 'product' }, null, 2) + '\n');
  // The workshop's OWN docs/specs/ is empty — this is the trap: a fallback
  // resolved against the workshop root points here, at nothing.
  fs.mkdirSync(path.join(root, 'docs', 'specs'), { recursive: true });
  // The product repo one directory down carries the real product docs.
  writeFile(root, 'product/docs/specs/reading-map.md', '# Reading map\n');
  writeFile(root, 'product/docs/specs/billing.md', '# Billing\n\n## Purpose\n\nCharges customers.\n');
  if (withBundle) {
    writeFile(
      root,
      'product/docs/knowledge/areas/billing/refunds.md',
      conceptText({
        id: 'billing-refunds',
        title: 'Billing — refunds',
        areas: ['billing'],
        authoritativeFor: 'billing: refunds and reversals',
      }),
    );
  }
  return root;
}

await check('R6 judge defeat (divorced topology): a migrated host with product_root is in BUNDLE MODE, not graded bundle-less', () => {
  const root = makeDivorcedRepo('bundle', { withBundle: true });
  assert(!fs.existsSync(path.join(root, 'docs', 'knowledge')), 'the workshop root has no bundle of its own — that is the point');
  assert(
    bundleMode(root) === true,
    'a migrated product one directory down IS a bundle; grading it bundle-less is the silent-rot class',
  );
});

await check('R6b (divorced topology): scribing routes to the PRODUCT bundle concept, not a workshop-side fork', () => {
  const root = makeDivorcedRepo('route', { withBundle: true });
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(target.bundle_mode === true, `bundle mode, got ${target.bundle_mode}`);
  assert(target.action === 'update-concept', `the product owner wins, got ${target.action}`);
  assert(target.path === 'docs/knowledge/areas/billing/refunds.md', `product-root-relative path, got ${target.path}`);
  assert(target.owner && target.owner.id === 'billing-refunds', 'the product concept is named as the owner');
  assert(!fs.existsSync(path.join(root, 'docs', 'knowledge')), 'resolving a target creates nothing on the workshop side');
});

await check('R6c (divorced topology): the anti-fork gate BITES across the divorce — a new concept for the product-owned subject is refused', () => {
  const root = makeDivorcedRepo('fork', { withBundle: true });
  const target = scribingTarget(root, {
    area: 'billing',
    subject: 'billing: refunds and reversals.',
    intent: 'new-concept',
  });
  assert(target.action === 'fork_denied', `the product owner must be visible to the gate, got ${target.action}`);
});

await check('R6d (divorced topology): the bundle-LESS fallback resolves the PRODUCT docs/specs/, not the empty workshop one', () => {
  const root = makeDivorcedRepo('fallback', { withBundle: false });
  assert(bundleMode(root) === false, 'no bundle anywhere in this fixture');
  const target = scribingTarget(root, { area: 'billing', subject: 'billing: refunds and reversals' });
  assert(
    target.action === 'update-spec',
    `product/docs/specs/billing.md exists, so this is an in-place UPDATE — grading it create-spec is the outage (#14); got ${target.action}`,
  );
  assert(target.path === 'docs/specs/billing.md', `product-root-relative path, got ${target.path}`);
  // And a genuinely new area is still a create, off the product root.
  const fresh = scribingTarget(root, { area: 'payouts', subject: 'payouts: schedule' });
  assert(fresh.action === 'create-spec', `an absent product spec is still a create, got ${fresh.action}`);
});

// ─── f3-2's proven guarantee, EXTENDED (never weakened) ────────────────────

await check('f3-2 guarantee held: the fallback still emits EXACTLY its key set, with no bundle mention, across every f3-3 path', () => {
  const cases = [
    { root: makeBundlelessRepo(), args: { area: 'billing', subject: 'billing: refunds and reversals' } },
    { root: makeBundlelessRepo(), args: { area: 'payouts', subject: 'payouts: schedule' } },
    { root: makeBundlelessRepo(), args: { area: 'billing', subject: 'billing: refunds.', intent: 'new-concept' } },
    { root: makeBundlelessRepo(), args: { area: 'billing', subject: '', intent: 'new-concept' } },
    { root: makeBundlelessRepo(), args: { area: 'billing', subject: null, intent: 'new-concept' } },
    { root: makeGitkeepRepo(), args: { area: 'billing', subject: '   ', intent: 'new-concept' } },
    { root: makeDivorcedRepo('keys', { withBundle: false }), args: { area: 'billing', subject: 'billing: refunds' } },
  ];
  for (const { root, args } of cases) {
    const target = scribingTarget(root, args);
    assert(target.bundle_mode === false, `fallback mode for ${JSON.stringify(args)}, got ${target.bundle_mode}`);
    const keys = Object.keys(target).sort();
    assert(
      JSON.stringify(keys) === JSON.stringify([...TARGET_KEYS].sort()),
      `the fallback result carries exactly the pinned keys for ${JSON.stringify(args)}, got ${JSON.stringify(keys)}`,
    );
    const serialized = JSON.stringify(target);
    assert(!serialized.includes('docs/knowledge'), `no bundle path in the fallback result: ${serialized}`);
    assert(!serialized.includes('areas/'), `no bundle-relative path in the fallback result: ${serialized}`);
    assert(!NAG_RE.test(serialized), `the fallback must say nothing new: ${serialized}`);
    assert(
      ['update-spec', 'create-spec'].includes(target.action),
      `G14's new refusals are BUNDLE-ONLY — an un-migrated host must not be able to tell this release happened; got ${target.action}`,
    );
  }
});

await check('f3-3: every bundle-mode result also carries exactly the pinned key set (the shape is one shape)', () => {
  const root = makeBundlefulRepo();
  writeFile(
    root,
    'docs/knowledge/areas/billing/a-fork.md',
    conceptText({ id: 'billing-fork', title: 'Fork', areas: ['billing'], authoritativeFor: 'billing: refunds and reversals' }),
  );
  for (const args of [
    { area: 'billing', subject: 'billing: refunds and reversals' }, // duplicate_authority
    { area: 'billing', subject: '', intent: 'new-concept' }, // subject_required
    { area: 'billing', subject: 'billing: dunning' }, // new-concept
    { area: 'shipping', subject: 'shipping: carriers' }, // new-area
  ]) {
    const target = scribingTarget(root, args);
    const keys = Object.keys(target).sort();
    assert(
      JSON.stringify(keys) === JSON.stringify([...TARGET_KEYS].sort()),
      `pinned keys for ${JSON.stringify(args)}, got ${JSON.stringify(keys)}`,
    );
  }
});

// The three-layer capture gate (subject_required / duplicate_authority /
// duplicate_authoritative_for) is machine-owned and pinned by the mechanical
// checks above; instruction prose no longer catalogs refusal codes.

// ═══════════════════════════════════════════════════════════════════════════
// cell f4-3 — the SESSION PREAMBLE routes on the same one predicate.
//
// The preamble is what every agent reads before doing anything, and two of its
// sections were still teaching the retired model (both confirmed by direct
// observation of a live preamble, not inferred):
//
//   D1  `### Critical patterns (digest)` printed the POINTER STUB's redirect
//       boilerplate as if it were the patterns.
//   D2  `### Project map` printed `Specced areas: N (docs/specs/ — read the
//       spec before the code)` — counting the compatibility surface and
//       instructing the reading order G4 replaced.
//   D3  the scribing-debt nudge hardcoded `docs/specs/` as where settled
//       behavior belongs.
//
// These rows live HERE rather than in test_misc.mjs for the reason stated at
// the top of this file: bee's own checkout HAS a bundle, so the fallback is
// structurally untestable against the real repo. The bundle-LESS fixture below
// pins all three surfaces BYTE-FOR-BYTE as they read before this cell — that
// is the whole promise to a host repo that never migrated.
// ═══════════════════════════════════════════════════════════════════════════

/** The 10 non-blank lines a legacy critical-patterns.md digest must reproduce. */
const LEGACY_PATTERN_LINES = [
  '# Critical Patterns',
  '## [20260101] A lesson that settled',
  '- Symptom: the thing broke.',
  '- Rule: do not do the thing.',
  '## [20260202] Another lesson',
  '- Symptom: it broke again.',
  '- Rule: really do not do the thing.',
  '## [20260303] A third lesson',
  '- Symptom: still broken.',
  '- Rule: stop.',
];

/**
 * A repo shaped like a real one at preamble time: both project maps, two area
 * specs, a backlog, a legacy critical-patterns file, and one capped
 * behavior_change cell so the scribing-debt nudge fires. `withBundle` adds a
 * parsing concept plus a generated root index carrying 12 date-ordered
 * critical-pattern rows.
 */
function makePreambleRepo(label, { withBundle }) {
  const root = makeRepo(`preamble-${label}`);
  writeFile(root, '.bee/onboarding.json', JSON.stringify({ schema_version: '1.0' }) + '\n');
  // phase compounding-complete: a closed feature with uncaptured scribing debt.
  // It also keeps the okf-8 knowledge-context bridge silent, so the sections
  // under test are the only ones speaking.
  writeFile(
    root,
    '.bee/state.json',
    JSON.stringify({ phase: 'compounding-complete', feature: 'demo-feature', mode: 'standard' }, null, 2) + '\n',
  );
  writeFile(
    root,
    '.bee/cells/demo-1.json',
    JSON.stringify(
      {
        id: 'demo-1',
        feature: 'demo-feature',
        title: 'Demo',
        lane: 'small',
        status: 'capped',
        deps: [],
        trace: { behavior_change: true, capped_at: new Date().toISOString() },
      },
      null,
      2,
    ) + '\n',
  );
  writeFile(root, 'docs/specs/system-overview.md', '# Overview\n');
  writeFile(root, 'docs/specs/reading-map.md', '# Reading map\n');
  writeFile(root, 'docs/specs/billing.md', '# Billing\n');
  writeFile(root, 'docs/specs/payouts.md', '# Payouts\n');
  writeFile(
    root,
    'docs/backlog.md',
    '| ID | Story | CoS | Status | Feature |\n| -- | ----- | --- | ------ | ------- |\n| 1 | A | x | done | f |\n| 2 | B | y | proposed | |\n',
  );
  writeFile(
    root,
    'docs/history/learnings/critical-patterns.md',
    `<!-- a comment line that the digest drops -->\n\n${LEGACY_PATTERN_LINES.join('\n\n')}\n\n- Rule: a line past the cap.\n`,
  );
  if (withBundle) {
    writeFile(
      root,
      'docs/knowledge/areas/billing/overview.md',
      conceptText({ id: 'billing-overview', title: 'Billing — purpose', areas: ['billing'] }),
    );
    writeFile(
      root,
      'docs/knowledge/areas/payouts/overview.md',
      conceptText({ id: 'payouts-overview', title: 'Payouts — purpose', areas: ['payouts'] }),
    );
    const rows = [];
    for (let n = 1; n <= 12; n += 1) {
      const id = String(n).padStart(2, '0');
      rows.push(`- [Pattern ${id}](patterns/2026${id}01-pattern-${id}.md) — hook ${id}`);
    }
    writeFile(
      root,
      'docs/knowledge/index.md',
      [
        '---',
        'okf_version: 0.1',
        '---',
        '',
        '# Knowledge Bundle',
        '',
        '## Sections',
        '',
        '- [areas/](areas/index.md) — 2 concept(s)',
        '',
        '## Critical patterns',
        '',
        ...rows,
        '',
        '## Not patterns',
        '',
        '- [Something else](areas/index.md) — never in the digest',
        '',
      ].join('\n'),
    );
  }
  return root;
}

function section(preamble, heading) {
  const all = preamble.split('\n');
  const start = all.findIndex((line) => line === heading || line.startsWith(heading));
  if (start === -1) return null;
  const out = [all[start]];
  for (let i = start + 1; i < all.length; i += 1) {
    if (all[i] === '' || all[i].startsWith('### ')) break;
    out.push(all[i]);
  }
  return out;
}

// ─── the bundle-LESS fixture: all three surfaces, byte-for-byte as before ───

await check('preamble fallback: the Project map section is BYTE-IDENTICAL to before f4-3', () => {
  const root = makePreambleRepo('nobundle-map', { withBundle: false });
  assert(bundleMode(root) === false, 'the fixture really has no bundle');
  const map = section(buildSessionPreamble(root), '### Project map');
  assert(
    JSON.stringify(map) ===
      JSON.stringify([
        '### Project map',
        '- System overview: docs/specs/system-overview.md',
        '- Reading map: docs/specs/reading-map.md',
        '- Specced areas: 2 (docs/specs/ — read the spec before the code)',
        '- PBI: 1 done / 0 in-flight / 1 proposed',
      ]),
    `today's section, line for line, got ${JSON.stringify(map)}`,
  );
  assert(map.length <= 5, `the 2-5 line cap holds, got ${map.length}`);
});

await check('preamble fallback: the missing-map WARNING branch is byte-identical too (and still the only line)', () => {
  const root = makePreambleRepo('nobundle-warn', { withBundle: false });
  fs.rmSync(path.join(root, 'docs', 'specs', 'system-overview.md'));
  fs.rmSync(path.join(root, 'docs', 'specs', 'reading-map.md'));
  const map = section(buildSessionPreamble(root), '### Project map');
  assert(
    JSON.stringify(map) ===
      JSON.stringify([
        '### Project map',
        '- Project map missing (Q1/Q2 unanswerable from repo) — bee-capturing bootstrap available.',
        '- PBI: 1 done / 0 in-flight / 1 proposed',
      ]),
    `the warning branch is untouched by D2, got ${JSON.stringify(map)}`,
  );
});

await check('preamble fallback: the critical-patterns digest still reads the legacy file, capped at 10 lines', () => {
  const root = makePreambleRepo('nobundle-digest', { withBundle: false });
  const digest = section(buildSessionPreamble(root), '### Critical patterns (digest)');
  assert(
    JSON.stringify(digest) === JSON.stringify(['### Critical patterns (digest)', ...LEGACY_PATTERN_LINES]),
    `the first 10 non-blank, non-comment lines, unchanged, got ${JSON.stringify(digest)}`,
  );
  assert(!digest.some((line) => /a line past the cap/.test(line)), 'the 10-line cap still bites');
});

await check('preamble fallback: the scribing-debt nudge still names docs/specs/, word for word', () => {
  const root = makePreambleRepo('nobundle-debt', { withBundle: false });
  const debt = section(buildSessionPreamble(root), '### Scribing debt:');
  assert(debt !== null, 'the fixture really does carry scribing debt');
  assert(
    debt[1] ===
      '- demo-1 capped since the last scribing run — run bee-capturing capture now; settled behavior belongs in docs/specs/ before it evaporates (decision 0011).',
    `today's nudge, verbatim, got ${JSON.stringify(debt[1])}`,
  );
});

await check('preamble fallback: the WHOLE preamble never mentions the bundle, and never nags', () => {
  const root = makePreambleRepo('nobundle-whole', { withBundle: false });
  const preamble = buildSessionPreamble(root);
  assert(!preamble.includes('docs/knowledge'), 'an un-migrated host is never told this release happened');
  assert(!/knowledge bundle/i.test(preamble), 'no bundle vocabulary anywhere');
  assert(!NAG_RE.test(preamble), `the fallback says nothing new: ${preamble}`);
});

// ─── the bundle-ful fixture: the bundle is what you read before the code ────

await check('preamble bundle mode (D2): the Project map names the BUNDLE and counts what it holds', () => {
  const root = makePreambleRepo('bundle-map', { withBundle: true });
  assert(bundleMode(root) === true, 'the fixture really has a bundle');
  const map = section(buildSessionPreamble(root), '### Project map');
  assert(map.length >= 2 && map.length <= 5, `the 2-5 line cap holds in the bundle branch too, got ${map.length}`);
  assert(map.some((line) => line.includes('docs/knowledge/')), 'the bundle is named');
  assert(
    map.some((line) => /read the bundle before the code/.test(line)),
    `the bundle is what you read before the code, got ${JSON.stringify(map)}`,
  );
  assert(
    map.some((line) => /2 area\(s\), 2 concept\(s\)/.test(line)),
    `counts what the bundle actually holds, got ${JSON.stringify(map)}`,
  );
  assert(!map.some((line) => /Specced areas/.test(line)), 'the compatibility surface is not the area count');
  assert(
    !map.some((line) => /read the spec before the code/.test(line)),
    'the reading order G4 replaced is gone in bundle mode',
  );
  assert(
    map.some((line) => /read-only compatibility surface/.test(line)),
    'docs/specs/ is described as what it now is, if it is named at all',
  );
  assert(map.some((line) => /PBI: 1 done \/ 0 in-flight \/ 1 proposed/.test(line)), 'the PBI line rides BOTH branches');
});

await check('preamble bundle mode (D1): the digest comes from the bundle index — total, newest-first, never the stub', () => {
  const root = makePreambleRepo('bundle-digest', { withBundle: true });
  const digest = section(buildSessionPreamble(root), '### Critical patterns (digest)');
  assert(digest.length <= 11, `heading + the same 10-line cap, got ${digest.length}`);
  assert(
    /12 critical pattern\(s\) in the bundle/.test(digest[1]),
    `the TOTAL is stated, not implied by the cut, got ${JSON.stringify(digest[1])}`,
  );
  assert(/docs\/knowledge\/index\.md/.test(digest[1]), 'the full index is named');
  const rows = digest.slice(2);
  assert(rows.length === 9, `the count line plus the 9 most recent rows fills the cap, got ${rows.length}`);
  assert(/Pattern 12/.test(rows[0]), `newest FIRST, got ${JSON.stringify(rows[0])}`);
  assert(/Pattern 04/.test(rows[rows.length - 1]), `oldest of the recent slice last, got ${JSON.stringify(rows[rows.length - 1])}`);
  for (const oldest of ['Pattern 01', 'Pattern 02', 'Pattern 03']) {
    assert(!rows.some((line) => line.includes(oldest)), `${oldest} is the OLDEST — a first-N cut would print it forever`);
  }
  assert(
    !rows.some((line) => /Something else/.test(line)),
    'the section ends at the next "## " heading — no rows from a later section',
  );
  assert(
    rows.every((line) => line.includes('](docs/knowledge/patterns/')),
    `bundle-relative links are rewritten to paths a session can open, got ${JSON.stringify(rows[0])}`,
  );
  assert(
    !digest.some((line) => /pointer stub|migrated_to|area: critical-patterns/.test(line)),
    'the retired stub\'s boilerplate never reaches a session again',
  );
});

await check('preamble bundle mode (D1): a bundle with no generated index degrades to SILENCE, never to the stub', () => {
  const root = makePreambleRepo('bundle-noindex', { withBundle: true });
  fs.rmSync(path.join(root, 'docs', 'knowledge', 'index.md'));
  const preamble = buildSessionPreamble(root);
  assert(section(preamble, '### Critical patterns (digest)') === null, 'no index, no digest section');
  assert(
    !preamble.includes(LEGACY_PATTERN_LINES[1]),
    'bundle mode never falls back to the retired file — that is the bug this cell fixes',
  );
});

await check('preamble bundle mode (D3): the scribing-debt nudge names the RESOLVED target', () => {
  const root = makePreambleRepo('bundle-debt', { withBundle: true });
  const debt = section(buildSessionPreamble(root), '### Scribing debt:');
  assert(debt !== null, 'the fixture really does carry scribing debt');
  assert(
    debt[1] ===
      '- demo-1 capped since the last scribing run — run bee-capturing capture now; settled behavior belongs in docs/knowledge/ before it evaporates (decision 0011).',
    `the bundle repo is told where its knowledge actually goes, got ${JSON.stringify(debt[1])}`,
  );
});

await check('preamble (G13): the divorced topology reads the PRODUCT bundle, never a bare-root join', () => {
  const root = makeDivorcedRepo('preamble', { withBundle: true });
  assert(!fs.existsSync(path.join(root, 'docs', 'knowledge')), 'the workshop root has no bundle of its own');
  const map = section(buildSessionPreamble(root), '### Project map');
  assert(map.some((line) => line.includes('docs/knowledge/')), `the product bundle is named, got ${JSON.stringify(map)}`);
  assert(
    map.some((line) => /1 area\(s\), 1 concept\(s\)/.test(line)),
    `the PRODUCT concepts are counted, not the empty workshop, got ${JSON.stringify(map)}`,
  );
  assert(!map.some((line) => /Specced areas/.test(line)), 'a migrated product one directory down is bundle mode');
});

console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} test_bundle_mode: ${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
