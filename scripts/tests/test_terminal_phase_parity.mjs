#!/usr/bin/env node
// Terminal-phase membership parity gate — derived-check-hardening E5.
//
// The literal set ['idle', 'compounding-complete'] is written out BY HAND in
// six module-level declarations under three different names, none importing
// KNOWN_PHASES and none cross-checked against each other:
//   TERMINAL_PHASES      lib/guards.mjs, lib/compaction.mjs, lib/scratch.mjs
//   NO_WORK_PHASES        lib/inject.mjs, lib/intent.mjs
//   TERMINAL_LANE_PHASES  lib/recovery.mjs
// guards.mjs's copy governs write-denial; recovery.mjs's governs whether a
// finished lane gets nagged for resumption; the rest govern compaction and
// context injection. One missed edit on the next phase change produces three
// different wrong behaviours with nothing to announce it. See
// docs/knowledge/patterns/20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm.md
//
// E5 deliberately does NOT refactor the six to derive from KNOWN_PHASES —
// each copy carries its own semantics layered on the shared list, and that
// derivation is a separate, larger change. This suite is the cheap
// intermediate: assert the copies agree with each other and with
// KNOWN_PHASES, and on drift name the offending file:line.
//
// ─── locations are DERIVED, not hardcoded ──────────────────────────────────
// A hand-written list of "the six file:line pairs" would be this suite's own
// version of the exact defect it exists to catch (a copy nobody re-checks).
// What IS hardcoded below is much smaller and considerably more stable than
// coordinates: the three constant NAMES (a naming-convention fact stated by
// the pattern doc itself: "three names for one concept is the tell that no
// module owns it") and the two directory ROOTS the cell names explicitly —
// the canonical packages/bee/lib sources and their generated .bee/bin/lib
// twins. Within those roots every *.mjs file is scanned for a top-level
// `const <NAME> = new Set([...])` declaration matching one of the three
// names; the FILE and LINE each currently lives at are read off the match,
// every run, so a future rename-of-file or shift-of-line can never silently
// desync this suite from the tree the way a hand-copied coordinate list
// would. A genuinely new fourth name is the one shape this scan cannot
// discover on its own — see the report for how that was weighed.
//
// Usage:
//   node scripts/tests/test_terminal_phase_parity.mjs             # selftest, then real check
//   node scripts/tests/test_terminal_phase_parity.mjs --selftest  # selftest only

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const NAME = "test_terminal_phase_parity";

// The three hand-copied names for the same membership (pattern doc, above).
// Not a location — a domain fact about the current three-name split.
const MEMBERSHIP_NAMES = ["TERMINAL_PHASES", "NO_WORK_PHASES", "TERMINAL_LANE_PHASES"];

// The canonical source and its generated .bee/bin twin — exactly what the
// cell asks to check ("both the canonical packages/bee/ copies and their
// .bee/bin/ twins"). Both are flat directories (verified: no subdirectories
// under either today), so a non-recursive scan covers them fully.
const LIB_ROOTS = ["packages/bee/lib", ".bee/bin/lib"];

const DECL_RE = /^const\s+([A-Za-z_$][\w$]*)\s*=\s*new\s+Set\(\s*\[([^\]]*)\]\s*\)\s*;/gm;

function parseStringArrayLiteral(raw) {
  const items = [];
  const re = /'([^'\\]*)'|"([^"\\]*)"/g;
  let m;
  while ((m = re.exec(raw))) items.push(m[1] ?? m[2]);
  return items;
}

function lineOf(text, index) {
  return text.slice(0, index).split("\n").length;
}

/**
 * Scans one root directory (non-recursive) for top-level `const <NAME> = new
 * Set([...])` declarations whose NAME is one of `names`. Returns one entry
 * per match with the file (relative to `root`'s parent, i.e. repo-relative
 * when `absRoot` sits under the repo), the derived line number, the constant
 * name, and its parsed members.
 */
function scanRoot(absRoot, repoRelRoot, names) {
  let entries;
  try {
    entries = fs.readdirSync(absRoot, { withFileTypes: true });
  } catch (err) {
    throw new Error(`${repoRelRoot}: could not be scanned — ${err.message}`);
  }
  const found = [];
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith(".mjs")) continue;
    const absFile = path.join(absRoot, entry.name);
    const relFile = `${repoRelRoot}/${entry.name}`;
    const text = fs.readFileSync(absFile, "utf8");
    DECL_RE.lastIndex = 0;
    let m;
    while ((m = DECL_RE.exec(text))) {
      const [, declName, arrayBody] = m;
      if (!names.includes(declName)) continue;
      found.push({
        file: relFile,
        line: lineOf(text, m.index),
        name: declName,
        members: parseStringArrayLiteral(arrayBody),
      });
    }
  }
  return found;
}

function scanAllRoots(repoRoot, libRoots, names) {
  const declarations = [];
  const rootHits = new Map();
  for (const root of libRoots) {
    const hits = scanRoot(path.join(repoRoot, root), root, names);
    rootHits.set(root, hits.length);
    declarations.push(...hits);
  }
  return { declarations, rootHits };
}

const sortedUnique = (arr) => [...new Set(arr)].sort();
const sameMembers = (a, b) => {
  const sa = sortedUnique(a);
  const sb = sortedUnique(b);
  return sa.length === sb.length && sa.every((v, i) => v === sb[i]);
};

/**
 * Checks that every discovered declaration (a) has members that are all
 * genuine entries of `knownPhases`, and (b) has the same member set as the
 * first-discovered declaration (the reference). Reports every offending
 * declaration by file:line, never just "N of M disagree".
 */
function checkParity({ declarations, rootHits, knownPhases, libRoots }) {
  const problems = [];

  if (declarations.length === 0) {
    return {
      ok: false,
      problems: [`derived zero membership declarations across ${libRoots.join(", ")} — the gate would assert nothing (scan itself is broken, or every copy was removed)`],
      declarations,
    };
  }

  for (const root of libRoots) {
    if ((rootHits.get(root) ?? 0) === 0) {
      problems.push(`${root}: zero of the tracked names (${MEMBERSHIP_NAMES.join(", ")}) were found here — this root would go unchecked`);
    }
  }

  const reference = declarations[0];
  const unknownMembers = [];
  const drifted = [];

  for (const decl of declarations) {
    const strangers = decl.members.filter((p) => !knownPhases.includes(p));
    if (strangers.length > 0) {
      unknownMembers.push({ ...decl, strangers });
    }
    if (!sameMembers(decl.members, reference.members)) {
      drifted.push(decl);
    }
  }

  if (unknownMembers.length > 0) {
    for (const d of unknownMembers) {
      problems.push(
        `${d.file}:${d.line} — ${d.name} contains ${JSON.stringify(d.strangers)}, not in KNOWN_PHASES (packages/bee/lib/state.mjs)`,
      );
    }
  }

  if (drifted.length > 0) {
    for (const d of drifted) {
      problems.push(
        `${d.file}:${d.line} — ${d.name} = ${JSON.stringify(sortedUnique(d.members))} disagrees with ` +
          `${reference.file}:${reference.line} (${reference.name} = ${JSON.stringify(sortedUnique(reference.members))})`,
      );
    }
  }

  return { ok: problems.length === 0, problems, declarations, reference };
}

// ─── selftest: prove the checker bites, on fixture roots, never this tree ──

function writeFixtureModule(root, relPath, body) {
  const abs = path.join(root, relPath);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, body);
}

function runSelftest() {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-terminal-phase-parity-selftest-"));
  try {
    const knownPhases = ["idle", "planning", "executing", "compounding-complete"];

    // Fixture A — all six-style copies agree with each other and with
    // knownPhases. Must be reported ok.
    const rootA = path.join(tmpRoot, "a");
    writeFixtureModule(rootA, "canonical/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootA, "canonical/inject.mjs", "const NO_WORK_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootA, "twin/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootA, "twin/inject.mjs", "const NO_WORK_PHASES = new Set(['idle', 'compounding-complete']);\n");
    const scanA = scanAllRoots(rootA, ["canonical", "twin"], MEMBERSHIP_NAMES);
    const resultA = checkParity({ ...scanA, knownPhases, libRoots: ["canonical", "twin"] });
    if (!resultA.ok) {
      console.error(`FAIL ${NAME} --selftest: a fully-agreeing fixture was reported as drifted`);
      console.error(`      problems: ${JSON.stringify(resultA.problems)}`);
      return 1;
    }

    // Fixture B — one copy (twin/inject.mjs) drifts: missing
    // 'compounding-complete'. Must name exactly that file:line.
    const rootB = path.join(tmpRoot, "b");
    writeFixtureModule(rootB, "canonical/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootB, "canonical/inject.mjs", "const NO_WORK_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootB, "twin/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(
      rootB,
      "twin/inject.mjs",
      "// two leading comment lines\n// pushing the declaration down\nconst NO_WORK_PHASES = new Set(['idle']);\n",
    );
    const scanB = scanAllRoots(rootB, ["canonical", "twin"], MEMBERSHIP_NAMES);
    const resultB = checkParity({ ...scanB, knownPhases, libRoots: ["canonical", "twin"] });
    const namedTheDrift = resultB.problems.some((p) => p.startsWith("twin/inject.mjs:3 —"));
    if (resultB.ok || !namedTheDrift) {
      console.error(`FAIL ${NAME} --selftest: a deliberately drifted copy (twin/inject.mjs:3) was not named`);
      console.error(`      ok: ${resultB.ok}, problems: ${JSON.stringify(resultB.problems)}`);
      return 1;
    }

    // Fixture C — a copy carries a phase value that is not in knownPhases at
    // all (the vd-1/vd-2 class of bug: a retired/renamed phase left behind
    // in one hand-copy). Must name that file:line and the stray value.
    const rootC = path.join(tmpRoot, "c");
    writeFixtureModule(rootC, "canonical/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootC, "canonical/scratch.mjs", "const TERMINAL_PHASES = new Set(['idle', 'retired-phase']);\n");
    writeFixtureModule(rootC, "twin/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    const scanC = scanAllRoots(rootC, ["canonical", "twin"], MEMBERSHIP_NAMES);
    const resultC = checkParity({ ...scanC, knownPhases, libRoots: ["canonical", "twin"] });
    const namedTheStranger = resultC.problems.some(
      (p) => p.startsWith("canonical/scratch.mjs:1 —") && p.includes("retired-phase"),
    );
    if (resultC.ok || !namedTheStranger) {
      console.error(`FAIL ${NAME} --selftest: a stray phase value outside KNOWN_PHASES was not named`);
      console.error(`      ok: ${resultC.ok}, problems: ${JSON.stringify(resultC.problems)}`);
      return 1;
    }

    // Fixture D — a root that carries none of the tracked names at all must
    // be flagged as unchecked, not silently skipped.
    const rootD = path.join(tmpRoot, "d");
    writeFixtureModule(rootD, "canonical/guards.mjs", "const TERMINAL_PHASES = new Set(['idle', 'compounding-complete']);\n");
    writeFixtureModule(rootD, "twin/unrelated.mjs", "const SOMETHING_ELSE = new Set(['a', 'b']);\n");
    const scanD = scanAllRoots(rootD, ["canonical", "twin"], MEMBERSHIP_NAMES);
    const resultD = checkParity({ ...scanD, knownPhases, libRoots: ["canonical", "twin"] });
    const flaggedEmptyRoot = resultD.problems.some((p) => p.startsWith("twin:"));
    if (resultD.ok || !flaggedEmptyRoot) {
      console.error(`FAIL ${NAME} --selftest: a root contributing zero tracked declarations was not flagged`);
      console.error(`      ok: ${resultD.ok}, problems: ${JSON.stringify(resultD.problems)}`);
      return 1;
    }

    console.log(
      `PASS ${NAME} --selftest: bites on a drifted member set (names the exact file:line), on a member ` +
        `outside KNOWN_PHASES (names the stray value), and on a root contributing zero tracked declarations, ` +
        `while passing a fully-agreeing fixture`,
    );
    return 0;
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// ─── main ─────────────────────────────────────────────────────────────────

async function main() {
  const selftestOnly = process.argv.includes("--selftest");

  const selftestCode = runSelftest();
  if (selftestCode !== 0) return selftestCode;
  if (selftestOnly) return 0;

  let knownPhases;
  try {
    ({ KNOWN_PHASES: knownPhases } = await import(path.join(REPO_ROOT, "packages", "bee", "lib", "state.mjs")));
  } catch (err) {
    console.error(`FAIL ${NAME}: could not import KNOWN_PHASES from packages/bee/lib/state.mjs — ${err.message}`);
    return 1;
  }
  if (!Array.isArray(knownPhases) || knownPhases.length === 0) {
    console.error(`FAIL ${NAME}: packages/bee/lib/state.mjs exported an empty/invalid KNOWN_PHASES`);
    return 1;
  }

  const { declarations, rootHits } = scanAllRoots(REPO_ROOT, LIB_ROOTS, MEMBERSHIP_NAMES);
  const result = checkParity({ declarations, rootHits, knownPhases, libRoots: LIB_ROOTS });

  if (!result.ok) {
    console.error(`FAIL ${NAME}: the hand-copied terminal-phase memberships have drifted:`);
    for (const p of result.problems) console.error(`      ${p}`);
    console.error(
      `      guards.mjs's TERMINAL_PHASES governs write-denial and must never drift silently — see ` +
        `docs/knowledge/patterns/20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm.md`,
    );
    return 1;
  }

  console.log(
    `PASS ${NAME}: ${declarations.length} membership declaration(s) across ${MEMBERSHIP_NAMES.length} name(s) ` +
      `(${MEMBERSHIP_NAMES.join(", ")}) in ${LIB_ROOTS.join(", ")} agree with each other and with the ` +
      `${knownPhases.length}-entry KNOWN_PHASES enum: ${JSON.stringify(sortedUnique(result.reference.members))}`,
  );
  for (const d of declarations) {
    console.log(`      ${d.file}:${d.line} — ${d.name}`);
  }
  return 0;
}

main().then((code) => process.exit(code));
