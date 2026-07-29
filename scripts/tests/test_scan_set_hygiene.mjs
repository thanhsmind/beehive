#!/usr/bin/env node
// Scan-set hygiene gate — derived-check-hardening E4/E8.
//
// Two sweeps that were each run BY HAND exactly once, then forgotten, become
// standing checks here. Both trace to
// docs/knowledge/patterns/20260728-a-scan-set-from-the-git-index-crashes-the-gate-that-guards-it.md.
//
//   CHECK 1 (E4) — an unguarded git-index scan set. `git ls-files` reflects
//   the INDEX, not the working tree: a file deleted on disk but not yet
//   staged still comes back. Reading such a path throws ENOENT and crashes
//   whatever gate derived its scan set that way — exactly what happened to
//   test_doctrine_parity.mjs (vd-13) and sat live, unnoticed, in
//   test_portable_paths.mjs until dch-4. This check flags any file under
//   scripts/tests/** or packages/bee/** that derives a path list from a
//   `git ... ls-files` invocation and later reads paths from that list
//   (readFileSync/statSync/readFile/copyFileSync) with no existence guard
//   between the two.
//
//   CHECK 2 (E8) — a retired workflow stage described as current. D11's own
//   acceptance criterion said "no live file may describe a retired stage as
//   current" — but it was a manual sweep run once, and two residuals
//   (skills/bee-xia/references/research-brief-template.md,
//   packages/bee/hooks/test_write_guard.mjs) slipped through it and had to
//   be cleaned up later, in dch-7. This check derives the retired-stage
//   token(s) from `LEGACY_PHASE_COERCIONS` in packages/bee/lib/state.mjs
//   (the phase enum's own record of what it used to carry) rather than
//   hardcoding a name, so it survives the next retirement unmodified.
//
// Both checks are STATIC SOURCE SCANS over plain text, not an AST walk or a
// dynamic import — there is no parser dependency in this repo (no
// package.json, no node_modules), and test_doctrine_parity.mjs already
// establishes regex-heuristic doctrine checks as the house convention here.
// Each classifier is proven on synthetic fixtures via --selftest before it
// is ever pointed at this tree, exactly like test_doctrine_parity.mjs's own
// runSelftest() — so "prove it fails correctly, naming the file" is a
// permanent, reproducible part of the suite, not a probe script written
// once and deleted.
//
// Usage:
//   node scripts/tests/test_scan_set_hygiene.mjs             # self-test, then real checks
//   node scripts/tests/test_scan_set_hygiene.mjs --selftest  # self-test only

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const NAME = "test_scan_set_hygiene";

// ════════════════════════════════════════════════════════════════════════
// CHECK 1 (E4) — unguarded git-index scan sets
// ════════════════════════════════════════════════════════════════════════

// Matches a real git-ls-files INVOCATION (execFileSync/spawnSync/execSync
// calling the "git" binary with "ls-files" somewhere in its argv/command),
// not a bare co-occurrence of the two words — packages/bee/lib/guards.mjs
// lists "ls-files" in a Set of read-only git subcommands, nowhere near an
// actual invocation, and must not trip this.
const GIT_LS_FILES_RE = /\b(execFileSync|spawnSync|execSync)\s*\(\s*[`"']git[`"']?[\s\S]{0,200}?ls-files/g;

const READ_CALL_RE = /\b(readFileSync|statSync|readFile|copyFileSync)\s*\(/;
const EXISTS_SYNC_RE = /existsSync/;

// Locates the function body (start/end line index, inclusive) enclosing a
// given line, by scanning backward for a function-opening line and then
// forward, counting braces, until they balance back to zero. This is a
// heuristic — it does not understand strings/comments/regex literals — but
// every real candidate in this repo (deriveScanSet, stagedSource,
// trackedGitignorePaths) is plainly formatted enough that it holds, and the
// two known-good files this check must never flag are exactly what its
// selftest proves it correctly clears.
function enclosingFunctionRange(lines, matchLineIdx) {
  const FUNC_START_RE =
    /^\s*(export\s+)?(default\s+)?(async\s+)?function\s+\w+\s*\(|^\s*(export\s+)?(const|let)\s+\w+\s*=\s*(async\s*)?\([^)]*\)\s*=>\s*\{|^\s*(async\s+)?\w+\s*\([^)]*\)\s*\{\s*$/;
  let start = -1;
  for (let i = matchLineIdx; i >= 0; i--) {
    if (FUNC_START_RE.test(lines[i])) {
      start = i;
      break;
    }
  }
  if (start === -1) return null;
  let depth = 0;
  let seenOpen = false;
  for (let i = start; i < lines.length; i++) {
    for (const ch of lines[i]) {
      if (ch === "{") {
        depth++;
        seenOpen = true;
      } else if (ch === "}") {
        depth--;
      }
    }
    if (seenOpen && depth <= 0) return [start, i];
  }
  return [start, lines.length - 1];
}

/**
 * Scans one file's text for unguarded git-ls-files-derived scan sets.
 * Returns an array of { line, reason } violations (empty when clean).
 */
function findUnguardedScanSets(text) {
  const violations = [];
  const lines = text.split("\n");
  let m;
  GIT_LS_FILES_RE.lastIndex = 0;
  while ((m = GIT_LS_FILES_RE.exec(text))) {
    const matchLine = text.slice(0, m.index).split("\n").length - 1;
    const range = enclosingFunctionRange(lines, matchLine);
    // No enclosing function found (top-level script code) — fall back to a
    // bounded forward window so the check still has something to look at,
    // rather than silently skipping module-scope derivations.
    const [start, end] = range || [matchLine, Math.min(matchLine + 80, lines.length - 1)];
    const bodyText = lines.slice(start, end + 1).join("\n");
    const hasRead = READ_CALL_RE.test(bodyText);
    if (!hasRead) continue; // derived but never read from disk here — E4 does not apply (E1/registry territory, not this check)
    const hasGuard = EXISTS_SYNC_RE.test(bodyText);
    if (!hasGuard) {
      violations.push({
        line: matchLine + 1,
        reason:
          "derives a path list from a git ls-files invocation and reads from it " +
          "(readFileSync/statSync/readFile/copyFileSync) with no existsSync guard " +
          "in the enclosing function",
      });
    }
  }
  return violations;
}

// `unquoteGitPath`/porcelain parsing mirrors scripts/run_verify.mjs:868-885's
// statusPorcelainFiles() and test_portable_paths.mjs's own copy of the same
// shape (skip the 2-char status code, resolve a rename's destination,
// unquote) — no third parsing approach invented for this suite.
function unquoteGitPath(raw) {
  const s = raw.trim();
  if (s.length >= 2 && s.startsWith('"') && s.endsWith('"')) {
    try {
      return JSON.parse(s);
    } catch {
      return s.slice(1, -1);
    }
  }
  return s;
}

function statusPorcelainPaths(root, pathspecs) {
  const out = execFileSync("git", ["-C", root, "status", "--porcelain", "--", ...pathspecs], {
    maxBuffer: 64 * 1024 * 1024,
  }).toString("utf8");
  const files = [];
  for (const line of out.split("\n")) {
    if (!line) continue;
    const rest = line.slice(3);
    const arrowIdx = rest.indexOf(" -> ");
    const rawPath = arrowIdx === -1 ? rest : rest.slice(arrowIdx + 4);
    const cleaned = unquoteGitPath(rawPath);
    if (cleaned) files.push(cleaned);
  }
  return files;
}

// git ls-files reflects the INDEX (this is exactly the E4 defect this suite
// is built to catch, applied to its own scan-set derivation): a file
// written but not yet staged is invisible to it. Unioned with `git status
// --porcelain` — the same fix dch-4 gave test_portable_paths.mjs for E3 —
// so a freshly written violation is caught before it is ever `git add`ed,
// not only after. Existence-filtered so a deleted-but-still-indexed path
// never reaches a read.
function trackedExistingFiles(root, pathspecs) {
  const raw = execFileSync("git", ["-C", root, "ls-files", "-z", "--", ...pathspecs], {
    maxBuffer: 64 * 1024 * 1024,
  }).toString("utf8");
  const tracked = raw.split("\0").filter(Boolean);
  const untracked = statusPorcelainPaths(root, pathspecs);
  return [...new Set([...tracked, ...untracked])]
    .filter((p) => fs.existsSync(path.join(root, p)))
    .sort();
}

// This suite's own path — excluded from its own Check 1 scan below. Its
// selftest fixtures are backtick template literals that quote the exact
// git-ls-files-plus-unguarded-read text as DATA, to prove the classifier
// bites (see runSelftestCheck1) — a bare text scan cannot tell "this line
// is fixture data proving the rule" from "this line is the rule's own
// violation" (the fixtures are plain example code, not JS syntax this file
// executes, so there is no live derivation here to guard). Every other file
// in scope is still fully scanned; this is the one legitimately
// self-referential source, not a general carve-out.
const SELF_PATH = "scripts/tests/test_scan_set_hygiene.mjs";

function checkUnguardedScanSets(root) {
  const files = trackedExistingFiles(root, ["scripts/tests/*.mjs", "packages/bee/**/*.mjs"]).filter(
    (p) => p !== SELF_PATH,
  );
  const failures = [];
  for (const rel of files) {
    const text = fs.readFileSync(path.join(root, rel), "utf8");
    const violations = findUnguardedScanSets(text);
    for (const v of violations) failures.push({ file: rel, ...v });
  }
  return { ok: failures.length === 0, failures, scanned: files.length };
}

// ════════════════════════════════════════════════════════════════════════
// CHECK 2 (E8) — a retired workflow stage described as current
// ════════════════════════════════════════════════════════════════════════

// Exception set, exactly — never widened to make a finding disappear.
const RETIRED_STAGE_EXCEPTION_PREFIXES = ["docs/decisions/", "docs/history/"];
// The legacy-coercion code path itself: the one place a retired token is
// SUPPOSED to appear, because it is the record that makes old data readable.
// packages/bee is the canonical source; the shipped .bee/bin/lib mirror is
// generated from it, so both are the same code path in spirit.
const RETIRED_STAGE_EXCEPTION_FILES = new Set(["packages/bee/lib/state.mjs", ".bee/bin/lib/state.mjs"]);

// A match is exempt when retirement is acknowledged in the same breath —
// "retired", "deleted", "legacy", "superseded", "deprecated", "coerced" (and
// stems). This is what actually distinguishes "bee-validating is deleted
// (validation-diet D1)" (every live mention in this tree today) from the
// real E7 shape, "route proof obligations to bee-validating" (no such
// acknowledgment anywhere nearby) — a bare name-match cannot tell those
// apart, and this repo's own docs/knowledge and skills/references corpus
// legitimately discusses retired stages by name constantly while explaining
// what replaced them.
const RETIREMENT_MARKER_RE = /\b(retir\w*|delet\w*|legacy\w*|supersed\w*|deprecat\w*|coerc\w*)\b/i;
const MARKER_WINDOW_CHARS = 250;

/**
 * Reads LEGACY_PHASE_COERCIONS out of state.mjs's source text (never
 * imported — the map is module-private) and returns its keys: the phase
 * name(s) the live enum no longer carries. Deriving from this record, not a
 * name written into this suite, is what lets the check survive the next
 * phase retirement with zero edits here — the day a phase is folded away,
 * whoever does it adds a LEGACY_PHASE_COERCIONS entry (D13's own
 * precedent), and this check picks it up for free.
 */
function deriveRetiredStageTokens(root) {
  const stateSrc = fs.readFileSync(path.join(root, "packages", "bee", "lib", "state.mjs"), "utf8");
  const objMatch = stateSrc.match(/LEGACY_PHASE_COERCIONS\s*=\s*\{([^}]*)\}/);
  if (!objMatch) {
    throw new Error(
      "packages/bee/lib/state.mjs: LEGACY_PHASE_COERCIONS object not found — " +
        "the retired-stage check would derive zero tokens and assert nothing",
    );
  }
  const keyRe = /(?:['"]([A-Za-z0-9_-]+)['"]|([A-Za-z0-9_$]+))\s*:/g;
  const tokens = [];
  let km;
  while ((km = keyRe.exec(objMatch[1]))) tokens.push(km[1] || km[2]);
  const unique = [...new Set(tokens)];
  if (unique.length === 0) {
    throw new Error(
      "packages/bee/lib/state.mjs: LEGACY_PHASE_COERCIONS has no keys — " +
        "the retired-stage check would assert nothing",
    );
  }
  return unique;
}

/** The exception set, applied — factored out so the selftest can prove it directly, without needing a real git repo to derive candidates from. */
function filterRetiredStageExceptions(paths) {
  return paths.filter(
    (p) =>
      !RETIRED_STAGE_EXCEPTION_PREFIXES.some((prefix) => p.startsWith(prefix)) &&
      !RETIRED_STAGE_EXCEPTION_FILES.has(p),
  );
}

/**
 * Scan set: the CURRENT-behavior instructional surface — the files whose
 * job is to state what the system does today, as opposed to narrating how
 * it got there. skills/**\/SKILL.md and skills/**\/references/** are what an
 * agent actually reads to act (bee-executing's own read_first convention);
 * docs/knowledge/** and docs/specs/** are AGENTS.md's named "state layer";
 * AGENTS.md/CLAUDE.md are loaded every session. Deliberately excludes
 * narrative genres this repo uses constantly and legitimately to discuss
 * retired things by name while explaining the retirement — architecture
 * essays (docs/0*.md), orientation (README.md), creation changelogs
 * (skills/**\/CREATION-LOG.md), external research notes (docs/REFs/**),
 * and code/test corpora that legitimately exercise the legacy-coercion path
 * itself (packages/bee/tests/**, scripts/tests/**) — none of those claim to
 * describe CURRENT behavior by genre, so a bare name match there answers a
 * different question than E8 asks. Check 1 above already owns the
 * scripts/tests/**\/packages/bee/** surface for its own (unrelated) concern.
 */
function currentBehaviorScanSet(root) {
  const globs = [
    "skills/*/SKILL.md",
    "skills/*/references/*.md",
    "skills/*/references/**/*.md",
    "docs/knowledge/**/*.md",
    "docs/specs/**/*.md",
    "AGENTS.md",
    "CLAUDE.md",
  ];
  const files = trackedExistingFiles(root, globs);
  return filterRetiredStageExceptions(files);
}

/**
 * Finds occurrences of a retired stage token described as CURRENT: either
 * the `bee-<token>` skill-name form (routing/invoking a skill that no
 * longer exists) or a bare quoted phase literal `'<token>'`/`"<token>"`,
 * with no retirement marker within MARKER_WINDOW_CHARS on either side.
 */
function findCurrentBehaviorMentions(text, token) {
  const patterns = [new RegExp(`\\bbee-${token}\\b`, "g"), new RegExp(`['"]${token}['"]`, "g")];
  const hits = [];
  const seenIdx = new Set();
  for (const re of patterns) {
    let m;
    re.lastIndex = 0;
    while ((m = re.exec(text))) {
      if (seenIdx.has(m.index)) continue; // same span already caught by the other pattern
      const windowStart = Math.max(0, m.index - MARKER_WINDOW_CHARS);
      const windowEnd = Math.min(text.length, m.index + m[0].length + MARKER_WINDOW_CHARS);
      const window = text.slice(windowStart, windowEnd);
      if (RETIREMENT_MARKER_RE.test(window)) continue; // acknowledged as retired nearby — not a violation
      seenIdx.add(m.index);
      const line = text.slice(0, m.index).split("\n").length;
      hits.push({ line, phrase: m[0] });
    }
  }
  return hits.sort((a, b) => a.line - b.line);
}

/**
 * Checks a given scan set (never derives one itself — see currentBehaviorScanSet
 * for the real, git-backed derivation) for retired-stage-as-current mentions.
 * Parameterised over {root, scanSet} the same way test_doctrine_parity.mjs's
 * checkDoctrine() is, so the selftest can aim it at fixture files without a
 * real git repository backing them.
 */
function checkRetiredStageAsCurrent(root, scanSet) {
  const tokens = deriveRetiredStageTokens(root);
  const files = scanSet;
  const failures = [];
  for (const rel of files) {
    const text = fs.readFileSync(path.join(root, rel), "utf8");
    for (const token of tokens) {
      const hits = findCurrentBehaviorMentions(text, token);
      for (const h of hits) failures.push({ file: rel, token, ...h });
    }
  }
  return { ok: failures.length === 0, failures, scanned: files.length, tokens };
}

// ════════════════════════════════════════════════════════════════════════
// self-test — proves each classifier on synthetic fixtures, never this tree
// ════════════════════════════════════════════════════════════════════════

function writeFixture(root, rel, body) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, body);
}

function runSelftestCheck1() {
  // Violation: derives from git ls-files, reads with readFileSync, no
  // existsSync anywhere in the enclosing function.
  const violation = `
function loadAll(root) {
  const raw = execFileSync("git", ["ls-files", "-z"], { cwd: root }).toString("utf8");
  const files = raw.split("\\0").filter(Boolean);
  const out = [];
  for (const rel of files) {
    out.push(fs.readFileSync(path.join(root, rel), "utf8"));
  }
  return out;
}
`;
  const v = findUnguardedScanSets(violation);
  if (v.length !== 1) {
    console.error(`FAIL ${NAME} --selftest: check1 did not flag the unguarded violation fixture (got ${v.length} hits)`);
    return 1;
  }

  // Known-good shape A: chained `.filter(existsSync)` on the derived list
  // (test_doctrine_parity.mjs:127-138's shape).
  const goodChainedFilter = `
function deriveScanSet(root) {
  const raw = execFileSync("git", ["ls-files", "-z", "*.md"], { cwd: root }).toString("utf8");
  return raw
    .split("\\0")
    .filter(Boolean)
    .filter((p) => fs.existsSync(path.join(root, p)))
    .sort();
}
function readAll(root, scanSet) {
  return scanSet.map((rel) => fs.readFileSync(path.join(root, rel), "utf8"));
}
`;
  if (findUnguardedScanSets(goodChainedFilter).length !== 0) {
    console.error(`FAIL ${NAME} --selftest: check1 false-flagged the chained-filter known-good fixture`);
    return 1;
  }

  // Known-good shape B: per-item existsSync-then-continue guard immediately
  // before the read (test_installers_e2e.mjs:191-199's shape).
  const goodPerItemGuard = `
function stagedSource(root) {
  const files = execFileSync("git", ["-C", root, "ls-files", "-z"], {}).toString("utf8").split("\\0").filter(Boolean);
  for (const rel of files) {
    const from = path.join(root, rel);
    if (!fs.existsSync(from)) continue;
    fs.copyFileSync(from, path.join("/tmp/x", rel));
  }
}
`;
  if (findUnguardedScanSets(goodPerItemGuard).length !== 0) {
    console.error(`FAIL ${NAME} --selftest: check1 false-flagged the per-item-guard known-good fixture`);
    return 1;
  }

  // Known-good shape C: derives a list and never reads from disk with it at
  // all (onboard_bee.mjs's trackedGitignorePaths — used only for a message).
  // No guard needed because there is nothing to guard against.
  const goodNoRead = `
function trackedGitignorePaths(root) {
  const output = execFileSync("git", ["ls-files", "-z", "--", "foo"], { cwd: root }).toString("utf8");
  return output.split("\\0").filter(Boolean);
}
`;
  if (findUnguardedScanSets(goodNoRead).length !== 0) {
    console.error(`FAIL ${NAME} --selftest: check1 false-flagged the derives-but-never-reads known-good fixture`);
    return 1;
  }

  console.log(
    `PASS ${NAME} --selftest: check1 bites on an unguarded git-ls-files-then-read fixture, and spares ` +
      `the chained-filter, per-item-guard, and derives-but-never-reads known-good shapes`,
  );
  return 0;
}

function runSelftestCheck2() {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-scan-set-hygiene-selftest-"));
  try {
    // A fixture state.mjs carrying TWO retired tokens (not one) — proves
    // derivation is not silently pinned to a single hardcoded name.
    writeFixture(
      tmpRoot,
      "packages/bee/lib/state.mjs",
      "export const PHASES = ['idle', 'exploring'];\n" +
        "const LEGACY_PHASE_COERCIONS = { qualifying: 'exploring', staging: 'swarming' };\n",
    );
    const tokens = deriveRetiredStageTokens(tmpRoot);
    if (tokens.length !== 2 || !tokens.includes("qualifying") || !tokens.includes("staging")) {
      console.error(`FAIL ${NAME} --selftest: check2 token derivation got ${JSON.stringify(tokens)}, expected ["qualifying","staging"]`);
      return 1;
    }

    // Violation: a live skill reference routes to a retired skill with no
    // acknowledgment nearby — the exact E7 (research-brief-template.md) shape.
    const violationText =
      "Open questions (for the user, or as proof obligations for bee-qualifying):\n" +
      "- none yet\n";
    const violationHits = findCurrentBehaviorMentions(violationText, "qualifying");
    if (violationHits.length !== 1 || violationHits[0].line !== 1) {
      console.error(
        `FAIL ${NAME} --selftest: check2 did not flag the unacknowledged bee-<retired> reference ` +
          `(got ${JSON.stringify(violationHits)})`,
      );
      return 1;
    }

    // Known-good: the same reference, but retirement is acknowledged nearby
    // (the shape every live mention in this repo actually uses today).
    const goodText = "`bee-qualifying` is deleted (some-feature D1); route proof obligations to `bee-planning` instead.\n";
    if (findCurrentBehaviorMentions(goodText, "qualifying").length !== 0) {
      console.error(`FAIL ${NAME} --selftest: check2 false-flagged an acknowledged-as-deleted mention`);
      return 1;
    }

    // Known-good: a bare phase-literal fixture that IS the coercion test
    // itself, framed with "legacy" — must not be flagged.
    const coercionTestText = "assert(readState(dir).phase === 'planning', \"legacy phase 'staging' must coerce to planning\");\n";
    if (findCurrentBehaviorMentions(coercionTestText, "staging").length !== 0) {
      console.error(`FAIL ${NAME} --selftest: check2 false-flagged a legacy-acknowledged phase-literal fixture`);
      return 1;
    }

    // Exception-set fixture: the same violation shape, but under
    // docs/history/** — must be excluded from the scan set, not merely
    // spared by the marker window (this proves the path filter; the
    // marker-window proof above already covers the text classifier). No
    // real git repo is needed here — filterRetiredStageExceptions is the
    // exact function the real, git-backed currentBehaviorScanSet() also
    // calls, so proving it here proves the real path too.
    writeFixture(
      tmpRoot,
      "docs/history/some-feature/CONTEXT.md",
      "Route proof obligations to bee-qualifying (frozen history, quotes the retired routing verbatim).\n",
    );
    writeFixture(
      tmpRoot,
      "skills/bee-example/references/example.md",
      "Route proof obligations to bee-qualifying.\n", // the live violation this run must catch
    );
    const candidatePaths = ["docs/history/some-feature/CONTEXT.md", "skills/bee-example/references/example.md"];
    const scanSet = filterRetiredStageExceptions(candidatePaths);
    if (scanSet.some((p) => p.startsWith("docs/history/"))) {
      console.error(`FAIL ${NAME} --selftest: check2 exception filter did not exclude docs/history/** (exception set widened or broken)`);
      return 1;
    }
    if (!scanSet.includes("skills/bee-example/references/example.md")) {
      console.error(`FAIL ${NAME} --selftest: check2 exception filter dropped a live skills/**/references/**.md fixture it must keep`);
      return 1;
    }
    const result = checkRetiredStageAsCurrent(tmpRoot, scanSet);
    if (result.ok || !result.failures.some((f) => f.file === "skills/bee-example/references/example.md")) {
      console.error(
        `FAIL ${NAME} --selftest: check2's real-scan-set path did not name the offending file ` +
          `(result: ${JSON.stringify(result)})`,
      );
      return 1;
    }

    console.log(
      `PASS ${NAME} --selftest: check2 derives multiple retired tokens from LEGACY_PHASE_COERCIONS (never a ` +
        "hardcoded name), bites on an unacknowledged bee-<retired> reference and names the file, spares " +
        "retirement-acknowledged mentions, and excludes docs/history/** by the exception set rather than by luck",
    );
    return 0;
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

function runSelftest() {
  const c1 = runSelftestCheck1();
  if (c1 !== 0) return c1;
  const c2 = runSelftestCheck2();
  if (c2 !== 0) return c2;
  return 0;
}

// ─── main ─────────────────────────────────────────────────────────────────

function main() {
  const selftestOnly = process.argv.includes("--selftest");

  const selftestCode = runSelftest();
  if (selftestCode !== 0) return selftestCode;
  if (selftestOnly) return 0;

  let failed = false;

  const c1 = checkUnguardedScanSets(REPO_ROOT);
  if (!c1.ok) {
    failed = true;
    console.error(`FAIL ${NAME} (check 1 — unguarded git-index scan set): ${c1.failures.length} file(s):`);
    for (const f of c1.failures) {
      console.error(`      ${f.file}:${f.line} — ${f.reason}`);
    }
    console.error(
      "      Filter the derived list through fs.existsSync (chained on the list, or a per-item guard " +
        "before each read) in the same function that derives it — see test_doctrine_parity.mjs:127-138 " +
        "or test_installers_e2e.mjs:191-199.",
    );
  } else {
    console.log(`PASS ${NAME} (check 1): ${c1.scanned} file(s) under scripts/tests/** and packages/bee/** derive no unguarded git-index scan set`);
  }

  const c2 = checkRetiredStageAsCurrent(REPO_ROOT, currentBehaviorScanSet(REPO_ROOT));
  if (!c2.ok) {
    failed = true;
    console.error(
      `FAIL ${NAME} (check 2 — retired workflow stage described as current): ${c2.failures.length} mention(s):`,
    );
    for (const f of c2.failures) {
      console.error(`      ${f.file}:${f.line} — "${f.phrase}" (retired token "${f.token}") with no retirement acknowledgment nearby`);
    }
    console.error(
      "      Either the reference is stale (route it to whatever now carries that role) or the file needs " +
        "a retirement note nearby (\"retired\"/\"deleted\"/\"legacy\" — see README.md's or " +
        "skills/bee-hive/references/routing-and-contracts.md's phrasing for the live convention).",
    );
  } else {
    console.log(
      `PASS ${NAME} (check 2): ${c2.scanned} current-behavior file(s) describe none of [${c2.tokens.join(", ")}] as current ` +
        "(exception set: docs/decisions/**, docs/history/**, and the legacy-coercion code path)",
    );
  }

  return failed ? 1 : 0;
}

process.exit(main());
