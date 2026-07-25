#!/usr/bin/env node
// test_skill_pointers.mjs — router-cost, cell rc-2: pointer integrity for the
// instruction layer. Every reference a skill defers to must actually exist,
// and if the pointer also names a section, that section must actually be
// there.
//
// WHY THIS EXISTS. D5 (docs/history/router-cost/CONTEXT.md): this guard lands
// BEFORE rc-4 slims the router by moving prose out of skill bodies and into
// reference docs. Moving prose is exactly the kind of migration whose
// instruction layer rots silently — pattern
// `20260722-a-migration-is-not-done-until-its-instructions-are` — and nothing
// in the chain today reads a `references/foo.md` pointer and checks it
// resolves. Cut first, guard later would be cutting without a net.
//
// WHAT IT CHECKS. Every `skills/**/*.md` SOURCE file is scanned for
// backtick-quoted paths that name a `references/` file, in the two forms this
// repo actually uses:
//   - bare:           `references/foo.md`            (resolved against the
//                      CITING skill's own directory — skills/<skill>/)
//   - skill-qualified: `bee-hive/references/foo.md`   (resolved against the
//                      skills/ root directly — skills/bee-hive/references/foo.md)
// For every such pointer:
//   1. the target file must EXIST.
//   2. if the pointer also names a SECTION — the three prose conventions
//      actually in use in this repo (and no others; see classifySection
//      below) — the target file must carry a matching heading.
//
// WHY ONLY skills/**/*.md, AND WHY THE PROJECTION ROOTS ARE EXCLUDED. This
// repo renders four byte-identical copies of the skills/ tree for different
// runtimes/hosts: `.claude/skills/`, `.agents/skills/`, `.claude-plugin/
// skills/`, `.codex-plugin/skills/` (see scripts/release_manifest.mjs's
// PLUGIN_SKILL_RENDER_ROOTS for the same fact used elsewhere). Scanning those
// too would report every real finding FIVE times over (once per copy) and
// bury the signal exactly the way the router-cost CONTEXT's own diagnosis
// describes: noise crowding out the one thing that mattered. The walker below
// only ever descends into the top-level `skills/` directory, so those four
// roots are never visited — the exclusion happens by construction. An
// explicit, testable `isExcludedProjection` guard sits alongside it anyway
// (defense in depth: the day this walker is generalized to start at repo
// root, the exclusion must not depend on nobody having generalized it
// wrong).
//
// THE HARD REQUIREMENT — NEGATIVE CONTROLS. `--selftest` proves the two
// failure modes are actually DETECTED (a missing target file, a missing
// section), not merely that valid pointers pass. This repo has already
// shipped three scans in one session whose scope was too narrow to catch
// what they existed to catch (pattern
// `20260723-a-scan-scope-set-from-assumption-passes-green-while-hiding-the-bug`).
// A gate never observed failing is not known to work.
//
// REPORTING STYLE follows okf_instructions_fence.mjs:211-259 — every finding
// names `file:line` and quotes the offending line verbatim.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, '..', '..');

// ─── the source root and the excluded rendered projections ─────────────────

const SOURCE_ROOT = 'skills';

// Rendered, byte-identical projections of skills/** — never the source of
// truth, and scanning them multiplies every real finding once per projection.
// Kept as an explicit, path-prefix guard (not just "the walker starts at
// skills/ so it never sees these") so the exclusion is stated and testable on
// its own, per rc-2's action spec.
const EXCLUDED_PROJECTION_PREFIXES = ['.claude/', '.agents/', '.claude-plugin/', '.codex-plugin/'];

export function isExcludedProjection(rel) {
  return EXCLUDED_PROJECTION_PREFIXES.some((prefix) => rel === prefix.slice(0, -1) || rel.startsWith(prefix));
}

const SKIP_DIRS = new Set(['node_modules', '.git', 'dist', 'build', 'coverage', '__pycache__']);

// ─── pointer extraction ─────────────────────────────────────────────────────

// A backtick-quoted path naming a references/ file, in either form this repo
// uses: bare (`references/foo.md`) or skill-qualified
// (`bee-hive/references/foo.md` — exactly one path segment before
// `references/`, never a deeper `skills/bee-hive/references/foo.md`, because
// that longer form does not occur anywhere in the live tree).
const POINTER_RE = /`((?:[A-Za-z0-9_.-]+\/)?references\/[A-Za-z0-9._-]+\.md)`/g;

// The three section-naming conventions actually in use in this repo (rc-2's
// action spec: "handle those three; do not invent more").
//
//   (1) HEADING form:      `references/foo.md`'s `## Bar`
//   (2) PARENTHETICAL form: `references/foo.md` ("Bar" ...)      — the path's
//       closing backtick is followed (after only whitespace) directly by a
//       `(` and a double-quoted string; anything after the quote inside the
//       parens (", D4", for instance) is not part of the section name.
//   (3) ARROW form:        -> references/foo.md ("Bar")           — the path
//       may or may not itself be backtick-quoted.
//
// All three bind the section to the SAME pointer path captured in the same
// match, so there is never any ambiguity about which pointer a section name
// belongs to.
const SECTION_HEADING_RE = /`((?:[A-Za-z0-9_.-]+\/)?references\/[A-Za-z0-9._-]+\.md)`'s\s*`(#{1,6}\s*[^`]+)`/g;
const SECTION_PAREN_RE = /`((?:[A-Za-z0-9_.-]+\/)?references\/[A-Za-z0-9._-]+\.md)`\s*\(\s*"([^"]+)"/g;
const SECTION_ARROW_RE = /->\s*`?((?:[A-Za-z0-9_.-]+\/)?references\/[A-Za-z0-9._-]+\.md)`?\s*\(\s*"([^"]+)"\s*\)/g;

/**
 * Every pointer named on ONE line, each carrying its column and — when the
 * line uses one of the three section-naming conventions — the section name
 * it names. Exported so the selftest can assert the per-line extraction
 * directly, the same shape okf_instructions_fence.mjs uses for classifyLine.
 */
export function extractPointers(line) {
  const byPath = new Map();

  POINTER_RE.lastIndex = 0;
  let m;
  while ((m = POINTER_RE.exec(line)) !== null) {
    if (!byPath.has(m[1])) byPath.set(m[1], { pointerPath: m[1], column: m.index + 1, section: null });
    if (m.index === POINTER_RE.lastIndex) POINTER_RE.lastIndex += 1;
  }

  const applySection = (re, extractSection) => {
    re.lastIndex = 0;
    let match;
    while ((match = re.exec(line)) !== null) {
      const pointerPath = match[1];
      const section = extractSection(match[2]);
      const existing = byPath.get(pointerPath);
      if (existing) {
        existing.section = section;
      } else {
        byPath.set(pointerPath, { pointerPath, column: match.index + 1, section });
      }
      if (match.index === re.lastIndex) re.lastIndex += 1;
    }
  };

  applySection(SECTION_HEADING_RE, (raw) => raw.replace(/^#{1,6}\s*/, '').trim());
  applySection(SECTION_PAREN_RE, (raw) => raw.trim());
  applySection(SECTION_ARROW_RE, (raw) => raw.trim());

  return [...byPath.values()].sort((a, b) => a.column - b.column);
}

/** The citing file's own skill directory: skills/<skill-name>. */
function skillDirOf(rel) {
  const parts = rel.split('/');
  return parts.slice(0, 2).join('/');
}

/** Resolve a captured pointer path to a repo-relative target path. */
export function resolvePointer(pointerPath, citingRel) {
  if (pointerPath.startsWith('references/')) {
    return path.posix.join(skillDirOf(citingRel), pointerPath);
  }
  return path.posix.join(SOURCE_ROOT, pointerPath);
}

/**
 * Does a target file's heading text match a cited section name? Exact match,
 * or the heading is the section name followed by a word boundary (so
 * `### Goal-check judge tier (D4/D5, self-correcting-loop) — verification`
 * matches a citation of "Goal-check judge tier" without matching "Goal-check
 * judge tiers" or similar near-misses).
 */
export function headingMatches(headingBody, sectionName) {
  const h = headingBody.trim();
  const s = sectionName.trim();
  if (h === s) return true;
  if (!h.startsWith(s)) return false;
  const rest = h.slice(s.length);
  return rest === '' || /^[\s(:—-]/.test(rest);
}

/** Every markdown heading in a file's text, in order. */
function headingsOf(text) {
  const out = [];
  text.split('\n').forEach((line, index) => {
    const m = /^(#{1,6})\s+(.*)$/.exec(line);
    if (m) out.push({ line: index + 1, body: m[2] });
  });
  return out;
}

// ─── scanning ────────────────────────────────────────────────────────────

function listSourceFiles(root) {
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
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        out.push(childRel);
      }
    }
  };
  walk(path.join(root, SOURCE_ROOT), SOURCE_ROOT);
  return out.filter((rel) => !isExcludedProjection(rel)).sort();
}

function readText(abs) {
  let buf;
  try {
    buf = fs.readFileSync(abs);
  } catch {
    return null;
  }
  if (buf.includes(0)) return null; // binary guard, same as okf_instructions_fence.mjs
  return buf.toString('utf8');
}

/**
 * Scan a repo root. Returns { findings, checked }, where `checked` is the
 * total number of distinct pointers examined and `findings` is every
 * file:line that fails EXISTS or (when named) the section check.
 */
export function scanPointers(root) {
  const findings = [];
  let checked = 0;

  for (const rel of listSourceFiles(root)) {
    const text = readText(path.join(root, rel));
    if (text === null) continue;
    const lines = text.split('\n');
    lines.forEach((line, index) => {
      for (const pointer of extractPointers(line)) {
        checked += 1;
        const targetRel = resolvePointer(pointer.pointerPath, rel);
        const targetAbs = path.join(root, ...targetRel.split('/'));
        const lineNo = index + 1;
        if (!fs.existsSync(targetAbs) || !fs.statSync(targetAbs).isFile()) {
          findings.push({
            rel,
            line: lineNo,
            text: line.trim(),
            pointerPath: pointer.pointerPath,
            targetRel,
            kind: 'missing-file',
            reason: `points at ${targetRel}, which does not exist`,
          });
          return; // no point checking a section inside a file that isn't there
        }
        if (pointer.section) {
          const targetText = readText(targetAbs) ?? '';
          const found = headingsOf(targetText).some((h) => headingMatches(h.body, pointer.section));
          if (!found) {
            findings.push({
              rel,
              line: lineNo,
              text: line.trim(),
              pointerPath: pointer.pointerPath,
              targetRel,
              kind: 'missing-section',
              reason: `names section "${pointer.section}" in ${targetRel}, which has no matching heading`,
            });
          }
        }
      }
    });
  }

  return { findings, checked };
}

// ─── selftest fixtures ──────────────────────────────────────────────────────

function makeRepo(label) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `bee-skill-pointers-${label}-`));
}

function writeFile(root, rel, text) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, text, 'utf8');
  return abs;
}

function findingKeys(result) {
  return result.findings.map((f) => `${f.rel}:${f.line}`).sort();
}

// ─── selftest ────────────────────────────────────────────────────────────

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

  // ── the two NEGATIVE CONTROLS this cell exists to prove (rc-2 hard req) ──

  check('NEGATIVE CONTROL: a pointer to a MISSING FILE is DETECTED', () => {
    const root = makeRepo('missing-file');
    writeFile(root, 'skills/bee-demo/SKILL.md', '# demo\n\nSee `references/does-not-exist.md` for details.\n');
    const result = scanPointers(root);
    assert(findingKeys(result).join(',') === 'skills/bee-demo/SKILL.md:3', `got ${JSON.stringify(findingKeys(result))}`);
    assert(result.findings[0].kind === 'missing-file', `classified as missing-file, got ${result.findings[0].kind}`);
    assert(result.findings[0].text.includes('does-not-exist.md'), 'the offending line is quoted verbatim');
  });

  check('NEGATIVE CONTROL: a pointer naming a MISSING SECTION is DETECTED (the target file exists, the heading does not)', () => {
    const root = makeRepo('missing-section');
    writeFile(root, 'skills/bee-demo/SKILL.md', '# demo\n\nSee `references/foo.md` ("Nonexistent Heading") for details.\n');
    writeFile(root, 'skills/bee-demo/references/foo.md', '# foo\n\n## An Entirely Different Heading\n\nBody.\n');
    const result = scanPointers(root);
    assert(findingKeys(result).join(',') === 'skills/bee-demo/SKILL.md:3', `got ${JSON.stringify(findingKeys(result))}`);
    assert(result.findings[0].kind === 'missing-section', `classified as missing-section, got ${result.findings[0].kind}`);
  });

  // ── the check actually passes valid input (the positive complement) ──

  check('a bare pointer to a file that exists in the SAME skill PASSES', () => {
    const root = makeRepo('bare-ok');
    writeFile(root, 'skills/bee-demo/SKILL.md', '# demo\n\nSee `references/foo.md` for details.\n');
    writeFile(root, 'skills/bee-demo/references/foo.md', '# foo\n');
    const result = scanPointers(root);
    assert(result.findings.length === 0, `got ${JSON.stringify(findingKeys(result))}`);
    assert(result.checked === 1, `exactly one pointer checked, got ${result.checked}`);
  });

  check('a skill-qualified pointer resolves against skills/ root, not the citing skill\'s own dir', () => {
    const root = makeRepo('qualified-ok');
    writeFile(root, 'skills/bee-swarming/SKILL.md', '# swarm\n\nSee `bee-hive/references/foo.md` for details.\n');
    writeFile(root, 'skills/bee-hive/references/foo.md', '# foo\n');
    const result = scanPointers(root);
    assert(result.findings.length === 0, `got ${JSON.stringify(findingKeys(result))}`);
  });

  check('a BARE pointer with the same filename as another skill\'s reference is NOT satisfied by that other skill\'s copy', () => {
    const root = makeRepo('bare-scoped');
    // bee-swarming cites a BARE `references/foo.md`, but only bee-hive has one.
    writeFile(root, 'skills/bee-swarming/SKILL.md', '# swarm\n\nSee `references/foo.md` for details.\n');
    writeFile(root, 'skills/bee-hive/references/foo.md', '# foo\n');
    const result = scanPointers(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-swarming/SKILL.md:3',
      `bare must resolve against the CITING skill's own dir, not any skill that happens to have the file, got ${JSON.stringify(findingKeys(result))}`,
    );
    assert(result.findings[0].kind === 'missing-file', 'classified as missing-file');
  });

  // ── the three section-naming conventions, each detected AND each passing when correct ──

  check('section form (1) HEADING — `path`\'s `## Bar` — passes when the heading exists, fails when it does not', () => {
    const root = makeRepo('section-heading');
    writeFile(root, 'skills/bee-demo/SKILL.md', "# demo\n\nRead `references/foo.md`'s `## Critical patterns` section.\n");
    writeFile(root, 'skills/bee-demo/references/foo.md', '# foo\n\n## Critical patterns\n\nBody.\n');
    const ok = scanPointers(root);
    assert(ok.findings.length === 0, `got ${JSON.stringify(findingKeys(ok))}`);

    const rootBad = makeRepo('section-heading-bad');
    writeFile(rootBad, 'skills/bee-demo/SKILL.md', "# demo\n\nRead `references/foo.md`'s `## Critical patterns` section.\n");
    writeFile(rootBad, 'skills/bee-demo/references/foo.md', '# foo\n\n## Something Else\n\nBody.\n');
    const bad = scanPointers(rootBad);
    assert(findingKeys(bad).join(',') === 'skills/bee-demo/SKILL.md:3', `got ${JSON.stringify(findingKeys(bad))}`);
    assert(bad.findings[0].kind === 'missing-section', 'classified as missing-section');
  });

  check('section form (2) PARENTHETICAL — `path` ("Bar") — passes when the heading exists (prefix match, trailing annotation allowed)', () => {
    const root = makeRepo('section-paren');
    writeFile(
      root,
      'skills/bee-swarming/SKILL.md',
      'dispatch the checklist judge from the tier table in `bee-hive/references/foo.md` ("Goal-check judge tier") and record it.\n',
    );
    writeFile(root, 'skills/bee-hive/references/foo.md', '### Goal-check judge tier (D4/D5, self-correcting-loop) — verification, not review\n\nBody.\n');
    const result = scanPointers(root);
    assert(result.findings.length === 0, `a trailing annotation after the cited name must still pass, got ${JSON.stringify(findingKeys(result))}`);
  });

  check('section form (2) PARENTHETICAL with an extra trailing citation — `path` ("Bar", D4) — the comma-tail does not corrupt the section name', () => {
    const root = makeRepo('section-paren-tail');
    writeFile(root, 'skills/bee-swarming/SKILL.md', 'per `bee-hive/references/foo.md` ("Goal-check judge tier", D4) applies here too.\n');
    writeFile(root, 'skills/bee-hive/references/foo.md', '### Goal-check judge tier (D4/D5) — verification\n');
    const result = scanPointers(root);
    assert(result.findings.length === 0, `got ${JSON.stringify(findingKeys(result))}`);
  });

  check('section form (3) ARROW — -> path ("Bar") — is detected and checked both ways', () => {
    const rootOk = makeRepo('section-arrow-ok');
    writeFile(rootOk, 'skills/bee-demo/SKILL.md', 'Full contract: -> references/foo.md ("Bar")\n');
    writeFile(rootOk, 'skills/bee-demo/references/foo.md', '## Bar\n\nBody.\n');
    const ok = scanPointers(rootOk);
    assert(ok.findings.length === 0, `got ${JSON.stringify(findingKeys(ok))}`);

    const rootBad = makeRepo('section-arrow-bad');
    writeFile(rootBad, 'skills/bee-demo/SKILL.md', 'Full contract: -> references/foo.md ("Bar")\n');
    writeFile(rootBad, 'skills/bee-demo/references/foo.md', '## Something Else\n\nBody.\n');
    const bad = scanPointers(rootBad);
    assert(findingKeys(bad).join(',') === 'skills/bee-demo/SKILL.md:1', `got ${JSON.stringify(findingKeys(bad))}`);
    assert(bad.findings[0].kind === 'missing-section', 'classified as missing-section');
  });

  check('a near-miss heading (the cited name as a substring, not a prefix-with-boundary) is NOT accepted', () => {
    const root = makeRepo('near-miss');
    writeFile(root, 'skills/bee-demo/SKILL.md', 'See `references/foo.md` ("Goal") for details.\n');
    writeFile(root, 'skills/bee-demo/references/foo.md', '## Goalkeeper notes\n');
    const result = scanPointers(root);
    assert(
      findingKeys(result).join(',') === 'skills/bee-demo/SKILL.md:1',
      `"Goal" must not match a heading that merely STARTS WITH those letters with no boundary, got ${JSON.stringify(findingKeys(result))}`,
    );
  });

  // ── scope: rendered projection roots are excluded, by path prefix ──

  check('rendered projection roots (.claude/, .agents/, .claude-plugin/, .codex-plugin/) are EXCLUDED — a broken pointer inside one is invisible', () => {
    const root = makeRepo('projections');
    writeFile(root, 'skills/bee-demo/SKILL.md', '# demo\n\nSee `references/foo.md` for details.\n');
    writeFile(root, 'skills/bee-demo/references/foo.md', '# foo\n');
    // Byte-copy the SAME broken pointer into all four projection roots.
    for (const proj of ['.claude', '.agents', '.claude-plugin', '.codex-plugin']) {
      writeFile(root, `${proj}/skills/bee-demo/SKILL.md`, '# demo\n\nSee `references/gone.md` for details.\n');
    }
    const result = scanPointers(root);
    assert(result.findings.length === 0, `projections must never be scanned, got ${JSON.stringify(findingKeys(result))}`);
    assert(result.checked === 1, `only the ONE source pointer is checked, got ${result.checked}`);
  });

  check('isExcludedProjection classifies by path prefix, not by substring anywhere in the path', () => {
    assert(isExcludedProjection('.claude/skills/bee-demo/SKILL.md'), 'a real projection path is excluded');
    assert(!isExcludedProjection('skills/bee-demo/references/uses-dot-claude-in-prose.md'), 'a source file must not be excluded just because a projection name appears later in its own path');
  });

  // ── one broken pointer in ONE source copy yields exactly one finding ──

  check('a broken pointer that exists only in the skills/ source (not any projection) yields exactly ONE finding, not five', () => {
    const root = makeRepo('single-finding');
    writeFile(root, 'skills/bee-demo/SKILL.md', '# demo\n\nSee `references/gone.md` for details.\n');
    for (const proj of ['.claude', '.agents', '.claude-plugin', '.codex-plugin']) {
      writeFile(root, `${proj}/skills/bee-demo/SKILL.md`, '# demo\n\nSee `references/gone.md` for details.\n');
    }
    const result = scanPointers(root);
    assert(findingKeys(result).join(',') === 'skills/bee-demo/SKILL.md:3', `got ${JSON.stringify(findingKeys(result))}`);
    assert(result.findings.length === 1, `exactly one finding, not one per projection copy, got ${result.findings.length}`);
  });

  // ── the live repo itself: the assertion the bare run makes ──

  check("bee's own skills/**/*.md pointers all resolve (0 findings against the real tree)", () => {
    const result = scanPointers(REPO_ROOT);
    assert(result.checked > 50, `the live tree has plenty of pointers to check, got ${result.checked}`);
    assert(
      result.findings.length === 0,
      `the live pointer layer must be clean, got:\n${result.findings.map((f) => `  ${f.rel}:${f.line} — ${f.reason}`).join('\n')}`,
    );
  });

  console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} test_skill_pointers --selftest: ${passed} passed, ${failed} failed`);
  return failed === 0 ? 0 : 1;
}

// ─── CLI ─────────────────────────────────────────────────────────────────

function main(argv) {
  const args = argv.slice(2);
  if (args.includes('--selftest')) return runSelftest();

  const rootIdx = args.indexOf('--root');
  const root = rootIdx === -1 ? REPO_ROOT : path.resolve(args[rootIdx + 1]);
  const result = scanPointers(root);

  if (args.includes('--json')) {
    console.log(JSON.stringify(result, null, 2));
    return result.findings.length === 0 ? 0 : 1;
  }

  if (result.findings.length === 0) {
    console.log(`PASS test_skill_pointers: ${result.checked} reference pointer(s) checked across skills/**/*.md, 0 broken`);
    return 0;
  }

  console.error('FAIL test_skill_pointers: a skill points at a reference that does not resolve (router-cost rc-2, D5)');
  for (const finding of result.findings) {
    console.error(`  ${finding.rel}:${finding.line} — ${finding.reason}`);
    console.error(`    ${finding.text}`);
  }
  console.error(`  ${result.findings.length} broken pointer(s) out of ${result.checked} checked.`);
  return 1;
}

process.exit(main(process.argv));
