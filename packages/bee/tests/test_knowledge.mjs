#!/usr/bin/env node
// test_knowledge.mjs — self-contained contract tests (no framework) for the
// OKF v0.1 bundle core: lib/knowledge.mjs (emitter-first frontmatter parser,
// concept model, checkBundle) plus the `bee knowledge check` CLI wiring
// (okf-foundation S1, cell okf-1; D4/D12/D13/D18/D19/D23/D32).
//
// RED-FIRST (cell okf-1): this file is written and run red (lib/knowledge.mjs
// does not exist yet) BEFORE any implementation, per docs/history/
// okf-foundation/plan.md §File order S1 and AGENTS.md critical rule 2.
//
// Advisor digest s1 items 1+2 are pinned here on purpose: the silent-misparse
// class (colon in an unquoted title, '#' mid-value, CRLF frontmatter) must
// surface as a not_canonical profile warning — parsed data preserved, never
// silently mangled, never a silent green.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import {
  CONCEPT_TYPES,
  LIFECYCLES,
  parseFrontmatter,
  emitFrontmatter,
  checkBundle,
  bundleDir,
  buildPromotion,
  CRITICAL_RELEVANCE,
} from '../lib/knowledge.mjs';

const TESTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const TEMPLATES_DIR = path.dirname(TESTS_DIR);
const BEE_MJS = path.join(TEMPLATES_DIR, 'bee.mjs');
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
    console.log(`      ${error instanceof Error ? (error.stack || error.message) : error}`);
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

// ─── fixture builders (mkdtempSync fixture roots only — never the real repo) ─

function makeRepo() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-knowledge-test-'));
  fs.mkdirSync(path.join(root, '.bee'), { recursive: true });
  writeJsonAtomic(path.join(root, '.bee', 'onboarding.json'), {
    schema_version: '1.0',
    bee_version: '0.1.0',
  });
  return root;
}

function writeBundleFile(root, rel, text) {
  const abs = path.join(root, 'docs', 'knowledge', rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

/** Canonical concept file text built through the real emitter (D12: the
 *  emitter is the subset's source of truth, so fixtures author through it). */
function conceptText({
  type = 'bee.pattern',
  title = 'A demo pattern',
  description = 'A canonical fixture concept',
  id = 'demo-pattern',
  lifecycle = 'active',
  omitRoot = [],
  extraBee = {},
  tags = ['demo'],
  body = 'Body.',
} = {}) {
  const data = {
    type,
    title,
    description,
    tags,
    timestamp: '2026-07-22',
    bee: {
      id,
      lifecycle,
      areas: ['demo-area'],
      required_context: [],
      decisions: [],
      sources: [],
      ...extraBee,
    },
  };
  for (const key of omitRoot) delete data[key];
  return `${emitFrontmatter(data)}\n# ${title}\n\n${body}\n`;
}

function findingCodes(list) {
  return list.map((f) => f.code);
}

async function runBee(args, cwd) {
  return runModuleWorker(BEE_MJS, { args, cwd });
}

// ─── exports: the closed vocabulary and lifecycle enums (D18/D19) ───────────

await check('CONCEPT_TYPES is the closed nine-type D18 vocabulary, slug-cased; LIFECYCLES the four D19 states', async () => {
  const expected = [
    'bee.area',
    'bee.feature',
    'bee.work-item',
    'bee.plan',
    'bee.delivery',
    'bee.decision',
    'bee.pattern',
    'bee.runbook',
    'bee.evidence',
  ];
  assert(JSON.stringify([...CONCEPT_TYPES].sort()) === JSON.stringify([...expected].sort()), `expected the nine D18 types, got ${JSON.stringify(CONCEPT_TYPES)}`);
  assert(CONCEPT_TYPES.length === 9, `vocabulary must be closed at nine, got ${CONCEPT_TYPES.length}`);
  assert(JSON.stringify([...LIFECYCLES].sort()) === JSON.stringify(['active', 'archived', 'draft', 'superseded'].sort()), `expected the four D19 lifecycles, got ${JSON.stringify(LIFECYCLES)}`);
});

// ─── emitter-first parser: round-trip on the emitted subset (D12) ──────────

await check('emit → parse → re-emit is byte-identical, including Vietnamese titles and every D19 field', async () => {
  const data = {
    type: 'bee.pattern',
    title: 'Định tuyến phiên làm việc — quy tắc vàng',
    description: 'Mô tả có dấu tiếng Việt: được trích dẫn vì chứa dấu hai chấm',
    tags: ['mẫu', 'quy-tắc'],
    timestamp: '2026-07-22T03:00:00Z',
    resource: 'https://example.com/x',
    bee: {
      id: 'vn-pattern',
      lifecycle: 'active',
      areas: ['workflow'],
      required_context: ['areas/workflow/overview.md'],
      decisions: ['D19', 'D32'],
      sources: ['docs/specs/advisor-protocol.md'],
      lane: 'high-risk',
      polarity: 'practice',
      critical: true,
      authoritative_for: 'session-routing',
      review_status: 'Draft',
      supersedes: 'old-pattern',
    },
  };
  const emitted = emitFrontmatter(data);
  const parsed = parseFrontmatter(`${emitted}\nBody.\n`);
  assert(parsed.ok === true && parsed.present === true, `canonical emit must parse: ${JSON.stringify(parsed)}`);
  assert(JSON.stringify(parsed.data) === JSON.stringify(data), `parsed data must equal the emitted data.\nemitted: ${JSON.stringify(data)}\nparsed: ${JSON.stringify(parsed.data)}`);
  assert(emitFrontmatter(parsed.data) === emitted, 're-emit of parsed data must be byte-identical to the first emit');
  assert(parsed.block === emitted, 'parsed.block must be the exact frontmatter bytes from the file');
});

await check('parser fails loudly (typed, never a guess) outside the emitted subset (D12)', async () => {
  const cases = [
    ['---\ntype: bee.pattern\n', 'unclosed ---'],
    ['---\ntype: bee.pattern\n\ttitle: tabbed\n---\n', 'tab in frontmatter'],
    ['---\ntype: bee.pattern\ntype: bee.area\n---\n', 'duplicate key'],
    ["---\ntitle: 'single quoted'\n---\n", 'single-quoted scalar'],
    ['---\ntitle: "unterminated\n---\n', 'bad quoted string'],
    ['---\ntitle:no-space\n---\n', 'key:value without space'],
    ['---\ntype: bee.pattern\n\ntitle: x\n---\n', 'blank line inside frontmatter'],
    ['---\nnested:\n  deep: x\n---\n', 'nested map other than bee:'],
    ['---\ntitle: [unclosed, list\n---\n', 'unclosed flow list'],
  ];
  for (const [text, label] of cases) {
    const parsed = parseFrontmatter(text);
    assert(parsed.ok === false, `${label}: must fail, got ${JSON.stringify(parsed)}`);
    assert(parsed.error && typeof parsed.error.code === 'string' && parsed.error.code, `${label}: failure must carry a typed code`);
    assert(typeof parsed.error.message === 'string' && parsed.error.message, `${label}: failure must carry a message`);
  }
});

await check('a file with no leading --- reports present:false (no frontmatter), never a throw', async () => {
  const parsed = parseFrontmatter('# Just a heading\n\nProse.\n');
  assert(parsed.ok === true && parsed.present === false, `expected present:false, got ${JSON.stringify(parsed)}`);
});

// ─── empty bundle is OK (D23: a bundle with nothing in it is conformant) ────

await check('empty bundle: missing docs/knowledge/ and an empty docs/knowledge/ both check OK with zeroed counts', async () => {
  const missing = makeRepo();
  const reportMissing = checkBundle(missing);
  assert(reportMissing.okf.errors.length === 0 && reportMissing.profile.warnings.length === 0, `missing bundle dir must yield zero findings: ${JSON.stringify(reportMissing)}`);
  assert(reportMissing.counts.files === 0 && reportMissing.counts.concepts === 0, `missing bundle dir must count zero files: ${JSON.stringify(reportMissing.counts)}`);
  assert(reportMissing.ok === true, 'empty bundle must be ok');

  const empty = makeRepo();
  fs.mkdirSync(bundleDir(empty), { recursive: true });
  const reportEmpty = checkBundle(empty);
  assert(reportEmpty.ok === true && reportEmpty.counts.files === 0, `empty bundle dir must be ok: ${JSON.stringify(reportEmpty)}`);
});

// ─── OKF error classes (D4, one per class) ──────────────────────────────────

await check('OKF error: a non-reserved .md with no frontmatter', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/naked.md', '# No frontmatter here\n');
  const report = checkBundle(root);
  assert(findingCodes(report.okf.errors).includes('missing_frontmatter'), `expected missing_frontmatter, got ${JSON.stringify(report.okf.errors)}`);
  assert(report.okf.errors.some((e) => e.file === 'patterns/naked.md'), 'error must name the file');
  assert(report.ok === false, 'an OKF error must fail the check');
});

await check('OKF error: unparseable frontmatter (unclosed ---) names the file with the typed parser failure', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/broken.md', '---\ntype: bee.pattern\nno closing delimiter\n');
  const report = checkBundle(root);
  const err = report.okf.errors.find((e) => e.code === 'unparseable_frontmatter');
  assert(err, `expected unparseable_frontmatter, got ${JSON.stringify(report.okf.errors)}`);
  assert(err.file === 'patterns/broken.md', `error must name the file, got ${JSON.stringify(err)}`);
});

await check('OKF error: empty type and absent type both flagged (OKF §4.1: type is REQUIRED)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/empty-type.md', conceptText({ type: '', id: 'empty-type' }));
  writeBundleFile(root, 'patterns/no-type.md', conceptText({ id: 'no-type', omitRoot: ['type'] }));
  const report = checkBundle(root);
  const emptyTypeErrors = report.okf.errors.filter((e) => e.code === 'empty_type');
  assert(emptyTypeErrors.length === 2, `expected two empty_type errors, got ${JSON.stringify(report.okf.errors)}`);
  assert(new Set(emptyTypeErrors.map((e) => e.file)).size === 2, 'each error must name its own file');
});

await check('OKF error: frontmatter in a non-root index.md; a root index.md with only okf_version is clean', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'index.md', '---\nokf_version: "0.1"\n---\n\n# Bundle index\n');
  writeBundleFile(root, 'areas/demo/index.md', '---\nokf_version: "0.1"\n---\n\n# Area index\n');
  const report = checkBundle(root);
  const err = report.okf.errors.find((e) => e.code === 'index_frontmatter');
  assert(err, `expected index_frontmatter, got ${JSON.stringify(report.okf.errors)}`);
  assert(err.file === 'areas/demo/index.md', `error must name the non-root index, got ${JSON.stringify(err)}`);
  assert(!report.okf.errors.some((e) => e.file === 'index.md'), `root index.md with only okf_version must be clean, got ${JSON.stringify(report.okf.errors)}`);
});

await check('OKF error: root index.md carrying any key but okf_version', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'index.md', '---\nokf_version: "0.1"\ntitle: sneaky\n---\n\n# Bundle index\n');
  const report = checkBundle(root);
  const err = report.okf.errors.find((e) => e.code === 'root_index_extra_keys');
  assert(err, `expected root_index_extra_keys, got ${JSON.stringify(report.okf.errors)}`);
  assert(/title/.test(err.message), `the offending key must be named, got ${JSON.stringify(err)}`);
});

await check('OKF error: log.md date heading not ISO 8601 (OKF §7); ISO headings pass', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'log.md', '# Log\n\n## 2026-07-22\n\nFine.\n\n## July 22, 2026\n\nNot fine.\n');
  const report = checkBundle(root);
  const errs = report.okf.errors.filter((e) => e.code === 'log_heading_not_iso');
  assert(errs.length === 1, `exactly the non-ISO heading must be flagged, got ${JSON.stringify(report.okf.errors)}`);
  assert(/July 22, 2026/.test(errs[0].message), `the offending heading must be quoted, got ${JSON.stringify(errs[0])}`);
});

// ─── profile warning classes (D4, one per class; never OKF errors) ─────────

await check('profile warning: type outside the D18 nine warns, does not error, exits ok un-strict', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/guide.md', conceptText({ type: 'bee.guide', id: 'guide-1' }));
  const report = checkBundle(root);
  assert(report.okf.errors.length === 0, `unknown type is a SHOULD, never an error: ${JSON.stringify(report.okf.errors)}`);
  const warn = report.profile.warnings.find((w) => w.code === 'unknown_type');
  assert(warn && warn.file === 'patterns/guide.md' && /bee\.guide/.test(warn.message), `expected unknown_type naming bee.guide, got ${JSON.stringify(report.profile.warnings)}`);
  assert(report.ok === true, 'warnings alone must not fail un-strict');
});

await check('profile warning: missing profile-required field (D10: never invented, warned by name)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/undescribed.md', conceptText({ id: 'undescribed', omitRoot: ['description'] }));
  const report = checkBundle(root);
  const warn = report.profile.warnings.find((w) => w.code === 'missing_profile_field' && /description/.test(w.message));
  assert(warn && warn.file === 'patterns/undescribed.md', `expected missing_profile_field naming description, got ${JSON.stringify(report.profile.warnings)}`);
  assert(report.okf.errors.length === 0, 'a missing profile field is never an OKF error');
});

await check('profile warning: dangling required_context path; a resolving path stays silent', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'areas/demo/overview.md', conceptText({ type: 'bee.area', id: 'demo-overview', extraBee: { authoritative_for: 'demo-overview-subject' } }));
  writeBundleFile(root, 'patterns/linked.md', conceptText({ id: 'linked', extraBee: { required_context: ['areas/demo/overview.md', 'areas/ghost/nothing.md'] } }));
  const report = checkBundle(root);
  const dangling = report.profile.warnings.filter((w) => w.code === 'dangling_required_context');
  assert(dangling.length === 1, `only the ghost path may warn, got ${JSON.stringify(report.profile.warnings)}`);
  assert(/areas\/ghost\/nothing\.md/.test(dangling[0].message) && dangling[0].file === 'patterns/linked.md', `warning must name file and target, got ${JSON.stringify(dangling[0])}`);
});

await check('profile warning: dangling supersedes id; a resolving id stays silent', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/old.md', conceptText({ id: 'old-pattern', lifecycle: 'superseded' }));
  writeBundleFile(root, 'patterns/new.md', conceptText({ id: 'new-pattern', extraBee: { supersedes: 'old-pattern' } }));
  writeBundleFile(root, 'patterns/orphan.md', conceptText({ id: 'orphan', extraBee: { supersedes: 'never-existed' } }));
  const report = checkBundle(root);
  const dangling = report.profile.warnings.filter((w) => w.code === 'dangling_supersedes');
  assert(dangling.length === 1 && dangling[0].file === 'patterns/orphan.md' && /never-existed/.test(dangling[0].message), `expected one dangling_supersedes for orphan, got ${JSON.stringify(report.profile.warnings)}`);
});

await check('profile warning: duplicate bee.id (D31: id is globally unique)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/a.md', conceptText({ id: 'same-id' }));
  writeBundleFile(root, 'patterns/b.md', conceptText({ id: 'same-id' }));
  const report = checkBundle(root);
  const dup = report.profile.warnings.find((w) => w.code === 'duplicate_id');
  assert(dup && /same-id/.test(dup.message), `expected duplicate_id naming same-id, got ${JSON.stringify(report.profile.warnings)}`);
  assert(/patterns\/a\.md/.test(dup.message) || dup.file === 'patterns/a.md', 'both claimants must be traceable from the warning');
});

// ─── G14 LAYER 3 (cell f3-3): the backstop BITES ───────────────────────────
//
// f3-2 shipped `duplicate_authoritative_for` as a profile WARNING, and the
// chain runs `knowledge check` WITHOUT --strict — so the cited backstop never
// blocked anything. Layer 1's normalization can never catch a genuine
// word-order paraphrase, so the backstop is the only thing standing between a
// forked subject and a bundle where no reader can tell which file is true.
// It is therefore promoted to a chain-FAILING profile error.
//
// ORDERING PROOF (cell f3-3, before the promotion was written): the live
// bundle was verified duplicate-free first —
//   $ node .bee/bin/bee.mjs knowledge check --json
//   counts: {"files":133,"concepts":116,"errors":0,"warnings":0}
//   116 concepts, 63 string authority claims, 0 malformed, 0 duplicates
//   (exact AND under the hardened normalization)
// so promoting cannot red existing content. The pin below keeps that true.

await check('profile ERROR: duplicate bee.authoritative_for FAILS the chain (D31: one subject, one authority)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'areas/x/one.md', conceptText({ type: 'bee.area', id: 'x-one', extraBee: { authoritative_for: 'gates' } }));
  writeBundleFile(root, 'areas/x/two.md', conceptText({ type: 'bee.area', id: 'x-two', extraBee: { authoritative_for: 'gates' } }));
  const report = checkBundle(root);
  const dup = report.profile.errors.find((w) => w.code === 'duplicate_authoritative_for');
  assert(dup && /gates/.test(dup.message), `expected duplicate_authoritative_for naming the subject, got ${JSON.stringify(report.profile)}`);
  assert(/areas\/x\/one\.md/.test(dup.message) && /areas\/x\/two\.md/.test(dup.message), `both claimants must be named, got ${dup.message}`);
  assert(report.ok === false, 'a duplicated authority is chain-failing WITHOUT --strict — a warning that never blocks is not a backstop');
  assert(
    !report.profile.warnings.some((w) => w.code === 'duplicate_authoritative_for'),
    'promoted, not duplicated across buckets',
  );
});

await check('profile ERROR: duplicate authority is grouped by the HARDENED subject — punctuation and homoglyphs do not launder a fork', async () => {
  for (const [label, second] of [
    ['trailing period', 'gates.'],
    ['case + whitespace', '  GATES  '],
    ['Cyrillic-a homoglyph', 'gаtes'], // U+0430
    ['fullwidth', 'ｇａｔｅｓ'],
  ]) {
    const root = makeRepo();
    writeBundleFile(root, 'areas/x/one.md', conceptText({ type: 'bee.area', id: 'x-one', extraBee: { authoritative_for: 'gates' } }));
    writeBundleFile(root, 'areas/x/two.md', conceptText({ type: 'bee.area', id: 'x-two', extraBee: { authoritative_for: second } }));
    const report = checkBundle(root);
    const dup = report.profile.errors.find((w) => w.code === 'duplicate_authoritative_for');
    assert(dup, `${label}: exact-string grouping misses this; the hardened grouping must not. got ${JSON.stringify(report.profile)}`);
    assert(report.ok === false, `${label}: must fail the chain`);
  }
});

await check('profile ERROR: a MALFORMED bee.authoritative_for is a chain-failing error naming the file, never a silent skip', async () => {
  // The reachable set, measured against the D12 parser: `42`/`null` parse as
  // STRINGS, and a mapping is already an `unparseable_frontmatter` OKF error.
  for (const [label, literal] of [
    ['array', '[gates, locks]'],
    ['boolean', 'true'],
    ['empty string', '""'],
    ['blank string', '"   "'],
  ]) {
    const root = makeRepo();
    const bent = conceptText({ type: 'bee.area', id: 'x-bad' }).replace(
      'lifecycle: active',
      `lifecycle: active\n  authoritative_for: ${literal}`,
    );
    writeBundleFile(root, 'areas/x/bad.md', bent);
    const report = checkBundle(root);
    assert(
      report.okf.errors.length === 0,
      `${label}: the frontmatter itself parses — this is a profile fault, not an OKF one: ${JSON.stringify(report.okf.errors)}`,
    );
    const bad = report.profile.errors.find((e) => e.code === 'malformed_authoritative_for');
    assert(bad && bad.file === 'areas/x/bad.md', `${label}: expected malformed_authoritative_for naming the file, got ${JSON.stringify(report.profile)}`);
    assert(report.ok === false, `${label}: must fail the chain`);
  }
});

await check('a clean bundle still reports profile.errors as an empty array and stays green (no new noise)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'areas/x/one.md', conceptText({ type: 'bee.area', id: 'x-one', extraBee: { authoritative_for: 'gates' } }));
  writeBundleFile(root, 'areas/x/two.md', conceptText({ type: 'bee.area', id: 'x-two', extraBee: { authoritative_for: 'locks' } }));
  const report = checkBundle(root);
  assert(Array.isArray(report.profile.errors) && report.profile.errors.length === 0, `expected [], got ${JSON.stringify(report.profile.errors)}`);
  assert(report.ok === true, `a clean bundle stays green, got ${JSON.stringify(report)}`);
});

await check("LIVE BUNDLE PIN (f3-3): bee's own bundle carries zero duplicate and zero malformed authority claims", async () => {
  const report = checkBundle(REPO_ROOT);
  assert(
    report.profile.errors.length === 0,
    `the promotion must never red existing content — live bundle profile errors: ${JSON.stringify(report.profile.errors)}`,
  );
  assert(report.okf.errors.length === 0, `live bundle OKF errors: ${JSON.stringify(report.okf.errors)}`);
  assert(report.ok === true, 'the live bundle passes the promoted check');
});

// ─── advisor round-trip guard: misparse candidates warn, never silently ────

await check('round-trip guard: colon in an unquoted title parses (data intact) and warns not_canonical', async () => {
  const root = makeRepo();
  const canonical = conceptText({ id: 'colon-title' });
  const bent = canonical.replace('title: A demo pattern', 'title: Routing: the golden rule');
  writeBundleFile(root, 'patterns/colon.md', bent);
  const report = checkBundle(root);
  assert(report.okf.errors.length === 0, `a colon title is not an OKF error: ${JSON.stringify(report.okf.errors)}`);
  const warn = report.profile.warnings.find((w) => w.code === 'not_canonical' && w.file === 'patterns/colon.md');
  assert(warn, `expected not_canonical naming the file, got ${JSON.stringify(report.profile.warnings)}`);
  const parsed = parseFrontmatter(fs.readFileSync(path.join(bundleDir(root), 'patterns/colon.md'), 'utf8'));
  assert(parsed.ok && parsed.data.title === 'Routing: the golden rule', `the colon value must survive parsing intact, got ${JSON.stringify(parsed.data)}`);
});

await check('round-trip guard: "#" mid-value is kept as data (never comment-stripped) and warns not_canonical', async () => {
  const root = makeRepo();
  const canonical = conceptText({ id: 'hash-title' });
  const bent = canonical.replace('title: A demo pattern', 'title: value # not a comment');
  writeBundleFile(root, 'patterns/hash.md', bent);
  const report = checkBundle(root);
  assert(report.okf.errors.length === 0, `a hash mid-value is not an OKF error: ${JSON.stringify(report.okf.errors)}`);
  assert(report.profile.warnings.some((w) => w.code === 'not_canonical' && w.file === 'patterns/hash.md'), `expected not_canonical, got ${JSON.stringify(report.profile.warnings)}`);
  const parsed = parseFrontmatter(fs.readFileSync(path.join(bundleDir(root), 'patterns/hash.md'), 'utf8'));
  assert(parsed.ok && parsed.data.title === 'value # not a comment', `the "#" must remain part of the value, got ${JSON.stringify(parsed.data && parsed.data.title)}`);
});

await check('round-trip guard: CRLF frontmatter parses (data intact) and warns not_canonical', async () => {
  const root = makeRepo();
  const canonical = conceptText({ id: 'crlf-concept' });
  writeBundleFile(root, 'patterns/crlf.md', canonical.replace(/\n/g, '\r\n'));
  const report = checkBundle(root);
  assert(report.okf.errors.length === 0, `CRLF is not an OKF error: ${JSON.stringify(report.okf.errors)}`);
  assert(report.profile.warnings.some((w) => w.code === 'not_canonical' && w.file === 'patterns/crlf.md'), `expected not_canonical for CRLF, got ${JSON.stringify(report.profile.warnings)}`);
  const parsed = parseFrontmatter(fs.readFileSync(path.join(bundleDir(root), 'patterns/crlf.md'), 'utf8'));
  assert(parsed.ok && parsed.data.bee && parsed.data.bee.id === 'crlf-concept', `CRLF content must parse to the same data, got ${JSON.stringify(parsed.data)}`);
});

await check('a fully canonical bundle yields zero not_canonical warnings', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/clean.md', conceptText({ id: 'clean', title: 'Tiêu đề tiếng Việt không dấu ngoặc' }));
  const report = checkBundle(root);
  assert(!report.profile.warnings.some((w) => w.code === 'not_canonical'), `canonical files must not warn, got ${JSON.stringify(report.profile.warnings)}`);
});

// ─── D23: the walk never leaves docs/knowledge/ ─────────────────────────────

await check('check ignores broken .md outside docs/knowledge/ (D23: files outside the bundle are not concepts)', async () => {
  const root = makeRepo();
  fs.mkdirSync(path.join(root, 'docs', 'specs'), { recursive: true });
  fs.writeFileSync(path.join(root, 'docs', 'specs', 'broken.md'), '---\ntotally: broken: yaml\nno close\n', 'utf8');
  fs.writeFileSync(path.join(root, 'docs', 'legacy.md'), '# no frontmatter\n', 'utf8');
  writeBundleFile(root, 'patterns/clean.md', conceptText({ id: 'clean' }));
  const report = checkBundle(root);
  assert(report.okf.errors.length === 0 && report.counts.files === 1, `only the one bundle file may be seen, got ${JSON.stringify(report)}`);
});

// ─── --strict promotes warnings (D4/D13) ────────────────────────────────────

await check('strict flip: a warnings-only bundle is ok un-strict and not ok under strict', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/guide.md', conceptText({ type: 'bee.guide', id: 'guide-1' }));
  const loose = checkBundle(root);
  assert(loose.okf.errors.length === 0 && loose.profile.warnings.length > 0 && loose.ok === true, `un-strict must pass on warnings only, got ${JSON.stringify({ ok: loose.ok, counts: loose.counts })}`);
  const strict = checkBundle(root, { strict: true });
  assert(strict.ok === false, 'strict must fail on any finding');
});

// ─── CLI wiring: bee knowledge check (D13 --json shape + exit codes) ────────

await check('CLI: knowledge check --json on an empty repo exits 0 with the D13 {okf,profile,counts} shape', async () => {
  const root = makeRepo();
  const result = await runBee(['knowledge', 'check', '--json'], root);
  assert(result.status === 0, `expected exit 0, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const report = JSON.parse(result.stdout);
  assert(report.okf && Array.isArray(report.okf.errors) && report.okf.errors.length === 0, `expected okf.errors [], got ${result.stdout}`);
  assert(report.profile && Array.isArray(report.profile.warnings) && report.profile.warnings.length === 0, `expected profile.warnings [], got ${result.stdout}`);
  assert(report.counts && report.counts.concepts === 0, `expected zero concepts, got ${result.stdout}`);
});

await check('CLI: an OKF error exits non-zero; warnings-only exits 0; --strict flips warnings-only to non-zero (D13)', async () => {
  const errRoot = makeRepo();
  writeBundleFile(errRoot, 'patterns/naked.md', '# No frontmatter\n');
  const errResult = await runBee(['knowledge', 'check', '--json'], errRoot);
  assert(errResult.status !== 0, `an OKF error must exit non-zero, got ${errResult.status}: ${errResult.stdout}`);
  const errReport = JSON.parse(errResult.stdout);
  assert(errReport.okf.errors.length === 1, `expected the one error in --json output, got ${errResult.stdout}`);

  const warnRoot = makeRepo();
  writeBundleFile(warnRoot, 'patterns/guide.md', conceptText({ type: 'bee.guide', id: 'guide-1' }));
  const loose = await runBee(['knowledge', 'check', '--json'], warnRoot);
  assert(loose.status === 0, `warnings-only must exit 0, got ${loose.status}: ${loose.stdout} ${loose.stderr}`);
  assert(JSON.parse(loose.stdout).profile.warnings.length > 0, `the warning must still be reported, got ${loose.stdout}`);
  const strict = await runBee(['knowledge', 'check', '--strict', '--json'], warnRoot);
  assert(strict.status !== 0, `--strict must exit non-zero on warnings, got ${strict.status}: ${strict.stdout}`);
});

await check('CLI (f3-3): a duplicated authority exits NON-ZERO with no --strict, and prints as an ERROR', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'areas/x/one.md', conceptText({ type: 'bee.area', id: 'x-one', extraBee: { authoritative_for: 'gates' } }));
  writeBundleFile(root, 'areas/x/two.md', conceptText({ type: 'bee.area', id: 'x-two', extraBee: { authoritative_for: 'gates' } }));
  const human = await runBee(['knowledge', 'check'], root);
  assert(human.status !== 0, `the chain runs check WITHOUT --strict; a fork must still fail it, got ${human.status}: ${human.stdout}`);
  assert(/ERROR \[duplicate_authoritative_for\]/.test(human.stdout), `it must print as an ERROR, not a WARN: ${human.stdout}`);
  const json = await runBee(['knowledge', 'check', '--json'], root);
  assert(json.status !== 0, `--json must agree, got ${json.status}`);
  const report = JSON.parse(json.stdout);
  assert(
    Array.isArray(report.profile.errors) && report.profile.errors.some((e) => e.code === 'duplicate_authoritative_for'),
    `the D13 payload carries profile.errors, got ${json.stdout}`,
  );
});

await check('CLI: human (no --json) output names each finding and ends with a summary line', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'patterns/naked.md', '# No frontmatter\n');
  const result = await runBee(['knowledge', 'check'], root);
  assert(result.status !== 0, `expected non-zero exit, got ${result.status}`);
  assert(/patterns\/naked\.md/.test(result.stdout), `finding must name the file, got ${result.stdout}`);
  assert(/knowledge check:/.test(result.stdout), `expected the summary line, got ${result.stdout}`);
});

await check('CLI: knowledge.check appears in the bee --help --json manifest (test_bee_cli conformance)', async () => {
  const result = await runBee(['--help', '--json'], makeRepo());
  assert(result.status === 0, `--help --json must exit 0, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  const entry = manifest.commands.find((c) => c.name === 'knowledge.check');
  assert(entry, 'manifest must carry knowledge.check');
  assert(entry.invoke === 'bee knowledge check', `invoke must be "bee knowledge check", got ${entry.invoke}`);
  assert(Array.isArray(entry.examples) && entry.examples.length > 0, 'entry must carry runnable examples');
  assert(entry.parameters && entry.parameters.type === 'object' && entry.parameters.properties.strict, 'parameters must be JSON-Schema with a strict flag');
});

// ═══ okf-4 (S3): bee knowledge index + bee knowledge list ═══════════════════
// RED-FIRST (cell okf-4): these checks are written and run red (the index and
// list verbs do not exist yet) BEFORE any implementation.

/** Collect every index.md under the fixture bundle: Map rel -> content. */
function collectIndexFiles(root) {
  const dir = bundleDir(root);
  const out = new Map();
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
      else if (entry.isFile() && entry.name === 'index.md') out.set(childRel, fs.readFileSync(path.join(abs, entry.name), 'utf8'));
    }
  };
  if (fs.existsSync(dir)) walk(dir, '');
  return out;
}

/** Fixture bundle exercising nested dirs, criticality, and every list filter. */
function makeIndexFixture() {
  const root = makeRepo();
  writeBundleFile(root, 'areas/demo/overview.md', conceptText({
    type: 'bee.area', id: 'demo-overview', title: 'Demo overview', description: 'Overview of the demo area',
    extraBee: { areas: ['routing'], authoritative_for: 'demo-overview' },
  }));
  writeBundleFile(root, 'areas/demo/rules.md', conceptText({
    type: 'bee.area', id: 'demo-rules', title: 'Demo rules', description: 'Rules of the demo area', lifecycle: 'draft',
    extraBee: { areas: ['routing'], authoritative_for: 'demo-rules' },
  }));
  writeBundleFile(root, 'patterns/critical-one.md', conceptText({
    id: 'critical-one', title: 'A critical pattern', description: 'Always in context',
    extraBee: { critical: true },
  }));
  writeBundleFile(root, 'patterns/plain.md', conceptText({
    id: 'plain-one', title: 'A plain pattern', description: 'Not critical',
  }));
  writeBundleFile(root, 'log.md', '# Log\n\n## 2026-07-22\n\n- Fixture bundle created.\n');
  return root;
}

await check('CLI: knowledge index generates an index at every level with concepts (root included); two consecutive runs are byte-identical, LF-only (D21)', async () => {
  const root = makeIndexFixture();
  const first = await runBee(['knowledge', 'index', '--json'], root);
  assert(first.status === 0, `first index run must exit 0, got ${first.status}: stdout=${first.stdout} stderr=${first.stderr}`);
  const snapshot1 = collectIndexFiles(root);
  const expected = ['areas/demo/index.md', 'areas/index.md', 'index.md', 'patterns/index.md'];
  assert(JSON.stringify([...snapshot1.keys()].sort()) === JSON.stringify(expected), `expected exactly ${JSON.stringify(expected)}, got ${JSON.stringify([...snapshot1.keys()].sort())}`);
  for (const [rel, content] of snapshot1) {
    assert(!content.includes('\r'), `${rel}: generated index must be LF-only`);
    assert(!/\d{2}:\d{2}:\d{2}/.test(content), `${rel}: generated index must carry no wall-clock value`);
  }
  const second = await runBee(['knowledge', 'index', '--json'], root);
  assert(second.status === 0, `second index run must exit 0, got ${second.status}: ${second.stderr}`);
  const snapshot2 = collectIndexFiles(root);
  assert(snapshot2.size === snapshot1.size, `run 2 must generate the same file set, got ${JSON.stringify([...snapshot2.keys()])}`);
  for (const [rel, content] of snapshot1) {
    assert(snapshot2.get(rel) === content, `${rel}: two consecutive runs must be byte-identical`);
  }
});

await check('CLI: index --check exits non-zero naming a doctored stale index; regeneration heals it (decisions render --check idiom)', async () => {
  const root = makeIndexFixture();
  const render = await runBee(['knowledge', 'index'], root);
  assert(render.status === 0, `render must exit 0, got ${render.status}: ${render.stderr}`);
  const fresh = await runBee(['knowledge', 'index', '--check', '--json'], root);
  assert(fresh.status === 0, `--check on a fresh render must exit 0, got ${fresh.status}: ${fresh.stdout}`);
  const doctored = path.join(bundleDir(root), 'areas', 'demo', 'index.md');
  fs.appendFileSync(doctored, '\nHand-edited drift.\n', 'utf8');
  const stale = await runBee(['knowledge', 'index', '--check'], root);
  assert(stale.status !== 0, `--check over a doctored index must exit non-zero, got ${stale.status}: ${stale.stdout}`);
  assert(/areas\/demo\/index\.md/.test(stale.stdout), `--check must NAME the stale file, got ${stale.stdout}`);
  const heal = await runBee(['knowledge', 'index'], root);
  assert(heal.status === 0, `regeneration must exit 0, got ${heal.status}: ${heal.stderr}`);
  const healed = await runBee(['knowledge', 'index', '--check', '--json'], root);
  assert(healed.status === 0, `--check after regeneration must exit 0 (healed), got ${healed.status}: ${healed.stdout}`);
});

await check('generated non-root indexes carry NO frontmatter — only the HTML provenance comment (D4/D21)', async () => {
  const root = makeIndexFixture();
  await runBee(['knowledge', 'index'], root);
  for (const rel of ['areas/index.md', 'areas/demo/index.md', 'patterns/index.md']) {
    const content = fs.readFileSync(path.join(bundleDir(root), rel), 'utf8');
    const parsed = parseFrontmatter(content);
    assert(parsed.ok === true && parsed.present === false, `${rel}: a non-root index must carry no frontmatter, got ${JSON.stringify(parsed)}`);
    assert(content.startsWith('<!--'), `${rel}: must open with the HTML provenance comment, got ${JSON.stringify(content.slice(0, 40))}`);
    assert(/GENERATED FILE — do not hand-edit/.test(content), `${rel}: provenance header must say GENERATED FILE — do not hand-edit`);
    assert(/bee knowledge index/.test(content), `${rel}: provenance header must name the regenerate command`);
  }
});

await check('generated root index keeps okf_version-only frontmatter + provenance comment, and the generated bundle passes knowledge check', async () => {
  const root = makeIndexFixture();
  await runBee(['knowledge', 'index'], root);
  const content = fs.readFileSync(path.join(bundleDir(root), 'index.md'), 'utf8');
  const parsed = parseFrontmatter(content);
  assert(parsed.ok === true && parsed.present === true, `root index must carry frontmatter, got ${JSON.stringify(parsed)}`);
  assert(JSON.stringify(Object.keys(parsed.data)) === JSON.stringify(['okf_version']), `root index frontmatter must carry ONLY okf_version, got ${JSON.stringify(parsed.data)}`);
  assert(parsed.data.okf_version === '0.1', `okf_version must be "0.1", got ${JSON.stringify(parsed.data.okf_version)}`);
  assert(/GENERATED FILE — do not hand-edit/.test(content), 'root index must carry the provenance comment');
  const checkResult = await runBee(['knowledge', 'check', '--json'], root);
  assert(checkResult.status === 0, `generated bundle must pass knowledge check, got ${checkResult.status}: ${checkResult.stdout}`);
  const report = JSON.parse(checkResult.stdout);
  assert(report.okf.errors.length === 0, `generated indexes must produce zero OKF errors, got ${JSON.stringify(report.okf.errors)}`);
});

await check('root index carries a "## Critical patterns" section listing exactly the bee.critical concepts (D21)', async () => {
  const root = makeIndexFixture();
  await runBee(['knowledge', 'index'], root);
  const content = fs.readFileSync(path.join(bundleDir(root), 'index.md'), 'utf8');
  const start = content.indexOf('## Critical patterns');
  assert(start !== -1, `root index must carry a "## Critical patterns" section, got ${content}`);
  const rest = content.slice(start + '## Critical patterns'.length);
  const nextHeading = rest.indexOf('\n## ');
  const section = nextHeading === -1 ? rest : rest.slice(0, nextHeading);
  assert(/\(patterns\/critical-one\.md\)/.test(section), `critical section must link the critical concept, got ${section}`);
  assert(/A critical pattern/.test(section), `critical section must carry the concept's title, got ${section}`);
  assert(!/patterns\/plain\.md/.test(section), `a non-critical concept must NOT appear in the critical section, got ${section}`);
});

await check('empty bundle: knowledge index generates the root index only', async () => {
  const root = makeRepo();
  const result = await runBee(['knowledge', 'index', '--json'], root);
  assert(result.status === 0, `index over an empty bundle must exit 0, got ${result.status}: ${result.stderr}`);
  const files = collectIndexFiles(root);
  assert(JSON.stringify([...files.keys()]) === JSON.stringify(['index.md']), `an empty bundle must generate the root index only, got ${JSON.stringify([...files.keys()])}`);
  const checkResult = await runBee(['knowledge', 'check', '--json'], root);
  assert(checkResult.status === 0, `the generated empty-bundle root index must pass check, got ${checkResult.status}: ${checkResult.stdout}`);
});

// ─── bee knowledge list (D15): rows, filters, never content ────────────────

await check('CLI: knowledge list --json rows carry exactly path,id,type,lifecycle,title — never file content (D15)', async () => {
  const root = makeIndexFixture();
  const result = await runBee(['knowledge', 'list', '--json'], root);
  assert(result.status === 0, `list must exit 0, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const report = JSON.parse(result.stdout);
  assert(Array.isArray(report.concepts) && report.count === 4, `expected 4 concept rows, got ${result.stdout}`);
  for (const row of report.concepts) {
    assert(JSON.stringify(Object.keys(row).sort()) === JSON.stringify(['id', 'lifecycle', 'path', 'title', 'type']), `row keys must be exactly path,id,type,lifecycle,title — got ${JSON.stringify(row)}`);
  }
  const overview = report.concepts.find((r) => r.path === 'areas/demo/overview.md');
  assert(overview && overview.id === 'demo-overview' && overview.type === 'bee.area' && overview.lifecycle === 'active' && overview.title === 'Demo overview', `overview row wrong: ${JSON.stringify(overview)}`);
  assert(!result.stdout.includes('Body.'), `list output must never carry file content, got ${result.stdout}`);
});

await check('CLI: knowledge list filters --type/--lifecycle/--area narrow the rows (D15)', async () => {
  const root = makeIndexFixture();
  const byType = JSON.parse((await runBee(['knowledge', 'list', '--type', 'bee.area', '--json'], root)).stdout);
  assert(byType.count === 2 && byType.concepts.every((r) => r.type === 'bee.area'), `--type bee.area must yield the 2 area rows, got ${JSON.stringify(byType)}`);
  const byLifecycle = JSON.parse((await runBee(['knowledge', 'list', '--lifecycle', 'draft', '--json'], root)).stdout);
  assert(byLifecycle.count === 1 && byLifecycle.concepts[0].id === 'demo-rules', `--lifecycle draft must yield only demo-rules, got ${JSON.stringify(byLifecycle)}`);
  const byArea = JSON.parse((await runBee(['knowledge', 'list', '--area', 'routing', '--json'], root)).stdout);
  assert(byArea.count === 2 && byArea.concepts.every((r) => r.path.startsWith('areas/demo/')), `--area routing must yield the 2 routing rows, got ${JSON.stringify(byArea)}`);
  const combined = JSON.parse((await runBee(['knowledge', 'list', '--type', 'bee.area', '--lifecycle', 'active', '--json'], root)).stdout);
  assert(combined.count === 1 && combined.concepts[0].id === 'demo-overview', `combined filters must intersect, got ${JSON.stringify(combined)}`);
});

await check('CLI: knowledge.index and knowledge.list appear in the bee --help --json manifest (test_bee_cli conformance)', async () => {
  const result = await runBee(['--help', '--json'], makeRepo());
  assert(result.status === 0, `--help --json must exit 0, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  for (const [name, invoke] of [['knowledge.index', 'bee knowledge index'], ['knowledge.list', 'bee knowledge list']]) {
    const entry = manifest.commands.find((c) => c.name === name);
    assert(entry, `manifest must carry ${name}`);
    assert(entry.invoke === invoke, `${name}: invoke must be "${invoke}", got ${entry.invoke}`);
    assert(Array.isArray(entry.examples) && entry.examples.length > 0, `${name}: entry must carry runnable examples`);
    assert(entry.examples.every((e) => !e.includes('--strict')), `${name}: no --strict anywhere in examples`);
  }
});

// ═══ okf-7 (S5): bee knowledge context --work <id> --budget <tokens> ═══════
// RED-FIRST (cell okf-7): these checks are written and run red (the context
// verb does not exist yet) BEFORE any implementation.
//
// D27 is the whole contract: resolve the work item by bee.id, walk
// required_context TRANSITIVELY with a cycle guard (a cycle is deduped
// SILENTLY, never an error), add the area decisions and every bee.critical
// concept, rank, cut at --budget using bytes/4 — and return an ordered
// MANIFEST of paths/sizes/reasons that NEVER carries file content.

/** Fixture bundle for `context`: a work item with a plan sibling, a 2-level
 *  required_context chain, TWO cycles (plan -> work item, and one <-> two), a
 *  critical pattern, a non-critical pattern (must be excluded), an area
 *  decision (must be included) and an off-area decision (must be excluded). */
function makeContextFixture() {
  const root = makeRepo();
  writeBundleFile(root, 'work/demo/work-item.md', conceptText({
    type: 'bee.work-item', id: 'demo-work', title: 'Demo work item', description: 'The work item context resolves',
    extraBee: { areas: ['alpha'], required_context: ['areas/alpha/one.md'], decisions: ['D1', 'D2 (the cited rationale)'] },
  }));
  writeBundleFile(root, 'work/demo/plan.md', conceptText({
    type: 'bee.plan', id: 'demo-plan', title: 'Demo plan', description: 'The plan sibling in the same work dir',
    extraBee: { areas: ['alpha'], required_context: ['work/demo/work-item.md'] },
  }));
  writeBundleFile(root, 'areas/alpha/one.md', conceptText({
    type: 'bee.area', id: 'alpha-one', title: 'Alpha one', description: 'Depth 1 required context',
    extraBee: { areas: ['alpha'], required_context: ['areas/alpha/two.md'], authoritative_for: 'alpha-one' },
  }));
  writeBundleFile(root, 'areas/alpha/two.md', conceptText({
    type: 'bee.area', id: 'alpha-two', title: 'Alpha two', description: 'Depth 2 required context, cycling back to one',
    extraBee: { areas: ['alpha'], required_context: ['areas/alpha/one.md'], authoritative_for: 'alpha-two' },
  }));
  writeBundleFile(root, 'patterns/critical.md', conceptText({
    id: 'crit-pattern', title: 'A critical pattern', description: 'Always in context',
    extraBee: { areas: ['unrelated'], critical: true },
  }));
  writeBundleFile(root, 'patterns/plain.md', conceptText({
    id: 'plain-pattern', title: 'A plain pattern', description: 'Never in context',
    extraBee: { areas: ['unrelated'] },
  }));
  writeBundleFile(root, 'decisions/alpha.md', conceptText({
    type: 'bee.decision', id: 'dec-alpha', title: 'An alpha decision', description: 'Overlaps the work item areas',
    extraBee: { areas: ['alpha'] },
  }));
  writeBundleFile(root, 'decisions/beta.md', conceptText({
    type: 'bee.decision', id: 'dec-beta', title: 'A beta decision', description: 'No area overlap',
    extraBee: { areas: ['beta'] },
  }));
  return root;
}

const CONTEXT_EXPECTED_ORDER = [
  'docs/knowledge/work/demo/work-item.md',
  'docs/knowledge/work/demo/plan.md',
  'docs/knowledge/areas/alpha/one.md',
  'docs/knowledge/areas/alpha/two.md',
  'docs/knowledge/patterns/critical.md',
  'docs/knowledge/decisions/alpha.md',
];

await check('CLI: knowledge context --json on a large budget returns the full ranked manifest in D27 order, cycles deduped silently', async () => {
  const root = makeContextFixture();
  const result = await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '100000', '--json'], root);
  assert(result.status === 0, `context must exit 0, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  // f3-1/G11 widened this key set: the conservation and audit fields are part
  // of the contract now, so a payload that drops one is a regression.
  assert(JSON.stringify(Object.keys(manifest).sort()) === JSON.stringify(['budget', 'critical_total', 'decisions', 'entries', 'estimator', 'excluded', 'floor', 'total_est', 'truncated', 'work', 'zero_signal_count'].sort()), `manifest keys must be exactly {work,decisions,budget,estimator,total_est,entries,truncated,excluded,floor,critical_total,zero_signal_count}, got ${JSON.stringify(Object.keys(manifest))}`);
  assert(manifest.work === 'demo-work', `work must echo the resolved id, got ${JSON.stringify(manifest.work)}`);
  assert(JSON.stringify(manifest.decisions) === JSON.stringify(['D1', 'D2 (the cited rationale)']), `the header must surface the work item's OWN bee.decisions list, got ${JSON.stringify(manifest.decisions)}`);
  assert(manifest.budget === 100000, `budget must echo the request, got ${JSON.stringify(manifest.budget)}`);
  assert(manifest.estimator === 'bytes/4', `the estimator must be NAMED as bytes/4 (D27/D12), got ${JSON.stringify(manifest.estimator)}`);
  assert(JSON.stringify(manifest.entries.map((e) => e.path)) === JSON.stringify(CONTEXT_EXPECTED_ORDER), `ranking law violated.\nexpected ${JSON.stringify(CONTEXT_EXPECTED_ORDER)}\ngot      ${JSON.stringify(manifest.entries.map((e) => e.path))}`);
  assert(manifest.truncated.length === 0, `a large budget truncates nothing, got ${JSON.stringify(manifest.truncated)}`);
  const seen = new Set(manifest.entries.map((e) => e.path));
  assert(seen.size === manifest.entries.length, 'a cycle must be deduped: no path may appear twice');
  assert(!seen.has('docs/knowledge/patterns/plain.md'), 'a non-critical pattern must never be included');
  assert(!seen.has('docs/knowledge/decisions/beta.md'), 'a decision whose areas do not overlap must never be included');
});

await check('knowledge context manifest entries carry path/bytes/est_tokens/reason — never content (D27)', async () => {
  const root = makeContextFixture();
  const result = await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '100000', '--json'], root);
  assert(result.status === 0, `context must exit 0, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  let sum = 0;
  for (const entry of manifest.entries) {
    assert(JSON.stringify(Object.keys(entry).sort()) === JSON.stringify(['bytes', 'est_tokens', 'path', 'reason']), `entry keys must be exactly path,bytes,est_tokens,reason — got ${JSON.stringify(entry)}`);
    const abs = path.join(root, entry.path);
    assert(fs.existsSync(abs), `every manifest path must exist on disk: ${entry.path}`);
    assert(entry.bytes === fs.statSync(abs).size, `bytes must be the file's real size, got ${JSON.stringify(entry)}`);
    assert(entry.est_tokens === Math.ceil(entry.bytes / 4), `est_tokens must be ceil(bytes/4), got ${JSON.stringify(entry)}`);
    assert(typeof entry.reason === 'string' && entry.reason.trim(), `every entry needs a one-line reason, got ${JSON.stringify(entry)}`);
    assert(!entry.reason.includes('\n'), `the reason must be ONE line, got ${JSON.stringify(entry.reason)}`);
    sum += entry.est_tokens;
  }
  assert(manifest.total_est === sum, `total_est must equal the sum of included est_tokens, got ${manifest.total_est} vs ${sum}`);
  assert(!result.stdout.includes('Body.'), `the manifest must NEVER carry file content, got ${result.stdout}`);
  const human = await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '100000'], root);
  assert(human.status === 0, `human form must exit 0, got ${human.status}: ${human.stderr}`);
  assert(!human.stdout.includes('Body.'), `the human table must NEVER carry file content, got ${human.stdout}`);
  assert(/bytes\/4/.test(human.stdout), `the human form must name the estimator, got ${human.stdout}`);
  assert(/work\/demo\/work-item\.md/.test(human.stdout), `the human table must row every entry, got ${human.stdout}`);
});

await check('knowledge context reasons name the tier and, for required_context, the depth and the parent (D27)', async () => {
  const root = makeContextFixture();
  const manifest = JSON.parse((await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '100000', '--json'], root)).stdout);
  const reasonOf = (p) => (manifest.entries.find((e) => e.path === p) || {}).reason;
  assert(reasonOf('docs/knowledge/work/demo/work-item.md') === 'work item', `expected "work item", got ${JSON.stringify(reasonOf('docs/knowledge/work/demo/work-item.md'))}`);
  assert(/plan/.test(reasonOf('docs/knowledge/work/demo/plan.md')), `the plan sibling's reason must say so, got ${JSON.stringify(reasonOf('docs/knowledge/work/demo/plan.md'))}`);
  assert(/required_context depth 1 via .*work\/demo\/work-item\.md/.test(reasonOf('docs/knowledge/areas/alpha/one.md')), `depth-1 reason must name depth and parent, got ${JSON.stringify(reasonOf('docs/knowledge/areas/alpha/one.md'))}`);
  assert(/required_context depth 2 via .*areas\/alpha\/one\.md/.test(reasonOf('docs/knowledge/areas/alpha/two.md')), `depth-2 reason must name depth and parent, got ${JSON.stringify(reasonOf('docs/knowledge/areas/alpha/two.md'))}`);
  // f3-1/G5: a critical's reason now carries the number that put it there and
  // the rank it holds — an entry an agent cannot audit is an entry it must
  // take on trust.
  assert(/^critical pattern \(relevance [0-9.]+, rank \d+ of \d+(, floor)?\)$/.test(reasonOf('docs/knowledge/patterns/critical.md')), `a critical's reason must name its relevance score and rank, got ${JSON.stringify(reasonOf('docs/knowledge/patterns/critical.md'))}`);
  assert(/decision/.test(reasonOf('docs/knowledge/decisions/alpha.md')) && /alpha/.test(reasonOf('docs/knowledge/decisions/alpha.md')), `the area-decision reason must name the area, got ${JSON.stringify(reasonOf('docs/knowledge/decisions/alpha.md'))}`);
});

await check('knowledge context cuts at --budget: total_est <= budget, the cut entries are NAMED in truncated, and the floor is the ONE named exception to the prefix (D27 + f3-1/G5)', async () => {
  const root = makeContextFixture();
  const full = JSON.parse((await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '100000', '--json'], root)).stdout);
  assert(full.entries.length === 6, `fixture must yield 6 entries before the cut, got ${full.entries.length}`);
  assert(JSON.stringify(full.floor) === JSON.stringify(['docs/knowledge/patterns/critical.md']), `the fixture's one critical is the floor, got ${JSON.stringify(full.floor)}`);
  // Room for rank 1 and the floor, and nothing else.
  const tightBudget = full.entries[0].est_tokens
    + full.entries.find((e) => e.path === 'docs/knowledge/patterns/critical.md').est_tokens;
  const result = await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', String(tightBudget), '--json'], root);
  assert(result.status === 0, `a budget cut is not an error, got ${result.status}: ${result.stderr}`);
  const cut = JSON.parse(result.stdout);
  assert(cut.total_est <= tightBudget, `the budget is a HARD ceiling even with a floor: total_est ${cut.total_est} must be <= ${tightBudget}`);
  // The floor is why this is no longer a pure prefix: the plan sibling (rank 2)
  // is truncated so the floor critical (rank 5) is not evicted. Everything not
  // included is still NAMED.
  assert(cut.entries.some((e) => e.path === 'docs/knowledge/patterns/critical.md'), `the floor critical must survive the tight budget, got ${JSON.stringify(cut.entries.map((e) => e.path))}`);
  assert(cut.entries[0].path === CONTEXT_EXPECTED_ORDER[0], `rank 1 still leads the manifest, got ${JSON.stringify(cut.entries.map((e) => e.path))}`);
  const named = new Set([...cut.entries.map((e) => e.path), ...cut.truncated, ...cut.excluded.map((e) => e.path)]);
  for (const p of CONTEXT_EXPECTED_ORDER) assert(named.has(p), `every ranked path must be named somewhere, missing ${p}`);
  assert(named.size === CONTEXT_EXPECTED_ORDER.length, `nothing may be invented or lost, got ${JSON.stringify([...named])}`);

  const zero = JSON.parse((await runBee(['knowledge', 'context', '--work', 'demo-work', '--budget', '0', '--json'], root)).stdout);
  assert(zero.entries.length === 0 && zero.total_est === 0, `a zero budget includes nothing — the floor reserves from the budget, it never overdraws it, got ${JSON.stringify(zero.entries)}`);
  assert(JSON.stringify(zero.truncated) === JSON.stringify(CONTEXT_EXPECTED_ORDER), `a zero budget truncates everything, got ${JSON.stringify(zero.truncated)}`);
});

await check('knowledge context: an unknown --work id exits 1 with a typed error (D27)', async () => {
  const root = makeContextFixture();
  const result = await runBee(['knowledge', 'context', '--work', 'no-such-work', '--budget', '1000', '--json'], root);
  assert(result.status === 1, `an unknown work id must exit 1, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(typeof payload.error === 'string' && /unknown_work/.test(payload.error), `the failure must carry the typed unknown_work code, got ${result.stdout}`);
  assert(/no-such-work/.test(payload.error), `the failure must name the id it could not resolve, got ${result.stdout}`);
  const human = await runBee(['knowledge', 'context', '--work', 'no-such-work', '--budget', '1000'], root);
  assert(human.status === 1 && /unknown_work/.test(human.stderr), `the human form must fail on stderr with the typed code, got status=${human.status} stderr=${human.stderr}`);
});

await check('knowledge context: a bundle with no plan sibling and no decisions still resolves (tolerant, never a throw)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'work/lonely/work-item.md', conceptText({
    type: 'bee.work-item', id: 'lonely', title: 'A lonely work item', description: 'No plan, no areas, one dangling link',
    extraBee: { areas: [], required_context: ['areas/ghost/missing.md'], decisions: [] },
  }));
  const result = await runBee(['knowledge', 'context', '--work', 'lonely', '--budget', '5000', '--json'], root);
  assert(result.status === 0, `a lonely work item must still resolve, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.entries.length === 1 && manifest.entries[0].path === 'docs/knowledge/work/lonely/work-item.md', `expected the work item alone, got ${JSON.stringify(manifest.entries)}`);
  assert(JSON.stringify(manifest.decisions) === JSON.stringify([]), `an empty decisions list stays empty, got ${JSON.stringify(manifest.decisions)}`);
});

await check('CLI: knowledge.context appears in the bee --help --json manifest with a runnable example (test_bee_cli conformance)', async () => {
  const result = await runBee(['--help', '--json'], makeRepo());
  assert(result.status === 0, `--help --json must exit 0, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  const entry = manifest.commands.find((c) => c.name === 'knowledge.context');
  assert(entry, 'manifest must carry knowledge.context');
  assert(entry.invoke === 'bee knowledge context', `invoke must be "bee knowledge context", got ${entry.invoke}`);
  assert(Array.isArray(entry.examples) && entry.examples.length > 0, 'entry must carry runnable examples');
  assert(entry.examples.every((e) => !e.includes('--strict')), 'no --strict anywhere in examples');
  assert(entry.parameters && entry.parameters.properties.work && entry.parameters.properties.budget, 'parameters must carry work and budget');
  assert(JSON.stringify([...entry.parameters.required].sort()) === JSON.stringify(['budget', 'work']), `--work and --budget are both required, got ${JSON.stringify(entry.parameters.required)}`);
});

// ═══ okf-9 (S7): bee knowledge promote --work <id> ═════════════════════════
// RED-FIRST (cell okf-9): these checks are written and run red (the promote
// verb does not exist yet) BEFORE any implementation.
//
// D38 + D2 are the whole contract: promote READS the bundle and the capped
// cell traces in .bee/cells/*.json (a read of the runtime store — permitted;
// never a write) and PROPOSES three sections — a delivery draft in canonical
// emitter form, candidate area spec-sync bullets, and candidate pitfall
// patterns. It writes NOTHING, anywhere: applying a proposal is a human or
// agent decision, never a silent truth-write.

/** Write a cell trace file into the fixture repo's .bee/cells/ store. */
function writeCellFixture(root, cell) {
  const dir = path.join(root, '.bee', 'cells');
  fs.mkdirSync(dir, { recursive: true });
  writeJsonAtomic(path.join(dir, `${cell.id}.json`), cell);
}

/** Snapshot EVERY file under a repo root as rel -> exact bytes (hex). The
 *  zero-write proof compares two of these across a real promote run. */
function snapshotTree(root) {
  const out = new Map();
  const walk = (abs, rel) => {
    for (const entry of fs.readdirSync(abs, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const childRel = rel ? `${rel}/${entry.name}` : entry.name;
      if (entry.isDirectory()) walk(path.join(abs, entry.name), childRel);
      else if (entry.isFile()) out.set(childRel, fs.readFileSync(path.join(abs, entry.name)).toString('hex'));
    }
  };
  walk(root, '');
  return out;
}

/**
 * Fixture: a work item declaring one area, an area concept whose bee.sources
 * names the area's subject file, and five cell traces —
 *   demo-1  capped, behavior_change, touches the area subject, ONE deviation
 *   demo-2  capped, NOT behavior_change, clean (no deviation, no failure)
 *   demo-3  OPEN (never capped) — must never be mined
 *   demo-4  capped, behavior_change, no deviation but a failure_signature
 *   other-1 capped but belongs to ANOTHER feature — must never be mined
 */
function makePromoteFixture() {
  const root = makeRepo();
  writeBundleFile(root, 'work/demo/work-item.md', conceptText({
    type: 'bee.work-item', id: 'demo-work', title: 'Demo work item', description: 'The work item promote mines',
    extraBee: { areas: ['alpha'], required_context: [], decisions: ['D38'], lane: 'high-risk' },
  }));
  writeBundleFile(root, 'areas/alpha/one.md', conceptText({
    type: 'bee.area', id: 'alpha-one', title: 'Alpha one', description: 'The area whose subject demo-1 touched',
    extraBee: { areas: ['alpha'], sources: ['docs/specs/alpha.md'], authoritative_for: 'alpha-one' },
  }));
  writeCellFixture(root, {
    id: 'demo-1', feature: 'demo-work', lane: 'high-risk', behavior_change: true, status: 'capped',
    title: 'Migrate the alpha area', verify: 'node scripts/run_verify.mjs',
    trace: {
      outcome: 'alpha migrated into one concept, chain green',
      files_changed: ['docs/specs/alpha.md', 'lib/alpha.mjs'],
      deviations: ['Renamed the flag from --a to --alpha because --a already meant --agent'],
      behavior_change: true,
      capped_at: '2026-07-22T04:00:00.000Z',
      verification_evidence: '{"verify_tail":"PASS run_verify: 64 suite(s)"}',
      verify_passed: true,
      attempts: [{ n: 1, verdict: 'pass', failure_signature: null }],
    },
  });
  writeCellFixture(root, {
    id: 'demo-2', feature: 'demo-work', lane: 'tiny', behavior_change: false, status: 'capped',
    title: 'Docs only', verify: 'node scripts/run_verify.mjs',
    trace: {
      outcome: 'README rewritten',
      files_changed: ['README.md'],
      deviations: [],
      behavior_change: false,
      capped_at: '2026-07-22T05:00:00.000Z',
      verification_evidence: '{"verify_tail":"PASS run_verify: 64 suite(s)"}',
      verify_passed: true,
      attempts: [{ n: 1, verdict: 'pass', failure_signature: null }],
    },
  });
  writeCellFixture(root, {
    id: 'demo-3', feature: 'demo-work', lane: 'small', behavior_change: true, status: 'open',
    title: 'Never capped', verify: 'node scripts/run_verify.mjs',
    trace: { outcome: null, files_changed: [], deviations: ['this deviation must never be mined'], capped_at: null },
  });
  writeCellFixture(root, {
    id: 'demo-4', feature: 'demo-work', lane: 'small', behavior_change: true, status: 'capped',
    title: 'Recovered from a red run', verify: 'node scripts/run_verify.mjs',
    trace: {
      outcome: 'lock contention fixed',
      files_changed: ['lib/lock.mjs'],
      deviations: [],
      behavior_change: true,
      capped_at: '2026-07-22T06:00:00.000Z',
      verification_evidence: '{"verify_tail":"PASS run_verify: 64 suite(s)"}',
      verify_passed: true,
      attempts: [
        { n: 1, verdict: 'fail', failure_signature: 'EEXIST: store lock held by a stale pid' },
        { n: 2, verdict: 'pass', failure_signature: null },
      ],
    },
  });
  writeCellFixture(root, {
    id: 'other-1', feature: 'other-feature', lane: 'tiny', behavior_change: true, status: 'capped',
    title: 'Another feature entirely', verify: 'node scripts/run_verify.mjs',
    trace: {
      outcome: 'unrelated outcome',
      files_changed: ['docs/specs/alpha.md'],
      deviations: ['unrelated deviation'],
      behavior_change: true,
      capped_at: '2026-07-22T07:00:00.000Z',
    },
  });
  return root;
}

await check('CLI: knowledge promote --json names exactly the CAPPED cells of the work item feature, in id order (D38)', async () => {
  const root = makePromoteFixture();
  const result = await runBee(['knowledge', 'promote', '--work', 'demo-work', '--json'], root);
  assert(result.status === 0, `promote must exit 0, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const proposal = JSON.parse(result.stdout);
  assert(JSON.stringify(Object.keys(proposal).sort()) === JSON.stringify(['area_updates', 'cells', 'delivery', 'pattern_candidates', 'work', 'work_item', 'writes'].sort()), `proposal keys must be exactly {work,work_item,cells,delivery,area_updates,pattern_candidates,writes}, got ${JSON.stringify(Object.keys(proposal))}`);
  assert(proposal.work === 'demo-work', `work must echo the resolved id, got ${JSON.stringify(proposal.work)}`);
  assert(proposal.work_item === 'work/demo/work-item.md', `work_item must name the resolved concept path, got ${JSON.stringify(proposal.work_item)}`);
  assert(JSON.stringify(proposal.cells.map((c) => c.id)) === JSON.stringify(['demo-1', 'demo-2', 'demo-4']), `only the capped cells of THIS feature, id-sorted, may be mined — got ${JSON.stringify(proposal.cells.map((c) => c.id))}`);
  assert(JSON.stringify(proposal.writes) === JSON.stringify([]), `promote must declare zero writes, got ${JSON.stringify(proposal.writes)}`);
  for (const cell of proposal.cells) {
    assert(typeof cell.outcome === 'string' && cell.outcome, `every mined cell must carry its recorded outcome, got ${JSON.stringify(cell)}`);
  }
});

await check('knowledge promote: the DELIVERY DRAFT is a complete canonical bee.delivery concept that round-trips through check (zero not_canonical)', async () => {
  const root = makePromoteFixture();
  const proposal = JSON.parse((await runBee(['knowledge', 'promote', '--work', 'demo-work', '--json'], root)).stdout);
  const draft = proposal.delivery;
  assert(draft && draft.path === 'work/demo/delivery.md', `the draft must be ready to save as the work item's delivery.md sibling, got ${JSON.stringify(draft && draft.path)}`);
  const parsed = parseFrontmatter(draft.content);
  assert(parsed.ok === true && parsed.present === true, `the draft must carry parseable frontmatter, got ${JSON.stringify(parsed)}`);
  assert(parsed.data.type === 'bee.delivery', `the draft must be typed bee.delivery, got ${JSON.stringify(parsed.data.type)}`);
  assert(emitFrontmatter(parsed.data) === parsed.block, 'the draft frontmatter must already be canonical emitter form (re-emit byte-identical)');
  assert(typeof parsed.data.title === 'string' && parsed.data.title, 'the draft must carry a title');
  assert(typeof parsed.data.description === 'string' && parsed.data.description, 'the draft must carry a description');
  assert(parsed.data.bee.id === 'demo-work-delivery' && parsed.data.bee.lifecycle === 'active', `draft identity must derive from the work id, got ${JSON.stringify(parsed.data.bee)}`);
  assert(Array.isArray(parsed.data.bee.sources) && parsed.data.bee.sources.includes('.bee/cells/demo-1.json'), `the draft must cite the cell traces it was mined from, got ${JSON.stringify(parsed.data.bee.sources)}`);
  for (const id of ['demo-1', 'demo-2', 'demo-4']) {
    assert(draft.content.includes(id), `the draft body must name capped cell ${id}, got ${draft.content}`);
  }
  assert(!draft.content.includes('demo-3') && !draft.content.includes('other-1'), 'the draft must never name an uncapped or foreign cell');

  // Saved into the bundle, the draft must pass `knowledge check` cleanly.
  const abs = path.join(bundleDir(root), draft.path);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, draft.content, 'utf8');
  const checked = await runBee(['knowledge', 'check', '--json'], root);
  assert(checked.status === 0, `the saved draft must pass knowledge check, got ${checked.status}: ${checked.stdout}`);
  const report = JSON.parse(checked.stdout);
  assert(report.okf.errors.length === 0, `the saved draft must produce zero OKF errors, got ${JSON.stringify(report.okf.errors)}`);
  assert(!report.profile.warnings.some((w) => w.file === draft.path), `the saved draft must produce zero profile warnings of its own, got ${JSON.stringify(report.profile.warnings)}`);
  fs.rmSync(abs);
});

await check('knowledge promote: a cell with a deviation or a failure signature yields a pitfall candidate; a clean cell yields none (D18 polarity)', async () => {
  const root = makePromoteFixture();
  const proposal = JSON.parse((await runBee(['knowledge', 'promote', '--work', 'demo-work', '--json'], root)).stdout);
  const ids = proposal.pattern_candidates.map((c) => c.cell);
  assert(JSON.stringify(ids) === JSON.stringify(['demo-1', 'demo-4']), `exactly the deviation-carrying and failure-carrying cells become candidates, got ${JSON.stringify(ids)}`);
  const first = proposal.pattern_candidates[0];
  const parsed = parseFrontmatter(first.content);
  assert(parsed.ok && parsed.data.type === 'bee.pattern', `a candidate must be a bee.pattern concept, got ${JSON.stringify(parsed.data && parsed.data.type)}`);
  assert(parsed.data.bee.polarity === 'pitfall', `a candidate must carry bee.polarity pitfall (D18), got ${JSON.stringify(parsed.data.bee)}`);
  assert(parsed.data.bee.lifecycle === 'draft', `a proposal is a draft until a human accepts it, got ${JSON.stringify(parsed.data.bee.lifecycle)}`);
  assert(emitFrontmatter(parsed.data) === parsed.block, 'a candidate frontmatter must already be canonical emitter form');
  assert(Array.isArray(parsed.data.bee.sources) && parsed.data.bee.sources.includes('.bee/cells/demo-1.json'), `a candidate must cite the cell trace it came from, got ${JSON.stringify(parsed.data.bee.sources)}`);
  assert(/Renamed the flag from --a to --alpha/.test(first.content), `the candidate body must quote the recorded deviation verbatim, got ${first.content}`);
  const fromFailure = proposal.pattern_candidates[1];
  assert(/EEXIST: store lock held by a stale pid/.test(fromFailure.content), `a failure-signature candidate must quote the signature verbatim, got ${fromFailure.content}`);
  assert(!JSON.stringify(proposal.pattern_candidates).includes('demo-2'), 'a clean capped cell must never yield a pitfall candidate');
});

await check('knowledge promote: AREA UPDATES list the behavior_change cells whose files touch that area subject, each citing its cell id', async () => {
  const root = makePromoteFixture();
  const proposal = JSON.parse((await runBee(['knowledge', 'promote', '--work', 'demo-work', '--json'], root)).stdout);
  assert(proposal.area_updates.length === 1 && proposal.area_updates[0].area === 'alpha', `one area update section per work-item area, got ${JSON.stringify(proposal.area_updates)}`);
  const bullets = proposal.area_updates[0].bullets;
  assert(bullets.length === 1 && bullets[0].cell === 'demo-1', `only the behavior_change cell that touched the area subject may be proposed, got ${JSON.stringify(bullets)}`);
  assert(Array.isArray(bullets[0].files) && bullets[0].files.includes('docs/specs/alpha.md'), `the bullet must name the touched subject file, got ${JSON.stringify(bullets[0])}`);
  assert(/alpha migrated into one concept/.test(bullets[0].text), `the bullet text must come from the cell's recorded outcome, got ${JSON.stringify(bullets[0].text)}`);
  assert(!JSON.stringify(bullets).includes('demo-2'), 'a non-behavior_change cell is never an area-update candidate');
  assert(!JSON.stringify(bullets).includes('demo-4'), 'a cell that touched no subject of the area is never an area-update candidate');
});

await check('knowledge promote WRITES NOTHING: buildPromotion leaves the whole repo tree byte-identical, in-process (D2)', async () => {
  const root = makePromoteFixture();
  const before = snapshotTree(root);
  const proposal = buildPromotion(root, { work: 'demo-work' });
  assert(proposal.cells.length === 3, `the in-process call must mine the same cells, got ${JSON.stringify(proposal.cells.map((c) => c.id))}`);
  assert(JSON.stringify(proposal.writes) === JSON.stringify([]), 'buildPromotion must declare zero writes');
  const after = snapshotTree(root);
  assert(JSON.stringify([...after.keys()]) === JSON.stringify([...before.keys()]), `buildPromotion must create and delete NO file.\nbefore: ${JSON.stringify([...before.keys()])}\nafter:  ${JSON.stringify([...after.keys()])}`);
  for (const [rel, bytes] of before) {
    assert(after.get(rel) === bytes, `buildPromotion must not modify ${rel} — the bundle and the runtime store are read-only inputs (D2)`);
  }
});

await check('knowledge promote WRITES NOTHING through the CLI either: bundle and .bee/cells/ byte-identical; the only delta is the dispatcher cache every verb writes', async () => {
  const root = makePromoteFixture();
  // The dispatcher writes its own per-invocation files (.bee/cache/ manifest,
  // .bee/logs/timings.jsonl) on EVERY bee verb, promote included. That set is
  // DERIVED, never hand-listed: a read-only `knowledge check` in a virgin repo
  // is run first, and whatever it creates is by definition the dispatcher's,
  // not promote's — so a new dispatcher-owned file never turns this guard red
  // and, equally, never silently widens what promote itself may write.
  const virgin = makeRepo();
  const virginBefore = snapshotTree(virgin);
  await runBee(['knowledge', 'check', '--json'], virgin);
  const virginDelta = [...snapshotTree(virgin).keys()].filter((rel) => !virginBefore.has(rel));

  /** Everything promote could possibly be accused of writing: the bundle and
   *  the runtime store — minus the dispatcher's own derived per-run files. */
  const owned = (snapshot) => new Map([...snapshot].filter(([rel]) => !virginDelta.includes(rel)));
  const before = snapshotTree(root);
  const result = await runBee(['knowledge', 'promote', '--work', 'demo-work', '--json'], root);
  assert(result.status === 0, `promote must exit 0, got ${result.status}: ${result.stderr}`);
  const human = await runBee(['knowledge', 'promote', '--work', 'demo-work'], root);
  assert(human.status === 0, `the human form must exit 0 too, got ${human.status}: ${human.stderr}`);
  const after = snapshotTree(root);
  const beforeOwned = owned(before);
  const afterOwned = owned(after);
  assert(JSON.stringify([...afterOwned.keys()]) === JSON.stringify([...beforeOwned.keys()]), `promote must create and delete NO bundle or runtime-store file.\nbefore: ${JSON.stringify([...beforeOwned.keys()])}\nafter:  ${JSON.stringify([...afterOwned.keys()])}`);
  for (const [rel, bytes] of beforeOwned) {
    assert(afterOwned.get(rel) === bytes, `promote must not modify ${rel} (D2)`);
  }
  const delta = [...after.keys()].filter((rel) => !before.has(rel));
  // The delta is the dispatcher's, not promote's: the read-only `knowledge
  // check` run in a virgin repo above produces exactly the same files.
  assert(JSON.stringify(delta) === JSON.stringify(virginDelta), `promote's only file delta must be the per-run files every verb writes.\npromote: ${JSON.stringify(delta)}\ncheck:   ${JSON.stringify(virginDelta)}`);
});

await check('knowledge promote: the human form carries the three proposal sections and says nothing was written', async () => {
  const root = makePromoteFixture();
  const result = await runBee(['knowledge', 'promote', '--work', 'demo-work'], root);
  assert(result.status === 0, `the human form must exit 0, got ${result.status}: ${result.stderr}`);
  assert(/\(a\) DELIVERY DRAFT/.test(result.stdout), `expected the delivery-draft section, got ${result.stdout}`);
  assert(/\(b\) AREA UPDATES/.test(result.stdout), `expected the area-updates section, got ${result.stdout}`);
  assert(/\(c\) PATTERN CANDIDATES/.test(result.stdout), `expected the pattern-candidates section, got ${result.stdout}`);
  assert(/nothing was written/i.test(result.stdout), `the output must state that promote proposes and writes nothing, got ${result.stdout}`);
  assert(/demo-1/.test(result.stdout) && /demo-4/.test(result.stdout), `the human form must name the mined cells, got ${result.stdout}`);
});

await check('knowledge promote: an unknown --work id exits 1 with a typed error (D38)', async () => {
  const root = makePromoteFixture();
  const result = await runBee(['knowledge', 'promote', '--work', 'no-such-work', '--json'], root);
  assert(result.status === 1, `an unknown work id must exit 1, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const payload = JSON.parse(result.stdout);
  assert(typeof payload.error === 'string' && /unknown_work/.test(payload.error), `the failure must carry the typed unknown_work code, got ${result.stdout}`);
  assert(/no-such-work/.test(payload.error), `the failure must name the id it could not resolve, got ${result.stdout}`);
  const human = await runBee(['knowledge', 'promote', '--work', 'no-such-work'], root);
  assert(human.status === 1 && /unknown_work/.test(human.stderr), `the human form must fail on stderr with the typed code, got status=${human.status} stderr=${human.stderr}`);
});

await check('knowledge promote: a work item with no capped cells still proposes a canonical (empty-evidence) draft, never a throw', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'work/lonely/work-item.md', conceptText({
    type: 'bee.work-item', id: 'lonely', title: 'A lonely work item', description: 'No cells have been capped for it yet',
    extraBee: { areas: [], required_context: [], decisions: [] },
  }));
  const result = await runBee(['knowledge', 'promote', '--work', 'lonely', '--json'], root);
  assert(result.status === 0, `a work item with no cells must still resolve, got ${result.status}: ${result.stderr}`);
  const proposal = JSON.parse(result.stdout);
  assert(proposal.cells.length === 0 && proposal.pattern_candidates.length === 0 && proposal.area_updates.length === 0, `nothing to mine means empty sections, got ${JSON.stringify(proposal)}`);
  const parsed = parseFrontmatter(proposal.delivery.content);
  assert(parsed.ok && parsed.data.type === 'bee.delivery' && emitFrontmatter(parsed.data) === parsed.block, `the draft must still be canonical, got ${JSON.stringify(parsed)}`);
});

await check('CLI: knowledge.promote appears in the bee --help --json manifest with a runnable example (test_bee_cli conformance)', async () => {
  const result = await runBee(['--help', '--json'], makeRepo());
  assert(result.status === 0, `--help --json must exit 0, got ${result.status}: ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  const entry = manifest.commands.find((c) => c.name === 'knowledge.promote');
  assert(entry, 'manifest must carry knowledge.promote');
  assert(entry.invoke === 'bee knowledge promote', `invoke must be "bee knowledge promote", got ${entry.invoke}`);
  assert(Array.isArray(entry.examples) && entry.examples.length > 0, 'entry must carry runnable examples');
  assert(entry.examples.every((e) => !e.includes('--strict')), 'no --strict anywhere in examples');
  assert(entry.parameters && entry.parameters.properties.work, 'parameters must carry work');
  assert(JSON.stringify(entry.parameters.required) === JSON.stringify(['work']), `--work is the one required flag, got ${JSON.stringify(entry.parameters.required)}`);
  assert(/never writes|proposes/i.test(entry.description), `the description must state that promote proposes and never writes, got ${entry.description}`);
});

// ═══ f3-1 (G5/G10/G11): critical patterns are RANKED BY RELEVANCE and CUT ═══
// RED-FIRST (cell f3-1): every check below is written and run red — the
// relevance ranking, the floor, the `excluded` conservation array and the
// zero_signal_count guard do not exist yet — BEFORE any implementation.
//
// The acceptance is DISCRIMINATION, never survival. The rejected oracle is
// "the N relevant entries of the live manifest survive": the work item, its
// plan and its required_context occupy ranks 1-3 and the budget cut is a
// prefix cut, so that assertion holds even if every critical pattern were
// dropped. It is deliberately NOT written here.
//
// Measured on the live bundle before this design (advisor digest f3, finding
// 3): the okf-migration-f2 work item carries no bee.areas, only 2 of 50
// patterns carry any, and tag overlap against its five tags hits 1 of 50 — so
// 49 of 50 tie at zero. Tag overlap alone is therefore disqualified as the
// ranking signal, and a ranking where most items tie at zero must FAIL rather
// than ship as a path sort wearing a relevance label.

/** A work item whose text is the query, and ten criticals hand-labelled
 *  relevant / irrelevant against it. The relevant four are written in the work
 *  item's own vocabulary (ledger rows, schema migration, coverage gate, pinned
 *  snapshot); the irrelevant six are topically disjoint but each shares one
 *  generic operations word, exactly as a real corpus does — so "irrelevant"
 *  means LOW, never zero, and the zero-signal guard is not what separates
 *  them. */
const DISCRIMINATION_RELEVANT = [
  'docs/knowledge/patterns/rel-ledger-rows.md',
  'docs/knowledge/patterns/rel-coverage-gate.md',
  'docs/knowledge/patterns/rel-schema-rollback.md',
  'docs/knowledge/patterns/rel-enumerated-rows.md',
];
const DISCRIMINATION_IRRELEVANT = [
  'docs/knowledge/patterns/irr-powershell-bom.md',
  'docs/knowledge/patterns/irr-scratchpad-polling.md',
  'docs/knowledge/patterns/irr-viewport-screenshot.md',
  'docs/knowledge/patterns/irr-spawn-heartbeat.md',
  'docs/knowledge/patterns/irr-emoji-columns.md',
  'docs/knowledge/patterns/irr-dns-cache.md',
];

function makeDiscriminationFixture() {
  const root = makeRepo();
  writeBundleFile(root, 'work/billing/work-item.md', conceptText({
    type: 'bee.work-item',
    id: 'billing-migration',
    title: 'Migrate the billing ledger onto the new invoice schema',
    description: 'Move every ledger row into the new billing schema behind a coverage gate that derives its ground truth from a pinned snapshot of the old ledger.',
    tags: ['billing', 'ledger', 'migration', 'schema', 'coverage'],
    extraBee: {
      areas: ['billing'],
      required_context: ['areas/billing/ledger-schema.md', 'areas/billing/invoice-rows.md', 'areas/billing/rollback-runbook.md'],
      decisions: [],
    },
    body: [
      'Every ledger row is migrated into the invoice schema, one migration cell per ledger',
      'table, and the coverage gate derives its ground truth by parsing a pinned snapshot of',
      'the old ledger rather than comparing two hand-authored row inventories. A rollback',
      'restores the pinned snapshot and re-derives the invoice totals. Enumerating rows by',
      'hand drifts from the ledger the moment a row is added, so the enumeration is derived.',
      'Each migration cell runs its own check and the worker records the row counts.',
    ].join('\n'),
  }));
  // Three required_context areas, each far larger than a pattern: under a plain
  // prefix cut these consume the whole budget and every critical is evicted.
  // They are what the floor has to beat.
  for (const [name, title] of [['ledger-schema', 'The ledger schema'], ['invoice-rows', 'Invoice rows'], ['rollback-runbook', 'Rollback runbook']]) {
    writeBundleFile(root, `areas/billing/${name}.md`, conceptText({
      type: 'bee.area', id: name, title, description: `${title} reference`,
      tags: ['billing'], extraBee: { areas: ['billing'], authoritative_for: name },
      body: `${title} reference material. `.repeat(60),
    }));
  }
  const rel = (name, id, title, description, body) => writeBundleFile(root, `patterns/${name}.md`, conceptText({
    id, title, description, body, tags: ['pattern'], extraBee: { areas: ['billing'], critical: true },
  }));
  rel('rel-ledger-rows', 'rel-ledger-rows',
    'A ledger migration verifies every row against the pinned snapshot',
    'Row coverage in a ledger migration is measured against the pinned snapshot, never against a hand inventory.',
    'A ledger migration that counts the rows it wrote proves ownership, not coverage. Derive\nthe row inventory from the pinned snapshot of the old ledger and compare the invoice\nschema totals row by row. A migration cell that cannot parse a ledger row must say so.');
  rel('rel-coverage-gate', 'rel-coverage-gate',
    'A coverage gate over a schema migration derives its ground truth',
    'A gate comparing two hand-authored inventories proves internal consistency, not coverage of the ledger.',
    'The coverage gate for the invoice schema derives its ground truth by parsing the pinned\nledger snapshot. A gate that compares a hand-authored row list against hand-authored\nschema claims drifts green while ledger rows go missing from the migration.');
  rel('rel-schema-rollback', 'rel-schema-rollback',
    'A billing schema rollback re-derives the invoice totals it restores',
    'Restoring the pinned ledger snapshot is not a rollback until the invoice totals are re-derived.',
    'A rollback that restores the pinned snapshot of the billing ledger but leaves the derived\ninvoice totals in the new schema leaves every row internally inconsistent. Re-derive the\ntotals from the restored ledger rows as part of the rollback, inside the same migration cell.');
  rel('rel-enumerated-rows', 'rel-enumerated-rows',
    'An enumerated row list in a migration cell drifts from the ledger',
    'A migration cell that hand-enumerates ledger rows rots the moment a row is added.',
    'Hand-enumerating the ledger rows a migration cell will move turns the cell into a second\ninventory that must be maintained. Derive the row enumeration from the ledger schema at\nrun time so a new invoice row is migrated by the same coverage gate as every other row.');
  const irr = (name, id, title, description, body) => writeBundleFile(root, `patterns/${name}.md`, conceptText({
    id, title, description, body, tags: ['pattern'], extraBee: { areas: ['tooling'], critical: true },
  }));
  irr('irr-powershell-bom', 'irr-powershell-bom',
    'Non-ASCII in a .ps1 without a BOM is a parse-time bomb on PowerShell 5.1',
    'Windows PowerShell 5.1 decodes a BOM-less script as the ANSI codepage and dies at parse time.',
    'PowerShell 5.1 assumes the ANSI codepage for a BOM-less .ps1, so an em dash becomes a\nsyntax error before the first statement executes. Emit UTF-8 with a BOM, and check the\nencoding in the installer suite.');
  irr('irr-scratchpad-polling', 'irr-scratchpad-polling',
    'Never poll scratchpad files while waiting for a background subagent',
    'Polling a scratchpad burns turns and reads half-written files; wait on the harness instead.',
    'Sleeping in a loop over a scratchpad path costs turns and can read a half-flushed file.\nThe harness already blocks until the subagent returns, so wait on it and check the\nreturned digest instead of the filesystem.');
  irr('irr-viewport-screenshot', 'irr-viewport-screenshot',
    'A screenshot harness must pin the viewport dimensions',
    'An unpinned viewport makes every visual diff a false positive on a different display.',
    'Screenshots taken at whatever viewport the display happens to offer diff against each\nother forever. Pin the width, the height and the device pixel ratio in the harness, and\ncheck the pinned numbers into the fixture.');
  irr('irr-spawn-heartbeat', 'irr-spawn-heartbeat',
    'A lock held across a synchronous child spawn cannot heartbeat',
    'The event loop is blocked for the whole spawn, so the lock mtime cannot be renewed.',
    'A synchronous child spawn blocks the event loop, so nothing renews the lock mtime while\nthe child runs. Stale takeover must probe whether the owning process is alive instead of\ntrusting an mtime, or a long build loses its lock to a peer.');
  irr('irr-emoji-columns', 'irr-emoji-columns',
    'Emoji in a terminal renderer break column alignment',
    'A double-width glyph counts as one code point and two cells, so padding maths drift.',
    'Padding a table by string length puts a double-width emoji one cell out on every row\nbelow it. Measure display width, not code points, or drop the glyph from the renderer and\ncheck the alignment on a narrow terminal.');
  irr('irr-dns-cache', 'irr-dns-cache',
    'A DNS cache warmed at boot hides a resolver outage',
    'A long TTL keeps a dead resolver invisible until the first cold lookup, hours later.',
    'A resolver that died after the cache warmed stays invisible while the TTL holds, then\nevery cold lookup fails at once. Probe the resolver directly on an interval and check the\nprobe result, never the cached answer.');
  return root;
}

await check('knowledge context DISCRIMINATES: every labelled-relevant critical outranks every labelled-irrelevant one, with NON-TIED scores (G10)', async () => {
  const root = makeDiscriminationFixture();
  const result = await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', '100000', '--json'], root);
  assert(result.status === 0, `context must exit 0, got ${result.status}: stdout=${result.stdout} stderr=${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  // Every critical carries a score, wherever it landed: entries name it in the
  // reason, excluded name it in the record.
  const scores = new Map();
  for (const entry of manifest.entries) {
    const m = /relevance ([0-9.]+)/.exec(entry.reason);
    if (m) scores.set(entry.path, Number(m[1]));
  }
  for (const ex of manifest.excluded) scores.set(ex.path, ex.score);
  for (const p of [...DISCRIMINATION_RELEVANT, ...DISCRIMINATION_IRRELEVANT]) {
    assert(scores.has(p), `every critical must carry a relevance score somewhere in the payload; missing ${p} (scored: ${JSON.stringify([...scores.keys()])})`);
    assert(typeof scores.get(p) === 'number' && Number.isFinite(scores.get(p)), `score must be a finite number, got ${JSON.stringify(scores.get(p))} for ${p}`);
  }
  const relScores = DISCRIMINATION_RELEVANT.map((p) => scores.get(p));
  const irrScores = DISCRIMINATION_IRRELEVANT.map((p) => scores.get(p));
  const minRel = Math.min(...relScores);
  const maxIrr = Math.max(...irrScores);
  assert(minRel > maxIrr, `DISCRIMINATION FAILED: worst relevant ${minRel} must outrank best irrelevant ${maxIrr}.\nrelevant=${JSON.stringify(relScores)}\nirrelevant=${JSON.stringify(irrScores)}`);
  const all = [...relScores, ...irrScores];
  assert(new Set(all).size === all.length, `scores must be NON-TIED across the labelled set, got ${JSON.stringify(all)}`);
  assert(!all.some((s) => s === 0), `no labelled pattern may score zero on a corpus this related, got ${JSON.stringify(all)}`);
  assert(manifest.zero_signal_count === 0, `zero_signal_count must be 0 on this fixture, got ${JSON.stringify(manifest.zero_signal_count)}`);
});

await check('knowledge context: the relevance CUT keeps the top-scoring criticals and NAMES every excluded one with score and reason (G11)', async () => {
  const root = makeDiscriminationFixture();
  const manifest = JSON.parse((await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', '100000', '--json'], root)).stdout);
  assert(Array.isArray(manifest.excluded), `the payload must carry an excluded array, got ${JSON.stringify(manifest.excluded)}`);
  for (const ex of manifest.excluded) {
    assert(JSON.stringify(Object.keys(ex).sort()) === JSON.stringify(['path', 'reason', 'score']), `every exclusion is {path, score, reason}, got ${JSON.stringify(ex)}`);
    assert(typeof ex.reason === 'string' && ex.reason.trim() && !ex.reason.includes('\n'), `every exclusion needs a one-line reason, got ${JSON.stringify(ex)}`);
    assert(/rank \d+ of \d+/.test(ex.reason), `an exclusion reason must name the rank it lost at, got ${JSON.stringify(ex.reason)}`);
  }
  // 10 criticals, KEEP is larger than 10, so nothing is cut for relevance here;
  // the cut is exercised by the tightened-keep fixture below.
  const kept = manifest.entries.filter((e) => /critical pattern/.test(e.reason)).map((e) => e.path);
  const relRanks = DISCRIMINATION_RELEVANT.map((p) => kept.indexOf(p));
  assert(relRanks.every((r) => r >= 0 && r < 4), `the four labelled-relevant criticals must occupy the top four ranks of the critical block, got ${JSON.stringify(kept)}`);
});

await check('knowledge context CONSERVES the critical set: entries + truncated + excluded == every bee.critical concept, no duplicates (G11)', async () => {
  const root = makeDiscriminationFixture();
  const all = [...DISCRIMINATION_RELEVANT, ...DISCRIMINATION_IRRELEVANT];
  for (const budget of [100000, 2500, 900, 0]) {
    const manifest = JSON.parse((await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', String(budget), '--json'], root)).stdout);
    const accounted = [
      ...manifest.entries.map((e) => e.path),
      ...manifest.truncated,
      ...manifest.excluded.map((e) => e.path),
    ].filter((p) => all.includes(p));
    assert(new Set(accounted).size === accounted.length, `budget ${budget}: a critical may be accounted for exactly ONCE, got ${JSON.stringify(accounted)}`);
    assert(new Set(accounted).size === all.length, `budget ${budget}: CONSERVATION FAILED — ${all.length} criticals exist, ${new Set(accounted).size} accounted for.\nmissing: ${JSON.stringify(all.filter((p) => !accounted.includes(p)))}`);
    assert(manifest.critical_total === all.length, `budget ${budget}: critical_total must state the full population, got ${JSON.stringify(manifest.critical_total)}`);
  }
});

await check('knowledge context FLOOR: the highest-scoring critical survives a budget that the plain prefix cut would have evicted it under', async () => {
  const root = makeDiscriminationFixture();
  const full = JSON.parse((await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', '100000', '--json'], root)).stdout);
  const workEntry = full.entries[0];
  assert(workEntry.path === 'docs/knowledge/work/billing/work-item.md', `rank 1 must be the work item, got ${workEntry.path}`);
  const floor = full.floor;
  assert(Array.isArray(floor) && floor.length > 0, `the payload must NAME the floor it guarantees, got ${JSON.stringify(floor)}`);
  const topCritical = full.entries.find((e) => /critical pattern/.test(e.reason));
  assert(floor.length === CRITICAL_RELEVANCE.FLOOR, `the floor is the pinned ${CRITICAL_RELEVANCE.FLOOR}, got ${JSON.stringify(floor)}`);
  assert(floor.includes(topCritical.path), `the highest-scoring critical must be in the floor, got floor=${JSON.stringify(floor)} top=${topCritical.path}`);
  const floorCost = full.entries.filter((e) => floor.includes(e.path)).reduce((s, e) => s + e.est_tokens, 0);
  // Exactly the work item plus the floor. Under a plain prefix cut this budget
  // buys the work item and part of the required_context chain, and every
  // critical is evicted — which is the failure the floor exists to stop.
  const tight = workEntry.est_tokens + floorCost;
  const reqCtxCost = full.entries.filter((e) => /required_context/.test(e.reason)).reduce((s, e) => s + e.est_tokens, 0);
  assert(reqCtxCost > floorCost, `the fixture must make the required_context chain the thing that would evict the floor (${reqCtxCost} vs ${floorCost})`);
  const cut = JSON.parse((await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', String(tight), '--json'], root)).stdout);
  assert(cut.total_est <= tight, `the budget stays a hard ceiling even with a floor: total_est ${cut.total_est} <= ${tight}`);
  for (const p of floor) assert(cut.entries.some((e) => e.path === p), `every floor critical must survive a tight budget, ${p} was evicted from ${JSON.stringify(cut.entries.map((e) => e.path))}`);
  assert(cut.entries[0].path === workEntry.path, `the work item is never displaced by its own floor, got ${JSON.stringify(cut.entries.map((e) => e.path))}`);
  assert(cut.entries.length === 1 + floor.length, `under this budget exactly the work item and the floor survive, got ${JSON.stringify(cut.entries.map((e) => e.path))}`);
  assert(cut.truncated.some((p) => /areas\/billing\//.test(p)), `the floor must beat the higher-ranked required_context chain, got truncated=${JSON.stringify(cut.truncated)}`);
});

await check('knowledge context: relevance ties break DETERMINISTICALLY by path, and repeat runs are byte-identical', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'work/twins/work-item.md', conceptText({
    type: 'bee.work-item', id: 'twins', title: 'Twin ranking', description: 'Two criticals with identical vocabulary must not flap',
    tags: ['twin'], extraBee: { areas: ['twin'], required_context: [], decisions: [] },
    body: 'Identical vocabulary twin ranking flap determinism.',
  }));
  const twinBody = 'Identical vocabulary twin ranking flap determinism, word for word.';
  for (const name of ['zulu-twin', 'alpha-twin']) {
    writeBundleFile(root, `patterns/${name}.md`, conceptText({
      id: name, title: 'Twin pattern', description: 'Identical vocabulary twin', body: twinBody,
      tags: ['twin'], extraBee: { areas: ['twin'], critical: true },
    }));
  }
  const first = (await runBee(['knowledge', 'context', '--work', 'twins', '--budget', '100000', '--json'], root)).stdout;
  const second = (await runBee(['knowledge', 'context', '--work', 'twins', '--budget', '100000', '--json'], root)).stdout;
  assert(first === second, 'two runs over the same bundle must be byte-identical');
  const manifest = JSON.parse(first);
  const criticals = manifest.entries.filter((e) => /critical pattern/.test(e.reason)).map((e) => e.path);
  assert(JSON.stringify(criticals) === JSON.stringify(['docs/knowledge/patterns/alpha-twin.md', 'docs/knowledge/patterns/zulu-twin.md']), `tied scores must order by path, got ${JSON.stringify(criticals)}`);
});

await check('knowledge context FAILS when zero_signal_count exceeds the pinned threshold — a ranking where most items score zero is not a ranking (G11)', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'work/lonely/work-item.md', conceptText({
    type: 'bee.work-item', id: 'signalless', title: 'Reconcile quarterly payroll withholding',
    description: 'Withholding reconciliation across payroll periods.',
    tags: ['payroll'], extraBee: { areas: ['payroll'], required_context: [], decisions: [] },
    body: 'Payroll withholding reconciliation across quarterly periods, employer contributions included.',
  }));
  const topics = ['kubernetes ingress', 'sourdough hydration', 'telescope collimation', 'bicycle derailleur',
    'harpsichord tuning', 'glacier moraine', 'origami tessellation', 'submarine ballast',
    'volcanic tephra', 'lighthouse fresnel', 'saffron cultivation', 'permafrost drilling'];
  topics.forEach((topic, i) => {
    writeBundleFile(root, `patterns/void-${String(i).padStart(2, '0')}.md`, conceptText({
      id: `void-${i}`, title: `${topic} guidance`, description: `${topic} guidance notes`,
      body: `${topic} guidance notes, ${topic} technique, ${topic} maintenance.`,
      tags: ['unrelated'], extraBee: { areas: ['unrelated'], critical: true },
    }));
  });
  const result = await runBee(['knowledge', 'context', '--work', 'signalless', '--budget', '100000', '--json'], root);
  assert(result.status === 1, `an all-zero ranking must FAIL the run, got status ${result.status}: ${result.stdout}`);
  const payload = JSON.parse(result.stdout);
  assert(/zero_signal/.test(payload.error), `the failure must carry the typed zero_signal code, got ${result.stdout}`);
  assert(/\b12\b/.test(payload.error), `the failure must name the zero count it measured, got ${result.stdout}`);
  const human = await runBee(['knowledge', 'context', '--work', 'signalless', '--budget', '100000'], root);
  assert(human.status === 1 && /zero_signal/.test(human.stderr), `the human form must fail on stderr with the typed code, got status=${human.status} stderr=${human.stderr}`);
});

await check('knowledge context: the zero-signal guard is inert below the pinned population floor — a two-pattern bundle is not a ranking problem', async () => {
  const root = makeRepo();
  writeBundleFile(root, 'work/tiny/work-item.md', conceptText({
    type: 'bee.work-item', id: 'tiny-work', title: 'Reconcile quarterly payroll withholding',
    description: 'Withholding reconciliation across payroll periods.',
    tags: ['payroll'], extraBee: { areas: ['payroll'], required_context: [], decisions: [] },
    body: 'Payroll withholding reconciliation across quarterly periods.',
  }));
  writeBundleFile(root, 'patterns/unrelated.md', conceptText({
    id: 'unrelated-one', title: 'Kubernetes ingress guidance', description: 'Ingress guidance notes',
    body: 'Kubernetes ingress guidance notes and technique.', tags: ['unrelated'],
    extraBee: { areas: ['unrelated'], critical: true },
  }));
  const result = await runBee(['knowledge', 'context', '--work', 'tiny-work', '--budget', '100000', '--json'], root);
  assert(result.status === 0, `a one-critical bundle must still resolve, got ${result.status}: ${result.stdout} ${result.stderr}`);
  const manifest = JSON.parse(result.stdout);
  assert(manifest.zero_signal_count === 1, `the count is still REPORTED below the floor, got ${JSON.stringify(manifest.zero_signal_count)}`);
});

await check('knowledge context: the relevance cut is a real cut — a bundle above the KEEP ceiling excludes the tail, and the excluded are the lowest-scoring', async () => {
  const root = makeDiscriminationFixture();
  // Push the population above CRITICAL_RELEVANCE.KEEP with further disjoint
  // criticals; the four labelled-relevant ones must still be kept.
  const filler = ['kubernetes ingress', 'sourdough hydration', 'telescope collimation', 'bicycle derailleur',
    'harpsichord tuning', 'glacier moraine', 'origami tessellation', 'submarine ballast',
    'volcanic tephra', 'lighthouse fresnel', 'saffron cultivation', 'permafrost drilling',
    'kiln firing', 'estuary silt', 'monsoon onset', 'quarry blasting'];
  filler.forEach((topic, i) => {
    writeBundleFile(root, `patterns/fill-${String(i).padStart(2, '0')}.md`, conceptText({
      id: `fill-${i}`, title: `${topic} guidance`, description: `${topic} guidance notes for the check`,
      body: `${topic} guidance notes, ${topic} technique, and one check the worker runs.`,
      tags: ['unrelated'], extraBee: { areas: ['unrelated'], critical: true },
    }));
  });
  const manifest = JSON.parse((await runBee(['knowledge', 'context', '--work', 'billing-migration', '--budget', '1000000', '--json'], root)).stdout);
  assert(manifest.critical_total === 26, `26 criticals exist in this fixture, got ${manifest.critical_total}`);
  assert(manifest.excluded.length === 26 - CRITICAL_RELEVANCE.KEEP, `the cut must exclude everything past KEEP=${CRITICAL_RELEVANCE.KEEP}, got ${manifest.excluded.length} excluded`);
  const keptPaths = new Set(manifest.entries.filter((e) => /critical pattern/.test(e.reason)).map((e) => e.path));
  for (const p of DISCRIMINATION_RELEVANT) assert(keptPaths.has(p), `a labelled-relevant critical must survive the cut, ${p} was excluded`);
  const excludedScores = manifest.excluded.map((e) => e.score);
  const keptScores = manifest.entries.filter((e) => /relevance/.test(e.reason)).map((e) => Number(/relevance ([0-9.]+)/.exec(e.reason)[1]));
  assert(Math.min(...keptScores) >= Math.max(...excludedScores), `every kept critical must score at or above every excluded one, kept min ${Math.min(...keptScores)} vs excluded max ${Math.max(...excludedScores)}`);
});

// ─── summary ────────────────────────────────────────────────────────────────

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
