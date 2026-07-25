#!/usr/bin/env node
// Parallel verify runner. Replaces the old sequential `&&`-chain in
// .bee/config.json `commands.verify` with a concurrency-capped promise pool,
// cutting wall-time (~90s sequential -> ~30s) without introducing flaky
// failures.
//
// Why not just run everything at unbounded concurrency: measured (see
// verify-parallel-runner-1 report) that unbounded -P16 concurrency is fast
// (~29s) but flaky — test_onboard_bee and test_claim_race intermittently
// fail under CPU/timing contention (the working tree stays clean after, so
// it is contention, not shared-state corruption). A handful of suites are
// timing/lock/fork-racer sensitive and must never be starved by unrelated
// concurrent suites; they are scheduled as ONE serial sub-chain so they
// never contend with each other, while still running in parallel with
// everything else (they hide under the long pole, test_onboard_bee).
//
// Each suite is spawned directly (no shell), stdout/stderr is buffered and
// only printed when a suite fails, so a green run stays quiet.

import { spawn, execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { buildRegistry, serializeRegistry, queryRegistry, normalizeQueryPath } from "./impact_registry.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..");
const IMPACT_REGISTRY_PATH = path.join(__dirname, "impact-registry.json");
const BEE_CONFIG_PATH = path.join(REPO_ROOT, ".bee", "config.json");

// test-economy D6: impacted-cap config reader. Key `verify_impacted_cap`
// (float 0-1) in .bee/config.json — a missing file, a parse failure, or a
// missing key all fail SAFE to the default 0.30 (measured against this
// repo's own registry: state.mjs's transitive closure sits at 36/109 ≈ 33%,
// just over this default, which is the whole point — see plan.md D6). A
// configured value of 1 is the documented off-switch (a ratio is never > 1).
const DEFAULT_IMPACTED_CAP = 0.3;

function readImpactedCap() {
  let raw;
  try {
    raw = fs.readFileSync(BEE_CONFIG_PATH, "utf8");
  } catch {
    return DEFAULT_IMPACTED_CAP;
  }
  let parsed;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return DEFAULT_IMPACTED_CAP;
  }
  const value = parsed?.verify_impacted_cap;
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return DEFAULT_IMPACTED_CAP;
  }
  return value;
}

// test-economy D6: best-effort CI nudge for the delegated transitive tail —
// client-side only, never touches .github/workflows/*. Missing/erroring `gh`
// fails open with a one-line note; it must never turn a green run red.
function triggerCiDelegation() {
  let result;
  try {
    result = spawnSync("gh", ["workflow", "run", "CI"], {
      cwd: REPO_ROOT,
      encoding: "utf8",
    });
  } catch (error) {
    result = { error };
  }
  if (!result || result.error || result.status !== 0) {
    console.log(
      "IMPACTED: best-effort `gh workflow run CI` did not succeed (gh missing or errored) — CI still picks up the delegated tail on its own cadence",
    );
  }
}

// ─── suite discovery (cs-4, contention-split) ──────────────────────────────
// This array used to be a manually maintained list: every feature adding a
// test suite had to edit these exact lines, making it a per-feature
// contention point (observed live: exec-xwh1 vs exec-cs3 collided here,
// 2026-07-20). Suites are now DISCOVERED by convention — glob a fixed set of
// directory roots for `test_*.mjs` files — so adding a new suite under one
// of these roots requires ZERO edits to this file.
//
// Roots are exactly the directories the old hand-written array drew suites
// from; nothing is lost (docs/history/contention-split/reports/cs-4.md
// records the old-vs-discovered set diff captured at flip time).
const DISCOVERY_ROOTS = [
  "scripts/tests",
  "packages/bee/tests",
  "skills/bee-hive/scripts",
  "packages/bee/hooks",
];

// `test_*.mjs` files under a discovery root that are NOT independent,
// standalone suites. Every entry needs a reason in its comment — an
// unexplained exclusion is a suite silently not run again, exactly the bug
// this discovery mechanism replaces the hand-written array to prevent.
const EXCLUDE = new Set([]);

// A handful of small scripts predate the `test_*.mjs` naming convention and
// were already part of commands.verify by hand; not worth renaming just to
// fit the glob, so they stay explicit extra entries alongside discovery.
const EXTRA_SUITES = [
  ["scripts/release_manifest.mjs", "--selftest"],
  ["scripts/release_manifest.mjs", "--check"],
  // lpsp-1: guards the third copy relationship — .bee/bin/** vs the
  // .bee/onboarding.json managed-hash ledger — so a release can never tag a
  // commit whose ledger was not refreshed by self-onboard (the drift:true
  // false-lead shipped in 1.9.0, hand-fixed in 6412017). Same idiom as
  // release_manifest.mjs --check just above.
  ["scripts/ledger_parity.mjs", "--check"],
  // i-1 (issues-46-53 D1/D2): docs/backlog.md's PBI ids have no allocator —
  // the id rule lives in prose and is executed by an agent hand-editing the
  // table. #49's live duplicate (two `P50` rows, authored a day apart on
  // different branches) proves a lock would prevent nothing; the missing
  // piece is this uniqueness check. Doesn't match the test_*.mjs discovery
  // glob, so EXTRA_SUITES membership is its only way into the chain — same
  // idiom as ledger_parity.mjs --check just above.
  ["scripts/backlog_uniqueness.mjs", "--check"],
  ["scripts/census_stale_spawn_syntax.mjs"],
  ["scripts/tests/test_installers_e2e.mjs", "--installer", "bash"],
  // okf-3: joins the chain as a chain-failing suite per D22/D34. Plain check
  // (NEVER --strict here — D8-graduation keeps profile warnings as warnings
  // until F2): docs/knowledge/ OKF errors fail the chain (exit 1), profile
  // warnings do not (exit 0, D13).
  [".bee/bin/bee.mjs", "knowledge", "check"],
  // okf-4: index freshness joins the chain per D21/D4 (stale generated
  // index). Read-only re-render + byte-diff against disk — the same idiom as
  // `bee decisions render --check`; a stale or missing generated index under
  // docs/knowledge/ fails the chain naming the file. No --strict here either.
  [".bee/bin/bee.mjs", "knowledge", "index", "--check"],
  // okf-5: the D35 coverage gate for the advisor-protocol migration — the
  // source anchor inventory, the pointer stub's anchor map
  // (docs/specs/advisor-protocol.md, D37), and the concepts' bee.sources
  // claims must stay in exact set-equality: every anchor owned by exactly one
  // concept, no loss, no duplication. Binary check; no --strict exists.
  // f2-1b (F8): the inventory is no longer a hand-authored constant — it is
  // DERIVED at check time from a content-addressed pin (commit + path +
  // blob_sha + scheme + expected_counts, all asserted, with a committed
  // verbatim source copy as the shallow-clone fallback). An empty, failed,
  // unresolvable, or unscheme'd extraction now exits 1 instead of reporting
  // 0/0 green. scripts/tests/test_okf_pins.mjs (auto-discovered) is what proves it.
  ["scripts/okf_migrate.mjs", "--check", "advisor-protocol"],
  // okf-6: the D35 coverage gate for critical-patterns.md's migration into
  // docs/knowledge/patterns/ — same coverage law, PATn anchors instead of
  // the BA-spec B*/R*/E*/P* shape (critical-patterns.md is a flat dated
  // pattern list, not a nine-section BA spec). Binary check; no --strict.
  ["scripts/okf_migrate.mjs", "--check-patterns"],
  // f2-3 (F6): doctrine-layer's own D35 coverage gate — the first area migrated
  // under the honest, derived, self-reporting pin. 39 anchors {B10, R17, E5, P7}
  // derived from blob 351bf72 at ed65720 with expected_counts.unparsed_blocks: 2
  // asserted, so the two id-less block starts this source really carries stay
  // VISIBLE instead of being invented into anchors (D10) or silently dropped.
  // A migration gate that exists but is not in the chain protects nothing after
  // its cell closes, so it lands here in the same commit as the stub.
  ["scripts/okf_migrate.mjs", "--check", "doctrine-layer"],
  // f2-5 (F6/F9): decision-memory's own D35 coverage gate. 9 anchors (0 B / 9 R
  // / 0 E / 0 P) derived from blob 2e8ec59 at 8710d03 with
  // expected_counts.unparsed_blocks: 0 asserted — this source's nine rules,
  // written `- **R1 — …**`, were derivable all along; the earlier "shapeless,
  // 0 anchors" verdict was an artifact of a classifier the f2-4 widening fixed.
  // One concept owns all nine (the source is one coherent topic at this size);
  // the F12 drift telemetry stayed inside its band once the concept count
  // matched the source's true size instead of splitting a 39-line spec by the
  // heading list.
  ["scripts/okf_migrate.mjs", "--check", "decision-memory"],
  // f2-6 (F6/F9): verify-pipeline's own D35 coverage gate. 14 anchors (0 B /
  // 5 R / 4 E / 5 P) derived from blob eab70d7 at 72fd828 with
  // expected_counts.unparsed_blocks: 7 asserted — this source's "Behaviors &
  // Operations" section carries 7 bold-lead bullets with no B-id at all, and
  // none is invented into an anchor (D10). Two concepts, split by TOPIC (how
  // suites are shaped/found vs how a run itself stays concurrency-safe and
  // hermetic) keep the F12 drift telemetry inside its band across the four
  // pinned "area"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "verify-pipeline"],
  // f2-7 (F6/F9): performance-log's own D35 coverage gate. 23 anchors (0 B /
  // 11 R / 5 E / 7 P) derived from blob efdc9f2 at 46a56a4 with
  // expected_counts.unparsed_blocks: 10 asserted — this source's entire
  // "Behaviors & Operations" section (7 bold-lead paragraphs plus 3 un-ided
  // Measurement-rules sub-bullets) carries no id at all, and none is
  // invented into an anchor (D10). Three concepts, split by TOPIC (the
  // operator-driven section lifecycle and its measurement rules; the
  // automatic persistent store and sync; the read-only cross-project
  // matrix) keep the F12 drift telemetry inside its band across the five
  // pinned "area"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "performance-log"],
  // f2-8 (F6/F9): feedback-digest's own D35 coverage gate. 29 anchors (0 B /
  // 15 R / 6 E / 8 P) derived from blob eeb447e at 3d69a2d with
  // expected_counts.unparsed_blocks: 26 asserted — the highest unparsed
  // ratio of the pinned areas so far: the entire "Behaviors & Operations"
  // section is five markdown subheadings (B1-B5) carrying no id at all, 18
  // unnumbered bold-lead paragraphs plus 8 un-ided continuation bullets,
  // none invented into an anchor (D10). Four concepts, split by TOPIC (the
  // digest's own data model; generating and refreshing a repository's own
  // digest; cross-repository collection and the trust boundary; ranking and
  // the self-improvement process) keep the F12 drift telemetry inside its
  // band across the six pinned "area"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "feedback-digest"],
  // f2-9 (F6/F9): onboarding's own D35 coverage gate. 58 anchors (0 B / 28 R /
  // 15 E / 15 P) derived from blob c78ca9b at a06f59d with
  // expected_counts.unparsed_blocks: 20 asserted — the largest area migrated so
  // far, and the one carrying the LETTER-SUFFIXED `R20b` that f2-4's widened
  // classifier reads and f2-3's widened stub-row parser and claim matcher can
  // match. The 20 unparsed blocks are the whole "Behaviors & Operations"
  // section: 16 unnumbered bold-lead paragraphs, the status-display lead
  // paragraph, and the ignore section's three un-ided continuation bullets,
  // none invented into an anchor (D10). Eight concepts, split by TOPIC (run
  // modes and actors; the opt-in status display; the managed ignore section;
  // distribution-source exclusivity; installer entry points and source
  // staging; release identity and version parity; repo-local guardrails; the
  // artifacts a host project keeps) keep the F12 drift telemetry inside its
  // band across the seven pinned "area"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "onboarding"],
  // f2-10 (F6/F9): hook-runtime's own D35 coverage gate, and the first pin
  // whose source had to be REPAIRED before it could be pinned at all. The
  // spec shipped the rule id `R14` TWICE — the gate-bypass block-verdict rule
  // and the write-guard command-shape rule, two genuinely different rules —
  // and because anchors are keyed by id, the first one's text was silently
  // overwritten by the second's: unmeasurable by the fidelity floor forever,
  // and invisible to a set-equality check as the pair's second member.
  // Neither rule was dropped or merged; the second occurrence in document
  // order was renumbered `R14a` in the source BEFORE the pin was captured, so
  // the pinned blob carries 81 anchors with 81 DISTINCT ids (22 B / 24 R /
  // 17 E / 18 P), unparsed_blocks: 8. Those repaired bytes are in no commit's
  // tree, so the pin declares repaired_from + repair_reason and is
  // content-addressed by its committed copy; the provenance blob at
  // ab8cf6ec:docs/specs/hook-runtime.md is still asserted exactly. Twelve
  // concepts, split by TOPIC (the cross-cutting frame; catalog, projections
  // and activation; write-guard request shapes; governed paths and the intake
  // gate; advisories and the one turn-control exception; delivery targets and
  // the fallback command; hook-source exclusivity; the dispatch guard; native
  // spawn and transport classification; child attribution and audit;
  // coordination refresh and session-init; health checks and proof surfaces)
  // keep the F12 drift telemetry inside its band across the eight pinned
  // "area"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "hook-runtime"],
  // f2-11 (F6/F9): worktree-parallelism's own D35 coverage gate — the one area
  // of the eleven that genuinely needed a THIRD anchor scheme. Every earlier
  // "shapeless" verdict was a blind reader (decision-memory's nine rules were
  // written `- **R1 — …**` and f2-4's widening found them), but this source
  // really does carry no `B*`/`R*`/`E*`/`P*` id and none of the four
  // anchor-bearing nine-section headings, so `ba-nine-section` derives 0
  // anchors AND 0 unparsed blocks from it. F9 forbids forcing it into the
  // nine-section shape and D10 forbids inventing numbered ids, so the anchors
  // are the source's OWN `## ` headings, slugified: `narrative-sections`, 10
  // anchors, unparsed_blocks: 2 (the `**Area:**` / `**Status:**` preamble
  // lines, which sit before the first heading and belong to no section —
  // reported, never invented into anchors). A `###` subheading is NOT an
  // anchor, a source with zero `## ` headings is REFUSED rather than passed
  // 0/0, and two headings that slugify alike are refused outright, so the
  // duplicate-id hazard f2-10 had to repair a source to escape cannot arise.
  // Seven concepts, split by TOPIC (purpose and boundary; the trust model;
  // entering — create and register; returning — the staged merge and its
  // verify gate; routing and visibility; the cross-worktree holds ledger;
  // store tiers and the implementation map). Its shape ratios are compared
  // only against other `narrative-sections` pins — there is one, so its F12
  // telemetry reports and never fails, exactly as flat-pattern-list's did
  // before it.
  ["scripts/okf_migrate.mjs", "--check", "worktree-parallelism"],
  // f2-13 (F6/F9/F10): workflow-state's own D35 coverage gate — the LAST area
  // of the migration, the largest by a wide margin (1464 lines, 140 anchors:
  // 37 B / 58 R / 25 E / 20 P, unparsed_blocks: 7), and the only one migrated
  // across SEVERAL cells rather than one commit (F10: f2-12 repaired and
  // pinned the source and authored the first cluster group; f2-13 authored the
  // remaining 136 anchors, wrote the stub, and wired this entry). Like
  // hook-runtime it is a REPAIRED pin, and worse: the source shipped `R19`,
  // `R20` AND `R21` twice each — the fresh-session-handoff triple and the
  // chain-integrity triple, six genuinely distinct rules — so 140 anchors
  // carried only 137 distinct ids while every declared count still added up.
  // The second occurrence of each id in document order was renumbered
  // `R19a`/`R20a`/`R21a` BEFORE the pin was captured; the pin declares
  // repaired_from + repair_reason and is content-addressed by its committed
  // copy (blob 506fef9), with the provenance blob ed1644c at
  // df3072d:docs/specs/workflow-state.md still asserted exactly.
  // Fifteen concepts, split by TOPIC (the cross-cutting frame; the gates of a
  // feature's life and the closing tail; review sessions and derived review
  // status; the unified command entry point; the worker adviser consult; unit
  // authoring and plan revision; the computed schedule and cycle refusal;
  // attempt history and lifetime budgets; completion teeth, judge verdicts and
  // the archive transaction; the two-kind handoff and the work puller; crash
  // detection and transcript recovery; atomic claims and claimed-unit
  // ownership; sessions, lanes and identity; cross-session holds and the
  // coordination lock; isolated worktrees and merge-back). D30 locked NINE
  // behavior clusters as F2's input, and nine is what the behaviors alone
  // want; once the 58 rules, 25 edge cases and 20 pointer bullets were
  // distributed to the concept each governs (D30's own "never a dumping
  // ground"), two of those clusters would have swallowed 35+ anchors apiece —
  // which is exactly the shape F12's anchors_per_concept ratio exists to
  // catch. Those two clusters are therefore split by topic INSIDE themselves,
  // keeping D30's map as the spine, and the F12 drift telemetry lands inside
  // its band across the nine pinned "ba-nine-section"-shaped sources.
  ["scripts/okf_migrate.mjs", "--check", "workflow-state"],
  // f3-5 (G6): okf-profile's own D35 coverage gate — the TWELFTH and last area
  // migration, and the only one whose source DEFINES the bundle it moved into,
  // graded by the profile's own loop and the profile's own gate. 24 anchors
  // (13 B — the REFINEMENT-suffixed B6b included / 0 R / 4 E / 7 P) derived
  // from blob 9267d3e at 53d8111, with expected_counts.unparsed_blocks: 17
  // asserted. `rules: 0` is MEASURED, not a gap: this source's nine `## Business
  // Rules` bullets carry no `R<n>` id at all, so D10 forbids inventing them and
  // they are counted unparsed instead — re-homed verbatim into the concept whose
  // subject each states, but never anchor-gated. Three of those 17 unparsed
  // blocks are the one hazard this source carries: the extractor does not track
  // markdown CODE FENCES, so the `bee.area` template's fenced `## Pointers
  // (implementation)` line opens a spurious accounting section over the
  // Templates prose. It moves no anchor — that stretch holds no top-level `- `
  // bullet, so P1-P7 are still the real Pointers section's seven — and pinning
  // 17 makes a future fence-aware extractor a loud failure rather than a silent
  // reshaping. Five concepts, split by TOPIC (purpose/entry points/actors; the
  // concept model and authoring; the conformance check; context and promote;
  // the migration loop and its gates).
  ["scripts/okf_migrate.mjs", "--check", "okf-profile"],
  // f3-4 (G2): `docs/specs/` is READ-ONLY for NEW content once a repo has a
  // knowledge bundle. Two entries, both required:
  //   --selftest  proves the fence actually BITES — the bundle-ful fixture
  //               with a new prose file fails, the bundle-LESS fixture with
  //               the SAME file stays silent (G1: a host repo that never
  //               migrated keeps writing docs/specs/ freely and must not be
  //               able to tell this release happened), stub recognition is
  //               structural (`migrated_to` frontmatter, never a filename
  //               list that rots the first time an area is added), and the
  //               `system-overview.md` placeholder allowlist stops applying
  //               the moment the file is actually written.
  //   (bare)      runs the fence against THIS repo's live docs/specs/.
  // Deliberately NOT in the LIVE_BUNDLE_GROUP below: test_okf_pins' mutator
  // writes an unhealthy concept under docs/knowledge/, which the fence never
  // reads — it only asks `bundleMode` (still true) and lists docs/specs/,
  // which no suite in the chain mutates.
  ["scripts/okf_specs_fence.mjs", "--selftest"],
  ["scripts/okf_specs_fence.mjs"],
  // f4-6: the fence over the INSTRUCTION layer — the sibling of the one above,
  // and the one nothing in this chain had. `okf_specs_fence` stops new prose
  // landing in the retired TREE; this one stops the skills, hooks and AGENTS.md
  // from TEACHING the retired tree with no bundle branch. Three hand audits
  // each found instruction-layer misroutes the previous audit missed (7, then
  // 1+1, then 6) and the chain stayed green through all three, because nothing
  // in it read prose. Two entries, both required:
  //   --selftest  proves it BITES — an unbranched line in a bundle-ful fixture
  //               fails while the SAME line in a bundle-LESS fixture stays
  //               silent (a never-migrated host cannot tell this shipped), the
  //               rule is LINE-local rather than file-local (a file-level rule
  //               was written first and scored GREEN on four of the six known
  //               misroutes), and all four exemption classes are exercised:
  //               branched line, legacy anchor citation, historical record,
  //               fenced example.
  //   (bare)      runs the fence against THIS repo's live instruction surfaces.
  // Deliberately NOT in the LIVE_BUNDLE_GROUP below, for the same reason as the
  // specs fence: it only asks `bundleMode` (still true under test_okf_pins'
  // unhealthy-concept mutator) and then reads skills/, hooks/ and AGENTS.md,
  // which no suite in the chain mutates.
  ["scripts/okf_instructions_fence.mjs", "--selftest"],
  ["scripts/okf_instructions_fence.mjs"],
];

// scripts/tests/test_installers_e2e.mjs is discovered by the glob too (it
// matches `test_*.mjs`); its args variant is supplied via EXTRA_SUITES above,
// so the bare no-args discovery hit for this one path is dropped to avoid
// running it twice.
const ARGS_OVERRIDE = new Set(["scripts/tests/test_installers_e2e.mjs"]);

function discoverSuites() {
  const found = [];
  for (const root of DISCOVERY_ROOTS) {
    const dir = path.join(REPO_ROOT, root);
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const entry of entries) {
      if (!entry.isFile()) continue;
      if (!entry.name.startsWith("test_") || !entry.name.endsWith(".mjs")) continue;
      const rel = `${root}/${entry.name}`;
      if (EXCLUDE.has(rel) || ARGS_OVERRIDE.has(rel)) continue;
      found.push([rel]);
    }
  }
  found.sort((a, b) => a[0].localeCompare(b[0]));
  return [...found, ...EXTRA_SUITES];
}

export const SUITES = discoverSuites();

// Timing/lock/fork-racer suites: measured flaky under concurrent CPU
// contention with other suites (not with each other). Run as ONE sequential
// scheduling unit so they never overlap each other, while that unit still
// runs concurrently with everything else in the pool.
//
// Membership is convention-based: a suite whose filename ends in `_race.mjs`,
// `_lock.mjs`, or `_concurrency.mjs` is serial-sensitive by construction. A
// small number of pre-existing serial suites don't match that naming
// convention; they are listed explicitly below.
const SERIAL_NAME_PATTERN = /_(race|lock|concurrency)\.mjs$/;
const SERIAL_EXCEPTIONS = new Set([
  "scripts/tests/test_heartbeat_touch.mjs",
]);

const SERIAL_SENSITIVE = new Set(
  SUITES.map((entry) => entry[0]).filter(
    (p) => SERIAL_NAME_PATTERN.test(p) || SERIAL_EXCEPTIONS.has(p),
  ),
);

function suiteLabel(entry) {
  return [entry[0], ...entry.slice(1)].join(" ");
}

// ─── the live-bundle group (f2-10) ──────────────────────────────────────────
//
// SERIAL_SENSITIVE above is about CPU contention: those suites must not
// overlap EACH OTHER, but overlapping the rest of the pool is harmless. This
// group is about shared mutable STATE, which needs the opposite property —
// its members must not overlap each other because one of them deliberately
// mutates a file the others read.
//
// `scripts/tests/test_okf_pins.mjs` section 22 proves the coverage gate's bundle
// invariants are actually WIRED, end to end, by writing one deliberately
// non-canonical concept into the REAL docs/knowledge/ bundle, asserting the
// real CLI turns red, and removing it again. That is the right test — an
// isolated copy would prove the invariants are computed, not that they gate —
// but for the length of that window every other suite that reads the live
// bundle sees an unhealthy, unindexed bundle and fails for a reason that has
// nothing to do with what it is testing. Observed exactly that way: two
// `okf_migrate --check <area>` suites went red naming
// `zz-f2-2-unhealthy-fixture.md` and three stale indexes, then the identical
// tree passed on the next run. A ~50/50 chain is worse than a red one — it
// teaches everyone to re-run instead of to read.
//
// So the mutator and every live-bundle reader run as ONE sequential unit.
// They still run concurrently with the ~60 suites that touch no bundle at
// all, so the wall clock is unchanged in practice. Membership is derived
// from what a suite actually runs, never hand-listed: the coverage gate, the
// pin suite, and the bundle's own check/index verbs.
function touchesLiveBundle(entry) {
  const [cmd, ...args] = entry;
  if (cmd === "scripts/okf_migrate.mjs") return true; // every --check <area>
  if (cmd === "scripts/tests/test_okf_pins.mjs") return true; // the deliberate mutator
  if (cmd === ".bee/bin/bee.mjs" && args[0] === "knowledge") return true; // check / index --check
  return false;
}

const LIVE_BUNDLE_GROUP = new Set(SUITES.filter(touchesLiveBundle).map(suiteLabel));

// Skip-marker convention (hardening-8, loud canary skip): a suite that
// self-skips its real work (e.g. scripts/canary_codex.mjs's no-codex-binary
// path) still exits 0 — the correct exit code, since an absent optional
// binary is an environment fact, never a failure — but that makes it read as
// an ORDINARY PASS once buried among dozens of other suites in this
// runner's summary: "ran and proved something" and "ran nothing" become
// indistinguishable. A suite opts in by printing one line matching this
// pattern to stdout; the summary loop below then annotates that suite's PASS
// line with the skip reason instead of silently folding it in. Exit codes
// are NEVER touched by this — only the printed line gains a note.
export const SKIP_MARKER_RE = /^CANARY_SKIP\s+(.*)$/m;

export function skipNote(stdout) {
  const m = SKIP_MARKER_RE.exec(stdout || "");
  return m ? m[1].trim() : null;
}

// Hermeticity (hardening-1-7-10 D1; BEE_AGENT_NAME closed in okf-integration-
// close-f4 f4-4): a suite must never inherit the calling harness's own
// identity — THREE variables, not two. CLAUDE_CODE_SESSION_ID and
// BEE_SESSION_ID are set in THIS process's env whenever run_verify.mjs runs
// from inside a live Claude Code or bee session, and would otherwise leak
// into every spawned child, silently changing sessionless-path behavior
// (resolveSessionId's env fallback) between "run locally from a live
// session" and "run in CI with no such env at all". BEE_AGENT_NAME is the
// third: AGENTS.md critical rule 5 mandates prefixing write-heavy shell
// commands with `BEE_AGENT_NAME=<name>` during swarms, so a worker that
// obeys that rule and then runs the configured verify command inherits its
// own agent name into every child suite — inside checkWrite's cross-session
// hold branch that leaked name becomes the acting agent identity, and "the
// acting session's own hold must never block its own write" flips to a
// false red purely because the harness, not the guard, leaked. Every child
// suite gets a scrubbed copy so local runs match CI byte-for-byte and
// survive rule 5's own prefix, regardless of the parent's own identity.
function childEnv() {
  const env = { ...process.env };
  delete env.CLAUDE_CODE_SESSION_ID;
  delete env.BEE_SESSION_ID;
  delete env.BEE_AGENT_NAME;
  return env;
}

// Windows CI (hardening-1-7-10 D1): rather than hand-maintain a second suite
// list in .github/workflows/windows.yml, that job reuses THIS file's own
// discovery, restricted to one root via BEE_VERIFY_ROOT_FILTER — a suite
// added under that root is picked up here automatically, same as it already
// is for the unfiltered run. Unset (the default, every non-Windows caller):
// zero behavior change, byte-identical to before this existed.
function filterSuitesByRoot(suites) {
  const rootFilter = process.env.BEE_VERIFY_ROOT_FILTER;
  if (!rootFilter) return suites;
  const prefix = rootFilter.endsWith('/') ? rootFilter : `${rootFilter}/`;
  return suites.filter((entry) => entry[0].startsWith(prefix));
}

// Per-run exclusion (rel1710rc-2): a small, comma-separated list of exact
// repo-relative suite paths to drop from THIS invocation's active run,
// without touching the discovered SUITES export or the global (compile-time)
// EXCLUDE set above — those stay the single source every platform/caller
// sees, so a suite excluded here for one CI lane never silently vanishes
// from test_verify_manifest.mjs's mandatory/floor checks (which read the raw
// SUITES export, unaffected by this env var) or from any other caller's
// unfiltered run. Exists so .github/workflows/windows.yml can drop suites
// with a genuine, named, honestly-documented Windows-only failure (see that
// file's own loud comment block) without hand-maintaining a second suite
// list there — new suites under the discovery roots are still picked up
// automatically; only the named exclusions are ever skipped, and only in the
// job that sets this env var. Unset (the default, every non-Windows caller):
// zero behavior change, byte-identical to before this existed.
function filterExcludedSuites(suites) {
  const raw = process.env.BEE_VERIFY_EXCLUDE;
  if (!raw) return suites;
  const excluded = new Set(
    raw
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean),
  );
  if (excluded.size === 0) return suites;
  return suites.filter((entry) => !excluded.has(entry[0]));
}

// Scoped include filter (vs-1): a repeatable/comma-separated `--only <token>`
// CLI flag plus a BEE_VERIFY_ONLY env var let a caller run a narrow subset of
// suites while iterating — full verify is still required before close, which
// is why every scoped run prints a loud banner rather than blending in with
// an ordinary green summary. CLI wins when both are set (parseOnlyArgs is
// only even consulted first). A token matches a runnable by case-insensitive
// substring against EITHER its repo-relative path OR its display label
// (suiteLabel), so both glob-discovered suites and EXTRA_SUITES entries with
// trailing args (e.g. "okf_migrate --check onboarding") are reachable by a
// short token. Applied BEFORE filterExcludedSuites so BEE_VERIFY_EXCLUDE can
// still narrow the included set, never widen it. Unset (the default, every
// caller today): zero behavior change, byte-identical to before this
// existed.
function parseOnlyArgs(argv) {
  const tokens = [];
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] !== "--only") continue;
    const value = argv[i + 1];
    if (!value) continue;
    for (const raw of value.split(",")) {
      const token = raw.trim();
      if (token) tokens.push(token);
    }
  }
  return tokens;
}

function resolveOnlyTokens() {
  const cliTokens = parseOnlyArgs(process.argv.slice(2));
  if (cliTokens.length > 0) return cliTokens; // CLI wins over env when both are set
  const raw = process.env.BEE_VERIFY_ONLY;
  if (!raw) return [];
  return raw
    .split(",")
    .map((t) => t.trim())
    .filter(Boolean);
}

export function filterSuitesByOnly(suites, tokens) {
  if (tokens.length === 0) return suites; // true no-op: same array reference
  const lowerTokens = tokens.map((t) => t.toLowerCase());
  return suites.filter((entry) => {
    const relPath = entry[0].toLowerCase();
    const label = suiteLabel(entry).toLowerCase();
    return lowerTokens.some((t) => relPath.includes(t) || label.includes(t));
  });
}

function printScopedBanner(selected, total) {
  console.log(
    `SCOPED RUN (--only): ${selected} of ${total} runnables selected — full verify still required before close`,
  );
}

// ─── per-suite timeout + heartbeat (i54-closeout-2, D2) ────────────────────
// runOne() used to spawn a suite with no upper bound: a hung child ran
// forever with zero visible signal beyond "the pool never finished". Two
// independent behaviors close that gap without touching the hermetic env
// scrub (childEnv(), unchanged below) or any pass/fail semantics for
// non-timeout suites: (a) a per-suite wall-clock timeout that kills the
// WHOLE child process group (not just the direct child — a suite may have
// spawned its own grandchildren) and reports a distinct TIMEOUT failure,
// never conflated with an ordinary red, never a silent hang; (b) a
// heartbeat line to STDERR (never stdout, so machine-readable summary
// output stays clean) naming whichever suites are still in flight, so a
// human or agent watching a long run never mistakes progress for a freeze.
const DEFAULT_SUITE_TIMEOUT_MS = 300000;
const DEFAULT_HEARTBEAT_MS = 30000;

// BEE_VERIFY_SUITE_TIMEOUT_MS: "0" or "none" (case-insensitive) disables the
// timeout outright; unset, blank, or unparseable falls back to the 300s
// default rather than silently running unbounded.
export function resolveSuiteTimeoutMs() {
  const raw = process.env.BEE_VERIFY_SUITE_TIMEOUT_MS;
  if (raw === undefined) return DEFAULT_SUITE_TIMEOUT_MS;
  const trimmed = raw.trim();
  if (trimmed === "") return DEFAULT_SUITE_TIMEOUT_MS;
  const lower = trimmed.toLowerCase();
  if (lower === "0" || lower === "none") return 0;
  const n = Number(trimmed);
  return Number.isFinite(n) && n >= 0 ? n : DEFAULT_SUITE_TIMEOUT_MS;
}

// BEE_VERIFY_HEARTBEAT_MS: cadence override, defaulting to ~30s; unset,
// blank, zero, negative, or unparseable falls back to the default rather
// than spinning a zero-delay interval.
export function resolveHeartbeatMs() {
  const raw = process.env.BEE_VERIFY_HEARTBEAT_MS;
  if (raw === undefined) return DEFAULT_HEARTBEAT_MS;
  const trimmed = raw.trim();
  if (trimmed === "") return DEFAULT_HEARTBEAT_MS;
  const n = Number(trimmed);
  return Number.isFinite(n) && n > 0 ? n : DEFAULT_HEARTBEAT_MS;
}

// In-flight tracker: shared between runOne/runSerialGroup (which start/end
// entries as they actually run) and the heartbeat interval (which only ever
// reads a snapshot). Kept as its own tiny object — injectable and directly
// testable without needing the full pool/process.exit machinery of
// runSelectedSuites.
export function createInFlightTracker() {
  const inFlight = new Map();
  return {
    start(label) {
      inFlight.set(label, Date.now());
    },
    end(label) {
      inFlight.delete(label);
    },
    snapshot() {
      return [...inFlight.entries()];
    },
  };
}

// Prints one HEARTBEAT line to stderr per tick naming every suite still in
// flight plus its own elapsed time and the run's total elapsed time. Silent
// on a tick where nothing is in flight (e.g. right at pool start) — never a
// blank/noise line. `intervalMs`/`wallStart`/`log` are all injectable so a
// test can assert emission on a short interval without an unconditional 30s
// sleep; `intervalMs <= 0` disables the heartbeat entirely (returns null,
// nothing to clear). Default `log` is `console.error` — stderr, never
// stdout, so machine-readable summary output is never corrupted.
export function startHeartbeat(tracker, { intervalMs = resolveHeartbeatMs(), wallStart = Date.now(), log = console.error } = {}) {
  if (!(intervalMs > 0)) return null;
  const timer = setInterval(() => {
    const active = tracker.snapshot();
    if (active.length === 0) return;
    const now = Date.now();
    const totalS = ((now - wallStart) / 1000).toFixed(1);
    const names = active
      .map(([label, startedAt]) => `${label} (${((now - startedAt) / 1000).toFixed(1)}s)`)
      .join(", ");
    log(`HEARTBEAT: ${active.length} suite(s) still running after ${totalS}s: ${names}`);
  }, intervalMs);
  if (typeof timer.unref === "function") timer.unref();
  return timer;
}

export function stopHeartbeat(timer) {
  if (timer) clearInterval(timer);
}

export function runOne(entry, opts = {}) {
  const [script, ...args] = entry;
  const label = suiteLabel(entry);
  const start = Date.now();
  const timeoutMs = opts.timeoutMs !== undefined ? opts.timeoutMs : resolveSuiteTimeoutMs();
  const tracker = opts.tracker;
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [script, ...args], {
      cwd: REPO_ROOT,
      env: childEnv(),
      // detached: true makes this child the leader of its own process group
      // (POSIX) so a timeout can kill the group — the child plus any
      // grandchildren it spawned — instead of leaving orphans running past
      // the kill.
      detached: true,
    });
    tracker?.start(label);
    let stdout = "";
    let stderr = "";
    let settled = false;
    let timedOut = false;
    let timer = null;

    if (timeoutMs > 0) {
      timer = setTimeout(() => {
        timedOut = true;
        try {
          // Negative pid = kill the whole process group, not just this pid.
          process.kill(-child.pid, "SIGKILL");
        } catch {
          try {
            child.kill("SIGKILL");
          } catch {
            // process already gone; nothing left to kill.
          }
        }
      }, timeoutMs);
    }

    child.stdout.on("data", (d) => {
      stdout += d;
    });
    child.stderr.on("data", (d) => {
      stderr += d;
    });
    child.on("close", (code) => {
      if (settled) return;
      settled = true;
      if (timer) clearTimeout(timer);
      tracker?.end(label);
      const ms = Date.now() - start;
      const finalCode = timedOut ? 124 : code;
      if (timedOut) {
        stderr += `\nTIMEOUT after ${timeoutMs}ms — suite killed\n`;
      }
      resolve({
        label,
        code: finalCode,
        ms,
        stdout,
        stderr,
        timedOut,
        timeoutMs,
      });
    });
  });
}

// Run a group of suite entries strictly one after another; return their
// individual results. Used for the SERIAL_SENSITIVE unit so those suites
// never overlap each other, even though the group as a whole runs
// concurrently with other pool units. `opts` (tracker, timeoutMs) passes
// straight through to each runOne() call.
async function runSerialGroup(entries, opts = {}) {
  const results = [];
  for (const entry of entries) {
    // eslint-disable-next-line no-await-in-loop
    results.push(await runOne(entry, opts));
  }
  return results;
}

// Concurrency-capped promise pool. `units` is an array of functions, each
// returning a Promise<result[]> (a single suite wraps its one result in an
// array so serial and parallel units share the same shape).
async function runPool(units, concurrency) {
  const results = [];
  let next = 0;
  async function worker() {
    for (;;) {
      const i = next;
      next += 1;
      if (i >= units.length) return;
      // eslint-disable-next-line no-await-in-loop
      const unitResults = await units[i]();
      results.push(...unitResults);
    }
  }
  const workers = [];
  const workerCount = Math.min(concurrency, units.length);
  for (let w = 0; w < workerCount; w += 1) {
    workers.push(worker());
  }
  await Promise.all(workers);
  return results;
}

// ─── Impacted-file selection (cov-2, ci-owned-verify CONTEXT.md D4) ────────
// A repeatable/comma-separated `--impacted <file...>` CLI flag plus
// `--impacted-from-git` (staged + unstaged + untracked tree changes, UNION
// committed changes since the merge-base with main/origin-main) select
// suites through the impact registry (scripts/impact_registry.mjs) instead
// of running the full pool. Mutually exclusive with --only/BEE_VERIFY_ONLY
// (typed refusal, mirroring the zero-match refusal idiom above) — the two
// selection modes answer different questions and silently combining them
// would narrow one against the other with no signal. A changed test_*.mjs
// suite always selects itself: the registry's own closure walk includes
// each suite's own entry file (closureFor visits the entry before any
// edge), so no special-casing is needed here — plain registry lookup
// already covers it.

function parseImpactedArgs(argv) {
  const files = [];
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] !== "--impacted") continue;
    const value = argv[i + 1];
    if (!value) continue;
    for (const raw of value.split(",")) {
      const f = raw.trim();
      if (f) files.push(f);
    }
  }
  return files;
}

function hasImpactedFromGitFlag(argv) {
  return argv.includes("--impacted-from-git");
}

// impacted-level1 D1: `--level 1` narrows impacted-mode selection to DIRECT
// edges only (files the suite's own source references), never the full
// transitive closure --impacted/--impacted-from-git use by default. Only
// `1` is a recognized value; main() refuses both a bare `--level` (used
// without --impacted/--impacted-from-git) and an unrecognized value outright
// rather than silently falling back to full selection.
function getLevelFlagValue(argv) {
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--level") return argv[i + 1];
  }
  return undefined;
}

// git plumbing is best-effort: not a git repo, no `main`/`origin/main` ref,
// or a detached HEAD with no merge-base all yield an empty contribution
// from that half rather than crashing the run.
function runGitLines(args) {
  try {
    const out = execFileSync("git", args, { cwd: REPO_ROOT, encoding: "utf8" });
    return out.split("\n").filter((l) => l.length > 0);
  } catch {
    return [];
  }
}

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

// `git status --porcelain` lines are "XY <path>" (2-char status + space) or
// "XY <path> -> <path>" for renames, where the destination is what changed
// under its new name. Untracked files carry the `??` code and are included
// — a brand-new file is exactly the kind of change --impacted-from-git
// exists to catch. This covers staged AND unstaged AND untracked in one
// pass, so a staged-but-uncommitted merge (the exact state `bee worktree
// merge` gates in) yields its full changed-file set.
function statusPorcelainFiles() {
  const files = [];
  for (const line of runGitLines(["status", "--porcelain"])) {
    const rest = line.slice(3);
    const arrowIdx = rest.indexOf(" -> ");
    const rawPath = arrowIdx === -1 ? rest : rest.slice(arrowIdx + 4);
    const cleaned = unquoteGitPath(rawPath);
    if (cleaned) files.push(cleaned);
  }
  return files;
}

function resolveMergeBaseCommit() {
  for (const ref of ["main", "origin/main"]) {
    const out = runGitLines(["merge-base", "HEAD", ref]);
    if (out.length > 0 && out[0]) return out[0];
  }
  return null;
}

// Committed-changes half of the union: every file touched by a commit made
// since this branch diverged from main/origin-main, independent of the
// working tree's current dirty state.
function committedDiffFiles() {
  const base = resolveMergeBaseCommit();
  if (!base) return [];
  return runGitLines(["diff", "--name-only", base, "HEAD"]).map(unquoteGitPath);
}

function gitImpactedFiles() {
  const files = new Set();
  for (const f of statusPorcelainFiles()) files.add(f);
  for (const f of committedDiffFiles()) files.add(f);
  return [...files];
}

// Registry-backed EXACT selection (must-have: never the --only substring
// matcher — a suite label used as a substring token could over-match a
// different suite whose label happens to contain it).
export function filterSuitesByLabels(suites, labelSet) {
  if (labelSet.size === 0) return [];
  return suites.filter((entry) => labelSet.has(suiteLabel(entry)));
}

// "load scripts/impact-registry.json when fresh; on staleness rebuild in
// memory with a one-line warning" (D4/cov-2 action): the freshness check
// itself requires recomputing the registry (same idiom as
// impact_registry.mjs --check), so this always rebuilds and byte-compares;
// build cost is ~130ms (cov-1 report) — negligible next to a suite run.
// Missing-on-disk counts as stale, same as --check.
async function loadImpactRegistry() {
  let onDisk = null;
  try {
    onDisk = fs.readFileSync(IMPACT_REGISTRY_PATH, "utf8");
  } catch {
    onDisk = null;
  }
  const registry = await buildRegistry();
  const fresh = serializeRegistry(registry);
  if (onDisk !== fresh) {
    console.log(
      `IMPACTED: scripts/impact-registry.json is ${onDisk === null ? "missing" : "stale"} — rebuilt in memory for this run (regen: node scripts/impact_registry.mjs --write)`,
    );
  }
  return registry;
}

function printImpactedUnmapped(unmapped) {
  for (const u of unmapped) {
    console.error(`UNMAPPED: ${u} (no known suite relates to this file — full verify still covers it)`);
  }
}

function printImpactedBanner(selectedCount, changedCount, level) {
  if (level === 1) {
    console.log(`IMPACTED RUN (level 1): ${selectedCount} direct suite(s) from ${changedCount} changed file(s)`);
    return;
  }
  console.log(`IMPACTED RUN: ${selectedCount} suite(s) from ${changedCount} changed file(s)`);
}

// Shared execution + reporting tail, used by both the normal (full/--only)
// path and the impacted path: build the serial/bundle/parallel units for
// whatever suite set was selected, run the pool, print PASS/FAIL lines and
// the summary, then exit. `beforeBanner`/`afterBanner` are the mode-specific
// scoped-run banners (--only's or --impacted's) — null for an unscoped run.
async function runSelectedSuites(activeSuites, { concurrency, beforeBanner, afterBanner }) {
  const serialEntries = activeSuites.filter((entry) => SERIAL_SENSITIVE.has(entry[0]));
  const rest = activeSuites.filter((entry) => !SERIAL_SENSITIVE.has(entry[0]));
  const bundleEntries = rest.filter((entry) => LIVE_BUNDLE_GROUP.has(suiteLabel(entry)));
  const parallelEntries = rest.filter((entry) => !LIVE_BUNDLE_GROUP.has(suiteLabel(entry)));

  const tracker = createInFlightTracker();

  const units = [];
  if (serialEntries.length > 0) {
    units.push(() => runSerialGroup(serialEntries, { tracker }));
  }
  if (bundleEntries.length > 0) {
    units.push(() => runSerialGroup(bundleEntries, { tracker }));
  }
  for (const entry of parallelEntries) {
    units.push(() => runOne(entry, { tracker }).then((r) => [r]));
  }

  if (beforeBanner) beforeBanner();

  const wallStart = Date.now();
  const heartbeatTimer = startHeartbeat(tracker, { wallStart });
  const results = await runPool(units, concurrency);
  stopHeartbeat(heartbeatTimer);
  const wallMs = Date.now() - wallStart;

  results.sort((a, b) => a.label.localeCompare(b.label));

  let anyFail = false;
  for (const r of results) {
    const status = r.timedOut ? "TIMEOUT" : r.code === 0 ? "PASS" : "FAIL";
    if (r.code !== 0) anyFail = true;
    let note = "";
    if (status === "PASS") {
      const skip = skipNote(r.stdout);
      if (skip) note = `  [SKIPPED: ${skip}]`;
    } else if (status === "TIMEOUT") {
      note = `  [after ${r.timeoutMs}ms]`;
    }
    console.log(`${status}  ${String(r.ms).padStart(6)}ms  ${r.label}${note}`);
  }

  const failed = results.filter((r) => r.code !== 0);
  if (failed.length > 0) {
    console.error("");
    console.error(`FAILED SUITES (${failed.length}):`);
    for (const r of failed) {
      console.error(`\n--- ${r.label} (exit ${r.code}) ---`);
      if (r.stdout.trim()) {
        console.error("stdout:");
        console.error(r.stdout);
      }
      if (r.stderr.trim()) {
        console.error("stderr:");
        console.error(r.stderr);
      }
    }
  }

  console.log("");
  console.log(
    `${anyFail ? "FAIL" : "PASS"} run_verify: ${results.length} suite(s), concurrency=${concurrency}, wall=${wallMs}ms`,
  );
  if (afterBanner) afterBanner();

  process.exit(anyFail ? 1 : 0);
}

function impactedConcurrency() {
  return Math.max(1, Number(process.env.BEE_VERIFY_CONCURRENCY) || Math.min(5, os.cpus().length));
}

// The impacted-mode entry point: resolves the changed-file set (explicit
// --impacted files UNION --impacted-from-git's git-derived set), maps it
// through the registry, and either exits with the loud zero-impacted pass
// or runs the exact-selected suite set through the shared execution tail.
async function runImpactedMode(explicitFiles, fromGit, level) {
  const changed = new Set();
  for (const raw of explicitFiles) changed.add(normalizeQueryPath(raw));
  if (fromGit) {
    for (const raw of gitImpactedFiles()) changed.add(normalizeQueryPath(raw));
  }
  const changedList = [...changed];

  const registry = await loadImpactRegistry();
  const { mappedSuites, unmapped } = queryRegistry(registry, changedList, { level });

  if (mappedSuites.length === 0) {
    printImpactedUnmapped(unmapped);
    if (level === 1) {
      // direct=0: check whether the same changed set maps to anything at
      // all transitively — that's a materially different, louder story
      // ("suites exist but are deferred") than a plain zero-impacted pass,
      // and must never be silent (must-have: "direct=0 => loud
      // transitive-deferred note, never silent").
      const { mappedSuites: transitiveSuites } = queryRegistry(registry, changedList);
      if (transitiveSuites.length > 0) {
        console.log(
          `IMPACTED RUN (level 1): 0 direct suite(s) from ${changedList.length} changed file(s) — ${transitiveSuites.length} suite(s) reachable only transitively, deferred to full verify/CI`,
        );
        process.exit(0);
        return;
      }
    }
    console.log(
      `IMPACTED RUN${level === 1 ? " (level 1)" : ""}: 0 suite(s) from ${changedList.length} changed file(s) — full verify delegated to CI`,
    );
    process.exit(0);
    return;
  }

  // test-economy D6: the impacted cap governs ONLY a transitive run — a
  // caller that already asked for `--level 1` gets today's behavior
  // unchanged, no banner, no cut (must-have: "--level 1 => no banner, no
  // cut"). When it does apply and the transitive selection's share of the
  // full suite pool exceeds the configured cap, the selection narrows to the
  // level-1 query over the SAME changed-file set — which, per
  // impact_registry.mjs (a suite's own path is always direct for itself,
  // :428), already keeps every self-selected suite (a changed test file
  // maps directly to itself) and every suite that imports a changed file
  // directly; only the pure-transitive tail is dropped.
  let selectedSuites = mappedSuites;
  let capInfo = null;
  if (level !== 1) {
    const cap = readImpactedCap();
    const ratio = mappedSuites.length / SUITES.length;
    if (cap < 1 && ratio > cap) {
      const capped = queryRegistry(registry, changedList, { level: 1 });
      selectedSuites = capped.mappedSuites;
      capInfo = { total: mappedSuites.length, cap };
    }
  }

  const activeSuites = filterSuitesByLabels(SUITES, new Set(selectedSuites));
  printImpactedUnmapped(unmapped);
  if (capInfo) {
    console.log(
      `impacted ${capInfo.total}/${SUITES.length} exceeds cap (${capInfo.cap}) — transitive tail delegated to CI`,
    );
  }
  // Banner reflects what's actually RUNNING: once capped, the selection IS a
  // level-1 query result, so the per-suite banner says so too, even though
  // the caller never passed --level 1 itself.
  const bannerLevel = capInfo ? 1 : level;

  await runSelectedSuites(activeSuites, {
    concurrency: impactedConcurrency(),
    beforeBanner: () => printImpactedBanner(activeSuites.length, changedList.length, bannerLevel),
    afterBanner: () => {
      printImpactedBanner(activeSuites.length, changedList.length, bannerLevel);
      // exit code is left to the shared execution tail's own anyFail
      // computation over `activeSuites` — "exit = kết quả phần đã chạy"
      // (D6): a red suite inside the capped selection stays red, never
      // silently swallowed the way the unmapped/zero-impacted branch's
      // unconditional exit-0 is.
      if (capInfo) triggerCiDelegation();
    },
  });
}

// Default concurrency: validated empirically on this repo's suite mix.
// Math.min(6, cpus) was the original target but produced a real flake in
// validation (test_onboard_bee starved to 56s when it landed alongside 4-5
// other 30s+ suites at once); Math.min(5, cpus) removed that clustering and
// passed 8/8 consecutive runs at ~32s wall-time. Still fully overridable via
// BEE_VERIFY_CONCURRENCY for machines with a different core/load profile.
async function main() {
  const argv = process.argv.slice(2);
  const impactedFilesArg = parseImpactedArgs(argv);
  const impactedFromGit = hasImpactedFromGitFlag(argv);
  const isImpactedRun = impactedFilesArg.length > 0 || impactedFromGit;
  const levelRaw = getLevelFlagValue(argv);
  const hasLevelFlag = levelRaw !== undefined;

  if (hasLevelFlag && !isImpactedRun) {
    console.error(
      "run_verify: --level is only valid alongside --impacted/--impacted-from-git — pick an impacted selection mode first.",
    );
    process.exit(1);
    return;
  }

  if (isImpactedRun) {
    const onlyTokensCheck = resolveOnlyTokens();
    if (onlyTokensCheck.length > 0) {
      console.error(
        "run_verify: --impacted/--impacted-from-git and --only/BEE_VERIFY_ONLY are mutually exclusive selection modes — pick one.",
      );
      process.exit(1);
      return;
    }
    if (hasLevelFlag && levelRaw !== "1") {
      console.error(
        `run_verify: --level "${levelRaw}" is not a recognized level — only "--level 1" is supported.`,
      );
      process.exit(1);
      return;
    }
    const level = hasLevelFlag ? 1 : undefined;
    await runImpactedMode(impactedFilesArg, impactedFromGit, level);
    return;
  }

  const concurrency = Math.max(
    1,
    Number(process.env.BEE_VERIFY_CONCURRENCY) || Math.min(5, os.cpus().length),
  );

  const rootFilteredSuites = filterSuitesByRoot(SUITES);
  if (process.env.BEE_VERIFY_ROOT_FILTER && rootFilteredSuites.length === 0) {
    console.error(
      `run_verify: BEE_VERIFY_ROOT_FILTER="${process.env.BEE_VERIFY_ROOT_FILTER}" matched zero suites — refusing a silent trivial-green run. FIX: check the root prefix.`,
    );
    process.exit(1);
  }

  const onlyTokens = resolveOnlyTokens();
  const onlyFilteredSuites = filterSuitesByOnly(rootFilteredSuites, onlyTokens);
  const isScopedRun = onlyTokens.length > 0;
  if (isScopedRun && onlyFilteredSuites.length === 0) {
    console.error(
      `run_verify: --only "${onlyTokens.join(",")}" matched zero suites — refusing a silent trivial-green run. FIX: check the token(s) against a suite's repo-relative path or display name.`,
    );
    process.exit(1);
  }

  const activeSuites = filterExcludedSuites(onlyFilteredSuites);
  if (process.env.BEE_VERIFY_EXCLUDE && activeSuites.length === 0) {
    console.error(
      `run_verify: BEE_VERIFY_EXCLUDE="${process.env.BEE_VERIFY_EXCLUDE}" excluded every remaining suite — refusing a silent trivial-green run. FIX: check the excluded paths.`,
    );
    process.exit(1);
  }

  const scopedBanner = isScopedRun
    ? () => printScopedBanner(onlyFilteredSuites.length, rootFilteredSuites.length)
    : null;

  await runSelectedSuites(activeSuites, {
    concurrency,
    beforeBanner: scopedBanner,
    afterBanner: scopedBanner,
  });
}

// Only run the suite pool when this file is executed directly (`node
// scripts/run_verify.mjs`) — NOT when imported, e.g. by
// scripts/tests/test_verify_manifest.mjs pulling in the exported SUITES list. An
// unconditional call here would spawn the entire suite as a side effect of
// a plain `import`.
const isMain = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url;
if (isMain) {
  main();
}
