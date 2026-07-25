#!/usr/bin/env node
// okf_instructions_fence.mjs — okf-integration-close-f4, cell f4-6: the
// INSTRUCTION layer may not teach the retired state layer without branching.
//
// WHY THIS EXISTS AT ALL. `okf-switchover-f3` flipped the system of record to
// the knowledge bundle and fenced `docs/specs/` read-only for new content — and
// then THREE separate hand audits each found instruction-layer gaps the audit
// before it had missed (7, then 1+1, then 6). Every one was found by a human
// reading prose, and the chain stayed green through all three, because nothing
// in the chain reads prose. A guard only a human runs is a guard that stops
// running. This is that guard, mechanized.
//
// WHAT IT ENFORCES. In a repo WITH a knowledge bundle, every LINE of an
// instruction surface that names the RETIRED state layer —
//     `docs/specs/`  or  `docs/history/learnings/critical-patterns.md`
// — must carry a BUNDLE BRANCH MARKER on that same line: the literal
// `docs/knowledge`, or the word `bundle`. A line that names only the retired
// tree, and gives the reader nothing to suggest another tree exists, teaches
// exactly one model — and it is the wrong one.
//
// WHY THE RULE IS LINE-LOCAL, AND NOT FILE-LOCAL. The obvious cheap rule is
// "the FILE must mention the bundle somewhere". It was written first, run, and
// thrown away, because it is provably blind to the failure that actually
// happens: of the six misroutes the third audit found by hand, FOUR live in
// files that already mention `docs/knowledge` elsewhere (`bee-hive/SKILL.md`
// names it four times and still told every session that
// `critical-patterns.md` is "mandatory pre-work reading"; `bee-session-close`
// is bundle-aware in its logic and still printed "merge it into the touched
// area's spec"). A file-level rule scores those files GREEN. The reason is not
// subtle: an agent reads a bullet, a table row, a checklist item, a hook
// message — in isolation. A branch three paragraphs up does not travel with
// the line. So the unit of the rule is the unit of reading.
//
// WHAT IT DELIBERATELY DOES NOT DO. It does not judge whether a branch is
// CORRECT — that is prose review, and prose review is the thing that kept
// failing. It asks the one question a machine can answer without lying: does
// this line know the bundle exists? The class of defect that walked past three
// audits — a directive with no idea there are two state layers — becomes
// impossible; whether the branch it now carries is well written stays a human's
// job.
//
// THE CLASSIFICATIONS, all structural, never a filename allowlist:
//   (i)   BRANCHED — the line names `docs/knowledge` or says `bundle`. EXEMPT,
//         per hit. This is the rule itself, stated as its own exemption.
//   (ii)  LEGACY ANCHOR CITATION — a hit shaped `docs/specs/<area>.md#<anchor>`
//         or `docs/specs/<area>.md <AnchorId>` (the two forms this repo uses;
//         `<AnchorId>` is the B/R/E/P scheme, e.g. `R17`, `B9a`). D20 keeps the
//         pointer stubs alive precisely so these resolve, so citing an anchor
//         is not teaching the retired reading order — it is using the
//         compatibility surface for the one job it still has. EXEMPT, per hit:
//         a file may cite an anchor on one line and misroute on the next, and
//         only the second line is a finding.
//   (iii) HISTORICAL RECORD — `CREATION-LOG.md`, and anything under a
//         `docs/history/` path. These state what was true when they were
//         written; rewriting them to the current model would falsify a log
//         rather than fix a misroute. EXEMPT, per file.
//   (iv)  BRANCHED SECTION — the nearest preceding markdown HEADING carries a
//         branch marker. A heading is the document's own scope declaration, and
//         a reader arriving at a section reads it before the body (`### 2b. No
//         bundle — the area's spec file`). Headings only, never proximity, and
//         markdown only. EXEMPT, per hit. Measured against the pre-fix tree it
//         covers exactly ONE line and changes no other verdict — added because
//         the one line it covers is pinned VERBATIM by an existing suite
//         (`test_bundle_mode`: "today's rule survives untouched in the
//         no-bundle branch"), not to make a red tree go green.
//   (v)   FENCED EXAMPLE — inside a ``` block in a MARKDOWN surface: a template
//         body, a tree diagram, a sample command. A fence is markdown's own
//         marker for "illustration, not directive". EXEMPT, per hit, markdown
//         only — a hook has no fences, and this repo already carries one
//         recorded hazard from a fence-BLIND extractor (the okf-profile pin's
//         17 unparsed blocks), so tracking fences here is deliberate.
//   (vi)  UNBRANCHED MISROUTE — anything else. Fails the chain, naming
//         `file:line` and quoting the offending line.
//
// THE SURFACES, and why they are these. An instruction surface is prose an
// agent is TOLD to read, plus prose an agent is SHOWN at runtime:
//   - `skills/**/*.md`  — every skill body and reference an agent reads.
//   - `AGENTS.md`       — the operating block loaded into every session.
//   - `hooks/**`        — every file, any extension: a hook's whole output is
//                         prose put in front of an agent, and the P1 this cell
//                         fixes was a string literal inside one.
// NON-markdown under `skills/**` is out of scope, and that is a decision with a
// reason: it is shipped MACHINERY (`templates/lib/*.mjs`, `templates/tests/*.mjs`,
// `scripts/onboard_bee.mjs`), where a retired path is DATA — a fixture path, a
// JSDoc example, a constant the onboarding writer emits — not a directive to an
// agent. Fencing data-shaped strings would demand cosmetic edits to test
// fixtures, which trains people to satisfy the gate instead of fixing the model.
// Those files are already fenced where it counts, by `bundleMode` and by their
// own suites. `scripts/` is not a surface either — it ships to no host and no
// agent is told to read it, which is also why this fence's own source falls
// outside its own scope rather than needing an exemption for itself.
//
// WHY IT IS GATED ON `bundleMode`, and WHICH ROOT EACH HALF RESOLVES AGAINST.
// bee ships to other repos. A host that never migrated has no bundle to branch
// to, and its skills naming `docs/specs/` are simply CORRECT. No bundle -> this
// does not scan at all and reports `inert: true`; a never-migrated host cannot
// tell this shipped. The two halves resolve against DIFFERENT roots, on
// purpose:
//   - the PREDICATE is product-root-resolved, because the bundle is a PRODUCT
//     doc tree (`bundleMode` -> `bundleDir` -> `resolveProductRoot`, G13). In
//     the repo-divorce topology the product's bundle one directory down is what
//     decides whether the fence is armed.
//   - the SURFACES are bee-root-resolved, because `skills/`, `hooks/` and
//     `AGENTS.md` are the HARNESS install, not product docs. They live beside
//     `.bee/`, wherever the product happens to be.
// Asserted both ways in `--selftest` (the divorced fixture arms the fence from
// `product/docs/knowledge/` while grading `skills/` at the workshop root).
//
// RED-FIRST (cell f4-6): this classifier was written and run against the live
// tree BEFORE a single one of the misroutes was fixed, and its red output —
// 25 findings at file:line, the third audit's six among them — is quoted
// verbatim in the cell trace, reproducible at any time with
// `--root <a pristine checkout of 9166943>`. A gate authored after the fixes
// proves only that the fixes were made.
//
// A cheaper file-level rule was written FIRST and thrown away, on evidence: run
// against the same tree it reported 23 findings and scored GREEN on four of the
// six misroutes a human had already found by hand. That run is in the trace too.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// The canonical lib is the TEMPLATE source, not the vendored `.bee/bin/lib`
// copy. `bundleMode` is f3-2/f3-3's ONE predicate — re-deriving "is this repo
// migrated" here would let the two answers drift apart, which is the failure
// class G12/G13 exist to prevent.
import { bundleMode } from '../packages/bee/lib/knowledge.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..');

// ─── the surfaces ──────────────────────────────────────────────────────────

export const SURFACES = [
  { id: 'skill-prose', spec: 'skills/**/*.md', reason: 'every skill body and reference an agent is told to read' },
  { id: 'operating-block', spec: 'AGENTS.md', reason: 'the operating block loaded into every session before anything else' },
  {
    id: 'hook-output',
    spec: 'hooks/** except test_*.mjs',
    reason:
      "any extension: a hook's whole output is prose put in front of an agent. `test_*.mjs` is excluded — run_verify's own suite-discovery glob, so no new naming rule is invented — because a suite's retired-layer strings are its ASSERTIONS and its output goes to a CI log, not to an agent",
  },
];

const SKIP_DIRS = new Set(['node_modules', '.git', 'dist', 'build', 'coverage', '__pycache__']);

// ─── the retired state layer ───────────────────────────────────────────────

const RETIRED_PATTERNS = [
  { id: 'specs-tree', re: /docs\/specs\//g, label: 'docs/specs/' },
  {
    id: 'critical-patterns',
    re: /docs\/history\/learnings\/critical-patterns\.md/g,
    label: 'docs/history/learnings/critical-patterns.md',
  },
];

// The BRANCH MARKERS, line-local. Two, and closed: the bundle's own path, and
// the word that names the predicate every branch in this codebase turns on.
// Either one proves the line's author knew there are two state layers.
const BRANCH_MARKERS = [
  { id: 'bundle-path', test: (line) => line.includes('docs/knowledge'), reason: 'the line names the bundle path itself' },
  { id: 'bundle-word', test: (line) => /\bbundles?\b/i.test(line), reason: 'the line says "bundle" — it states which branch it is, or names the predicate' },
];

// A LEGACY ANCHOR CITATION, anchored at the start of the hit. Both forms this
// repo uses: the `#`-fragment form the bundle's own `sources:` frontmatter
// emits, and the bare `<file>.md <AnchorId>` prose form the skills use. A
// `#`-fragment can only be an anchor, so any fragment counts; the bare form
// must show a real B/R/E/P anchor id, or "docs/specs/x.md Business Rules"
// would buy a pass it has not earned.
const ANCHOR_CITATION = /^docs\/specs\/[A-Za-z0-9._-]+\.md(?:#[A-Za-z0-9._-]+|[\s(]+[BREP]\d+[a-z]?\b)/;

// ─── the exemptions, each NAMED and REASONED (printed by --json) ────────────

export const EXEMPTIONS = [
  {
    id: 'branched',
    scope: 'hit',
    reason:
      'the LINE carries a bundle branch marker (it names docs/knowledge, or says "bundle") — an agent reading that line alone learns the bundle exists and that the retired tree is the no-bundle branch. This is the rule itself, stated as its own exemption.',
  },
  {
    id: 'legacy-anchor',
    scope: 'hit',
    reason:
      'a legacy ANCHOR citation (docs/specs/<area>.md#<anchor> or docs/specs/<area>.md <AnchorId>) — D20 keeps the pointer stubs alive exactly so these resolve; citing an anchor uses the compatibility surface for the one job it still has, it does not teach the retired reading order',
  },
  {
    id: 'historical-record',
    scope: 'file',
    reason:
      'a HISTORICAL record (CREATION-LOG.md, or anything under docs/history/) — it states what was true when it was written; rewriting it to the current model would falsify a log rather than fix a misroute',
  },
  {
    id: 'branched-section',
    scope: 'hit',
    reason:
      "the nearest preceding markdown HEADING carries a branch marker — a heading is the document's own declaration of scope, and a reader arriving at a section reads its heading before its body (`### 2b. No bundle — the area's spec file` is exactly this). Headings only: markdown structure, never proximity. Measured against the pre-fix tree, this class covers ONE line and changes no other verdict — it is not a loophole retrofitted to make the tree pass.",
  },
  {
    id: 'fenced-example',
    scope: 'hit',
    reason:
      'inside a ``` fenced block in a MARKDOWN surface — a template body, a tree diagram, a sample command. A fence is markdown\'s own marker for "this is an illustration, not a directive", and directives live in prose. Markdown only: a hook is not fenced, and this repo already carries one recorded hazard from a fence-blind extractor (okf_migrate, okf-profile pin), so fence tracking here is deliberate rather than incidental.',
  },
];

const MISROUTE_REASON =
  'UNBRANCHED MISROUTE — this line names the retired state layer and carries no bundle branch marker, so an agent that reads it (a bullet, a table row, a hook message) is taught the pre-switchover model as the only model. Add the branch on the line: bundle present -> docs/knowledge/; no bundle -> today\'s guidance, unchanged.';

// ─── classification ────────────────────────────────────────────────────────

export function isHistoricalRecord(rel) {
  const base = rel.slice(rel.lastIndexOf('/') + 1);
  if (base === 'CREATION-LOG.md') return true;
  return rel === 'docs/history' || rel.startsWith('docs/history/') || rel.includes('/docs/history/');
}

export function branchMarkerOn(line) {
  for (const marker of BRANCH_MARKERS) if (marker.test(line)) return marker.id;
  return null;
}

/**
 * Every retired-layer hit on one line, each already marked exempt or not.
 * Exported so the selftest can assert the per-hit rules directly.
 */
export function classifyLine(line, { fenced = false, heading = null } = {}) {
  const branch = branchMarkerOn(line);
  const sectionBranch = !branch && heading ? branchMarkerOn(heading) : null;
  const hits = [];
  for (const pattern of RETIRED_PATTERNS) {
    pattern.re.lastIndex = 0;
    let match;
    while ((match = pattern.re.exec(line)) !== null) {
      const anchored = pattern.id === 'specs-tree' && ANCHOR_CITATION.test(line.slice(match.index));
      hits.push({
        pattern: pattern.id,
        label: pattern.label,
        column: match.index + 1,
        exempt: branch
          ? 'branched'
          : anchored
            ? 'legacy-anchor'
            : sectionBranch
              ? 'branched-section'
              : fenced
                ? 'fenced-example'
                : null,
        marker: branch || sectionBranch,
      });
      if (match.index === pattern.re.lastIndex) pattern.re.lastIndex += 1;
    }
  }
  return hits.sort((a, b) => a.column - b.column);
}

/**
 * Classify ONE instruction-surface file. Returns
 * { verdict, reason, hits, findings } where `verdict` is one of
 * `clean` | `branched` | `legacy-anchor` | `historical-record` | `misroute`.
 */
export function classifyInstructionFile({ rel, text }) {
  const markdown = rel.endsWith('.md');
  const lines = text.split('\n');
  const hits = [];
  let fenced = false;
  let heading = null;
  lines.forEach((line, index) => {
    const isDelimiter = markdown && /^\s*(```|~~~)/.test(line);
    if (markdown && !fenced && !isDelimiter && /^#{1,6}\s/.test(line)) heading = line;
    for (const hit of classifyLine(line, { fenced: fenced && !isDelimiter, heading: markdown ? heading : null })) {
      hits.push({ rel, line: index + 1, text: line.trim(), ...hit });
    }
    if (isDelimiter) fenced = !fenced;
  });

  if (hits.length === 0) return { verdict: 'clean', reason: 'names no retired state layer path', hits, findings: [] };

  if (isHistoricalRecord(rel)) {
    return { verdict: 'historical-record', reason: exemptionReason('historical-record'), hits, findings: [] };
  }

  const unexempt = hits.filter((hit) => !hit.exempt);
  if (unexempt.length === 0) {
    // Report the file by the STRONGEST class present, so `--json` says why it
    // passed rather than just that it did.
    const classes = new Set(hits.map((hit) => hit.exempt));
    const verdict = ['branched', 'legacy-anchor', 'branched-section', 'fenced-example'].find((id) => classes.has(id));
    return { verdict, reason: exemptionReason(verdict), hits, findings: [] };
  }
  // One finding per LINE, not per occurrence: a line naming `docs/specs/`
  // twice is one misroute to fix, and reporting it twice makes the red output
  // read as worse than it is.
  const seen = new Set();
  const perLine = unexempt.filter((hit) => (seen.has(hit.line) ? false : seen.add(hit.line)));
  return {
    verdict: 'misroute',
    reason: MISROUTE_REASON,
    hits,
    findings: perLine.map((hit) => ({
      rel: hit.rel,
      line: hit.line,
      column: hit.column,
      names: hit.label,
      text: hit.text,
      reason: MISROUTE_REASON,
    })),
  };
}

function exemptionReason(id) {
  const hit = EXEMPTIONS.find((entry) => entry.id === id);
  return hit ? hit.reason : id;
}

// ─── scanning ──────────────────────────────────────────────────────────────

export function isSurface(rel) {
  if (rel === 'AGENTS.md') return 'operating-block';
  const base = rel.slice(rel.lastIndexOf('/') + 1);
  // A SUITE is not a hook. `hooks/test_*.mjs` matches run_verify's own suite
  // discovery glob — the repo's existing convention for "this is a test", so no
  // new naming rule is invented here. Its retired-layer strings are the
  // ASSERTIONS ("the no-bundle nudge must still say exactly this") and its
  // output goes to a CI log, never in front of an agent. Same machinery-not-
  // directive boundary as non-markdown under skills/, and the reason the
  // suite that pins this very hook's no-bundle wording verbatim is not itself
  // a misroute.
  if (rel.startsWith('hooks/')) return /^test_.*\.mjs$/.test(base) ? null : 'hook-output';
  if (rel.startsWith('skills/') && rel.endsWith('.md')) return 'skill-prose';
  return null;
}

function listSurfaceFiles(root) {
  const out = [];
  const walk = (abs, rel) => {
    let entries;
    try {
      entries = fs.readdirSync(abs, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const childRel = `${rel}/${entry.name}`;
      if (entry.isDirectory()) {
        if (SKIP_DIRS.has(entry.name)) continue;
        walk(path.join(abs, entry.name), childRel);
      } else if (entry.isFile() && isSurface(childRel)) {
        out.push(childRel);
      }
    }
  };
  walk(path.join(root, 'skills'), 'skills');
  walk(path.join(root, 'hooks'), 'hooks');
  if (fs.existsSync(path.join(root, 'AGENTS.md'))) out.push('AGENTS.md');
  return out.sort();
}

function readText(abs) {
  let buf;
  try {
    buf = fs.readFileSync(abs);
  } catch {
    return null;
  }
  // Binary guard: a NUL byte means this is not prose anybody reads.
  if (buf.includes(0)) return null;
  return buf.toString('utf8');
}

/**
 * Scan a repo. Returns { inert, root, surfaces, exemptions, entries, findings }.
 *   inert    — true when this repo has no knowledge bundle: NOT scanned at all,
 *              never a finding, never a word.
 *   entries  — every instruction-surface file that names the retired state
 *              layer, with its verdict and the reason for it.
 *   findings — the individual file:line misroutes that fail the chain.
 */
export function fenceFindings(root) {
  if (!bundleMode(root)) {
    return { inert: true, root, surfaces: SURFACES, exemptions: EXEMPTIONS, entries: [], findings: [] };
  }

  const entries = [];
  const findings = [];
  for (const rel of listSurfaceFiles(root)) {
    const text = readText(path.join(root, rel));
    if (text === null) continue;
    const result = classifyInstructionFile({ rel, text });
    if (result.verdict === 'clean') continue;
    entries.push({ rel, surface: isSurface(rel), verdict: result.verdict, reason: result.reason, hits: result.hits.length });
    findings.push(...result.findings);
  }

  return { inert: false, root, surfaces: SURFACES, exemptions: EXEMPTIONS, entries, findings };
}

// ─── selftest fixtures ─────────────────────────────────────────────────────

function makeRepo(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `bee-instr-fence-${label}-`));
}

function writeFile(root, rel, text) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

const CONCEPT = [
  '---',
  'type: bee.area',
  'title: Demo area — purpose',
  'description: A canonical fixture concept.',
  'timestamp: 2026-07-22',
  '---',
  '',
  '# Demo area',
  '',
  'Body.',
  '',
].join('\n');

/** The misroute every fixture reuses: an unbranched "read the spec" directive. */
const MISROUTE_LINE = '- Read `docs/specs/<area>.md` before touching the code.';

function bundle(root, prefix = '') {
  writeFile(root, path.join(prefix, 'docs/knowledge/areas/demo-area/overview.md'), CONCEPT);
  return root;
}

function findingKeys(result) {
  return (result.findings || []).map((f) => `${f.rel}:${f.line}`).sort();
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

  // (A) the compatibility guarantee (G1). A never-migrated host has no bundle
  // to branch to; its skills naming docs/specs/ are simply correct. This case
  // must be silent BEFORE and AFTER this fence exists — the one assertion the
  // fence may never change.
  check('bundle-LESS repo with a flagrant misroute is SILENT and reports inert (G1)', () => {
    const root = makeRepo('bundleless');
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    writeFile(root, 'AGENTS.md', '# bee\n\nRead docs/history/learnings/critical-patterns.md first.\n');
    writeFile(root, 'hooks/bee-session-close.mjs', 'const msg = "merge it into the area spec under docs/specs/";\n');
    const result = fenceFindings(root);
    assert(result.inert === true, 'the result says so explicitly: inert');
    assert(result.findings.length === 0, `a host repo with no bundle is never fenced, got ${JSON.stringify(findingKeys(result))}`);
    assert(result.entries.length === 0, 'and it does not even scan');
  });

  // (B) it BITES — the whole point. Same file, same line, bundle present.
  check('bundle-ful repo: an unbranched skill line FAILS, naming file:line and quoting the line', () => {
    const root = bundle(makeRepo('bites'));
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    const result = fenceFindings(root);
    assert(result.inert === false, 'a migrated repo is fenced');
    assert(findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:3', `the offending line is named, got ${JSON.stringify(findingKeys(result))}`);
    assert(result.findings[0].text === MISROUTE_LINE, `the offending line is quoted, got ${JSON.stringify(result.findings[0].text)}`);
    assert(verdictOf(result, 'skills/bee-hive/SKILL.md') === 'misroute', 'classified as a misroute');
  });

  check('the OTHER retired path — critical-patterns.md — fails the same way', () => {
    const root = bundle(makeRepo('patterns'));
    writeFile(root, 'skills/bee-swarming/SKILL.md', '# swarm\n\n- `docs/history/learnings/critical-patterns.md` has been read when present.\n');
    const result = fenceFindings(root);
    assert(findingKeys(result).join(',') === 'skills/bee-swarming/SKILL.md:3', `got ${JSON.stringify(findingKeys(result))}`);
    assert(result.findings[0].names === 'docs/history/learnings/critical-patterns.md', 'the finding names which retired path it saw');
  });

  // (C) the surfaces, and their boundary.
  check('hooks/** (any extension) and AGENTS.md are surfaces; scripts/ and docs/ are not', () => {
    const root = bundle(makeRepo('surfaces'));
    writeFile(root, 'hooks/bee-session-close.mjs', `// close\nconst msg = "merge it into the touched area's spec under docs/specs/";\n`);
    writeFile(root, 'AGENTS.md', `# bee\n\n${MISROUTE_LINE}\n`);
    writeFile(root, 'scripts/some_tool.mjs', `// ${MISROUTE_LINE}\n`);
    writeFile(root, 'docs/backlog.md', MISROUTE_LINE);
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'AGENTS.md:3,hooks/bee-session-close.mjs:2',
      `exactly the declared surfaces, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  check('a SUITE is not a hook: hooks/test_*.mjs is out of scope, the hook beside it is not', () => {
    const root = bundle(makeRepo('hooksuite'));
    writeFile(root, 'hooks/test_hook_contracts.mjs', 'const PINNED = "area spec under docs/specs/ — settled outcome";\n');
    writeFile(root, 'hooks/bee-session-close.mjs', 'const msg = "merge it into the area spec under docs/specs/";\n');
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'hooks/bee-session-close.mjs:1',
      `a suite pins wording, it does not teach it, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  check('shipped MACHINERY under skills/ (non-markdown) is out of scope, by decision', () => {
    const root = bundle(makeRepo('machinery'));
    writeFile(root, 'skills/bee-hive/scripts/test_onboard_bee.mjs', `const ok = checkWrite(root, state, 'docs/specs/tasks.md');\n`);
    writeFile(root, 'skills/bee-hive/scripts/onboard_bee.mjs', '// `into` names where it landed (e.g. "docs/specs/<area>.md").\n');
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:3',
      `fixture paths and JSDoc examples are DATA, not directives, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  // (D) EXEMPTION 1 — the branch marker, line-local. Both markers.
  check('exemption (i): a line naming docs/knowledge PASSES as branched', () => {
    const root = bundle(makeRepo('branched-path'));
    writeFile(root, 'skills/bee-scribing/SKILL.md', '# scribing\n\nWith a bundle write docs/knowledge/areas/<area>/; with none, docs/specs/<area>.md.\n');
    const result = fenceFindings(root);
    assert(findingKeys(result).length === 0, `got ${JSON.stringify(findingKeys(result))}`);
    assert(verdictOf(result, 'skills/bee-scribing/SKILL.md') === 'branched', 'classified as branched');
  });

  check('exemption (i): the word "bundle" alone also branches the line', () => {
    const root = bundle(makeRepo('branched-word'));
    writeFile(root, 'AGENTS.md', '# bee\n\ndocs/specs/  <- read-only compat surface (the state layer when no bundle)\n');
    const result = fenceFindings(root);
    assert(findingKeys(result).length === 0, `got ${JSON.stringify(findingKeys(result))}`);
    assert(verdictOf(result, 'AGENTS.md') === 'branched', 'classified as branched');
  });

  // THE assertion that a file-level rule would fail: this is exactly the shape
  // of four of the six misroutes the third audit found by hand.
  check('a file that names the bundle ELSEWHERE does NOT buy an unbranched line a pass', () => {
    const root = bundle(makeRepo('filelevel'));
    writeFile(
      root,
      'skills/bee-hive/SKILL.md',
      [
        '# hive',
        '',
        '- **With a bundle** read `docs/knowledge/areas/<area>/` first.',
        '',
        '7. `docs/history/learnings/critical-patterns.md` is mandatory pre-work reading.',
        '',
      ].join('\n'),
    );
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:5',
      `a branch three paragraphs up does not travel with the line, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  // (E) EXEMPTION 2 — legacy anchor citations, both forms, and PER HIT.
  check('exemption (ii): a legacy anchor citation PASSES — both the #fragment and the bare-id form', () => {
    const root = bundle(makeRepo('anchor'));
    writeFile(root, 'skills/bee-executing/SKILL.md', '# exec\n\nThe one canonical scratch home (docs/specs/doctrine-layer.md R17).\n');
    writeFile(root, 'skills/bee-validating/SKILL.md', '# validate\n\nSee docs/specs/workflow-state.md#B9a for the rule.\n');
    const result = fenceFindings(root);
    assert(findingKeys(result).length === 0, `anchor citations resolve through the stubs, got ${JSON.stringify(findingKeys(result))}`);
    assert(verdictOf(result, 'skills/bee-executing/SKILL.md') === 'legacy-anchor', 'classified as an anchor citation, not as clean');
    assert(verdictOf(result, 'skills/bee-validating/SKILL.md') === 'legacy-anchor', 'the #fragment form too');
  });

  check('exemption (ii) is PER HIT: an anchor citation buys the LINE a pass, never the FILE', () => {
    const root = bundle(makeRepo('perhit'));
    writeFile(
      root,
      'skills/bee-executing/SKILL.md',
      `# exec\n\nThe one canonical scratch home (docs/specs/doctrine-layer.md R17).\n${MISROUTE_LINE}\n`,
    );
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-executing/SKILL.md:4',
      `only the unbranched line is a finding, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  check('a near-miss of the anchor shape is NOT exempt (a spec path with no anchor id)', () => {
    const root = bundle(makeRepo('nearmiss'));
    writeFile(root, 'skills/bee-hive/SKILL.md', '# hive\n\nOne canonical scratch home (docs/specs/doctrine-layer.md Business Rules).\n');
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:3',
      `"<file>.md Business Rules" names no anchor and must not pass, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  // (F) EXEMPTION 3 — historical record.
  check('exemption (iii): CREATION-LOG.md and docs/history/ PASS as historical record', () => {
    const root = bundle(makeRepo('history'));
    writeFile(root, 'skills/bee-hive/CREATION-LOG.md', `# log\n\n${MISROUTE_LINE}\n`);
    writeFile(root, 'skills/bee-hive/docs/history/old-note.md', MISROUTE_LINE);
    const result = fenceFindings(root);
    assert(findingKeys(result).length === 0, `history is a record, not a routing instruction, got ${JSON.stringify(findingKeys(result))}`);
    assert(verdictOf(result, 'skills/bee-hive/CREATION-LOG.md') === 'historical-record', 'classified as historical record');
    assert(verdictOf(result, 'skills/bee-hive/docs/history/old-note.md') === 'historical-record', 'path-based, not just basename');
  });

  // (F1) EXEMPTION 4 — the section heading is the document's own scope
  // declaration. Narrow on purpose: HEADINGS only, so a branch stated in a
  // paragraph three lines up still buys nothing.
  check('exemption (iv): a line under a heading that names the branch PASSES', () => {
    const root = bundle(makeRepo('section'));
    writeFile(
      root,
      'skills/bee-scribing/SKILL.md',
      ["# scribing", '', "### 2b. No bundle — the area's spec file", '', '**One area = one file, forever.** Check `docs/specs/reading-map.md` first.', ''].join('\n'),
    );
    const result = fenceFindings(root);
    assert(findingKeys(result).length === 0, `the heading scopes the section, got ${JSON.stringify(findingKeys(result))}`);
    assert(verdictOf(result, 'skills/bee-scribing/SKILL.md') === 'branched-section', 'classified by its heading');
  });

  check('exemption (iv) is HEADINGS only — a branch in a nearby PARAGRAPH buys nothing', () => {
    const root = bundle(makeRepo('notsection'));
    writeFile(
      root,
      'skills/bee-scribing/SKILL.md',
      ['# scribing', '', '## Templates', '', '**With no bundle**, today’s guidance stands:', '', 'Path: `docs/specs/<area>.md`.', ''].join('\n'),
    );
    const result = fenceFindings(root);
    assert(findingKeys(result).join(',') === 'skills/bee-scribing/SKILL.md:7', `got ${JSON.stringify(findingKeys(result))}`);
  });

  check('a later heading REPLACES the branch scope — the next section is graded on its own', () => {
    const root = bundle(makeRepo('sectionreset'));
    writeFile(
      root,
      'skills/bee-hive/SKILL.md',
      ['# hive', '', '## With a bundle', '', 'See `docs/specs/x.md` for the stub.', '', '## Runtime Files', '', '- `docs/specs/<area>.md` — read the spec before the code', ''].join('\n'),
    );
    const result = fenceFindings(root);
    assert(findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:9', `got ${JSON.stringify(findingKeys(result))}`);
  });

  // (F2) EXEMPTION 5 — a fenced example is an illustration, and only in
  // markdown. The same two lines outside the fence are still findings, and the
  // same two lines in a HOOK are still findings.
  check('exemption (iv): a ``` fenced block in markdown is an example, not a directive', () => {
    const root = bundle(makeRepo('fenced'));
    writeFile(
      root,
      'skills/bee-scribing/references/scribing-reference.md',
      ['# ref', '', '```markdown', '- `src/auth/` — guards; spec: docs/specs/auth.md', '```', '', MISROUTE_LINE, ''].join('\n'),
    );
    const result = fenceFindings(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-scribing/references/scribing-reference.md:7',
      `only the PROSE line is a finding, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  check('a fence closes: prose AFTER the closing ``` is fenced no longer', () => {
    const root = bundle(makeRepo('fenceclose'));
    writeFile(root, 'skills/bee-hive/SKILL.md', ['# hive', '', '```', 'docs/specs/x.md', '```', 'docs/specs/y.md', ''].join('\n'));
    const result = fenceFindings(root);
    assert(findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:6', `got ${JSON.stringify(findingKeys(result))}`);
  });

  check('fence tracking is MARKDOWN only — a hook that happens to contain ``` is still graded', () => {
    const root = bundle(makeRepo('fencehook'));
    writeFile(root, 'hooks/bee-chain-nudge.mjs', ['// ```', 'const msg = "read docs/specs/<area>.md first";', '// ```'].join('\n'));
    const result = fenceFindings(root);
    assert(findingKeys(result).join(',') === 'hooks/bee-chain-nudge.mjs:2', `got ${JSON.stringify(findingKeys(result))}`);
  });

  // (G) a directory is not a bundle (the .gitkeep rot case, f3-2).
  check('a docs/knowledge/ holding only a .gitkeep does NOT arm the fence', () => {
    const root = makeRepo('gitkeep');
    writeFile(root, 'docs/knowledge/.gitkeep', '');
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    const result = fenceFindings(root);
    assert(result.inert === true, 'a directory is not a bundle');
    assert(result.findings.length === 0, `no findings, got ${JSON.stringify(findingKeys(result))}`);
  });

  // (H) G13 — the two halves resolve against DIFFERENT roots, on purpose: the
  // bundle predicate follows the PRODUCT, the surfaces stay with the HARNESS.
  check('divorced topology: the PRODUCT bundle arms the fence, the WORKSHOP skills/ are graded', () => {
    const root = makeRepo('divorced');
    writeFile(root, '.bee/config.json', `${JSON.stringify({ product_root: 'product' }, null, 2)}\n`);
    bundle(root, 'product');
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    const result = fenceFindings(root);
    assert(result.inert === false, 'a migrated product one directory down IS a bundle');
    assert(
      findingKeys(result).join(',') === 'skills/bee-hive/SKILL.md:3',
      `the harness-root surface is what gets graded, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  // (I) the catalogue is reported, not buried — --json prints every exemption
  // and every surface with its reason.
  check('every exemption and surface is named and reasoned, and every finding quotes its line', () => {
    const root = bundle(makeRepo('reasons'));
    writeFile(root, 'skills/bee-hive/SKILL.md', `# hive\n\n${MISROUTE_LINE}\n`);
    const result = fenceFindings(root);
    assert(result.exemptions.length === 5, `five exemption classes, got ${result.exemptions.length}`);
    for (const exemption of result.exemptions) {
      assert(typeof exemption.reason === 'string' && exemption.reason.length > 20, `${exemption.id} states its reason`);
      assert(exemption.scope === 'hit' || exemption.scope === 'file', `${exemption.id} states its scope`);
    }
    assert(result.surfaces.length === 3, `three surfaces, got ${result.surfaces.length}`);
    for (const surface of result.surfaces) assert(typeof surface.reason === 'string' && surface.reason.length > 10, `${surface.id} states its reason`);
    for (const entry of result.entries) assert(typeof entry.reason === 'string' && entry.reason.length > 0, `${entry.rel} carries a stated reason`);
    for (const finding of result.findings) assert(typeof finding.text === 'string' && finding.text.length > 0, `${finding.rel} quotes the line`);
  });

  // (J) the live repo — the assertion that would have caught all three hand
  // audits, and every audit after them.
  check("bee's own instruction surfaces produce ZERO unbranched misroutes", () => {
    const result = fenceFindings(REPO_ROOT);
    assert(result.inert === false, "bee's own checkout has a bundle");
    assert(result.entries.length > 0, 'the live surfaces are enumerated');
    assert(
      result.findings.length === 0,
      `the live instruction layer must be silent, got:\n${result.findings.map((f) => `  ${f.rel}:${f.line} — ${f.text}`).join('\n')}`,
    );
  });

  console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} okf_instructions_fence --selftest: ${passed} passed, ${failed} failed`);
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
    console.log('okf_instructions_fence: no knowledge bundle here — inert (G1)');
    return 0;
  }
  if (result.findings.length === 0) {
    const byVerdict = new Map();
    for (const entry of result.entries) byVerdict.set(entry.verdict, (byVerdict.get(entry.verdict) || 0) + 1);
    const summary = [...byVerdict.entries()].sort().map(([verdict, n]) => `${n} ${verdict}`).join(', ');
    console.log(
      `PASS okf_instructions_fence: ${result.entries.length} instruction-surface file(s) name the retired state layer (${summary}), 0 unbranched misroutes`,
    );
    return 0;
  }
  console.error(
    'FAIL okf_instructions_fence: the instruction layer teaches the retired state layer with no bundle branch (okf-integration-close-f4 f4-6)',
  );
  for (const finding of result.findings) {
    console.error(`  ${finding.rel}:${finding.line} names ${finding.names}`);
    console.error(`    ${finding.text}`);
  }
  console.error(`  ${result.findings.length} unbranched misroute(s) — add the branch on the line, or cite an anchor if the reference is a legacy citation.`);
  return 1;
}

process.exit(main(process.argv));
