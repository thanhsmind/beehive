// cells.mjs — one JSON file per cell in .bee/cells/. Enforces lane tiers,
// gate-locked claiming, cap-requires-verify.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { readJson, writeJsonAtomic, ensureDir, removeFileIfExists, readJsonl, appendJsonl } from './fsutil.mjs';
import {
  readState,
  gateApproved,
  MODEL_TIERS,
  lanePath,
  readLane,
  readLaneStrict,
  resolvePipeline,
  listLanes,
  resolveRoots,
  readConfig,
  NO_TEST_SENTINEL,
  isNoTestRepo,
  isNoTestCommand,
  controlRootFor,
} from './state.mjs';
// fsh-11 (D2/D4): claim-next's cross-session selection + throw-safe two-store
// claim needs claims.mjs's atomic primitive, reservations.mjs's cross-session
// hold check, and backlog.mjs's Feature-column rank — none of these create an
// import cycle (claims.mjs/reservations.mjs import only fsutil/node builtins;
// backlog.mjs imports only fs/path — same discipline state.mjs already relies
// on for pathsOverlap/readSession above it in the module graph).
import {
  sweepExpiredClaims,
  claimCellFile,
  releaseClaim,
  clearClaim,
  listSessionRecords,
  heartbeatStale,
  readClaim,
  isClaimActive,
  claimExpiry,
  resolveSessionId,
} from './claims.mjs';
import { findSessionConflicts } from './reservations.mjs';
// xwh-3: claim-next's hold-free predicate also consults the shared
// cross-worktree ledger (worktree-holds.mjs, wired standalone by xwh-1/xwh-2)
// so a cell whose files are held by a DIFFERENT checkout is skipped exactly
// like a same-checkout reservation hold — read-only, never writes the
// ledger. worktree-holds.mjs imports only fsutil/lock/reservations.mjs, so
// this creates no cycle (same discipline reservations.mjs's own import
// already relies on above it in the module graph).
import { findForeignHolds } from './worktree-holds.mjs';
import { featureBacklogRank } from './backlog.mjs';
// D2 (self-correcting-loop) — resetCellBudget logs a decision through the
// SAME event-sourced log every other decision-logging verb uses (bee-bypass-
// gate toggles, etc.). decisions.mjs imports only fsutil/node builtins, so
// this creates no import cycle.
import { logDecision } from './decisions.mjs';
// parallel-scheduler D2: cycle refusal at every dep-mutating write reuses the
// SAME structural check schedule.mjs runs for diagnostics (one algorithm, one
// definition of "cycle") — cells.mjs -> schedule.mjs stays one-directional
// (schedule.mjs never imports cells.mjs back).
import { detectCycles } from './schedule.mjs';
// D5 (self-correcting-loop) — validateJudgeVerdict/deriveModelIndependence
// are pure, zero-I/O (judge.mjs imports only dispatch-guard.mjs's
// PINNED_MODEL_STATUS, no cycle back to cells.mjs); recordJudgeVerdict below
// is the sole mutator that turns a validated verdict into a trace entry.
import { validateJudgeVerdict, deriveModelIndependence } from './judge.mjs';
// GH #27.2 (cell ghf-4) — the attempt ledger's read-mutate-write mutators
// below (recordVerify/capCell/blockCell/resetCellBudget) are wrapped in
// withStoreLock('cells:<id>', ...) so two concurrent verify/cap/block calls
// on the SAME cell can no longer both read the same trace.attempts snapshot
// and have the later write silently drop the earlier append — same
// discipline reservations.mjs's reserve/release and state.mjs's startFeature
// already use. The lock key includes the cell id (not a single 'cells' name)
// so distinct cells never serialize against each other. Pure readers
// (readCell, listCells, etc.) stay lock-free.
import { withStoreLock, acquireStoreLockOnceSync } from './lock.mjs';
// derived-check-hardening E1: the cap-time impact-registry cross-check below
// reuses the SAME derivation scripts/impact_registry.mjs already computes
// (queryRegistry) rather than re-deriving file->suite relatedness here.
// dch-8: NOT a static top-level import — scripts/ is a sibling of lib/'s
// package root, not lib/ itself, so a fixture that vendors only
// .bee/bin/lib/*.mjs (e.g. test_write_guard.mjs's copyLib()) has no
// scripts/ directory at all. A static import of a missing module fails
// the whole module load with ERR_MODULE_NOT_FOUND before any try/catch in
// this file can run, which silently disabled the caller's fail-open path.
// The import is done lazily with `await import(...)` inside the guarded
// try block right where it's used (below), so an absent module resolves
// to the SAME silent skip E1 already requires for an absent or malformed
// registry file.

export const LANES = ['tiny', 'small', 'standard', 'high-risk', 'spike'];

// D3 (self-correcting-loop) — judge-standard change classification. Optional
// cell field `change_class`; the ONLY auto-derivation this feature permits
// (CONTEXT prohibition: "no auto-derivation beyond behavior_change=>behavior")
// is absent field + `behavior_change:true` -> 'behavior'. Anything else
// absent derives null ("unclassified" — no matrix advisory at authoring, no
// cap teeth at cap; CONTEXT: "unclassified ⇒ no matrix check").
// test-economy D1: 'refactor' added to the enum — a change with no new
// behavior to prove, distinct from 'formatting' (both share the same proof
// tier below, but are named separately since CONTEXT.md and downstream
// classification prose treat them as distinct authoring choices).
// slice-tail-test-batching P2 (spec #80/#85): 'test' joins the enum as the
// slice's ONE consolidated test-authoring cell — the class the implementation
// cells' deferred authoring obligation lands in. It is a normal cell in every
// other respect (no new worker type, no slice record: slices exist only in
// skill text, so the machine-checkable scope is the active feature's
// test-class cells — see the P4 guard in bee.mjs).
export const CHANGE_CLASSES = ['formatting', 'bugfix', 'behavior', 'api', 'security', 'migration', 'refactor', 'test'];

export function deriveChangeClass(cell) {
  if (!cell || typeof cell !== 'object') return null;
  if (typeof cell.change_class === 'string' && cell.change_class) return cell.change_class;
  return cell.behavior_change === true ? 'behavior' : null;
}

// test-economy D1/D2 — the proof-tier matrix: given an ALREADY-RESOLVED
// change_class (callers resolve null/behavior_change=>'behavior' via
// deriveChangeClass BEFORE calling this — no derivation happens here) and a
// lane, returns the minimum proof capCell enforces:
//   'red-first'      — security/migration (every lane, regardless of lane or
//                       bc), or a behavior-bearing class (bugfix/behavior/
//                       api) riding the high-risk lane. The pre-existing
//                       Decision 0009 "before" characterization + the D3
//                       self-correcting-loop 80-char/anti-duplicate floor
//                       both stay enforced ONLY where this tier applies —
//                       CONTEXT test-economy D2's named supersession.
//   'targeted-green'  — bugfix outside high-risk, and the slice 'test' cell
//                       in every lane: a single targeted test passing
//                       (ordinary verification_evidence) is proof enough, no
//                       red_failure_evidence required. bugfix keeps its
//                       repro-first discipline (the failing repro test IS the
//                       diagnosis evidence) — slice-tail-test-batching P3
//                       restates that row as UNCHANGED, and the 'test' cell
//                       sits here because authoring is its whole mandate.
//   'existing-targeted-green'
//                     — behavior/api outside high-risk (slice-tail-test-
//                       batching P1, spec #80/#85). Same targeted scope the
//                       worker ran before; what changes is the suite's
//                       CONTENT: the EXISTING suite (plus prior slices'
//                       tests) staying green, with NO new-test authoring
//                       obligation at this cell — that obligation moves to
//                       the slice's one trailing 'test' cell. The cell's
//                       `verify` is still a runnable command recorded with
//                       output (decision 0004 is untouched), so every cap
//                       still proves the cell did not break what exists.
//                       Downstream it behaves exactly like 'targeted-green':
//                       every teeth-bearing check below keys on
//                       'red-first', which this tier is not.
//   'suite-green'     — refactor/formatting, in EVERY lane including
//                       high-risk (test-economy plan.md "Pin refactor/
//                       formatting × high-risk": a refactor has no new
//                       behavior to red-test — forcing red-first there would
//                       just be pressure to misclassify). The existing suite
//                       staying green is proof enough. Cap additionally
//                       refuses outright if the diff adds a new test file
//                       (see the diff_stats check in capCell) — no
//                       new_suite_reason can override that (D1).
//   null              — unclassified with behavior_change=false: NO matrix
//                       check at all, today's pre-test-economy behavior,
//                       untouched (CONTEXT: "muốn hưởng tier nhẹ hơn behavior
//                       phải KHAI class, mặc định không nới").
export function requiredProofTier(change_class, lane) {
  if (change_class === 'security' || change_class === 'migration') return 'red-first';
  if (change_class === 'refactor' || change_class === 'formatting') return 'suite-green';
  // slice-tail-test-batching P2: the slice's consolidated test cell — its
  // whole mandate IS authoring, so it caps on its own new targeted suite
  // passing, in every lane. Never red-first: a test cell has no production
  // change to characterize a "before" for.
  if (change_class === 'test') return 'targeted-green';
  // slice-tail-test-batching P3 — bugfix is SPLIT OUT and left EXACTLY as it
  // was: 'targeted-green' outside high-risk, 'red-first' on high-risk. The
  // repro-first discipline riding this row is diagnosis evidence, not
  // coverage ceremony, and P1 deliberately does not touch it. Reading the P1
  // amendment as "behavior-bearing classes all moved" would silently drop it
  // — hence two statements instead of one shared branch.
  if (change_class === 'bugfix') {
    return lane === 'high-risk' ? 'red-first' : 'targeted-green';
  }
  // slice-tail-test-batching P1: behavior/api outside high-risk now cap on
  // the EXISTING suite green; high-risk keeps red-first, unchanged.
  if (change_class === 'behavior' || change_class === 'api') {
    return lane === 'high-risk' ? 'red-first' : 'existing-targeted-green';
  }
  return null;
}

// Tolerant parse of `verification_evidence` shared by the D3 cap teeth below
// and by bee.mjs's own F5 STDERR advisory (recomputed from the returned cell,
// pah-2 precedent — never a side channel). Never throws: a string that fails
// to parse, or a non-object shape, degrades to {} so callers see "no
// evidence" rather than crashing on a malformed cell.
export function parseVerificationEvidence(raw) {
  let evidence = raw;
  if (typeof evidence === 'string') {
    try {
      evidence = JSON.parse(evidence);
    } catch {
      return {};
    }
  }
  return evidence && typeof evidence === 'object' && !Array.isArray(evidence) ? evidence : {};
}

// F5: a behavior-class cap riding the pre-existing deliberate_exceptions door
// keeps today's contract untouched (no length/duplicate floor) — this just
// answers "did it ride that door", shared by the cap teeth and the advisory.
export function evidenceRidesExceptionDoor(evidence) {
  const exceptions = evidence && evidence.deliberate_exceptions;
  return Array.isArray(exceptions)
    ? exceptions.some((e) => typeof e === 'string' && e.trim().length > 0)
    : typeof exceptions === 'string' && exceptions.trim().length > 0;
}

// ─── derived regen obligation (regen-obligation-derived D1/D2) ─────────────
// A cell whose OWN `files` touch a path a standing repo-wide guard hashes
// owes that guard's `--check` in its OWN `verify` and the guard's regen in
// its own work (PAT37, five recurrences). bee.mjs's manifestLintWarning (H2)
// asks the dodgeable question — "your verify mentions release_manifest but
// your files omit the manifest?" — so a cell that never mentions it at all
// walks straight past. This is the other direction: the obligation is DERIVED
// from what the cell touches, and it REFUSES the write (D1). H2 keeps its
// advisory behavior for what it already covers; nothing about it changes.
//
// D2 — THE ROOTS ARE NEVER HARD-CODED HERE. They are parsed out of the guard
// scripts themselves at every add/update, so a root added to
// release_manifest.mjs tomorrow is enforced here with no edit to this file.
// That is the whole point of the cell: the trigger scope for this trap has
// now been mis-stated three times running (D20 "templates/lib/ only", D24
// "two roots", the routing brief "two roots" again), every time by copying
// the previous summary instead of re-reading the script. Measured 2026-07-23,
// the real answer is TWELVE manifest roots — the six anyone recites plus
// .claude-plugin/marketplace.json, both installers and both distribution
// tests. A literal list in this guard would inherit exactly that defect.
//
// Absent scripts = inactive guard (a host repo has no scripts/ of ours), so
// this is silent everywhere but a repo that actually carries the guard.
const REGEN_ACK_FIELD = 'regen_obligation_ack';

const REGEN_GUARDS = [
  {
    key: 'manifest',
    script: 'scripts/release_manifest.mjs',
    covers: 'the release manifest hashes',
    required: 'release_manifest.mjs --check',
    command: 'node scripts/release_manifest.mjs --check',
    regen: 'node scripts/render_plugin_skill_trees.mjs, then node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply, then node scripts/release_manifest.mjs --write (in that order)',
    derive: deriveManifestScope,
  },
  {
    key: 'ledger',
    script: 'scripts/ledger_parity.mjs',
    covers: 'the .bee/onboarding.json managed-hash ledger covers',
    required: 'ledger_parity.mjs --check',
    command: 'node scripts/ledger_parity.mjs --check',
    regen: 'node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply',
    derive: deriveLedgerScope,
  },
];

/**
 * Leading string-literal arguments of one path.join(...) argument list, as
 * path segments. Stops at the first non-literal argument, so an
 * expression-built path contributes only its literal prefix (or nothing).
 */
function literalJoinSegments(argText) {
  const segments = [];
  for (const rawArg of String(argText).split(',')) {
    const arg = rawArg.trim();
    if (!arg) continue;
    const literal = /^"([^"\\]*)"$|^'([^'\\]*)'$/.exec(arg);
    if (!literal) break;
    segments.push(literal[1] !== undefined ? literal[1] : literal[2]);
  }
  return segments;
}

/** Every `path.join(<baseIdent>, "a", "b", …)` in `source`, as posix paths. */
function joinedLiteralPaths(source, baseIdent) {
  const found = [];
  const call = new RegExp(`path\\.join\\(\\s*${baseIdent}\\s*,([^)]*)\\)`, 'g');
  let match;
  while ((match = call.exec(source)) !== null) {
    const segments = literalJoinSegments(match[1]);
    if (segments.length > 0) found.push(segments.join('/'));
  }
  return found;
}

// release_manifest.mjs builds every inventory root as path.join(REPO_ROOT, …)
// — enumerated dirs and individually named files alike — so one pattern picks
// up all of them, including any added later. MANIFEST_PATH is derived the same
// way (by its const name, not its literal value) and subtracted: the manifest
// is the RECORD of the hashed set, never a member of it. It comes back as the
// path the cell's `files` must also list, which is what makes a cold worker's
// regen sanctioned instead of out-of-scope.
function deriveManifestScope(source) {
  const named = /\bMANIFEST_PATH\s*=\s*path\.join\(\s*REPO_ROOT\s*,([^)]*)\)/.exec(source);
  const manifestPath = named ? literalJoinSegments(named[1]).join('/') : null;
  const roots = [...new Set(joinedLiteralPaths(source, 'REPO_ROOT'))]
    .filter((candidate) => candidate && candidate !== manifestPath)
    .sort();
  return { roots, requiredFiles: manifestPath ? [manifestPath] : [] };
}

// ledger_parity.mjs's scope is "whatever checkGroup(managed.X, relDir) is
// called with", resolved against the base its own checked path is built from
// (path.join(root, ".bee", "bin", relDir, name)). Both halves are read out of
// the script, so a third group added there is covered here for free.
function deriveLedgerScope(source) {
  const based = /path\.join\(\s*root\s*,((?:\s*(?:"[^"\\]*"|'[^'\\]*')\s*,)+)\s*relDir\b/.exec(source);
  const base = based ? literalJoinSegments(based[1]).join('/') : null;
  if (!base) return { roots: [], requiredFiles: [] };
  const roots = new Set();
  const group = /checkGroup\(\s*managed\.\w+\s*,\s*(?:"([^"\\]*)"|'([^'\\]*)')\s*\)/g;
  let match;
  while ((match = group.exec(source)) !== null) {
    const relDir = match[1] !== undefined ? match[1] : match[2];
    roots.add([base, relDir].filter(Boolean).join('/'));
  }
  return { roots: [...roots].sort(), requiredFiles: [] };
}

/**
 * The active guards for `root`, each with its DERIVED roots. A guard whose
 * script is absent is inactive. A guard whose script is present but yields no
 * root throws: a guard that cannot see its own scope is blind, and blind is
 * the failure this whole rule exists to end — never a silent pass.
 */
export function deriveRegenGuards(root) {
  const active = [];
  for (const guard of REGEN_GUARDS) {
    const abs = path.join(root, ...guard.script.split('/'));
    let source;
    try {
      source = fs.readFileSync(abs, 'utf8');
    } catch {
      continue; // guard not installed in this repo — nothing to owe
    }
    const scope = guard.derive(source);
    if (!scope.roots.length) {
      throw new Error(
        `regen obligation: could not derive any covered root from "${guard.script}" — the guard would be blind, so the write is refused rather than passed silently. FIX: the script's shape changed; update deriveRegenGuards in lib/cells.mjs to read the new shape (never paste a literal root list in — see D2).`,
      );
    }
    active.push({ ...guard, ...scope });
  }
  return active;
}

function normalizeCellPath(value) {
  return String(value).trim().replace(/\\/g, '/').replace(/^\.\//, '').replace(/\/+$/, '');
}

function pathUnderRoot(filePath, rootPath) {
  return filePath === rootPath || filePath.startsWith(`${rootPath}/`);
}

/**
 * The refusal message for `cell`, or null when nothing is owed. Pure apart
 * from reading the guard scripts; never throws on a malformed cell (that
 * judgment belongs to validateNewCell/updateCell's own field refusals).
 */
export function regenObligationRefusal(root, cell, verb = 'addCell') {
  if (!cell || typeof cell !== 'object') return null;
  const ack = cell[REGEN_ACK_FIELD];
  if (typeof ack === 'string' && ack.trim()) return null; // D1 escape hatch, recorded on the cell
  const files = (Array.isArray(cell.files) ? cell.files : [])
    .filter((f) => typeof f === 'string' && f.trim())
    .map(normalizeCellPath);
  if (files.length === 0) return null;
  const verify = typeof cell.verify === 'string' ? cell.verify : '';
  const id = typeof cell.id === 'string' && cell.id ? cell.id : '(unknown id)';

  for (const guard of deriveRegenGuards(root)) {
    let hitPath = null;
    let hitRoot = null;
    for (const file of files) {
      const matched = guard.roots.find((rootPath) => pathUnderRoot(file, rootPath));
      if (matched) {
        hitPath = file;
        hitRoot = matched;
        break;
      }
    }
    if (!hitPath) continue;

    const missing = [];
    if (!verify.includes(guard.required)) {
      missing.push(`verify does not contain "${guard.required}"`);
    }
    for (const requiredFile of guard.requiredFiles) {
      if (!files.includes(requiredFile)) missing.push(`files does not list "${requiredFile}"`);
    }
    if (missing.length === 0) continue;

    const fixes = [];
    if (!verify.includes(guard.required)) fixes.push(`add \`${guard.command}\` to this cell's verify`);
    for (const requiredFile of guard.requiredFiles) {
      if (!files.includes(requiredFile)) fixes.push(`add "${requiredFile}" to its files`);
    }
    return (
      `${verb}: REGEN_OBLIGATION — cell "${id}" touches "${hitPath}", which falls under "${hitRoot}", ` +
      `a root ${guard.covers} (derived at runtime from ${guard.script}, never a list kept here). ` +
      `Missing: ${missing.join('; ')}. FIX: ${fixes.join(', ')}, and run the regen inside THIS cell — ${guard.regen}. ` +
      `To skip deliberately, set "${REGEN_ACK_FIELD}" on the cell to a one-line reason; it is recorded in the cell, ` +
      `so skipping is a named act rather than an oversight. For parallel waves, the recognized value "wave-barrier" ` +
      `defers the regen to the orchestrator, which owes the full regen chain once at wave close, in the wave-close ` +
      `commit, before the wave is declared clean (parallel-default D2). The write is refused; nothing was written.`
    );
  }
  return null;
}

function assertRegenObligation(root, cell, verb) {
  const refusal = regenObligationRefusal(root, cell, verb);
  if (refusal) throw new Error(refusal);
}

const ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;

function utcNow() {
  return new Date().toISOString();
}

// D1 — deterministic failure-signature normalizer, exported so `cells verify`
// (mechanical fallback) and any future caller share ONE definition of
// "logically the same failure". Strips the noise that makes two runs of the
// identical failure look different byte-for-byte — ISO timestamps, absolute
// paths (never leaked into the signature even before hashing — prohibition
// in CONTEXT D1), and hex-looking runs (commit SHAs, addresses) — then picks
// the single most diagnostic line: the first line that reads as a failure
// (`FAIL|Error|refus|denied`, case-insensitive), else the first non-empty
// line, else "" for genuinely empty output. sha256 of that line, first 12
// hex chars: short enough to eyeball in a ledger row, long enough that two
// unrelated failures collide only by coincidence.
const ISO_TIMESTAMP_RE = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?/g;
const WIN_ABS_PATH_RE = /[A-Za-z]:\\[^\s"'<>]*/g;
// Requires at least two path segments (two slashes) so ordinary fraction-
// shaped text ("3/45 passed") is never mistaken for a path.
const UNIX_ABS_PATH_RE = /\/(?:[\w.-]+\/)+[\w.-]*/g;
const HEX_RUN_RE = /\b[0-9a-fA-F]{6,}\b/g;
const FAILING_LINE_RE = /FAIL|Error|refus|denied/i;

export function normalizeFailureSignature(output) {
  const text = typeof output === 'string' ? output : '';
  const scrubbed = text
    .replace(ISO_TIMESTAMP_RE, '<ts>')
    .replace(WIN_ABS_PATH_RE, '<path>')
    .replace(UNIX_ABS_PATH_RE, '<path>')
    .replace(HEX_RUN_RE, '<hex>');
  const lines = scrubbed.split(/\r?\n/).map((line) => line.trim());
  const chosen = lines.find((line) => line && FAILING_LINE_RE.test(line)) || lines.find((line) => line) || '';
  return crypto.createHash('sha256').update(chosen, 'utf8').digest('hex').slice(0, 12);
}

// D1 (+Δ1) — revision ledger: append-only per-cell attempt history, written
// by every recordVerify outcome and by blockCell. Appended to
// trace.attempts, NEVER trace.deviations (capCell REPLACES that key wholesale
// from its own `deviations` argument — see appendOwnershipOverride's comment
// above for the same trap with ownership_overrides). Every mutator's own
// `...trace` spread is what keeps this array alive across later transitions;
// no verb here ever rewrites or removes an existing entry.
//
// claim_session/claimed_at are read from the LIVE claim file (Δ1), not from
// the cell's own trace.claimed_at (a separate, claim-time-only stamp) —
// budget counting (D2) must key off exactly what claims.mjs itself considers
// the current acquisition. A swept/absent/sessionless claim reads as
// null/null; that undercount is conservative-safe (F2), never a refusal.
//
// GH #27.1 (D-GHF-B): acquired_at is read alongside claimed_at, falling back
// to claimed_at when the claim file predates acquired_at (legacy claims) —
// it is the immutable acquisition identity checkCellBudgets pairs on below.
// claimed_at stays exactly as before for compat (it is the heartbeat-mutated
// expiry clock, still recorded verbatim).
function appendAttempt(root, id, trace, { verdict, failureSignature = null, note = null }) {
  const attempts = Array.isArray(trace.attempts) ? trace.attempts : [];
  // msn-18b: claims are control-plane (PLANE RULE) — resolved through
  // controlRootFor so an attempt appended from a linked worktree reads the
  // SAME claim record main sees, never a worktree-local one. `id`/`trace`
  // themselves stay entirely on the cell-file plane (unaffected — cell files
  // are workspace-local, never re-rooted).
  const claim = readClaim(controlRootFor(root), id);
  return {
    ...trace,
    attempts: [
      ...attempts,
      {
        n: attempts.length + 1,
        at: utcNow(),
        claim_session: claim && typeof claim.session === 'string' ? claim.session : null,
        claimed_at: claim && typeof claim.claimed_at === 'string' ? claim.claimed_at : null,
        acquired_at:
          claim && typeof claim.acquired_at === 'string'
            ? claim.acquired_at
            : claim && typeof claim.claimed_at === 'string'
              ? claim.claimed_at
              : null,
        worker: typeof trace.worker === 'string' ? trace.worker : null,
        verdict,
        failure_signature: failureSignature,
        note,
      },
    ],
  };
}

// D1 (Δ2-amended): release the O_EXCL claim file on EVERY claim-clearing
// transition — cap, unclaim, block, drop, reopen — not only the claim-next
// unwind path. Without this a same-session block -> reopen -> claim round
// trip self-refuses CLAIMED for the claim's full TTL, since nothing else
// ever removes the file the original claim created. Best-effort by design:
// a rare GATE_HELD (another in-flight adopt/sweep on the SAME cell,
// millisecond-scale) or an unexpected fs error here must never fail a cell
// transition that has already been written to disk by the time this runs —
// the claim file is a secondary, TTL-bounded artifact, never the source of
// truth for cell status. Ownership checking on these verbs is a separate,
// later concern (D4/msh-4); this only guarantees the claim file never
// outlives the "claimed" status it was created for.
function releaseClaimFileBestEffort(root, id) {
  try {
    // msn-18b: claims are control-plane — see appendAttempt's comment above.
    clearClaim(controlRootFor(root), id);
  } catch {
    // never let claim-file cleanup fail a cell transition that already committed
  }
}

// D4 (msh-4) — typed ownership guard: composes readClaim/isClaimActive/
// claimExpiry (never a new claims.mjs reader). Never throws for contention,
// matching claims.mjs's own typed-failure convention (claimCellFile,
// releaseClaim) — the mutators below are the guard-CONSUMED callers that turn
// a refusal into their own thrown Error, the same style every other
// cells.mjs refusal already uses.
//
// ok:true (proceed unchanged) when: no claim file, an expired claim (rescue
// stays possible), a sessionless claim (single-session `cells claim` writes
// exactly this shape — D1 Δ2 — so single-session use never reaches a
// refusal), or the caller's resolved session (D3) matches the claim's. A
// LIVE claim carrying a session that differs from the caller ⇒ typed
// NOT_OWNER, naming owner + expiry in claimCellFile's own wording.
function checkClaimOwnership(root, id, sessionId) {
  // msn-18b: claims are control-plane — see appendAttempt's comment above.
  const claim = readClaim(controlRootFor(root), id);
  if (!claim || !isClaimActive(claim)) return { ok: true };
  const owner = claim.session;
  if (!owner) return { ok: true };
  const caller = resolveSessionId({ flag: sessionId });
  if (caller === owner) return { ok: true };
  return {
    ok: false,
    code: 'NOT_OWNER',
    reason: `cell "${id}" is claimed by session "${owner}" (${claimExpiry(claim)}) — another session owns it. Pass --force-ownership to override (audited).`,
    holder: owner,
  };
}

// D4 Δ5-amended — the force-ownership audit is a DISTINCT append-only trace
// key, trace.ownership_overrides, never trace.deviations: capCell REPLACES
// trace.deviations wholesale from its own `deviations` argument, so an
// append there would be silently wiped at the very next cap; the `...trace`
// spread every mutator already does preserves this unknown key across every
// transition. "Force always leaves an audit line" (must-have truth) — every
// call made with forceOwnership true appends one row here, whether or not a
// live conflicting claim actually existed to bypass (owner_bypassed is null
// when there was nothing to bypass, so the line still tells the truth).
function appendOwnershipOverride(trace, { verb, sessionId, ownership }) {
  const overrides = Array.isArray(trace.ownership_overrides) ? trace.ownership_overrides : [];
  return {
    ...trace,
    ownership_overrides: [
      ...overrides,
      {
        verb,
        forced_by: resolveSessionId({ flag: sessionId }),
        owner_bypassed: ownership.ok ? null : ownership.holder,
        at: utcNow(),
      },
    ],
  };
}

// Shared guard entrypoint for every D4-covered mutator: runs the ownership
// check, and either throws the typed refusal (no force) or returns the
// possibly-audited trace to use going forward (force-ownership was passed).
// `verb` names the caller for both the thrown message and the audit row.
function guardClaimOwnership(root, id, trace, verb, { sessionId, forceOwnership = false } = {}) {
  const ownership = checkClaimOwnership(root, id, sessionId);
  if (!forceOwnership) {
    if (!ownership.ok) throw new Error(`${verb}: ${ownership.reason}`);
    return trace;
  }
  return appendOwnershipOverride(trace, { verb, sessionId, ownership });
}

function defaultTrace() {
  return {
    worker: null,
    outcome: null,
    files_changed: [],
    deviations: [],
    friction: null,
    capped_at: null,
    behavior_change: false,
    verification_evidence: null,
    verify_output: null,
    verify_passed: null,
    // hardening-4b: the CURRENT claim's session (set by claimCell, cleared by
    // releaseTrace on every claim-clearing transition) — distinct from the
    // per-attempt ledger's own claim_session (appendAttempt), which is a
    // point-in-time snapshot read from the live claims-store file at
    // verify/block time. This one is a stable field on the cell itself, so
    // claims.mjs's sweepExpiredClaims can check "does this cell's claim still
    // belong to the session whose claim file I just swept" without reading
    // the ledger at all.
    claim_session: null,
  };
}

export function cellsDir(root) {
  return path.join(root, '.bee', 'cells');
}

// ARCHIVE_DIR_NAME is a reserved child of cellsDir — never a cell id, so the
// default listCells scan below must skip it explicitly (a directory entry
// fails the `.json` filter anyway, but the guard stays explicit per the
// cells-archive-1 must-have: verify, don't assume, that the filter excludes it).
const ARCHIVE_DIR_NAME = 'archive';

// cellsArchiveDir(root, feature) — the feature-scoped archive tree closed
// features' cells move into, out of the hot .bee/cells/ scan path. Pure path
// composition; creating/writing the directory is a later cell's job (this
// cell adds only the lookup primitive layer).
export function cellsArchiveDir(root, feature) {
  return path.join(cellsDir(root), ARCHIVE_DIR_NAME, feature);
}

function cellFile(root, id) {
  return path.join(cellsDir(root), `${id}.json`);
}

// hardening-1 (P0 data-loss fix): archiveFeature/unarchiveFeature take a
// caller-supplied `feature` string straight into path.join with no format
// check at all — a value like "../../../x" walks the resulting path outside
// .bee/cells/archive/ entirely. FEATURE_SLUG_PATTERN mirrors ID_PATTERN's
// discipline elsewhere in this file (letters/digits/dot/dash/underscore
// only, no path separators), and the bare "." / ".." forms are rejected on
// top of that even though the pattern alone already excludes "/" — a
// same-directory or parent-directory SEGMENT is a traversal primitive
// regardless of whether a separator is present.
const FEATURE_SLUG_PATTERN = /^[A-Za-z0-9._-]+$/;

class InvalidFeatureSlugError extends Error {
  constructor(verb, feature) {
    super(
      `${verb}: invalid feature "${feature}" — use letters, digits, dot, dash, underscore only (no path separators, and never "." or ".."). Refusing before any file is touched.`,
    );
    this.name = 'InvalidFeatureSlugError';
    this.code = 'INVALID_FEATURE_SLUG';
  }
}

class ArchivePathEscapeError extends Error {
  constructor(verb, resolved, base) {
    super(
      `${verb}: resolved archive path "${resolved}" escapes the archive root "${base}" — refusing before any file is touched.`,
    );
    this.name = 'ArchivePathEscapeError';
    this.code = 'ARCHIVE_PATH_ESCAPE';
  }
}

function assertValidFeatureSlug(verb, feature) {
  if (typeof feature !== 'string' || !feature.trim()) {
    throw new Error(`${verb}: feature is required.`);
  }
  if (!FEATURE_SLUG_PATTERN.test(feature) || /^\.+$/.test(feature)) {
    throw new InvalidFeatureSlugError(verb, feature);
  }
}

// Defense-in-depth alongside assertValidFeatureSlug: even though the slug
// pattern above already forecloses every practical escape, this asserts the
// FINAL computed archive directory is canonically contained inside the
// archive root before any rename touches disk — the same "verify, don't
// assume" discipline the cells-archive-1 must-have already applies to the
// listCells directory filter (ARCHIVE_DIR_NAME comment above).
function assertArchiveDirContained(verb, root, archiveDir) {
  const base = path.resolve(path.join(cellsDir(root), ARCHIVE_DIR_NAME));
  const resolved = path.resolve(archiveDir);
  if (resolved !== base && !resolved.startsWith(base + path.sep)) {
    throw new ArchivePathEscapeError(verb, resolved, base);
  }
}

// hardening-1 (P0 data-loss fix): the mutators below (updateCell, claimCell,
// recordVerify, capCell, blockCell, dropCell) all write through cellFile()
// (the ACTIVE path) while readCell()/resolveCellFile() transparently fall
// back into the archive tree. Without this guard, calling e.g. updateCell on
// an id that exists ONLY in the archive reads the archived copy fine, then
// writeCell() creates a BRAND NEW file at the active path — forking one
// logical cell id into two on-disk copies (one archived, one freshly active)
// that silently drift apart. assertNotArchived is a no-op for an ordinary
// active cell (cellFile exists — never even calls resolveCellFile) and for a
// genuinely missing id (resolveCellFile returns null, so the caller's own
// not-found handling fires as before); it throws ONLY when the id resolves
// exclusively inside archive/.
class CellArchivedError extends Error {
  constructor(verb, id) {
    super(`${verb}: cell "${id}" is archived — unarchive its feature first (bee.mjs cells unarchive --feature <feature>).`);
    this.name = 'CellArchivedError';
    this.code = 'CELL_ARCHIVED';
  }
}

function assertNotArchived(root, verb, id) {
  if (!id || !ID_PATTERN.test(String(id))) return; // malformed id — the caller's own validation handles this
  if (fs.existsSync(cellFile(root, id))) return; // active copy exists — never archived-only
  if (resolveCellFile(root, id)) throw new CellArchivedError(verb, id);
}

// hardening-1-7-10 (D4): writeCell's own contention refusal. A mutator's
// read-check-write (assertNotArchived + readCell + ... + writeCell) can
// still race a concurrent archiveFeature/unarchiveFeature transaction
// between its own assertNotArchived call and the moment writeCell actually
// touches disk — archive/unarchive hold the SAME 'cells-archive' lock for
// their WHOLE transaction (see archiveFeature/unarchiveFeature below), so
// writeCell's brief acquire of it either succeeds immediately (no live
// archive txn) or refuses cleanly with this typed error (one IS in
// flight) — it never blocks and waits (see acquireStoreLockOnceSync).
class CellsArchiveBusyError extends Error {
  constructor(id, holder) {
    const who =
      holder && typeof holder === 'object'
        ? `pid=${holder.pid ?? 'unknown'} session=${holder.session ?? 'unknown'} since ${holder.ts ?? 'unknown'}`
        : 'unknown holder';
    super(
      `writeCell: cell "${id}" write refused — the "cells-archive" lock is held by ${who} (a live archive/unarchive transaction). Retry once it completes.`,
    );
    this.name = 'CellsArchiveBusyError';
    this.code = 'CELLS_ARCHIVE_BUSY';
  }
}

// resolveCellFile(root, id) — the REAL on-disk path for a cell, active or
// archived. Unlike cellFile (always the active path, meaning unchanged for
// every existing caller), this searches .bee/cells/archive/*/<id>.json when
// the active file is absent, so a caller that needs to actually read/stat the
// file (not just resolve through readCell) gets the true location. Returns
// null if the cell exists in neither place. Never throws: a missing/absent
// archive root degrades to "not found there", same as every other tolerant
// reader in this file.
export function resolveCellFile(root, id) {
  if (!id || !ID_PATTERN.test(String(id))) return null;
  const active = cellFile(root, id);
  if (fs.existsSync(active)) return active;
  const archiveRoot = path.join(cellsDir(root), ARCHIVE_DIR_NAME);
  let featureDirs;
  try {
    featureDirs = fs.readdirSync(archiveRoot, { withFileTypes: true });
  } catch {
    return null;
  }
  for (const entry of featureDirs) {
    if (!entry.isDirectory()) continue;
    const candidate = path.join(archiveRoot, entry.name, `${id}.json`);
    if (fs.existsSync(candidate)) return candidate;
  }
  return null;
}

// listCells(root, {feature, status, includeArchived}) — default scans ONLY
// the active .bee/cells/ dir (the hot-path speedup this feature exists for),
// explicitly skipping the reserved `archive` child directory so it is never
// mistaken for a cell entry. includeArchived:true additionally folds in every
// .bee/cells/archive/*/*.json, same tolerant per-file readJson, same sorted-
// by-id shape. When no archive dir exists (today's repos), includeArchived
// has nothing to fold in and the output is byte-identical to the pre-archive
// behavior either way.
export function listCells(root, { feature = null, status = null, includeArchived = false } = {}) {
  const dir = cellsDir(root);
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    entries = [];
  }
  const cells = [];
  for (const entry of entries) {
    if (entry.isDirectory()) continue; // explicit guard: `archive` (or any dir) is never a cell
    if (!entry.name.endsWith('.json')) continue;
    const cell = readJson(path.join(dir, entry.name), null);
    if (!cell || typeof cell !== 'object') continue;
    if (feature && cell.feature !== feature) continue;
    if (status && cell.status !== status) continue;
    cells.push(cell);
  }
  if (includeArchived) {
    const archiveRoot = path.join(dir, ARCHIVE_DIR_NAME);
    let featureDirs;
    try {
      featureDirs = fs.readdirSync(archiveRoot, { withFileTypes: true });
    } catch {
      featureDirs = [];
    }
    for (const featureDir of featureDirs) {
      if (!featureDir.isDirectory()) continue;
      const featureRoot = path.join(archiveRoot, featureDir.name);
      let files;
      try {
        files = fs.readdirSync(featureRoot);
      } catch {
        continue;
      }
      for (const file of files) {
        if (!file.endsWith('.json')) continue;
        const cell = readJson(path.join(featureRoot, file), null);
        if (!cell || typeof cell !== 'object') continue;
        if (feature && cell.feature !== feature) continue;
        if (status && cell.status !== status) continue;
        cells.push(cell);
      }
    }
  }
  cells.sort((a, b) => String(a.id).localeCompare(String(b.id), 'en', { numeric: true }));
  return cells;
}

// readCell(root, id) — resolves the active cell first (byte-identical fast
// path to today, including when no archive dir exists at all: readJson on a
// missing active file returns null exactly as before, and the archive
// readdir below then also fails closed to null, no throw, no behavior
// change). Only when the active file is genuinely absent does it fall back to
// searching .bee/cells/archive/*/<id>.json, so dep-resolution (depsAllCapped)
// and any other readCell caller transparently see archived capped cells.
export function readCell(root, id) {
  if (!id || !ID_PATTERN.test(String(id))) return null;
  const active = readJson(cellFile(root, id), null);
  if (active !== null) return active;
  const archiveRoot = path.join(cellsDir(root), ARCHIVE_DIR_NAME);
  let featureDirs;
  try {
    featureDirs = fs.readdirSync(archiveRoot, { withFileTypes: true });
  } catch {
    return null;
  }
  for (const entry of featureDirs) {
    if (!entry.isDirectory()) continue;
    const candidate = path.join(archiveRoot, entry.name, `${id}.json`);
    const archived = readJson(candidate, null);
    if (archived !== null) return archived;
  }
  return null;
}

function archiveSummaryFile(root) {
  return path.join(cellsDir(root), ARCHIVE_DIR_NAME, 'summary.json');
}

// archivedSummary(root) — the on-disk archive ledger: {feature: {capped,
// dropped, archived_at}}. Absent file reads as {} (a fresh repo, or one that
// has never archived anything, is never a crash). This is the ONLY read
// status/buildStatus should use for archived figures — never a directory
// scan of .bee/cells/archive/ (that is exactly the hot-path cost archiving
// exists to avoid).
export function archivedSummary(root) {
  const summary = readJson(archiveSummaryFile(root), {});
  return summary && typeof summary === 'object' && !Array.isArray(summary) ? summary : {};
}

// archivedTotals(root) — {capped, dropped, total} summed across every
// archived feature's summary entry. Tolerant of a partially-shaped entry
// (missing/non-numeric capped/dropped reads as 0, never throws/NaNs).
export function archivedTotals(root) {
  const summary = archivedSummary(root);
  let capped = 0;
  let dropped = 0;
  for (const entry of Object.values(summary)) {
    if (!entry || typeof entry !== 'object') continue;
    capped += Number.isFinite(entry.capped) ? entry.capped : 0;
    dropped += Number.isFinite(entry.dropped) ? entry.dropped : 0;
  }
  return { capped, dropped, total: capped + dropped };
}

// ─── hardening-1-7-10 (D4): archive-transaction journal ────────────────────
// A journal is the crash-recovery record for an in-flight archiveFeature/
// unarchiveFeature transaction: the FULL set of planned {id, from, to}
// renames, written to disk BEFORE the first rename runs. If the process
// dies mid-loop (kill -9, disk full, host reboot) with no chance to run its
// own in-memory rollback, the journal is what lets the NEXT archiveFeature
// or unarchiveFeature call on this feature notice the half-migrated state
// and repair it before doing anything else — see recoverArchiveJournal
// below, invoked at the top of both functions' locked critical section.
//
// Lives at .bee/cells/archive/<feature>/.journal.json — inside the SAME
// per-feature directory the moves themselves target, so a single
// `cellsArchiveDir` scan never needs a second path to reason about. Its
// `op` field ('archive' | 'unarchive') is recorded for audit/debugging but
// recovery itself is direction-agnostic: recoverArchiveJournal only ever
// asks "did this specific move complete" (destination present, source
// absent) and reverses exactly the ones that did — the SAME check restores
// an interrupted archive (partially in .../archive/<feature>/) back to
// active, or an interrupted unarchive (partially in the active dir) back to
// archive, with no branch on `op` at all.
const ARCHIVE_JOURNAL_FILE = '.journal.json';

function archiveJournalPath(root, feature) {
  return path.join(cellsArchiveDir(root, feature), ARCHIVE_JOURNAL_FILE);
}

function writeArchiveJournal(root, feature, op, planned) {
  writeJsonAtomic(archiveJournalPath(root, feature), { op, feature, planned, started_at: utcNow() });
}

function clearArchiveJournal(root, feature) {
  removeFileIfExists(archiveJournalPath(root, feature));
}

// recoverArchiveJournal(root, feature) — called at the very top of BOTH
// archiveFeature's and unarchiveFeature's locked critical section (same
// 'cells-archive' lock both already hold for their whole transaction, so
// this can never race a live one). A missing or unparsable/malformed
// journal is "nothing to recover" (fail-open, matching every other tolerant
// reader in this file) — any leftover corrupt file is simply dropped so it
// never blocks the caller's own operation. For a well-formed journal: any
// planned move whose destination exists but whose source does NOT (i.e. it
// completed before the crash) is renamed back to its origin; a move that
// never started (source still present, destination absent) needs no repair.
// Best-effort per move — a single stuck rename during recovery must never
// crash the caller's own operation; the caller's own preflight/rename loop
// surfaces any genuinely blocking disk condition on its own terms.
function recoverArchiveJournal(root, feature) {
  const journalPath = archiveJournalPath(root, feature);
  const journal = readJson(journalPath, null);
  if (!journal || typeof journal !== 'object' || !Array.isArray(journal.planned)) {
    removeFileIfExists(journalPath);
    return;
  }
  for (const move of journal.planned) {
    if (!move || typeof move.from !== 'string' || typeof move.to !== 'string') continue;
    if (fs.existsSync(move.to) && !fs.existsSync(move.from)) {
      try {
        fs.renameSync(move.to, move.from);
      } catch {
        // best-effort: see function comment above
      }
    }
  }
  removeFileIfExists(journalPath);
}

// hardening-1-7-10 (D4): typed refusal shared by archiveFeature's
// destination-collision preflight and unarchiveFeature's active-destination
// preflight — never overwrite an existing file, named before any rename.
class ArchiveDestinationCollisionError extends Error {
  constructor(verb, feature, ids, kind) {
    super(
      `${verb}: feature "${feature}" refused — a ${kind} file already exists for ${ids.join(
        ', ',
      )}. Refusing before any file is touched (never overwrite existing data).`,
    );
    this.name = 'ArchiveDestinationCollisionError';
    this.code = 'ARCHIVE_DESTINATION_COLLISION';
  }
}

// archiveFeature(root, feature) — moves every cell of a fully-terminal
// feature (every cell capped or dropped — no open/claimed cell survives)
// from the hot .bee/cells/ scan path into .bee/cells/archive/<feature>/,
// out of listCells' default (non-includeArchived) scan. The caller (bee.mjs
// handler) is responsible for refusing when `feature` is the active
// state.feature — this primitive has no access to state.json and does not
// duplicate that check.
//
// Refuses (throws) on: no feature, zero cells for the feature (nothing to
// archive — named), any open/claimed cell (named), or a destination
// collision (named) — archiving is an all-terminal-or-nothing operation,
// never a partial move that could hide a still-live cell from the hot scan,
// and never an overwrite of an existing archived file.
export async function archiveFeature(root, feature) {
  assertValidFeatureSlug('archiveFeature', feature);
  // hardening-1: the whole read-check-move-summarize body runs under a
  // GLOBAL cross-process lock named 'cells-archive' (colon-free — a
  // whole-store operation, not a per-cell one like the `cells:<id>` locks
  // elsewhere in this file). The name is NOT scoped per feature, so this
  // also serializes archive/unarchive calls across DIFFERENT features, not
  // only the same one. hardening-1-7-10 (D4) corrects the other half of
  // this comment, which used to also claim a concurrent MUTATOR could never
  // interleave — that was false at the time: mutators held no lock against
  // this at all, which is exactly how a mutator could resurrect a cell
  // mid-archive (writeCell creating a brand-new active file while this
  // rename loop was in flight). writeCell now briefly takes this SAME lock
  // around its own final write (see writeCell above), so a concurrent
  // mutator's write is now serialized against a live archive/unarchive
  // transaction too — writeCell never takes `cells:<id>` itself here, so
  // lock order stays `cells:<id>` -> `cells-archive` and this can never
  // deadlock against a mutator that holds `cells:<id>`.
  return withStoreLock(root, 'cells-archive', () => {
    // hardening-1-7-10 (D4): repair any half-migrated leftover from an
    // interrupted PRIOR transaction on this feature before doing anything
    // else — see recoverArchiveJournal's own comment.
    recoverArchiveJournal(root, feature);
    const cells = listCells(root, { feature });
    if (cells.length === 0) {
      throw new Error(`archiveFeature: no cells found for feature "${feature}" — nothing to archive.`);
    }
    // hardening-1: allowlist, not a denylist — archive ONLY capped/dropped
    // cells. The prior denylist (open/claimed) silently let a `blocked` cell
    // through, contradicting the documented "capped/dropped only" contract;
    // negation here also fails closed against any future status this file
    // doesn't yet know about.
    const nonTerminal = cells.filter((cell) => cell.status !== 'capped' && cell.status !== 'dropped');
    if (nonTerminal.length > 0) {
      throw new Error(
        `archiveFeature: feature "${feature}" has non-terminal cell(s) — ${nonTerminal
          .map((cell) => `${cell.id} (${cell.status})`)
          .join(', ')} — only a feature whose cells are ALL capped/dropped can be archived.`,
      );
    }
    const archiveDir = cellsArchiveDir(root, feature);
    assertArchiveDirContained('archiveFeature', root, archiveDir);
    ensureDir(archiveDir);
    const statusById = new Map(cells.map((cell) => [cell.id, cell.status]));
    const planned = cells.map((cell) => ({
      id: cell.id,
      from: cellFile(root, cell.id),
      to: path.join(archiveDir, `${cell.id}.json`),
    }));
    // hardening-1-7-10 (D4): preflight EVERY destination BEFORE any rename —
    // a collision refuses the whole batch, named, with nothing touched.
    const collisions = planned.filter((move) => fs.existsSync(move.to)).map((move) => move.id);
    if (collisions.length > 0) {
      throw new ArchiveDestinationCollisionError('archiveFeature', feature, collisions, 'archived');
    }
    // Journal written before the first rename (D4) — the crash-recovery
    // sweep above (this call and every future one on this feature) replays
    // exactly this plan if the process dies before the loop below finishes.
    writeArchiveJournal(root, feature, 'archive', planned);
    const moved = [];
    let capped = 0;
    let dropped = 0;
    try {
      for (const move of planned) {
        fs.renameSync(move.from, move.to);
        moved.push(move);
        const status = statusById.get(move.id);
        if (status === 'capped') capped += 1;
        else if (status === 'dropped') dropped += 1;
      }
    } catch (err) {
      // hardening-1: a mid-loop crash (disk full, permission change, a
      // blocked destination) used to leave cells half-moved with no summary
      // update at all — the worst of both worlds (a cell invisible to the
      // hot scan AND absent from the archive ledger). Roll every
      // already-moved file back to its origin, in reverse order, before
      // rethrowing — best-effort per file so one stuck rename during
      // rollback never masks the original failure.
      for (const entry of moved.reverse()) {
        try {
          fs.renameSync(entry.to, entry.from);
        } catch {
          // best-effort: the original error below is what the caller needs
        }
      }
      // hardening-1-7-10 (D4): this in-process rollback already fully
      // unwound the transaction, so the journal has nothing left to recover
      // — clear it now rather than leaving a stale file for the next call's
      // recovery sweep to redundantly (but harmlessly) no-op through.
      clearArchiveJournal(root, feature);
      throw err;
    }
    // Written LAST, only once every rename above has succeeded — summary.json
    // must never advance ahead of what is actually on disk.
    const summary = archivedSummary(root);
    const counts = { capped, dropped };
    summary[feature] = { ...counts, archived_at: utcNow() };
    writeJsonAtomic(archiveSummaryFile(root), summary);
    // Deleted only after the summary write above succeeds (D4) — until then
    // the journal is the only record that this transaction is still live.
    clearArchiveJournal(root, feature);
    return { feature, moved: moved.map((entry) => entry.id), counts };
  });
}

// unarchiveFeature(root, feature) — the reverse of archiveFeature: moves
// every .bee/cells/archive/<feature>/*.json cell back to the active
// .bee/cells/ dir, drops that feature's summary.json entry, and removes the
// now-empty archive/<feature> dir (best-effort — a leftover non-.json file
// left by something else never blocks the unarchive itself). Refuses
// (throws) when the feature has nothing archived, or when any planned
// active destination already exists (D4 — never overwrite live data).
export async function unarchiveFeature(root, feature) {
  assertValidFeatureSlug('unarchiveFeature', feature);
  // hardening-1: same whole-store lock as archiveFeature, same reserved name
  // — archive/unarchive on the same feature (or a concurrent mutator racing
  // an in-flight archive) must never interleave.
  return withStoreLock(root, 'cells-archive', () => {
    // hardening-1-7-10 (D4): same entry-point recovery sweep as
    // archiveFeature — repair a half-migrated leftover before proceeding.
    recoverArchiveJournal(root, feature);
    const archiveDir = cellsArchiveDir(root, feature);
    assertArchiveDirContained('unarchiveFeature', root, archiveDir);
    let files;
    try {
      files = fs.readdirSync(archiveDir);
    } catch {
      throw new Error(`unarchiveFeature: no archived cells found for feature "${feature}".`);
    }
    // ARCHIVE_JOURNAL_FILE also ends in .json but is never a cell — excluded
    // explicitly (recoverArchiveJournal above already cleared any leftover
    // journal, so in practice none survives to this point, but the filter
    // stays defensive rather than assuming that).
    const jsonFiles = files.filter((file) => file.endsWith('.json') && file !== ARCHIVE_JOURNAL_FILE);
    if (jsonFiles.length === 0) {
      throw new Error(`unarchiveFeature: no archived cells found for feature "${feature}".`);
    }
    const planned = jsonFiles.map((file) => ({
      id: file.slice(0, -'.json'.length),
      from: path.join(archiveDir, file),
      to: path.join(cellsDir(root), file),
    }));
    // hardening-1-7-10 (D4): refuse to overwrite an existing ACTIVE file —
    // preflight BEFORE any rename, named, nothing touched on a collision.
    const collisions = planned.filter((move) => fs.existsSync(move.to)).map((move) => move.id);
    if (collisions.length > 0) {
      throw new ArchiveDestinationCollisionError('unarchiveFeature', feature, collisions, 'active');
    }
    writeArchiveJournal(root, feature, 'unarchive', planned);
    const moved = [];
    try {
      for (const move of planned) {
        fs.renameSync(move.from, move.to);
        moved.push(move);
      }
    } catch (err) {
      // hardening-1: same rollback discipline as archiveFeature — restore
      // every already-moved file to the archive before rethrowing.
      for (const entry of moved.reverse()) {
        try {
          fs.renameSync(entry.to, entry.from);
        } catch {
          // best-effort: the original error below is what the caller needs
        }
      }
      clearArchiveJournal(root, feature); // D4: already fully unwound in-process
      throw err;
    }
    // The journal's job ends once every rename above lands — clear it before
    // rmdir so the now-empty archiveDir can actually be removed.
    clearArchiveJournal(root, feature);
    try {
      fs.rmdirSync(archiveDir);
    } catch {
      // best-effort: a non-.json leftover (or a concurrent write) keeps the
      // dir non-empty — the cells themselves are already moved back, which is
      // the part that matters.
    }
    // Written LAST, only once every rename above has succeeded.
    const summary = archivedSummary(root);
    delete summary[feature];
    writeJsonAtomic(archiveSummaryFile(root), summary);
    return moved.map((entry) => entry.id);
  });
}

// writeCell — the single write funnel every mutator in this file goes
// through (hardening-1-7-10, D4). MUST STAY SYNCHRONOUS: addCells maps it
// synchronously over a batch (`normalized.map((cell) => writeCell(root,
// cell))`, no awaited cascade), so this can never become `async` without
// breaking that call site's return shape (an array of promises instead of
// cells). That is why the archive-boundary guard below uses
// acquireStoreLockOnceSync — a SYNC, single-attempt O_EXCL acquire, never
// withStoreLock's async retry/backoff loop.
//
// The critical section is deliberately tiny: acquire 'cells-archive' (the
// SAME lock name archiveFeature/unarchiveFeature hold for their whole
// transaction), re-check archived-only status ONE LAST TIME under that lock,
// write, release. Lock order is always `cells:<id>` -> `cells-archive` —
// every caller here already holds `cells:<id>` (or, for addCell/addCells,
// no per-cell lock at all) before ever reaching writeCell, and
// archiveFeature/unarchiveFeature never take `cells:<id>` and never call
// writeCell (they renameSync the file directly) — so this can never
// deadlock and never re-enters its own lock.
//
// On contention (a live archive/unarchive transaction holding
// 'cells-archive' right now) this throws typed CELLS_ARCHIVE_BUSY rather
// than waiting — archive transactions are short, so busy is rare and an
// honest refusal beats a hidden multi-second stall inside every cell write.
//
// The re-check closes the TOCTOU a plain assertNotArchived-at-entry left
// open: a mutator's own assertNotArchived ran before it read+modified the
// cell, but archiveFeature could complete its move in the window between
// that check and this write — without a check HELD UNDER THE SAME LOCK
// archive uses, the mutator's write would resurrect a brand-new active file
// for a cell that is now archived. Never taking `cells:<id>` itself,
// archiveFeature/unarchiveFeature can only ever run their rename either
// fully before or fully after this critical section, never during it — so
// this check, taken under the lock, is authoritative.
export function writeCell(root, cell) {
  if (!cell || !cell.id || !ID_PATTERN.test(String(cell.id))) {
    throw new Error(`writeCell: cell needs a valid id (got ${JSON.stringify(cell?.id)}).`);
  }
  const lock = acquireStoreLockOnceSync(root, 'cells-archive');
  if (!lock.acquired) {
    throw new CellsArchiveBusyError(cell.id, lock.holder);
  }
  try {
    const active = cellFile(root, cell.id);
    if (!fs.existsSync(active) && resolveCellFile(root, cell.id)) {
      throw new CellArchivedError('writeCell', cell.id);
    }
    writeJsonAtomic(active, cell);
    return cell;
  } finally {
    lock.release();
  }
}

// no-test-repos D1/D2 (decision 55b951e1) — the sentinel is a legal `verify`
// VALUE only in a repo that has declared itself no-test (isNoTestRepo reads
// commands.verify/commands.test from .bee/config.json); everywhere else it
// is refused exactly like any other non-command prose string would be.
// Shared by addCell/addCells (validateNewCell) and updateCell below so both
// entry points enforce the same rule the same way.
function assertVerifySentinelAllowed(root, verb, verifyValue) {
  if (verifyValue !== NO_TEST_SENTINEL) return;
  if (isNoTestRepo(readConfig(root))) return;
  throw new Error(
    `${verb}: verify "${NO_TEST_SENTINEL}" is refused — this repo has not declared itself a no-test repo. FIX: use a real, runnable verify command, or declare the repo no-test first by setting commands.verify (or commands.test) to "${NO_TEST_SENTINEL}" in .bee/config.json (decision 55b951e1).`,
  );
}

function validateNewCell(root, cell) {
  if (!cell || typeof cell !== 'object' || Array.isArray(cell)) {
    throw new Error('addCell: cell must be a JSON object.');
  }
  for (const field of ['id', 'feature', 'title', 'action', 'verify']) {
    if (typeof cell[field] !== 'string' || !cell[field].trim()) {
      throw new Error(`addCell: cell is missing required field "${field}" (non-empty string).`);
    }
  }
  assertVerifySentinelAllowed(root, 'addCell', cell.verify);
  if (!ID_PATTERN.test(cell.id)) {
    throw new Error(
      `addCell: invalid id "${cell.id}" — use letters, digits, dot, dash, underscore (e.g. "auth-3").`,
    );
  }
  if (!LANES.includes(cell.lane)) {
    throw new Error(
      `addCell: invalid lane "${cell.lane}" — must be one of: ${LANES.join(', ')}.`,
    );
  }
  if (cell.lane === 'standard' || cell.lane === 'high-risk') {
    const truths = cell.must_haves && cell.must_haves.truths;
    if (!Array.isArray(truths) || truths.length === 0) {
      throw new Error(
        `addCell: lane "${cell.lane}" requires non-empty must_haves.truths (observable truths to verify).`,
      );
    }
  }
  // D9: optional pbi field references a backlog id — persisted verbatim, no
  // validation coupling (a missing/stale reference is a grooming find, never a
  // cap/claim blocker). Only reject an outright non-string value.
  if (cell.pbi !== undefined && cell.pbi !== null && typeof cell.pbi !== 'string') {
    throw new Error('addCell: optional "pbi" must be a string backlog id when present.');
  }
  // D11/D12: optional model tier — planning assigns it so swarming can resolve
  // tier → model and the harness can keep the ceiling model scarce (P7). Absent
  // = untiered (never a blocker); a present value must be a known tier.
  if (cell.tier !== undefined && cell.tier !== null && !MODEL_TIERS.includes(cell.tier)) {
    throw new Error(
      `addCell: optional "tier" must be one of ${MODEL_TIERS.join(', ')} when present.`,
    );
  }
  // D3 (self-correcting-loop): optional judge-standard classification — the
  // enum itself is a hard validation (same shape as tier/lane above); the
  // MATRIX check that reads it is advisory-only and lives in bee.mjs (F4).
  if (
    cell.change_class !== undefined &&
    cell.change_class !== null &&
    !CHANGE_CLASSES.includes(cell.change_class)
  ) {
    throw new Error(
      `addCell: optional "change_class" must be one of ${CHANGE_CLASSES.join(', ')} when present.`,
    );
  }
  // GH #27.3 (D-GHF-C): an authored `budgets` object is validated strictly —
  // unlike resolveCellBudgets's forgiving runtime fallback (default on bad
  // input, clamp on over-ceiling input), a malformed value here is refused
  // outright so a typo at authoring time surfaces immediately instead of
  // silently degrading at claim time.
  if (cell.budgets !== undefined && cell.budgets !== null) {
    if (typeof cell.budgets !== 'object' || Array.isArray(cell.budgets)) {
      throw new Error('addCell: optional "budgets" must be a plain object when present.');
    }
    const knownKeys = Object.keys(DEFAULT_BUDGETS);
    for (const key of Object.keys(cell.budgets)) {
      if (!knownKeys.includes(key)) {
        throw new Error(
          `addCell: unknown "budgets" key "${key}" — must be one of: ${knownKeys.join(', ')}.`,
        );
      }
    }
    for (const key of knownKeys) {
      if (!(key in cell.budgets)) continue;
      const value = cell.budgets[key];
      const hardMax = BUDGET_HARD_MAX[key];
      if (!Number.isInteger(value) || value < 1 || value > hardMax) {
        throw new Error(
          `addCell: "budgets.${key}" must be an integer in [1, ${hardMax}] when present, got ${JSON.stringify(value)}.`,
        );
      }
    }
  }
  // D1: the escape hatch is a REASON, not a boolean — a bare `true` records
  // that someone skipped, never why, and the trace is the only place anyone
  // will ever read it back.
  if (cell[REGEN_ACK_FIELD] !== undefined && cell[REGEN_ACK_FIELD] !== null) {
    if (typeof cell[REGEN_ACK_FIELD] !== 'string' || !cell[REGEN_ACK_FIELD].trim()) {
      throw new Error(
        `addCell: optional "${REGEN_ACK_FIELD}" must be a non-empty string (the one-line reason the derived regen obligation is being skipped).`,
      );
    }
  }
  if (readCell(root, cell.id)) {
    throw new Error(`addCell: cell "${cell.id}" already exists.`);
  }
  // D1/D2 — derived regen obligation, last because it is the only check here
  // that reads outside the cell (the guard scripts).
  assertRegenObligation(root, cell, 'addCell');
}

function normalizeNewCell(cell) {
  return {
    ...cell,
    status: cell.status || 'open',
    deps: Array.isArray(cell.deps) ? cell.deps : [],
    decisions: Array.isArray(cell.decisions) ? cell.decisions : [],
    files: Array.isArray(cell.files) ? cell.files : [],
    read_first: Array.isArray(cell.read_first) ? cell.read_first : [],
    trace: { ...defaultTrace(), ...(cell.trace || {}) },
  };
}

// assertNoCycle (D2) — the ONE cycle refusal used at every dep-mutating
// write: addCell, addCells, updateCell (when it changes deps). Union of ALL
// on-disk cells (every status — detectCycles is a structural check, not a
// schedulability one; overlap stays legal and is never checked here) with the
// incoming set (new cells being added, or an existing cell carrying a patched
// `deps`), overlaid by id so a batch/patch that also touches an on-disk id
// sees its OWN version, not the stale disk copy. Must run BEFORE any
// writeCell — a refusal here must never leave partial state behind (addCells
// stays all-or-nothing; addCell/updateCell touch nothing on a refusal).
// Refusal is scoped to cycles the WRITE introduces or participates in: a
// pre-existing cycle among untouched on-disk cells never blocks unrelated
// writes — per D2 those are reported by `cells schedule` diagnostics, not
// enforced here.
// computeIncomingCycles (D2) — pure, non-throwing core of the cycle check:
// union of ALL on-disk cells (every status — detectCycles is a structural
// check, not a schedulability one; overlap stays legal and is never checked
// here) with the incoming set (new cells being added, or an existing cell
// carrying a patched `deps`), overlaid by id so a batch/patch that also
// touches an on-disk id sees its OWN version, not the stale disk copy.
// Returns only the cycles that touch an incoming id — a pre-existing cycle
// among untouched on-disk cells is never reported here (per D2, that is
// `cells schedule` diagnostics territory). Factored out of assertNoCycle
// (ce-2) so `cells add --dry-run`'s report can fold cycle failures into the
// SAME per-cell verdicts the write path validates against, without either
// side re-implementing cycle detection.
function computeIncomingCycles(root, incomingCells) {
  const byId = new Map();
  for (const cell of listCells(root)) {
    if (cell && typeof cell.id === 'string' && cell.id) byId.set(cell.id, cell);
  }
  const incomingIds = new Set();
  for (const cell of incomingCells) {
    if (cell && typeof cell.id === 'string' && cell.id) {
      byId.set(cell.id, cell);
      incomingIds.add(cell.id);
    }
  }
  return detectCycles([...byId.values()]).filter((cycle) => cycle.some((id) => incomingIds.has(id)));
}

function formatCycleRefusal(verb, cycles) {
  const named = cycles.map((cycle) => cycle.join(' -> ')).join('; ');
  return `${verb}: dependency cycle refused — ${named}. Cycles are illegal at every dep-mutating write (D2); file overlap stays legal and is never refused.`;
}

// assertNoCycle (D2) — the ONE cycle refusal used at every dep-mutating
// write: addCell, addCells (write path), updateCell (when it changes deps).
// Must run BEFORE any writeCell — a refusal here must never leave partial
// state behind (addCells stays all-or-nothing; addCell/updateCell touch
// nothing on a refusal). Unchanged by ce-2: same detection, same message.
function assertNoCycle(root, verb, incomingCells) {
  const cycles = computeIncomingCycles(root, incomingCells);
  if (cycles.length > 0) {
    throw new Error(formatCycleRefusal(verb, cycles));
  }
}

export function addCell(root, cell) {
  validateNewCell(root, cell);
  const normalized = normalizeNewCell(cell);
  assertNoCycle(root, 'addCell', [normalized]);
  return writeCell(root, normalized);
}

// buildAddCellsReport (D2/ce-2) — the ONE aggregator both addCells' all-or-
// nothing write path and `cells add --dry-run`'s no-write preview build on.
// Runs EVERY per-cell validateNewCell (schema + regen obligations included)
// and the duplicate-id check across the WHOLE array before returning — a
// multi-cell payload never needs re-sending to discover the next error,
// unlike the pre-ce-2 code that threw at the first bad cell in the loop.
// Only once every cell clears its own validation does the batch-wide cycle
// check (D2, same computeIncomingCycles detection assertNoCycle always used)
// run, folded into the SAME per-cell problems array so a cycle spanning
// otherwise-valid cells names exactly the cells it touches. Never writes,
// never throws — callers decide whether to throw (addCells) or report
// (previewAddCells).
function buildAddCellsReport(root, cells) {
  const seen = new Set();
  const entries = cells.map((cell, index) => {
    const id = cell && typeof cell.id === 'string' && cell.id ? cell.id : `(index ${index})`;
    const problems = [];
    try {
      validateNewCell(root, cell);
    } catch (err) {
      problems.push(err instanceof Error ? err.message : String(err));
    }
    if (cell && typeof cell.id === 'string' && cell.id) {
      if (seen.has(cell.id)) {
        problems.push(`addCells: duplicate id "${cell.id}" within the batch.`);
      } else {
        seen.add(cell.id);
      }
    }
    return { id, cell, problems };
  });
  let normalized = null;
  if (entries.every((entry) => entry.problems.length === 0)) {
    normalized = entries.map((entry) => normalizeNewCell(entry.cell));
    const cycles = computeIncomingCycles(root, normalized);
    if (cycles.length > 0) {
      const cycleIds = new Set(cycles.flat());
      const message = formatCycleRefusal('addCells', cycles);
      for (const entry of entries) {
        if (cycleIds.has(entry.id)) entry.problems.push(message);
      }
      normalized = null;
    }
  }
  const ok = entries.every((entry) => entry.problems.length === 0);
  return {
    ok,
    cells: entries.map((entry) => ({ id: entry.id, ok: entry.problems.length === 0, problems: entry.problems })),
    normalized: ok ? normalized : null,
  };
}

// Batch add: validates EVERY cell of the WHOLE array (schema, regen
// obligations, duplicate ids, then — only once every cell is individually
// clean — the batch-wide cycle check) before writing any — all-or-nothing,
// so a failing cell anywhere in a slice never leaves partial state behind.
// ce-2: the refusal on failure names EVERY failing cell and EVERY problem in
// one message (buildAddCellsReport), not just the first cell the old
// first-throw-wins loop happened to reach.
export function addCells(root, cells) {
  if (!Array.isArray(cells) || cells.length === 0) {
    throw new Error('addCells: expected a non-empty JSON array of cells.');
  }
  const report = buildAddCellsReport(root, cells);
  if (!report.ok) {
    const failing = report.cells.filter((c) => !c.ok);
    const named = failing.map((c) => `${c.id} (${c.problems.join('; ')})`).join(', ');
    throw new Error(
      `addCells: ${failing.length} of ${cells.length} cell(s) failed validation — ${named}. Nothing written.`,
    );
  }
  return report.normalized.map((cell) => writeCell(root, cell));
}

// previewAddCells (D2/ce-2) — the non-throwing, non-writing sibling of
// addCells behind `cells add --dry-run`: same whole-array aggregate
// validation (buildAddCellsReport), reported instead of thrown, so a caller
// can discover every problem in a batch without ever risking a write. The
// CLI handler wraps a single-object payload as a one-element array before
// calling this — `cells` here is always an array.
export function previewAddCells(root, cells) {
  if (!Array.isArray(cells) || cells.length === 0) {
    throw new Error('previewAddCells: expected a non-empty JSON array of cells.');
  }
  const report = buildAddCellsReport(root, cells);
  return { ok: report.ok, cells: report.cells };
}

// ─── updateCell — door-validated in-place revision (cells-update-verb) ─────
// Validation repair loops legitimately revise a cell after creation (a plan
// checker or cell reviewer prescribes a fix). Before this verb the only path
// was rule 11's hand-edit fallback, which renders full JSON diffs into the
// user's working view — the exact noise the CLI-owned-state contract
// (decision bb4bb18e) removed for state.json/backlog.jsonl.
//
// The field list is derived FROM the validator map (critical pattern
// 20260710: a boundary that lists names leaks the field you forgot — an
// unmapped key is a refusal, not a pass-through). Frozen surfaces are named
// in the refusal so the caller learns the right verb: status/trace belong to
// claim/verify/cap/block/drop, tier to the tier verb, id/feature to nothing.

const UPDATE_FIELD_VALIDATORS = {
  title: (v) => (typeof v === 'string' && v.trim() ? null : 'must be a non-empty string'),
  action: (v) => (typeof v === 'string' && v.trim() ? null : 'must be a non-empty string'),
  verify: (v) => (typeof v === 'string' && v.trim() ? null : 'must be a non-empty string'),
  files: (v) => (isStringArray(v) ? null : 'must be an array of strings'),
  read_first: (v) => (isStringArray(v) ? null : 'must be an array of strings'),
  deps: (v) => (isStringArray(v) ? null : 'must be an array of strings'),
  decisions: (v) => (isStringArray(v) ? null : 'must be an array of strings'),
  must_haves: (v) =>
    v && typeof v === 'object' && !Array.isArray(v) ? null : 'must be a JSON object',
  behavior_change: (v) => (typeof v === 'boolean' ? null : 'must be a boolean'),
  lane: (v) => (LANES.includes(v) ? null : `must be one of: ${LANES.join(', ')}`),
  pbi: (v) => (v === null || typeof v === 'string' ? null : 'must be a string or null'),
  // D3: nullable so a cell can un-set an explicit change_class back to
  // "derive from behavior_change" — same null-allowed shape as pbi above.
  change_class: (v) =>
    v === null || CHANGE_CLASSES.includes(v) ? null : `must be null or one of: ${CHANGE_CLASSES.join(', ')}`,
  // D1: updatable so a repair loop can either satisfy the derived regen
  // obligation (patch verify/files) or record the deliberate skip — nullable
  // to un-set it again, same shape as pbi/change_class above.
  [REGEN_ACK_FIELD]: (v) =>
    v === null || (typeof v === 'string' && v.trim())
      ? null
      : 'must be null or a non-empty string (the one-line reason for skipping the derived regen obligation)',
};

function isStringArray(value) {
  return Array.isArray(value) && value.every((item) => typeof item === 'string');
}

const UPDATE_FROZEN_HINTS = {
  id: 'a cell id is permanent — add a new cell instead',
  feature: 'a cell never moves between features — drop and re-add instead',
  status: 'status moves only through claim/verify/cap/block/drop',
  trace: 'the trace is the frozen audit record — claim/verify/cap own it',
  tier: 'use the tier verb (bee.mjs cells tier --id ID --tier T)',
};

// Strict read for the update path only (readReviewStrict/readStateStrict
// pattern): fail-open reads elsewhere are untouched; a write verb must never
// merge a patch into defaults over a present-but-corrupt file.
function readCellStrictForUpdate(root, id) {
  const file = cellFile(root, id);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') {
      throw new Error(`updateCell: cell "${id}" not found.`);
    }
    throw new Error(
      `updateCell: could not read "${file}" (${err && err.code ? err.code : err}) — refusing to touch it. FIX: inspect/restore the file, then retry.`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error(
      `updateCell: "${file}" exists but is not valid JSON — refusing to merge a patch over a corrupt cell. FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(
      `updateCell: "${file}" exists but is not a JSON object — refusing to merge a patch over a corrupt cell.`,
    );
  }
  return parsed;
}

// hardening-4b: the read-check-write body (readCellStrictForUpdate through
// writeCell) now runs inside withStoreLock(`cells:${id}`) — the audit finding
// was that updateCell's status/merge check was not CAS-protected against a
// concurrent claim/unclaim/reopen. Pure, store-agnostic validation (id shape,
// patch shape, per-field validators) stays OUTSIDE the lock — it never reads
// or writes the store, so there is nothing for it to race.
export async function updateCell(root, id, patch) {
  if (!id || !ID_PATTERN.test(String(id))) {
    throw new Error(`updateCell: invalid id "${id}".`);
  }
  if (!patch || typeof patch !== 'object' || Array.isArray(patch)) {
    throw new Error('updateCell: patch must be a JSON object.');
  }
  const keys = Object.keys(patch);
  if (keys.length === 0) {
    throw new Error('updateCell: patch is empty — nothing to update.');
  }
  for (const key of keys) {
    const validator = UPDATE_FIELD_VALIDATORS[key];
    if (!validator) {
      const hint = UPDATE_FROZEN_HINTS[key];
      throw new Error(
        hint
          ? `updateCell: field "${key}" is frozen — ${hint}. The whole patch is refused; the cell is untouched.`
          : `updateCell: unknown field "${key}" — updatable fields: ${Object.keys(UPDATE_FIELD_VALIDATORS).join(', ')}. The whole patch is refused; the cell is untouched.`,
      );
    }
    const problem = validator(patch[key]);
    if (problem) {
      throw new Error(
        `updateCell: field "${key}" ${problem}. The whole patch is refused; the cell is untouched.`,
      );
    }
  }
  // no-test-repos D1/D2: same sentinel gate addCell enforces, applied to a
  // patch that sets/changes `verify`.
  if (Object.prototype.hasOwnProperty.call(patch, 'verify')) {
    assertVerifySentinelAllowed(root, 'updateCell', patch.verify);
  }

  return withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'updateCell', id); // hardening-1: refuse before ever reading/writing
    const cell = readCellStrictForUpdate(root, id);
    if (cell.status !== 'open' && cell.status !== 'blocked') {
      throw new Error(
        `updateCell: cell "${id}" has status "${cell.status}" — only open or blocked cells are updatable (claimed = a live worker owns it; capped/dropped = frozen audit). The cell is untouched.`,
      );
    }

    const merged = { ...cell, ...patch };
    if (merged.lane === 'standard' || merged.lane === 'high-risk') {
      const truths = merged.must_haves && merged.must_haves.truths;
      if (!Array.isArray(truths) || truths.length === 0) {
        throw new Error(
          `updateCell: lane "${merged.lane}" requires non-empty must_haves.truths — the patch would leave "${id}" without them. The cell is untouched.`,
        );
      }
    }
    // D2: only a patch that changes `deps` can reintroduce a cycle — checking
    // unconditionally would re-validate every unrelated field edit against the
    // whole store for no reason. Runs before writeCell; a refusal leaves the
    // cell untouched, same guarantee as every other updateCell refusal above.
    if (Object.prototype.hasOwnProperty.call(patch, 'deps')) {
      assertNoCycle(root, 'updateCell', [merged]);
    }
    // D1/D2 — the derived regen obligation is checked against the MERGED cell:
    // a patch that adds a hashed-root path to `files` owes the same checks a
    // fresh cell would, and a patch that fixes `verify` clears it. Refusal
    // leaves the cell untouched, same guarantee as every refusal above.
    assertRegenObligation(root, merged, 'updateCell');
    return writeCell(root, merged);
  });
}

function depsAllCapped(root, cell) {
  const missing = [];
  for (const dep of cell.deps || []) {
    const depCell = readCell(root, dep);
    if (!depCell || depCell.status !== 'capped') missing.push(dep);
  }
  return missing;
}

export function readyCells(root, feature = null) {
  return listCells(root, { feature, status: 'open' }).filter(
    (cell) => depsAllCapped(root, cell).length === 0,
  );
}

// fsh-5 (D2/D4) — the cell-feature → lane-record read for ENFORCEMENT. The
// per-feature lane is keyed by cell.feature (NAMING TRAP: the cell field named
// `lane` is the risk tier — tiny/small/standard/high-risk — a different thing
// entirely). A path-shaped feature can never name a lane record (null — the
// default gate governs, exactly today's behavior); a missing record is null;
// a present-but-corrupt record THROWS via readLaneStrict — the default gate
// must never silently authorize a lane cell over a corrupt lane record.
function laneRecordForFeature(root, feature) {
  if (typeof feature !== 'string' || !feature.trim()) return null;
  let file;
  try {
    file = lanePath(root, feature);
  } catch {
    return null;
  }
  if (!fs.existsSync(file)) return null;
  return readLaneStrict(root, feature);
}

// hardening-4b: the whole read-check-write (gate resolution through
// writeCell) now runs inside withStoreLock(`cells:${id}`) — the audit finding
// was that the status flip was not CAS-protected against a concurrent
// unclaim/reopen/drop. `sessionId` (optional) is stamped onto
// trace.claim_session — a NEW top-level trace field, distinct from the
// per-attempt ledger's own claim_session (appendAttempt) — so
// claims.mjs's sweepExpiredClaims can later verify, before resetting a swept
// cell back to open, that the cell's CURRENT claim still matches the exact
// session whose claim file it just removed (never resetting a cell some
// OTHER, fresher claim already owns). claimCellCrossSession is the only
// production caller that supplies sessionId; a bare/direct claimCell call
// (tests, or any future sessionless caller) leaves it null, same shape a
// sessionless claim already uses elsewhere in this module.
export async function claimCell(root, id, worker, { sessionId } = {}) {
  if (typeof worker !== 'string' || !worker.trim()) {
    throw new Error('claimCell: worker name is required.');
  }
  return withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'claimCell', id); // hardening-1: refuse before ever reading/writing
    // fsh-5 (D2): the execution gate resolves from the CELL's own feature — a
    // lane record for cell.feature authorizes (or refuses) the claim with ITS
    // gate; no lane record means the default pipeline's gate, byte-identical to
    // the single-pipeline model (D4 zero-lane parity). A missing cell resolves
    // through the default gate so the error precedence (gate first, then
    // not-found) matches today exactly.
    const cell = readCell(root, id);
    const laneRecord = cell ? laneRecordForFeature(root, cell.feature) : null;
    const gateSource = laneRecord || readState(root);
    if (!gateApproved(gateSource, 'execution')) {
      throw new Error(
        laneRecord
          ? `claimCell: lane "${cell.feature}" gate "execution" is not approved — cells of this feature cannot be claimed before ITS lane passes Gate 3 (D2: only the lane's own approvals authorize its cells — the default pipeline's gate never does). Surface Gate 3 to the user for lane "${cell.feature}" and set its approved_gates.execution once approved.`
          : 'claimCell: gate "execution" is not approved — cells cannot be claimed before execution is approved. Surface Gate 3 to the user ("Feasibility validated. Approve execution?") and set approved_gates.execution once approved. The opt-in gate_bypass switch may self-approve: level "normal" covers tiny/small/standard non-hard-gate work only; levels "full" and "total" also self-approve high-risk/hard-gate execution (decision 0010, total-autopilot dcf01d7b).',
      );
    }
    if (!cell) throw new Error(`claimCell: cell "${id}" not found.`);
    if (cell.status !== 'open') {
      throw new Error(
        `claimCell: cell "${id}" is "${cell.status}", not "open" — only open cells can be claimed. Run bee.mjs cells ready to list claimable cells.`,
      );
    }
    const uncapped = depsAllCapped(root, cell);
    if (uncapped.length > 0) {
      throw new Error(
        `claimCell: cell "${id}" has uncapped deps: ${uncapped.join(', ')} — deps must be capped first.`,
      );
    }
    cell.status = 'claimed';
    cell.trace = {
      ...defaultTrace(),
      ...(cell.trace || {}),
      worker: worker.trim(),
      claim_session: sessionId ?? null,
    };
    cell.trace.claimed_at = utcNow();
    return writeCell(root, cell);
  });
}

export async function recordVerify(
  root,
  id,
  { command, output = null, passed, sessionId, forceOwnership = false, signature = null },
) {
  if (typeof command !== 'string' || !command.trim()) {
    throw new Error('recordVerify: command is required.');
  }
  if (typeof passed !== 'boolean') {
    throw new Error('recordVerify: passed must be true or false.');
  }
  return withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'recordVerify', id); // hardening-1: refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`recordVerify: cell "${id}" not found.`);
    let trace = { ...defaultTrace(), ...(cell.trace || {}) };
    trace = guardClaimOwnership(root, id, trace, 'recordVerify', { sessionId, forceOwnership }); // D4
    trace.verify_command = command;
    trace.verify_output = output;
    trace.verify_passed = passed;
    trace.verified_at = utcNow();
    // D1: worker-suppliable --signature wins over the mechanical fallback; a
    // passing verify never carries a failure signature at all.
    const failureSignature = passed
      ? null
      : typeof signature === 'string' && signature.trim()
        ? signature.trim()
        : normalizeFailureSignature(output);
    trace = appendAttempt(root, id, trace, { verdict: passed ? 'pass' : 'fail', failureSignature });
    cell.trace = trace;
    return writeCell(root, cell);
  });
}

// D3+Δ5 — the anti-boilerplate floor for behavior-class cap teeth: 80 chars,
// and never byte-identical (sha256 of the trimmed text) to another cell's own
// recorded red evidence. Reuses listCells' own tolerant readJson (Δ5: "the
// cap-time duplicate scan tolerant-parses — skips unparseable sibling files,
// never throws") instead of a second hand-rolled readdir/parse loop.
const RED_EVIDENCE_MIN_CHARS = 80;

// test-economy D3: the new-test-file justification floor and the
// ratio-ceiling waiver floor share RED_EVIDENCE_MIN_CHARS's "evidence must be
// JSON with this named field" shape above, just a lower bar (20 chars, a
// one-line reason — no anti-boilerplate/anti-duplicate check like red
// evidence gets). RATIO_WARN_CEILING/RATIO_REFUSE_CEILING are the "Ngưỡng &
// pin chốt" numbers pinned in plan.md: tiny/small warns above 3, standard/
// high-risk refuses above 4.
const NEW_SUITE_REASON_MIN_CHARS = 20;
const RATIO_WAIVER_MIN_CHARS = 20;
const RATIO_WARN_CEILING = 3;
const RATIO_REFUSE_CEILING = 4;

function findDuplicateRedEvidence(root, id, trimmed) {
  const hash = crypto.createHash('sha256').update(trimmed, 'utf8').digest('hex');
  for (const sibling of listCells(root)) {
    if (!sibling || sibling.id === id) continue;
    const evidence = parseVerificationEvidence(sibling.trace && sibling.trace.verification_evidence);
    const red = typeof evidence.red_failure_evidence === 'string' ? evidence.red_failure_evidence.trim() : '';
    if (red.length < RED_EVIDENCE_MIN_CHARS) continue;
    if (crypto.createHash('sha256').update(red, 'utf8').digest('hex') === hash) return sibling.id;
  }
  return null;
}

// E6 (derived-check-hardening): cells are commonly authored with
// behavior_change nested under trace.behavior_change rather than the
// top-level field validated by cells add/update — so a genuine behavior
// change was capping with the flag false (live evidence: vd-9, vd-10,
// vd-11). Resolve ONCE here, at the cap read, so no downstream consumer
// needs its own fallback: an explicitly-set top-level true/false always
// wins; only when the top-level field is genuinely unset (undefined) does
// the nested trace value stand in. Forward-only — this never touches an
// already-capped cell's stored record.
function resolveDeclaredBehaviorChange(cell) {
  if (cell.behavior_change === true) return true;
  if (cell.behavior_change === false) return false;
  return Boolean(cell.trace && cell.trace.behavior_change === true);
}

export async function capCell(
  root,
  id,
  {
    files_changed = [],
    deviations = [],
    friction = null,
    behavior_change,
    verification_evidence = null,
    outcome,
    sessionId,
    forceOwnership = false,
    overrideJudge = null,
    // test-economy D1: computed by the bee.mjs `cells cap` handler (the sole
    // caller — capCell itself has no child_process/git dependency) from
    // `files_changed` via git. undefined (the default — legacy callers,
    // tests that don't pass it, or a git-lookup failure at the handler)
    // means every diff_stats-driven check below skips outright: fail-open,
    // never a false refusal for a repo/handler that couldn't compute it.
    diff_stats = undefined,
    // main-verifies D1: cap through the feature-verify-pending path — no
    // per-cell verify evidence demanded, trace.feature_verify stamped
    // "pending". Default false keeps the classic evidence path byte-unchanged.
    feature_verify_pending = false,
  } = {},
) {
  const overrideReason = typeof overrideJudge === 'string' ? overrideJudge.trim() : '';
  // dch-8: async so the E1 impact-registry cross-check below can `await
  // import(...)` its optional dependency lazily instead of a static
  // top-level import — withStoreLock already does `return await fn()`, so
  // an async callback here changes nothing about the lock's semantics.
  const saved = await withStoreLock(root, `cells:${id}`, async () => {
    assertNotArchived(root, 'capCell', id); // hardening-1: refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`capCell: cell "${id}" not found.`);
    // Honor the cell's declared behavior_change when the caller omits it — the CLI
    // flag is opt-in, so a cell planned as behavior_change must not silently lose
    // its evidence/before-state guards (and its scribing debt) at cap just because
    // --behavior-change was not repeated. Explicit false/true from the caller wins.
    // E6: the declared value itself may live at the top level or nested under
    // trace.behavior_change — see resolveDeclaredBehaviorChange.
    const bc =
      behavior_change === undefined ? resolveDeclaredBehaviorChange(cell) : behavior_change === true;
    if (cell.status === 'capped') throw new Error(`capCell: cell "${id}" is already capped.`);
    if (cell.status === 'dropped') throw new Error(`capCell: cell "${id}" was dropped.`);
    let trace = { ...defaultTrace(), ...(cell.trace || {}) };
    trace = guardClaimOwnership(root, id, trace, 'capCell', { sessionId, forceOwnership }); // D4
    // main-verifies D1: the sanctioned feature-verify-pending cap path. The
    // cell caps with NO per-cell verify evidence at all, stamping
    // trace.feature_verify: "pending" — its proof event is deliberately
    // relocated to ONE feature-level verify (recorded via `bee state
    // feature-verify record`), and the close-door guard in bee.mjs
    // (guardFeatureVerifyDebt, D3) is what keeps that relocation honest: no
    // departure from phase "swarming" while any pending marker lacks a
    // fresh GREEN feature-verify record. Combining the flag with per-cell
    // evidence claims is REFUSED — the two proof paths are exclusive, and a
    // cap must never be ambiguous about which one vouched for it. The
    // classic evidence path below is byte-unchanged when the flag is off.
    const pendingFeatureVerify = feature_verify_pending === true;
    if (pendingFeatureVerify) {
      const evidenceSupplied =
        verification_evidence != null &&
        (typeof verification_evidence !== 'string' || verification_evidence.trim().length > 0);
      if (evidenceSupplied) {
        throw new Error(
          `capCell: cell "${id}" cannot combine --feature-verify-pending with verification_evidence — the pending path defers ALL per-cell proof to the feature-level verify (main-verifies D1), so attaching per-cell evidence contradicts the deferral. FIX: drop --evidence-file/--evidence-stdin, or cap through the classic evidence path without --feature-verify-pending.`,
        );
      }
      if (trace.verify_passed === true) {
        throw new Error(
          `capCell: cell "${id}" cannot combine --feature-verify-pending with a recorded PASSING verify — a green per-cell verify IS the classic evidence path (main-verifies D1); the pending marker would falsely defer proof this cell already holds. FIX: cap without --feature-verify-pending.`,
        );
      }
    }
    // no-test-repos D2 (decision 55b951e1): a verify-none cell in a
    // repo that has declared itself no-test skips the passing-verify-result
    // requirement below outright — there is no verify command that could
    // ever have run. The auto note stands in as recorded evidence wherever
    // else this function (and the lane-tier proof check further down) would
    // otherwise expect one; an explicitly supplied verification_evidence
    // still wins over the auto note.
    const noTestWaiver = cell.verify === NO_TEST_SENTINEL && isNoTestRepo(readConfig(root));
    if (noTestWaiver && !verification_evidence && !pendingFeatureVerify) {
      verification_evidence = `no-test repo: verification waived by repo declaration (commands.verify: ${NO_TEST_SENTINEL})`;
    }
    if (!pendingFeatureVerify && trace.verify_passed !== true && !noTestWaiver) {
      throw new Error(
        `capCell: cell "${id}" has no passing verify result — run the cell's verify command and record it (bee.mjs cells verify --id ${id} --command CMD --passed true) before capping.`,
      );
    }
    // D-GHF-C (GH #27.5): a NEEDS_REVISION semantic-judge verdict (judge.mjs
    // JUDGE_VERDICTS — the enum has no 'FAIL', 'NEEDS_REVISION' is the fail
    // value) blocks cap unless an audited --override-judge reason is
    // supplied. Only the LATEST trace.semantic_judge entry is consulted — a
    // cell with no semantic_judge entries at all is untouched by this guard
    // (byte-identical to pre-ghf-6 behavior).
    const judgeEntries = Array.isArray(trace.semantic_judge) ? trace.semantic_judge : [];
    const latestJudge = judgeEntries.length ? judgeEntries[judgeEntries.length - 1] : null;
    if (latestJudge && latestJudge.verdict === 'NEEDS_REVISION' && !overrideReason) {
      const err = new Error(
        `capCell: cell "${id}" has a NEEDS_REVISION semantic-judge verdict — rework the cell and record a PASS verdict (bee.mjs cells judge-record), or cap with an audited override (bee.mjs cells cap --id ${id} --override-judge "<reason>").`,
      );
      err.code = 'JUDGE_REWORK_REQUIRED';
      throw err;
    }
    // Override always audited: appended to append-only trace.judge_overrides
    // and logged as a decision BEFORE the cell write (inside the lock,
    // mirroring resetCellBudget's D-GHF-C audit-before-write ordering) so the
    // decision record survives even if the write itself fails. Recorded
    // whenever a reason is supplied, not only when the guard above actually
    // fired — an override is audit-worthy on its own, regardless of whether
    // it was strictly needed.
    if (overrideReason) {
      const overrides = Array.isArray(trace.judge_overrides) ? trace.judge_overrides : [];
      const overrideEntry = {
        overridden_at: utcNow(),
        reason: overrideReason,
        last_verdict: latestJudge ? latestJudge.verdict : null,
      };
      logDecision(root, {
        decision: `«cells cap: cell "${id}" judge override by ${trace.worker || 'unknown'} — ${overrideReason}»`,
        rationale:
          'Audited cap over a NEEDS_REVISION (or absent) semantic-judge verdict (D-GHF-C, GH #27.5) — the verdict itself is never rewritten, only a judge_overrides marker appended.',
        scope: 'repo',
        source: 'user',
        tags: ['cells', 'judge'], // jrt-1: an internal caller with no tags throws typed DECISIONS_UNTAGGED_REFUSED once docs/decisions/taxonomy.json exists — census swept, see test_cells.mjs
      });
      trace = { ...trace, judge_overrides: [...overrides, overrideEntry] };
    }
    if (bc && !verification_evidence && !pendingFeatureVerify) {
      throw new Error(
        `capCell: cell "${id}" declares behavior_change but provides no verification_evidence — attach evidence (--evidence-file) or drop the behavior_change flag.`,
      );
    }
    // test-economy D1/D2: resolve the effective class (same derivation
    // deriveChangeClass always used — null/bc=true -> 'behavior', nothing new
    // introduced here) and its required proof tier ONCE, reused by both the
    // Decision 0009 "before" check and the D3 self-correcting-loop floor
    // below, plus the refactor/formatting new-test-file refusal further down.
    const effectiveClass = deriveChangeClass({ ...cell, behavior_change: bc });
    const proofTier = requiredProofTier(effectiveClass, cell.lane);
    // test-economy D1: a refactor/formatting cell must never introduce a new
    // test file — CONTEXT: "cấm tuyệt đối — new_suite_reason của D3 KHÔNG
    // override được; refactor cần suite mới nghĩa là phân loại sai." Gated on
    // diff_stats (undefined — no git, or a legacy caller — skips this check
    // entirely, fail-open); the ratio/new_suite_reason checks for the OTHER
    // classes belong to a later cell (test-economy D3) and are deliberately
    // not added here.
    if (
      (effectiveClass === 'refactor' || effectiveClass === 'formatting') &&
      diff_stats &&
      Array.isArray(diff_stats.new_test_files) &&
      diff_stats.new_test_files.length > 0
    ) {
      throw new Error(
        `capCell: cell "${id}" is classified "${effectiveClass}" but its diff adds new test file(s) (${diff_stats.new_test_files.join(', ')}) — a refactor/formatting change must not need a new test suite (test-economy D1: no override, not even via new_suite_reason). Reclassify the cell (e.g. "behavior") or drop the new test file(s) from this diff.`,
      );
    }
    // test-economy D3: any OTHER change_class (refactor/formatting are
    // refused unconditionally just above — D1 wins over D3, new_suite_reason
    // cannot rescue those) that introduces a new test file must justify it:
    // verification_evidence must be a JSON object carrying `new_suite_reason`
    // (>=20 chars) — a new test_*.mjs/tests/ path becomes a permanent
    // CI-registered suite forever (run_verify.mjs's discoverSuites), so the
    // decision to add one needs a stated reason, not silence. Gated on
    // diff_stats the same way D1 is (undefined — no git, or a legacy caller —
    // skips this check entirely, fail-open).
    // main-verifies D1: the pending path skips this check — its justification
    // channel IS verification_evidence, which the pending path refuses by
    // construction; the feature-level verify (and the slice-tail test cell's
    // own review) owns that scrutiny instead.
    if (
      !pendingFeatureVerify &&
      effectiveClass !== 'refactor' &&
      effectiveClass !== 'formatting' &&
      diff_stats &&
      Array.isArray(diff_stats.new_test_files) &&
      diff_stats.new_test_files.length > 0
    ) {
      const newSuiteEvidence = parseVerificationEvidence(verification_evidence);
      const newSuiteReason =
        typeof newSuiteEvidence.new_suite_reason === 'string' ? newSuiteEvidence.new_suite_reason.trim() : '';
      if (newSuiteReason.length < NEW_SUITE_REASON_MIN_CHARS) {
        throw new Error(
          `capCell: cell "${id}" adds new test file(s) (${diff_stats.new_test_files.join(', ')}) — evidence phải là JSON có field new_suite_reason (≥${NEW_SUITE_REASON_MIN_CHARS} ký tự) explaining why a new permanent CI suite is warranted (test-economy D3). Attach --evidence-file/--evidence-stdin with a JSON object like {"new_suite_reason": "..."}.`,
        );
      }
    }
    // test-economy D3: test-lines-added / source-lines-changed ratio ceiling
    // (both counts already mirror-deduped upstream by bee.mjs's
    // computeDiffStats, per plan.md's dedupe rule). tiny/small only ADD a
    // non-blocking warning — collected into `ratioWarning` here, folded into
    // `trace.warnings` near the bottom of this function. standard/high-risk
    // REFUSE past the ceiling unless verification_evidence carries a JSON
    // `ratio_waiver` (>=20 chars) — using the waiver is itself audited as a
    // decision (D8: a loosened guard's use stays visible, never silent).
    let ratioWarning = null;
    if (
      diff_stats &&
      Number.isFinite(diff_stats.test_lines_added) &&
      Number.isFinite(diff_stats.source_lines_changed)
    ) {
      const ratio = diff_stats.test_lines_added / Math.max(diff_stats.source_lines_changed, 1);
      if ((cell.lane === 'tiny' || cell.lane === 'small') && ratio > RATIO_WARN_CEILING) {
        ratioWarning = `capCell: cell "${id}" test-to-source line ratio is ${ratio.toFixed(2)} (>${RATIO_WARN_CEILING}) for lane "${cell.lane}" — test-economy D3 non-blocking warning.`;
      } else if (pendingFeatureVerify && (cell.lane === 'standard' || cell.lane === 'high-risk') && ratio > RATIO_REFUSE_CEILING) {
        // main-verifies D1: the pending path cannot carry a ratio_waiver (its
        // waiver channel is verification_evidence, refused by construction),
        // so past-ceiling stays VISIBLE as a recorded warning instead of an
        // unsatisfiable refusal — the feature-level verify owns the proof.
        ratioWarning = `capCell: cell "${id}" test-to-source line ratio is ${ratio.toFixed(2)} (>${RATIO_REFUSE_CEILING}) for lane "${cell.lane}" — recorded as a warning on the feature-verify-pending path (main-verifies D1; test-economy D3's waiver channel is per-cell evidence, which this path defers).`;
      } else if ((cell.lane === 'standard' || cell.lane === 'high-risk') && ratio > RATIO_REFUSE_CEILING) {
        const ratioEvidence = parseVerificationEvidence(verification_evidence);
        const ratioWaiver =
          typeof ratioEvidence.ratio_waiver === 'string' ? ratioEvidence.ratio_waiver.trim() : '';
        if (ratioWaiver.length < RATIO_WAIVER_MIN_CHARS) {
          throw new Error(
            `capCell: cell "${id}" test-to-source line ratio is ${ratio.toFixed(2)} (>${RATIO_REFUSE_CEILING}) for lane "${cell.lane}" — evidence phải là JSON có field ratio_waiver (≥${RATIO_WAIVER_MIN_CHARS} ký tự) justifying the ratio, or shrink the diff (test-economy D3).`,
          );
        }
        // D8: the waiver's use is audited the same way a judge override is
        // (line ~1826 above) — the ceiling itself is never rewritten, only a
        // decision record marks that this one cap was exempted, and why.
        logDecision(root, {
          decision: `«cells cap: cell "${id}" test-to-source ratio ${ratio.toFixed(2)} waived by ${trace.worker || 'unknown'} (lane ${cell.lane}) — ${ratioWaiver}»`,
          rationale:
            'Audited ratio_waiver over the test-economy D3 lane ceiling (standard/high-risk >4) — the ceiling itself is unchanged, only this cap is exempted with a recorded reason (D8: a loosened guard stays visible).',
          scope: 'repo',
          source: 'user',
          tags: ['cells', 'test-economy'],
        });
      }
    }
    // derived-check-hardening E1: cap-time cross-check of the cell's static
    // `verify` command against the impact registry's DIRECT-edge (level:1 —
    // a transitive sweep would name so many suites the warning becomes noise
    // nobody reads) suites for `cell.files`. LOUD, never a refusal: a
    // refusal on this door would gate every future cap on registry
    // freshness, so any returned suite missing from `cell.verify` is only
    // named on stderr. Reuses queryRegistry verbatim (see the note above) —
    // no new derivation logic here. dch-8: the module itself is imported
    // lazily, inside this same try, so a vendored-lib-only fixture with no
    // sibling scripts/ directory hits the catch below exactly like a
    // missing, unreadable, or malformed registry FILE — this runs on EVERY
    // cap, so it must never throw either way.
    let impactWarning = null;
    try {
      const cellFiles = Array.isArray(cell.files) ? cell.files : [];
      if (cellFiles.length > 0) {
        const registryPath = path.join(root, 'scripts', 'impact-registry.json');
        const registry = JSON.parse(fs.readFileSync(registryPath, 'utf8'));
        const { queryRegistry } = await import('../../../scripts/impact_registry.mjs');
        const { mappedSuites } = queryRegistry(registry, cellFiles, { level: 1 });
        const verifyText = typeof cell.verify === 'string' ? cell.verify : '';
        const missingSuites = mappedSuites.filter((suite) => !verifyText.includes(suite));
        if (missingSuites.length > 0) {
          impactWarning = `capCell: cell "${id}" verify does not mention impact-registry direct-edge suite(s) ${missingSuites.join(', ')} for file(s) ${cellFiles.join(', ')} — derived-check-hardening E1 non-blocking warning.`;
          process.stderr.write(`${impactWarning}\n`);
        }
      }
    } catch {
      // Missing, unreadable, or malformed registry FILE, or the registry
      // MODULE itself being absent (vendored-lib-only fixture) — either way
      // a silent skip (E1: never a throw).
      impactWarning = null;
    }
    // Decision 0009, NARROWED by test-economy D2: a behavior_change cell must
    // record the "before" it changed — a characterization of prior behavior —
    // not just an assertion that the new behavior works. This blocks
    // assertion-capping at the source. test-economy D2 supersedes the
    // original "every bc=true cell" scope: this "before" requirement now
    // fires ONLY when the resolved proof tier is 'red-first' (security/
    // migration in any lane, or a behavior-bearing class in a high-risk
    // lane) — a targeted-green tier (e.g. bugfix/behavior/api outside
    // high-risk) accepts ordinary verification_evidence with no
    // red_failure_evidence at all.
    if (proofTier === 'red-first' && verification_evidence) {
      let evidence = verification_evidence;
      if (typeof evidence === 'string') {
        try {
          evidence = JSON.parse(evidence);
        } catch {
          evidence = null; // freeform evidence — the non-empty check above already applies
        }
      }
      if (evidence && typeof evidence === 'object' && !Array.isArray(evidence)) {
        const before = evidence.red_failure_evidence;
        const hasBefore = typeof before === 'string' && before.trim().length > 0;
        const exceptions = evidence.deliberate_exceptions;
        const hasException = Array.isArray(exceptions)
          ? exceptions.some((e) => typeof e === 'string' && e.trim().length > 0)
          : typeof exceptions === 'string' && exceptions.trim().length > 0;
        if (!hasBefore && !hasException) {
          throw new Error(
            `capCell: behavior_change cell "${id}" needs a "before" characterization — set red_failure_evidence in the evidence (the prior behavior this change alters: a git-show of the old state, or a pre-change check that failed). If there is genuinely no prior behavior (a brand-new surface), say so in deliberate_exceptions. An assertion that the new behavior works is not evidence that behavior changed.`,
          );
        }
      }
    }
    // D3 (self-correcting-loop) — behavior-class cap teeth, ADDITIVE to the
    // Decision 0009 "before" check above. test-economy D1/D2: NARROWED from
    // "effectiveClass === 'behavior'" to "proofTier === 'red-first'" — the
    // same scope as the Decision 0009 check right above (security/migration
    // in any lane, or a behavior-bearing class in a high-risk lane). F5: a
    // cap riding the deliberate_exceptions door keeps today's contract
    // untouched — no length/duplicate floor; the STDERR advisory noting that
    // lives in bee.mjs's handler layer (F4 precedent), recomputed from the
    // returned cell post-cap.
    if (proofTier === 'red-first' && !pendingFeatureVerify) {
      const evidence = parseVerificationEvidence(verification_evidence);
      if (!evidenceRidesExceptionDoor(evidence)) {
        const before = typeof evidence.red_failure_evidence === 'string' ? evidence.red_failure_evidence.trim() : '';
        if (!before) {
          throw new Error(
            `capCell: behavior-class cell "${id}" (D3 judge-standard matrix) is missing verification_evidence.red_failure_evidence — the matrix minimum for a "behavior" change. FIX: attach red_failure_evidence (>=${RED_EVIDENCE_MIN_CHARS} chars characterizing the prior behavior) or record deliberate_exceptions.`,
          );
        }
        if (before.length < RED_EVIDENCE_MIN_CHARS) {
          throw new Error(
            `capCell: behavior-class cell "${id}" red_failure_evidence is only ${before.length} char(s) — the D3 judge-standard matrix requires >=${RED_EVIDENCE_MIN_CHARS} chars (anti-boilerplate floor). FIX: expand the evidence to genuinely characterize the prior behavior, or record deliberate_exceptions.`,
          );
        }
        const collision = findDuplicateRedEvidence(root, id, before);
        if (collision) {
          throw new Error(
            `capCell: behavior-class cell "${id}" red_failure_evidence is byte-identical to cell "${collision}"'s recorded evidence — the D3 judge-standard matrix refuses reused boilerplate. FIX: write evidence specific to this cell's own prior behavior.`,
          );
        }
      }
    }
    // Decision 0004: small+ lanes cap only on recorded proof, never on an assertion.
    if (cell.lane === 'small' || cell.lane === 'standard' || cell.lane === 'high-risk') {
      const output = trace.verify_output;
      const hasOutput = typeof output === 'string' ? output.trim().length > 0 : output != null;
      const hasEvidence =
        verification_evidence != null &&
        (typeof verification_evidence !== 'string' || verification_evidence.trim().length > 0);
      if (!hasOutput && !hasEvidence && !pendingFeatureVerify) {
        throw new Error(
          `capCell: lane "${cell.lane}" cell "${id}" has a passing verify flag but no recorded proof — re-record the verify with its output (bee.mjs cells verify --id ${id} --command CMD --output "..." --passed true) or attach verification_evidence (--evidence-file). An assertion is not evidence.`,
        );
      }
      if (!Array.isArray(files_changed) || files_changed.length === 0) {
        throw new Error(
          `capCell: lane "${cell.lane}" cell "${id}" requires non-empty files_changed (--files a.js,b.js) — record what the worker actually touched. A cell that changed nothing is a drop or a NOOP, not a cap.`,
        );
      }
    }
    if (cell.lane === 'high-risk') {
      if (typeof outcome !== 'string' || !outcome.trim()) {
        throw new Error(`capCell: high-risk cell "${id}" requires an outcome summary.`);
      }
    }
    // worker-conformance D10/D12/D14: the absence-of-proof marker. Computed
    // HERE — after the ENTIRE refusal chain above has run, on a cap that is
    // already going to succeed — so it can never decide whether a cap is
    // refused, only describe one that isn't. D10's hole it closes: recordVerify
    // (above) validates only `command` and `passed`, so `--passed true` with no
    // output at all is legal; without a marker such a cap would leave the
    // feature closeable with zero tests executed anywhere.
    // D12: this is a NEW, inert field — never a reuse of the pending flag.
    // `pendingFeatureVerify` is not neutral: it short-circuits six refusal
    // sites (:1886-1899, :1912, :1999, :2032, :2135, :2164), so routing an
    // unproven cap onto it would have voided D2's red-first tier and D6's
    // brakes. `trace.proof` is read by nothing in this function.
    // D14: "unrecorded" means NEITHER channel carried proof — a cell holding
    // genuine verification_evidence (e.g. a tiny-lane security cell whose
    // red_failure_evidence already passed the red-first door at :2135) is
    // never marked, even when verify_output is empty.
    // Exemptions: the explicit --feature-verify-pending path (it already
    // carries its own marker), and a repo that declared `commands.verify:
    // "none"` (decision 55b951e1) — a feature-level verify can never run
    // there, so marking would arm a close-door that repo could never satisfy.
    // Keyed on commands.verify alone, deliberately narrower than
    // isNoTestRepo() at :1908: a repo with only `commands.test: "none"` can
    // still run a real feature verify and must keep arming the door.
    const proofOutput = trace.verify_output;
    const hasProofOutput =
      typeof proofOutput === 'string' ? proofOutput.trim().length > 0 : proofOutput != null;
    const hasProofEvidence =
      verification_evidence != null &&
      (typeof verification_evidence !== 'string' || verification_evidence.trim().length > 0);
    const proofUnrecorded =
      !hasProofOutput &&
      !hasProofEvidence &&
      !pendingFeatureVerify &&
      !isNoTestCommand((readConfig(root).commands || {}).verify);
    cell.status = 'capped';
    cell.trace = {
      ...trace,
      files_changed: Array.isArray(files_changed) ? files_changed : [],
      deviations: Array.isArray(deviations) ? deviations : [],
      friction: friction ?? null,
      behavior_change: bc,
      verification_evidence: verification_evidence ?? null,
      outcome: typeof outcome === 'string' && outcome.trim() ? outcome : trace.outcome,
      capped_at: utcNow(),
      // test-economy D3: non-blocking ratio-ceiling warning (tiny/small
      // only, computed above) — additive field, an empty array whenever
      // diff_stats is absent or the ratio is within bounds, so existing
      // consumers of trace see no schema break. derived-check-hardening E1
      // folds the impact-registry warning (also computed above, also
      // non-blocking) into the SAME array — same channel, same semantics.
      warnings: [ratioWarning, impactWarning].filter(Boolean),
      // main-verifies D1: the pending marker guardFeatureVerifyDebt (bee.mjs)
      // reads at both swarming exits. Stamped ONLY on the pending path — the
      // classic evidence path's trace stays byte-identical to before.
      ...(pendingFeatureVerify ? { feature_verify: 'pending' } : {}),
      // worker-conformance D10/D12/D14 (computed just above): additive and
      // absent whenever the cap recorded real proof, so every existing trace
      // consumer sees a byte-identical record on the proved path.
      ...(proofUnrecorded ? { proof: 'unrecorded' } : {}),
    };
    return writeCell(root, cell);
  });
  releaseClaimFileBestEffort(root, id); // D1 Δ2: cap is a claim-clearing transition
  return saved;
}

export async function blockCell(root, id, reason, { sessionId, forceOwnership = false } = {}) {
  if (typeof reason !== 'string' || !reason.trim()) {
    throw new Error('blockCell: a reason is required.');
  }
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'blockCell', id); // hardening-1: refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`blockCell: cell "${id}" not found.`);
    let trace = { ...defaultTrace(), ...(cell.trace || {}) };
    trace = guardClaimOwnership(root, id, trace, 'blockCell', { sessionId, forceOwnership }); // D4
    // D1: block has no verify output to normalize — the reason text itself is
    // the closest analog, so it feeds both the ledger's note and its signature.
    trace = appendAttempt(root, id, trace, {
      verdict: 'blocked',
      failureSignature: normalizeFailureSignature(reason),
      note: reason,
    });
    cell.status = 'blocked';
    cell.trace = { ...trace, blocked_reason: reason };
    return writeCell(root, cell);
  });
  releaseClaimFileBestEffort(root, id); // D1 Δ2: block is a claim-clearing transition
  return saved;
}

// hardening-4b: read-check-write now runs inside withStoreLock(`cells:${id}`)
// — the same CAS protection every other mutator below gets.
export async function dropCell(root, id, reason) {
  if (typeof reason !== 'string' || !reason.trim()) {
    throw new Error('dropCell: a reason is required.');
  }
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'dropCell', id); // hardening-1: refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`dropCell: cell "${id}" not found.`);
    cell.status = 'dropped';
    cell.trace = { ...defaultTrace(), ...(cell.trace || {}), dropped_reason: reason };
    return writeCell(root, cell);
  });
  releaseClaimFileBestEffort(root, id); // D1 Δ2: drop is a claim-clearing transition
  return saved;
}

// Clear the claim and any recorded verify from a trace, so a cell returned to
// "open" must be re-claimed and re-verified before it can cap again — a stale
// verify_passed must never let a later re-cap skip its proof (capCell gates on
// trace.verify_passed === true). Keeps the rest of the trace for audit.
function releaseTrace(existing) {
  const trace = { ...defaultTrace(), ...(existing || {}) };
  trace.worker = null;
  trace.claimed_at = null;
  trace.claim_session = null; // hardening-4b: no owner once the claim is released
  trace.verify_command = null;
  trace.verify_output = null;
  trace.verify_passed = null;
  trace.verified_at = null;
  return trace;
}

// unclaimCell — the inverse of claim: a mis-claimed or abandoned "claimed" cell
// goes back to "open" so another worker can pick it up. Refuses on any other
// status (GitHub #12). Mirrors claimCell's own-status assertion shape.
// hardening-4b: read-check-write now runs inside withStoreLock(`cells:${id}`).
export async function unclaimCell(root, id, { sessionId, forceOwnership = false } = {}) {
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    const cell = readCell(root, id);
    if (!cell) throw new Error(`unclaimCell: cell "${id}" not found.`);
    if (cell.status !== 'claimed') {
      throw new Error(
        `unclaimCell: cell "${id}" is "${cell.status}", not "claimed" — only a claimed cell can be unclaimed (returned to open). For a capped/blocked/dropped cell use bee.mjs cells reopen.`,
      );
    }
    let trace = { ...defaultTrace(), ...(cell.trace || {}) };
    trace = guardClaimOwnership(root, id, trace, 'unclaimCell', { sessionId, forceOwnership }); // D4
    cell.status = 'open';
    cell.trace = releaseTrace(trace);
    return writeCell(root, cell);
  });
  releaseClaimFileBestEffort(root, id); // D1 Δ2: unclaim is a claim-clearing transition (forced unclaim also clears — D4 Δ5)
  return saved;
}

// reopenCell — bring a terminal cell (capped / blocked / dropped) back to "open"
// for rework, recording why. Refuses on "open" (already there) and on "claimed"
// (that is unclaim's job). Clears the recorded verify so the reopened cell must
// prove itself again before capping (GitHub #12).
// hardening-4b: read-check-write now runs inside withStoreLock(`cells:${id}`).
export async function reopenCell(root, id, reason, { sessionId, forceOwnership = false } = {}) {
  if (typeof reason !== 'string' || !reason.trim()) {
    throw new Error('reopenCell: a reason is required.');
  }
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'reopenCell', id); // hardening-1-7-10 (D4): refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`reopenCell: cell "${id}" not found.`);
    if (cell.status === 'open') {
      throw new Error(`reopenCell: cell "${id}" is already "open".`);
    }
    if (cell.status === 'claimed') {
      throw new Error(
        `reopenCell: cell "${id}" is "claimed" — use bee.mjs cells unclaim to release the claim back to open.`,
      );
    }
    let guardedTrace = { ...defaultTrace(), ...(cell.trace || {}) };
    guardedTrace = guardClaimOwnership(root, id, guardedTrace, 'reopenCell', { sessionId, forceOwnership }); // D4
    cell.status = 'open';
    const trace = releaseTrace(guardedTrace);
    trace.blocked_reason = null;
    trace.dropped_reason = null;
    trace.reopened_at = utcNow();
    trace.reopened_reason = reason;
    cell.trace = trace;
    return writeCell(root, cell);
  });
  releaseClaimFileBestEffort(root, id); // D1 Δ2: reopen is a claim-clearing transition
  return saved;
}

// Decision 0016 — the orchestrator assesses a cell's difficulty at dispatch and
// records the tier it chose (extraction/generation/ceiling), rather than a fixed
// planning-time label. Keeps tierMix/scarcity accurate against real dispatch
// decisions. Idempotent; validates the tier.
// hardening-4b: read-check-write now runs inside withStoreLock(`cells:${id}`).
export async function setTier(root, id, tier) {
  if (!MODEL_TIERS.includes(tier)) {
    throw new Error(`setTier: tier must be one of ${MODEL_TIERS.join(', ')}, got "${tier}".`);
  }
  return withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'setTier', id); // hardening-1-7-10 (D4): refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`setTier: cell "${id}" not found.`);
    cell.tier = tier;
    return writeCell(root, cell);
  });
}

// Decision 0011 — capture-mode spine. The behavior_change cells capped for the
// active feature since the last scribing run: the mechanical proxy for "settled
// behavior not yet in docs/specs/". Threshold prefers last_scribing_run.at
// (precise ISO, written by newer scribing runs) and falls back to .date (day
// granularity) for older runs. A last run for a DIFFERENT feature (or none)
// means the whole active feature is debt. Returns { count, cells: [ids] }; empty
// while idle (no feature in flight).
//
// Still a pure read — but it is no longer only a signal (chain-integrity D2).
// It is advisory everywhere it is DISPLAYED (status line, session preamble,
// Stop-hook nudge — all fail-open) and a WALL at exactly one place: entering
// phase `compounding-complete`, enforced at the bee.mjs choke point, which
// refuses the close while debt stands. That reversal is the whole point: debt
// being "never a blocker" is precisely why a feature could be marked closed
// with six capped behavior_change cells whose behavior never reached
// docs/specs/. Debt is a signal through the work, and a wall at the door.
//
// scribing-integrity si-1: `opts.feature`/`opts.sinceTs` are OPTIONAL
// overrides so every debt-wall door (feature close, feature swap, lane
// close) can reuse this one function instead of forking its logic. Calling
// scribingDebt(root) with no opts stays byte-for-byte identical to before
// this cell — it still derives both the feature and the threshold from the
// default state record alone. Passing opts.feature scopes the read to that
// feature; passing opts.sinceTs replaces the derived threshold outright
// (used by the lane-close wall, which threshold's off the LANE record's own
// last_scribing_run, never the default record's).
export function scribingDebt(root, opts = {}) {
  const state = readState(root);
  const feature = opts.feature !== undefined && opts.feature !== null ? opts.feature : state.feature;
  if (!feature) return { count: 0, cells: [] };
  let threshold = 0;
  if (opts.sinceTs !== undefined && opts.sinceTs !== null) {
    threshold = Number.isFinite(opts.sinceTs) ? opts.sinceTs : 0;
  } else {
    // scribing-stamp-seam sss-1: a workflow-backed default record's
    // last_scribing_run mutation can be silently dropped before it ever
    // reaches disk. handleStateScribingRun (bee.mjs) mutates the in-memory
    // record then calls write(state) -> writeStateRecordThroughProjection,
    // which — whenever a LIVE workflow record names this feature — never
    // persists the caller's mutated object at all: it pushes only the D1
    // fields (phase/mode/summary/next_action/gates) onto the workflow record
    // via updateWorkflowAssumingLock, then rebuildStateProjection derives the
    // new state.json by spreading `...current` (state-projection.mjs
    // ~238-250), where `current` is a FRESH readState(root) off the STILL-
    // STALE disk file — the just-stamped last_scribing_run never lived
    // there, so it is not in `current` either, and vanishes for good. The
    // durable ledger (appendScribingLedger) is written unconditionally on
    // every scribing-run call regardless of whether the record write landed
    // — bestScribingStampMs's ledger-max-then-lane-then-record logic (si-1/
    // si-3, reused verbatim, not forked here) survives this seam even when
    // the state-field stamp does not, so it is the threshold's fallback/max,
    // scoped to THIS feature the same way bestScribingStampMs already scopes
    // every source it consults.
    const best = bestScribingStampMs(root, feature, { state });
    if (best !== null) threshold = best;
  }
  const cells = listCells(root, { feature, status: 'capped' })
    .filter((cell) => {
      const trace = cell.trace || {};
      if (trace.behavior_change !== true) return false;
      const cappedAt = Date.parse(trace.capped_at);
      return Number.isFinite(cappedAt) && cappedAt > threshold;
    })
    .map((cell) => cell.id);
  return { count: cells.length, cells };
}

// scribing-integrity si-1: the durable ledger. `state scribing-run` used to
// be the ONLY record of a sync ever happening, and it lived exclusively on
// whichever record (default or lane) the call happened to target — so a
// feature that never got its own explicit close (a dead session, never came
// back) left no trace ANYWHERE that scribing had or hadn't run for it. This
// is that trace: one durable, append-only line per scribing-run call,
// independent of which record was mutated (or whether one was mutated at
// all — see the non-active-feature repair path in bee.mjs).
export function scribingLedgerPath(root) {
  return path.join(root, '.bee', 'logs', 'scribing-runs.jsonl');
}

// sqs-b3: exported so `state scribing-run --show` (bee.mjs) can read the
// SAME ledger rows globalScribingDebt reads below, instead of a second
// hand-rolled readJsonl(scribingLedgerPath(root)) composition.
export function readScribingLedger(root) {
  return readJsonl(scribingLedgerPath(root));
}

// LOUD, never a crash: a scribing-run call's whole job is recording that a
// sync happened, and the state-field stamp on the record being mutated stays
// authoritative for that record regardless of this call's outcome — but a
// silent ledger-write failure would defeat globalScribingDebt's orphan sweep
// without anyone ever finding out. fs.appendFileSync/mkdir failures (full
// disk, permissions, ...) are caught here and reported to stderr; the caller
// (bee.mjs's scribing-run verb) never sees or handles this failure.
export function appendScribingLedger(root, { feature, ts, areas }) {
  try {
    appendJsonl(scribingLedgerPath(root), { ts, feature, areas });
  } catch (error) {
    process.stderr.write(
      `[bee] WARNING: failed to append the scribing ledger (${scribingLedgerPath(root)}): ${
        error && error.message ? error.message : error
      }. The state-field stamp still recorded for the active record; the durable ledger did not.\n`,
    );
  }
}

function scribingRunStampMs(run) {
  if (!run) return null;
  const parsed = Date.parse(run.at || run.date);
  return Number.isFinite(parsed) ? parsed : null;
}

// scribing-integrity si-1 (D1-D3): the orphan sweep. scribingDebt() only ever
// looks at ONE record's own idea of "the active feature" — the default
// record's `state.feature`, or (via opts) a single named lane. A feature that
// never comes back to its own close (a dead session) is invisible to every
// existing debt surface: nothing ever asks "is THIS feature's debt paid",
// because nothing routes through it again. This walks every LIVE capped
// behavior_change cell instead — across every feature at once — and asks the
// same question the wall asks, using the best sync evidence available for
// that cell's OWN feature: the durable ledger (any repair stamp, ever,
// anywhere), that feature's lane record (if it has one), or the default
// record's own last_scribing_run (only when it names that same feature).
// Archived cells are out of scope (decision: sweep covers live cells only —
// rewriting historical debt is not this cell's job). Returns
// { count, features: [{ feature, cells }] }, features sorted by name.
// sqs-b3: extracted out of globalScribingDebt so `state scribing-run --show
// --feature <slug>` (bee.mjs) can reuse the EXACT SAME "best known stamp for
// this feature" logic the orphan sweep below uses — ledger max, then the
// feature's own lane stamp, then the default record's stamp (only when it
// names this same feature) — never a second hand-rolled traversal. `ctx`
// lets a caller that already has the ledger/state in hand (globalScribingDebt
// iterating many features) pass them in instead of re-reading per call.
export function bestScribingStampMs(root, feature, ctx = {}) {
  const ledger = ctx.ledger || readScribingLedger(root);
  const state = ctx.state || readState(root);
  let best = null;
  for (const entry of ledger) {
    if (!entry || entry.feature !== feature) continue;
    const parsed = Date.parse(entry.ts);
    if (Number.isFinite(parsed) && (best === null || parsed > best)) best = parsed;
  }
  const lane = readLane(root, feature);
  const laneStamp = lane ? scribingRunStampMs(lane.last_scribing_run) : null;
  if (laneStamp !== null && (best === null || laneStamp > best)) best = laneStamp;
  // si-3: honor the default record's last_scribing_run by ITS OWN feature
  // attribution, never by whether that feature happens to be the currently
  // active one. `state.feature === feature` was wrong in both directions:
  // (a) it hid a properly-closed feature's own stamp the moment a new
  // feature became active (live repro: cli-ergonomics stamped 04:02:40,
  // after its own caps, yet counted orphaned because scribing-integrity had
  // since become active), and (b) it would have let an unrelated OLDER
  // feature's stamp wrongly clear the active feature's cells, since the
  // stamp's own `feature` field was never checked at all.
  if (state.last_scribing_run && state.last_scribing_run.feature === feature) {
    const defaultStamp = scribingRunStampMs(state.last_scribing_run);
    if (defaultStamp !== null && (best === null || defaultStamp > best)) best = defaultStamp;
  }
  return best;
}

export function globalScribingDebt(root) {
  const cells = listCells(root, { status: 'capped' }).filter(
    (cell) => (cell.trace || {}).behavior_change === true,
  );
  if (cells.length === 0) return { count: 0, features: [] };

  const state = readState(root);
  const ledger = readScribingLedger(root);
  const stampCache = new Map();

  const bestStampMs = (feature) => {
    if (stampCache.has(feature)) return stampCache.get(feature);
    const best = bestScribingStampMs(root, feature, { ledger, state });
    stampCache.set(feature, best);
    return best;
  };

  const byFeature = new Map();
  for (const cell of cells) {
    const feature = cell.feature;
    if (!feature) continue;
    const cappedAt = Date.parse((cell.trace || {}).capped_at);
    const stamp = bestStampMs(feature);
    const orphaned = stamp === null || (Number.isFinite(cappedAt) && cappedAt > stamp);
    if (!orphaned) continue;
    if (!byFeature.has(feature)) byFeature.set(feature, []);
    byFeature.get(feature).push(cell.id);
  }

  const features = Array.from(byFeature.entries())
    .map(([feature, ids]) => ({ feature, cells: ids }))
    .sort((a, b) => a.feature.localeCompare(b.feature, 'en'));
  const count = features.reduce((sum, entry) => sum + entry.cells.length, 0);
  return { count, features };
}

// P12 / decision 0018 — the frozen judge. A worker that rewrites the test
// suite, CI config, lockfiles, or the verify configuration has not passed the
// judge — it has replaced the judge. Files matching these patterns that were
// changed WITHOUT being declared in the cell's `files` scope are tamper
// signals: the orchestrator never counts such a cell toward a clean wave and
// flags it for review (source: delegator's frozen-judge globs, LOOP survey).
export const FROZEN_JUDGE_PATTERNS = [
  { rule: 'test sources', pattern: /(^|\/)(tests?|__tests__|specs?)\//i },
  { rule: 'test file', pattern: /\.(test|spec)\.[a-z]+$/i },
  { rule: 'snapshot', pattern: /(^|\/)__snapshots__\/|\.snap$/i },
  {
    rule: 'CI config',
    pattern: /(^|\/)\.github\/workflows\/|(^|\/)\.gitlab-ci\.yml$|(^|\/)Jenkinsfile$|(^|\/)azure-pipelines\.yml$|(^|\/)\.circleci\//i,
  },
  {
    rule: 'lockfile',
    pattern:
      /(^|\/)(package-lock\.json|yarn\.lock|pnpm-lock\.yaml|bun\.lockb?|Cargo\.lock|poetry\.lock|uv\.lock|go\.sum|composer\.lock|Gemfile\.lock)$/i,
  },
  {
    rule: 'package manifest',
    pattern: /(^|\/)(package\.json|pyproject\.toml|Cargo\.toml|go\.mod|composer\.json|Gemfile)$/i,
  },
  {
    rule: 'test config',
    pattern: /(^|\/)(jest\.config|vitest\.config|playwright\.config|karma\.conf|pytest\.ini|tox\.ini|phpunit\.xml)[^/]*$/i,
  },
  { rule: 'bee verify config', pattern: /(^|\/)\.bee\/config\.json$/i },
];

function normalizePath(p) {
  return String(p).replace(/\\/g, '/').replace(/^\.\//, '').trim();
}

// A declared entry covers a changed file when it matches exactly, is a
// directory prefix (entry ends with '/'), or is a simple '*' glob.
function declaredCovers(declared, file) {
  for (const raw of declared) {
    const entry = normalizePath(raw);
    if (!entry) continue;
    if (entry === file) return true;
    if (entry.endsWith('/') && file.startsWith(entry)) return true;
    if (entry.includes('*')) {
      // '**' crosses directories, '*' stays within one segment. Escape regex
      // metacharacters first, then translate the stars via a placeholder that
      // cannot appear in an escaped path (escaping leaves no bare '+').
      const DOUBLE_STAR = '+';
      const source = entry
        .replace(/[.+?^${}()|[\]\\]/g, '\\$&')
        .replace(/\*\*/g, DOUBLE_STAR)
        .replace(/\*/g, '[^/]*')
        .split(DOUBLE_STAR)
        .join('.*');
      if (new RegExp(`^${source}$`).test(file)) return true;
    }
  }
  return false;
}

/**
 * Frozen-judge check: judge-pattern files changed outside the declared scope.
 * @param {string[]} changedFiles - the worker's trace.files_changed
 * @param {string[]} declaredFiles - the cell's declared `files` scope
 * @returns {{file:string, rule:string}[]} hits — empty means the judge is intact.
 */
export function frozenJudgeHits(changedFiles, declaredFiles = []) {
  const declared = Array.isArray(declaredFiles) ? declaredFiles : [];
  const hits = [];
  for (const raw of Array.isArray(changedFiles) ? changedFiles : []) {
    const file = normalizePath(raw);
    if (!file) continue;
    const match = FROZEN_JUDGE_PATTERNS.find(({ pattern }) => pattern.test(file));
    if (!match) continue;
    if (declaredCovers(declared, file)) continue;
    hits.push({ file, rule: match.rule });
  }
  return hits;
}

/** Convenience: run the frozen-judge check on a capped/claimed cell's trace. */
export function judgeCell(root, id) {
  const cell = readCell(root, id);
  if (!cell) throw new Error(`judgeCell: cell "${id}" not found.`);
  const changed = (cell.trace && cell.trace.files_changed) || [];
  const declared = Array.isArray(cell.files) ? cell.files : [];
  return { id: cell.id, hits: frozenJudgeHits(changed, declared) };
}

// Decision 0012 / P7 — keep the ceiling (strongest) model scarce, measurably.
// Above this share of tiered cells on the ceiling tier, the scarcity is at risk
// (the cost lever of "the strong model touches few dispatches" is eroding).
export const CEILING_MAX_SHARE = 0.4;
const SCARCITY_MIN_TIERED = 3; // below this, any share is noise — stay silent.

/** Tier assignment across a feature's cells (all statuses). */
export function tierMix(root, { feature = null } = {}) {
  const cells = listCells(root, feature ? { feature } : {});
  const counts = { extraction: 0, generation: 0, ceiling: 0, untiered: 0 };
  for (const cell of cells) {
    if (MODEL_TIERS.includes(cell.tier)) counts[cell.tier] += 1;
    else counts.untiered += 1;
  }
  const tiered = counts.extraction + counts.generation + counts.ceiling;
  const ceilingShare = tiered > 0 ? counts.ceiling / tiered : 0;
  return { counts, tiered, ceilingShare };
}

/**
 * P7 scarcity signal: returns { pct, ceiling, tiered } when the active feature
 * leans too much on the ceiling model, else null (nothing to warn about).
 * Scoped to the active feature when set. Advisory — never a blocker.
 */
export function ceilingScarcityWarning(root) {
  const state = readState(root);
  const mix = tierMix(root, { feature: state.feature || null });
  if (mix.tiered < SCARCITY_MIN_TIERED) return null;
  if (mix.ceilingShare <= CEILING_MAX_SHARE) return null;
  return { pct: Math.round(mix.ceilingShare * 100), ceiling: mix.counts.ceiling, tiered: mix.tiered };
}

// ─── D2 (self-correcting-loop): cell-lifetime budgets at the claim door ────
// Closes gap #1 of the Builder-Judge-Manager assessment: budgets reset per
// claim by design (rescue rung 1 grants a fresh consult budget), so a
// claim -> block -> re-dispatch cycle could loop indefinitely at cell
// lifetime with no ceiling. `budgets` is an optional cell field; absent
// falls back to DEFAULT_BUDGETS (D6: additive, existing single-claim single-
// attempt cells behave byte-identically until a cell actually loops).
const DEFAULT_BUDGETS = { max_claims: 3, max_failed_attempts: 4, max_same_signature: 2 };

// GH #27.3 (D-GHF-C): a hard ceiling on every resolved budget, 3x
// DEFAULT_BUDGETS — closes the authoring-time gap the #27 claim verdicts
// flagged (workers can never raise budgets live, but an authored cell could
// declare an absurd budgets.max_claims and never hit the claim door at all).
// resolveCellBudgets clamps a too-high DECLARED value down to this ceiling
// (defensive at resolve time, for legacy/pre-guard cells); validateNewCell
// below refuses an over-ceiling value outright at authoring time instead.
const BUDGET_HARD_MAX = { max_claims: 9, max_failed_attempts: 12, max_same_signature: 6 };

export function resolveCellBudgets(cell) {
  const declared =
    cell && typeof cell.budgets === 'object' && cell.budgets && !Array.isArray(cell.budgets)
      ? cell.budgets
      : {};
  // A non-integer or below-floor (<1) declared value is untrustworthy input,
  // not merely "too big" — it falls back to DEFAULT_BUDGETS for that key
  // rather than being clamped. A valid integer >=1 that exceeds the hard max
  // is clamped down to it instead (D-GHF-C: no path ever raises a budget
  // above BUDGET_HARD_MAX).
  const pick = (key) => {
    const value = declared[key];
    if (!Number.isInteger(value) || value < 1) return DEFAULT_BUDGETS[key];
    return Math.min(value, BUDGET_HARD_MAX[key]);
  };
  return {
    max_claims: pick('max_claims'),
    max_failed_attempts: pick('max_failed_attempts'),
    max_same_signature: pick('max_same_signature'),
  };
}

// D2+Δ1: counters restart after the latest trace.budget_resets marker — an
// append-only audit array written ONLY by resetCellBudget below. Lexical
// (string) comparison is safe here because every `at`/`reset_at` value is
// the same utcNow() ISO-8601 shape, which sorts lexicographically exactly
// like it sorts chronologically.
function attemptsSinceBudgetReset(cell) {
  const trace = cell.trace || {};
  const attempts = Array.isArray(trace.attempts) ? trace.attempts : [];
  const resets = Array.isArray(trace.budget_resets) ? trace.budget_resets : [];
  const marker = resets.length > 0 ? resets[resets.length - 1].reset_at : null;
  if (typeof marker !== 'string' || !marker) return attempts;
  return attempts.filter((a) => a && typeof a.at === 'string' && a.at > marker);
}

function budgetExhaustedRefusal(id, name, limit, used, relevant) {
  const failed = relevant.filter((a) => a.verdict === 'fail' || a.verdict === 'blocked').length;
  const passed = relevant.filter((a) => a.verdict === 'pass').length;
  return {
    ok: false,
    code: 'CELL_BUDGET_EXHAUSTED',
    reason: `cell "${id}" exhausted its "${name}" budget (limit ${limit}, used ${used}) — the claim door is closed until an audited reset.`,
    budget: { name, limit },
    used,
    history_summary: `${relevant.length} attempt(s) recorded since the last reset (${failed} failed/blocked, ${passed} passed).`,
    fix: `bee.mjs cells reset-budget --id ${id} --reason "<why a retry is warranted>" — audited (logs a decision), never rewrites the attempt ledger.`,
  };
}

// D2+Δ2: the STRUCTURAL loop-safety check — deliberately never imports or
// reads gate_bypass/config.json (bypassLevel/readConfig never appear in this
// function). CELL_BUDGET_EXHAUSTED and REPEATED_FAILURE are not approval
// gates a human can wave through; the only door back open is
// resetCellBudget's audited reset below. Returns {ok:true} when the cell may
// proceed, else the typed refusal — surfaced as-is by claimCellCrossSession
// (direct `cells claim --id`) or used as a selection filter by
// claimNextCell (Δ3/F3: a bricked candidate is skipped, never surfaced).
export function checkCellBudgets(cell) {
  const budgets = resolveCellBudgets(cell);
  const relevant = attemptsSinceBudgetReset(cell);

  // D2+Δ1 (GH #27.1, D-GHF-B): claims_used = distinct (claim_session,
  // acquired_at ?? claimed_at) pairs, +1 for the acquisition currently being
  // attempted (it has no ledger entry yet — nothing has been
  // verified/blocked under it). The key is the heartbeat-invariant
  // acquisition identity: acquired_at is stamped once at claim creation and
  // never rewritten by renewClaimTTL, so N heartbeats between failures under
  // one claim epoch still collapse to a single pair (D-GHF-B) — the pre-fix
  // key of claimed_at alone changed on every heartbeat and inflated the
  // count. Legacy ledger entries with no acquired_at fall back to
  // claimed_at, counting exactly as before (D6). A legacy cell with no
  // ledger reads as 0 pairs, so its first claim is exactly 1 — byte-
  // identical to today (D6).
  const pairs = new Set();
  for (const a of relevant) {
    pairs.add(`${a.claim_session ?? ''} ${(a.acquired_at ?? a.claimed_at) ?? ''}`);
  }
  const claimsUsed = pairs.size + 1;
  if (claimsUsed > budgets.max_claims) {
    return budgetExhaustedRefusal(cell.id, 'max_claims', budgets.max_claims, claimsUsed, relevant);
  }

  const failedAttempts = relevant.filter((a) => a.verdict === 'fail' || a.verdict === 'blocked').length;
  if (failedAttempts >= budgets.max_failed_attempts) {
    return budgetExhaustedRefusal(cell.id, 'max_failed_attempts', budgets.max_failed_attempts, failedAttempts, relevant);
  }

  // Same-signature: two (or budgets.max_same_signature) fail/blocked entries
  // sharing an identical failure_signature mean the Manager is re-running
  // the exact same fix rather than changing approach — a distinct refusal
  // from generic exhaustion, so it is checked independently of max_claims/
  // max_failed_attempts above.
  const signatureCounts = new Map();
  for (const a of relevant) {
    if (a.verdict !== 'fail' && a.verdict !== 'blocked') continue;
    if (typeof a.failure_signature !== 'string' || !a.failure_signature) continue;
    signatureCounts.set(a.failure_signature, (signatureCounts.get(a.failure_signature) || 0) + 1);
  }
  for (const [signature, count] of signatureCounts) {
    if (count >= budgets.max_same_signature) {
      return {
        ok: false,
        code: 'REPEATED_FAILURE',
        reason: `cell "${cell.id}" failed ${count} time(s) with the identical signature "${signature}" — change approach or escalate, this is not a re-run.`,
        signature,
        fix: `bee.mjs cells reset-budget --id ${cell.id} --reason "<why a retry is warranted>" — audited (logs a decision), never rewrites the attempt ledger.`,
      };
    }
  }

  return { ok: true };
}

// D2 + GH #27.4 (D-GHF-C): the ONLY door that reopens a budget-exhausted or
// repeated-failure cell. Requires a reason (audited), logs a decision, and
// appends to the append-only trace.budget_resets — it NEVER touches
// trace.attempts, so the full attempt history the marker is scoped against
// stays intact for post-hoc review (mirrors trace.ownership_overrides'
// append-only shape).
//
// GUARD ORDERING (plan-check constraint, keeps existing error contracts
// byte-identical): reason-check first (outside the lock — cheapest, needs no
// disk read), then not-found (inside the lock, first thing after readCell),
// THEN the two new guards — actor-required, then budget-blocked. An unknown
// cell id refuses "not found" regardless of whether an actor was supplied;
// only once the cell is known to exist do the new guards get a say.
//
// D-GHF-C closes claim-verdict #4 (#27): reset previously had no exhaustion
// guard at all (any cell, blocked or not, could be "reset") and the actor
// was optional. Now: refuses (typed RESET_NOT_NEEDED) unless
// checkCellBudgets(cell) currently reports not-ok, and refuses without an
// actor (--operator flag or BEE_AGENT_NAME env fallback). The audit
// logDecision runs BEFORE writeCell so a crash/failure in the write itself
// still leaves the decision recorded — the audit trail never silently loses
// a reset just because the store write failed.
export async function resetCellBudget(root, id, reason, { sessionId, operator } = {}) {
  if (typeof reason !== 'string' || !reason.trim()) {
    throw new Error('resetCellBudget: a reason is required.');
  }
  const reasonText = reason.trim();
  const bySession = resolveSessionId({ flag: sessionId }) || null;
  const actor =
    typeof operator === 'string' && operator.trim()
      ? operator.trim()
      : typeof process.env.BEE_AGENT_NAME === 'string' && process.env.BEE_AGENT_NAME.trim()
        ? process.env.BEE_AGENT_NAME.trim()
        : null;
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'resetCellBudget', id); // hardening-1-7-10 (D4): refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`resetCellBudget: cell "${id}" not found.`);
    if (!actor) {
      throw new Error(
        `resetCellBudget: an actor is required — pass --operator "<name>" or set BEE_AGENT_NAME in the environment before resetting cell "${id}"'s budget.`,
      );
    }
    const budgetCheck = checkCellBudgets(cell);
    if (budgetCheck.ok) {
      const err = new Error(
        `resetCellBudget: cell "${id}" is not budget-blocked (checkCellBudgets reports ok) — a reset is only needed once the claim door is actually closed by CELL_BUDGET_EXHAUSTED or REPEATED_FAILURE.`,
      );
      err.code = 'RESET_NOT_NEEDED';
      throw err;
    }
    const trace = { ...defaultTrace(), ...(cell.trace || {}) };
    const resets = Array.isArray(trace.budget_resets) ? trace.budget_resets : [];
    const resetEntry = { reset_at: utcNow(), reason: reasonText, by_session: bySession, by_actor: actor };
    // Audit BEFORE write (D-GHF-C): logDecision is synchronous and completes
    // here first, so the decision record survives even if writeCell below
    // throws (e.g. an unwritable store).
    logDecision(root, {
      decision: `«cells reset-budget: cell "${id}" claim-lifetime budget reset by ${actor} — ${reasonText}»`,
      rationale:
        'Audited reopening of a D2 loop-safety door (self-correcting-loop); the attempt ledger itself is never rewritten, only a budget_resets marker appended.',
      scope: 'repo',
      source: 'user',
      tags: ['cells'], // jrt-1: an internal caller with no tags throws typed DECISIONS_UNTAGGED_REFUSED once docs/decisions/taxonomy.json exists — census swept, see test_cells.mjs
    });
    cell.trace = { ...trace, budget_resets: [...resets, resetEntry] };
    return writeCell(root, cell);
  });
  return saved;
}

// ─── D5 (self-correcting-loop): trace.semantic_judge — one structured judge
// verdict schema, append-only, stamped with honest model independence ──────
// validateJudgeVerdict/deriveModelIndependence live in judge.mjs (pure, zero
// I/O — CONTEXT D5 prohibition "no dispatching logic in lib, validation
// only"); recordJudgeVerdict is the sole mutator that turns a validated
// verdict into a trace entry, mirroring appendAttempt's append-only
// discipline (D1) and resetCellBudget's own dedicated-trace-key split (D2).
// semantic_judge is its OWN append-only array, never folded into
// trace.attempts or trace.deviations (capCell REPLACES deviations wholesale
// on every cap — the same trap appendOwnershipOverride's own comment names
// above); the `...trace` spread every mutator already does is what keeps
// semantic_judge alive across a LATER verify/cap/block on the same cell.
//
// Δ6: builder_model/judge_model are CALLER-supplied (the orchestrator's own
// pinned dispatch params at record time) — this function never reads
// .bee/logs/dispatch.jsonl itself; that fail-open audit log is corroboration
// only and must never feed a fail-closed guard (a missing/absent log here
// simply means the caller passed no model, which already degrades
// model_independence to 'unverified' via deriveModelIndependence — never a
// refusal).
//
// hardening-3: a NEEDS_REVISION verdict recorded AFTER the cell already
// capped (the realistic D4 goal-check ordering) used to be toothless — it
// appended to trace.semantic_judge but never touched cell.status, so
// capCell's own NEEDS_REVISION guard (line ~1145) never re-fires because the
// cell is already past cap. hardening-3 first fixed this by reopening
// "capped" -> "claimed", but that left a stale-evidence re-cap hole
// (hardening-1-7-10 D7): trace.verify_passed/verify_command/verify_output
// from the ORIGINAL cap survived the reopen untouched, so a later PASS
// verdict recorded with NO fresh verify run at all could re-cap the cell —
// capCell's `trace.verify_passed !== true` gate never re-fired because the
// stale `true` from before was still sitting there. D7: recording a
// NEEDS_REVISION verdict against a cell whose CURRENT status is "capped" now
// reopens it to "open" (NOT "claimed" — reusing the existing CELL_STATUSES
// 'open' value, never a new status) with releaseTrace clearing the claim +
// verify evidence, exactly like reopenCell/unclaimCell do for every other
// claim-clearing transition. A fresh claim (claimCell requires "open") and a
// fresh verify are then structurally required before the existing capCell
// gates (verify_passed, latest judge verdict) will pass again — the
// stale-evidence hole closes. trace.semantic_judge (append-only, just
// extended above) and trace.reopened_for_rework survive releaseTrace, which
// only clears the claim/verify keys, never semantic history. A PASS
// verdict, or a NEEDS_REVISION verdict on a non-capped cell, leaves
// cell.status untouched, byte-identical to pre-hardening-3 behavior. This is
// a logical read-check-write (readCell -> possible status flip ->
// writeCell), so it runs under the same withStoreLock discipline
// capCell/reopenCell use, and is therefore async (mirrors msh-5's
// startFeature: refusals reject a Promise instead of throwing synchronously
// — callers await it).
export async function recordJudgeVerdict(
  root,
  id,
  verdictInput,
  { builderModel = null, builderStatus = null, judgeModel = null, judgeStatus = null, sessionId, forceOwnership = false } = {},
) {
  const { ok, errors } = validateJudgeVerdict(verdictInput);
  if (!ok) {
    throw new Error(
      `recordJudgeVerdict: cell "${id}" verdict rejected against schema "judge-verdict/1" — ${errors.join(' ')} FIX: the judge dispatch must return the schema verbatim (never free prose); re-dispatch once, then record model_independence "unverified" if it fails again (D5).`,
    );
  }
  const independence = deriveModelIndependence(builderModel, builderStatus, judgeModel, judgeStatus);
  let reopenedForRework = false;
  const saved = await withStoreLock(root, `cells:${id}`, () => {
    assertNotArchived(root, 'recordJudgeVerdict', id); // hardening-1-7-10 (D4): refuse before ever reading/writing
    const cell = readCell(root, id);
    if (!cell) throw new Error(`recordJudgeVerdict: cell "${id}" not found.`);
    const entry = {
      schema: verdictInput.schema,
      verdict: verdictInput.verdict,
      checks: verdictInput.checks,
      failure_signature: verdictInput.failure_signature ?? null,
      fixability: verdictInput.fixability,
      confidence: verdictInput.confidence,
      builder_model: typeof builderModel === 'string' && builderModel.trim() ? builderModel : null,
      judge_model: typeof judgeModel === 'string' && judgeModel.trim() ? judgeModel : null,
      model_independence: independence,
      recorded_at: utcNow(),
    };
    let trace = { ...defaultTrace(), ...(cell.trace || {}) };
    trace = guardClaimOwnership(root, id, trace, 'recordJudgeVerdict', { sessionId, forceOwnership });
    const existing = Array.isArray(trace.semantic_judge) ? trace.semantic_judge : [];
    trace = { ...trace, semantic_judge: [...existing, entry] };
    if (verdictInput.verdict === 'NEEDS_REVISION' && cell.status === 'capped') {
      cell.status = 'open'; // D7: reopen to a clean slate, not "claimed" — no owner survives a NEEDS_REVISION
      trace.reopened_for_rework = {
        at: utcNow(),
        reason: 'NEEDS_REVISION semantic-judge verdict recorded after cap',
      };
      // D7: clear the claim + verify evidence (releaseTrace) AFTER stamping
      // reopened_for_rework and appending semantic_judge above — releaseTrace
      // only clears worker/claimed_at/claim_session/verify_* keys, so both of
      // those survive intact. This is what closes the stale-evidence re-cap
      // hole: capCell's verify_passed gate can never again pass on the
      // evidence from before this reopen.
      trace = releaseTrace(trace);
      reopenedForRework = true;
      logDecision(root, {
        decision: `«cells judge-record: cell "${id}" reopened capped->open by a NEEDS_REVISION semantic-judge verdict»`,
        rationale:
          'A NEEDS_REVISION verdict recorded after cap must have teeth: the cell is reopened to open (clean slate) for rework, with claim + verify evidence cleared, instead of being silently logged into an inert trace entry (hardening-3) or left falsely "claimed" with stale verify_passed that a later PASS verdict could re-cap on with zero fresh verify (hardening-1-7-10 D7).',
        scope: 'repo',
        source: 'user',
        tags: ['cells', 'judge'], // jrt-1: an internal caller with no tags throws typed DECISIONS_UNTAGGED_REFUSED once docs/decisions/taxonomy.json exists — census swept, see test_cells.mjs
      });
    }
    cell.trace = trace;
    return writeCell(root, cell);
  });
  if (reopenedForRework) {
    // D7: reconcile the claims-store defensively. capCell already deleted the
    // claim file on the ORIGINAL cap (releaseClaimFileBestEffort, "cap is a
    // claim-clearing transition") — so ordinarily there is nothing left here.
    // This call exists for the rare case a claims-store entry survived that
    // cleanup (a missed release, a manually-recreated claim, or any other
    // drift between the cells store and the claims store): a reopened-to-open
    // cell must never leave an orphaned cross-session lock behind it, exactly
    // the same "never orphan the claim file" discipline every other
    // claim-clearing verb (cap/unclaim/block/drop/reopen) already applies.
    releaseClaimFileBestEffort(root, id);
  }
  return saved;
}

// ─── claim-next: cross-session selection + throw-safe two-store claim ──────
// (fresh-session-handoff fsh-11, D2/D4).

// claimCellCrossSession — the CLAIMING primitive alone, exported separately
// from claimNextCell below so the throw-unwind guarantee is directly
// testable without re-deriving a whole selection scenario.
//
// UNWIND PIN (validation-s4 panel W4): claims.mjs's claimCellFile is a typed
// return ({ok:false,...} on contention, never throws for that), but this
// module's OWN claimCell signals every failure by THROW (gate refused, cell
// not found, wrong status, uncapped deps — there is no {ok:false} shape to
// check). Skipping the try/catch here would let a real claimCell failure
// leave the just-created claims-store file behind FOREVER — an orphaned
// cross-session lock with no cells.mjs-side owner, since nothing else ever
// looks at it again. Every throw here releases the claim via claims.mjs's own
// releaseClaim before surfacing a typed failure; this function itself never
// throws for a claimCell failure, only for bad arguments (mirrors claims.mjs
// requireId's bad-argument convention).
//
// D1/D3 (msh-2): sessionId is nullable — null/undefined is a legal
// SESSIONLESS claim (this is now the CLI's `cells claim --id` path too, via
// bee.mjs's handleCellsClaim, for single-user use with no
// CLAUDE_CODE_SESSION_ID and no explicit --session-id). claimNextCell (the
// only other caller) still enforces a non-empty sessionId at its OWN
// boundary before ever reaching here, so the cross-session selection flow is
// byte-unchanged for a real session id.
export async function claimCellCrossSession(root, { sessionId, worker, cellId, ttl } = {}) {
  if (sessionId !== null && sessionId !== undefined && (typeof sessionId !== 'string' || !sessionId.trim())) {
    throw new Error('claimCellCrossSession: sessionId must be a non-empty string, or null/absent for a sessionless claim.');
  }
  if (typeof worker !== 'string' || !worker.trim()) {
    throw new Error('claimCellCrossSession: worker is required.');
  }
  if (typeof cellId !== 'string' || !cellId.trim()) {
    throw new Error('claimCellCrossSession: cellId is required.');
  }
  const session = sessionId == null ? null : sessionId.trim();
  const id = cellId.trim();
  // msn-18b (PLANE RULE): the claim itself is control-plane — resolved
  // through controlRootFor so a claim taken from a linked worktree lands in
  // main's shared claims store (the topology this cell's must_have tests),
  // never a worktree-local one. `id` is the join key back to the cell FILE,
  // which stays on the caller's own workspace-local `root` below (readCell/
  // claimCell) — same-plane claim-to-cell resolution: any workspace can
  // resolve the shared claim by cell id, then read ITS OWN local cell file
  // for that same id.
  const controlRoot = controlRootFor(root);

  const fileClaim = claimCellFile(controlRoot, session, id, ttl);
  if (!fileClaim.ok) return fileClaim; // typed CLAIMED failure, propagated as-is

  // D2+Δ2: budget check runs INSIDE the O_EXCL critical section — only after
  // claimCellFile's atomic acquisition succeeded, so the count below can
  // never be raced by a concurrent claimant (TOCTOU-safe by construction:
  // claimCellFile's O_EXCL is the only door anyone can win). A missing cell
  // is left to claimCell's own not-found error a few lines down, unchanged.
  // A refusal here unwinds the just-acquired claim file via releaseClaim —
  // same precedent as the CLAIM_CELL_FAILED unwind below — so a refused
  // acquisition never orphans a claims-store file. Enforcement therefore
  // lands at the NEXT claim attempt; this claim's own bounded overrun (the
  // O_EXCL file briefly existed for one refusal-length window) is the
  // documented tradeoff over a pre-acquire check, which would be TOCTOU-racy
  // against a concurrent claim-fail-release.
  const cellForBudget = readCell(root, id);
  if (cellForBudget) {
    const budgetCheck = checkCellBudgets(cellForBudget);
    if (!budgetCheck.ok) {
      releaseClaim(controlRoot, session, id);
      return budgetCheck;
    }
  }

  try {
    // hardening-4b: claimCell is now withStoreLock-wrapped (async); pass the
    // resolved session through so its trace.claim_session matches the claim
    // file this same call just created — the pairing sweepExpiredClaims'
    // reset guard checks.
    const cell = await claimCell(root, id, worker, { sessionId: session });
    return { ok: true, cell, claim: fileClaim.claim };
  } catch (err) {
    releaseClaim(controlRoot, session, id); // never orphan the claim file we just created
    return {
      ok: false,
      code: 'CLAIM_CELL_FAILED',
      reason: err instanceof Error ? err.message : String(err),
    };
  }
}

// claimNextCell — the SELECTION half. Given the acting session, picks the
// next open cell to claim and runs it through claimCellCrossSession above.
// Selection order:
//   (1) the acting session's OWN pipeline (its bound lane, or the default
//       state.json pipeline when unbound/no binding — resolvePipeline's own
//       session -> lane -> default resolution seam) — ONLY when that
//       pipeline's execution gate is approved. An own pipeline whose gate is
//       unapproved contributes nothing here, on purpose: D2's authority
//       boundary ("only a human's, or a recorded bypass's, gate decision
//       authorizes a lane's cells, never the puller") holds for the acting
//       session's own lane too — never select from an unapproved lane even
//       when its cells are the only ready ones.
//   (2) empty (no ready cells, or the gate is unapproved) -> every OTHER
//       pipeline (the default state.json pipeline plus every
//       .bee/lanes/*.json record) whose OWN execution gate is approved,
//       pooled and ordered by backlog rank (docs/backlog.md's Feature
//       column, via backlog.mjs's featureBacklogRank) then lane created_at,
//       oldest first; a pipeline with no backlog row, or no created_at,
//       sorts after one that has it. GH#20: a LANE (never the default
//       state.json pipeline, which has no binding concept) actively owned by
//       another live session — some OTHER session record has lane === that
//       feature and a fresh heartbeat (!heartbeatStale, claims.mjs's own
//       staleness rule) — is skipped entirely, never pooled, so claim-next
//       cannot steal a cell out from under a session mid-lane on it; a lane
//       whose only owner's heartbeat has gone stale is fair game again
//       (steal-after-death is preserved, only live ownership blocks).
//   (3) any candidate whose declared files intersect ANOTHER session's
//       active reservation hold (findSessionConflicts, D3) is skipped
//       outright — the acting session's own holds never exclude a cell.
//   (3b, xwh-3) any candidate whose declared files intersect an unexpired
//       cross-worktree ledger hold owned by a DIFFERENT checkout (resolved
//       via resolveHoldTopology below + findForeignHolds, worktree-holds.mjs)
//       is skipped exactly the same way — silently, next candidate — never
//       a crash or a refusal. Read-only consultation: claim-next never
//       mirrors or releases anything in the ledger. A repo with no ledger
//       file, or a checkout resolveRoots cannot place (topology throws, or
//       an ungranted linked worktree that already shares the main store
//       directly), degrades to today's exact behavior — no foreign-hold
//       check runs at all.
//   (4) nothing claimable anywhere -> typed { ok:false, code:'NO_APPROVED_WORK' }.
//
// SWEEP PIN (validation-s4 panel B1): sweepExpiredClaims runs FIRST, every
// call, unconditionally — this is sweepExpiredClaims's production trigger (it
// had zero production callers before this, tests only). A dead session's
// stale claim (TTL expired AND heartbeat stale, re-verified under its own
// gate by sweepExpiredClaims itself) is reclaimed in THIS SAME pass, before
// selection reads anything else, so a just-swept cell is immediately
// claimable and the typed NO_APPROVED_WORK stop is never returned while one
// still exists.

// xwh-3: resolves the cross-worktree HOLD topology for claim-next's
// foreign-hold consultation — same shape/naming as bee.mjs's own
// resolveHoldTopology (xwh-2), rebased on the `root` claim-next already
// carries instead of `process.cwd()` (claim-next is a library call, not a
// CLI entrypoint pinned to the process's own cwd; `root` IS the checkout
// under selection). Returns `{ mainRoot, holder }` for the two topologies
// worth consulting:
//   - an ORDINARY checkout: holder = 'main', mainRoot = the checkout itself.
//   - a GRANTED linked worktree (its own storeRoot === its own worktreeRoot,
//     i.e. resolveRoots did NOT fall back to main): holder = its
//     git-verified id, mainRoot = resolveRoots' own `mainRoot`.
// Returns `null` for every other case — an UNGRANTED linked worktree
// (storeRoot === mainRoot already: `root` here already IS the shared main
// store, so a foreign-hold check would just be redundant with the
// same-checkout reservation check above it) and an unresolvable/invalid
// checkout (resolveRoots threw) both fall through to `null`, which the
// caller below treats as "skip the foreign-hold consultation entirely,
// exactly like before this cell" — never a crash, never a refusal on its
// own.
function resolveHoldTopology(root) {
  let resolution;
  try {
    resolution = resolveRoots(root);
  } catch {
    return null;
  }
  if (resolution.worktreeResolution === 'ordinary') {
    return { mainRoot: resolution.workRoot || root, holder: 'main' };
  }
  if (resolution.worktreeResolution === 'linked-valid' && resolution.mainRoot && resolution.id) {
    const granted =
      resolution.storeRoot && resolution.worktreeRoot && path.resolve(resolution.storeRoot) === path.resolve(resolution.worktreeRoot);
    if (granted) {
      return { mainRoot: resolution.mainRoot, holder: resolution.id };
    }
  }
  return null;
}

// hardening-4b: async — sweepExpiredClaims may now itself acquire
// withStoreLock (the sweep-reset), and this composes claimCellCrossSession
// (also now async) at the bottom.
export async function claimNextCell(root, { sessionId, worker, ttl } = {}) {
  if (typeof sessionId !== 'string' || !sessionId.trim()) {
    throw new Error('claimNextCell: sessionId is required.');
  }
  if (typeof worker !== 'string' || !worker.trim()) {
    throw new Error('claimNextCell: worker is required.');
  }
  const session = sessionId.trim();
  // msn-18b (PLANE RULE): claims/sessions are control-plane — resolved
  // through controlRootFor once here and reused for every claims.mjs/
  // reservations.mjs touch below. Cell-file reads (readyCells/readState/
  // listLanes) stay on the caller's own workspace-local `root`.
  const controlRoot = controlRootFor(root);

  // Unconditional, first thing — the production sweep trigger (C10). A swept
  // cell's stale claims-store file is gone by the time selection below reads
  // anything, so it is claimable in this exact pass.
  await sweepExpiredClaims(controlRoot);

  const resolved = resolvePipeline(root, { sessionId: session });
  if (!resolved.ok) {
    // A bound-but-broken lane (missing/corrupt) refuses loudly rather than
    // silently falling back to the default pipeline — same discipline
    // resolvePipeline itself documents; claim-next never masks it.
    return { ok: false, code: resolved.code, reason: resolved.reason };
  }
  const ownFeature = resolved.record.feature || null;

  const holdFree = (cell) => {
    const files = Array.isArray(cell.files) ? cell.files : [];
    if (files.length === 0) return true;
    if (findSessionConflicts(controlRoot, session, files).length > 0) return false;
    // xwh-3: read-only foreign-hold consultation, same silent-skip posture
    // as the same-checkout reservation check just above — a missing ledger
    // (findForeignHolds' own fail-open read) or an unresolvable/ungranted
    // topology (resolveHoldTopology -> null) both fall through as "no
    // foreign holds", byte-identical to today.
    const topology = resolveHoldTopology(root);
    if (!topology) return true;
    return findForeignHolds(topology.mainRoot, topology.holder, files).length === 0;
  };
  // D2 Δ3/F3: a budget-exhausted or repeated-failure candidate is skipped at
  // SELECTION (reads the candidate's own ledger cheaply, no extra I/O) so a
  // single bricked cell never bricks the whole pool — only a direct
  // `cells claim --id` surfaces the typed refusal.
  const candidateOk = (cell) => holdFree(cell) && checkCellBudgets(cell).ok;

  let candidate = null;
  if (ownFeature && gateApproved(resolved.record, 'execution')) {
    candidate = readyCells(root, ownFeature).find(candidateOk) || null;
  }

  if (!candidate) {
    const state = readState(root);
    const pipelines = new Map(); // feature -> { approved, created_at }
    if (state.feature && state.feature !== ownFeature) {
      // The default state.json pipeline stays poolable here on purpose: it
      // has no binding concept, so the live-owner guard below (which is
      // strictly a lane-record check) never applies to it.
      pipelines.set(state.feature, { approved: gateApproved(state, 'execution'), created_at: null });
    }
    // GH#20: build the set of lanes actively owned by another live session
    // ONCE, from every session record, so the lane loop below is a plain
    // membership check. "Owned" = some OTHER session (id !== the acting
    // session — rule: the acting session's own binding never blocks
    // anything) is bound to that lane (record.lane) with a fresh heartbeat
    // (!heartbeatStale, claims.mjs's own staleness rule/threshold — the same
    // rule claim-sweep uses). An unreadable/corrupt session record already
    // reads as absent via listSessionRecords' fail-open posture, matching
    // heartbeatStale's own "missing/unparseable = stale" posture: it can
    // never mark a lane as live-owned.
    const liveOwnedLanes = new Set();
    for (const record of listSessionRecords(controlRoot)) {
      if (!record || record.id === session) continue;
      const boundLane = typeof record.lane === 'string' ? record.lane.trim() : '';
      if (!boundLane || heartbeatStale(record)) continue;
      liveOwnedLanes.add(boundLane);
    }
    // msn-18b: deliberately NOT re-rooted. Lane records are themselves still
    // written by bee.mjs's own lane-mutation handlers (writeLane/
    // updateWorkflowAssumingLock, `state gate`/`state set --lane`) on the
    // bare, workspace-local root — that write side is msn-18c's scope, not
    // this cell's. Re-rooting only this READ ahead of its own writers would
    // desync the pool from the granted-worktree lanes actually being
    // mutated, trading a mostly-harmless staleness gap for a
    // silently-wrong one. `liveOwnedLanes` above IS control-rooted (session
    // binding is unambiguously control-plane and has no such writer-lag
    // risk); GH#20's anti-steal check therefore degrades gracefully — it may
    // simply not see every lane a granted worktree has, never crash or
    // mis-claim.
    for (const lane of listLanes(root)) {
      if (!lane.feature || lane.feature === ownFeature || pipelines.has(lane.feature)) continue;
      if (liveOwnedLanes.has(lane.feature)) continue; // GH#20: a lane actively owned by another live session is never pooled
      pipelines.set(lane.feature, {
        approved: gateApproved(lane, 'execution'),
        created_at: lane.created_at || null,
      });
    }

    const rank = featureBacklogRank(root);
    const pool = [];
    for (const [feature, meta] of pipelines) {
      if (!meta.approved) continue; // D2: an unapproved lane is never touched
      for (const cell of readyCells(root, feature)) {
        if (candidateOk(cell)) pool.push({ cell, feature, meta });
      }
    }
    pool.sort((a, b) => {
      const rankA = rank.has(a.feature) ? rank.get(a.feature) : Infinity;
      const rankB = rank.has(b.feature) ? rank.get(b.feature) : Infinity;
      if (rankA !== rankB) return rankA - rankB;
      const createdA = a.meta.created_at ? Date.parse(a.meta.created_at) : NaN;
      const createdB = b.meta.created_at ? Date.parse(b.meta.created_at) : NaN;
      const aKnown = Number.isFinite(createdA);
      const bKnown = Number.isFinite(createdB);
      if (aKnown && bKnown && createdA !== createdB) return createdA - createdB;
      if (aKnown !== bKnown) return aKnown ? -1 : 1; // a known created_at outranks an unknown one
      return 0;
    });
    candidate = pool.length > 0 ? pool[0].cell : null;
  }

  if (!candidate) {
    return {
      ok: false,
      code: 'NO_APPROVED_WORK',
      reason:
        "no claimable cell: the acting session's own pipeline has none ready, and no other execution-approved pipeline has a ready cell free of another session's hold.",
    };
  }

  return claimCellCrossSession(root, { sessionId: session, worker, cellId: candidate.id, ttl });
}
