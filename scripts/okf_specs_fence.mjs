#!/usr/bin/env node
// okf_specs_fence.mjs — G2 (okf-switchover-f3, cell f3-4): `docs/specs/` is
// READ-ONLY for NEW content, enforced mechanically.
//
// WHAT IT ENFORCES: once a repo has a knowledge bundle, a file under
// `docs/specs/` may only be one of
//   (i)   a POINTER STUB — recognised STRUCTURALLY, by `migrated_to` in its
//         frontmatter (the field the migration loop already writes), never by
//         a filename list. A filename list rots the first time an area is
//         added or renamed, and a rotted allowlist stops fencing SILENTLY —
//         the exact failure class this whole programme exists to prevent.
//   (ii)  `reading-map.md` — the hand-written navigation surface (G4 keeps it,
//         pointing at the bundle).
//   (iii) a NAMED, REASONED, PINNED placeholder — see PLACEHOLDERS below.
//
// `okf-profile.md` used to be a third named exception here, protecting the
// interval between this fence (f3-4) and G6's migration of the profile into the
// bundle it describes (f3-5). That interval is over: the profile is migrated and
// the file is now an ordinary structural stub, classified `stub` through branch
// (i) like every other migrated area. The exception was REMOVED rather than
// relabelled, because leaving it would have been a hole — a name-based pass that
// keeps saying yes if the stub's `migrated_to` is ever dropped or its
// frontmatter breaks, exactly the silent-rot failure branch (i) exists to
// prevent. Asserted both ways in `--selftest`: the real stub passes as `stub`,
// and an `okf-profile.md` WITHOUT `migrated_to` fails as new content.
// Anything else is NEW CONTENT landing in a read-only compatibility surface,
// and the chain fails naming the file.
//
// WHY IT IS GATED ON `bundleMode` (G1): bee ships to other repos. A host that
// never migrated must keep writing `docs/specs/` freely and must not be able
// to tell this release happened. No bundle -> the check does not scan at all
// and reports `inert: true`. (`scripts/` is not shipped to hosts either, so
// this is belt AND braces — but the predicate is the contract, not the
// shipping accident.)
//
// RED-FIRST (cell f3-4): every assertion in `--selftest` below was written and
// run against a `fenceFindings` that classified nothing, BEFORE this
// implementation existed. Captured verbatim in the cell trace.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// The canonical lib is the TEMPLATE source, not the vendored `.bee/bin/lib`
// copy (`scripts/ledger_parity.mjs --check` keeps the two identical; the
// template is the one under edit). `bundleMode` is f3-2/f3-3's ONE predicate —
// the fence never re-implements "is this repo migrated", or the two answers
// could drift apart.
import { bundleMode, parseFrontmatter, emitFrontmatter } from '../packages/bee/lib/knowledge.mjs';
import { resolveProductRoot } from '../packages/bee/lib/state.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');

/**
 * `docs/specs/` is a PRODUCT doc tree, exactly like `docs/knowledge/` — so it
 * resolves through `resolveProductRoot` (G13, cell f3-3). Identical to `root`
 * in every ordinary single-root repo; it matters only in the repo-divorce
 * topology, where the workshop's own empty `docs/specs/` must not be the tree
 * that gets graded.
 */
export function specsDir(root) {
  return path.join(resolveProductRoot(root), 'docs', 'specs');
}

// ─── the named, reasoned exceptions (NOT the stub rule) ────────────────────
//
// This name IS hardcoded, deliberately and narrowly: G2 names it. The
// prohibition is on recognising STUBS by filename — that set grows and renames
// with every area, so it must be structural. This set is closed by decision and
// each member carries its reason in the code, printed by --json.
//
// It SHRANK by one in f3-5: `okf-profile.md` left it the moment G6 migrated the
// profile and gave the file a `migrated_to` of its own. A named exception that
// outlives its interval is a standing permission for that filename to hold
// anything at all.

const NAMED_EXCEPTIONS = new Map([
  [
    'reading-map.md',
    ['navigation', 'the hand-written navigation surface — "where does X live" (G2/G4); it points AT the bundle, it is not area truth'],
  ],
]);

// system-overview.md is a DECISION, not an oversight (cell f3-4).
//
// It is a 7-line unwritten placeholder: it holds no content to migrate, so
// there is nothing for the migration loop to move and no stub to write. It is
// therefore allowlisted BY NAME with the reason below — and PINNED to the
// placeholder state, so the allowlist can never quietly become the hole
// through which the system overview gets authored as prose in the very tree
// this fence just made read-only. The moment the "not written yet" marker
// goes — i.e. the moment somebody actually writes it — the fence fires and
// says where it belongs: an `overview` concept in the bundle.
const PLACEHOLDERS = new Map([
  [
    'system-overview.md',
    {
      marker: /\(not written yet/i,
      reason:
        'a KNOWN UNWRITTEN PLACEHOLDER (7 lines, no content to migrate, so no stub exists to point anywhere) — allowlisted only while it stays unwritten; writing it here fails this fence, because a written system overview belongs in the bundle',
      written:
        'system-overview.md was ALLOWLISTED only as an unwritten placeholder and now carries real prose — author it as a concept in docs/knowledge/ instead (G2/G3)',
    },
  ],
]);

const NEW_CONTENT_REASON =
  'new content under a read-only compatibility surface — write it into the knowledge bundle instead (docs/knowledge/areas/<area>/, G2/G3)';

function listSpecsFiles(dir) {
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
      else if (entry.isFile()) out.push(childRel);
    }
  };
  walk(dir, '');
  return out.sort();
}

/**
 * Classify ONE file under docs/specs/. `base` is its path relative to
 * docs/specs/. Returns { verdict, reason }; every verdict except the three
 * failing ones is an allowed entry.
 */
export function classifySpecsFile({ base, text, bundleRoot }) {
  // (i) STRUCTURAL stub recognition — the only rule that scales with areas.
  let parsed = null;
  try {
    parsed = parseFrontmatter(text);
  } catch {
    parsed = null;
  }
  const migratedTo = parsed && parsed.ok && parsed.present ? parsed.data?.migrated_to : undefined;
  if (typeof migratedTo === 'string' && migratedTo.trim()) {
    const target = path.join(bundleRoot, migratedTo.trim());
    if (!fs.existsSync(target)) {
      return {
        verdict: 'dangling-stub',
        reason: `pointer stub whose migrated_to target does not exist: ${migratedTo.trim()} — a stub that resolves nowhere is a silently broken doc (G7)`,
      };
    }
    return { verdict: 'stub', reason: `pointer stub — frontmatter migrated_to: ${migratedTo.trim()}` };
  }

  // (ii)/(iii) the two named exceptions G2 states.
  if (NAMED_EXCEPTIONS.has(base)) {
    const [verdict, reason] = NAMED_EXCEPTIONS.get(base);
    return { verdict, reason };
  }

  // (iv) the pinned placeholder.
  if (PLACEHOLDERS.has(base)) {
    const pin = PLACEHOLDERS.get(base);
    if (pin.marker.test(text)) return { verdict: 'placeholder', reason: pin.reason };
    return { verdict: 'placeholder-written', reason: pin.written };
  }

  return { verdict: 'new-content', reason: NEW_CONTENT_REASON };
}

const FAILING_VERDICTS = new Set(['new-content', 'dangling-stub', 'placeholder-written']);

/**
 * Scan a repo. Returns { inert, root, entries, findings }.
 *   inert    — true when this repo has no knowledge bundle: NOT scanned at
 *              all, never a finding, never a word (G1).
 *   entries  — every file under docs/specs/, with its verdict and the reason
 *              for it.
 *   findings — the subset whose verdict fails the chain.
 */
export function fenceFindings(root) {
  if (!bundleMode(root)) return { inert: true, root, entries: [], findings: [] };

  const productRoot = resolveProductRoot(root);
  const dir = specsDir(root);
  const entries = [];
  for (const base of listSpecsFiles(dir)) {
    let text = '';
    try {
      text = fs.readFileSync(path.join(dir, base), 'utf8');
    } catch {
      text = '';
    }
    const { verdict, reason } = classifySpecsFile({ base, text, bundleRoot: productRoot });
    entries.push({ rel: `docs/specs/${base}`, verdict, reason });
  }

  return {
    inert: false,
    root,
    entries,
    findings: entries.filter((entry) => FAILING_VERDICTS.has(entry.verdict)),
  };
}

// ─── selftest fixtures ─────────────────────────────────────────────────────

function makeRepo(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `bee-specs-fence-${label}-`));
}

function writeFile(root, rel, text) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

function conceptText(id) {
  const data = {
    type: 'bee.area',
    title: 'Demo area — purpose',
    description: 'A canonical fixture concept.',
    tags: ['demo'],
    timestamp: '2026-07-22',
    bee: { id, lifecycle: 'active', areas: ['demo-area'] },
  };
  return `${emitFrontmatter(data)}\n# Demo area\n\nBody.\n`;
}

function stubText(area) {
  return [
    '---',
    `area: ${area}`,
    'updated: 2026-07-22',
    `migrated_to: docs/knowledge/areas/${area}/`,
    '---',
    '',
    `# ${area} (migrated — pointer stub)`,
    '',
    'Anchors resolve through the bundle.',
    '',
  ].join('\n');
}

const NEW_PROSE = '# Billing\n\n## Purpose\n\nCharges customers.\n';

/** A host repo that never migrated: docs/specs/ only, and NEW prose in it. */
function makeBundlelessRepo() {
  const root = makeRepo('bundleless');
  writeFile(root, 'docs/specs/reading-map.md', '# Reading map\n');
  writeFile(root, 'docs/specs/billing.md', NEW_PROSE);
  return root;
}

/** A migrated repo, with the SAME new prose file sitting under docs/specs/. */
function makeBundlefulRepo(label = 'bundleful') {
  const root = makeRepo(label);
  writeFile(root, 'docs/knowledge/areas/demo-area/overview.md', conceptText('demo-area-overview'));
  writeFile(root, 'docs/specs/reading-map.md', '# Reading map\n');
  writeFile(root, 'docs/knowledge/areas/billing/index.md', '# billing\n');
  return root;
}

function findingPaths(result) {
  return (result.findings || []).map((f) => f.rel).sort();
}

function verdictOf(result, rel) {
  const hit = (result.entries || []).find((e) => e.rel === rel);
  return hit ? hit.verdict : null;
}

// ─── selftest ──────────────────────────────────────────────────────────────

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

  // (A) G1 — the compatibility guarantee. A repo with no bundle keeps writing
  // docs/specs/ freely. This case must be silent BEFORE and AFTER the fence
  // exists; it is the one assertion the fence may never change.
  check('bundle-LESS repo with NEW prose under docs/specs/ is SILENT (G1)', () => {
    const root = makeBundlelessRepo();
    const result = fenceFindings(root);
    assert(result.findings.length === 0, `a host repo with no bundle is never fenced, got ${JSON.stringify(findingPaths(result))}`);
    assert(result.inert === true, 'the result says so explicitly: inert');
  });

  // (B) G2 — the fence itself.
  check('bundle-ful repo: a NEW non-stub prose file under docs/specs/ FAILS, naming the file', () => {
    const root = makeBundlefulRepo('newprose');
    writeFile(root, 'docs/specs/billing.md', NEW_PROSE);
    const result = fenceFindings(root);
    assert(result.inert === false, 'a migrated repo is fenced');
    assert(findingPaths(result).join(',') === 'docs/specs/billing.md', `the offending file is named, got ${JSON.stringify(findingPaths(result))}`);
  });

  // (C) stub recognition is STRUCTURAL, never a filename list.
  check('a file with NO real-spec name passes purely because its frontmatter carries migrated_to', () => {
    const root = makeBundlefulRepo('structural-pass');
    writeFile(root, 'docs/knowledge/areas/zzz-never-shipped/index.md', '# zzz\n');
    writeFile(root, 'docs/specs/zzz-never-shipped.md', stubText('zzz-never-shipped'));
    const result = fenceFindings(root);
    assert(findingPaths(result).length === 0, `an unknown-but-structural stub passes, got ${JSON.stringify(findingPaths(result))}`);
    assert(verdictOf(result, 'docs/specs/zzz-never-shipped.md') === 'stub', 'classified as a stub by structure');
  });

  check('a file NAMED exactly like a shipped stub but WITHOUT migrated_to still FAILS', () => {
    const root = makeBundlefulRepo('structural-fail');
    writeFile(root, 'docs/specs/hook-runtime.md', '# Hook runtime\n\n## Purpose\n\nBrand new prose.\n');
    const result = fenceFindings(root);
    assert(findingPaths(result).join(',') === 'docs/specs/hook-runtime.md', `a familiar filename buys nothing, got ${JSON.stringify(findingPaths(result))}`);
  });

  check('a stub whose migrated_to points nowhere FAILS as a dangling pointer', () => {
    const root = makeBundlefulRepo('dangling');
    writeFile(root, 'docs/specs/ghost.md', stubText('ghost'));
    const result = fenceFindings(root);
    assert(findingPaths(result).join(',') === 'docs/specs/ghost.md', `a stub pointing at nothing is a broken doc, got ${JSON.stringify(findingPaths(result))}`);
    assert(verdictOf(result, 'docs/specs/ghost.md') === 'dangling-stub', 'reported as its own class, not as new content');
  });

  // (D) the named exception G2 states, with its reason in the code.
  check('reading-map.md is the named exception, and it passes', () => {
    const root = makeBundlefulRepo('exceptions');
    const result = fenceFindings(root);
    assert(findingPaths(result).length === 0, `got ${JSON.stringify(findingPaths(result))}`);
    assert(verdictOf(result, 'docs/specs/reading-map.md') === 'navigation', 'reading-map is the navigation surface');
  });

  // (D2) f3-5/G6 — the profile moved INTO the bundle it describes, so its
  // filename bought it a pass that must now be gone. Both directions are
  // asserted, because relabelling the exception instead of removing it would
  // have left exactly the hole branch (i) exists to close.
  check('okf-profile.md passes as an ordinary STRUCTURAL stub once it carries migrated_to (G6)', () => {
    const root = makeBundlefulRepo('profile-stub');
    writeFile(root, 'docs/knowledge/areas/okf-profile/index.md', '# okf-profile\n');
    writeFile(root, 'docs/specs/okf-profile.md', stubText('okf-profile'));
    const result = fenceFindings(root);
    assert(findingPaths(result).length === 0, `got ${JSON.stringify(findingPaths(result))}`);
    assert(verdictOf(result, 'docs/specs/okf-profile.md') === 'stub', 'classified by structure, not by name');
  });

  check('okf-profile.md WITHOUT migrated_to FAILS — its named exception is gone, not renamed', () => {
    const root = makeBundlefulRepo('profile-nostub');
    writeFile(root, 'docs/specs/okf-profile.md', '---\narea: okf-profile\n---\n\n# OKF profile\n\n## Purpose\n\nBrand new prose.\n');
    const result = fenceFindings(root);
    assert(
      findingPaths(result).join(',') === 'docs/specs/okf-profile.md',
      `a stub without migrated_to must never pass on its filename, got ${JSON.stringify(findingPaths(result))}`,
    );
    assert(verdictOf(result, 'docs/specs/okf-profile.md') === 'new-content', 'reported as new content, not as a profile');
  });

  // (E) system-overview.md — allowlisted as a KNOWN UNWRITTEN PLACEHOLDER, and
  // pinned to that state so the allowlist can never become a hole.
  check('system-overview.md passes ONLY while it is still the unwritten placeholder', () => {
    const root = makeBundlefulRepo('placeholder');
    writeFile(root, 'docs/specs/system-overview.md', '# System Overview\n\n(not written yet — run a bee-scribing bootstrap pass to fill this in)\n');
    const ok = fenceFindings(root);
    assert(findingPaths(ok).length === 0, `the placeholder passes, got ${JSON.stringify(findingPaths(ok))}`);
    assert(verdictOf(ok, 'docs/specs/system-overview.md') === 'placeholder', 'classified as an allowlisted placeholder');

    writeFile(root, 'docs/specs/system-overview.md', '# System Overview\n\nbee is a workflow harness. It has areas, cells and gates...\n');
    const written = fenceFindings(root);
    assert(
      findingPaths(written).join(',') === 'docs/specs/system-overview.md',
      `the moment it is actually WRITTEN the allowlist stops applying, got ${JSON.stringify(findingPaths(written))}`,
    );
  });

  // (F) a directory is not a bundle (the .gitkeep rot case, f3-2).
  check('a docs/knowledge/ holding only a .gitkeep does NOT arm the fence', () => {
    const root = makeRepo('gitkeep');
    writeFile(root, 'docs/knowledge/.gitkeep', '');
    writeFile(root, 'docs/specs/billing.md', NEW_PROSE);
    const result = fenceFindings(root);
    assert(result.inert === true, 'a directory is not a bundle');
    assert(result.findings.length === 0, `no findings, got ${JSON.stringify(findingPaths(result))}`);
  });

  // (G) G13 — the divorced topology reads the PRODUCT docs/specs/.
  check('divorced topology (product_root): the fence scans the PRODUCT docs/specs/, not the workshop one', () => {
    const root = makeRepo('divorced');
    writeFile(root, '.bee/config.json', `${JSON.stringify({ product_root: 'product' }, null, 2)}\n`);
    fs.mkdirSync(path.join(root, 'docs', 'specs'), { recursive: true });
    writeFile(root, 'docs/specs/workshop-only.md', NEW_PROSE); // must be invisible
    writeFile(root, 'product/docs/knowledge/areas/demo-area/overview.md', conceptText('demo-area-overview'));
    writeFile(root, 'product/docs/specs/billing.md', NEW_PROSE);
    const result = fenceFindings(root);
    assert(result.inert === false, 'a migrated product one directory down IS a bundle');
    assert(findingPaths(result).join(',') === 'docs/specs/billing.md', `product-root-relative, got ${JSON.stringify(findingPaths(result))}`);
  });

  // (H) the live repo — every entry under docs/specs/ has a verdict, and the
  // fence is silent on all of them.
  check("bee's own docs/specs/ is fully classified and produces ZERO findings", () => {
    const result = fenceFindings(REPO_ROOT);
    assert(result.inert === false, "bee's own checkout has a bundle");
    assert(result.entries.length > 0, 'the live tree is enumerated');
    assert(
      result.findings.length === 0,
      `the live tree must be silent, got ${JSON.stringify(findingPaths(result))}`,
    );
    for (const entry of result.entries) {
      assert(typeof entry.reason === 'string' && entry.reason.length > 0, `${entry.rel} carries a stated reason`);
    }
  });

  console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} okf_specs_fence --selftest: ${passed} passed, ${failed} failed`);
  return failed === 0 ? 0 : 1;
}

// ─── CLI ───────────────────────────────────────────────────────────────────

function main(argv) {
  const args = argv.slice(2);
  if (args.includes('--selftest')) return runSelftest();

  const rootIdx = args.indexOf('--root');
  const root = rootIdx === -1 ? REPO_ROOT : path.resolve(args[rootIdx + 1]);
  const result = fenceFindings(root);

  if (args.includes('--json')) {
    console.log(JSON.stringify(result, null, 2));
    return result.findings.length === 0 ? 0 : 1;
  }

  if (result.inert) {
    console.log('okf_specs_fence: no knowledge bundle here — inert (G1)');
    return 0;
  }
  if (result.findings.length === 0) {
    const byVerdict = new Map();
    for (const entry of result.entries) byVerdict.set(entry.verdict, (byVerdict.get(entry.verdict) || 0) + 1);
    const summary = [...byVerdict.entries()].map(([verdict, n]) => `${n} ${verdict}`).join(', ');
    console.log(
      `PASS okf_specs_fence: ${result.entries.length} file(s) under docs/specs/ (${summary}), 0 findings`,
    );
    return 0;
  }
  console.error('FAIL okf_specs_fence: docs/specs/ is READ-ONLY for new content (okf-switchover-f3 G2)');
  for (const finding of result.findings) {
    console.error(`  ${finding.rel} — ${finding.reason}`);
  }
  return 1;
}

process.exit(main(process.argv));
