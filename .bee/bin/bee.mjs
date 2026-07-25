#!/usr/bin/env node
// bee.mjs — unified CLI dispatcher covering all 9 command groups (status,
// cells, reservations, decisions, state, backlog, capture, reviews,
// feedback; dispatcher-unify DB1/DB2).
//
// This dispatcher imports the SAME lib/*.mjs functions the 9 bee_*.mjs
// entrypoints (bee_status.mjs, bee_cells.mjs, bee_reservations.mjs,
// bee_decisions.mjs, bee_state.mjs, bee_backlog.mjs, bee_capture.mjs,
// bee_reviews.mjs, bee_feedback.mjs) used to import directly — those 9
// files are now thin shims that prepend their group name and call this
// file's exported `main()` (DB2), so handlers run in-process (no spawnSync,
// no subprocess) and `bee <group> <verb>` output is byte-identical to
// invoking a shim directly (verified by tests/test_bee_cli.mjs).
//
// Usage:
//   bee status [--json]
//   bee cells <list|ready|show|add|claim|verify|cap|block|drop|tier|judge|claim-next|reset-budget|judge-record|schedule|archive|unarchive> ... [--json]
//   bee reservations <reserve|release|list|sweep> ... [--json]
//   bee decisions <log|supersede|redact|active|search|archive|tag> ... [--json]
//   bee state <set|gate|plan-rev bump|worker add/update/remove/clear/prune|scribing-run|start-feature|lanes|session list/bind/unbind> ... [--json]
//   bee backlog <add|counts|rank|badges> ... [--json]
//   bee capture <add|list|flush|count> ... [--json]
//   bee intent <set|show|advance|clear> ... [--json]
//   bee reviews <create|list|show|record|candidate add|candidates|status> ... [--json]
//   bee feedback <digest|count|collect|rank> ... [--json]
//   bee knowledge <check|index|list|context> ... [--json]
//   bee tmp <sweep> ... [--json]
//   bee --help [--json]
//
// D3: `bee --help --json` emits {schema_version, commands:[{name, invoke,
// description, parameters, examples, deprecated}]} — the same JSON-Schema
// tool-definition shape Claude Code's own tool/subagent surface uses.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

import {
  findRepoRoot,
  readConfig,
  bypassLevel,
  bypassBanner,
  readState,
  readStateStrict,
  writeState,
  readHandoff,
  readOnboarding,
  BEE_VERSION,
  COMMAND_KEYS,
  GATE_NAMES,
  PHASES,
  KNOWN_PHASES,
  MODEL_TIERS,
  NO_TEST_SENTINEL,
  isKnownPhase,
  checkPhaseTransition,
  checkScribingRunPhase,
  startFeature,
  hasStaleAdvisorKey,
  STALE_ADVISOR_KEY_WARNING,
  validateModelsConfig,
  validateAgentFilesDrift,
  readLaneStrict,
  writeLane,
  listLanes,
  writeHandoff,
  adoptHandoff,
  resolveRoots,
  cacheFilePath,
  advisorRefAnchors,
  advisorRefStale,
  localConfigPath,
  isLocalOnlyConfigKey,
  trackedLocalOnlyKeyWarning,
} from './lib/state.mjs';
// multisession-native-7/10 (C1/C4/C5/F8): workflow-store.mjs's read/update/
// lock primitives and state-projection.mjs's rebuild functions, composed
// here at the CLI layer — see resolveMutationTarget, withMutationLock, and
// handleStateStartFeature below for exactly where each is used, and
// lib/state-projection.mjs's own header for the C1/F8/C5 contract they
// implement. withWorkflowLock/updateWorkflowAssumingLock (msn-10): the
// mutation verbs below hold `workflow:<id>` for their whole read-validate-
// write body, so the write-through helpers must use the ASSUMING-LOCK form —
// updateWorkflow's own self-locking form would nest a second acquire of the
// SAME lock name and deadlock (see updateWorkflowAssumingLock's own comment
// in workflow-store.mjs).
import { listWorkflows, updateWorkflow, withWorkflowLock, updateWorkflowAssumingLock } from './lib/workflow-store.mjs';
import { rebuildLaneProjection, rebuildStateProjection, rebuildAllProjections } from './lib/state-projection.mjs';
// Lane + session CLI surface (fresh-session-handoff fsh-4, D2/D4): claims.mjs
// stays out of this cell's file scope — these are already-exported read/
// mutate primitives from fsh-3, composed here for presentation (session list)
// and forwarded as-is (bind/unbind), never a second implementation.
import { sessionsDir, readSession, bindSessionLane, unbindSessionLane, resolveSessionId, activeWorkers } from './lib/claims.mjs';
import {
  listCells,
  readyCells,
  readCell,
  addCell,
  addCells,
  previewAddCells,
  updateCell,
  claimCellCrossSession,
  recordVerify,
  capCell,
  blockCell,
  dropCell,
  unclaimCell,
  reopenCell,
  setTier,
  judgeCell,
  scribingDebt,
  globalScribingDebt,
  appendScribingLedger,
  readScribingLedger,
  bestScribingStampMs,
  tierMix,
  ceilingScarcityWarning,
  claimNextCell,
  resetCellBudget,
  deriveChangeClass,
  parseVerificationEvidence,
  evidenceRidesExceptionDoor,
  recordJudgeVerdict,
  archiveFeature,
  unarchiveFeature,
  archivedTotals,
} from './lib/cells.mjs';
import { reserve, release, listReservations, sweepExpired } from './lib/reservations.mjs';
// xwh-2: wires the cross-worktree holds ledger (xwh-1, worktree-holds.mjs)
// into the reservation seam below (handleReservationsReserve/Release/Sweep/
// List) — see resolveHoldTopology's own comment for the holder/mainRoot
// resolution this relies on. hardening-1-7-10 (D3): handleReservationsReserve
// now composes findForeignHolds + reserve() + insertHold() as ONE atomic
// section under withHoldsLock (the standalone, self-locking `mirrorHold` is
// no longer used there — calling it from inside a withHoldsLock section
// would self-deadlock on the same lock; `insertHold` is its unlocked core).
import { findForeignHolds, releaseHolds, sweepExpiredHolds, withHoldsLock, insertHold } from './lib/worktree-holds.mjs';
// D6 — the state.set/gate/worker-add|update|remove/scribing-run verbs below
// each wrap their read-check-write body in this lock (startFeature already
// wraps its own body inside lib/state.mjs); CLI verbs WAIT normally, so no
// maxAttempts override is ever passed here.
import { withStoreLock } from './lib/lock.mjs';
import { writeGrant, removeGrant, listGrants, bootstrapWorktreeStore, createFeatureWorktree, mergeFeatureWorktree } from './lib/worktree-store.mjs';
import { enableHerding, disableHerding, herdingStatus } from './lib/herding.mjs';
import { prepareDispatch } from './lib/dispatch-prepare.mjs';
import {
  classifyNativeTransport,
  NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  PINNED_MODEL_STATUS,
} from './lib/dispatch-guard.mjs';
import { computeSchedule } from './lib/schedule.mjs';
import {
  logDecision,
  supersedeDecision,
  redactDecision,
  activeDecisions,
  archiveDecisions,
  tagDecision,
  tagDecisionsBatch,
  datamark,
  taxonomyFileExists,
  renderDecisionIndex,
  decisionIndexDrift,
  escapeRegExp,
} from './lib/decisions.mjs';
import { captureQueue, addCaptureStub, pendingCaptureStubs, flushCaptureStub } from './lib/capture.mjs';
// intent-anchor ia-1 (D1/D2): the durable verbatim objective. Same
// import-the-lib-directly idiom as capture.mjs above — the CLI is a thin
// presentation layer over lib/intent.mjs, never a second implementation.
import {
  writeIntent,
  readIntent,
  advanceIntent,
  clearIntent,
  precompactBlock,
  resumeBlock,
} from './lib/intent.mjs';
// compaction-hardening (D3): the helper floor — state compact-log/compact-check/
// compact-capsule are thin CLI wrappers over lib/compaction.mjs, the ONE module
// every compaction surface (hooks and verbs alike) calls.
import {
  appendCompactionRecord,
  compactCheck,
  anchorMissing,
  buildCompactCapsule,
  nonAdoptingHandoffOutcome,
  ANCHOR_NUDGE_COMMAND,
} from './lib/compaction.mjs';
import {
  checkBundle,
  knowledgeIndexDrift,
  renderKnowledgeIndexes,
  listConcepts,
  buildContextManifest,
  buildPromotion,
  bundleMode,
  KNOWLEDGE_CONTEXT_LANE_BUDGETS,
} from './lib/knowledge.mjs';
import {
  readBacklogCounts,
  rankBacklog,
  updateReadmeBadges,
  addPbi,
  setPbiStatus,
  amendPbi,
  listPbis,
  renderBacklogPbiView,
} from './lib/backlog.mjs';
import {
  createReview,
  listReviews,
  readReview,
  recordOnReview,
  addCandidate,
  listCandidates,
  deriveCandidateStatus,
  CANDIDATE_STATUSES,
  REVIEW_MODES,
} from './lib/reviews.mjs';
import { readJson, writeJsonAtomic, appendJsonl, hashFile, removeFileIfExists, readJsonl } from './lib/fsutil.mjs';
// tree-hygiene th-4 (D1/D2): the canonical scratch home + its broom.
import { runSweep } from './lib/scratch.mjs';
// perf.mjs is imported ONLY here (never by command-registry.mjs) so it stays
// out of the write-guard fixture's hand-listed VENDORED_LIB_MODULES.
import {
  claudeProjectsRoot,
  resolveTranscript,
  computeMetrics,
  buildSection,
  appendSection,
  readSections,
  writeReport,
  scanCachePath,
  syncSessionsToLog,
  readSessionRecords,
  buildMatrixFromLog,
} from './lib/perf.mjs';
import { KIND_ALIASES, NORMALIZED_KINDS, buildDigest, mergeDigests, clusterEntries, rankClusters } from './lib/feedback.mjs';
// recovery.mjs is imported ONLY here (never by command-registry.mjs), the
// same import discipline perf.mjs already follows above (transcript-recovery
// D-decisions, docs/history/transcript-recovery/CONTEXT.md).
import {
  detectCrashCandidates,
  readTranscriptTail,
  lastDurableSettlement,
  computeMiningWindow,
  buildMiningPrompt,
  scanTranscriptRoots,
} from './lib/recovery.mjs';
import { SCHEMA_VERSION, COMMAND_REGISTRY } from './lib/command-registry.mjs';
import { validate } from './lib/validate-args.mjs';
import { classifySource } from './lib/source-identity.mjs';

// ─── shared small helpers (mirrors requireFlag/readFileText across all 4) ──

function requireFlag(flags, name) {
  const value = flags[name];
  if (value === undefined || value === '' || value === true) {
    throw new Error(`Missing required flag --${name}.`);
  }
  return String(value);
}

// i54-closeout D3: knowledge.context's `--lane` shorthand resolves to a
// numeric --budget BEFORE the generic validate() layer runs (dispatch, below)
// so `budget` can stay a required field, byte-identical to before this cell —
// a bare call with neither --budget nor --lane hits the exact same
// required-missing refusal it always did. Explicit --budget always wins: this
// only fills the gap when --budget is absent. An unrecognized --lane value is
// left unfilled on purpose — validate() then reports it exactly like any
// other invalid enum value would, never a silent fallback to a wrong number.
function resolveKnowledgeContextLaneBudget(commandName, flags) {
  if (commandName !== 'knowledge.context') return;
  if (flags.budget !== undefined && flags.budget !== '' && flags.budget !== true) return;
  const lane = flags.lane;
  if (lane === undefined || lane === '' || lane === true) return;
  const preset = KNOWLEDGE_CONTEXT_LANE_BUDGETS[String(lane)];
  if (preset !== undefined) flags.budget = preset;
}

// ce-1 (cli-ergonomics D1): batch cousin of requireFlag — collects EVERY
// missing/invalid flag in `spec` ({name, enum?}[]) instead of throwing at the
// first, then throws ONE Error naming all of them plus a runnable Example
// line. Same channel as requireFlag (a thrown Error, routed to STDERR by
// emitError — DB3, command-registry.mjs:1024-1031/641-648): only the
// message batches, nothing moves off stderr. Returns {name: string} for
// every spec entry once every flag is present and valid.
function requireFlags(flags, spec, example) {
  const missing = [];
  const invalid = [];
  const values = {};
  for (const { name, enum: allowed } of spec) {
    const value = flags[name];
    if (value === undefined || value === '' || value === true) {
      missing.push(`--${name}`);
      continue;
    }
    const str = String(value);
    if (allowed && !allowed.includes(str)) {
      invalid.push(`--${name} "${str}" (must be one of ${allowed.join(', ')})`);
      continue;
    }
    values[name] = str;
  }
  if (missing.length === 0 && invalid.length === 0) return values;
  const parts = [];
  if (missing.length) parts.push(`missing required flag(s): ${missing.join(', ')}`);
  if (invalid.length) parts.push(`invalid flag(s): ${invalid.join('; ')}`);
  const exampleLine = example ? ` Example: ${example}` : '';
  throw new Error(`${parts.join('; ')}.${exampleLine}`);
}

// Looks up a registry entry's first documented example — used to append a
// runnable "Example: ..." line to batch refusals (requireFlags callers and
// the generic validate()-failure message below) so a fix is one copy-paste
// away, never just a list of what's wrong.
function exampleFor(commandName) {
  const entry = COMMAND_REGISTRY.find((e) => e.name === commandName);
  return entry && Array.isArray(entry.examples) ? entry.examples[0] : undefined;
}

function readFileText(file, label) {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    throw new Error(`Cannot read ${label} file: ${file}`);
  }
}

function parseDeviationsFile(file) {
  const raw = readFileText(file, 'deviations');
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [String(parsed)];
  } catch {
    return raw.split(/\r?\n/).filter((line) => line.trim());
  }
}

function summarizeCell(cell) {
  return `${cell.id} [${cell.status}] (${cell.lane}) ${cell.title}`;
}

function formatDecision(event) {
  const head = `[${event.date}] ${datamark(event.decision)} (id ${event.id}, ${event.type})`;
  const why = `  why: ${datamark(event.rationale)}`;
  const alt = event.alternatives ? `  alternatives: ${datamark(event.alternatives)}` : null;
  return [head, why, alt].filter(Boolean).join('\n');
}

// ─── status: verbatim port of bee_status.mjs's buildStatus/renderText ─────
// (byte-parity requirement, D5 — must stay identical to the original)

const STALE_HANDOFF_MS = 7 * 24 * 60 * 60 * 1000;

// Phases past execution where a feature can close honestly without full
// independent review (SPEC R3/§11.5, decision 565e68d0). Full review is
// user-invoked only — reaching these phases with unreviewed candidates is
// the NORMAL truthful state, not drift, so bee_status posts an informational
// §9 completion line here instead of a staleness warning.
const POST_EXECUTION_REVIEW_PHASES = ['scribing', 'compounding', 'compounding-complete'];

/**
 * review-on-demand summary (review-od-3, SPEC R3/R7/R10/§8/§9): candidate
 * counts by derived status + open (non-approved) session ids + a high-risk
 * unreviewed/stale count (R7). Sourced entirely from lib/reviews.mjs's own
 * derivation (review-od-2) — no second derivation implementation here.
 * Fail-open by construction (per SPEC + cell prohibition): every reviews.mjs
 * read path already degrades rather than throwing (corrupt session file,
 * corrupt/missing ledger, missing git binary), but the whole block is still
 * wrapped so a future change to that contract can never crash bee_status —
 * a corrupt .bee/reviews dir or missing git degrades this block, it never
 * breaks the scout.
 */
function buildReviewBlock(root) {
  const empty = {
    candidates: { total: 0, unreviewed: 0, in_review: 0, reviewed: 0, stale: 0 },
    open_sessions: [],
    high_risk_unreviewed: 0,
  };
  try {
    const candidates = listCandidates(root);
    const sessions = listReviews(root);
    const counts = { total: candidates.length, unreviewed: 0, in_review: 0, reviewed: 0, stale: 0 };
    let highRiskUnreviewed = 0;
    // D2 (cli-performance CONTEXT): one pass-local memo for the whole loop —
    // candidates sharing a covering session's (head,ref)/(ref) pair answer
    // the underlying git question once instead of once per candidate.
    const gitMemo = new Map();
    for (const candidate of candidates) {
      const derived = deriveCandidateStatus(root, candidate, { sessions, gitMemo });
      if (derived.status === 'unreviewed') counts.unreviewed += 1;
      else if (derived.status === 'in review') counts.in_review += 1;
      else if (derived.status === 'reviewed') counts.reviewed += 1;
      else if (derived.status === 'review stale') counts.stale += 1;
      if (
        candidate &&
        candidate.mode === 'high-risk' &&
        (derived.status === 'unreviewed' || derived.status === 'review stale')
      ) {
        highRiskUnreviewed += 1;
      }
    }
    const openSessions = sessions
      .filter((s) => !s.decision || s.decision.status !== 'approved')
      .map((s) => s.id);
    return { candidates: counts, open_sessions: openSessions, high_risk_unreviewed: highRiskUnreviewed };
  } catch {
    return { ...empty, degraded: true };
  }
}

// Session-start crash-detection block for the status payload (transcript-
// recovery D2: "detection is cheap and automatic"). Same fail-open shape as
// buildReviewBlock immediately above: detectCrashCandidates is already
// fail-open at every intermediate read (missing sessions dir, missing
// projects root on hosts with no transcript store e.g. Codex, corrupt
// session/lane/claim records), so this try/catch is belt-and-suspenders —
// a future change to that contract still can never crash bee_status.
// hardening-5: `roots` is additive alongside the pre-existing `candidates`
// field — every configured (or default-only) transcript root's scan result
// (scanned/skipped+reason), so a second-runtime (e.g. Codex) user configuring
// `recovery.transcript_roots` can SEE whether it was actually consulted.
function buildRecoveryBlock(root) {
  try {
    return { candidates: detectCrashCandidates(root), roots: scanTranscriptRoots(root) };
  } catch {
    return { candidates: [], degraded: true };
  }
}

// Per-lane phase/gates/binding rows for the status payload (fresh-session-
// handoff fsh-6, D4). Reused by handleStateLanes below (this cell composes
// no second implementation): every lane record plus the session ids currently
// bound to it (listSessionRecords/listLanes are already-exported fsh-3
// primitives). Zero lanes on disk -> []: the field is additive, never
// altering any pre-existing status field's shape (D4 zero-lane parity).
function buildLaneRows(root) {
  const lanes = listLanes(root);
  const sessions = listSessionRecords(root);
  const boundBy = {};
  for (const session of sessions) {
    if (typeof session.lane === 'string' && session.lane) {
      (boundBy[session.lane] ||= []).push(session.id);
    }
  }
  return lanes.map((lane) => ({ ...lane, bound_sessions: boundBy[lane.feature] || [] }));
}

// lpsp-2 (P2, payload-size): the `lanes` block was measured at 58% of a full
// `status --json` payload on this repo — a per-session context tax paid at
// EVERY session start/compaction (AGENTS.md step 3), not just this call's
// latency. Default `status` now summarizes: the ACTIVE lane in full (the one
// the CALLING session is bound to — resolveSessionId's own env/root-inference
// chain, the exact identity primitive claims/reservations already use, so no
// new precondition logic here), plus counts-by-phase and bare ids for every
// OTHER lane record. `--lanes-full` (buildStatus's lanesFull option) restores
// buildLaneRows's full array unchanged — today's exact shape, byte-for-byte.
// This is a payload-size change only: it never touches what any OTHER
// top-level status field means (phase/mode/feature/gates/cells/
// recommended_next stay byte-identical whichever way this function is
// called).
function buildLaneSummary(root) {
  const lanes = buildLaneRows(root);
  if (lanes.length === 0) return { active: null, counts: {}, ids: [] };
  const sessionId = resolveSessionId({ root });
  let active = null;
  if (sessionId) {
    const session = readSession(root, sessionId);
    if (session && typeof session.lane === 'string' && session.lane) {
      active = lanes.find((l) => l.feature === session.lane) || null;
    }
  }
  const rest = active ? lanes.filter((l) => l.feature !== active.feature) : lanes;
  const counts = {};
  for (const l of rest) counts[l.phase] = (counts[l.phase] || 0) + 1;
  return { active, counts, ids: rest.map((l) => l.feature) };
}

// Honest runtime drift (codex-harness-hardening 1c, decisions 485e949a /
// 579bbad7). The false-green it replaces compared only the ledger version
// string against the running constant — both of which a downgrade rewrites in
// lockstep. Instead, compare the LIVE vendored runtime bytes against the
// per-file sha256 the onboarding ledger recorded at install
// (managed.lib + managed.helpers). A content mismatch, a missing/extra managed
// lib file, or a version-string mismatch is drift — even at the same
// bee_version (PROJ-08). Report-only and fail-open: an absent/legacy/unreadable
// ledger degrades to the version-only signal and NEVER throws, so status always
// renders. The managed file set is derived from the recorded map, never
// hand-listed (crit-pattern 20260714).
function computeRuntimeDrift(root, onboardingRaw) {
  const versionDrift = Boolean(
    onboardingRaw && onboardingRaw.bee_version && onboardingRaw.bee_version !== BEE_VERSION,
  );
  const managed = onboardingRaw && onboardingRaw.managed;
  if (!managed || typeof managed !== 'object') {
    // Legacy/absent managed map: fail-open to the version-only signal.
    return { drift: versionDrift, detail: [] };
  }
  const detail = [];
  const checkGroup = (recorded, relDir) => {
    if (!recorded || typeof recorded !== 'object') return;
    for (const [name, recordedHash] of Object.entries(recorded)) {
      const abs = path.join(root, '.bee', 'bin', relDir, name);
      const relPosix = ['.bee', 'bin', relDir, name].filter(Boolean).join('/');
      let live;
      try {
        // hashFile is the SAME function buildManagedVersions (onboard_bee.mjs)
        // records with — one hasher, so recorder and reader can never disagree.
        live = hashFile(abs);
      } catch {
        detail.push(`${relPosix} (missing)`);
        continue;
      }
      if (live !== recordedHash) detail.push(relPosix);
    }
  };
  checkGroup(managed.lib, 'lib');
  checkGroup(managed.helpers, '');
  // File-set drift for the fully-managed lib dir: an extra .mjs on disk that
  // the recorded map does not list. (Helpers live beside non-managed files in
  // .bee/bin, so extra-detection is scoped to lib to avoid false positives.)
  if (managed.lib && typeof managed.lib === 'object') {
    try {
      for (const f of fs.readdirSync(path.join(root, '.bee', 'bin', 'lib'))) {
        // hasOwnProperty, not `in`: a lib file literally named constructor.mjs /
        // toString.mjs would otherwise resolve through Object.prototype and
        // escape extra-detection.
        if (f.endsWith('.mjs') && !Object.prototype.hasOwnProperty.call(managed.lib, f)) {
          detail.push(`.bee/bin/lib/${f} (extra)`);
        }
      }
    } catch {
      /* fail-open: unreadable lib dir degrades to the checks above */
    }
  }
  return { drift: versionDrift || detail.length > 0, detail };
}

// The bee-hive source tree this repo carries, in canonical-first order: a dev
// checkout's real source (skills/), else a host's vendored projection
// (.claude/skills or .agents/skills). Used to classify the repo's source
// identity for the status `source` field (DIST-04, SRC-01).
function findRepoHive(root) {
  for (const segs of [['skills'], ['.claude', 'skills'], ['.agents', 'skills']]) {
    const hive = path.join(root, ...segs, 'bee-hive');
    try {
      if (fs.existsSync(hive)) return hive;
    } catch {
      /* fail-open: unreadable path is simply not this candidate */
    }
  }
  return null;
}

// config-validate (ao-2ai-1) — readJson(file, undefined) does NOT do what it
// looks like: a default parameter (`fallback = null` in fsutil.mjs) fires on
// an explicitly-passed `undefined` argument too, so that call silently
// returns `null` regardless of whether the file is missing or malformed.
// validateModelsConfig needs to tell those two cases apart (undefined = no
// config file at all, the common/harmless case; null = something WAS read
// and is unusable) — so check existence ourselves rather than leaning on a
// readJson fallback value that JS itself collapses back to the default.
function readRawConfigForValidation(root) {
  const file = path.join(root, '.bee', 'config.json');
  return fs.existsSync(file) ? readJson(file, null) : undefined;
}

// GH #30: `root` here is main()'s already-resolved storeRoot, which for an
// UNGRANTED linked worktree already fell back to the main store (P40
// default, resolveRoots' own comment at its definition) — so a plain read of
// `root` cannot tell "ordinary checkout" apart from "ungranted linked
// worktree quietly sharing main's store". Re-resolving process.cwd() here
// (same pattern as resolveMainRoot/resolveHoldTopology below) recovers the
// worktree identity resolveRoots already computed once inside main()'s own
// findRepoRoot call. Messaging only — this NEVER changes storeRoot selection
// or grant semantics, it only decides whether to print a notice about the
// selection main() already made.
function ungrantedWorktreeNotice(root) {
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch {
    return null;
  }
  if (resolution.worktreeResolution !== 'linked-valid' || !resolution.storeRoot || !resolution.mainRoot) {
    return null;
  }
  const ungranted = path.resolve(resolution.storeRoot) === path.resolve(resolution.mainRoot);
  if (!ungranted) return null;
  return (
    `⚠ This linked worktree is UNGRANTED — it SHARES the main checkout's store ` +
    `(same feature/phase/claims; no isolation). To work an isolated feature: run ` +
    `"bee worktree new --feature <slug>" from the main checkout. To grant isolation ` +
    `to THIS existing worktree instead: run "bee worktree register --feature <slug>" ` +
    `from inside it.`
  );
}

// multisession-native-4 (advisor consult stage 0-1, condition C4): bounded
// tail-window summary of lock-contention telemetry
// (.bee/logs/contention.jsonl, appended by lock.mjs's
// appendContentionTelemetry — msn-3/C3) for `bee status` / `bee status
// --json`. Reads ONLY the last CONTENTION_TAIL_MAX_BYTES of the file via
// readTranscriptTail (the SAME bounded-window primitive transcript-recovery
// already uses for a possibly multi-MB file) — never a full-file scan, no
// matter how large the log grows (must_haves prohibition). Malformed lines
// inside the window are already skipped by readTranscriptTail; a missing or
// unreadable file resolves to [] there, never throwing.
//
// Returns null (buildStatus omits the `contention` key entirely) when the
// log is absent OR when the tail window contains no LOCK_BUSY ('busy')
// events — a status payload with nothing to report about waiting sessions
// stays silent rather than emitting an empty/zero-value block (must_haves:
// "absent log = silent, zero errors").
//
// JSON shape (documented here, mirrored in bee --help --json's schema for
// the `status` command is not required — this is a computed report field,
// not an input parameter):
//   contention: {
//     busy_count: number,            // count of 'busy' (LOCK_BUSY) events in the tail window
//     recent_busy: [{                // newest-first, capped at CONTENTION_RECENT_BUSY_LIMIT
//       ts: string|null,
//       lock_name: string|null,
//       holder_session: string|null, // who held the lock the caller was waiting on
//       caller_session: string|null, // who was waiting
//       lock_wait_ms: number|null,
//     }],
//     top_locks: [{ lock_name: string, busy_count: number }], // sorted desc, capped at CONTENTION_TOP_LOCKS_LIMIT
//     worst_wait_ms: number|null,    // max lock_wait_ms across ALL events in the window (busy or eventually-acquired)
//     worst_wait_lock: string|null,  // the lock_name that produced worst_wait_ms
//   }
const CONTENTION_TAIL_MAX_BYTES = 65536; // 64KB tail window — see must_haves prohibition on full-file scans
const CONTENTION_RECENT_BUSY_LIMIT = 5;
const CONTENTION_TOP_LOCKS_LIMIT = 5;

function buildContentionSummary(root) {
  const file = path.join(root, '.bee', 'logs', 'contention.jsonl');
  const events = readTranscriptTail(file, CONTENTION_TAIL_MAX_BYTES);
  if (events.length === 0) return null;
  const busyEvents = events.filter((e) => e && typeof e === 'object' && e.result === 'busy');
  if (busyEvents.length === 0) return null;

  const byLock = new Map();
  for (const e of busyEvents) {
    const name = typeof e.lock_name === 'string' && e.lock_name ? e.lock_name : 'unknown';
    byLock.set(name, (byLock.get(name) || 0) + 1);
  }
  const topLocks = [...byLock.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, CONTENTION_TOP_LOCKS_LIMIT)
    .map(([lock_name, busy_count]) => ({ lock_name, busy_count }));

  // worst_wait_ms spans every event in the window (not just 'busy') — a
  // caller that eventually acquired after a long retry still answers "why
  // was I waiting" just as informatively as one that hit LOCK_BUSY outright.
  let worstWaitMs = null;
  let worstWaitLock = null;
  for (const e of events) {
    if (!e || typeof e !== 'object') continue;
    const waitMs = typeof e.lock_wait_ms === 'number' && Number.isFinite(e.lock_wait_ms) ? e.lock_wait_ms : null;
    if (waitMs !== null && (worstWaitMs === null || waitMs > worstWaitMs)) {
      worstWaitMs = waitMs;
      worstWaitLock = typeof e.lock_name === 'string' && e.lock_name ? e.lock_name : null;
    }
  }

  const recentBusy = busyEvents
    .slice(-CONTENTION_RECENT_BUSY_LIMIT)
    .reverse()
    .map((e) => ({
      ts: typeof e.ts === 'string' ? e.ts : null,
      lock_name: typeof e.lock_name === 'string' ? e.lock_name : null,
      holder_session: e.holder_session ?? null,
      caller_session: e.caller_session ?? null,
      lock_wait_ms: typeof e.lock_wait_ms === 'number' ? e.lock_wait_ms : null,
    }));

  return {
    busy_count: busyEvents.length,
    recent_busy: recentBusy,
    top_locks: topLocks,
    worst_wait_ms: worstWaitMs,
    worst_wait_lock: worstWaitLock,
  };
}

function buildStatus(root, { lanesFull = false } = {}) {
  const state = readState(root);
  const onboardingRaw = readOnboarding(root);
  const handoff = readHandoff(root);
  const cells = listCells(root);
  const counts = { open: 0, claimed: 0, capped: 0, blocked: 0 };
  for (const cell of cells) {
    if (counts[cell.status] !== undefined) counts[cell.status] += 1;
  }
  // cells-archive-2: sourced from the archive summary ledger (archivedSummary
  // via archivedTotals), NEVER a directory scan of .bee/cells/archive/ — that
  // scan is exactly the hot-path cost archiving exists to avoid. The active
  // counts above stay untouched (still one fast readdir of .bee/cells/), so
  // `capped + archived.capped` is the honest grand total across both stores.
  const archived = archivedTotals(root);
  const allReservations = listReservations(root);
  const active = listReservations(root, { activeOnly: true });
  const expiredUnreleased = allReservations.filter(
    (r) => r.released_at == null && !active.includes(r),
  );

  const commands = readConfig(root).commands || {};
  const backlog = readBacklogCounts(root);

  const staleness = [];
  if (Object.keys(commands).length === 0) {
    staleness.push(
      "No standard commands recorded — capture the host project's setup/start/test/verify into .bee/config.json `commands` so sessions can run the CI status gate.",
    );
  }
  if (onboardingRaw && onboardingRaw.bee_version && onboardingRaw.bee_version !== BEE_VERSION) {
    staleness.push(
      `Onboarding installed bee ${onboardingRaw.bee_version} but plugin is ${BEE_VERSION} — re-run onboarding.`,
    );
  }
  if (handoff && handoff.written_at) {
    const age = Date.now() - Date.parse(handoff.written_at);
    if (Number.isFinite(age) && age > STALE_HANDOFF_MS) {
      staleness.push(`HANDOFF.json is older than 7 days (written ${handoff.written_at}).`);
    }
  }
  if (expiredUnreleased.length > 0) {
    staleness.push(
      `${expiredUnreleased.length} reservation(s) expired but never released — run bee_reservations.mjs sweep.`,
    );
  }
  if (hasStaleAdvisorKey(root)) {
    staleness.push(STALE_ADVISOR_KEY_WARNING);
  }
  // config-validate (ao-2ai-1): the same validator `bee config validate` runs
  // explicitly is also read here so a malformed/prompt-less/unsafe cli-tier
  // config surfaces on every `bee status` — joined onto staleness_warnings,
  // never replacing anything already collected above.
  for (const problem of validateModelsConfig(readRawConfigForValidation(root))) {
    staleness.push(`config validate [${problem.code}]${problem.runtime ? ` models.${problem.runtime}.${problem.slot}:` : ''} ${problem.message}`);
  }
  // W3 drift advisory (ao-3b-2, AO12): a rendered .claude/agents/bee-*.md
  // whose model: frontmatter no longer matches the configured tier. Advisory
  // only — the dispatch itself is already protected by the guard's marker+
  // param equality rule, independent of this check. Appended onto the same
  // staleness_warnings output as the config-validate problems above, never a
  // separate field.
  for (const problem of validateAgentFilesDrift(root, readRawConfigForValidation(root))) {
    staleness.push(`config validate [${problem.code}] ${problem.agent} (${problem.slot}): ${problem.message}`);
  }
  if (!isKnownPhase(state.phase)) {
    staleness.push(
      `Unknown phase "${state.phase}" — not in the enum (${PHASES.join(', ')}; terminal alias: compounding-complete). Set state.phase to a valid value (idle at feature close); invented phases break machine-checkable handoffs (decision 0004).`,
    );
  }
  const review = buildReviewBlock(root);
  const recovery = buildRecoveryBlock(root);

  const executionApproved = state.approved_gates?.execution === true;
  const ready = readyCells(root, state.feature || null);
  let recommended;
  if (!onboardingRaw) {
    recommended = 'Onboarding missing — run bee-hive onboarding.';
  } else if (handoff) {
    recommended = 'HANDOFF present — present it to the user and WAIT. Never auto-resume.';
  } else if (state.phase === 'swarming' && !executionApproved) {
    recommended = 'NOT ready to swarm: gate "execution" is not approved.';
  } else if (executionApproved && ready.length > 0) {
    recommended = `${ready.length} ready cell(s): ${ready.map((c) => c.id).join(', ')} — orchestrator assigns them.`;
  } else if (POST_EXECUTION_REVIEW_PHASES.includes(state.phase) && review.candidates.unreviewed > 0) {
    // §11.5 — never propose bee-reviewing as an automatic post-execution
    // step; report the candidate count and wait for explicit user intent.
    recommended = `${review.candidates.unreviewed} review candidate(s) awaiting: full review is user-invoked only, never dispatched automatically.`;
  } else {
    recommended = state.next_action || 'Invoke bee-hive.';
  }

  const runtimeDrift = computeRuntimeDrift(root, onboardingRaw);
  const repoHive = findRepoHive(root);
  const sourceId = repoHive
    ? classifySource({ hiveDir: repoHive, homeDir: os.homedir() })
    : { kind: 'unknown', root: null };
  const worktreeNotice = ungrantedWorktreeNotice(root);
  const contention = buildContentionSummary(root);
  return {
    onboarding: {
      installed: Boolean(onboardingRaw),
      bee_version: onboardingRaw?.bee_version ?? null,
      plugin_version: BEE_VERSION,
      drift: runtimeDrift.drift,
      ...(runtimeDrift.detail.length > 0 ? { drift_detail: runtimeDrift.detail } : {}),
    },
    // Source identity of the bee-hive tree this repo carries (DIST-04, SRC-01):
    // report-only — never a decision input here. Same classifier onboarding uses.
    source: { kind: sourceId.kind, root: sourceId.root },
    phase: state.phase,
    mode: state.mode,
    feature: state.feature,
    gates: state.approved_gates,
    gate_bypass: bypassLevel(root) !== 'off',
    gate_bypass_level: bypassLevel(root),
    models: readConfig(root).models,
    tier_mix: tierMix(root, { feature: state.feature || null }),
    ceiling_scarcity: ceilingScarcityWarning(root),
    handoff,
    cells: { ...counts, archived },
    // lpsp-2: summarized by default (active lane in full + counts/ids for the
    // rest); --lanes-full restores buildLaneRows's full array unchanged.
    lanes: lanesFull ? buildLaneRows(root) : buildLaneSummary(root),
    review,
    recovery,
    // scribing-integrity si-1 (D5): `orphaned` is ADDITIVE — the pre-existing
    // {count, cells} shape scoped to the active feature is unchanged; the
    // sweep across every feature rides alongside it as a new sub-field.
    scribing_debt: { ...scribingDebt(root), orphaned: globalScribingDebt(root) },
    capture_queue: (() => {
      const queue = captureQueue(root);
      return { count: queue.count, ids: queue.stubs.map((s) => s.id) };
    })(),
    pbi: backlog
      ? { proposed: backlog.proposed, in_flight: backlog.inFlight, done: backlog.done }
      : null,
    commands,
    active_reservations: active,
    // D6 (multisession-native-8): derived from live-heartbeat sessions x
    // claims (claims.mjs activeWorkers) — never state.json's hand-mutated
    // `workers` array (that field is now display-only, written by the
    // `state worker *` CLI verbs for backward compat but read by nothing
    // here). Same always-present-array convention as active_reservations
    // above (an empty array reports "nobody active", not an omitted key).
    workers: activeWorkers(root),
    critical_patterns_present: fs.existsSync(
      path.join(root, 'docs', 'history', 'learnings', 'critical-patterns.md'),
    ),
    recent_decisions: activeDecisions(root, { recent: 3 }).map((event) => ({
      id: event.id,
      date: event.date,
      decision: datamark(event.decision),
    })),
    staleness_warnings: staleness,
    recommended_next: recommended,
    // GH #30 (messaging only, wux-1): omitted entirely (not even `null`) for
    // an ordinary checkout or a GRANTED linked worktree — those two cases
    // must stay byte-identical to pre-cell output; the field only appears at
    // all when the current checkout is an ungranted linked worktree.
    ...(worktreeNotice ? { worktree_notice: worktreeNotice } : {}),
    // multisession-native-4: same additive, omit-when-nothing-to-report
    // convention as worktree_notice above — an absent/empty-of-busy-events
    // contention.jsonl keeps status byte-identical to pre-cell output.
    ...(contention ? { contention } : {}),
  };
}

function formatSlot(value) {
  if (value == null) return 'null';
  if (typeof value === 'string') return value;
  if (value.kind === 'cli') return `cli(${String(value.command).split(/\s+/)[0]})`;
  if (value.model) return value.effort ? `${value.model}@${value.effort}` : value.model;
  return 'null';
}

// One full lane row's text rendering (fsh-6, D4) — shared by the legacy
// full-array render (--lanes-full) and, for the active lane only, the lpsp-2
// default summary render below.
function formatLaneRow(l) {
  const gates = GATE_NAMES.map((g) => `${g}=${l.approved_gates[g] ? 'approved' : 'pending'}`).join(' ');
  const bound = l.bound_sessions.length ? ` sessions=${l.bound_sessions.join(',')}` : '';
  return `${l.feature} [${l.phase}] ${gates}${bound}`;
}

// lpsp-2: text-render counterpart of buildLaneSummary's {active, counts, ids}
// shape. null when there is nothing to report (zero lanes on disk) — same
// additive, no-line-at-all convention the legacy array render already uses,
// so a zero-lane text render stays byte-identical to before this cell.
function formatLaneSummaryLine(summary) {
  const parts = [];
  if (summary.active) parts.push(`active: ${formatLaneRow(summary.active)}`);
  if (summary.ids.length > 0) {
    const countsStr = Object.entries(summary.counts)
      .map(([phase, n]) => `${phase}=${n}`)
      .join(' ');
    parts.push(`${summary.ids.length} other lane(s) [${countsStr}] (ids: ${summary.ids.join(', ')})`);
  }
  return parts.length > 0 ? `Lanes: ${parts.join(' | ')}` : null;
}

function renderStatusText(status) {
  const lines = [
    // GH #30 (wux-1): prepended ONLY when buildStatus set the field (ungranted
    // linked worktree) — an ordinary checkout or a granted linked worktree has
    // no `worktree_notice` key at all, so this line is simply absent and every
    // line below stays byte-identical to pre-cell output.
    ...(status.worktree_notice ? [status.worktree_notice] : []),
    `bee status (plugin v${BEE_VERSION})`,
    `Onboarding: ${status.onboarding.installed ? `installed (bee ${status.onboarding.bee_version})` : 'MISSING'}${status.onboarding.drift ? ` [drift${status.onboarding.drift_detail ? `: ${status.onboarding.drift_detail.length} file(s)` : ''}]` : ''}`,
    `Phase: ${status.phase} | Mode: ${status.mode ?? 'none'} | Feature: ${status.feature ?? 'none'}`,
    `Gates: ${GATE_NAMES.map((g) => `${g}=${status.gates?.[g] ? 'approved' : 'pending'}`).join(' ')}`,
    ...(status.gate_bypass_level && status.gate_bypass_level !== 'off'
      ? [bypassBanner(status.gate_bypass_level)]
      : []),
    `Handoff: ${status.handoff ? 'PRESENT — surface it and WAIT' : 'none'}`,
    `Cells: open=${status.cells.open} claimed=${status.cells.claimed} capped=${status.cells.capped} blocked=${status.cells.blocked} archived=${status.cells.archived.total} (total capped=${status.cells.capped + status.cells.archived.capped})`,
    // Lanes (fsh-6, D4): additive — zero lanes on disk renders no line at
    // all, keeping every zero-lane text render byte-identical to today.
    // lpsp-2: `status.lanes` is either the legacy full array (--lanes-full,
    // byte-identical rendering to before this cell) or the new default
    // summary object ({active, counts, ids}) — Array.isArray tells them apart
    // without buildStatus threading a second flag through here.
    ...(() => {
      if (Array.isArray(status.lanes)) {
        return status.lanes.length > 0 ? [`Lanes: ${status.lanes.map(formatLaneRow).join(' | ')}`] : [];
      }
      const line = formatLaneSummaryLine(status.lanes);
      return line ? [line] : [];
    })(),
    // §9 — reaching a post-execution phase with unreviewed candidates is the
    // NORMAL truthful close (R3): informational, never a staleness warning.
    ...(POST_EXECUTION_REVIEW_PHASES.includes(status.phase) && status.review?.candidates?.unreviewed > 0
      ? [
          `Completed and verified; independent review not requested; ${status.review.candidates.unreviewed} candidate(s) awaiting review.`,
        ]
      : []),
    ...(status.scribing_debt && status.scribing_debt.count > 0
      ? [`Scribing debt: ${status.scribing_debt.count} behavior_change cell(s) uncaptured (${status.scribing_debt.cells.join(', ')}) — run bee-scribing capture (decision 0011)`]
      : []),
    ...(status.capture_queue && status.capture_queue.count > 0
      ? [`Capture queue: ${status.capture_queue.count} stub(s) pending flush — run bee-scribing flush at wrap-up, before compact/clear, or now if idle (decision 0017)`]
      : []),
    ...(status.pbi
      ? [`PBI: ${status.pbi.done} done / ${status.pbi.in_flight} in-flight / ${status.pbi.proposed} proposed`]
      : []),
    `Standard commands: ${
      COMMAND_KEYS.filter((key) => status.commands?.[key])
        .map((key) => `${key}=${status.commands[key]}`)
        .join(' | ') || 'none recorded'
    }`,
    `Active reservations: ${status.active_reservations.length}`,
    // D6 (multisession-native-8): derived-view count, same always-present
    // convention as the reservations line above.
    `Active workers: ${status.workers.length}`,
    // multisession-native-4: additive line, omitted whenever buildStatus
    // omitted the `contention` key (no log, or no busy events in the tail
    // window) — same "absent = silent" convention as the review/scribing
    // lines above.
    ...(status.contention
      ? [
          `Contention: ${status.contention.busy_count} LOCK_BUSY event(s) recently (top: ${status.contention.top_locks.map((l) => `${l.lock_name}×${l.busy_count}`).join(', ')}); worst wait ${status.contention.worst_wait_ms}ms${status.contention.worst_wait_lock ? ` on "${status.contention.worst_wait_lock}"` : ''}`,
        ]
      : []),
    `Critical patterns file: ${status.critical_patterns_present ? 'present' : 'absent'}`,
    ...(status.models
      ? [
          `Models (claude): generation=${formatSlot(status.models.claude.generation)} extraction=${formatSlot(status.models.claude.extraction)} review=${formatSlot(status.models.claude.review)} · ceiling = the session model (keep it scarce; decisions 0012/0015/0021)`,
        ]
      : []),
    ...(status.tier_mix && status.tier_mix.tiered > 0
      ? [`Tier mix: extraction=${status.tier_mix.counts.extraction} generation=${status.tier_mix.counts.generation} ceiling=${status.tier_mix.counts.ceiling} untiered=${status.tier_mix.counts.untiered} (ceiling ${Math.round(status.tier_mix.ceilingShare * 100)}%)`]
      : []),
    ...(status.ceiling_scarcity
      ? [`⚠ Ceiling scarcity: ${status.ceiling_scarcity.ceiling}/${status.ceiling_scarcity.tiered} tiered cells on ceiling (${status.ceiling_scarcity.pct}%) — re-tier routine cells (decision 0012)`]
      : []),
    // R7 — high-risk changes never silently trigger review; bee only warns.
    ...(status.review?.high_risk_unreviewed > 0
      ? [
          `⚠ High-risk unreviewed: ${status.review.high_risk_unreviewed} high-risk candidate(s) have not passed independent review — bee will not auto-dispatch reviewers; request review before merge/release.`,
        ]
      : []),
  ];
  if (status.recent_decisions.length > 0) {
    lines.push('Recent decisions:');
    for (const d of status.recent_decisions) lines.push(`- ${d.decision} (${d.date})`);
  }
  if (status.staleness_warnings.length > 0) {
    lines.push('Staleness warnings:');
    for (const w of status.staleness_warnings) lines.push(`- ${w}`);
  }
  lines.push(`Recommended next: ${status.recommended_next}`);
  return lines.join('\n');
}

// ─── per-group handlers: reimplement each existing CLI's run() against the
// same lib functions (D5) — every handler's {result, text} matches the
// original byte-for-byte in the steady state (no manifest drift). ──────────

function handleStatus(root, flags) {
  const lanesFull = Boolean(flags && flags['lanes-full'] === true);
  const status = buildStatus(root, { lanesFull });
  return { result: status, text: renderStatusText(status) };
}

function handleCellsList(root, flags) {
  const cells = listCells(root, {
    feature: flags.feature ? String(flags.feature) : null,
    status: flags.status ? String(flags.status) : null,
  });
  return { result: cells, text: cells.length ? cells.map(summarizeCell).join('\n') : 'No cells.' };
}

function handleCellsReady(root, flags) {
  const cells = readyCells(root, flags.feature ? String(flags.feature) : null);
  return { result: cells, text: cells.length ? cells.map(summarizeCell).join('\n') : 'No ready cells.' };
}

function handleCellsShow(root, flags) {
  const id = requireFlag(flags, 'id');
  const cell = readCell(root, id);
  if (!cell) throw new Error(`Cell "${id}" not found.`);
  return { result: cell, text: JSON.stringify(cell, null, 2) };
}

// H2 (post-advisor-hardening, learnings 20260717 "the release-manifest trap
// recurred at cell-writing time, second sighting"): a cell whose verify chain
// ends in `release_manifest.mjs --check` but whose `files` omits the manifest
// itself strands a cold worker at a red verify with no sanctioned fix. This
// lint is advisory ONLY — it never refuses the write and never changes the
// exit code (CONTEXT.md H2); it just names the trap and the fix at authoring
// time, the moment `cells add`/`cells update` writes the offending shape.
const RELEASE_MANIFEST_LINT_PATH = 'docs/history/codex-harness-hardening/release-manifest.json';

// Tolerates every malformed shape silently (missing/non-string verify,
// missing/non-array files) — the lint must never throw on a bad cell; that
// judgment belongs to validateNewCell/updateCell's own refusals, not this
// advisory pass.
export function manifestLintWarning(cell) {
  if (!cell || typeof cell !== 'object') return null;
  if (typeof cell.verify !== 'string' || !cell.verify.includes('release_manifest')) return null;
  const files = Array.isArray(cell.files) ? cell.files : [];
  if (files.includes(RELEASE_MANIFEST_LINT_PATH)) return null;
  const id = typeof cell.id === 'string' && cell.id ? cell.id : '(unknown id)';
  return (
    `WARNING: cell "${id}" verify mentions release_manifest but files is missing ` +
    `"${RELEASE_MANIFEST_LINT_PATH}" — a cold worker will hit red verify with no ` +
    `sanctioned fix. FIX: add the manifest path to files; regenerate it only via ` +
    `"node scripts/release_manifest.mjs --write".`
  );
}

// Written straight to stderr (drift-warning precedent at emit()'s
// manifest_changed line below) rather than folded into the handler's
// {result, text} — so the warning surfaces identically whether the caller
// used --json or text output, and never reshapes stdout's machine-parseable
// result (P1 discipline the same file already documents for drift).
function emitManifestLintWarnings(cells) {
  for (const cell of Array.isArray(cells) ? cells : [cells]) {
    const warning = manifestLintWarning(cell);
    if (warning) process.stderr.write(`${warning}\n`);
  }
}

// D3 (self-correcting-loop) — judge-standard sufficiency matrix (F4): advisory
// WARNING at `cells add`/`cells update`, STDERR only, manifest-lint pattern
// (pah-2 emitManifestLintWarnings precedent above) — NEVER folded into the
// JSON result, NEVER a refusal at authoring (CONTEXT D3). change_class
// resolution matches cells.mjs's own deriveChangeClass exactly: an
// unclassified cell (no change_class, no behavior_change:true) gets no check
// at all. Each class' minimum is checked against what is knowable at
// authoring time — the cell's own `verify` string, or (for `behavior`) any
// pre-attached verification_evidence; most `behavior` cells will warn at add
// time since evidence is normally attached later at cap, which is expected
// and harmless (advisory-only).
// Plain case-insensitive substring checks, not \b-anchored regexes — verify
// strings are free-form shell commands where the keyword often sits inside
// an underscore-joined filename (e.g. "test_contract.mjs"), and \w includes
// underscore, so a \b boundary silently fails to match right there. This is
// an advisory heuristic (never a refusal), so a substring match is the right
// amount of precision.
function verifyMentions(cell, ...needles) {
  const verify = String(cell.verify || '').toLowerCase();
  return needles.some((needle) => verify.includes(needle));
}

const JUDGE_STANDARD_MINIMUMS = {
  formatting: {
    label: 'a lint/typecheck check present in verify',
    test: (cell) => verifyMentions(cell, 'lint', 'typecheck', 'tsc'),
  },
  bugfix: {
    label: 'verify names a test path',
    test: (cell) => verifyMentions(cell, 'test', 'spec'),
  },
  behavior: {
    label: 'red_failure_evidence attached (verification_evidence.red_failure_evidence)',
    test: (cell) => {
      const evidence = parseVerificationEvidence(cell.verification_evidence);
      return typeof evidence.red_failure_evidence === 'string' && evidence.red_failure_evidence.trim().length > 0;
    },
  },
  api: {
    label: 'a contract/integration test named in verify',
    test: (cell) => verifyMentions(cell, 'contract', 'integration'),
  },
  security: {
    label: 'a negative-path/security test named in verify',
    test: (cell) => verifyMentions(cell, 'security', 'negative'),
  },
  migration: {
    label: 'forward + rollback checks named in verify',
    test: (cell) => verifyMentions(cell, 'forward') && verifyMentions(cell, 'rollback', 'down', 'revert'),
  },
};

// Tolerates every malformed shape silently, same discipline as
// manifestLintWarning above — the advisory must never throw on a bad cell.
export function judgeStandardWarning(cell) {
  if (!cell || typeof cell !== 'object') return null;
  const changeClass = deriveChangeClass(cell);
  if (!changeClass) return null; // unclassified — no matrix check (CONTEXT D3)
  const minimum = JUDGE_STANDARD_MINIMUMS[changeClass];
  if (!minimum || minimum.test(cell)) return null;
  const id = typeof cell.id === 'string' && cell.id ? cell.id : '(unknown id)';
  return (
    `JUDGE_STANDARD_INSUFFICIENT: cell "${id}" is change_class "${changeClass}" but is missing the matrix ` +
    `minimum — ${minimum.label}. Advisory only (never a refusal at authoring); see CONTEXT.md D3 for the full matrix.`
  );
}

function emitJudgeStandardWarnings(cells) {
  for (const cell of Array.isArray(cells) ? cells : [cells]) {
    const warning = judgeStandardWarning(cell);
    if (warning) process.stderr.write(`${warning}\n`);
  }
}

// F5: at CAP time (not authoring), a behavior-class cell that rode the
// pre-existing deliberate_exceptions door skipped the D3 length/duplicate
// floor entirely (capCell's own contract) — note that on STDERR so it is
// never silent, without turning it into a refusal. Recomputed from the
// returned (already-capped) cell, same recompute-not-side-channel discipline
// as emitManifestLintWarnings/emitJudgeStandardWarnings above.
function emitJudgeStandardCapAdvisory(cell) {
  if (!cell || typeof cell !== 'object') return;
  if (deriveChangeClass(cell) !== 'behavior') return;
  const evidence = parseVerificationEvidence(cell.trace && cell.trace.verification_evidence);
  if (!evidenceRidesExceptionDoor(evidence)) return;
  process.stderr.write(
    `JUDGE_STANDARD_INSUFFICIENT: behavior-class cell "${cell.id}" capped via the deliberate_exceptions door — ` +
      `the D3 red_failure_evidence floor (>=80 chars, non-duplicate) was not enforced for this cap (F5).\n`,
  );
}

function handleCellsAdd(root, flags) {
  let text;
  if (flags.stdin === true) text = fs.readFileSync(0, 'utf8');
  else text = readFileText(requireFlag(flags, 'file'), 'cell');
  let cell;
  try {
    cell = JSON.parse(text);
  } catch {
    throw new Error('add: input is not valid JSON.');
  }
  // ce-2: --dry-run reports the SAME whole-batch validation addCells uses
  // (previewAddCells — schema, regen obligations, duplicate ids, dependency
  // cycle, all folded per-cell) instead of writing. A single-object payload
  // is wrapped as a one-element array so the report shape is always
  // {dry_run, ok, cells:[{id, ok, problems}]} — nothing is ever persisted,
  // clean or dirty (worker-prune's --dry-run precedent), but unlike prune's
  // always-exit-0 report this one carries a pass/fail verdict, so the exit
  // code follows it (reservations reserve's `result.ok ? 0 : 1` precedent).
  if (flags['dry-run'] !== undefined) {
    const batch = Array.isArray(cell) ? cell : [cell];
    const preview = previewAddCells(root, batch);
    const lines = preview.cells.map(
      (c) => `${c.ok ? 'OK' : 'FAIL'} ${c.id}${c.problems.length ? `: ${c.problems.join('; ')}` : ''}`,
    );
    const summary = preview.ok
      ? `dry-run: ${batch.length} cell(s) valid — nothing written.`
      : `dry-run: ${preview.cells.filter((c) => !c.ok).length} of ${batch.length} cell(s) failed validation — nothing written.`;
    return {
      result: { dry_run: true, ok: preview.ok, cells: preview.cells },
      text: [summary, ...lines].join('\n'),
      exitCode: preview.ok ? 0 : 1,
    };
  }
  // A JSON array is a batch: every cell validated before any is written
  // (all-or-nothing), so one heredoc creates a whole slice in one call
  // (ported from bee_cells.mjs's own add case, dispatcher-unify du-4 —
  // the cells-batch-add regression the pinned test_lib "a JSON array on
  // --stdin creates the whole slice in one call" check exercises).
  if (Array.isArray(cell)) {
    const added = addCells(root, cell);
    emitManifestLintWarnings(added);
    emitJudgeStandardWarnings(added);
    return {
      result: added,
      text: added.map((c) => `Added ${summarizeCell(c)}`).join('\n'),
    };
  }
  const added = addCell(root, cell);
  emitManifestLintWarnings(added);
  emitJudgeStandardWarnings(added);
  return { result: added, text: `Added ${summarizeCell(added)}` };
}

async function handleCellsUpdate(root, flags) {
  // Strict flag validation (workers-prune discipline): a typoed flag on a
  // mutating verb must refuse, never silently no-op into a bad patch.
  for (const name of Object.keys(flags)) {
    if (!['id', 'file', 'stdin'].includes(name)) {
      throw new Error(`update: unknown flag --${name}. Use: --id ID --file patch.json | --stdin [--json].`);
    }
  }
  const id = requireFlag(flags, 'id');
  let text;
  if (flags.stdin === true) text = fs.readFileSync(0, 'utf8');
  else text = readFileText(requireFlag(flags, 'file'), 'patch');
  let patch;
  try {
    patch = JSON.parse(text);
  } catch {
    throw new Error('update: patch input is not valid JSON.');
  }
  // hardening-4b: updateCell's read-check-write now runs under
  // withStoreLock, so it is async — every handler below awaits it (dispatch
  // already does `await handler(...)`, so this only needed the local await).
  const updated = await updateCell(root, id, patch);
  // Lint the MERGED cell (updateCell's return), not the raw patch — a patch
  // that only touches `title` still carries the cell's existing verify/files
  // through the merge, and the trap is exactly as live post-update as it was
  // pre-update if the merged shape now qualifies.
  emitManifestLintWarnings(updated);
  emitJudgeStandardWarnings(updated);
  return {
    result: updated,
    text: `Updated ${updated.id} (${Object.keys(patch).join(', ')}).`,
  };
}

// D1 (msh-2): re-backed by the same claims.mjs claim-file-first sequence
// claim-next already uses (via claimCellCrossSession) — the claim file is
// acquired BEFORE the cell JSON flips, so a losing concurrent claimant gets a
// typed CLAIMED refusal naming the owner + expiry instead of silently
// double-owning the cell. D3: --session-id is optional — resolveSessionId
// falls back to CLAUDE_CODE_SESSION_ID, then to a legal sessionless claim
// (single-session flow keeps working exactly as before, with no id at all).
async function handleCellsClaim(root, flags) {
  const id = requireFlag(flags, 'id');
  const worker = requireFlag(flags, 'worker');
  const sessionId = resolveSessionId({
    flag: flags['session-id'] !== undefined ? String(flags['session-id']) : undefined,
  });
  const ttl = flags.ttl !== undefined ? Number.parseInt(String(flags.ttl), 10) : undefined;
  if (flags.ttl !== undefined && (!Number.isFinite(ttl) || ttl <= 0)) {
    throw new Error('--ttl must be a positive integer (seconds).');
  }
  // hardening-4b: claimCellCrossSession composes claimCell, now
  // withStoreLock-wrapped (async).
  const result = await claimCellCrossSession(root, { sessionId, worker, cellId: id, ttl });
  if (!result.ok) {
    throw new Error(`claim: ${result.code} — ${result.reason}`);
  }
  return { result: result.cell, text: `Claimed ${result.cell.id} for ${result.cell.trace.worker}.` };
}

// D4 (msh-4): the ownership pair shared by every claim-aware mutator below —
// --session-id resolves like everywhere else (explicit flag, else
// CLAUDE_CODE_SESSION_ID at the lib layer via resolveSessionId), and
// --force-ownership is the audited rescue door (typed refusal otherwise).
function ownershipFlags(flags) {
  return {
    sessionId: flags['session-id'] !== undefined ? String(flags['session-id']) : undefined,
    forceOwnership: flags['force-ownership'] === true,
  };
}

async function handleCellsVerify(root, flags) {
  const id = requireFlag(flags, 'id');
  const command = requireFlag(flags, 'command');
  const passedRaw = requireFlag(flags, 'passed');
  if (passedRaw !== 'true' && passedRaw !== 'false') {
    throw new Error('--passed must be "true" or "false".');
  }
  const output = flags['output-file']
    ? readFileText(String(flags['output-file']), 'output')
    : flags.output
      ? String(flags.output)
      : null;
  // D1: --signature is the worker-suppliable override for the ledger's
  // failure_signature; omitted, recordVerify falls back to the mechanical
  // normalizer on `output`.
  const signature = flags.signature !== undefined ? String(flags.signature) : null;
  // GH #27.2 (ghf-4): recordVerify's read-mutate-write body now runs under
  // withStoreLock, so it is async — every handler below awaits it (dispatch
  // already does `await handler(...)`, so this only needed the local await).
  const cell = await recordVerify(root, id, {
    command,
    output,
    passed: passedRaw === 'true',
    signature,
    ...ownershipFlags(flags),
  });
  return { result: cell, text: `Recorded verify on ${cell.id}: passed=${cell.trace.verify_passed}.` };
}

async function handleCellsCap(root, flags) {
  const id = requireFlag(flags, 'id');
  const deviations = flags['deviations-file'] ? parseDeviationsFile(String(flags['deviations-file'])) : [];
  const cell = await capCell(root, id, {
    outcome: flags.outcome ? String(flags.outcome) : undefined,
    files_changed: flags.files
      ? String(flags.files)
          .split(',')
          .map((f) => f.trim())
          .filter(Boolean)
      : [],
    behavior_change: flags['behavior-change'] === true ? true : undefined,
    verification_evidence: flags['evidence-stdin']
      ? fs.readFileSync(0, 'utf8')
      : flags['evidence-file']
        ? readFileText(String(flags['evidence-file']), 'evidence')
        : null,
    deviations,
    friction: flags.friction ? String(flags.friction) : null,
    overrideJudge: flags['override-judge'] !== undefined ? String(flags['override-judge']) : null,
    ...ownershipFlags(flags),
  });
  emitJudgeStandardCapAdvisory(cell); // F5
  return { result: cell, text: `Capped ${cell.id} at ${cell.trace.capped_at}.` };
}

async function handleCellsBlock(root, flags) {
  const cell = await blockCell(root, requireFlag(flags, 'id'), requireFlag(flags, 'reason'), ownershipFlags(flags));
  return { result: cell, text: `Blocked ${cell.id}.` };
}

// hardening-4b: dropCell/unclaimCell/reopenCell are now withStoreLock-wrapped
// (async) — every handler below awaits it (dispatch already does `await
// handler(...)`, so this only needed the local await + async keyword).
async function handleCellsDrop(root, flags) {
  const cell = await dropCell(root, requireFlag(flags, 'id'), requireFlag(flags, 'reason'));
  return { result: cell, text: `Dropped ${cell.id}.` };
}

async function handleCellsUnclaim(root, flags) {
  const cell = await unclaimCell(root, requireFlag(flags, 'id'), ownershipFlags(flags));
  return { result: cell, text: `Unclaimed ${cell.id} — back to open.` };
}

async function handleCellsReopen(root, flags) {
  const cell = await reopenCell(root, requireFlag(flags, 'id'), requireFlag(flags, 'reason'), ownershipFlags(flags));
  return { result: cell, text: `Reopened ${cell.id} — back to open.` };
}

// cells-archive-2: moves a fully-terminal feature's cells out of the hot
// .bee/cells/ scan path into .bee/cells/archive/<feature>/. The active-
// feature guard lives HERE (not in archiveFeature itself, which has no
// access to state.json) — archiving the feature currently in flight would
// hide its own cells from readyCells/claim-next mid-swarm.
async function handleCellsArchive(root, flags) {
  const feature = requireFlag(flags, 'feature');
  const state = readState(root);
  if (state.feature && state.feature === feature) {
    throw new Error(
      `cells archive: feature "${feature}" is the active feature (state.feature) — only a closed/inactive feature can be archived. Switch or clear state.feature first, or archive a different feature.`,
    );
  }
  const result = await archiveFeature(root, feature);
  return {
    result,
    text: `Archived feature "${result.feature}": ${result.moved.length} cell(s) moved (capped=${result.counts.capped} dropped=${result.counts.dropped}).`,
  };
}

async function handleCellsUnarchive(root, flags) {
  const feature = requireFlag(flags, 'feature');
  const moved = await unarchiveFeature(root, feature);
  return {
    result: { feature, moved },
    text: `Unarchived feature "${feature}": ${moved.length} cell(s) restored to .bee/cells/.`,
  };
}

// hardening-4b: setTier is now withStoreLock-wrapped (async).
async function handleCellsTier(root, flags) {
  const cell = await setTier(root, requireFlag(flags, 'id'), String(requireFlag(flags, 'tier')));
  return { result: cell, text: `Cell ${cell.id} tier set to ${cell.tier}.` };
}

function handleCellsJudge(root, flags) {
  const verdict = judgeCell(root, requireFlag(flags, 'id'));
  const text = verdict.hits.length
    ? `FROZEN-JUDGE HITS for ${verdict.id}: ${verdict.hits
        .map((h) => `${h.file} (${h.rule})`)
        .join('; ')} — do not count this cell toward a clean wave; flag it for review (decision 0018).`
    : `Judge intact for ${verdict.id}: no undeclared test/CI/lockfile changes.`;
  return { result: verdict, text };
}

// D2 + GH #27.4 (D-GHF-C): the audited reset door for a cell whose claim
// door is closed by CELL_BUDGET_EXHAUSTED/REPEATED_FAILURE. --reason is
// required at the lib layer (resetCellBudget throws otherwise); --session-id
// follows the same optional/env-resolved convention as every other
// ownership-aware verb, but resetCellBudget never enforces claim ownership
// (a budget-exhausted cell has already been claim-cleared by the refusal
// path — there is no live claim to own). resetCellBudget itself now refuses
// unless the cell is actually budget-blocked, and refuses without an actor
// (--operator here, or its own BEE_AGENT_NAME env fallback when --operator
// is omitted).
async function handleCellsResetBudget(root, flags) {
  const id = requireFlag(flags, 'id');
  const reason = requireFlag(flags, 'reason');
  const sessionId = flags['session-id'] !== undefined ? String(flags['session-id']) : undefined;
  const operator = flags['operator'] !== undefined ? String(flags['operator']) : undefined;
  const cell = await resetCellBudget(root, id, reason, { sessionId, operator });
  return { result: cell, text: `Reset the claim-lifetime budget door for ${cell.id}.` };
}

// D5 (self-correcting-loop): validates the --file payload against schema
// judge-verdict/1 and appends the stamped result to trace.semantic_judge.
// --builder-model/--judge-model presence is what marks that side PINNED —
// the orchestrator only ever supplies a model name from its OWN pinned
// dispatch param (Δ6; rule 13's mandatory transport means there is no code
// path that would hand this flag an unverified guess) — so no separate
// --*-status flag is needed at the CLI boundary; deriveModelIndependence
// itself stays 4-arg/testable directly in test_lib.mjs regardless.
// .bee/logs/dispatch.jsonl is never read here — Δ6: it is corroboration
// only and must never feed a fail-closed guard.
async function handleCellsJudgeRecord(root, flags) {
  const id = requireFlag(flags, 'id');
  const raw = readFileText(String(requireFlag(flags, 'file')), 'judge verdict');
  let verdict;
  try {
    verdict = JSON.parse(raw);
  } catch {
    // Free prose — validateJudgeVerdict rejects this with a typed error
    // (never throws itself); recordJudgeVerdict surfaces that as a refusal.
    verdict = raw;
  }
  const builderModel = flags['builder-model'] !== undefined ? String(flags['builder-model']) : null;
  const judgeModel = flags['judge-model'] !== undefined ? String(flags['judge-model']) : null;
  // hardening-3: recordJudgeVerdict is now async (withStoreLock-wrapped, so
  // it can flip a capped cell back to claimed on a NEEDS_REVISION verdict).
  const cell = await recordJudgeVerdict(root, id, verdict, {
    builderModel,
    builderStatus: builderModel ? PINNED_MODEL_STATUS : null,
    judgeModel,
    judgeStatus: judgeModel ? PINNED_MODEL_STATUS : null,
    ...ownershipFlags(flags),
  });
  const entries = cell.trace.semantic_judge || [];
  const latest = entries[entries.length - 1];
  return {
    result: cell,
    text: `Recorded judge verdict on ${cell.id}: ${latest.verdict} (model_independence=${latest.model_independence}).`,
  };
}

// fresh-session-handoff fsh-11 (D2/D4): typed refusals (NO_APPROVED_WORK,
// CLAIMED, CLAIM_CELL_FAILED, LANE_INVALID/LANE_MISSING/LANE_CORRUPT) surface
// as a thrown Error at the CLI boundary — same convention handleStateHandoffAdopt
// already uses for adoptHandoff's own typed refusals — so the process exits
// non-zero with the reason on stderr rather than a misleadingly "successful" exit.
async function handleCellsClaimNext(root, flags) {
  const worker = requireFlag(flags, 'worker');
  // D3: --session-id keeps working exactly as before; it is now also
  // resolvable from CLAUDE_CODE_SESSION_ID when the flag is omitted.
  // claim-next's own cross-session selection logic still genuinely needs a
  // session id (it resolves the acting session's bound lane), so — unlike
  // the sessionless-claim relaxation in `cells claim` — neither source
  // resolving is still a refusal, just from one of two places now.
  // hardening-1-7-10 D5/1710-10: `root` is threaded through so
  // resolveSessionId's durable single-live-session fallback (claims.mjs) can
  // fire here too — a solo native Codex session has a real session record
  // (from the session-init hook) but no CLAUDE_CODE_SESSION_ID/BEE_SESSION_ID
  // env var, so without `root` this call site refused it every time even
  // though claimCellFile's own fallback (a layer deeper) would have adopted
  // it. Two-or-more fresh live sessions still resolves null here (real
  // ambiguity) and falls through to the unchanged refusal below.
  const sessionId = resolveSessionId({
    flag: flags['session-id'] !== undefined ? String(flags['session-id']) : undefined,
    root,
  });
  if (!sessionId) {
    throw new Error('claim-next: --session-id or CLAUDE_CODE_SESSION_ID env is required.');
  }
  const ttl = flags.ttl !== undefined ? Number.parseInt(String(flags.ttl), 10) : undefined;
  if (flags.ttl !== undefined && (!Number.isFinite(ttl) || ttl <= 0)) {
    throw new Error('--ttl must be a positive integer (seconds).');
  }
  // hardening-4b: claimNextCell now awaits sweepExpiredClaims (sweep-reset)
  // and composes the now-async claimCellCrossSession.
  const result = await claimNextCell(root, { sessionId, worker, ttl });
  if (!result.ok) {
    throw new Error(`claim-next: ${result.code} — ${result.reason}`);
  }
  return { result, text: `Claimed ${result.cell.id} for ${worker} (session ${sessionId}).` };
}

// D1/D4: plan-time only, read-only — loads the feature's declared cells (or
// every cell when --feature is omitted) and hands them to computeSchedule
// unmodified. Feature resolution mirrors handleCellsReady exactly: no
// state.json fallback (plan-checker W3 resolution) — an unscoped call means
// "every cell", not "the current session's feature".
function handleCellsSchedule(root, flags) {
  const cells = listCells(root, { feature: flags.feature ? String(flags.feature) : null });
  const schedule = computeSchedule(cells);
  const lines = [];
  if (schedule.waves.length === 0) {
    lines.push('No schedulable cells.');
  } else {
    schedule.waves.forEach((wave, index) => {
      lines.push(`Wave ${index + 1}: ${wave.join(', ')}`);
    });
  }
  const { cycles, unsatisfiable_deps: unsatisfiableDeps, empty_files: emptyFiles } = schedule.diagnostics;
  if (cycles.length > 0) {
    lines.push('Cycles:');
    for (const cycle of cycles) lines.push(`- ${cycle.join(' -> ')}`);
  }
  if (unsatisfiableDeps.length > 0) {
    lines.push('Unsatisfiable deps:');
    for (const row of unsatisfiableDeps) lines.push(`- ${row.cell} -> ${row.dep} (${row.reason})`);
  }
  if (emptyFiles.length > 0) {
    lines.push(`Empty files: ${emptyFiles.join(', ')}`);
  }
  return { result: schedule, text: lines.join('\n') };
}

async function handleReservationsReserve(root, flags) {
  const ttl = flags.ttl !== undefined ? Number.parseInt(String(flags.ttl), 10) : undefined;
  if (flags.ttl !== undefined && (!Number.isFinite(ttl) || ttl <= 0)) {
    throw new Error('--ttl must be a positive integer (seconds).');
  }
  const requestedPath = requireFlag(flags, 'path');
  const topology = resolveHoldTopology(root);

  const doReserve = () =>
    reserve(root, {
      agent: requireFlag(flags, 'agent'),
      cell: requireFlag(flags, 'cell'),
      path: requestedPath,
      ...(ttl !== undefined ? { ttl } : {}),
      ...(flags.session ? { session: String(flags.session) } : {}),
    });

  // hardening-1-7-10 (D3): when a topology exists, the foreign-hold check,
  // the local reserve, and the mirror-insert all run as ONE atomic section
  // under withHoldsLock(topology.mainRoot, ...) — the shared cross-worktree
  // lock outermost, reserve()'s own local 'reservations' lock taken inside
  // it (a DIFFERENT lock name/often a different root, so no self-deadlock).
  // Before D3 this was check-then-act: an UNLOCKED findForeignHolds read,
  // then reserve(), then a separately-locked mirrorHold — three independent
  // critical sections with real gaps between them, so two checkouts racing
  // the SAME path could both pass the foreign-hold check before either had
  // mirrored, and both land an active grant (test_worktree_holds_race.mjs's
  // same-path scenario (c) demonstrates the double-grant against that old
  // shape; scenario (d) proves this atomic section yields exactly one
  // winner). reserve() only performs fs reads/writes, never spawns a child
  // process, so it is safe to run while holding this lock (never hold the
  // shared lock across a child-process spawn).
  let sectionResult;
  if (topology) {
    sectionResult = await withHoldsLock(topology.mainRoot, async () => {
      const foreignHolds = findForeignHolds(topology.mainRoot, topology.holder, [requestedPath]);
      if (foreignHolds.length > 0) {
        return { refusal: foreignHolds[0] };
      }
      const reserveResult = await doReserve();
      if (reserveResult.ok) {
        insertHold(topology.mainRoot, {
          path: reserveResult.reservation.path,
          holder: topology.holder,
          session: reserveResult.reservation.session || null,
          cell: reserveResult.reservation.cell,
          ttl: reserveResult.reservation.ttl_seconds,
        });
      }
      return { reserveResult };
    });
  } else {
    sectionResult = { reserveResult: await doReserve() };
  }

  if (sectionResult.refusal) {
    const hold = sectionResult.refusal;
    const result = {
      ok: false,
      code: 'FOREIGN_HOLD',
      holder: hold.holder,
      feature: hold.feature,
      cell: hold.cell,
      path: hold.path,
      expires: holdForeignExpiry(hold),
    };
    const text =
      `bee cross-worktree hold: "${hold.path}" is held by checkout "${hold.holder}" ` +
      `(feature ${hold.feature || 'unknown'}, cell ${hold.cell || 'unknown'}), ${holdForeignExpiry(hold)}. ` +
      'Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.';
    return { result, text, exitCode: 1 };
  }

  const result = sectionResult.reserveResult;
  const text = result.ok
    ? `Reserved "${result.reservation.path}" for ${result.reservation.agent} (cell ${result.reservation.cell}, ttl ${result.reservation.ttl_seconds}s).`
    : [
        'Reservation CONFLICT — return [BLOCKED] to the orchestrator:',
        ...result.conflicts.map((c) => `- ${c.agent} holds "${c.path}" (cell ${c.cell})`),
      ].join('\n');

  return { result, text, exitCode: result.ok ? 0 : 1 };
}

async function handleReservationsRelease(root, flags) {
  const agent = requireFlag(flags, 'agent');
  const cell = flags.cell ? String(flags.cell) : null;

  // xwh-2 hardening (found live, post-cap): a mirrored hold has NO agent
  // field — worktree-holds.mjs's shape is only {path, holder, feature,
  // session, cell, ttl_seconds, ...} — so in an ordinary checkout every
  // agent's mirrors share the SAME holder ('main'). Calling
  // releaseHolds({holder, cell: null}) whenever --cell is omitted (a normal,
  // common call shape: "release everything I hold") would release EVERY
  // mirrored hold under that holder, including ones mirrored by a
  // COMPLETELY DIFFERENT agent's cell — confirmed live in this session: an
  // agent-wide `reservations release --agent exec-xwh2` (no --cell) wrongly
  // cleared 8 of a concurrent agent's still-active mirrored holds. Fix:
  // never pass the raw --cell flag straight through. Instead, read this
  // agent's own ACTIVE local rows first (before release() marks them),
  // derive the exact distinct cell id(s) they belong to, and scope the
  // ledger release to precisely those cells — never a blanket null, even
  // when the local release itself IS agent-wide (cell:null there is safe:
  // reservations.json rows already carry `agent`, so it can never touch
  // another agent's row; the ledger has no such field, so it needs the
  // narrower, derived scope instead).
  const affectedCells = [
    ...new Set(
      listReservations(root, { activeOnly: true })
        .filter((r) => r.agent === agent && (!cell || r.cell === cell))
        .map((r) => r.cell)
        .filter(Boolean),
    ),
  ];

  const result = await release(root, { agent, cell });

  // xwh-2: also clear this checkout's mirrored entries in the shared ledger
  // — same topology as the reserve side, so a release never leaves a stale
  // mirrored hold behind for a checkout that only ever mirrored via reserve.
  const topology = resolveHoldTopology(root);
  let holdsReleased = 0;
  if (topology) {
    for (const affectedCell of affectedCells) {
      const holdsResult = await releaseHolds(topology.mainRoot, { holder: topology.holder, cell: affectedCell });
      holdsReleased += holdsResult.released;
    }
  }

  return {
    result: { ...result, holds_released: holdsReleased },
    text: `Released ${result.released} reservation(s)${holdsReleased ? ` and ${holdsReleased} cross-worktree hold(s)` : ''}.`,
  };
}

function handleReservationsList(root, flags) {
  const reservations = listReservations(root, { activeOnly: flags['active-only'] === true });

  // xwh-2: also surface active cross-worktree ledger entries. Reuses
  // findForeignHolds (worktree-holds.mjs's only read query) with a synthetic
  // acting holder that can never match a real one and a bare '*' path
  // (pathsOverlap's own documented "bare '*' covers everything" rule) rather
  // than adding a second, near-duplicate "list all" export to that module —
  // this cell's file list does not include worktree-holds.mjs. A missing/no
  // ledger reads as an empty list, same fail-open posture as reservations.
  const mainRoot = resolveMainRoot(root);
  const crossWorktree = findForeignHolds(mainRoot, LIST_ALL_HOLDS_SENTINEL, ['*']);

  const lines = [];
  lines.push(
    reservations.length
      ? reservations
          .map(
            (r) =>
              `${r.agent} | cell ${r.cell} | ${r.path} | reserved ${r.reserved_at} | ${r.released_at ? `released ${r.released_at}` : 'active/expired by TTL'}`,
          )
          .join('\n')
      : 'No reservations.',
  );
  if (crossWorktree.length) {
    lines.push('cross_worktree:');
    lines.push(
      ...crossWorktree.map(
        (h) => `${h.holder} | cell ${h.cell || 'unknown'} | ${h.path} | mirrored ${h.mirrored_at} | ${holdForeignExpiry(h)}`,
      ),
    );
  }
  return { result: { reservations, cross_worktree: crossWorktree }, text: lines.join('\n') };
}

async function handleReservationsSweep(root) {
  const released = await sweepExpired(root);

  // xwh-2: also prune TTL-expired entries in the shared cross-worktree
  // ledger — sweepExpiredHolds resolves its own empty/missing ledger, so
  // this is safe to call unconditionally (no topology gate needed: sweeping
  // an empty or absent ledger is a no-op, mirroring sweepExpired's own
  // posture for reservations.json).
  const mainRoot = resolveMainRoot(root);
  const holdsReleased = await sweepExpiredHolds(mainRoot);

  return {
    result: { released, holds_released: holdsReleased },
    text: `Swept ${released} expired reservation(s) and ${holdsReleased} expired cross-worktree hold(s).`,
  };
}

function handleDecisionsLog(root, flags) {
  const confidence =
    flags.confidence !== undefined ? Number.parseInt(String(flags.confidence), 10) : null;
  if (flags.confidence !== undefined && !Number.isFinite(confidence)) {
    throw new Error('--confidence must be an integer.');
  }
  const event = logDecision(root, {
    decision: requireFlag(flags, 'decision'),
    rationale: requireFlag(flags, 'rationale'),
    alternatives: flags.alternatives ? String(flags.alternatives) : null,
    scope: flags.scope ? String(flags.scope) : 'repo',
    source: flags.source ? String(flags.source) : 'user',
    confidence,
    tags: flags.tags !== undefined ? splitList(flags.tags) : undefined,
  });
  // decision-propagation dp-6 (CONTEXT D7b): bootstrap-safe warn-only path —
  // no docs/decisions/taxonomy.json means logDecision never refused a
  // zero-tag event above; surface that as a human-readable warning (JSON
  // output stays data-only, see emit()'s result-vs-text split).
  const warning =
    !taxonomyFileExists(root) && !(Array.isArray(event.tags) && event.tags.length)
      ? '\nWarning: no taxonomy.json found — this decision was logged without tags. Create docs/decisions/taxonomy.json to require classification going forward.'
      : '';
  return { result: event, text: `Logged decision ${event.id}.${warning}` };
}

// decision-propagation dp-2 (CONTEXT D2): capture stub creation lives here,
// not in lib/decisions.mjs — capture.mjs already imports the secret/
// injection pattern constants FROM decisions.mjs, so having decisions.mjs
// import addCaptureStub back from capture.mjs would create a module cycle.
// The lock doctrine (sweep computed before the append, written once) is
// still fully satisfied inside supersedeDecision itself; this is purely a
// downstream side effect using the sweep result the returned event already
// carries.
function handleDecisionsSupersede(root, flags) {
  const event = supersedeDecision(root, {
    supersedes: requireFlag(flags, 'id'),
    decision: requireFlag(flags, 'decision'),
    rationale: requireFlag(flags, 'rationale'),
    tags: flags.tags !== undefined ? splitList(flags.tags) : undefined,
    scope: flags.scope !== undefined ? String(flags.scope) : undefined,
  });

  const hits = event.sweep?.files || [];
  for (const hit of hits) {
    addCaptureStub(root, {
      outcome: `${hit.file}:${hit.line} still cites superseded decision ${event.supersedes} — reconcile against replacement ${event.id}.`,
      dids: [event.supersedes, event.id],
      files: [hit.file],
      source: 'supersede-sweep',
    });
  }

  const header = `Superseded ${event.supersedes} with ${event.id}.`;
  const sweepLines = hits.length
    ? [
        `Propagation sweep: ${hits.length} citation(s) found under docs/** — a capture stub was queued for each.`,
        ...hits.map((hit) => `  ${hit.file}:${hit.line}  ${hit.excerpt}`),
      ]
    : ['Propagation sweep: no citations found under docs/**.'];
  return { result: event, text: [header, ...sweepLines].join('\n') };
}

function handleDecisionsRedact(root, flags) {
  const event = redactDecision(root, {
    redacts: requireFlag(flags, 'id'),
    reason: requireFlag(flags, 'reason'),
  });
  return { result: event, text: `Redacted ${event.redacts}.` };
}

// decision-propagation dp-1 (CONTEXT D4a, GH #32): structured recall filters
// shared by `decisions search` and `decisions active` (the latter a
// deliberate sibling extension beyond D4a's letter — logged as a decision at
// implementation time). --scope/--area is one filter (--area is an exact
// alias, never a second dimension — no new `area` field, fresh-eyes P2).
// Every filter is exact-match case-insensitive except --since (inclusive
// lower bound on event.date) and --text. A legacy event with no `tags`
// array never matches a --tag filter (it has nothing to match), but is
// untouched by every other filter — so it stays reachable via
// --text/--scope/--since exactly as before.
//
// decision-propagation dp-6 (CONTEXT D7d): --untagged keeps exactly the
// events with no tags AFTER overlay (the events already passed in here
// carry their dp-5 overlay applied — see activeDecisions) — composable with
// every other filter, including --all upstream.
//
// decision-propagation dp-6 (CONTEXT D8b): --text upgrades from a single
// substring match to multi-term: whitespace-split, case-insensitive, OR
// across terms, matched over decision/rationale/alternatives AND (now)
// tags — a single term still matches everything the old substring check
// matched (decision/rationale/alternatives are still searched), so
// single-term results are a strict superset of the pre-dp-6 behavior.
// Matches are ranked by deterministic term-hit count descending; the sort
// is STABLE (spec-guaranteed since ES2019), so it preserves the incoming
// newest-first order (activeDecisions' own date-desc, index-tiebroken
// ordering) as the secondary key — "hit count desc, then date desc" falls
// out of "stable-sort the already-date-ordered list by hit count" with no
// separate date comparison, no wall-clock read, and no dependence on Map or
// object iteration order.
// sqs-b1 (D 5ca69717): --cell/--feature are word-boundary TEXT matches over
// decision/rationale/alternatives — a decide event has no structural cell/
// feature field (gather Q1), cell ids and feature slugs only ever appear in
// free-text prose. A plain \b...\b is not enough for hyphenated ids/slugs:
// "-" is itself a non-word character, so \b treats a following hyphen as a
// boundary and would let "si-1" falsely match inside "si-1-extra", or
// "billing-export" falsely match inside "billing-export-v2". The lookaround
// form below explicitly excludes "-" (alongside \w) from both edges, so
// "si-1" matches "si-1" and excludes both "si-10" and "si-1-extra".
function matchesWholeToken(haystacks, token) {
  const pattern = new RegExp(`(?<![\\w-])${escapeRegExp(token)}(?![\\w-])`, 'i');
  return haystacks.some((h) => pattern.test(h));
}

function filterDecisionEvents(decisions, { text, tag, scope, since, untagged, cell, feature } = {}) {
  let result = decisions;
  if (untagged) {
    result = result.filter((event) => !(Array.isArray(event.tags) && event.tags.length > 0));
  }
  if (cell) {
    const token = String(cell);
    result = result.filter((event) => {
      const haystacks = [event.decision, event.rationale, event.alternatives]
        .filter((v) => v !== null && v !== undefined && v !== '')
        .map((v) => String(v));
      return matchesWholeToken(haystacks, token);
    });
  }
  if (feature) {
    const token = String(feature);
    result = result.filter((event) => {
      const haystacks = [event.decision, event.rationale, event.alternatives]
        .filter((v) => v !== null && v !== undefined && v !== '')
        .map((v) => String(v));
      return matchesWholeToken(haystacks, token);
    });
  }
  if (tag) {
    const needle = tag.toLowerCase();
    result = result.filter(
      (event) => Array.isArray(event.tags) && event.tags.some((t) => String(t).toLowerCase() === needle),
    );
  }
  if (scope) {
    const needle = scope.toLowerCase();
    result = result.filter((event) => typeof event.scope === 'string' && event.scope.toLowerCase() === needle);
  }
  if (since) {
    const sinceMs = Date.parse(since);
    result = result.filter((event) => {
      const eventMs = Date.parse(event.date);
      return Number.isFinite(eventMs) && eventMs >= sinceMs;
    });
  }
  if (text) {
    const terms = String(text).toLowerCase().split(/\s+/).filter(Boolean);
    const scored = result
      .map((event) => {
        const haystacks = [event.decision, event.rationale, event.alternatives, ...(Array.isArray(event.tags) ? event.tags : [])]
          .filter((v) => v !== null && v !== undefined && v !== '')
          .map((v) => String(v).toLowerCase());
        const hitCount = terms.reduce((count, term) => (haystacks.some((h) => h.includes(term)) ? count + 1 : count), 0);
        return { event, hitCount };
      })
      .filter(({ hitCount }) => hitCount > 0);
    scored.sort((a, b) => b.hitCount - a.hitCount);
    result = scored.map(({ event }) => event);
  }
  return result;
}

// --scope/--area share one filter value; --area is an exact alias (D4a: no
// second navigation dimension). An explicit --scope wins if both are somehow
// passed (matches how most other dual-flag call sites in this file resolve
// ties — first-named flag wins).
function resolveScopeFilter(flags) {
  if (flags.scope !== undefined) return String(flags.scope);
  if (flags.area !== undefined) return String(flags.area);
  return null;
}

function resolveSinceFilter(flags) {
  if (flags.since === undefined) return null;
  const since = String(flags.since);
  if (!Number.isFinite(Date.parse(since))) {
    throw new Error(`--since must be a valid ISO date, got ${JSON.stringify(since)}.`);
  }
  return since;
}

function handleDecisionsActive(root, flags) {
  const recent =
    flags.recent !== undefined ? Number.parseInt(String(flags.recent), 10) : null;
  if (flags.recent !== undefined && (!Number.isFinite(recent) || recent <= 0)) {
    throw new Error('--recent must be a positive integer.');
  }
  const tag = flags.tag !== undefined ? String(flags.tag) : null;
  const scope = resolveScopeFilter(flags);
  const since = resolveSinceFilter(flags);
  const all = flags.all !== undefined; // decision-propagation dp-3 (D4c): union read including .bee/decisions-archive.jsonl
  const untagged = flags.untagged !== undefined; // decision-propagation dp-6 (D7d): events with no tags after overlay
  const cell = flags.cell !== undefined ? String(flags.cell) : null; // sqs-b1 (D 5ca69717): word-boundary text match, no structural field
  const feature = flags.feature !== undefined ? String(flags.feature) : null; // sqs-b1 (D 5ca69717): word-boundary text match, no structural field
  let decisions = filterDecisionEvents(activeDecisions(root, { all }), { tag, scope, since, untagged, cell, feature });
  if (recent != null) decisions = decisions.slice(0, recent);
  const text = decisions.length ? decisions.map(formatDecision).join('\n') : 'No active decisions.';
  return { result: { decisions }, text };
}

function handleDecisionsSearch(root, flags) {
  const text = flags.text !== undefined ? String(flags.text) : null;
  const tag = flags.tag !== undefined ? String(flags.tag) : null;
  const scope = resolveScopeFilter(flags);
  const since = resolveSinceFilter(flags);
  const all = flags.all !== undefined; // decision-propagation dp-3 (D4c): union read including .bee/decisions-archive.jsonl
  const untagged = flags.untagged !== undefined; // decision-propagation dp-6 (D7d): events with no tags after overlay
  const cell = flags.cell !== undefined ? String(flags.cell) : null; // sqs-b1 (D 5ca69717): word-boundary text match, no structural field
  const feature = flags.feature !== undefined ? String(flags.feature) : null; // sqs-b1 (D 5ca69717): word-boundary text match, no structural field
  if (!text && !tag && !scope && !since && !untagged && !cell && !feature) {
    throw new Error(
      'decisions search requires --text, or at least one structured filter (--tag/--scope/--area/--since/--untagged/--cell/--feature).',
    );
  }
  const decisions = filterDecisionEvents(activeDecisions(root, { all }), { text, tag, scope, since, untagged, cell, feature });
  const resultText = decisions.length
    ? decisions.map(formatDecision).join('\n')
    : 'No active decisions matching the given filters.';
  return { result: { decisions }, text: resultText };
}

// decision-propagation dp-3 (CONTEXT D4c): moves superseded/redacted events
// (always) plus decide events strictly older than the explicit --before to
// .bee/decisions-archive.jsonl. All refusal/crash-safety logic lives in
// lib/decisions.mjs's archiveDecisions — this is presentation only.
function handleDecisionsArchive(root, flags) {
  const before = flags.before !== undefined ? String(flags.before) : undefined;
  const result = archiveDecisions(root, { before });
  return {
    result,
    text: `Archived ${result.archived.length} decision(s) to .bee/decisions-archive.jsonl (kept ${result.kept} active, cutoff ${result.before}).`,
  };
}

// decision-propagation dp-5 (CONTEXT D7c): `decisions tag --target <id|
// short8> --tags a,b [--scope s]` appends a retro-tag event; `--stdin`
// accepts a JSON array of {target, tags, scope?} for a batch. All
// validation (target resolution + tag shape) and the all-or-nothing
// atomicity live in lib/decisions.mjs's tagDecisionsBatch — this handler is
// presentation only, mirroring handleCellsAdd's --stdin-array-is-a-batch
// shape.
function tagEventSummary(event) {
  const scopeSuffix = event.scope ? ` scope=${event.scope}` : '';
  return `Tagged ${event.target} with [${event.tags.join(', ')}]${scopeSuffix}.`;
}

function handleDecisionsTag(root, flags) {
  if (flags.stdin === true) {
    const text = fs.readFileSync(0, 'utf8');
    let entries;
    try {
      entries = JSON.parse(text);
    } catch {
      throw new Error('decisions tag --stdin: input is not valid JSON.');
    }
    if (!Array.isArray(entries)) {
      throw new Error('decisions tag --stdin: input must be a JSON array of {target, tags, scope?}.');
    }
    const events = tagDecisionsBatch(root, entries);
    return { result: events, text: events.map(tagEventSummary).join('\n') };
  }
  const event = tagDecision(root, {
    target: requireFlag(flags, 'target'),
    tags: splitList(requireFlag(flags, 'tags')),
    scope: flags.scope !== undefined ? String(flags.scope) : undefined,
  });
  return { result: event, text: tagEventSummary(event) };
}

// decision-propagation dp-4 (CONTEXT D4b/D6, overlay-aware per D7/D8):
// `decisions render` writes docs/decisions/index.md; `--check` is read-only
// and refuses (non-zero exit) on drift instead of writing — all computation
// (grouping, overlay, byte-diff) lives in lib/decisions.mjs, this handler is
// presentation + the --check-refuses-loudly policy only. `--all` reaches the
// archive, matching search/active's own flag (D4c).
function handleDecisionsRender(root, flags) {
  const all = flags.all !== undefined;
  if (flags.check !== undefined) {
    const { drift, path: relPath } = decisionIndexDrift(root, { all });
    if (drift) {
      throw new Error(
        `decisions render --check: ${relPath} is out of date — run \`bee decisions render\` to regenerate (never hand-edit it).`,
      );
    }
    return { result: { drift: false, path: relPath }, text: `${relPath} is up to date.` };
  }
  const result = renderDecisionIndex(root, { all });
  return { result, text: `Wrote ${result.path} (${result.count} decision(s)).` };
}

// ─── state: full port of bee_state.mjs's verb logic (dispatcher-unify du-1).
// Reuses lib/state.mjs's read/write/validation exports exactly as bee_state.mjs
// did — no logic change in lib/state.mjs. Every stdout/stderr byte and exit
// code stays as the existing test_lib bee_state checks pin them (DB3). ────────

// Dispatch transients written by bee-swarming: <cell-id>.prompt.md / .out*.log
// / .result.md|json. Files outside this suffix set are never prune candidates.
const WORKER_TRANSIENT_SUFFIX = /\.(prompt\.md|result\.md|result\.json|out\d*\.log|log)$/;

function splitList(raw) {
  return String(raw)
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
}

// A safety flag must never ride along with a real mutation: every state verb
// except "worker prune" mutates state, so --dry-run there is a hard error, not
// an ignored no-op that mutates anyway (bee_state.mjs:405-409, a cross-verb rule).
function rejectDryRun(flags) {
  if (flags['dry-run'] !== undefined) {
    throw new Error(
      '--dry-run is only supported by "worker prune" — refusing to run a mutating verb with a dry-run flag.',
    );
  }
}

// Optional --lane target resolution (fresh-session-handoff fsh-4, D2/D4),
// shared by state.set/gate/scribing-run. `--lane` bare (no value) is a
// malformed call, not "no lane requested" — refuse rather than silently
// falling back to the default record. A named lane that is missing or
// corrupt refuses loudly here, before any read/write of the target record —
// readLaneStrict already throws loud on corrupt; missing reads as null
// (creation is start-feature's job, never an implicit side effect of set/
// gate/scribing-run), so that case is refused explicitly below.
function optionalLaneFlag(flags, verb) {
  if (flags.lane === undefined) return null;
  if (flags.lane === true || flags.lane === '') {
    throw new Error(`${verb}: --lane requires a value (the lane's feature name).`);
  }
  return String(flags.lane);
}

// i54-closeout-7 (D7) — write-path lane selection, shared by the four
// mutation verbs (state set/gate/scribing-run/advisor-ref record). Pairs the
// existing --lane parse with the --no-lane escape hatch: a session bound to a
// lane targets that lane by default (resolveMutationTarget below), so
// --no-lane is the explicit way for a bound session to address the DEFAULT
// record. Combining the two is contradictory — refuse, never pick one.
function mutationLaneSelector(flags, verb) {
  const laneFeature = optionalLaneFlag(flags, verb);
  const noLane = flags['no-lane'] !== undefined;
  if (noLane && laneFeature) {
    throw new Error(
      `${verb}: --no-lane cannot be combined with --lane — --no-lane forces the default record, --lane names a lane record. Pick one.`,
    );
  }
  return { laneFeature, noLane };
}

// i54-closeout-7 (D7) — writes resolve like reads. Resolution order, symmetric
// with the read path's resolvePipeline (state.mjs): explicit --lane always
// wins > the calling session's bound lane (identity self-resolved at operation
// moment via resolveSessionId — B22, the exact primitive claims/reservations
// and buildLaneSummary already use) > the default state.json. --no-lane skips
// the session step and forces the default record. A bound session whose lane
// record is missing or corrupt refuses loudly with zero writes (B12/B13 —
// the lane never borrows the default record's authority, and resolution never
// silently guesses back to the default; fresh-session-handoff D2). An unbound
// session (no resolvable identity, no session record, or no lane binding)
// targets the default record exactly as before this cell.
//
// multisession-native-7/10 (C5 — "no consumer writes through a projection"):
// neither branch below calls writeLane/writeState directly anymore. Both
// route through a write-through-projection helper — writeLaneRecordThrough
// Projection (lane) or writeStateRecordThroughProjection (default, msn-10) —
// which updates the live workflow record naming the target's feature and
// rebuilds its projection from it (state-projection.mjs, "record wins").
// Each captures its target FEATURE NAME here, at resolution time, BEFORE any
// caller mutation — never re-derived from the post-mutation record — mirroring
// how `laneFeature` below is already fixed at resolution time for the whole
// call.
function resolveMutationTarget(root, laneFeature, verb, { noLane = false } = {}) {
  const defaultTarget = () => {
    const record = readStateStrict(root);
    // Captured NOW, before the caller mutates `record` in place — the ONE
    // exception is a `--feature` SWAP (state set --feature X on the default
    // record), whose whole point is to change this value out from under
    // itself; writeStateRecordThroughProjection's own comment documents why
    // that case is deliberately excluded from workflow-routing.
    const targetFeature = record.feature || null;
    return {
      record,
      write: (updated, gateStamp) => writeStateRecordThroughProjection(root, targetFeature, updated, gateStamp),
      source: 'default',
      lane: null,
    };
  };
  if (laneFeature) {
    const record = readLaneStrict(root, laneFeature);
    if (!record) {
      throw new Error(
        `${verb}: refused — lane "${laneFeature}" does not exist (no .bee/lanes/${laneFeature}.json). FIX: start it first ("state start-feature --feature ${laneFeature} --as-lane"), then retry.`,
      );
    }
    return {
      record,
      write: (updated, gateStamp) => writeLaneRecordThroughProjection(root, laneFeature, updated, gateStamp),
      source: 'lane',
      lane: laneFeature,
    };
  }
  if (noLane) return defaultTarget();
  const sessionId = resolveSessionId({ root });
  const session = sessionId ? readSession(root, sessionId) : null;
  const bound = session && typeof session.lane === 'string' ? session.lane.trim() : '';
  if (!bound) return defaultTarget();
  // Bound session: the lane IS the target. readLaneStrict throws loud on a
  // corrupt/unreadable record; a missing record is refused explicitly here —
  // never a silent fall-through to the default record.
  const record = readLaneStrict(root, bound);
  if (!record) {
    throw new Error(
      `${verb}: refused — calling session "${sessionId}" is bound to lane "${bound}" but no .bee/lanes/${bound}.json exists; resolution never guesses back to the default record. FIX: start the lane ("state start-feature --feature ${bound} --as-lane"), unbind the session, or pass --no-lane to target the default record explicitly.`,
    );
  }
  return {
    record,
    write: (updated, gateStamp) => writeLaneRecordThroughProjection(root, bound, updated, gateStamp),
    source: 'lane',
    lane: bound,
  };
}

// writeLaneRecordThroughProjection — the C5-scoped lane write path (see
// resolveMutationTarget's own comment above). `updated` is the CALLER's
// already-fully-mutated lane-shaped record (every existing caller mutates
// `record` in place field-by-field before calling `write`); this function's
// job is to persist it correctly rather than to decide what changed.
//
// Live (non-closed) workflow record found for this feature: update it with
// the equivalent D1 fields, then rebuild the lane projection from the
// now-updated record — the file callers read back is always
// projection-builder output, never a hand-written copy.
//
// `gateStamp` (multisession-native-9, D7/C2 — optional, `{ name, approvedForPlanRev }`):
// ONLY handleStateGate passes this, and ONLY for the 'execution' gate (D7
// default: a plan_rev bump invalidates only the execution gate — context/
// shape/review are never stamped with a real rev, so mergeGates in
// workflow-store.mjs preserves their `approved_for_plan_rev` (always null in
// practice) untouched across every other write here). When present, the
// named gate's patch carries an explicit `approved_for_plan_rev` (the
// workflow's CURRENT plan_rev at approval time, or null on revocation);
// every other gate's patch carries `approved` only, so mergeGates preserves
// its existing `approved_for_plan_rev` field exactly as before this write —
// this is what keeps a plain `state set`/`scribing-run` write (which also
// round-trips `updated.approved_gates` through this same function, unchanged
// booleans and all) from ever silently re-stamping or clearing a rev it
// wasn't asked to touch.
//
// C1 fallback: no live workflow record names this feature (zero workflow
// records exist anywhere, or this lane's workflow was never seeded/created —
// should not happen once msn-6's seed has run, but never assumed here)
// falls back to writeLane directly, unchanged from before this cell.
//
// multisession-native-10: uses updateWorkflowAssumingLock, NOT updateWorkflow
// — this function only ever runs as the `write` callback inside
// withMutationLock's closure below, which already holds `workflow:<id>` for
// this exact workflow for the WHOLE mutation body. Calling the self-locking
// updateWorkflow here would try to acquire that SAME lock a second time from
// the SAME process and deadlock (see updateWorkflowAssumingLock's own
// comment in workflow-store.mjs). The projection rebuild that follows
// (rebuildLaneProjection -> writeLane) takes no lock of its own here either
// — it is protected by that same outer `workflow:<id>` hold, same as the
// updateWorkflowAssumingLock call above it.
async function writeLaneRecordThroughProjection(root, laneFeature, updated, gateStamp = null) {
  const wf = listWorkflows(root).workflows.find((w) => w.feature === laneFeature && w.status !== 'closed');
  if (!wf) {
    writeLane(root, updated);
    return updated;
  }
  updateWorkflowAssumingLock(root, wf.id, {
    phase: updated.phase,
    mode: updated.mode == null ? null : String(updated.mode),
    summary: updated.summary,
    next_action: updated.next_action,
    gates: Object.fromEntries(
      GATE_NAMES.map((name) => {
        const entry = { approved: Boolean(updated.approved_gates && updated.approved_gates[name] === true) };
        if (gateStamp && gateStamp.name === name) {
          entry.approved_for_plan_rev = gateStamp.approvedForPlanRev;
        }
        return [name, entry];
      }),
    ),
  });
  const rebuilt = rebuildLaneProjection(root, laneFeature);
  return rebuilt.lane;
}

// writeStateRecordThroughProjection — the DEFAULT-record sibling of
// writeLaneRecordThroughProjection above (multisession-native-10, closing
// the C5 residual seam). Same shape, same updateWorkflowAssumingLock/no-
// self-lock discipline (this also only ever runs inside withMutationLock's
// closure), same C1 fallback (no live workflow names `targetFeature` ->
// writeState directly, unchanged from every verb's pre-msn-10 behavior).
//
// The ONE default-only wrinkle: `targetFeature` is captured by
// resolveMutationTarget BEFORE any caller mutation (mirrors `laneFeature`,
// which can never change mid-call because a lane's feature is its filename
// identity). The default record's `feature` CAN change mid-call — `state set
// --feature NEW` (a swap) — and a workflow record's own `feature` field is
// immutable identity (updateWorkflow/updateWorkflowAssumingLock never let a
// patch touch it). Routing a swap through the OLD feature's workflow would
// silently update the WRONG record's phase/summary/etc under the NEW
// feature's name change; routing it through a (usually nonexistent) NEW
// feature's workflow would misattribute state that workflow never asked
// for. Neither is safe, so a swap (`updated.feature !== targetFeature`) is
// deliberately excluded from workflow-routing altogether and falls to the
// SAME direct writeState the C1 fallback uses — byte-identical to every
// verb's behavior before this cell, for this one narrow case only.
async function writeStateRecordThroughProjection(root, targetFeature, updated, gateStamp = null) {
  const routable = targetFeature && updated.feature === targetFeature;
  const wf = routable ? listWorkflows(root).workflows.find((w) => w.feature === targetFeature && w.status !== 'closed') : null;
  if (!wf) {
    writeState(root, updated);
    return updated;
  }
  updateWorkflowAssumingLock(root, wf.id, {
    phase: updated.phase,
    mode: updated.mode == null ? null : String(updated.mode),
    summary: updated.summary,
    next_action: updated.next_action,
    gates: Object.fromEntries(
      GATE_NAMES.map((name) => {
        const entry = { approved: Boolean(updated.approved_gates && updated.approved_gates[name] === true) };
        if (gateStamp && gateStamp.name === name) {
          entry.approved_for_plan_rev = gateStamp.approvedForPlanRev;
        }
        return [name, entry];
      }),
    ),
  });
  // multisession-native-10 (extends msn-7's idle-gate, state-projection.mjs):
  // rebuildStateProjection now adopts a LIVE default feature's own workflow
  // record even while non-idle, precisely because this call just kept that
  // record in sync — see rebuildStateProjection's own comment for the full
  // two-branch contract this depends on.
  const rebuilt = rebuildStateProjection(root);
  return rebuilt.state;
}

// resolveMutationLockFeature — a read-only PEEK at which feature's workflow
// lock (if any) should protect a mutation, computed BEFORE any lock is
// acquired. Mirrors resolveMutationTarget's own resolution order (explicit
// --lane > session-bound lane > default) but never reads/validates the
// target record itself — only the feature NAME, via a fail-open read
// (readState/readSession, never the *Strict throwing siblings): anything
// uncertain here — a corrupt or missing default record, a session with no
// binding — degrades to null, which withMutationLock below treats as "use
// the legacy 'state' lock", exactly today's behavior. The real,
// authoritative read (and any throw it can produce) still happens inside
// resolveMutationTarget, under whichever lock this function's answer
// selected.
//
// C4 (advisor consult slice 2, binding — sessions and workflow:<id> are
// NEVER held together): this function reads a session record via
// readSession, but it NEVER acquires the 'sessions' store lock to do so —
// readSession is a plain, lock-free file read (see claims.mjs). The only
// lock this whole seam ever holds at once is EITHER 'sessions' (inside
// bindSessionLane/unbindSessionLane, claims.mjs — see that module's own
// D10a comment) OR 'workflow:<id>' (inside withMutationLock below) — never
// both, because binding a session to a lane and mutating a workflow's
// record are always two separate calls/transactions, never nested inside
// one another's lock hold.
function resolveMutationLockFeature(root, laneFeature, noLane) {
  if (laneFeature) return laneFeature;
  if (noLane) return null;
  const sessionId = resolveSessionId({ root });
  const session = sessionId ? readSession(root, sessionId) : null;
  const bound = session && typeof session.lane === 'string' ? session.lane.trim() : '';
  if (bound) return bound;
  const state = readState(root); // fail-open peek; readStateStrict's throw still happens inside resolveMutationTarget
  return (state && state.feature) || null;
}

// withMutationLock — C4-safe lock routing for the four record-mutating state
// verbs (set/gate/scribing-run/advisor-ref record; multisession-native-10).
// Replaces the blanket `withStoreLock(root, 'state', fn)` every one of these
// verbs used through msn-9: a workflow-record-routed target (lane, OR
// default once msn-10 keeps it in sync) now locks on `workflow:<id>` ONLY —
// the SAME per-workflow lock workflow-store.mjs's own mutating verbs use —
// so two sessions mutating two DIFFERENT workflows never contend on a
// shared lock (invariants 1/2, CONTEXT.md D9). A target with no live
// workflow record (C1 fallback: a legacy lane/default record, or a
// --feature swap that intentionally stays off the workflow-routing model —
// see writeStateRecordThroughProjection) keeps today's single 'state' lock,
// byte-identical to every verb's pre-msn-10 behavior.
//
// fn itself still calls resolveMutationTarget (the real, strict read +
// validation + mutation), same as every caller did before this cell — this
// wrapper only changes WHICH lock protects that call.
async function withMutationLock(root, laneFeature, noLane, fn) {
  const feature = resolveMutationLockFeature(root, laneFeature, noLane);
  const wf = feature ? listWorkflows(root).workflows.find((w) => w.feature === feature && w.status !== 'closed') : null;
  if (wf) {
    return withWorkflowLock(root, wf.id, fn);
  }
  return withStoreLock(root, 'state', fn);
}

// D6 — async: the whole record-read through write() body runs inside
// withStoreLock('state', ...) below so a concurrent set/gate/worker/
// scribing-run/start-feature CLI invocation can no longer race this
// function's read-check-write into a lost update. Argument-only validation
// (no store I/O) stays outside the lock.
async function handleStateSet(root, flags) {
  rejectDryRun(flags);
  if (flags.phase !== undefined) {
    const phase = String(flags.phase);
    if (!isKnownPhase(phase)) {
      throw new Error(
        `set: invalid phase "${phase}" — not in the known-phase enum (isKnownPhase, not the bare PHASES array — the terminal alias "compounding-complete" must pass). FIX: use one of ${KNOWN_PHASES.join(', ')}.`,
      );
    }
  }
  if (
    flags.phase === undefined &&
    flags.mode === undefined &&
    flags.feature === undefined &&
    flags['next-action'] === undefined &&
    flags.summary === undefined
  ) {
    throw new Error(
      'set: at least one of --phase, --mode, --feature, --next-action, --summary is required.',
    );
  }
  const { laneFeature, noLane } = mutationLaneSelector(flags, 'set');
  if (laneFeature && flags.feature !== undefined) {
    throw new Error(
      "set: --feature cannot be combined with --lane — a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or start a new lane instead.",
    );
  }

  const { state, changed, waivedClose, waivedSwap, swapFromFeature, targetLane } = await withMutationLock(root, laneFeature, noLane, async () => {
    const target = resolveMutationTarget(root, laneFeature, 'set', { noLane });
    const { record: state, write } = target;
    // i54-closeout-7 (D7): the fsh-4 identity guard above only sees an
    // EXPLICIT --lane; a session-bound auto-resolved lane hits the same
    // writeLane path (record.feature names the lane file), so --feature must
    // refuse here too — letting it through would silently rename the lane's
    // identity into a second file.
    if (target.source === 'lane' && flags.feature !== undefined) {
      throw new Error(
        `set: --feature cannot target lane "${target.lane}" (auto-resolved from this session's lane binding) — a lane's feature is its identity (the lane record's filename), not a mutable field. FIX: omit --feature, or pass --no-lane to address the default record.`,
      );
    }
    // chain-integrity D1-REVISED / D2 — read `from` off the record actually being
    // mutated (lanes included), never off global state.
    let waivedClose = null;
    if (flags.phase !== undefined) {
      const targetPhase = String(flags.phase);
      const transition = checkPhaseTransition(state.phase, targetPhase);
      if (!transition.ok) throw new Error(transition.reason);
      if (targetPhase === 'compounding-complete') {
        // scribing-integrity si-1 (D2): a lane close checks the LANE's own
        // feature debt (thresholded on the lane's own last_scribing_run) —
        // closeGuardScribingDebt stays the default-record path, unchanged.
        waivedClose = target.source === 'lane'
          ? laneCloseGuardScribingDebt(root, state, flags)
          : closeGuardScribingDebt(root, flags);
      }
    }
    // scribing-integrity si-1 (D1): a feature SWAP over unpaid debt on the
    // CURRENT feature is the second door — same wall, same waiver. Only the
    // default record ever reaches here: --feature is already refused
    // alongside --lane above, and a same-value --feature is not a swap.
    let waivedSwap = null;
    let swapFromFeature = null;
    if (target.source === 'default' && flags.feature !== undefined) {
      const newFeature = String(flags.feature);
      if (state.feature && newFeature !== state.feature) {
        swapFromFeature = state.feature;
        waivedSwap = featureSwapGuardScribingDebt(root, state.feature, flags);
      }
    }
    const selectedRecord = target.source === 'lane' ? `lane "${target.lane}"` : 'default state';
    if (!isKnownPhase(state.phase)) {
      throw new Error(
        `set: refused — selected ${selectedRecord} has missing or invalid pre-mutation phase "${state.phase ?? ''}". Ownership cannot be derived from a corrupt routing record, so nothing was written. FIX: restore a valid phase before retrying.`,
      );
    }
    if (flags.owner === undefined || flags.owner === true || flags.owner === '') {
      throw new Error(
        `set: missing --owner — selected ${selectedRecord}'s pre-mutation phase is "${state.phase}". FIX: retry with --owner ${state.phase}.`,
      );
    }
    const owner = String(flags.owner);
    if (owner !== state.phase) {
      throw new Error(
        `set: owner mismatch — selected ${selectedRecord}'s pre-mutation phase is "${state.phase}", not "${owner}". FIX: retry with --owner ${state.phase}.`,
      );
    }
    const changed = [];
    if (flags.phase !== undefined) {
      state.phase = String(flags.phase);
      changed.push(`phase=${state.phase}`);
    }
    if (flags.mode !== undefined) {
      state.mode = String(flags.mode);
      changed.push(`mode=${state.mode}`);
    }
    if (flags.feature !== undefined) {
      state.feature = String(flags.feature);
      changed.push(`feature=${state.feature}`);
    }
    if (flags['next-action'] !== undefined) {
      state.next_action = String(flags['next-action']);
      changed.push('next_action');
    }
    if (flags.summary !== undefined) {
      state.summary = String(flags.summary);
      changed.push('summary');
    }
    await write(state);
    return { state, changed, waivedClose, waivedSwap, swapFromFeature, targetLane: target.lane };
  });

  // D4 — the waiver is loud and attributable. Logged AFTER the write succeeds so
  // a refused close never leaves a decision claiming one happened. decisions.jsonl
  // is append-only and outside the 'state' lock's store scope, so this stays
  // after the lock releases, unchanged from before.
  const waived = waivedClose; // legacy alias — keeps the block below untouched
  if (waived && waived.length > 0) {
    // jrt-2: this audit record was itself UNTAGGED — the one internal
    // decision-logging call the widened census (test_cells.mjs) was written
    // to catch. It is the worst-placed of the five: the comment above already
    // notes the write happens after the 'state' lock releases, so a throw
    // here would leave the close half-done — state already mutated, the
    // record explaining the waiver never written. Tags drawn from the
    // existing taxonomy (docs/decisions/taxonomy.json): 'scribing' (this IS
    // scribing debt) + 'state' (this IS a feature-close phase transition).
    //
    // Second, separate defect found on this same line: the decision text
    // named "docs/specs/" unconditionally, which is false in a repo that has
    // migrated to a knowledge bundle — docs/specs/ is then the retired,
    // read-only compat surface (see docs/specs/reading-map.md), and the
    // settled behavior actually lives under docs/knowledge/. Resolved the
    // same way hooks/bee-session-close.mjs's capture nudge already does —
    // branching on knowledge.mjs's bundleMode(root) — rather than inventing a
    // second convention. `.bee/bin/**` sits outside the instruction fence's
    // covered surfaces (docs/specs/ and docs/knowledge/ content only), which
    // is why no gate caught this drift when the bundle migration landed.
    const stateLayer = bundleMode(root) ? 'the knowledge bundle (docs/knowledge/)' : 'docs/specs/';
    logDecision(root, {
      decision: `Closed feature "${state.feature}" with scribing debt WAIVED for ${waived.length} capped behavior_change cell(s): ${waived.join(', ')}. Their settled behavior is NOT in ${stateLayer}.`,
      rationale:
        'Explicitly waived via `state set --phase compounding-complete --waive-scribing-debt`. bee refuses this close by default (chain-integrity D2); the waiver is the sanctioned door, and this record is its price.',
      scope: 'repo',
      source: 'agent',
      tags: ['scribing', 'state'],
    });
  }
  // scribing-integrity si-1 (D1) — the swap-waiver twin of the block above:
  // same logged-after-write discipline, same tags, its own decision text
  // naming the ABANDONED feature (there is no "closed feature" here — the
  // record just moved on to a different one).
  if (waivedSwap && waivedSwap.length > 0) {
    const stateLayer = bundleMode(root) ? 'the knowledge bundle (docs/knowledge/)' : 'docs/specs/';
    logDecision(root, {
      decision: `Swapped away from feature "${swapFromFeature}" with scribing debt WAIVED for ${waivedSwap.length} capped behavior_change cell(s): ${waivedSwap.join(', ')}. Their settled behavior is NOT in ${stateLayer}.`,
      rationale:
        'Explicitly waived via `state set --feature <new> --waive-scribing-debt`. bee refuses a feature swap over unpaid debt by default (scribing-integrity D1); the waiver is the sanctioned door, and this record is its price.',
      scope: 'repo',
      source: 'agent',
      tags: ['scribing', 'state'],
    });
  }
  const allWaived = [...(waivedClose || []), ...(waivedSwap || [])];
  const waiverNote = allWaived.length > 0
    ? ` — SCRIBING DEBT WAIVED for ${allWaived.length} cell(s): ${allWaived.join(', ')} (decision logged)`
    : '';
  return {
    result: state,
    text: `Updated state: ${changed.join(' ')}.${targetLane ? ` (lane "${targetLane}")` : ''}${waiverNote}`,
  };
}

// chain-integrity D2/D4 — the close boundary is the ONE place scribing debt is a
// wall instead of a signal. It lives here, not in state.mjs: scribingDebt is in
// cells.mjs, and cells.mjs already imports state.mjs (a back-import would close
// the cycle state.mjs:6-7 exists to avoid). Returns the waived cell ids when the
// caller explicitly waived them, otherwise null. Throws when debt stands.
function closeGuardScribingDebt(root, flags) {
  const debt = scribingDebt(root);
  if (debt.count === 0) return null;
  if (flags['waive-scribing-debt']) return debt.cells;
  throw new Error(
    `set: refusing to close this feature — ${debt.count} capped behavior_change cell(s) have not been synced to docs/specs/: ${debt.cells.join(', ')}.\n` +
      '"compounding-complete" asserts that scribing already ran for them. It has not.\n' +
      'FIX: run bee-scribing to merge the settled behavior into its area spec, then `bee state scribing-run ...` to stamp it.\n' +
      'If the behavior genuinely belongs in no spec, close with --waive-scribing-debt — it is permitted, but it logs a decision naming every cell you waived.',
  );
}

// scribing-integrity si-1 (D1) — the SECOND door. Swapping the default
// record's `feature` to something else while the CURRENT feature carries
// unpaid scribing debt has the exact same effect as closing it silently: the
// abandoned feature's capped behavior_change cells never get their sync, and
// (unlike an explicit close attempt) nothing ever even TRIES the
// compounding-complete wall for it — a dead session just never comes back.
// Same message shape, same --waive-scribing-debt escape, same caller-side
// logged-waiver pattern as closeGuardScribingDebt above (reused, not forked).
// Lanes never reach this: --feature is already refused alongside --lane.
function featureSwapGuardScribingDebt(root, currentFeature, flags) {
  if (!currentFeature) return null; // idle → nothing was abandoned
  const debt = scribingDebt(root, { feature: currentFeature });
  if (debt.count === 0) return null;
  if (flags['waive-scribing-debt']) return debt.cells;
  throw new Error(
    `set: refusing to swap away from feature "${currentFeature}" — ${debt.count} capped behavior_change cell(s) have not been synced to docs/specs/: ${debt.cells.join(', ')}.\n` +
      `Setting --feature abandons "${currentFeature}" without ever reaching its close — the scribing debt would go silent, with no session left to hit the compounding-complete wall for it.\n` +
      'FIX: run bee-scribing to merge the settled behavior into its area spec, then `bee state scribing-run ...` to stamp it.\n' +
      'If the behavior genuinely belongs in no spec, swap with --waive-scribing-debt — it is permitted, but it logs a decision naming every cell you waived.',
  );
}

// scribing-integrity si-1 (D2) — the THIRD door. A lane close used to run
// through closeGuardScribingDebt unconditionally, which reads ONLY the
// default record's `state.feature` — a lane close never computed debt for
// its OWN feature at all, so a debt-carrying lane closed clean whenever the
// default record happened to be idle or on some other feature. This computes
// debt scoped to the LANE's own feature, thresholded on the LANE's own
// last_scribing_run (never the default record's) — same wall shape, same
// waiver escape as the other two doors.
function laneCloseGuardScribingDebt(root, laneRecord, flags) {
  const sinceTs = scribingRunStampMsForGuard(laneRecord.last_scribing_run);
  const debt = scribingDebt(root, { feature: laneRecord.feature, sinceTs });
  if (debt.count === 0) return null;
  if (flags['waive-scribing-debt']) return debt.cells;
  throw new Error(
    `set: refusing to close lane "${laneRecord.feature}" — ${debt.count} capped behavior_change cell(s) have not been synced to docs/specs/: ${debt.cells.join(', ')}.\n` +
      '"compounding-complete" asserts that scribing already ran for them. It has not.\n' +
      'FIX: run bee-scribing to merge the settled behavior into its area spec, then `bee state scribing-run --lane ' +
      `${laneRecord.feature} ...\` to stamp it.\n` +
      'If the behavior genuinely belongs in no spec, close with --waive-scribing-debt — it is permitted, but it logs a decision naming every cell you waived.',
  );
}

// Local mirror of cells.mjs's internal scribingRunStampMs — the lane-close
// guard above is the only bee.mjs caller that needs a lane record's own
// last_scribing_run turned into a threshold ms value, so this stays a small
// unexported helper rather than growing cells.mjs's export surface for one
// caller.
function scribingRunStampMsForGuard(run) {
  if (!run) return 0;
  const parsed = Date.parse(run.at || run.date);
  return Number.isFinite(parsed) ? parsed : 0;
}

// D6 — async: record-read through write() runs inside withStoreLock('state').
async function handleStateGate(root, flags) {
  rejectDryRun(flags);
  if (flags.owner !== undefined) {
    throw new Error(
      'gate: --owner is not accepted — routing ownership protects generic `state set` fields only. FIX: omit --owner and use the dedicated gate command.',
    );
  }
  // ce-1: name/approved batched into one refusal (requireFlags) — 'approved'
  // is enum-checked against the literal CLI-string encoding ('true'/'false')
  // rather than parsed as a boolean first, so an invalid value still lands
  // in the SAME batch as a missing --name instead of throwing separately.
  const { name, approved: approvedRaw } = requireFlags(
    flags,
    [
      { name: 'name', enum: GATE_NAMES },
      { name: 'approved', enum: ['true', 'false'] },
    ],
    exampleFor('state.gate'),
  );
  const approved = approvedRaw === 'true';
  const { laneFeature, noLane } = mutationLaneSelector(flags, 'gate');

  const { state, targetLane } = await withMutationLock(root, laneFeature, noLane, async () => {
    const target = resolveMutationTarget(root, laneFeature, 'gate', { noLane });
    const { record: state, write } = target;
    // Gate 3 advisor precondition (AO3/AO13): high-risk execution never opens
    // without a non-stale advisor_ref. Computed BEFORE any write, so a refusal
    // makes zero mutations. Bound to the SELECTED record's feature (M1): a lane
    // approval checks the lane's own advisor_ref against the lane's plan.md.
    if (name === 'execution' && approved === true && state.mode === 'high-risk') {
      const staleness = advisorRefStale(root, state.advisor_ref, state);
      if (staleness.stale) {
        throw new Error(
          `gate: execution approval refused for high-risk work — the advisor consult is missing or stale (AO3/AO13). ` +
            `Reason(s): ${staleness.reasons.join('; ')}. ` +
            `FIX: resolve the advisor from config (models.<runtime>.advisor), run it read-only with the evidence bundle on stdin, ` +
            `then record the consult: bee state advisor-ref record --advisor "<identity>" --digest-file <path>` +
            `${target.lane ? ` --lane ${target.lane}` : ''}. Nothing is written until a non-stale advisor_ref exists.`,
        );
      }
    }
    // Revocation tracking (AO13): stamp the execution revocation moment so a ref
    // recorded before it reads stale. Only the execution gate is tracked — it is
    // the only revocation the staleness rule needs.
    if (name === 'execution' && approved === false) {
      state.gate_revoked_at = { ...state.gate_revoked_at, execution: new Date().toISOString() };
    }
    state.approved_gates = { ...state.approved_gates, [name]: approved };
    // multisession-native-9 (D7, C2): only the execution gate is ever
    // stamped with a real approved_for_plan_rev — approving it records the
    // TARGET LANE's live workflow's CURRENT plan_rev, so a later `state
    // plan-rev bump` on that same workflow flips this approval's projected
    // effectiveness false without touching any other workflow (invariant 3)
    // or any other gate on THIS workflow (D7 default: context/shape/review
    // stay rev-immune, never stamped). No live workflow for this lane (C1)
    // leaves gateStamp null: nothing plan-rev-scoped to stamp. Deliberately
    // lane-only, permanently — not a msn-10 gap: `state plan-rev bump`
    // itself still refuses outright on the default record (see its own
    // comment below), so stamping a plan_rev on the default path's execution
    // gate would record a value that can never be bumped, and could never
    // go stale. msn-10 keeps the default record's OTHER fields (phase, mode,
    // summary, approved booleans) in sync via workflow-routing, but this ONE
    // rev-stamping behavior stays scoped to lanes exactly as msn-9 shipped it.
    let gateStamp = null;
    if (name === 'execution' && target.lane) {
      const wf = listWorkflows(root).workflows.find((w) => w.feature === target.lane && w.status !== 'closed');
      if (wf) {
        gateStamp = { name: 'execution', approvedForPlanRev: approved ? wf.plan_rev : null };
      }
    }
    await write(state, gateStamp);
    return { state, targetLane: target.lane };
  });
  return {
    result: state,
    text: `Gate "${name}" set to ${approved}.${targetLane ? ` (lane "${targetLane}")` : ''}`,
  };
}

// state.plan-rev.bump (multisession-native-9, CONTEXT.md D7, advisor consult
// slice 2 C2 — binding): bumps a single workflow's plan_rev by 1. plan_rev
// lives ONLY on the workflow record (workflow-store.mjs), so this verb
// REFUSES outright when resolution lands on the default record; it only
// ever targets a lane (explicit --lane, or the calling session's bound
// lane — same resolution order as gate/set/scribing-run via
// resolveMutationTarget, minus the default fallback). Deliberately
// permanent, not a msn-10 gap: msn-10 closed C5 for set/gate/scribing-run/
// advisor-ref record (the default record's OTHER fields now stay in sync
// with its own live workflow), but plan_rev bumping stays lane-only by
// design — see handleStateGate's own gateStamp comment for why stamping the
// default path's execution gate with a plan_rev would be pointless (nothing
// can ever bump it).
//
// The bump's ONLY effect on gates is indirect, via the projection formula in
// state-projection.mjs's workflowGatesToApprovedGates (C2): a gate whose
// `approved_for_plan_rev` was stamped to the PRE-bump rev now reads
// ineffective (`rev !== planRev`) the moment the projection is next
// rebuilt — which this verb does immediately (rebuildLaneProjection), so
// `.bee/lanes/<feature>.json`'s `approved_gates.execution` flips false in
// the SAME call, and a subsequent `cells claim` against that lane's cells
// refuses immediately (cells.mjs's claimCell reads gateApproved off exactly
// this projected file, laneRecordForFeature). Per D7's default (see
// handleStateGate's own gateStamp comment), only the execution gate is ever
// stamped with a real rev, so this is the ONLY gate a bump can ever flip —
// context/shape/review keep whatever they had (null in practice, always
// effective). Bumping workflow W never reads or writes any OTHER workflow's
// record — cross-workflow isolation (invariant 3) falls out of updateWorkflow
// only ever touching the single `wf.id` resolved here, under nothing but
// that workflow's own `workflow:<id>` lock.
async function handleStatePlanRevBump(root, flags) {
  rejectDryRun(flags);
  const { laneFeature, noLane } = mutationLaneSelector(flags, 'plan-rev bump');
  const { feature, planRev, lane } = await withStoreLock(root, 'state', async () => {
    const target = resolveMutationTarget(root, laneFeature, 'plan-rev bump', { noLane });
    if (!target.lane) {
      throw new Error(
        'plan-rev bump: refused — resolution landed on the default (non-lane) record. plan_rev bumping is scoped ' +
          'to lanes only, by design (nothing else ever reads or bumps the default pipeline\'s plan_rev, so ' +
          'stamping it would be meaningless). ' +
          'FIX: target a lane explicitly with --lane <feature>, or bind the calling session to one first ' +
          '("state session bind --session-id <id> --lane <feature>").',
      );
    }
    const wf = listWorkflows(root).workflows.find((w) => w.feature === target.lane && w.status !== 'closed');
    if (!wf) {
      throw new Error(
        `plan-rev bump: no live workflow record found for lane "${target.lane}" — nothing to bump. ` +
          `FIX: start the lane first ("state start-feature --feature ${target.lane} --as-lane").`,
      );
    }
    const updated = await updateWorkflow(root, wf.id, (current) => ({ plan_rev: (current.plan_rev || 0) + 1 }));
    const rebuilt = rebuildLaneProjection(root, target.lane);
    return { feature: target.lane, planRev: updated.plan_rev, lane: rebuilt.lane };
  });
  return {
    result: { feature, plan_rev: planRev, lane },
    text: `Bumped plan_rev to ${planRev} for lane "${feature}" (workflow); lane projection rebuilt.`,
  };
}

// D6 — async: shared by worker add/update/remove/clear, so all four get the
// lock for free. The read-check-write body runs inside withStoreLock('state').
//
// multisession-native-8: state.json's `workers` array is now DISPLAY-ONLY —
// these four verbs stay CLI-compatible (same flags, same output shape,
// same read-check-write-through-state.json mechanics) purely so existing
// scripts/callers keep working, but nothing in bee reads this array as
// truth anymore. Gates/preconditions read claims.mjs's activeWorkers
// derived view instead (state.mjs startFeature, bee status's `workers`
// field) — see claims.mjs activeWorkers's own doc comment.
async function stateWorkerMutate(root, flags, mutate, text) {
  rejectDryRun(flags);
  return withStoreLock(root, 'state', () => {
    const state = readStateStrict(root);
    const workers = Array.isArray(state.workers) ? [...state.workers] : [];
    const resultText = mutate(workers);
    state.workers = workers;
    writeState(root, state);
    return { result: state, text: text ?? resultText };
  });
}

function handleStateWorkerAdd(root, flags) {
  return stateWorkerMutate(root, flags, (workers) => {
    const nickname = requireFlag(flags, 'nickname');
    const cell = requireFlag(flags, 'cell');
    let tier = null;
    if (flags.tier !== undefined) {
      tier = String(flags.tier);
      if (!MODEL_TIERS.includes(tier)) {
        throw new Error(`worker add: invalid tier "${tier}" — must be one of ${MODEL_TIERS.join(', ')}.`);
      }
    }
    const status = flags.status !== undefined ? String(flags.status) : null;
    workers.push({ nickname, cell, tier, status });
    return `Added worker "${nickname}" (cell ${cell}).`;
  });
}

function handleStateWorkerUpdate(root, flags) {
  return stateWorkerMutate(root, flags, (workers) => {
    const nickname = requireFlag(flags, 'nickname');
    const idx = workers.findIndex((w) => w && w.nickname === nickname);
    if (idx === -1) {
      throw new Error(
        `worker update: nickname "${nickname}" not found — use "worker add" to create it first.`,
      );
    }
    const worker = { ...workers[idx] };
    if (flags.cell !== undefined) worker.cell = String(flags.cell);
    if (flags.tier !== undefined) {
      const tier = String(flags.tier);
      if (!MODEL_TIERS.includes(tier)) {
        throw new Error(`worker update: invalid tier "${tier}" — must be one of ${MODEL_TIERS.join(', ')}.`);
      }
      worker.tier = tier;
    }
    if (flags.status !== undefined) worker.status = String(flags.status);
    workers[idx] = worker;
    return `Updated worker "${nickname}".`;
  });
}

function handleStateWorkerRemove(root, flags) {
  return stateWorkerMutate(root, flags, (workers) => {
    const nickname = requireFlag(flags, 'nickname');
    const next = workers.filter((w) => !(w && w.nickname === nickname));
    if (next.length === workers.length) {
      throw new Error(`worker remove: nickname "${nickname}" not found.`);
    }
    workers.length = 0;
    workers.push(...next);
    return `Removed worker "${nickname}".`;
  });
}

function handleStateWorkerClear(root, flags) {
  return stateWorkerMutate(root, flags, (workers) => {
    const removedCount = workers.length;
    workers.length = 0;
    return `Cleared ${removedCount} worker(s).`;
  });
}

// multisession-native-8 (D6): this is a DELIBERATE exception to "no reader
// treats state.workers as truth anymore" — justified, not an oversight.
// worker prune deletes stale files under .bee/workers/ (transient dispatch
// output, a *different* store from the state.json array it shares a name
// with); state.workers here is only an extra, opt-in "protect this cell id's
// transient file" hint an operator can hand-maintain — never a liveness
// check, never a gate/precondition on what work may proceed. Rerouting it to
// activeWorkers' derived (session+claim) view would silently narrow what
// survives a prune to "currently claimed cells", which is a real behavior
// change this cell's contract does not ask for and existing coverage
// (test_cli_state.mjs's worker-prune suite) pins against a hand-written
// array with no live session/claim in the fixture at all.
function readPruneKeepSet(root) {
  // Strict read: a corrupt state.json fails loud here, before any deletion.
  // Prune never writes state.json — it is a read-only verb on state.
  const state = readStateStrict(root);
  if (state.workers !== undefined && state.workers !== null && !Array.isArray(state.workers)) {
    throw new Error(
      'worker prune: state.workers is not an array — refusing to prune against a malformed keep set (a destructive verb fails closed). FIX: repair .bee/state.json via the bee_state.mjs worker verbs first.',
    );
  }
  const keep = new Set();
  for (const w of state.workers || []) {
    if (w && w.cell !== undefined && w.cell !== null) keep.add(String(w.cell));
  }
  const cellsDir = path.join(root, '.bee', 'cells');
  if (fs.existsSync(cellsDir)) {
    for (const file of fs.readdirSync(cellsDir)) {
      if (!file.endsWith('.json')) continue;
      let cell;
      try {
        cell = JSON.parse(fs.readFileSync(path.join(cellsDir, file), 'utf8'));
      } catch {
        cell = null;
      }
      if (!cell || cell.status !== 'capped') keep.add(file.slice(0, -'.json'.length));
    }
  }
  return keep;
}

// Prefix keep-check: "<id>" or "<id>.<anything>" is protected. The suffix
// regex never decides what is kept — only what class of file is a prune
// candidate — so a dotted cell id can never be mis-stemmed into deletion.
function keptByPruneKeepSet(name, keep) {
  for (const id of keep) {
    if (name === id || name.startsWith(`${id}.`)) return true;
  }
  return false;
}

function handleStateWorkerPrune(root, flags) {
  for (const name of Object.keys(flags)) {
    if (name !== 'dry-run') {
      throw new Error(`worker prune: unknown flag --${name}. Use: worker prune [--dry-run] [--json].`);
    }
  }
  const dryRun = flags['dry-run'] !== undefined;
  const workersDir = path.join(root, '.bee', 'workers');
  let keep = readPruneKeepSet(root);
  const entries = fs.existsSync(workersDir)
    ? fs.readdirSync(workersDir, { withFileTypes: true })
    : [];
  const candidates = [];
  const kept = [];
  for (const entry of entries) {
    if (!entry.isFile()) continue;
    const name = entry.name;
    const match = name.match(WORKER_TRANSIENT_SUFFIX);
    if (!match) continue;
    if (name.length === match[0].length) continue; // empty stem is not a transient
    if (keptByPruneKeepSet(name, keep)) {
      kept.push(name);
      continue;
    }
    candidates.push(name);
  }
  const pruned = [];
  if (dryRun) {
    pruned.push(...candidates);
  } else if (candidates.length > 0) {
    // C1: re-read the keep set immediately before the destructive loop.
    keep = readPruneKeepSet(root);
    for (const name of candidates) {
      if (keptByPruneKeepSet(name, keep)) {
        kept.push(name);
        continue;
      }
      fs.rmSync(path.join(workersDir, name));
      pruned.push(name);
    }
  }
  pruned.sort();
  kept.sort();
  const verb = dryRun ? 'Would prune' : 'Pruned';
  const text = `${verb} ${pruned.length} worker transient(s) from .bee/workers/ (kept ${kept.length} still-active).`;
  return { result: { dry_run: dryRun, pruned, kept }, text };
}

// D6 — async: record-read through write() runs inside withStoreLock('state').
async function handleStateScribingRun(root, flags) {
  // sqs-b3: --show is a READ-ONLY query mode — it MUST return here, ABOVE
  // rejectDryRun/requireFlags below. Those two calls demand --areas/
  // --next-action, which a read never supplies; falling through to them
  // would wrongly refuse every --show call for lacking write-only flags
  // (the exact trap this cell exists to avoid — validation-slice1.md's
  // sqs-b3 WARNING). --show never appends to the scribing ledger and never
  // advances phase: it only reads the most-recent recorded stamp, overall or
  // for --feature <slug> if given, reusing readScribingLedger/
  // bestScribingStampMs (lib/cells.mjs) — the same reader and per-feature
  // max-stamp logic globalScribingDebt's orphan sweep uses, never a second
  // implementation.
  if (flags.show === true) {
    const showFeature = flags.feature !== undefined ? String(flags.feature) : null;
    let stampMs = null;
    if (showFeature) {
      stampMs = bestScribingStampMs(root, showFeature);
    } else {
      const ledger = readScribingLedger(root);
      for (const entry of ledger) {
        if (!entry) continue;
        const parsed = Date.parse(entry.ts);
        if (Number.isFinite(parsed) && (stampMs === null || parsed > stampMs)) stampMs = parsed;
      }
    }
    const stampIso = stampMs === null ? null : new Date(stampMs).toISOString();
    const result = { feature: showFeature, stamp: stampIso };
    const text = stampIso
      ? `Last scribing run${showFeature ? ` for "${showFeature}"` : ''}: ${stampIso}`
      : `No scribing run recorded${showFeature ? ` for "${showFeature}"` : ''}.`;
    return { result, text };
  }

  rejectDryRun(flags);
  // ce-1: feature/areas/next-action batched into one refusal (requireFlags).
  const { feature, areas: areasRaw, 'next-action': nextAction } = requireFlags(
    flags,
    [{ name: 'feature' }, { name: 'areas' }, { name: 'next-action' }],
    exampleFor('state.scribing-run'),
  );
  const areas = splitList(areasRaw);
  const now = new Date();
  const at = now.toISOString();
  const date = at.slice(0, 10);
  const { laneFeature, noLane } = mutationLaneSelector(flags, 'scribing-run');

  // scribing-integrity si-1 (D2): a DEFAULT-record call stamping some OTHER
  // feature than the one currently active is a REPAIR — evidence for
  // globalScribingDebt's orphan sweep, not a claim that the default record's
  // own phase should advance. A lane call has no such ambiguity: --lane
  // already pins the record to one feature, so its phase-production stays
  // exactly as today, unconditional.
  let stampedActive = true;
  let activeFeatureAtCall = null;
  let targetLane = null;
  const state = await withMutationLock(root, laneFeature, noLane, async () => {
    const target = resolveMutationTarget(root, laneFeature, 'scribing-run', { noLane });
    const { record: state, write } = target;
    targetLane = target.lane;
    activeFeatureAtCall = state.feature;
    // No active feature at all (idle) means nothing is being abandoned — a
    // lane call is never ambiguous either (a lane record, explicit --lane or
    // session-resolved, is already pinned to one feature). Only a genuine
    // mismatch against an EXISTING active feature is the repair path.
    stampedActive = target.source === 'lane' ? true : !state.feature || state.feature === feature;
    // chain-integrity D3 — scribing-run is the SOLE producer of phase=compounding,
    // so it is also the door that must be guarded. It used to advance the phase
    // from anywhere, with no check that execution had happened at all.
    //
    // scribing-run-terminal-stamp (tst-1): the gate protects the run that
    // ACTUALLY produces a phase transition on the record it targets — the
    // default record stamping its OWN active feature (or no active feature
    // yet), or any lane call (always its own lane record). A non-active
    // default-record call is the si-1 repair path: it never touches phase or
    // last_scribing_run, so it is not the door D3 guards and must stay legal
    // from any phase, including the terminal "compounding-complete"/"idle".
    if (stampedActive) {
      const phaseCheck = checkScribingRunPhase(state.phase);
      if (!phaseCheck.ok) throw new Error(phaseCheck.reason);
      state.last_scribing_run = { feature, date, at, areas_synced: areas, next_action: nextAction };
      // "plus top-level phase/next_action" (bee-scribing SKILL.md:112).
      state.phase = 'compounding';
      state.next_action = nextAction;
      await write(state);
    }
    return state;
  });

  // scribing-integrity si-1 (D2): the durable ledger append — ALWAYS, even on
  // the repair path above where the record itself was left untouched. This is
  // the trace globalScribingDebt's orphan sweep reads for a feature that has
  // no lane and is no longer the active default feature. Fail-open but LOUD
  // (appendScribingLedger's own contract): never masks or blocks this verb's
  // success, never silent either.
  appendScribingLedger(root, { feature, ts: at, areas });

  const repairNote = stampedActive
    ? ''
    : ` — recorded in the durable ledger only: the default record tracks feature "${activeFeatureAtCall}", not "${feature}", so its phase/last_scribing_run were left untouched (repair path for an orphaned feature; \`bee status --json\`'s scribing_debt.orphaned names it).`;
  return {
    result: state,
    text: `Recorded scribing run for "${feature}" at ${at}.${targetLane ? ` (lane "${targetLane}")` : ''}${repairNote}`,
  };
}

// D6 — async: startFeature (lib/state.mjs) already wraps its own
// read-check-write in withStoreLock('state'); this just awaits it.
async function handleStateStartFeature(root, flags) {
  rejectDryRun(flags);
  const feature = requireFlag(flags, 'feature');
  const mode = flags.mode !== undefined ? String(flags.mode) : null;
  const phase = flags.phase !== undefined ? String(flags.phase) : 'exploring';
  // Lane mode (fresh-session-handoff fsh-4, D2/D4): --as-lane starts the
  // feature as a per-feature lane record instead of mutating state.json;
  // --session-id/--paths feed the declared-paths holds-overlap check that
  // startFeature's lane path already implements (fsh-3) — this cell only
  // wires the CLI surface through, no new precondition logic here.
  const lane = flags['as-lane'] === true;
  const sessionId = flags['session-id'] !== undefined ? String(flags['session-id']) : null;
  const paths = flags.paths !== undefined ? splitList(flags.paths) : [];
  // startFeature() re-reads state and performs every precondition check (C1).
  const state = await startFeature(root, { feature, mode, phase, lane, sessionId, paths });
  // multisession-native-7/10 (C5/F8 — "startFeature's dual-write can now
  // route its write through the projection builder"): the legacy record
  // (lane file OR state.json) startFeature just wrote is a hand-built record
  // inside msn-6's own documented three-transaction crash window (legacy
  // write, then a separately-locked workflow create). Reconcile it here
  // against the workflow record startFeature also just created, so the FILE
  // this call leaves behind is always what the matching rebuild derives from
  // that record ("record wins" self-heal, F8) — not merely the raw
  // hand-written copy. `state` (returned above) already carries the same
  // field values, so nothing needs to be re-read for this call's own
  // response, on EITHER branch: rebuildStateProjection's feature-matched
  // branch (msn-10) always finds this exact just-created workflow record
  // (same feature name, freshly created, nothing else could have raced
  // ahead of it), so it can only reproduce byte-identical D1 fields here —
  // a true self-heal, never a surprise.
  if (lane) {
    rebuildLaneProjection(root, state.feature);
  } else {
    rebuildStateProjection(root);
  }
  return {
    result: state,
    text: `Started feature "${state.feature}"${lane ? ' as a lane' : ''} at phase "${state.phase}" (mode ${state.mode ?? 'null'}); all four gates reset.`,
  };
}

// multisession-native-7 — recovery verb (must_have: "A rebuild verb/function
// callable for recovery"): thin CLI wrapper over
// state-projection.mjs's rebuildAllProjections. Read-only from the CLI's own
// perspective in the sense that it never fails or refuses — it is a no-op
// (authoritative:false per record) wherever the projection layer has no
// authority yet: C1 (zero workflow records anywhere), a lane naming no live
// workflow record, or — for state.json specifically — a LIVE non-idle
// default feature (C5's safety gate; see rebuildStateProjection's own
// comment in state-projection.mjs for why).
function handleStateRebuildProjections(root) {
  const result = rebuildAllProjections(root);
  const laneCount = result.lanes.filter((l) => l.authoritative).length;
  const stateNote = result.state.authoritative
    ? `rebuilt .bee/state.json from workflow ${result.state.source}`
    : 'state.json left untouched (no workflow records yet, or a live non-idle default feature — see D1 field scoping)';
  const text = `${stateNote}; ${laneCount} lane projection(s) rebuilt.`;
  return { result, text };
}

// ─── state.lanes / state.session.*: read-only lane listing + session→lane
// binding (fresh-session-handoff fsh-4, D2/D4). claims.mjs stays out of this
// cell's file scope (fsh-3 owns it) — listSessionRecords composes already-
// exported primitives (sessionsDir, readSession) for presentation, exactly
// the same discipline buildReviewBlock above uses for reviews.mjs; it is not
// a second implementation of a mutation, only a read-side enumeration no
// lib module currently offers.

function listSessionRecords(root) {
  let entries;
  try {
    entries = fs.readdirSync(sessionsDir(root));
  } catch {
    return [];
  }
  const sessions = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const record = readSession(root, entry.slice(0, -'.json'.length));
    if (record) sessions.push(record);
  }
  return sessions;
}

function summarizeSession(session) {
  const laneNote = typeof session.lane === 'string' && session.lane ? `-> lane "${session.lane}"` : '(unbound)';
  return `${session.id} ${laneNote} | started ${session.started_at} | heartbeat ${session.last_heartbeat}`;
}

function handleStateLanes(root) {
  const rows = buildLaneRows(root);
  const text = rows.length
    ? rows
        .map((l) => {
          const gates = GATE_NAMES.map((g) => `${g}=${l.approved_gates[g] ? 'approved' : 'pending'}`).join(' ');
          const bindingsNote = l.bound_sessions.length ? ` sessions=${l.bound_sessions.join(',')}` : '';
          return `${l.feature} [${l.phase}] ${gates}${bindingsNote}`;
        })
        .join('\n')
    : 'No lane records.';
  return { result: rows, text };
}

function handleStateSessionList(root) {
  const sessions = listSessionRecords(root);
  return {
    result: sessions,
    text: sessions.length ? sessions.map(summarizeSession).join('\n') : 'No session records.',
  };
}

function handleStateSessionBind(root, flags) {
  const sessionId = requireFlag(flags, 'session-id');
  const laneFeature = requireFlag(flags, 'lane');
  const result = bindSessionLane(root, sessionId, laneFeature);
  if (!result.ok) {
    throw new Error(`session bind: ${result.reason}`);
  }
  return { result: result.session, text: `Session "${sessionId}" bound to lane "${laneFeature}".` };
}

function handleStateSessionUnbind(root, flags) {
  const sessionId = requireFlag(flags, 'session-id');
  const result = unbindSessionLane(root, sessionId);
  if (!result.ok) {
    throw new Error(`session unbind: ${result.reason}`);
  }
  return { result: result.session, text: `Session "${sessionId}" unbound from its lane.` };
}

// ─── state.handoff.*: the two-kind handoff lifecycle CLI surface (fresh-
// session-handoff fsh-9, D1) over lib/state.mjs's guarded writeHandoff/
// adoptHandoff. This cell owns the LIFECYCLE only — claim-next selection is
// fsh-11's, SessionStart wiring (register/rehydrate/auto-resume) is fsh-10's.

function handleStateHandoffWrite(root, flags) {
  rejectDryRun(flags);
  const kind = requireFlag(flags, 'kind');
  const input = { kind };
  if (flags.feature !== undefined) input.feature = String(flags.feature);
  if (flags.phase !== undefined) input.phase = String(flags.phase);
  if (flags.mode !== undefined) input.mode = String(flags.mode);
  if (flags['next-action'] !== undefined) input.next_action = String(flags['next-action']);
  if (kind === 'planned-next') {
    input.writer_session = requireFlag(flags, 'writer-session');
    input.previous_cell = requireFlag(flags, 'previous-cell');
    input.next_cell = requireFlag(flags, 'next-cell');
  } else {
    if (flags.cell !== undefined) input.cell = String(flags.cell);
    if (flags.files !== undefined) input.files = splitList(flags.files);
    if (flags.done !== undefined) input.done = splitList(flags.done);
    if (flags.remaining !== undefined) input.remaining = splitList(flags.remaining);
  }
  const record = writeHandoff(root, input);
  return { result: record, text: `Wrote "${record.kind}" handoff.` };
}

function handleStateHandoffAdopt(root, flags) {
  rejectDryRun(flags);
  const sessionId = requireFlag(flags, 'session-id');
  const result = adoptHandoff(root, sessionId);
  if (!result.ok) {
    throw new Error(`state handoff adopt: ${result.reason}`);
  }
  return {
    result,
    text: `Adopted the handoff's carried claim on "${result.next_cell}" into session "${sessionId}"; handoff cleared.`,
  };
}

function handleStateHandoffShow(root) {
  const handoff = readHandoff(root);
  if (!handoff) return { result: null, text: 'No handoff.' };
  return {
    result: handoff,
    text: `kind=${handoff.kind} feature=${handoff.feature ?? 'unknown'} phase=${handoff.phase ?? 'unknown'} mode=${handoff.mode ?? 'unknown'}`,
  };
}

// ─── state advisor-ref: record/show the AO3/AO13 advisor consult ────────────
// hive law 12: the Gate 3 precondition needs a state field AND a CLI verb. The
// verb stamps the staleness anchors ITSELF (current feature, newest active
// decision id, sha256 of that feature's plan.md) — the caller supplies only the
// advisor identity and a digest for audit; anchors are never caller-supplied.
// multisession-native-7: async now — a lane target's write() routes through
// updateWorkflow (workflow-store.mjs), which is itself async (its own
// per-workflow lock). Dispatch already awaits every handler uniformly
// (main()'s `const response = await handler(...)`), so this is a safe,
// isolated widening.
// multisession-native-10: now wrapped in withMutationLock, closing a
// pre-existing gap — through msn-9 this verb ran its whole read-check-write
// with NO lock at all (unlike set/gate/scribing-run, which at least held
// 'state'). Same lock-routing rule as the other three: a workflow-record-
// routed target locks on `workflow:<id>` only; the C1/no-workflow fallback
// keeps the legacy 'state' lock.
async function handleStateAdvisorRefRecord(root, flags) {
  rejectDryRun(flags);
  const advisor = requireFlag(flags, 'advisor');
  const digestFile = requireFlag(flags, 'digest-file');
  const { laneFeature, noLane } = mutationLaneSelector(flags, 'advisor-ref record');
  const { state, anchors, targetLane } = await withMutationLock(root, laneFeature, noLane, async () => {
    const target = resolveMutationTarget(root, laneFeature, 'advisor-ref record', { noLane });
    const { record: state, write } = target;
    const phase = state.phase;
    if (!state.feature || phase === 'idle' || phase === 'compounding-complete') {
      throw new Error(
        `advisor-ref record: refused — no active feature to anchor the consult to (phase "${phase ?? 'idle'}", feature "${state.feature ?? 'none'}"). ` +
          'FIX: start a feature and reach an in-flight phase before recording an advisor consult.',
      );
    }
    let digestHead = '';
    try {
      digestHead = fs.readFileSync(path.resolve(String(digestFile)), 'utf8').slice(0, 500);
    } catch (err) {
      throw new Error(
        `advisor-ref record: could not read --digest-file "${digestFile}" (${err && err.code ? err.code : err}). ` +
          'FIX: pass the path to the captured advisor consult digest.',
      );
    }
    // Anchors bound to the SELECTED record's feature (M1), stamped by the verb.
    const anchors = advisorRefAnchors(root, state.feature);
    state.advisor_ref = {
      consulted_at: new Date().toISOString(),
      feature: anchors.feature,
      newest_decision_id: anchors.newest_decision_id,
      plan_sha256: anchors.plan_sha256,
      advisor: String(advisor),
      digest_head: digestHead,
    };
    await write(state);
    return { state, anchors, targetLane: target.lane };
  });
  return {
    result: state.advisor_ref,
    text: `Recorded advisor_ref (advisor "${advisor}", feature "${anchors.feature}").${targetLane ? ` (lane "${targetLane}")` : ''}`,
  };
}

function handleStateAdvisorRefShow(root, flags) {
  const laneFeature = optionalLaneFlag(flags, 'advisor-ref show');
  const state = laneFeature ? readLaneStrict(root, laneFeature) : readStateStrict(root);
  if (laneFeature && !state) {
    throw new Error(`advisor-ref show: lane "${laneFeature}" does not exist (no .bee/lanes/${laneFeature}.json).`);
  }
  const raw = state ? state.advisor_ref : null;
  const ref = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : null;
  if (!ref) return { result: null, text: 'No advisor_ref recorded.' };
  const staleness = advisorRefStale(root, ref, state);
  return {
    result: { advisor_ref: ref, stale: staleness.stale, reasons: staleness.reasons },
    text:
      `advisor="${ref.advisor}" feature="${ref.feature}" consulted_at=${ref.consulted_at} stale=${staleness.stale}` +
      `${staleness.reasons.length ? ` (${staleness.reasons.join('; ')})` : ''}`,
  };
}

// ─── state compact-*: compaction-hardening's helper floor (D3) — thin CLI
// wrappers over lib/compaction.mjs. All three of D3's locked verbs are wired
// here now that cz-5 has landed buildCompactCapsule.

function handleStateCompactLog(root, flags) {
  const event = requireFlag(flags, 'event');
  const sessionId = requireFlag(flags, 'session-id');
  const record = appendCompactionRecord(root, { event, sessionId });
  return {
    result: record,
    text: `Recorded "${record.event}" for session "${record.session}" (compact_index=${record.compact_index}, cell_compact_count=${record.cell_compact_count}, anchor_present=${record.anchor_present}).`,
  };
}

// D12/D13: reports only, never mutates. compactCheck's own 'anchor' check
// (item 7) is a plain presence test; the `anchor_missing` check added here is
// the DISTINCT D10 nudge predicate (phase/gate-scoped) so the exact nudge
// command stays reachable by command even where the predicate does not fire
// (D3's helper floor). Both checks are legitimately present together.
function handleStateCompactCheck(root, flags) {
  const sessionId = requireFlag(flags, 'session-id');
  const sweep = compactCheck(root, { sessionId });
  const nudge = anchorMissing(root, { sessionId });
  const anchorMissingCheck = {
    name: 'anchor_missing',
    ok: !nudge,
    detail: nudge ? nudge.message : 'no anchor-missing nudge applies for this session right now.',
    command: ANCHOR_NUDGE_COMMAND,
  };
  const checks = [...sweep.checks, anchorMissingCheck];
  const mismatches = checks.filter((entry) => entry.ok !== true);
  const result = { ok: mismatches.length === 0, session: sweep.session, checks, mismatches };
  const lines = [`compact-check ${result.ok ? 'OK' : 'MISMATCH'} (session "${sessionId}")`];
  for (const entry of checks) {
    if (entry.skipped) continue;
    lines.push(`  [${entry.ok ? 'ok' : 'MISMATCH'}] ${entry.name}: ${entry.detail}`);
  }
  return { result, text: lines.join('\n') };
}

// D6/D27: the capsule's `handoffOutcome` is a CALL-SITE obligation, not merely
// a parameter the renderer accepts. The SessionStart hook computes it and
// passes it to buildSessionPreamble (bee-session-init.mjs:113-121,144); this
// verb reaches the same truth through nonAdoptingHandoffOutcome so the
// command-line surface renders the exact bytes the hook renders — including
// the `- Adoption not applied:` line a compacted session holding a planned-next
// handoff would otherwise silently lose. Read-only, like compact-check.
function handleStateCompactCapsule(root, flags) {
  const sessionId = requireFlag(flags, 'session-id');
  const capsule = buildCompactCapsule(root, {
    sessionId,
    handoffOutcome: nonAdoptingHandoffOutcome(root),
  });
  return { result: { session: sessionId, capsule }, text: capsule };
}

// ─── backlog: full port of bee_backlog.mjs's counts/rank/badges/add verbs
// (dispatcher-unify du-2). Reuses lib/backlog.mjs's read/rank/badge exports
// and lib/feedback.mjs's KIND_ALIASES/NORMALIZED_KINDS exactly as
// bee_backlog.mjs did — no logic change in either lib file. `add`'s
// validation refusal texts and exit codes stay as the existing test_lib
// bee_backlog checks pin them (DB3). ───────────────────────────────────────

const BACKLOG_SEVERITIES = ['P1', 'P2', 'P3'];
const BACKLOG_MAX_TITLE = 200;
const BACKLOG_MAX_LAYER = 40;
// `propose` targets docs/backlog.md's own Story/CoS cells (PD1), a
// different table than `add`'s .bee/backlog.jsonl title/layer above —
// BACKLOG_MAX_STORY mirrors BACKLOG_MAX_TITLE's convention on purpose.
const BACKLOG_MAX_STORY = 200;
const BACKLOG_MAX_COS = 2000;

function backlogAllowedTypes() {
  return [...new Set([...Object.keys(KIND_ALIASES), ...NORMALIZED_KINDS])].sort();
}

function handleBacklogCounts(root) {
  const counts = readBacklogCounts(root);
  if (!counts) return { result: null, text: 'No docs/backlog.md found.' };
  return {
    result: counts,
    text: `PBI: ${counts.done} done / ${counts.inFlight} in-flight / ${counts.proposed} proposed (${counts.total} total)`,
  };
}

// backlog-unification D3: rank --write is RETIRED — the generated view
// (renderBacklogPbiView / `bee backlog render`) now owns docs/backlog.md.
// The bare dry-run report stays (still useful, still read-only) — only
// --write refuses, naming render.
function handleBacklogRank(root, flags) {
  if (flags.write === true) {
    throw new Error(
      'backlog rank --write is retired — "bee backlog render --write" now owns the generated docs/backlog.md view.',
    );
  }
  const ranked = rankBacklog(root, { write: false });
  if (!ranked) return { result: null, text: 'No parseable backlog table in docs/backlog.md.' };
  const verb = ranked.changed ? 'Would reorder to' : 'Already ordered';
  return {
    result: ranked,
    text: `${verb}: ${ranked.order.join(', ')}${ranked.changed ? ' ("rank --write" is retired — run "bee backlog render --write" instead)' : ''}`,
  };
}

function handleBacklogBadges(root, flags) {
  const write = flags.write === true;
  const badges = updateReadmeBadges(root, { write });
  if (!badges) return { result: null, text: 'README.md or docs/backlog.md missing — nothing to badge.' };
  const verb = write ? (badges.changed ? 'README badges refreshed' : 'README badges already current') : badges.changed ? 'README badges stale (re-run with --write to apply)' : 'README badges already current';
  return { result: badges, text: `${verb}: ${badges.badges}` };
}

const BACKLOG_COMMIT_SUBJECT_MAX = 72;

/** Runs a git subcommand scoped to `root` — same spawnSync('git', args, {
 * cwd, encoding: 'utf8' }) shape as lib/worktree-store.mjs's runGit and
 * lib/reviews.mjs's defaultRunGit, kept local here since bee.mjs's own
 * handleBacklogAdd is the only caller. */
function runBacklogGit(root, args) {
  return spawnSync('git', args, { cwd: root, encoding: 'utf8' });
}

/** Commits the just-appended backlog row on its own, scoped to ONLY
 * .bee/backlog.jsonl — never `git add -A` / a plain `git commit` — so any
 * other staged or unstaged change in the tree is left untouched. Degrades
 * to {committed:false, sha:null} without throwing on every failure path
 * (no .git, spawn error, non-zero exit): appendJsonl already succeeded by
 * the time this runs, and that success must never be rolled back or turned
 * into a command failure — this is what keeps the no-.git tmpdir fixture in
 * test_bee_cli.mjs's backlog.add example passing unchanged.
 *
 * Gated on `queueSubmit` (D1): a human queue-submit row (`--queue-submit`)
 * is the only case that attempts a commit at all — an agent's own
 * self-observation row (friction/debt/finding about its own session) is
 * appended by appendJsonl but never committed, so the flag stays false and
 * this returns immediately without invoking git. When queueSubmit is true,
 * a merge in progress is detected up front (D2) by resolving the REAL
 * git-dir via `git rev-parse --git-dir` — never a hardcoded .git/MERGE_HEAD,
 * since linked worktrees point .git elsewhere — and checking for a
 * MERGE_HEAD file there before attempting the scoped commit. */
function commitBacklogRow(root, line, queueSubmit) {
  if (!queueSubmit) {
    return { committed: false, sha: null };
  }

  const inWorkTree = runBacklogGit(root, ['rev-parse', '--is-inside-work-tree']);
  if (inWorkTree.error || inWorkTree.status !== 0 || (inWorkTree.stdout || '').trim() !== 'true') {
    return { committed: false, sha: null };
  }

  const gitDirResult = runBacklogGit(root, ['rev-parse', '--git-dir']);
  if (gitDirResult.error || gitDirResult.status !== 0 || !(gitDirResult.stdout || '').trim()) {
    return { committed: false, sha: null };
  }
  const gitDir = path.resolve(root, gitDirResult.stdout.trim());
  if (fs.existsSync(path.join(gitDir, 'MERGE_HEAD'))) {
    return { committed: false, sha: null, commit_skipped_reason: 'merge_in_progress' };
  }

  const backlogPathspec = path.join('.bee', 'backlog.jsonl');
  const addResult = runBacklogGit(root, ['add', '--', backlogPathspec]);
  if (addResult.error || addResult.status !== 0) {
    return { committed: false, sha: null };
  }

  const truncatedTitle =
    line.title.length > BACKLOG_COMMIT_SUBJECT_MAX ? `${line.title.slice(0, BACKLOG_COMMIT_SUBJECT_MAX)}...` : line.title;
  const bodyLines = [`type: ${line.type}`, `severity: ${line.severity}`, `layer: ${line.layer}`];
  if (line.feature) bodyLines.push(`feature: ${line.feature}`);
  if (line.detail) bodyLines.push(`detail: ${line.detail}`);
  const message = `chore(backlog): ${truncatedTitle}\n\n${bodyLines.join('\n')}`;

  const commitResult = runBacklogGit(root, ['commit', '-m', message, '--', backlogPathspec]);
  if (commitResult.error || commitResult.status !== 0) {
    return { committed: false, sha: null };
  }

  const shaResult = runBacklogGit(root, ['rev-parse', 'HEAD']);
  if (shaResult.error || shaResult.status !== 0 || !(shaResult.stdout || '').trim()) {
    return { committed: false, sha: null };
  }
  return { committed: true, sha: shaResult.stdout.trim() };
}

function handleBacklogAdd(root, flags) {
  // ce-1: every missing/invalid flag reported in ONE refusal (batch), via
  // requireFlags — type/severity enums enforced here (not by the generic
  // validate() layer: backlog.add's registry entry stays `required: []`,
  // DB3). Length checks (title/layer) stay separate, ad hoc, AFTER the
  // batch — they are not part of the {name, enum?} spec shape.
  const { type, title, severity, layer } = requireFlags(
    flags,
    [
      { name: 'type', enum: backlogAllowedTypes() },
      { name: 'title' },
      { name: 'severity', enum: BACKLOG_SEVERITIES },
      { name: 'layer' },
    ],
    exampleFor('backlog.add'),
  );
  if (title.length > BACKLOG_MAX_TITLE) {
    throw new Error(`add: --title is ${title.length} chars, over the ${BACKLOG_MAX_TITLE}-char limit. FIX: shorten the title.`);
  }
  if (layer.length > BACKLOG_MAX_LAYER) {
    throw new Error(`add: --layer is ${layer.length} chars, over the ${BACKLOG_MAX_LAYER}-char limit. FIX: shorten the layer.`);
  }
  const detail = flags.detail !== undefined && flags.detail !== true ? String(flags.detail) : '';
  const feature = flags.feature !== undefined && flags.feature !== true ? String(flags.feature) : '';
  const queueSubmit = flags['queue-submit'] === true;
  const line = {
    ts: new Date().toISOString(),
    type,
    title,
    detail,
    severity,
    layer,
    feature,
  };
  appendJsonl(path.join(root, '.bee', 'backlog.jsonl'), line);
  const { committed, sha, commit_skipped_reason: commitSkippedReason } = commitBacklogRow(root, line, queueSubmit);
  const commitSuffix = committed
    ? ` (committed ${sha.slice(0, 7)})`
    : commitSkippedReason === 'merge_in_progress'
      ? ' (auto-commit skipped: merge in progress)'
      : '';
  return {
    result: {
      ...line,
      committed,
      ...(committed ? { commit_sha: sha } : {}),
      ...(commitSkippedReason ? { commit_skipped_reason: commitSkippedReason } : {}),
    },
    text: `Appended ${severity} ${type} row to .bee/backlog.jsonl: "${title}"${commitSuffix}`,
  };
}

/** `backlog.propose` (backlog-submit-command D1/D2/D3/D5): the human-facing
 * submit verb — a story + acceptance criteria, no id and no title ceremony.
 * It targets the PBI layer, not the friction queue `add` above writes.
 * Storage is the .bee/backlog.jsonl PBI fold, NOT docs/backlog.md: since
 * backlog-unification D3 that table is a GENERATED view (`bee backlog
 * render`), so appending a row to it would be erased by the next render and
 * flagged as drift by `render --check` in between. propose is therefore a
 * validating front door onto addPbi — the length caps and required-ness live
 * here, before the single append, so any rejection leaves the log
 * byte-untouched, matching `add`'s own convention. */
function handleBacklogPropose(root, flags) {
  const story = requireFlag(flags, 'story').trim();
  if (!story) {
    throw new Error('propose: --story is required (non-empty).');
  }
  if (story.length > BACKLOG_MAX_STORY) {
    throw new Error(`propose: --story is ${story.length} chars, over the ${BACKLOG_MAX_STORY}-char limit. FIX: shorten the story.`);
  }
  const cos = requireFlag(flags, 'cos').trim();
  if (!cos) {
    throw new Error('propose: --cos is required (non-empty).');
  }
  if (cos.length > BACKLOG_MAX_COS) {
    throw new Error(`propose: --cos is ${cos.length} chars, over the ${BACKLOG_MAX_COS}-char limit. FIX: shorten the cos.`);
  }
  const feature = flags.feature !== undefined && flags.feature !== true ? String(flags.feature) : '';
  const item = addPbi(root, { title: story, cos, status: 'proposed', feature });
  const row = { id: item.id, story: item.title, cos: item.cos || cos, feature: item.feature || '—' };
  return {
    result: row,
    text: `Proposed ${row.id}: "${row.story}" (feature: ${row.feature})`,
  };
}

// ─── backlog pbi / backlog render (backlog-unification D1-D5): the
// event-sourced product-backlog layer. Enum/duplicate/unknown-id validation
// lives in lib/backlog.mjs's writers themselves (addPbi/setPbiStatus/
// amendPbi/listPbis all throw a FIX-bearing Error) — these handlers just
// shape flags in and the result out, same division of labor as every other
// thin handler in this file. ─────────────────────────────────────────────

function handleBacklogPbiAdd(root, flags) {
  const title = requireFlag(flags, 'title');
  const cos = flags.cos !== undefined && flags.cos !== true ? String(flags.cos) : '';
  const status = flags.status !== undefined && flags.status !== true ? String(flags.status) : 'proposed';
  const feature = flags.feature !== undefined && flags.feature !== true ? String(flags.feature) : undefined;
  const id = flags.id !== undefined && flags.id !== true ? String(flags.id) : undefined;
  const item = addPbi(root, { id, title, cos, status, feature });
  return { result: item, text: `Added PBI ${item.id}: "${item.title}"` };
}

function handleBacklogPbiStatus(root, flags) {
  const id = requireFlag(flags, 'id');
  const to = requireFlag(flags, 'to');
  const feature = flags.feature !== undefined && flags.feature !== true ? String(flags.feature) : undefined;
  const item = setPbiStatus(root, { id, to, feature });
  return { result: item, text: `PBI ${id} -> ${to}${feature ? ` (feature: ${feature})` : ''}` };
}

function handleBacklogPbiAmend(root, flags) {
  const id = requireFlag(flags, 'id');
  const title = flags.title !== undefined && flags.title !== true ? String(flags.title) : undefined;
  const cos = flags.cos !== undefined && flags.cos !== true ? String(flags.cos) : undefined;
  const item = amendPbi(root, { id, title, cos });
  return { result: item, text: `Amended PBI ${id}` };
}

function handleBacklogPbiList(root, flags) {
  const status = flags.status !== undefined && flags.status !== true ? String(flags.status) : undefined;
  const list = listPbis(root, { status });
  return {
    result: list,
    text: list.length ? list.map((item) => `${item.id} [${item.status}] ${item.title}`).join('\n') : 'No PBIs.',
  };
}

function handleBacklogRender(root, flags) {
  const write = flags.write === true;
  const check = flags.check === true;
  const rendered = renderBacklogPbiView(root, { write });
  if (check && rendered.changed) {
    throw new Error('backlog render --check: docs/backlog.md is stale. FIX: run "bee backlog render --write" to refresh it.');
  }
  const verb = write
    ? rendered.changed
      ? 'Rendered'
      : 'Already current'
    : check
      ? rendered.changed
        ? 'STALE'
        : 'Current'
      : rendered.changed
        ? 'Would render (re-run with --write to apply)'
        : 'Already current';
  return { result: rendered, text: `${verb}: docs/backlog.md` };
}

// sqs-b2 (state-query-surface plan §Approach cell sqs-b2, gather Q3): a read
// verb over .bee/backlog.jsonl's friction/finding rows. TWO schemas coexist
// on this stream: legacy rows carry `kind: "friction"|"finding"` (pre-
// backlog-unification), current rows (handleBacklogAdd above) carry
// `type: "friction"|"finding"` — a row counts if EITHER field matches. Reuses
// readJsonl's generic read/parse/skip-malformed loop (fsutil.mjs, the same
// backbone lib/feedback.mjs's collectFeedback and lib/backlog.mjs's foldPbis
// build their own row-specific logic on top of); kind:'pbi' rows are always
// skipped first — a disjoint shape (foldPbis), never a friction/finding.
const BACKLOG_FINDING_KINDS = new Set(['friction', 'finding']);

function isBacklogFindingRow(row) {
  if (!row || typeof row !== 'object') return false;
  if (row.kind === 'pbi') return false;
  return BACKLOG_FINDING_KINDS.has(row.kind) || BACKLOG_FINDING_KINDS.has(row.type);
}

// --feature is a word-boundary/exact match, NOT substring ("auth" must not
// match "authz") — reuses matchesWholeToken, the same discipline sqs-b1
// applied to decisions --cell/--feature (there is no structural join there;
// here there IS a structural `feature` field on the row, so this is exact-
// modulo-case rather than a free-text scan, but the collision guard is the
// same word-boundary regex).
function matchesBacklogFeature(row, feature) {
  const value = row.feature != null ? String(row.feature) : '';
  if (!value) return false;
  return matchesWholeToken([value], feature);
}

// --text is substring over title+detail (like the existing decisions --text
// convention): any whitespace-split term hitting either field is enough.
function matchesBacklogText(row, text) {
  const terms = String(text).toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) return true;
  const haystacks = [row.title, row.detail]
    .filter((v) => v !== null && v !== undefined && v !== '')
    .map((v) => String(v).toLowerCase());
  return terms.some((term) => haystacks.some((h) => h.includes(term)));
}

function readBacklogFindings(root, { feature, text }) {
  const rows = readJsonl(path.join(root, '.bee', 'backlog.jsonl'));
  return rows.filter((row) => {
    if (!isBacklogFindingRow(row)) return false;
    if (!matchesBacklogFeature(row, feature)) return false;
    if (text && !matchesBacklogText(row, text)) return false;
    return true;
  });
}

function handleBacklogFindings(root, flags) {
  const feature = requireFlag(flags, 'feature');
  const text = flags.text !== undefined && flags.text !== true ? String(flags.text) : null;
  const findings = readBacklogFindings(root, { feature, text });
  return {
    result: { findings },
    text: findings.length
      ? findings.map((f) => `[${f.severity || '—'}] ${f.title}`).join('\n')
      : `No friction/finding rows for feature "${feature}".`,
  };
}

// ─── capture: full port of bee_capture.mjs's add/list/flush/count verbs
// (dispatcher-unify du-2). Reuses lib/capture.mjs's exports exactly as
// bee_capture.mjs did — no logic change there. ─────────────────────────────

function formatCaptureStub(stub) {
  const marker = stub.source === 'mined' ? ' [mined]' : '';
  const parts = [`[${stub.at}] ${stub.outcome}${marker} (id ${stub.id})`];
  if (stub.dids && stub.dids.length) parts.push(`  decisions: ${stub.dids.join(', ')}`);
  if (stub.area) parts.push(`  area: ${stub.area}`);
  if (stub.files && stub.files.length) parts.push(`  files: ${stub.files.join(', ')}`);
  if (stub.source) parts.push(`  source: ${stub.source}`);
  return parts.join('\n');
}

function handleCaptureAdd(root, flags) {
  const stub = addCaptureStub(root, {
    outcome: requireFlag(flags, 'outcome'),
    dids: flags.did ? String(flags.did) : null,
    area: flags.area ? String(flags.area) : null,
    files: flags.files ? String(flags.files) : null,
    lane: flags.lane ? String(flags.lane) : null,
    source: flags.source ? String(flags.source) : null,
  });
  return {
    result: stub,
    text: `Queued capture stub ${stub.id}. Flush via bee-scribing at wrap-up, before compact/clear, or next session (decision 0017).`,
  };
}

function handleCaptureList(root) {
  const stubs = pendingCaptureStubs(root);
  const text = stubs.length ? stubs.map(formatCaptureStub).join('\n') : 'Capture queue is empty.';
  return { result: { count: stubs.length, stubs }, text };
}

function handleCaptureFlush(root, flags) {
  const record = flushCaptureStub(root, requireFlag(flags, 'id'), {
    into: flags.into ? String(flags.into) : null,
  });
  return {
    result: record,
    text: `Flushed stub ${record.id}${record.into ? ` into ${record.into}` : ''}.`,
  };
}

function handleCaptureCount(root) {
  const queue = captureQueue(root);
  return { result: { count: queue.count }, text: `${queue.count} pending capture stub(s).` };
}

// ─── intent: the intent anchor CLI (intent-anchor ia-1, D1/D2). Presentation
// only — every rule (verbatim request, immutability of request/acceptance,
// key resolution, fail-open reads) lives in lib/intent.mjs. ────────────────

function intentLookupOptions(flags) {
  return {
    feature: flags.feature ? String(flags.feature) : null,
    sessionId: flags.session ? String(flags.session) : null,
  };
}

function formatAnchor(anchor) {
  const lines = [
    `Intent anchor "${anchor.key}" (written ${anchor.written_at})`,
    'ORIGINAL REQUEST (verbatim):',
    anchor.request,
    `DONE MEANS: ${anchor.acceptance}`,
  ];
  if (anchor.next_action) lines.push(`NEXT ACTION: ${anchor.next_action}`);
  if (anchor.do_not_reverse.length) lines.push(`DO NOT REVERSE: ${anchor.do_not_reverse.join(' | ')}`);
  if (anchor.stop_conditions.length) lines.push(`STOP IF: ${anchor.stop_conditions.join(' | ')}`);
  return lines.join('\n');
}

function handleIntentSet(root, flags) {
  const anchor = writeIntent(
    root,
    {
      // requireFlag returns the flag's own string unchanged — the request is
      // never trimmed, re-wrapped, or summarized anywhere in this pipeline.
      request: requireFlag(flags, 'request'),
      acceptance: requireFlag(flags, 'acceptance'),
      next_action: flags['next-action'] ? String(flags['next-action']) : null,
      feature: flags.feature ? String(flags.feature) : null,
      lane: flags.lane ? String(flags.lane) : null,
      cell: flags.cell ? String(flags.cell) : null,
      do_not_reverse: flags['do-not-reverse'] ? String(flags['do-not-reverse']) : null,
      stop_conditions: flags['stop-conditions'] ? String(flags['stop-conditions']) : null,
    },
    { ...intentLookupOptions(flags), force: String(flags.force) === 'true' },
  );
  return {
    result: anchor,
    text: `Intent anchored at "${anchor.key}". It is re-asserted at PreCompact and read first on a compact/resume start.`,
  };
}

function handleIntentShow(root, flags) {
  const anchor = readIntent(root, intentLookupOptions(flags));
  const render = flags.render ? String(flags.render) : null;
  if (render === 'precompact' || render === 'resume') {
    const block = render === 'precompact' ? precompactBlock(anchor) : resumeBlock(anchor);
    return { result: { anchor, render, block }, text: block || '(no intent anchor)' };
  }
  return { result: anchor, text: anchor ? formatAnchor(anchor) : '(no intent anchor)' };
}

function handleIntentAdvance(root, flags) {
  const nextAction = requireFlag(flags, 'next-action');
  const anchor = advanceIntent(root, nextAction, intentLookupOptions(flags));
  if (!anchor) {
    throw new Error('intent advance: no intent anchor exists to advance — run `bee intent set` first.');
  }
  return {
    result: anchor,
    text: `Advanced intent anchor "${anchor.key}" — next action: ${anchor.next_action}. Request and acceptance are unchanged.`,
  };
}

function handleIntentClear(root, flags) {
  const record = clearIntent(root, intentLookupOptions(flags));
  return {
    result: record,
    text: record.cleared ? `Cleared intent anchor "${record.key}".` : `No intent anchor at "${record.key}" to clear.`,
  };
}

// ─── knowledge: OKF v0.1 bundle verbs (okf-foundation S1, lib/knowledge.mjs).
// check is the two-level D4 validator over docs/knowledge/ ONLY (D23); the
// stdout JSON is exactly the D13 shape {okf:{errors},profile:{errors,warnings},
// counts}, and the exit code is non-zero on OKF errors, on any chain-failing
// PROFILE error (G14 layer 3, cell f3-3 — the chain runs this verb without
// --strict, so a backstop that only lived in `warnings` never blocked
// anything), or on any finding at all under --strict. Read-only end to end (D2).

function handleKnowledgeCheck(root, flags) {
  const strict = flags.strict === true;
  const report = checkBundle(root, { strict });
  const failing = !report.ok;
  const profileErrors = report.profile.errors || [];
  const lines = [];
  for (const finding of report.okf.errors) {
    lines.push(`ERROR [${finding.code}] ${finding.file}: ${finding.message}`);
  }
  for (const finding of profileErrors) {
    lines.push(`ERROR [${finding.code}] ${finding.file}: ${finding.message}`);
  }
  for (const finding of report.profile.warnings) {
    lines.push(`${strict ? 'ERROR(strict)' : 'WARN'} [${finding.code}] ${finding.file}: ${finding.message}`);
  }
  lines.push(
    `knowledge check: ${report.counts.concepts} concept(s) in ${report.counts.files} file(s), ` +
      `${report.okf.errors.length} OKF error(s), ${profileErrors.length} profile error(s), ` +
      `${report.profile.warnings.length} profile warning(s)` +
      `${strict ? ' [--strict]' : ''} — ${failing ? 'FAIL' : 'OK'}`,
  );
  return {
    result: { okf: report.okf, profile: report.profile, counts: report.counts },
    text: lines.join('\n'),
    exitCode: failing ? 1 : 0,
  };
}

// index (okf-4, D21): write mode renders the full generated-index set —
// index writes ONLY generated index.md files inside docs/knowledge/, never a
// .bee store (D2). --check is the read-only decisions-render idiom: re-render
// in memory, byte-compare against disk, exit non-zero NAMING each stale file.
function handleKnowledgeIndex(root, flags) {
  if (flags.check === true) {
    const { stale, checked } = knowledgeIndexDrift(root);
    const drift = stale.length > 0;
    const lines = stale.map((file) => `STALE ${file}`);
    lines.push(
      `knowledge index --check: ${checked} expected index file(s), ${stale.length} stale — ` +
        `${drift ? 'FAIL (regenerate: bee knowledge index)' : 'OK'}`,
    );
    return {
      result: { checked, stale, drift },
      text: lines.join('\n'),
      exitCode: drift ? 1 : 0,
    };
  }
  const { written, count } = renderKnowledgeIndexes(root);
  return {
    result: { written, count },
    text: `Rendered ${count} generated index file(s) under docs/knowledge/.`,
  };
}

// list (okf-4, D15): one row per concept — path, id, type, lifecycle, title —
// never file content. Filters are exact; --area matches bee.areas membership.
function handleKnowledgeList(root, flags) {
  const filters = {
    type: typeof flags.type === 'string' ? flags.type : null,
    lifecycle: typeof flags.lifecycle === 'string' ? flags.lifecycle : null,
    area: typeof flags.area === 'string' ? flags.area : null,
  };
  const rows = listConcepts(root, filters);
  const lines = rows.map(
    (row) => `${row.path} · ${row.id ?? '-'} · ${row.type ?? '-'} · ${row.lifecycle ?? '-'} · ${row.title ?? '-'}`,
  );
  lines.push(`${rows.length} concept(s).`);
  return {
    result: { concepts: rows, count: rows.length },
    text: lines.join('\n'),
  };
}

// context (okf-7, D27; ranking hardened by G5/G11): the budget-aware consumer.
// Read-only end to end — buildContextManifest resolves the work item by bee.id,
// walks required_context transitively (cycles deduped silently), RANKS the
// critical concepts by relevance to the work item and cuts them, adds the area
// decisions, and cuts at --budget using the NAMED bytes/4 estimator. The human
// form is a table of the same rows the --json manifest carries — plus one
// EXCLUDED line per critical dropped by the relevance cut, because a silent
// exclusion is worse than the noise it replaces. Neither form ever carries file
// content. An unresolvable --work id, or a zero-signal ranking over a real
// population, throws a typed error the dispatcher's catch turns into exit 1
// ({"error":...} on stdout under --json, stderr otherwise).
function handleKnowledgeContext(root, flags) {
  const manifest = buildContextManifest(root, { work: flags.work, budget: flags.budget });
  const lines = [
    `work: ${manifest.work} · budget: ${manifest.budget} token(s) · estimator: ${manifest.estimator}`,
  ];
  if (manifest.decisions.length > 0) lines.push(`decisions: ${manifest.decisions.join(' · ')}`);
  lines.push('PATH · BYTES · EST TOKENS · REASON');
  for (const entry of manifest.entries) {
    lines.push(`${entry.path} · ${entry.bytes} · ${entry.est_tokens} · ${entry.reason}`);
  }
  for (const cut of manifest.truncated) lines.push(`TRUNCATED ${cut}`);
  for (const dropped of manifest.excluded) {
    lines.push(`EXCLUDED ${dropped.path} · ${dropped.score} · ${dropped.reason}`);
  }
  lines.push(
    `knowledge context: ${manifest.entries.length} entry(ies), ${manifest.total_est} est token(s) ` +
      `of ${manifest.budget} budget (estimator ${manifest.estimator}), ${manifest.truncated.length} truncated, ` +
      `${manifest.excluded.length} excluded of ${manifest.critical_total} critical pattern(s); ` +
      `zero_signal_count ${manifest.zero_signal_count}; floor ${manifest.floor.length}.`,
  );
  return { result: manifest, text: lines.join('\n') };
}

// promote (okf-9, D38): the loop closer. Reads the bundle and the CAPPED cell
// traces in .bee/cells/ (a read of the runtime store — D2 permits reads and
// forbids writes) and PRINTS three proposals. This handler performs no writes
// of any kind: there is no --apply, and `writes` in the payload is always [].
// Saving a proposal is a separate, human or agent, decision. An unresolvable
// --work id throws the typed unknown_work error the dispatcher's catch turns
// into exit 1 ({"error":...} on stdout under --json, stderr otherwise).
function handleKnowledgePromote(root, flags) {
  const proposal = buildPromotion(root, { work: flags.work });
  const lines = [
    `promote proposal for work item "${proposal.work}" (${proposal.work_item}) — ${proposal.cells.length} capped cell(s)` +
      `${proposal.cells.length > 0 ? `: ${proposal.cells.map((cell) => cell.id).join(', ')}` : ''}`,
    'PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.',
    '',
    `(a) DELIVERY DRAFT — save as ${proposal.delivery.repo_path}`,
    '',
    proposal.delivery.content.replace(/\n$/, ''),
    '',
    '(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell',
    '',
  ];
  if (proposal.area_updates.length === 0) {
    lines.push(`None: the work item declares no bee.areas, so there is no area to sync (D19).`, '');
  }
  for (const update of proposal.area_updates) {
    lines.push(`area ${update.area}:`);
    if (update.bullets.length === 0) {
      lines.push('  (no capped behavior_change cell touched this area\'s subjects)');
    }
    for (const bullet of update.bullets) {
      lines.push(`  - [${bullet.cell}] ${bullet.text} — touched ${bullet.files.join(', ')} (trace ${bullet.trace})`);
    }
    lines.push('');
  }
  lines.push('(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall', '');
  if (proposal.pattern_candidates.length === 0) {
    lines.push('None: no capped cell trace carries a deviation or a failure signature.', '');
  }
  for (const candidate of proposal.pattern_candidates) {
    lines.push(`from cell ${candidate.cell} — save as ${candidate.repo_path}`, '', candidate.content.replace(/\n$/, ''), '');
  }
  lines.push(
    `knowledge promote: ${proposal.cells.length} capped cell(s) mined, 1 delivery draft, ` +
      `${proposal.area_updates.reduce((sum, update) => sum + update.bullets.length, 0)} area bullet(s), ` +
      `${proposal.pattern_candidates.length} pattern candidate(s), 0 file(s) written.`,
  );
  return { result: proposal, text: lines.join('\n') };
}

// ─── reviews: full port of bee_reviews.mjs's create/list/show/record/
// candidate add/candidates/status verbs (dispatcher-unify du-3). Reuses
// lib/reviews.mjs's exports exactly as bee_reviews.mjs did — no logic change
// there. `required: []` on every reviews registry entry is deliberate (DB3,
// same discipline as state.*/backlog.*): the generic validate() layer would
// emit its structured error on STDOUT, but the legacy bee_reviews.mjs
// contract (pinned by test_lib.mjs) emits its validation refusals on
// STDERR. So each handler owns its own required-flag / enum checks (via the
// shared requireFlag above), throwing the legacy message text — which the
// dispatcher routes to STDERR through the catch-block -> emitError path.
// reviews.candidate.add is a NESTED 3-segment name resolved by the
// dispatcher's longest-prefix match (du-1), sitting alongside the separate
// FLAT reviews.candidates verb (bee_reviews.mjs:186-207/199-207) — two
// distinct verbs, both pinned.

function readReviewsJsonInput(flags, label) {
  const text = flags.stdin === true ? fs.readFileSync(0, 'utf8') : readFileText(requireFlag(flags, 'file'), label);
  try {
    return JSON.parse(text);
  } catch {
    throw new Error(`${label}: input is not valid JSON.`);
  }
}

function summarizeReview(session) {
  return `${session.id} [${session.decision && session.decision.status}] ${session.scope_description}`;
}

// A7: a candidate reviewed by an unchanged approved session names the
// covering review-id so the orchestrator never re-dispatches a full panel.
function candidateStatusLine(candidate, derived) {
  const target = `${candidate.feature}@${candidate.head} (${candidate.mode})`;
  if (derived.status === 'reviewed') {
    return `${target} — reviewed (covered by ${derived.session})`;
  }
  if (derived.status === 'review stale') {
    const note = derived.note ? `, ${derived.note}` : '';
    return `${target} — review stale (was covered by ${derived.session}${note})`;
  }
  if (derived.status === 'in review') {
    return `${target} — in review (session ${derived.session})`;
  }
  return `${target} — unreviewed`;
}

function buildReviewsStatusSummary(root, { feature } = {}) {
  const candidates = listCandidates(root).filter((c) => !feature || c.feature === feature);
  const sessions = listReviews(root);
  const counts = { verified: candidates.length };
  for (const label of CANDIDATE_STATUSES) counts[label] = 0;

  // D2 (cli-performance CONTEXT): same pass-local gitMemo idiom as
  // buildReviewBlock above — born once per summary pass, never persisted.
  const gitMemo = new Map();
  const rows = candidates.map((candidate) => {
    const derived = deriveCandidateStatus(root, candidate, { sessions, gitMemo });
    counts[derived.status] += 1;
    return {
      ...candidate,
      review_status: derived.status,
      review_session: derived.session || null,
      note: derived.note || null,
    };
  });

  return { counts, candidates: rows };
}

function renderReviewsStatusText(summary) {
  const counts = summary.counts;
  const headline =
    `verified: ${counts.verified}  unreviewed: ${counts.unreviewed}  ` +
    `in review: ${counts['in review']}  reviewed: ${counts.reviewed}  review stale: ${counts['review stale']}`;
  if (summary.candidates.length === 0) return `${headline}\nNo review candidates.`;
  return [headline, ...summary.candidates.map((c) => candidateStatusLine(c, { status: c.review_status, session: c.review_session, note: c.note }))].join('\n');
}

function handleReviewsCreate(root, flags) {
  const scope = readReviewsJsonInput(flags, 'scope');
  const session = createReview(root, scope);
  return { result: session, text: `Created review session ${session.id}.` };
}

function handleReviewsList(root) {
  const sessions = listReviews(root);
  return {
    result: sessions,
    text: sessions.length ? sessions.map(summarizeReview).join('\n') : 'No review sessions.',
  };
}

function handleReviewsShow(root, flags) {
  const id = requireFlag(flags, 'id');
  const session = readReview(root, id);
  if (!session) throw new Error(`Review session "${id}" not found.`);
  return { result: session, text: JSON.stringify(session, null, 2) };
}

function handleReviewsRecord(root, flags) {
  const id = requireFlag(flags, 'id');
  const kind = requireFlag(flags, 'kind');
  const payload = readReviewsJsonInput(flags, 'payload');
  const session = recordOnReview(root, id, { kind, payload });
  return { result: session, text: `Recorded ${kind} on ${session.id} (updated_at ${session.updated_at}).` };
}

function handleReviewsCandidateAdd(root, flags) {
  const feature = requireFlag(flags, 'feature');
  // GitHub #16: when --cells is omitted, auto-fill from the feature's capped
  // cells. A cells-less candidate can never cell-match a review session that
  // included the work by CELL, so it stayed stuck "unreviewed" even after the
  // session approved it. Deriving the cells here closes that coverage gap.
  const cells = flags.cells
    ? splitList(flags.cells)
    : listCells(root)
        .filter((c) => c.feature === feature && c.status === 'capped')
        .map((c) => c.id);
  const entry = addCandidate(root, {
    feature,
    head: requireFlag(flags, 'head'),
    mode: requireFlag(flags, 'mode'),
    baseline: flags.baseline ? String(flags.baseline) : null,
    cells,
  });
  return {
    result: entry,
    text: `Added candidate ${entry.id} for feature "${entry.feature}" (mode ${entry.mode}${
      entry.cells.length ? `, ${entry.cells.length} cell(s)` : ''
    }).`,
  };
}

function handleReviewsCandidates(root) {
  const entries = listCandidates(root);
  return {
    result: entries,
    text: entries.length
      ? entries.map((e) => `${e.date} ${e.feature} @${e.head} (${e.mode})`).join('\n')
      : 'No review candidates.',
  };
}

function handleReviewsStatus(root, flags) {
  const feature = flags.feature ? String(flags.feature) : null;
  const summary = buildReviewsStatusSummary(root, { feature });
  return { result: summary, text: renderReviewsStatusText(summary) };
}

// ─── feedback: full port of bee_feedback.mjs's digest/count/collect/rank
// verbs (dispatcher-unify du-3). Reuses lib/feedback.mjs's buildDigest/
// mergeDigests/clusterEntries/rankClusters exactly as bee_feedback.mjs did —
// no logic change there. NO collection, redaction, or pain logic lives here.

const DEFAULT_FEEDBACK_DIGEST_PATH = path.join('.bee', 'feedback-digest.json');

// Presentation only — groups the digest's own `dropped[].reason` values for a
// human-readable one-line summary. No new drop reasons are invented here; the
// category vocabulary is DROP_REASONS in lib/feedback.mjs.
function summarizeDropped(dropped) {
  const byReason = {};
  for (const d of dropped) {
    const key = (d && d.reason) || 'unknown';
    byReason[key] = (byReason[key] || 0) + 1;
  }
  const keys = Object.keys(byReason).sort();
  if (keys.length === 0) return 'none';
  return keys.map((k) => `${k}: ${byReason[k]}`).join(', ');
}

function feedbackSummaryLine(digest) {
  const { counts, dropped } = digest;
  const entryWord = counts.entries === 1 ? 'entry' : 'entries';
  return `${counts.entries} ${entryWord}, ${counts.dropped} dropped (${summarizeDropped(dropped)})`;
}

function handleFeedbackDigest(root, flags) {
  const digest = buildDigest(root, { now: new Date() });
  const outRel = flags.out ? String(flags.out) : DEFAULT_FEEDBACK_DIGEST_PATH;
  const outPath = path.resolve(root, outRel);
  writeJsonAtomic(outPath, digest);
  return {
    result: { path: outRel, digest },
    text: `Digest written to ${outRel} — ${feedbackSummaryLine(digest)}.`,
  };
}

function handleFeedbackCount(root) {
  const digest = buildDigest(root, { now: new Date() });
  return {
    result: digest.counts,
    text: `${feedbackSummaryLine(digest)}.`,
  };
}

function handleFeedbackCollect(root) {
  const digest = mergeDigests(root, { now: new Date() });
  const foreign = Array.isArray(digest.merged) ? digest.merged.length : 0;
  const suffix = foreign > 0 ? ` + ${foreign} dogfood repo${foreign === 1 ? '' : 's'}` : '';
  return {
    result: digest,
    text: `Merged digest — ${feedbackSummaryLine(digest)}${suffix}.`,
  };
}

function handleFeedbackRank(root) {
  const digest = mergeDigests(root, { now: new Date() });
  const clusters = clusterEntries(digest);
  const ranked = rankClusters(clusters);
  const top = ranked.length > 0 ? ranked[0] : null;
  const topWord = top ? `top rank ${top.rank} (pain ${top.pain} × frequency ${top.frequency} × corroboration ${top.corroboration})` : 'no clusters';
  return {
    result: ranked,
    text: `${ranked.length} cluster${ranked.length === 1 ? '' : 's'} — ${topWord}.`,
  };
}

// ─── perf group (lib/perf.mjs) — global cross-project performance log ──────
// Sections are computed post-hoc from the Claude Code session transcript on
// disk. Handlers degrade gracefully: a missing transcript / open marker yields
// zeroed metrics or a clear message, never a throw (so the read-only registry
// examples pass under assertExampleOk in a transcript-less CI).

function perfMarkerPath(root) {
  return cacheFilePath(root, 'perf-open.json');
}
// Legacy location (pre-#11): the marker used to sit directly in .bee/ root.
function legacyPerfMarkerPath(root) {
  return path.join(root, '.bee', 'perf-open.json');
}

function perfGitBranch(root) {
  try {
    const head = fs.readFileSync(path.join(root, '.git', 'HEAD'), 'utf8').trim();
    const m = /ref:\s*refs\/heads\/(.+)$/.exec(head);
    return m ? m[1] : head || null;
  } catch {
    return null;
  }
}

// perfParseSince — a trailing-window start: "30m"/"2h"/"1d"/"45s" relative to
// nowMs, or an ISO timestamp. Returns epoch-ms, or NaN when unparseable.
function perfParseSince(since, nowMs) {
  if (!since) return NaN;
  const m = /^(\d+)\s*([smhd])$/.exec(String(since).trim());
  if (m) {
    const unit = { s: 1000, m: 60000, h: 3600000, d: 86400000 }[m[2]];
    return nowMs - Number(m[1]) * unit;
  }
  const t = Date.parse(since);
  return Number.isNaN(t) ? NaN : t;
}

function perfShortModel(model) {
  return String(model).replace(/^claude-/, '').replace(/-\d{6,}$/, '');
}

function perfSectionLine(rec) {
  const models = Object.entries(rec.models || {})
    .map(([m, v]) => `${perfShortModel(m)} ${v.total}`)
    .join(', ') || '—';
  const flag = rec.parallel ? '∥' : 'seq';
  const scope = rec.project ? `${rec.project}${rec.branch ? `@${rec.branch}` : ''}` : '';
  return `${rec.started_at}  ${rec.label || '(unlabeled)'}  · ${rec.running_time_human} · ${flag} · ${models}${scope ? ` · ${scope}` : ''}`;
}

function perfRenderMarkdown(sections) {
  if (!sections.length) return '# bee performance log\n\n_No sections logged yet._\n';
  const lines = ['# bee performance log', ''];
  for (const rec of sections) {
    lines.push(`## ${rec.label || '(unlabeled)'} — ${rec.started_at}`);
    if (rec.note) lines.push(`_${rec.note}_`);
    lines.push('');
    lines.push(`- Project: \`${rec.project || '?'}\`${rec.branch ? ` (branch \`${rec.branch}\`)` : ''}`);
    lines.push(`- Running time: **${rec.running_time_human}** (${rec.running_time_ms} ms active)`);
    lines.push(`- Parallel: ${rec.parallel ? 'yes' : 'no'}${rec.subagent_count ? ` (${rec.subagent_count} subagent${rec.subagent_count === 1 ? '' : 's'})` : ''}`);
    const models = Object.entries(rec.models || {});
    if (models.length) {
      lines.push('- Models:');
      for (const [m, v] of models) {
        lines.push(`  - \`${perfShortModel(m)}\`: total ${v.total} (new ${v.new}, cached ${v.cached})`);
      }
    } else {
      lines.push('- Models: —');
    }
    const sub = Object.entries(rec.subagent_models || {});
    if (sub.length) {
      lines.push('- Subagent tokens:');
      for (const [m, v] of sub) lines.push(`  - \`${perfShortModel(m)}\`: total ${v.total} (new ${v.new}, cached ${v.cached})`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

function handlePerfStart(root, flags) {
  const projectPath = root;
  const transcript = resolveTranscript(claudeProjectsRoot(), projectPath, { sessionId: flags.session });
  const marker = {
    label: flags.label || null,
    transcript,
    session_id: transcript ? path.basename(transcript, '.jsonl') : flags.session || null,
    started_at: new Date().toISOString(),
    project: projectPath,
    branch: perfGitBranch(root),
  };
  writeJsonAtomic(perfMarkerPath(root), marker);
  removeFileIfExists(legacyPerfMarkerPath(root));
  const text = transcript
    ? `perf: section "${marker.label || '(unlabeled)'}" started — measuring session ${marker.session_id}.`
    : `perf: section "${marker.label || '(unlabeled)'}" started, but no session transcript resolved yet — metrics will be empty at stop.`;
  return { result: marker, text };
}

function handlePerfStop(root, flags) {
  const markerPath = perfMarkerPath(root);
  // Prefer the new .bee/cache/ marker; fall back to a legacy root marker opened
  // before the #11 migration so an in-flight section can still be stopped.
  const marker = readJson(markerPath, null) || readJson(legacyPerfMarkerPath(root), null);
  if (!marker) {
    return { result: { ok: false, reason: 'no-open-section' }, text: 'perf: no open section — run `bee perf start` first.' };
  }
  const endTs = new Date().toISOString();
  const metrics = computeMetrics(marker.transcript, Date.parse(marker.started_at), Date.parse(endTs));
  const rec = buildSection({
    label: marker.label,
    note: flags.note || null,
    projectPath: marker.project || root,
    branch: marker.branch || perfGitBranch(root),
    sessionId: marker.session_id,
    startTs: marker.started_at,
    endTs,
    metrics,
  });
  const file = appendSection(rec);
  // Clear both the new and any legacy marker (best-effort; the section is logged).
  removeFileIfExists(markerPath);
  removeFileIfExists(legacyPerfMarkerPath(root));
  return { result: rec, text: `${perfSectionLine(rec)}\nlogged → ${file}` };
}

function handlePerfSection(root, flags) {
  const endTs = new Date().toISOString();
  const endMs = Date.parse(endTs);
  const startMs = perfParseSince(flags.since, endMs);
  if (Number.isNaN(startMs)) {
    return { result: { ok: false, reason: 'bad-since' }, text: 'perf: --since must be a duration (e.g. 30m, 2h, 1d) or an ISO timestamp.' };
  }
  const transcript = resolveTranscript(claudeProjectsRoot(), root, { sessionId: flags.session });
  const metrics = computeMetrics(transcript, startMs, endMs);
  const rec = buildSection({
    label: flags.label || null,
    note: flags.note || null,
    projectPath: root,
    branch: perfGitBranch(root),
    sessionId: transcript ? path.basename(transcript, '.jsonl') : flags.session || null,
    startTs: new Date(startMs).toISOString(),
    endTs,
    metrics,
  });
  const file = appendSection(rec);
  return { result: rec, text: `${perfSectionLine(rec)}\nlogged → ${file}` };
}

function handlePerfLog(_root, flags) {
  const limit = flags.limit ? Number(flags.limit) : 20;
  const sections = readSections({ limit: Number.isFinite(limit) && limit > 0 ? limit : undefined });
  const text = sections.length ? sections.map(perfSectionLine).join('\n') : 'perf: no sections logged yet.';
  return { result: sections, text };
}

function handlePerfRender(_root, flags) {
  const limit = flags.limit ? Number(flags.limit) : undefined;
  const sections = readSections({ limit: Number.isFinite(limit) && limit > 0 ? limit : undefined });
  return { result: sections, text: perfRenderMarkdown(sections) };
}

function handlePerfSync(_root, _flags) {
  const res = syncSessionsToLog(claudeProjectsRoot(), { cachePath: scanCachePath() });
  return { result: res, text: `perf: synced ${res.sessions} session(s) across ${res.projects} project(s) into the log.` };
}

function handlePerfReport(_root, flags) {
  // The report READS the persistent store (performance.jsonl); it never scans
  // transcripts at view time. If the store is empty (first run), backfill once.
  if (readSessionRecords().length === 0) {
    try {
      syncSessionsToLog(claudeProjectsRoot(), { cachePath: scanCachePath() });
    } catch {
      // backfill is best-effort; an empty matrix is still valid output.
    }
  }
  const matrix = buildMatrixFromLog(process.env, os.homedir(), { since: flags.since });
  if (flags.html || flags.out) {
    const file = writeReport(matrix, { out: flags.out });
    return {
      result: { path: file, projects: matrix.projects.length, sessions: matrix.totals.sessions },
      text: `perf: matrix for ${matrix.projects.length} project(s) written → ${file}`,
    };
  }
  if (!matrix.projects.length) return { result: matrix, text: 'perf: no session activity found yet. Run `bee perf sync`.' };
  const lines = matrix.projects.map(
    (p) => `${p.project}  · ${p.sessions} sess · ${Math.round(p.running_time_ms / 1000)}s · ${p.total_tokens} tok (${p.parallel_sessions}/${p.sessions} parallel)`,
  );
  return { result: matrix, text: lines.join('\n') };
}

// ─── recovery: crash-candidate detection + bounded mining-window CLI (D1-D6,
// docs/history/transcript-recovery/CONTEXT.md). Mining itself never runs
// here (D4) — `recovery window` only emits the down-tier worker's prompt;
// the orchestrator dispatches it to a down-tier worker, never an LLM call
// from inside the CLI. `recovery scan` never auto-triggers mining (D2). ────

// lastTranscriptActivity — the newest event timestamp in a candidate's
// transcript tail, for `recovery scan`'s text summary only (the JSON result
// carries detectCrashCandidates()'s own shape unchanged, no added field). A
// tiny display-only re-derivation, not a second copy of recovery.mjs's own
// since/work-signal math: a null transcript or an unreadable/empty tail
// resolves to null, never throws.
function lastTranscriptActivity(transcript) {
  if (!transcript) return null;
  let maxMs = null;
  for (const event of readTranscriptTail(transcript)) {
    const ts =
      event && typeof event === 'object'
        ? typeof event.timestamp === 'string'
          ? Date.parse(event.timestamp)
          : typeof event.at === 'string'
            ? Date.parse(event.at)
            : NaN
        : NaN;
    if (Number.isFinite(ts) && (maxMs === null || ts > maxMs)) maxMs = ts;
  }
  return maxMs === null ? null : new Date(maxMs).toISOString();
}

function summarizeRecoveryCandidate(c) {
  return `${c.session_id} [${c.lane || 'no-lane'}] runtime=${c.runtime || 'claude'} last_heartbeat=${c.last_heartbeat || 'unknown'} transcript=${c.transcript || 'null'} last_activity=${lastTranscriptActivity(c.transcript) || 'unknown'}`;
}

// summarizeTranscriptRoot — one line per scanned/skipped transcript root
// (hardening-5), appended to `recovery scan`'s human-readable text summary so
// a second-runtime user can see a configured root was actually consulted (or
// why it was skipped) without needing --json. The JSON `result` field below
// stays the bare candidates array, unchanged, to keep `bee recovery scan
// --json`'s existing shape byte-identical for every caller that parses it.
function summarizeTranscriptRoot(r) {
  return `root ${r.runtime} (${r.path}): ${r.scanned ? 'scanned' : `skipped (${r.reason})`}`;
}

function handleRecoveryScan(root, _flags) {
  const candidates = detectCrashCandidates(root);
  const roots = scanTranscriptRoots(root);
  const candidateText = candidates.length ? candidates.map(summarizeRecoveryCandidate).join('\n') : 'recovery: no crash candidates.';
  const rootsText = roots.map(summarizeTranscriptRoot).join('\n');
  const text = `${candidateText}\n${rootsText}`;
  return { result: candidates, text };
}

// handleRecoveryWindow — from the bare session id alone, re-derive the whole
// window: read the session record, resolve its transcript, compute sinceTs
// from the last durable settlement (lane-scoped, global fallback, else the
// session's own started_at — D3), then the bounded window and the miner
// prompt (D4). The orchestrator dispatches `prompt`; this handler never
// calls an LLM.
function handleRecoveryWindow(root, flags) {
  const sessionId = requireFlag(flags, 'session');
  const session = readSession(root, sessionId);
  if (!session) throw new Error(`Session "${sessionId}" not found.`);
  const transcript = resolveTranscript(claudeProjectsRoot(), root, { sessionId });
  const lane = session.lane || null;
  const settled = lastDurableSettlement(root, lane);
  const sinceTs = settled != null ? settled : session.started_at || null;
  const window = computeMiningWindow(transcript, sinceTs);
  const prompt = buildMiningPrompt({ session_id: sessionId, lane }, window);
  const result = {
    transcript,
    since_ts: sinceTs,
    event_count: window.event_count,
    window_truncated: window.window_truncated,
    prompt,
  };
  return { result, text: prompt };
}

// ─── worktree: register/list/unregister the opt-in per-worktree store grant
// (worktree-feature-parallelism Slice A). `root` here is main()'s already-
// resolved storeRoot, whose value is GRANT-STATE-DEPENDENT for a linked
// worktree (resolveRoots falls back to mainRoot when ungranted, but returns
// the worktree's own root once granted) — so every handler below re-resolves
// `process.cwd()` itself via resolveRoots to get the authoritative, grant-
// state-INDEPENDENT `{ id, mainRoot, worktreeRoot }` rather than trusting the
// ambient `root` value. ───────────────────────────────────────────────────

/**
 * The MAIN store's checkout root, regardless of the current directory's own
 * grant state: for a linked worktree (granted or not), resolveRoots' own
 * `mainRoot` field is authoritative; for an ordinary checkout, `root` (the
 * dispatcher's already-resolved storeRoot) already IS the main root.
 */
function resolveMainRoot(root) {
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch {
    return root;
  }
  if (resolution.worktreeResolution === 'linked-valid' && resolution.mainRoot) {
    return resolution.mainRoot;
  }
  return root;
}

// xwh-2: a synthetic "acting holder" for findForeignHolds' list-all reuse in
// handleReservationsList — control-char-wrapped so it can never collide with
// a real holder value (either the literal string 'main' or a git-verified
// worktree id, itself always a plain directory basename): findForeignHolds
// only ever excludes entries whose `holder` equals this string, and nothing
// ever mirrors a hold under it.
const LIST_ALL_HOLDS_SENTINEL = '\u0000bee-reservations-list-all\u0000';

// xwh-2: resolves the cross-worktree HOLD topology for a reservation call —
// distinct from resolveMainRoot above (which only ever answers "where is the
// main store"). Returns `{ mainRoot, holder }` for the two topologies the
// cell's action names as hold-worthy:
//   - an ORDINARY checkout: holder = 'main', mainRoot = the checkout itself.
//   - a GRANTED linked worktree (its own storeRoot === its own worktreeRoot,
//     i.e. resolveRoots did NOT fall back to main): holder = its
//     git-verified id, mainRoot = resolveRoots' own `mainRoot`.
// Returns `null` for every other case — an UNGRANTED linked worktree
// (storeRoot === mainRoot: `root` here already IS the shared main store, so
// mirroring it again under a synthetic identity would just be a duplicate
// entry for a reservation the shared store already carries directly) and an
// unresolvable/invalid checkout (resolveRoots threw) both fall through to
// `null`, which callers treat as "skip the cross-worktree wiring entirely,
// exactly like before this cell" — never a refusal on its own.
function resolveHoldTopology(root) {
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch {
    return null;
  }
  if (resolution.worktreeResolution === 'ordinary') {
    return { mainRoot: resolution.workRoot || root, holder: 'main' };
  }
  if (resolution.worktreeResolution === 'linked-valid' && resolution.mainRoot && resolution.id) {
    const granted = resolution.storeRoot && resolution.worktreeRoot && path.resolve(resolution.storeRoot) === path.resolve(resolution.worktreeRoot);
    if (granted) {
      return { mainRoot: resolution.mainRoot, holder: resolution.id };
    }
  }
  return null;
}

/** Same expiry-string convention as guards.mjs's private `holdExpiry`
 * (reservations), rebased on a ledger hold's `mirrored_at`/`ttl_seconds`
 * fields instead of a reservation's `reserved_at`/`ttl_seconds` — kept as its
 * own tiny helper rather than importing guards.mjs's unexported one. */
function holdForeignExpiry(hold) {
  const mirroredMs = Date.parse(hold?.mirrored_at);
  const ttl = hold?.ttl_seconds;
  if (!Number.isFinite(mirroredMs) || !Number.isFinite(ttl) || ttl <= 0) return 'no expiry';
  return `expires ${new Date(mirroredMs + ttl * 1000).toISOString()}`;
}

async function handleWorktreeRegister(_root, flags) {
  const feature = requireFlag(flags, 'feature');
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch (error) {
    throw new Error(
      `"bee worktree register" must be run from inside a linked git worktree (git worktree add): ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  if (resolution.worktreeResolution !== 'linked-valid') {
    throw new Error(
      `"bee worktree register" must be run from inside a linked git worktree (git worktree add), not an "${resolution.worktreeResolution}" checkout.`,
    );
  }
  const { id, mainRoot, worktreeRoot } = resolution;
  const mainStoreRoot = path.join(mainRoot, '.bee');
  // hardening-4b: writeGrant is now withStoreLock-wrapped (async, serialized
  // under the 'worktree-admin' lock).
  await writeGrant(mainStoreRoot, id);
  const bootstrap = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, feature);
  const result = { ok: true, id, feature, main_root: mainRoot, worktree_root: worktreeRoot, bootstrap };
  const text = [
    `Registered worktree grant: id ${id} (feature "${feature}").`,
    `  worktree:    ${worktreeRoot}`,
    `  main store:  ${mainStoreRoot}`,
    bootstrap.created
      ? `  bootstrapped ${bootstrap.worktreeStoreRoot} (phase idle, gates unapproved).`
      : `  worktree .bee/state.json already existed — left untouched (${bootstrap.reason}).`,
  ].join('\n');
  return { result, text };
}

// "bee worktree new --feature <slug>" (GH #21, decision D7): create AND
// register a fresh linked git worktree in one move. MUST run from the MAIN
// (ordinary) checkout — resolveRoots is the same primitive
// handleWorktreeRegister uses to require the opposite ('linked-valid'); here
// it must be 'ordinary', because "new" is what CREATES the linked worktree
// register later runs inside of.
async function handleWorktreeNew(_root, flags) {
  const feature = requireFlag(flags, 'feature');
  const baseRef = flags['base-ref'] !== undefined ? String(flags['base-ref']) : undefined;
  const withCompanion = flags['with-companion'] === true;
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch (error) {
    throw new Error(
      `"bee worktree new" must be run from inside the main checkout (not a linked worktree): ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  if (resolution.worktreeResolution !== 'ordinary' || !resolution.workRoot) {
    throw new Error(
      `"bee worktree new" must be run from inside the main checkout, not a "${resolution.worktreeResolution}" checkout — run it from the main repo root, then open your next session inside the created worktree.`,
    );
  }
  const mainRoot = resolution.workRoot;
  // worktree-companion-hook: resolved HERE (readConfig(mainRoot).commands.*)
  // and passed down as plain option strings, same posture as verifyCommand
  // below in handleWorktreeMerge — worktree-store.mjs stays zero-deps-beyond-
  // node-builtins. --with-companion with no commands.worktree_companion_start
  // configured is refused HERE, before any worktree is created, rather than
  // surfacing as a less obvious failure from inside createFeatureWorktree.
  let companionStartCommand;
  let companionMountPath;
  if (withCompanion) {
    const commands = readConfig(mainRoot).commands;
    companionStartCommand = commands.worktree_companion_start || undefined;
    companionMountPath = commands.worktree_companion_mount || undefined;
    if (!companionStartCommand) {
      throw new Error('--with-companion requires commands.worktree_companion_start to be set in .bee/config.json.');
    }
    if (!companionMountPath) {
      throw new Error('--with-companion requires commands.worktree_companion_mount to be set in .bee/config.json.');
    }
  }
  // hardening-4b: createFeatureWorktree now runs its whole body inside
  // withStoreLock('worktree-admin') (async).
  const created = await createFeatureWorktree(mainRoot, { feature, baseRef, companionStartCommand, companionMountPath });
  // GH #31 (wux-1, messaging only): the explicit session-boundary next-step —
  // this session (in mainRoot) never cd's into the new worktree itself, so
  // the success output has to say so plainly: open a NEW session there, and
  // name the merge-back command up front so it isn't rediscovered later.
  const nextStep = `Open a new session with cwd=${created.worktreeRoot} to work the "${feature}" feature there — this session stays on main. Merge back later with "bee worktree merge --id ${created.id}".`;
  const result = {
    id: created.id,
    worktreeRoot: created.worktreeRoot,
    branch: created.branch,
    baseRef: created.baseRef,
    baseRefSha: created.baseRefSha,
    skillsSync: created.skillsSync,
    companion: created.companion || null,
    next_step: nextStep,
  };
  const skillsLine = created.skillsSync.applied
    ? '  skills:      bee-* skill trees synced into the worktree.'
    : `  skills:      NOT synced (${created.skillsSync.reason}) — bee* skills may be missing in a session opened there.`;
  const text = [
    `Created worktree for feature "${feature}": ${created.worktreeRoot}`,
    created.baseRefSha
      ? `  branch:      ${created.branch} (based on ${JSON.stringify(created.baseRef)}, resolved to ${created.baseRefSha})`
      : `  branch:      ${created.branch}`,
    created.bootstrap.created
      ? `  bootstrapped ${created.bootstrap.worktreeStoreRoot} (phase idle, gates unapproved).`
      : `  worktree .bee/state.json already existed — left untouched (${created.bootstrap.reason}).`,
    skillsLine,
    created.companion
      ? `  companion:   mounted at ${created.companion.mountPath} (${created.companion.worktreePath}${created.companion.sessionId ? `, session ${created.companion.sessionId}` : ''}).`
      : null,
    nextStep,
  ]
    .filter((line) => line !== null)
    .join('\n');
  return { result, text };
}

// "bee worktree merge --id <id>" (GH #21, decision D8): merge a granted
// worktree's branch back into MAIN and run the host project's configured
// verify against the merged tree — the semantic-conflict alarm for a merge
// that is textually clean but breaks behavior. Requires an ORDINARY checkout
// (same resolveRoots-based guard handleWorktreeNew uses, for the same
// reason: running merge from inside ANY linked worktree — including the one
// named by --id — already fails this check, which IS the "a worktree cannot
// merge itself" refusal; mergeFeatureWorktree's own isOrdinaryCheckout(mainRoot)
// re-check is the belt-and-braces layer, exactly like createFeatureWorktree's).
// verifyCommand is resolved HERE (readConfig(mainRoot).commands.test ||
// commands.verify — ci-owned-verify D5/cov-4) and passed down as a plain
// option, per worktree-store.mjs's zero-deps-beyond-node-builtins module
// contract (see mergeFeatureWorktree's header comment).
// companionEndCommand (worktree-companion-hook) is resolved the same way,
// from commands.worktree_companion_end — see teardownCompanionIfPresent's
// own doc comment in worktree-store.mjs for why it runs unconditionally.
async function handleWorktreeMerge(_root, flags) {
  const id = requireFlag(flags, 'id');
  const cleanup = flags.cleanup === true;
  let resolution;
  try {
    resolution = resolveRoots(process.cwd());
  } catch (error) {
    throw new Error(
      `"bee worktree merge" must be run from inside the main checkout (not a linked worktree): ${error instanceof Error ? error.message : String(error)}`,
    );
  }
  if (resolution.worktreeResolution !== 'ordinary' || !resolution.workRoot) {
    throw new Error(
      `"bee worktree merge" must be run from inside the main checkout, not a "${resolution.worktreeResolution}" checkout — a worktree, including the one being merged, cannot merge itself.`,
    );
  }
  const mainRoot = resolution.workRoot;
  const configCommands = readConfig(mainRoot).commands;
  // ci-owned-verify D5 (cov-4): prefer commands.test (the impacted/dev-loop
  // command) over commands.verify (the full CI-owned run) as the
  // semantic-conflict gate — impacted over the merge diff; the full pass
  // lands in CI on push. Falls back to commands.verify when commands.test
  // is unset, so a host project that only configured commands.verify keeps
  // today's behavior unchanged.
  // no-test-repos D1/D2 (decision 55b951e1): the resolved command may be the
  // literal sentinel string — it must NEVER be spawned as a shell command.
  // Map it to `undefined` so the existing verifySkipped loud-warning path
  // (mergeFeatureWorktree: verify: verifyCommand ? 'green' : 'skipped') fires
  // exactly as it already does for a repo with no verify command configured
  // at all — one gate, no new code path.
  const resolvedVerifyCommand = configCommands.test || configCommands.verify || undefined;
  const verifyCommand = resolvedVerifyCommand === NO_TEST_SENTINEL ? undefined : resolvedVerifyCommand;
  // worktree-companion-hook: no --with-companion flag here — unlike `new`,
  // where a bare worktree is a real, valid choice, `merge` needs none: the
  // worktree's own .bee/companion-session.json marker (written by `new
  // --with-companion`) is the only signal teardownCompanionIfPresent needs.
  // Resolved unconditionally (cheap) so a worktree WITH a marker still gets
  // torn down even if this specific merge invocation forgets to opt in to
  // anything — there is nothing to opt in to.
  const companionEndCommand = configCommands.worktree_companion_end || undefined;
  // xwh-2: mergeFeatureWorktree is now async (its --cleanup path awaits
  // releaseAllForHolder) — the dispatcher already does `await handler(...)`
  // for every command, so this only needed the local await.
  const mergeResultValue = await mergeFeatureWorktree(mainRoot, { id, cleanup, verifyCommand, companionEndCommand });

  const lines = [];
  if (mergeResultValue.ok) {
    // issues-46-53 D3 (#47): the no-op branch used to end here, printing one
    // headline and nothing else — so a `--cleanup` that had just been silently
    // dropped by the library was invisible in the text output too. The
    // companion/warning/cleanup reporting below is now SHARED by both ok
    // outcomes, so whatever cleanup actually did (or was suggested) is always
    // said out loud.
    if (mergeResultValue.code === 'ALREADY_UP_TO_DATE') {
      lines.push(`Worktree ${id} (branch ${mergeResultValue.branch}) is already up to date with ${mainRoot} — nothing to merge; no commit was made.`);
    } else {
      lines.push(`Merged worktree ${id} (branch ${mergeResultValue.branch}) into ${mainRoot}.`);
      lines.push(`  verify: ${mergeResultValue.verify}`);
    }
    if (mergeResultValue.companion) {
      lines.push(
        mergeResultValue.companion.warning
          ? `  companion: WARNING — ${mergeResultValue.companion.warning}`
          : `  companion: ended${mergeResultValue.companion.sessionId ? ` (session ${mergeResultValue.companion.sessionId})` : ''}.`,
      );
    }
    if (mergeResultValue.warning) {
      lines.push(`  WARNING (${mergeResultValue.warning.code}): ${mergeResultValue.warning.message}`);
    }
    if (mergeResultValue.cleanup) {
      lines.push(
        mergeResultValue.cleanup.ok
          ? '  cleanup: worktree removed, branch deleted.'
          : `  cleanup: refused (${mergeResultValue.cleanup.code}) — ${mergeResultValue.cleanup.reason}`,
      );
      if (mergeResultValue.cleanup.warning) {
        lines.push(`  WARNING: ${mergeResultValue.cleanup.warning}`);
      }
    } else if (mergeResultValue.cleanup_suggested_command) {
      lines.push(`  cleanup: run \`${mergeResultValue.cleanup_suggested_command}\` when ready.`);
    }
  } else if (mergeResultValue.code === 'MERGE_VERIFY_RED') {
    lines.push(`Merge of worktree ${id} (branch ${mergeResultValue.branch}) was TEXTUALLY CLEAN, but verify is RED (semantic-conflict alarm).`);
    lines.push(`The merge was aborted — ${mainRoot} was left byte-untouched; no merge commit exists. Fix-first before release, then retry the merge.`);
    lines.push('--- verify output tail ---');
    lines.push(mergeResultValue.output_tail);
  } else {
    lines.push(`Merge of worktree ${id} hit a textual conflict — the merge was aborted and ${mainRoot} was left byte-untouched; bee does not auto-resolve a textual conflict. Resolve it in the worktree and retry.`);
  }
  return { result: mergeResultValue, text: lines.join('\n'), exitCode: mergeResultValue.ok ? 0 : 1 };
}

function handleWorktreeList(root, _flags) {
  const mainRoot = resolveMainRoot(root);
  const mainStoreRoot = path.join(mainRoot, '.bee');
  const grants = listGrants(mainStoreRoot);
  const ids = Object.keys(grants).filter((id) => grants[id] === true);
  const text = ids.length ? ids.map((id) => `${id} (granted)`).join('\n') : 'No worktree grants.';
  return { result: { grants, main_root: mainRoot }, text };
}

async function handleWorktreeUnregister(root, flags) {
  const mainRoot = resolveMainRoot(root);
  const mainStoreRoot = path.join(mainRoot, '.bee');
  let id = flags.id ? String(flags.id) : null;
  if (!id) {
    let resolution;
    try {
      resolution = resolveRoots(process.cwd());
    } catch (error) {
      throw new Error(
        `--id not given, and the current directory is not a linked worktree: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
    if (resolution.worktreeResolution !== 'linked-valid' || !resolution.id) {
      throw new Error('--id not given, and the current directory is not a linked worktree — pass --id explicitly.');
    }
    id = resolution.id;
  }
  // hardening-4b: removeGrant is now withStoreLock-wrapped (async).
  await removeGrant(mainStoreRoot, id);
  return { result: { ok: true, id, main_root: mainRoot }, text: `Removed worktree grant for id ${id}.` };
}

// ─── herding: enable/disable/status the dispatch loop's owner enable marker
// (herding-dispatch-lock-toggle, decisions D1-D5). lib/herding.mjs owns the
// filesystem operation and mirrors dispatch-interlock.mjs's own git-common-dir
// resolution and ENABLE_BASENAME exactly, so this group and the read-only
// interlock always agree on the same file. D4: these verbs are a convenience
// for the human owner's own terminal action only — never called from
// dispatch-interlock.mjs, bootstrap, dispatch, merge, or any other bee
// automation/skill/agent code. D5: no runtime guard (no TTY check, not
// excluded from --help --json) — convention-only safety, same level as
// today's manual touch/rm. ──────────────────────────────────────────────────
function handleHerdingEnable(_root, _flags) {
  const result = enableHerding();
  return { result, text: `Enabled bee-herding dispatch: ${result.marker}` };
}

function handleHerdingDisable(_root, _flags) {
  const result = disableHerding();
  return { result, text: `Disabled bee-herding dispatch: ${result.marker}` };
}

function handleHerdingStatus(_root, _flags) {
  const result = herdingStatus();
  const text = result.enabled
    ? `enabled — owner marker present (${result.marker})`
    : `disabled — no owner marker at ${result.marker}`;
  return { result, text };
}

// ─── tmp sweep (tree-hygiene th-4, CONTEXT D1/D2) ──────────────────────────
// `bee tmp sweep` — the broom for the one canonical scratch home (.bee/tmp/
// and .bee/spikes/). All safety (containment, symlink-escape refusal) and
// target-selection logic lives in lib/scratch.mjs; this handler is
// presentation + the "no default purge" refusal policy only. Refuses (typed,
// zero mutation) unless at least one of --feature/--before/--all/--dry-run is
// given — an all-defaults call would otherwise silently pick a target set,
// which is exactly the "no default purge" discipline `decisions archive`
// already established for its own mandatory --before.
function handleTmpSweep(root, flags) {
  const feature = flags.feature !== undefined ? String(flags.feature) : undefined;
  const before = flags.before !== undefined ? String(flags.before) : undefined;
  const all = flags.all === true;
  const dryRun = flags['dry-run'] === true;
  if (!feature && !before && !all && !dryRun) {
    throw new Error(
      'tmp sweep requires at least one of --feature/--before/--all/--dry-run — no default purge (same discipline as `decisions archive`). ' +
        'FIX: pass --dry-run to preview the default (closed/absent-feature) target set, --feature <slug> to target one feature explicitly (even a live one), --before <ISO> to age-gate scratch with no feature/lane record, or --all to clear everything.',
    );
  }
  const result = runSweep(root, { feature, before, all, dryRun });
  const verb = dryRun ? 'Would remove' : 'Removed';
  const text = `${verb} ${result.removed.length} scratch entr(y|ies) (${result.bytes_freed} bytes, ${result.files_freed} files) from .bee/tmp/ and .bee/spikes/.`;
  return { result, text };
}

// config (ao-2ai-1) — reads the RAW .bee/config.json (readJson fallback
// `undefined`, never `null`: a missing file is the normal, silent "no config
// yet" state every other config reader tolerates — only content that was
// actually read and is unusable counts as a problem, per validateModelsConfig's
// own undefined-vs-null contract) and runs the shared validator so a
// malformed/prompt-less/unsafe cli-tier value gets a LOUD, non-zero-exit
// refusal instead of today's silent revert to the seeded default.
function handleConfigValidate(root, _flags) {
  const raw = readRawConfigForValidation(root);
  // W3 (ao-3b-2, AO12): the same drift advisory `bee status` surfaces joins
  // the models-config problems here too, one CLI verb covering both checks.
  const problems = [...validateModelsConfig(raw), ...validateAgentFilesDrift(root, raw)];
  const result = { ok: problems.length === 0, problem_count: problems.length, problems };
  const text =
    problems.length === 0
      ? 'config validate: OK — no malformed/prompt-less/unsafe cli-tier config or rendered-agent drift found.'
      : problems
          .map(
            (p) =>
              `[${p.code}]${p.runtime ? ` models.${p.runtime}.${p.slot}:` : ''} ${p.message}`,
          )
          .join('\n');
  return { result, text, exitCode: problems.length === 0 ? 0 : 1 };
}

// ─── config get/set/unset (GitHub #15) ────────────────────────────────────
// So a config value (product_root, guards.idle_gate, gate_bypass, …) is changed
// through a validated CLI instead of hand-editing .bee/config.json — the same
// "everything through the CLI" contract every other .bee file already has.

// hardening-8 (config overlay): --local redirects every config get/set/unset
// verb at the machine-local overlay (.bee/config.local.json, gitignored)
// instead of the tracked .bee/config.json. Omitting --local is byte-identical
// to today (D4 zero-flag parity) — every existing caller is unaffected.
function configFilePath(root, { local = false } = {}) {
  return local ? localConfigPath(root) : path.join(root, '.bee', 'config.json');
}

// Read the RAW config object for editing (not readConfig — that normalizes and
// fills defaults, which would balloon the file). Refuses on a present-but-broken
// file so a set/unset never silently clobbers an unparseable config and loses it.
function readRawConfigForEdit(root, { local = false } = {}) {
  const file = configFilePath(root, { local });
  if (!fs.existsSync(file)) return {};
  const raw = readJson(file, undefined);
  if (raw === undefined || raw === null) {
    throw new Error(
      `config: ${path.relative(root, file)} exists but is not valid JSON — fix it before "config set"/"config unset" (refusing to overwrite and lose your config).`,
    );
  }
  if (typeof raw !== 'object' || Array.isArray(raw)) {
    throw new Error(`config: ${path.relative(root, file)} is not a JSON object.`);
  }
  return raw;
}

// Coerce a CLI value string: JSON when it parses (true/false/numbers/objects),
// else the literal string. So `set guards.idle_gate false` stores boolean false
// and `set product_root repo` stores the string "repo". --string forces a string.
function coerceConfigValue(raw, asString) {
  if (asString) return String(raw);
  try {
    return JSON.parse(String(raw));
  } catch {
    return String(raw);
  }
}

function getConfigAtPath(obj, keyPath) {
  let cur = obj;
  for (const part of keyPath.split('.')) {
    if (cur == null || typeof cur !== 'object') return undefined;
    cur = cur[part];
  }
  return cur;
}

function setConfigAtPath(obj, keyPath, value) {
  const parts = keyPath.split('.');
  let cur = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i];
    if (cur[p] == null || typeof cur[p] !== 'object' || Array.isArray(cur[p])) cur[p] = {};
    cur = cur[p];
  }
  cur[parts[parts.length - 1]] = value;
}

function unsetConfigAtPath(obj, keyPath) {
  const parts = keyPath.split('.');
  const chain = []; // [container, key] pairs, so empty parents can be pruned
  let cur = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    const p = parts[i];
    if (cur[p] == null || typeof cur[p] !== 'object' || Array.isArray(cur[p])) return false;
    chain.push([cur, p]);
    cur = cur[p];
  }
  const leaf = parts[parts.length - 1];
  if (!Object.prototype.hasOwnProperty.call(cur, leaf)) return false;
  delete cur[leaf];
  // Prune ancestors left empty by the delete (removing guards.idle_gate must not
  // leave a stray "guards": {} — that is the config clutter #15 is about).
  for (let i = chain.length - 1; i >= 0; i--) {
    const [parent, key] = chain[i];
    const child = parent[key];
    if (child && typeof child === 'object' && !Array.isArray(child) && Object.keys(child).length === 0) {
      delete parent[key];
    } else {
      break;
    }
  }
  return true;
}

// Refuse a write that INTRODUCES a new models/cli-safety problem (the same check
// `config validate` runs). Pre-existing, unrelated problems never block an
// unrelated set — only newly-introduced ones do.
function refuseIfNewConfigProblem(verb, before, after) {
  const key = (p) => `${p.code}|${p.runtime || ''}|${p.slot || ''}`;
  const had = new Set(before.map(key));
  const introduced = after.filter((p) => !had.has(key(p)));
  if (introduced.length > 0) {
    throw new Error(
      `config ${verb}: refusing to write — this would make the config invalid:\n` +
        introduced.map((p) => `[${p.code}] ${p.message}`).join('\n'),
    );
  }
}

// D2 (intake-gate-git-exemption): if `key` is present in the TRACKED
// config.json (regardless of whether it's ALSO in the overlay now), warn
// that it is stuck there — guards.*/hooks.* keys are never written to or
// removed from the tracked file by this CLI anymore, so a value found there
// is either legacy or was hand-edited back in. Returns null when the key
// isn't a local-only namespace, when the tracked file is unreadable/absent,
// or when the key simply isn't present in it — never throws.
function trackedKeyWarningIfPresent(root, key) {
  if (!isLocalOnlyConfigKey(key)) return null;
  let trackedRaw;
  try {
    trackedRaw = readRawConfigForEdit(root, { local: false });
  } catch {
    return null;
  }
  return getConfigAtPath(trackedRaw, key) !== undefined ? trackedLocalOnlyKeyWarning(key) : null;
}

function handleConfigGet(root, flags) {
  const key = requireFlag(flags, 'key');
  const explicitLocal = flags.local === true;
  // guards.*/hooks.* without an explicit --local: surface the ACTUAL
  // effective value (overlay wins over tracked, same precedence readConfig()
  // uses) instead of a single raw file — otherwise `config get` would read
  // "not set" immediately after a `config set` that (by D2) always lands in
  // the overlay for these namespaces. --local still means exactly what it
  // always has: read the overlay file only.
  if (!explicitLocal && isLocalOnlyConfigKey(key)) {
    const overlayValue = getConfigAtPath(readRawConfigForEdit(root, { local: true }), key);
    const trackedRaw = readRawConfigForEdit(root, { local: false });
    const trackedValue = getConfigAtPath(trackedRaw, key);
    const present = overlayValue !== undefined || trackedValue !== undefined;
    const value = overlayValue !== undefined ? overlayValue : trackedValue;
    const local = overlayValue !== undefined;
    const warning = trackedValue !== undefined ? trackedLocalOnlyKeyWarning(key) : null;
    return {
      result: { key, present, value: present ? value : null, local, warning },
      text:
        (present ? `${key} = ${JSON.stringify(value)}` : `config get: "${key}" is not set.`) +
        (warning ? `\n${warning}` : ''),
    };
  }
  const value = getConfigAtPath(readRawConfigForEdit(root, { local: explicitLocal }), key);
  const present = value !== undefined;
  return {
    result: { key, present, value: present ? value : null, local: explicitLocal, warning: null },
    text: present ? `${key} = ${JSON.stringify(value)}` : `config get: "${key}" is not set.`,
  };
}

function handleConfigSet(root, flags) {
  const key = requireFlag(flags, 'key');
  const value = coerceConfigValue(requireFlag(flags, 'value'), flags.string === true);
  // D2: guards.*/hooks.* ALWAYS route to the local overlay, regardless of
  // --local — a machine-local safety toggle must be structurally incapable
  // of reaching the tracked, git-committed config.json (incident a7d2069).
  const forcedLocal = isLocalOnlyConfigKey(key);
  const local = forcedLocal || flags.local === true;
  const config = readRawConfigForEdit(root, { local });
  // The models-config cli-safety guard only ever applies to the TRACKED
  // config (the overlay is for machine-local values like dogfood_repos, not
  // model/cli wiring) — an overlay write skips it rather than comparing a
  // local-only object against the tracked validator's expectations.
  if (!local) {
    const before = validateModelsConfig(readRawConfigForValidation(root));
    setConfigAtPath(config, key, value);
    refuseIfNewConfigProblem('set', before, validateModelsConfig(config));
  } else {
    setConfigAtPath(config, key, value);
  }
  writeJsonAtomic(configFilePath(root, { local }), config);
  const warning = forcedLocal ? trackedKeyWarningIfPresent(root, key) : null;
  return {
    result: { key, value, local, warning },
    text: `config set${local ? ' --local' : ''}: ${key} = ${JSON.stringify(value)}` + (warning ? `\n${warning}` : ''),
  };
}

function handleConfigUnset(root, flags) {
  const key = requireFlag(flags, 'key');
  // D2: same forced routing as set — guards.*/hooks.* unset never touches
  // the tracked file, even to remove a legacy value there (never auto-edit).
  const forcedLocal = isLocalOnlyConfigKey(key);
  const local = forcedLocal || flags.local === true;
  const config = readRawConfigForEdit(root, { local });
  const before = !local ? validateModelsConfig(readRawConfigForValidation(root)) : null;
  const removed = unsetConfigAtPath(config, key);
  const warning = forcedLocal ? trackedKeyWarningIfPresent(root, key) : null;
  if (!removed) {
    return {
      result: { key, removed: false, local, warning },
      text: `config unset${local ? ' --local' : ''}: "${key}" was not set (no change).` + (warning ? `\n${warning}` : ''),
    };
  }
  if (!local) {
    refuseIfNewConfigProblem('unset', before, validateModelsConfig(config));
  }
  writeJsonAtomic(configFilePath(root, { local }), config);
  return {
    result: { key, removed: true, local, warning },
    text: `config unset${local ? ' --local' : ''}: removed "${key}".` + (warning ? `\n${warning}` : ''),
  };
}

// ─── doctor (codex-native-runtime-v2 D11): fail-closed runtime health report
// ─────────────────────────────────────────────────────────────────────────
// Every row states its own evidence and status (ok/warn/unknown/unsupported);
// overall_status is a THREE-state verdict (g22-3, D4): 'blocked' when any
// MECHANICAL row marked `blocking: true` is not-ok (hooks file missing,
// capability-baseline drift, handlers unresolvable, skills missing/warn);
// 'degraded' when every mechanical row is ok but codex's four trust rows
// (marked `degrades: true`, never `blocking` anymore) are still structurally
// unknown and no valid attestation covers them; 'ready' only when mechanical
// rows are all ok AND (codex) a valid attestation exists, or (claude, which
// has no trust-unknown rows) mechanical green alone — no attestation
// concept on claude, deliberately kept simple. File presence alone never
// grants 'ready'. This whole command performs ZERO writes EXCEPT `doctor
// attest`, which records a static attestation file on request (D5-REVISED)
// — `doctor` (no verb) itself still performs zero writes, every helper below
// still only reads.

// D6: the codex-cli version the capability matrix (docs/history/
// codex-native-runtime-v2/reports/capability-matrix.md) actually probed.
// Trust/discovery verdicts below are conclusions about THIS version only —
// a live codex whose --version differs is unprobed territory, never
// silently asserted as if it shared the same F1 capability-matrix row.
const PROBED_CODEX_VERSION = '0.145.0';

const CODEX_DOCTOR_TRUST_UNKNOWN_REASON =
  `codex-cli ${PROBED_CODEX_VERSION} exposes no machine-readable hook-discovery/trust surface — \`codex doctor --json\` reports no hook/trust/agent rows (capability matrix row F1); trust state lives only in the interactive \`/hooks\` TUI, which is not machine-readable.`;

// `codex --version` prints a full label ("codex-cli 0.145.0"), not a bare
// semver — extract just the number for comparison against
// PROBED_CODEX_VERSION so this never false-mismatches on the label text.
function doctorExtractVersionNumber(raw) {
  if (typeof raw !== 'string') return null;
  const match = raw.match(/(\d+\.\d+\.\d+)/);
  return match ? match[1] : null;
}

// D6: when the live codex --version does not match PROBED_CODEX_VERSION, the
// trust rows must not assert the probed version's conclusions — they report
// `unprobed_version` instead (evidence carries that literal token so callers
// can grep for it), naming the mismatch and asking for a re-probe rather
// than a silent guess either way (ready or blocked). An unresolved live
// version (codex not on PATH) cannot be proven to differ, so it keeps the
// default (probed) wording rather than a speculative mismatch claim.
function doctorCodexTrustUnknownReason(liveVersionRaw) {
  const liveVersionNumber = doctorExtractVersionNumber(liveVersionRaw);
  if (liveVersionNumber && liveVersionNumber !== PROBED_CODEX_VERSION) {
    return (
      `unprobed_version: live codex --version reports "${liveVersionRaw}" (${liveVersionNumber}), which has not been ` +
      `capability-probed (only ${PROBED_CODEX_VERSION} has — capability matrix row F1); re-run the probe before ` +
      'trusting any hook-discovery/trust conclusion for this version. Trust state lives only in the interactive ' +
      '`/hooks` TUI, which is not machine-readable.'
    );
  }
  return CODEX_DOCTOR_TRUST_UNKNOWN_REASON;
}

function doctorRow(row, status, value, evidence, extra = {}) {
  return { row, status, value, evidence, ...extra };
}

function doctorSafeReadText(file) {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }
}

function doctorSafeReadJson(file) {
  const text = doctorSafeReadText(file);
  if (text === null) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function doctorExtractHookCommands(hooksJson) {
  const commands = [];
  const events = hooksJson && hooksJson.hooks && typeof hooksJson.hooks === 'object' ? hooksJson.hooks : {};
  for (const matchers of Object.values(events)) {
    if (!Array.isArray(matchers)) continue;
    for (const matcher of matchers) {
      const list = matcher && Array.isArray(matcher.hooks) ? matcher.hooks : [];
      for (const h of list) {
        if (h && typeof h.command === 'string') commands.push(h.command);
      }
    }
  }
  return commands;
}

// Every rendered hook command (both the repo and plugin targets) execs
// "node .../hooks/<file>.mjs" — pull every referenced filename regardless of
// which absolute prefix precedes it (repo target resolves it at runtime from
// $r, so no static prefix is provable here; only the filename is).
function doctorHookHandlerFilenames(commands) {
  const files = new Set();
  const re = /hooks\/([A-Za-z0-9_.-]+\.mjs)/g;
  for (const command of commands) {
    let match;
    while ((match = re.exec(command)) !== null) files.add(match[1]);
  }
  return [...files];
}

export function doctorCodexVersion() {
  try {
    const result = spawnSync('codex', ['--version'], { encoding: 'utf8', timeout: 5000 });
    if (result.error || typeof result.status !== 'number' || result.status !== 0) {
      return doctorRow(
        'codex_version',
        'warn',
        null,
        'codex binary not found on PATH (or exited non-zero) — cannot report an installed version.',
      );
    }
    const value = (result.stdout || '').trim() || null;
    return doctorRow(
      'codex_version',
      value ? 'ok' : 'warn',
      value,
      value ? `codex --version -> ${value}` : 'codex --version produced no output.',
    );
  } catch (error) {
    return doctorRow(
      'codex_version',
      'warn',
      null,
      `codex --version threw: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}

// Mechanical/blocking (D4): a missing hooks file is a MECHANICAL fact, not a
// codex-runtime-trust unknown — it blocks readiness outright rather than
// merely degrading it.
function doctorHooksFilePresent(root) {
  const present = fs.existsSync(path.join(root, '.codex', 'hooks.json'));
  return doctorRow(
    'hooks_file_present',
    present ? 'ok' : 'warn',
    present,
    present ? '.codex/hooks.json exists.' : '.codex/hooks.json is missing.',
    { blocking: true },
  );
}

// Byte-compares the LIVE repo-fallback hooks file against the sha256 the
// onboarding ledger recorded at install time (managed.repo_hooks) — the same
// honest-drift discipline bee_status already uses for vendored runtime files
// (no second hashing implementation; reuses lib/fsutil.mjs's hashFile).
// Mechanical/blocking (D4): every non-ok status here (warn OR unknown) is a
// file-state fact, never a trust unknown — all of them block readiness.
function doctorCapabilityBaselineMatch(root) {
  const hooksPath = path.join(root, '.codex', 'hooks.json');
  if (!fs.existsSync(hooksPath)) {
    return doctorRow('capability_baseline_match', 'warn', false, '.codex/hooks.json is absent — cannot compare against the recorded baseline.', { blocking: true });
  }
  const onboarding = readOnboarding(root);
  const recorded = onboarding?.managed?.repo_hooks?.['.codex/hooks.json'] ?? null;
  if (!recorded) {
    return doctorRow('capability_baseline_match', 'unknown', null, 'no recorded baseline hash in .bee/onboarding.json managed.repo_hooks — run onboarding.', { blocking: true });
  }
  const live = hashFile(hooksPath);
  if (live !== recorded) {
    return doctorRow(
      'capability_baseline_match',
      'warn',
      false,
      `live .codex/hooks.json sha256 (${live}) does not match the recorded baseline (${recorded}).`,
      { fix: 'Re-render via self-onboard sync: node skills/bee-hive/scripts/onboard_bee.mjs --repo-root . --apply', blocking: true },
    );
  }
  return doctorRow('capability_baseline_match', 'ok', true, `live .codex/hooks.json byte-matches the recorded baseline (${live}).`, { blocking: true });
}

// D4 re-class: these four rows used to carry `blocking: true` (holding
// overall_status at not_ready outright). They now carry `degrades: true`
// instead — structurally unknown trust state no longer BLOCKS readiness by
// itself, it only prevents 'ready' until a valid attestation (D5-REVISED,
// `doctor attest`) covers it; `degraded_reason` is the short, user-facing
// instruction (review /hooks) distinct from the long evidence string.
function doctorCodexTrustUnknownRows(liveVersion) {
  const reason = doctorCodexTrustUnknownReason(liveVersion);
  return ['hooks_discovered', 'hooks_trusted', 'project_trust', 'pending_hook_review'].map((row) =>
    doctorRow(row, 'unknown', null, reason, {
      degrades: true,
      degraded_reason: 'trust state is not machine-verifiable — review it yourself via the interactive `/hooks` TUI, then run `bee doctor attest --runtime codex` once satisfied.',
    }),
  );
}

// Ported from skills/bee-hive/scripts/onboard_bee.mjs::repoOwnsHookCatalog
// (a bare reference here would ReferenceError — bee.mjs and onboard_bee.mjs
// are separate files, not mirrors of each other). Used only for evidence
// labeling below: which install topology produced the resolution, never to
// change which locations are checked.
function repoOwnsHookCatalog(root) {
  return fs.existsSync(path.join(root, 'hooks', 'catalog.mjs'));
}

// GH #22 P1-1: a normal host install renders hook commands as
// "$r"/.bee/bin/hooks/<f>.mjs and has NO root hooks/ dir at all — only bee's
// own source checkout (and the conformance fixture that mimics it) also has
// a root hooks/. Checking a single hard-coded dir (formerly always "hooks")
// reported every healthy hybrid host install as broken. Mirrors the
// Claude-side resolver precedent (doctorClaudeHandlersResolvable below):
// resolvable = file exists at .bee/bin/hooks/<f> OR hooks/<f>; the evidence
// names WHICH location resolved each file (or that neither did).
// Mechanical/blocking (D4): every branch below is a file-resolution fact.
function doctorHookHandlersResolvable(root) {
  const hooksPath = path.join(root, '.codex', 'hooks.json');
  const hooksJson = doctorSafeReadJson(hooksPath);
  if (!hooksJson) {
    return doctorRow('hook_handlers_resolvable', 'warn', null, `${hooksPath} is missing or unparsable — no command paths to resolve.`, { blocking: true });
  }
  const commands = doctorExtractHookCommands(hooksJson);
  const files = doctorHookHandlerFilenames(commands);
  if (files.length === 0) {
    return doctorRow('hook_handlers_resolvable', 'warn', [], 'no hooks/*.mjs command references found in .codex/hooks.json.', { blocking: true });
  }
  const topology = repoOwnsHookCatalog(root)
    ? 'repo owns hook catalog -> source-checkout topology'
    : 'host topology (.bee/bin/hooks)';
  const resolvedAt = [];
  const missing = [];
  for (const f of files) {
    if (fs.existsSync(path.join(root, '.bee', 'bin', 'hooks', f))) {
      resolvedAt.push(`${f} -> .bee/bin/hooks/`);
    } else if (fs.existsSync(path.join(root, 'hooks', f))) {
      resolvedAt.push(`${f} -> hooks/`);
    } else {
      missing.push(f);
    }
  }
  if (missing.length) {
    return doctorRow(
      'hook_handlers_resolvable',
      'warn',
      files,
      `${topology}; missing handler file(s) under .bee/bin/hooks/ or hooks/: ${missing.join(', ')}.`,
      { fix: `Restore ${missing.join(', ')} under .bee/bin/hooks/ (or hooks/ in a source checkout), or re-render .codex/hooks.json from the catalog.`, blocking: true },
    );
  }
  return doctorRow(
    'hook_handlers_resolvable',
    'ok',
    files,
    `${topology}; ${resolvedAt.length} handler file(s) resolved: ${resolvedAt.join(', ')}.`,
    { blocking: true },
  );
}

// Provable ONLY against a session-start boundary, which a fresh bee.mjs
// process never has — never inferred from recent log activity; the newest
// row timestamp(s) are surfaced as context only, not as proof of observation.
function doctorHooksObservedThisSession(root) {
  const text = doctorSafeReadText(path.join(root, '.bee', 'logs', 'hooks.jsonl'));
  if (!text || !text.trim()) {
    return doctorRow('hooks_observed_this_session', 'unknown', null, 'no .bee/logs/hooks.jsonl on disk yet — no session-start boundary to test against.');
  }
  const lines = text.split(/\r?\n/).filter(Boolean);
  const recent = lines
    .slice(-3)
    .map((line) => {
      try {
        return JSON.parse(line).ts;
      } catch {
        return null;
      }
    })
    .filter(Boolean);
  return doctorRow(
    'hooks_observed_this_session',
    'unknown',
    null,
    `bee.mjs runs as a fresh process with no session-start boundary to test against — never inferred from recent log activity alone. Newest logged row timestamp(s) as context only: ${recent.join(', ') || '(none)'}.`,
  );
}

function doctorPermissionModeCodex(root) {
  const text = doctorSafeReadText(path.join(root, '.codex', 'config.toml'));
  const match = text ? text.match(/approval_policy\s*=\s*"([^"]*)"/) : null;
  const configured = match ? match[1] : null;
  return doctorRow(
    'permission_mode',
    'unknown',
    { configured, observed: null },
    configured
      ? `configured approval_policy = "${configured}" in .codex/config.toml; observed permission_mode has no doctor-time runtime surface on codex-cli 0.144.4 (it only appears inside a fired hook envelope, matrix row C2).`
      : 'no approval_policy configured in .codex/config.toml; observed permission_mode has no doctor-time runtime surface.',
  );
}

function doctorHookSourcesCodex(root) {
  const repoPresent = fs.existsSync(path.join(root, '.codex', 'hooks.json'));
  const pluginProjectionCheckedIn = fs.existsSync(path.join(root, 'hooks', 'hooks.json'));
  // D5/#54 item 8: hooks/claude-hooks.json (the plugin.json-declared Claude
  // manifest) is a distinct rendered target from hooks/hooks.json (the Codex
  // plugin projection) — named separately here so the two Claude-shaped
  // renders are never conflated in the configured listing or the evidence text.
  const claudeHooksManifestCheckedIn = fs.existsSync(path.join(root, 'hooks', 'claude-hooks.json'));
  const configured = {
    repo: repoPresent,
    plugin_projection_checked_in: pluginProjectionCheckedIn,
    claude_hooks_manifest_checked_in: claudeHooksManifestCheckedIn,
  };
  // D5: both-present is a distinct, named risk state — never silently folded
  // into the same "ok" sentence as the single-source case. Verdict semantics
  // (ok/warn from repoPresent alone) and active:unknown honesty stay exactly
  // as before; this only widens what the evidence text says (hook-runtime
  // knowledge: hook-source-exclusivity B14 — exactly one source is active).
  const dualSourceNote = repoPresent && pluginProjectionCheckedIn
    ? ` Both hooks/hooks.json (plugin projection) and .codex/hooks.json (repo fallback) are present on disk, so two hook sources exist; exactly-one-active is the law (hook-source-exclusivity B14). The current premise is capability matrix row B1 — plugin hooks are not-observed on codex-cli ${PROBED_CODEX_VERSION} — and this premise must be re-proved whenever the probed codex version changes.`
    : '';
  const claudeHooksNote = claudeHooksManifestCheckedIn
    ? ` hooks/claude-hooks.json (the plugin.json-declared Claude manifest) is a distinct rendered target from hooks/hooks.json and is tracked separately in this row's configured listing.`
    : '';
  return doctorRow(
    'hook_sources',
    repoPresent ? 'ok' : 'warn',
    { configured, active: 'unknown' },
    (repoPresent
      ? `repo-fallback .codex/hooks.json is configured and is the sole exercisable source today (plugin hooks not-observed on codex-cli ${PROBED_CODEX_VERSION}, capability matrix row B1); which source is actively loaded has no runtime surface, so "active" stays unknown rather than inferred from presence.`
      : 'no repo-fallback .codex/hooks.json found; nothing configured to load.')
      + dualSourceNote
      + claudeHooksNote,
  );
}

// D7/g22-4: the bee-render/2 sidecar schema this deep audit expects. bee.mjs
// cannot import skills/bee-hive/scripts/onboard_bee.mjs (separate
// distribution target — templates/bee.mjs and .bee/bin/bee.mjs ship without
// the scripts/ tree; see the mirror-discipline note at the top of this
// file), so this literal and the digest algorithm below are hand-mirrors of
// onboard_bee.mjs's RENDER_SCHEMA / skillDigest / walkSkillTree — keep them
// in lockstep by hand when either side changes.
const SKILL_RENDER_SCHEMA_V2 = 'bee-render/2';

// Walks one installed skill dir exactly like onboard_bee.mjs's
// walkSkillTree(dir) (no transform — reading already-rendered bytes off
// disk): a symlink or unsupported entry anywhere blocks the whole walk
// (never partially hashed), every plain file is hashed by its raw bytes.
// Returns { blocked } or { sha256 } — sha256 is
// sha256(JSON.stringify(sorted [relPath, sha256(fileBytes)] pairs)), the
// same fold onboard_bee.mjs's skillDigest(manifestFingerprint(files)) uses.
function doctorWalkSkillDir(dirAbs) {
  const files = [];
  let blocked = null;
  const walk = (dir, relPrefix) => {
    let entries;
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      blocked = { path: relPrefix || '.', reason: 'unreadable directory' };
      return;
    }
    entries.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
    for (const entry of entries) {
      if (blocked) return;
      const rel = relPrefix ? `${relPrefix}/${entry.name}` : entry.name;
      const abs = path.join(dir, entry.name);
      if (entry.isSymbolicLink()) {
        blocked = { path: rel, reason: 'symlink' };
        return;
      }
      if (entry.isDirectory()) {
        walk(abs, rel);
      } else if (entry.isFile()) {
        files.push([rel, crypto.createHash('sha256').update(fs.readFileSync(abs)).digest('hex')]);
      } else {
        blocked = { path: rel, reason: 'unsupported entry type' };
        return;
      }
    }
  };
  walk(dirAbs, '');
  if (blocked) return { blocked };
  files.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  return { sha256: crypto.createHash('sha256').update(JSON.stringify(files)).digest('hex') };
}

// Deep-audits the installed skill set at `dir` against a bee-render/2
// sidecar: every expected skill dir (sidecar.skills[]) must be present, no
// unexpected plain bee-* stray dir may exist, and every expected skill's
// recomputed content digest must match. A blocked walk (e.g. a symlink
// inside an installed skill dir) counts as drifted — it can never be proven
// to match.
function doctorDeepAuditSkills(dir, sidecar) {
  const expected = new Map((Array.isArray(sidecar.skills) ? sidecar.skills : []).map((s) => [s.name, s.sha256]));
  const installedNames = fs.existsSync(dir)
    ? fs
        .readdirSync(dir, { withFileTypes: true })
        .filter((e) => e.isDirectory() && /^bee-/.test(e.name))
        .map((e) => e.name)
    : [];
  const installed = new Set(installedNames);
  const missing = [...expected.keys()].filter((name) => !installed.has(name)).sort();
  const stray = installedNames.filter((name) => !expected.has(name)).sort();
  const drifted = [];
  for (const [name, expectedHash] of expected) {
    if (!installed.has(name)) continue; // already reported as missing
    const walk = doctorWalkSkillDir(path.join(dir, name));
    if (walk.blocked || walk.sha256 !== expectedHash) {
      drifted.push(name);
    }
  }
  drifted.sort();
  return { ok: missing.length === 0 && stray.length === 0 && drifted.length === 0, missing, stray, drifted };
}

// Mechanical/blocking (D4) — shared by both runtimes (codex's .agents/skills,
// claude's .claude/skills). g22-4/D7: a v2 sidecar drives a DEEP audit
// (missing/stray/hash-drift, all mechanical/blocking — a drifted skill makes
// doctor `blocked`, which is correct: g22-3's three-state has no room for a
// silently-wrong installed skill tree). A v1 sidecar (pre-D7) or a missing
// sidecar both fall back to the shallow "dir count + provenance present"
// check that shipped before this cell — v1 additionally warns that deep
// inventory is unavailable, but per decision it must NEVER block (legacy
// hosts stay usable until they re-onboard); a missing sidecar keeps its
// original blocking behavior unchanged.
function doctorSkillsInstalled(root, skillsDir) {
  const dir = path.join(root, skillsDir);
  const sidecar = doctorSafeReadJson(path.join(dir, '.bee-render.json'));
  const exists = fs.existsSync(dir);
  const entries = exists ? fs.readdirSync(dir).filter((n) => !n.startsWith('.')) : [];
  if (!exists) {
    return doctorRow('skills_installed', 'warn', { count: 0, provenance: null }, `${skillsDir}/ is absent.`, { blocking: true });
  }
  if (!sidecar) {
    return doctorRow(
      'skills_installed',
      'warn',
      { count: entries.length, provenance: null },
      `${entries.length} skill dir(s) under ${skillsDir}/, provenance sidecar MISSING; runtime-side discovery has no machine-readable surface to confirm against.`,
      { blocking: true },
    );
  }
  if (sidecar.schema !== SKILL_RENDER_SCHEMA_V2) {
    return doctorRow(
      'skills_installed',
      'warn',
      { count: entries.length, provenance: sidecar },
      `${entries.length} skill dir(s) under ${skillsDir}/, provenance sidecar present (${sidecar.schema || '(unversioned)'}) — inventory unavailable (bee-render/1) — re-run onboarding/render to upgrade.`,
      { blocking: false },
    );
  }
  const audit = doctorDeepAuditSkills(dir, sidecar);
  if (!audit.ok) {
    const parts = [];
    if (audit.missing.length) parts.push(`missing: ${audit.missing.join(', ')}`);
    if (audit.stray.length) parts.push(`stray: ${audit.stray.join(', ')}`);
    if (audit.drifted.length) parts.push(`drifted: ${audit.drifted.join(', ')}`);
    return doctorRow(
      'skills_installed',
      'warn',
      { count: entries.length, provenance: sidecar, audit },
      `${entries.length} skill dir(s) under ${skillsDir}/ do not match the bee-render/2 sidecar inventory — ${parts.join('; ')}.`,
      { blocking: true, fix: 'Re-render via self-onboard sync: node skills/bee-hive/scripts/onboard_bee.mjs --repo-root . --apply' },
    );
  }
  return doctorRow(
    'skills_installed',
    'ok',
    { count: entries.length, provenance: sidecar },
    `${entries.length} skill dir(s) under ${skillsDir}/ match the bee-render/2 sidecar inventory (deep audit: every skill's content digest verified).`,
    { blocking: true },
  );
}

function doctorCustomAgentsCodex(codexVersionValue) {
  return doctorRow(
    'custom_agents',
    'unsupported',
    codexVersionValue,
    `${codexVersionValue || '(codex version unknown)'}: .codex/agents/*.toml discovery is not-observed on codex-cli ${PROBED_CODEX_VERSION} (capability matrix rows A1/A2) — only built-in default/explorer/worker agent types spawn, carrying no bee developer_instructions. This verdict is version-scoped: other versions are unverified until re-probed.`,
  );
}

// Claude's analogous mechanical set (D4): hook_wiring_resolvable /
// handlers_resolvable / model_guard_entry_present / skills_installed (shared
// helper above) — the four rows that block claude's readiness outright.
// Claude has no structurally-unknown trust rows (no `degrades` concept), so
// mechanical-green alone reaches 'ready' — no attestation required; see
// doctorOverallStatus for why that is kept deliberately simple.
function doctorClaudeHookWiring(root) {
  const settings = doctorSafeReadJson(path.join(root, '.claude', 'settings.json'));
  if (!settings || !settings.hooks) {
    return doctorRow('hook_wiring_resolvable', 'warn', false, '.claude/settings.json has no hooks block.', { blocking: true });
  }
  const events = Object.keys(settings.hooks);
  const required = ['SessionStart', 'UserPromptSubmit', 'PreToolUse', 'PostToolUse', 'Stop'];
  const missing = required.filter((e) => !events.includes(e));
  if (missing.length) {
    return doctorRow('hook_wiring_resolvable', 'warn', events, `missing lifecycle event(s) in .claude/settings.json hooks: ${missing.join(', ')}.`, { blocking: true });
  }
  return doctorRow('hook_wiring_resolvable', 'ok', events, `${events.length} lifecycle event(s) wired in .claude/settings.json.`, { blocking: true });
}

function doctorClaudeHandlersResolvable(root) {
  const settings = doctorSafeReadJson(path.join(root, '.claude', 'settings.json'));
  const commands = doctorExtractHookCommands({ hooks: settings ? settings.hooks : {} });
  if (!commands.length) {
    return doctorRow('handlers_resolvable', 'warn', [], 'no hook commands found in .claude/settings.json.', { blocking: true });
  }
  const files = doctorHookHandlerFilenames(commands);
  const missing = files.filter(
    (f) => !fs.existsSync(path.join(root, '.bee', 'bin', 'hooks', f)) && !fs.existsSync(path.join(root, 'hooks', f)),
  );
  if (missing.length) {
    return doctorRow(
      'handlers_resolvable',
      'warn',
      files,
      `missing handler file(s): ${missing.join(', ')}.`,
      { fix: 'Re-run onboarding to restore .bee/bin/hooks/*.', blocking: true },
    );
  }
  return doctorRow('handlers_resolvable', 'ok', files, `${files.length} handler file(s) resolved.`, { blocking: true });
}

function doctorClaudeModelGuardEntry(root) {
  const settings = doctorSafeReadJson(path.join(root, '.claude', 'settings.json'));
  const preToolUse = settings && settings.hooks && Array.isArray(settings.hooks.PreToolUse) ? settings.hooks.PreToolUse : [];
  const found = preToolUse.some(
    (m) =>
      m &&
      typeof m.matcher === 'string' &&
      /Agent/.test(m.matcher) &&
      Array.isArray(m.hooks) &&
      m.hooks.some((h) => h && typeof h.command === 'string' && h.command.includes('bee-model-guard')),
  );
  return doctorRow(
    'model_guard_entry_present',
    found ? 'ok' : 'warn',
    found,
    found
      ? 'PreToolUse Agent-shaped matcher is wired to bee-model-guard.mjs.'
      : 'no PreToolUse Agent-shaped bee-model-guard entry found in .claude/settings.json.',
    { blocking: true },
  );
}

function doctorClaudeRenderedAgents(root) {
  const onboarding = readOnboarding(root);
  const files = Array.isArray(onboarding?.agents_sync?.files) ? onboarding.agents_sync.files : [];
  if (!files.length) {
    return doctorRow('rendered_agents_present', 'unknown', [], 'no agents_sync.files recorded in .bee/onboarding.json.');
  }
  const missing = files.filter((f) => !fs.existsSync(path.join(root, f)));
  if (missing.length) {
    return doctorRow('rendered_agents_present', 'warn', files, `missing rendered agent file(s): ${missing.join(', ')}.`);
  }
  return doctorRow('rendered_agents_present', 'ok', files, `${files.length} rendered agent file(s) present.`);
}

function doctorClaudePermissionMode(root) {
  const settings = doctorSafeReadJson(path.join(root, '.claude', 'settings.json'));
  const configured =
    settings && settings.permissions && typeof settings.permissions.defaultMode === 'string' ? settings.permissions.defaultMode : null;
  return doctorRow(
    'permission_mode',
    configured ? 'ok' : 'unknown',
    { configured, observed: null },
    configured
      ? `configured permissions.defaultMode = "${configured}" in .claude/settings.json; observed mode has no doctor-time runtime surface.`
      : 'no permissions.defaultMode configured in .claude/settings.json.',
  );
}

// A row is load-bearing (blocking) only when the cell that produced it says
// so explicitly — never inferred from its status alone, so a `warn` on a
// non-load-bearing informational row (codex_version, hooks_observed_this_
// session, permission_mode, hook_sources, custom_agents, claude's rendered_
// agents_present) can never silently promote itself into the verdict. D4:
// three states, computed in two passes —
//   1. any `blocking` row not-ok -> 'blocked', full stop (mechanical facts:
//      a missing/drifted hooks file or an unresolvable handler/skill set
//      means nothing downstream is provable, attested or not).
//   2. otherwise, any `degrades` row (codex's 4 structurally-unknown trust
//      rows) without a currently-VALID attestation -> 'degraded'; a valid
//      attestation covers them -> 'ready'.
//   3. no blocking, no degrading (or claude, which has neither concept for
//      today's row set) -> 'ready' outright — mechanical green is the whole
//      bar; no attestation concept applies.
// `attestation` is null for claude (and for a codex call that never reaches
// this far) — see doctorValidateAttestation below.
function doctorOverallStatus(rows, attestation = null) {
  const blocked = rows.filter((r) => r.blocking && r.status !== 'ok');
  if (blocked.length > 0) {
    return {
      overall_status: 'blocked',
      reasons: blocked.map((r) => `${r.row}: BLOCKS readiness — ${r.evidence}`),
    };
  }
  const degrading = rows.filter((r) => r.degrades);
  if (degrading.length === 0) {
    return { overall_status: 'ready', reasons: [] };
  }
  if (attestation && attestation.valid) {
    return { overall_status: 'ready', reasons: [] };
  }
  const reasons = [
    ...degrading.map((r) => `${r.row}: ${r.evidence}${r.degraded_reason ? ` ${r.degraded_reason}` : ''}`),
    attestation
      ? `${attestation.reason}: ${attestation.detail}`
      : 'no_attestation: no attestation recorded — run `bee doctor attest --runtime codex` once trust state has been reviewed via /hooks.',
  ];
  return { overall_status: 'degraded', reasons };
}

// ─── doctor attest (g22-3, D5-REVISED): a static, request-only attestation
// that a human (or an agent on the human's behalf) reviewed codex trust
// state via the interactive /hooks TUI and is vouching for THIS exact
// hooks-file/codex-version/repo pairing. Recorded to a gitignored runtime-
// tier file — never tracked state, never auto-run by `doctor` itself (D5-
// REVISED: attest is a distinct, deliberate verb, not a doctor side effect).
// No liveness leg exists on codex: hooks.jsonl only ever logs deny/crash
// events and tools.jsonl is claude-only (bee-tools-logger.mjs, PostToolUse)
// — a healthy codex session writes NOTHING codex-side that doctor could
// observe, so attestation validity is purely static (hash/version/identity),
// and the reason string says so honestly rather than implying a liveness
// check that does not exist.
function doctorAttestPath(root) {
  return path.join(root, '.bee', 'doctor-attest.json');
}

function doctorRepoIdentity(root) {
  try {
    return fs.realpathSync(root);
  } catch {
    return root;
  }
}

// Validates a recorded attestation against LIVE state: hooks-file sha256,
// codex --version, and repo identity must all still match what was attested.
// Any single failed leg makes the whole attestation inert (never partially
// trusted) — the specific stale reason is one of hash_changed / version_
// changed / identity_changed / no_attestation, exactly as D5-REVISED names
// them, so a caller can branch on `reason` without parsing prose.
function doctorValidateAttestation(root, liveCodexVersion) {
  const record = doctorSafeReadJson(doctorAttestPath(root));
  if (!record) {
    return {
      valid: false,
      reason: 'no_attestation',
      detail: 'no attestation recorded — run `bee doctor attest --runtime codex` once trust state has been reviewed via /hooks.',
      record: null,
    };
  }
  const hooksPath = path.join(root, '.codex', 'hooks.json');
  const liveHash = fs.existsSync(hooksPath) ? hashFile(hooksPath) : null;
  if (!liveHash || liveHash !== record.hooks_file_sha256) {
    return {
      valid: false,
      reason: 'hash_changed',
      detail: `live .codex/hooks.json sha256 (${liveHash || '(file missing)'}) no longer matches the attested hash (${record.hooks_file_sha256}) — re-review /hooks and re-attest.`,
      record,
    };
  }
  if ((liveCodexVersion || null) !== (record.codex_version || null)) {
    return {
      valid: false,
      reason: 'version_changed',
      detail: `live codex --version (${liveCodexVersion || '(unresolved)'}) no longer matches the attested version (${record.codex_version || '(unresolved)'}) — re-review /hooks and re-attest.`,
      record,
    };
  }
  const liveIdentity = doctorRepoIdentity(root);
  if (liveIdentity !== record.repo_identity) {
    return {
      valid: false,
      reason: 'identity_changed',
      detail: `live repo identity (${liveIdentity}) no longer matches the attested identity (${record.repo_identity}) — an attestation from a different checkout never carries over; re-attest here.`,
      record,
    };
  }
  return {
    valid: true,
    reason: null,
    detail:
      `attested at ${record.at} — hooks-file sha256, codex version, and repo identity all still match. ` +
      'codex exposes no hook-fire event surface (hooks.jsonl is deny/crash-only, tools.jsonl is claude-only) — attestation is static, not a liveness check.',
    record,
  };
}

function handleDoctorAttest(root, flags) {
  const runtime = requireFlag(flags, 'runtime');
  if (runtime !== 'codex') {
    throw new Error(`doctor attest: --runtime must be "codex" (got "${runtime}") — claude has no trust-unknown rows and no attestation model.`);
  }
  const hooksPath = path.join(root, '.codex', 'hooks.json');
  if (!fs.existsSync(hooksPath)) {
    throw new Error(`doctor attest: ${hooksPath} does not exist — nothing to attest.`);
  }
  const versionRow = doctorCodexVersion();
  const sessionId =
    typeof flags.session === 'string' && flags.session
      ? flags.session
      : process.env.CODEX_SESSION_ID || process.env.CLAUDE_SESSION_ID || null;
  const record = {
    hooks_file_sha256: hashFile(hooksPath),
    codex_version: versionRow.value,
    session_id: sessionId,
    at: new Date().toISOString(),
    repo_identity: doctorRepoIdentity(root),
  };
  writeJsonAtomic(doctorAttestPath(root), record);
  const result = { ok: true, attestation: record };
  return {
    result,
    text: `doctor attest: recorded (hooks sha256 ${record.hooks_file_sha256.slice(0, 12)}…, codex ${record.codex_version || '(unresolved)'}, repo ${record.repo_identity}).`,
  };
}

// ─── native transport capability probe (codex-native-transport D3/D4,
// advisor Δ2/R3 — binding): a version+config-scoped record of whether the
// installed codex-cli can accept a native spawn_agent model override,
// mirroring the g22-3 doctor-attest pattern above (same validity-leg
// discipline: any single failed leg invalidates the whole record, never
// partially trusted) but stored in its OWN separate gitignored file —
// doctor-attest's 3 legs (hooks hash, codex version, repo identity) cannot
// see codex FEATURE/config changes, so this record needs a 4th leg
// (config_scope_hash) alongside version+identity. Never merged into
// doctor-attest.json (Δ2, binding: attest is a human-reviewed /hooks trust
// vouching, this is a machine-observed capability fact — different
// questions, different files).
//
// Evidence is produced by `codex features list` (read-only, safe against
// the real environment — it lists, it never sets) and, for the actual
// override-spawn acceptance leg, the g22-6 canary harness (cnt-5) running
// under an ISOLATED per-run CODEX_HOME (D4: bee never flips flags on the
// user's real config). This cell (cnt-2) only defines the record shape,
// the writer cnt-5 calls with its canary evidence, and the reader every
// downstream consumer gates on (readNativeTransportClassification — R3).
export function nativeTransportProbePath(root) {
  return path.join(root, '.bee', 'native-transport-probe.json');
}

const NATIVE_TRANSPORT_PROBE_SCHEMA = 'native-transport-probe/1';

// The feature/config scope this probe cares about (D3a/Δ2-amended,
// decisions c0cba64e/760e9b05 — authoritative): the hash covers ALL FOUR
// verdict-determining flags — multi_agent, multi_agent_v2 (both directly
// observable via `codex features list`, doctorCodexFeaturesList below —
// D3a made base multi_agent a determinant too, via the external_cli_only
// trigger, so it must be hashed or an unhashed multi_agent toggle would
// leave a stale verdict standing) plus hide_spawn_agent_metadata and
// tool_namespace (config.toml-only settings the canary configures inside
// its isolated CODEX_HOME and cannot be independently re-observed later
// without re-running the canary — included in the schema/hash for
// completeness and provenance, but honestly excluded from the LIVE
// re-check in readNativeTransportClassification below, which only has a
// re-observation surface for the first two).
const NATIVE_TRANSPORT_SCOPE_KEYS = ['multi_agent', 'multi_agent_v2', 'hide_spawn_agent_metadata', 'tool_namespace'];

function nativeTransportScopeFromEvidence(evidence) {
  const scope = {};
  for (const key of NATIVE_TRANSPORT_SCOPE_KEYS) {
    scope[key] = evidence && typeof evidence === 'object' && key in evidence ? evidence[key] : null;
  }
  return scope;
}

export function nativeTransportConfigScopeHash(scope) {
  const flat = scope && typeof scope === 'object' ? scope : {};
  const keys = Object.keys(flat).sort();
  return crypto.createHash('sha256').update(JSON.stringify(flat, keys)).digest('hex');
}

// Read-only `codex features list` — the same tolerance for a subprocess call
// as doctorCodexVersion() above (identical failure handling: binary absent,
// non-zero exit, or a hung call all degrade to null rather than throwing).
// Never mutates codex state; safe to run against the user's real CODEX_HOME.
export function doctorCodexFeaturesList() {
  try {
    const result = spawnSync('codex', ['features', 'list'], { encoding: 'utf8', timeout: 5000 });
    if (result.error || typeof result.status !== 'number' || result.status !== 0) {
      return null;
    }
    const flags = {};
    for (const rawLine of (result.stdout || '').split('\n')) {
      const line = rawLine.trimEnd();
      const match = /^(\S+)\s+(.+?)\s+(true|false)\s*$/.exec(line);
      if (match) {
        flags[match[1]] = { maturity: match[2].trim(), enabled: match[3] === 'true' };
      }
    }
    return flags;
  } catch {
    return null;
  }
}

/**
 * writeNativeTransportProbe(root, { codexVersion, evidence }) — the writer
 * cnt-5's canary calls after it runs. `evidence` carries every raw
 * observation: { multi_agent, multi_agent_v2, hide_spawn_agent_metadata,
 * tool_namespace, override_spawn_accepted } (D3a/Δ2-amended). The
 * classification (classifyNativeTransport, dispatch-guard.mjs, pure) and the
 * config_scope hashed for the 3rd validity leg (the 4 verdict-determining
 * flags, Δ2-amended) are BOTH derived from this single evidence object —
 * one source, so classification and hash can never independently drift the
 * way two separately-passed parameters could. Stamps the remaining validity
 * legs (repo identity, codex version) at write time and stores atomically.
 * Returns the written record.
 */
export function writeNativeTransportProbe(root, { codexVersion = null, evidence = null } = {}) {
  const configScope = nativeTransportScopeFromEvidence(evidence);
  const record = {
    schema: NATIVE_TRANSPORT_PROBE_SCHEMA,
    at: new Date().toISOString(),
    codex_version: codexVersion || null,
    repo_identity: doctorRepoIdentity(root),
    config_scope: configScope,
    config_scope_hash: nativeTransportConfigScopeHash(configScope),
    evidence: evidence || null,
    classification: classifyNativeTransport(evidence),
  };
  writeJsonAtomic(nativeTransportProbePath(root), record);
  return record;
}

/**
 * readNativeTransportClassification(root) — R3 (binding), the ONE reader
 * every downstream consumer (cnt-3's prepare, cnt-4's guard) gates on.
 * Applies the validity legs in order — repo identity, codex version,
 * config-scope integrity, then a live re-check of the codex-observable
 * subset of the scope — and returns `native_budget_only` the moment any leg
 * fails, or the record is missing/malformed. D3: unknown/absent evidence
 * stays inert until proven on the host's actual build.
 *
 * Returns { classification, valid, reason, record }: `classification` is
 * the single string downstream code should branch on; `valid`/`reason`/
 * `record` exist so a caller can build the named-reason refusal D1 requires
 * ("reports its reason; it never silently runs CLI") without re-deriving
 * the leg logic itself.
 */
export function readNativeTransportClassification(root) {
  const record = doctorSafeReadJson(nativeTransportProbePath(root));
  if (!record || record.schema !== NATIVE_TRANSPORT_PROBE_SCHEMA) {
    return { classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, valid: false, reason: 'no_probe_record', record: null };
  }
  const liveIdentity = doctorRepoIdentity(root);
  if (liveIdentity !== record.repo_identity) {
    return { classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, valid: false, reason: 'identity_changed', record };
  }
  const liveVersion = doctorCodexVersion().value;
  if ((liveVersion || null) !== (record.codex_version || null)) {
    return { classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, valid: false, reason: 'version_changed', record };
  }
  const recomputedHash = nativeTransportConfigScopeHash(record.config_scope);
  if (recomputedHash !== record.config_scope_hash) {
    return { classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, valid: false, reason: 'config_scope_corrupt', record };
  }
  const liveFlags = doctorCodexFeaturesList();
  if (liveFlags) {
    const scope = record.config_scope || {};
    const liveMultiAgent = liveFlags.multi_agent ? liveFlags.multi_agent.enabled : null;
    const liveMultiAgentV2 = liveFlags.multi_agent_v2 ? liveFlags.multi_agent_v2.enabled : null;
    if (liveMultiAgent !== (scope.multi_agent ?? null) || liveMultiAgentV2 !== (scope.multi_agent_v2 ?? null)) {
      return { classification: NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY, valid: false, reason: 'flag_state_changed', record };
    }
  }
  return { classification: record.classification, valid: true, reason: null, record };
}

// D4 (binding): informational only — this row NAMES the unlock, it never
// applies it. Bee never writes features.multi_agent_v2 / hide_spawn_agent_
// metadata into the user's real ~/.codex/config.toml (only the canary's
// isolated per-run copy does). Deliberately non-blocking, non-degrading —
// same "informational, never load-bearing" family as codex_version /
// hooks_observed_this_session / permission_mode / hook_sources / custom_
// agents (see the doctorOverallStatus comment above).
export function doctorNativeTransportUnlock(root, liveFeatures) {
  const probe = readNativeTransportClassification(root);
  const shipsFlag = !!(liveFeatures && liveFeatures.multi_agent_v2);
  if (probe.classification === NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE) {
    return doctorRow(
      'native_transport_unlock',
      'ok',
      { classification: probe.classification, ships_flag: shipsFlag },
      'native model-override transport is classified native_model_override — no unlock needed.',
    );
  }
  if (probe.classification !== NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY || !shipsFlag) {
    return doctorRow(
      'native_transport_unlock',
      'ok',
      { classification: probe.classification, ships_flag: shipsFlag },
      `native transport classification is "${probe.classification}"; this codex-cli install does not ship the multi_agent_v2 feature flag, so there is no unlock to name.`,
    );
  }
  return doctorRow(
    'native_transport_unlock',
    'ok',
    { classification: probe.classification, ships_flag: true },
    'native model-override transport is classified native_budget_only, but this codex-cli build ships the multi_agent_v2 feature flag (currently disabled). ' +
      'To unlock native per-agent model override, enable `features.multi_agent_v2 = true` and `hide_spawn_agent_metadata = false` in YOUR OWN ~/.codex/config.toml ' +
      '(D4: bee never writes this for you), then re-run the canary probe to re-classify. This row only names the unlock.',
  );
}

// dispatch (g22-1, GH #22 P0-3) — thin flag-parsing wrapper: every actual
// resolution/payload-construction/prepare-time-record decision lives in
// lib/dispatch-prepare.mjs's prepareDispatch, so this handler's only job is
// pulling flags and letting prepareDispatch's own errors/refusals surface
// (a bad --runtime/--kind or a missing --cell throws; a cli-shaped cell
// resolution or an unconfigured advisor slot is a typed {ok:false} result,
// not a throw — same discipline as reservations.reserve's conflict result).
// Native-transport classification (codex-native-transport D1/D3, R3 —
// binding): this handler is the ONE place that reads
// readNativeTransportClassification(root) and hands its `.classification`
// string into prepareDispatch — dispatch-prepare.mjs (lib) deliberately never
// imports that reader itself (it lives here, in the bin layer; a lib module
// reaching back into bin would invert the repo's bin->lib import direction —
// see prepareDispatch's own docstring). Only the codex runtime ever carries a
// native-transport probe; every other runtime passes classification
// undefined, which prepareDispatch treats exactly like an unprobed host (D3:
// "unprobed/unknown => native_budget_only") — inert for every non-native slot.
function handleDispatchPrepare(root, flags) {
  const runtime = requireFlag(flags, 'runtime');
  const kind = requireFlag(flags, 'kind');
  const cellId = typeof flags.cell === 'string' && flags.cell ? flags.cell : null;
  // hardening-7: --worker/--force-ownership are inert for every kind but
  // 'cell' (prepareDispatch's own claim-ownership guard only reads them
  // there) — passed through unconditionally so every existing gather/
  // reviewer/advisor call site stays byte-identical. Restored here
  // (hardening-4b) after this handler was found reverted to its
  // pre-hardening-7 shape while dispatch-prepare.mjs's own worker-required
  // logic was still live — the two must travel together.
  const worker = typeof flags.worker === 'string' && flags.worker ? flags.worker : null;
  const forceOwnership = flags['force-ownership'] === true;
  const classification = runtime === 'codex' ? readNativeTransportClassification(root).classification : undefined;
  const out = prepareDispatch(root, { runtime, kind, cell: cellId, worker, forceOwnership, classification });
  return { result: out, text: JSON.stringify(out, null, 2) };
}

function handleDoctor(root, flags) {
  const runtime = requireFlag(flags, 'runtime');
  if (runtime !== 'codex' && runtime !== 'claude') {
    throw new Error(`doctor: --runtime must be "codex" or "claude", got "${runtime}".`);
  }
  let rows;
  let attestation = null;
  if (runtime === 'codex') {
    const versionRow = doctorCodexVersion();
    rows = [
      versionRow,
      doctorHooksFilePresent(root),
      doctorCapabilityBaselineMatch(root),
      ...doctorCodexTrustUnknownRows(versionRow.value),
      doctorHookHandlersResolvable(root),
      doctorHooksObservedThisSession(root),
      doctorPermissionModeCodex(root),
      doctorHookSourcesCodex(root),
      doctorSkillsInstalled(root, path.join('.agents', 'skills')),
      doctorCustomAgentsCodex(versionRow.value),
      doctorNativeTransportUnlock(root, doctorCodexFeaturesList()),
    ];
    attestation = doctorValidateAttestation(root, versionRow.value);
  } else {
    rows = [
      doctorClaudeHookWiring(root),
      doctorClaudeHandlersResolvable(root),
      doctorClaudeModelGuardEntry(root),
      doctorSkillsInstalled(root, path.join('.claude', 'skills')),
      doctorClaudeRenderedAgents(root),
      doctorClaudePermissionMode(root),
      doctorHooksObservedThisSession(root),
    ];
  }
  const { overall_status, reasons } = doctorOverallStatus(rows, attestation);
  const result = { runtime, overall_status, rows, reasons };
  if (runtime === 'codex') {
    result.attestation = attestation.valid
      ? { status: 'valid', at: attestation.record.at, codex_version: attestation.record.codex_version, repo_identity: attestation.record.repo_identity }
      : { status: 'invalid', reason: attestation.reason, detail: attestation.detail };
  }
  const lines = [`bee doctor --runtime ${runtime}: ${overall_status.toUpperCase()}`];
  for (const row of rows) lines.push(`  [${row.status}] ${row.row}: ${row.evidence}`);
  if (reasons.length) {
    lines.push('', 'Reasons:');
    for (const reason of reasons) lines.push(`  - ${reason}`);
  }
  return { result, text: lines.join('\n') };
}

// Per-group usage fallback (dispatcher-unify du-1): the shim always supplies
// the group token, so the generic no-command path can never fire for helper
// calls. When a leading group token resolves to no registry entry, its group's
// fallback emits the legacy "Use:" line byte-exact and exits non-zero.
function stateUsageFallback(leading) {
  const verb = leading[1];
  if (verb === 'worker') {
    const sub = leading[2];
    return `Unknown worker action "${sub || '(missing)'}". Use: add, update, remove, clear, prune.`;
  }
  // session (fresh-session-handoff fsh-4, D2/D4): a new nested verb family,
  // mirroring the worker branch above exactly.
  if (verb === 'session') {
    const sub = leading[2];
    return `Unknown session action "${sub || '(missing)'}". Use: list, bind, unbind.`;
  }
  // handoff (fresh-session-handoff fsh-9, D1): a new nested verb family for
  // the two-kind handoff lifecycle, mirroring the worker/session branches.
  if (verb === 'handoff') {
    const sub = leading[2];
    return `Unknown handoff action "${sub || '(missing)'}". Use: write, adopt, show.`;
  }
  // advisor-ref (ao-4-1, AO3/AO13): the two-verb advisor-consult family,
  // mirroring the worker/session/handoff branches above.
  if (verb === 'advisor-ref') {
    const sub = leading[2];
    return `Unknown advisor-ref action "${sub || '(missing)'}". Use: record, show.`;
  }
  // plan-rev (multisession-native-9, D7/C2): a one-verb family, mirroring
  // the worker/session/handoff/advisor-ref branches above.
  if (verb === 'plan-rev') {
    const sub = leading[2];
    return `Unknown plan-rev action "${sub || '(missing)'}". Use: bump.`;
  }
  return `Unknown command "${verb || '(missing)'}". Use: set, gate, plan-rev bump, worker, scribing-run, start-feature, lanes, rebuild-projections, session, handoff, advisor-ref, compact-log, compact-check, compact-capsule.`;
}

function backlogUsageFallback(leading) {
  const verb = leading[1];
  // pbi (backlog-unification D3): a new nested verb family, mirroring the
  // worker/session/handoff branches in stateUsageFallback above.
  if (verb === 'pbi') {
    const sub = leading[2];
    return `Unknown pbi action "${sub || '(missing)'}". Use: add, status, amend, list.`;
  }
  return `Unknown command "${verb || '(missing)'}". Use: counts, rank, badges, add, propose, pbi, render, findings.`;
}

function captureUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: add, list, flush, count.`;
}

function intentUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: set, show, advance, clear.`;
}

// bee_reviews.mjs's own 'candidate' verb has a nested sub-action ('add')
// with its own distinct legacy error text (bee_reviews.mjs:186-189); every
// other unknown top-level verb falls through to the default review-modes
// message (bee_reviews.mjs:213-217), preserved byte-exact including its
// trailing "(review modes: ...)" annotation (DB3).
function reviewsUsageFallback(leading) {
  const verb = leading[1];
  if (verb === 'candidate') {
    const sub = leading[2];
    return `Unknown "candidate" subcommand "${sub || '(missing)'}". Use: candidate add.`;
  }
  return (
    `Unknown command "${verb || '(missing)'}". Use: create, list, show, record, candidate add, candidates, status. ` +
    `(review modes: ${REVIEW_MODES.join(', ')})`
  );
}

function feedbackUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: digest, count, collect, rank.`;
}

function perfUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: start, stop, section, log, render, report, sync.`;
}

function configUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: get, set, unset, validate.`;
}

function worktreeUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: register, list, unregister, new, merge.`;
}

function herdingUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: enable, disable, status.`;
}

function dispatchUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: prepare.`;
}

function recoveryUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: scan, window.`;
}

function tmpUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: sweep.`;
}

function knowledgeUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: check, index, list, context, promote.`;
}

// Legacy-4 group fallbacks (dispatcher-unify du-4): bee_cells.mjs/
// bee_reservations.mjs/bee_decisions.mjs are now shims, so their own
// default-case "Unknown command ... Use: ..." messages (previously emitted
// by each helper's own run() switch) must be reproduced byte-exact here —
// the DA5 bijection probe (test_bee_cli.mjs) still spawns the shims
// directly and parses this exact stderr line.
function cellsUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: list, ready, show, add, update, claim, verify, cap, block, drop, unclaim, reopen, tier, judge, claim-next, reset-budget, judge-record, schedule, archive, unarchive.`;
}

function reservationsUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: reserve, release, list, sweep.`;
}

function decisionsUsageFallback(leading) {
  const verb = leading[1];
  return `Unknown command "${verb || '(missing)'}". Use: log, supersede, redact, active, search, archive, tag, render.`;
}

const GROUP_USAGE_FALLBACKS = {
  cells: cellsUsageFallback,
  reservations: reservationsUsageFallback,
  decisions: decisionsUsageFallback,
  state: stateUsageFallback,
  backlog: backlogUsageFallback,
  capture: captureUsageFallback,
  intent: intentUsageFallback,
  reviews: reviewsUsageFallback,
  feedback: feedbackUsageFallback,
  perf: perfUsageFallback,
  worktree: worktreeUsageFallback,
  herding: herdingUsageFallback,
  config: configUsageFallback,
  dispatch: dispatchUsageFallback,
  recovery: recoveryUsageFallback,
  tmp: tmpUsageFallback,
  knowledge: knowledgeUsageFallback,
};

const HANDLERS = {
  status: handleStatus,
  'cells.list': handleCellsList,
  'cells.ready': handleCellsReady,
  'cells.show': handleCellsShow,
  'cells.add': handleCellsAdd,
  'cells.update': handleCellsUpdate,
  'cells.claim': handleCellsClaim,
  'cells.verify': handleCellsVerify,
  'cells.cap': handleCellsCap,
  'cells.block': handleCellsBlock,
  'cells.drop': handleCellsDrop,
  'cells.unclaim': handleCellsUnclaim,
  'cells.reopen': handleCellsReopen,
  'cells.tier': handleCellsTier,
  'cells.judge': handleCellsJudge,
  'cells.claim-next': handleCellsClaimNext,
  'cells.reset-budget': handleCellsResetBudget,
  'cells.judge-record': handleCellsJudgeRecord,
  'cells.schedule': handleCellsSchedule,
  'cells.archive': handleCellsArchive,
  'cells.unarchive': handleCellsUnarchive,
  'reservations.reserve': handleReservationsReserve,
  'reservations.release': handleReservationsRelease,
  'reservations.list': handleReservationsList,
  'reservations.sweep': handleReservationsSweep,
  'decisions.log': handleDecisionsLog,
  'decisions.supersede': handleDecisionsSupersede,
  'decisions.redact': handleDecisionsRedact,
  'decisions.active': handleDecisionsActive,
  'decisions.search': handleDecisionsSearch,
  'decisions.archive': handleDecisionsArchive,
  'decisions.tag': handleDecisionsTag,
  'decisions.render': handleDecisionsRender,
  'state.set': handleStateSet,
  'state.gate': handleStateGate,
  'state.plan-rev.bump': handleStatePlanRevBump,
  'state.worker.add': handleStateWorkerAdd,
  'state.worker.update': handleStateWorkerUpdate,
  'state.worker.remove': handleStateWorkerRemove,
  'state.worker.clear': handleStateWorkerClear,
  'state.worker.prune': handleStateWorkerPrune,
  'state.scribing-run': handleStateScribingRun,
  'state.start-feature': handleStateStartFeature,
  'state.lanes': handleStateLanes,
  'state.rebuild-projections': handleStateRebuildProjections,
  'state.session.list': handleStateSessionList,
  'state.session.bind': handleStateSessionBind,
  'state.session.unbind': handleStateSessionUnbind,
  'state.handoff.write': handleStateHandoffWrite,
  'state.handoff.adopt': handleStateHandoffAdopt,
  'state.handoff.show': handleStateHandoffShow,
  'state.advisor-ref.record': handleStateAdvisorRefRecord,
  'state.advisor-ref.show': handleStateAdvisorRefShow,
  'state.compact-log': handleStateCompactLog,
  'state.compact-check': handleStateCompactCheck,
  'state.compact-capsule': handleStateCompactCapsule,
  'backlog.counts': handleBacklogCounts,
  'backlog.rank': handleBacklogRank,
  'backlog.badges': handleBacklogBadges,
  'backlog.add': handleBacklogAdd,
  'backlog.propose': handleBacklogPropose,
  'backlog.pbi.add': handleBacklogPbiAdd,
  'backlog.pbi.status': handleBacklogPbiStatus,
  'backlog.pbi.amend': handleBacklogPbiAmend,
  'backlog.pbi.list': handleBacklogPbiList,
  'backlog.render': handleBacklogRender,
  'backlog.findings': handleBacklogFindings,
  'capture.add': handleCaptureAdd,
  'capture.list': handleCaptureList,
  'capture.flush': handleCaptureFlush,
  'capture.count': handleCaptureCount,
  'intent.set': handleIntentSet,
  'intent.show': handleIntentShow,
  'intent.advance': handleIntentAdvance,
  'intent.clear': handleIntentClear,
  'knowledge.check': handleKnowledgeCheck,
  'knowledge.index': handleKnowledgeIndex,
  'knowledge.list': handleKnowledgeList,
  'knowledge.context': handleKnowledgeContext,
  'knowledge.promote': handleKnowledgePromote,
  'reviews.create': handleReviewsCreate,
  'reviews.list': handleReviewsList,
  'reviews.show': handleReviewsShow,
  'reviews.record': handleReviewsRecord,
  'reviews.candidate.add': handleReviewsCandidateAdd,
  'reviews.candidates': handleReviewsCandidates,
  'reviews.status': handleReviewsStatus,
  'feedback.digest': handleFeedbackDigest,
  'feedback.count': handleFeedbackCount,
  'feedback.collect': handleFeedbackCollect,
  'feedback.rank': handleFeedbackRank,
  'perf.start': handlePerfStart,
  'perf.stop': handlePerfStop,
  'perf.section': handlePerfSection,
  'perf.log': handlePerfLog,
  'perf.render': handlePerfRender,
  'perf.report': handlePerfReport,
  'perf.sync': handlePerfSync,
  'worktree.register': handleWorktreeRegister,
  'worktree.list': handleWorktreeList,
  'worktree.unregister': handleWorktreeUnregister,
  'worktree.new': handleWorktreeNew,
  'worktree.merge': handleWorktreeMerge,
  'herding.enable': handleHerdingEnable,
  'herding.disable': handleHerdingDisable,
  'herding.status': handleHerdingStatus,
  'tmp.sweep': handleTmpSweep,
  'config.get': handleConfigGet,
  'config.set': handleConfigSet,
  'config.unset': handleConfigUnset,
  'config.validate': handleConfigValidate,
  'dispatch.prepare': handleDispatchPrepare,
  doctor: handleDoctor,
  'doctor.attest': handleDoctorAttest,
  'recovery.scan': handleRecoveryScan,
  'recovery.window': handleRecoveryWindow,
};

// ─── argv parsing: "bee <group> [<action>] [--flag value|--flag=value ...]" ─
// The flag-alone boolean set is the closed union of the helper files' own
// hardcoded boolean-flag lists (bee_cells: json/stdin/behavior-change/
// evidence-stdin; bee_reservations: json/active-only; bee_decisions: json;
// bee_state: json/dry-run; bee_backlog: json/write) — every OTHER flag, even
// one the registry declares as JSON-Schema type "boolean" (e.g. cells.verify's
// --passed), takes an explicit "true"/"false" argument exactly as the
// original CLIs parse it; this keeps `bee cells verify ... --passed true`
// byte-parity-correct. `dry-run` MUST be here or `state worker prune
// --dry-run --json` would consume `--json` as the value of `--dry-run`
// (bee_state.mjs parsed it boolean-alone too); `write` MUST be here for the
// same reason on `backlog rank --write --json` / `backlog badges --write --json`
// (bee_backlog.mjs parsed it boolean-alone too). `as-lane` (fresh-session-
// handoff fsh-4, D2/D4) is state.start-feature's lane-mode opt-in — a
// DISTINCT flag name from the `--lane <feature>` string flag used by
// state.set/gate/scribing-run/session.bind, so the two never collide here.
// `cleanup` (worktree-session-routing wsr-2, GH #21, decision D8b) is
// `worktree merge`'s flag-alone opt-in for post-merge worktree removal.
// `show` (sqs-b3) is state.scribing-run's read-only query opt-in — MUST be
// here or `state scribing-run --show --feature X` would consume `--feature`
// as --show's own value, same class of bug dry-run/write/as-lane guard
// against above.
export const FLAG_ALONE_BOOLEANS = new Set(['json', 'stdin', 'behavior-change', 'evidence-stdin', 'active-only', 'dry-run', 'write', 'as-lane', 'no-lane', 'waive-scribing-debt', 'html', 'string', 'cleanup', 'force-ownership', 'local', 'all', 'untagged', 'check', 'with-companion', 'lanes-full', 'strict', 'queue-submit', 'show']);

export function splitCommandTokens(argv) {
  const leading = [];
  let i = 0;
  while (i < argv.length && !argv[i].startsWith('--')) {
    leading.push(argv[i]);
    i += 1;
  }
  return { leading, rest: argv.slice(i) };
}

/**
 * Longest-prefix match over the registry names, so a 3-token command like
 * "state worker add" resolves to state.worker.add while a 2-token one like
 * "cells ready" resolves to cells.ready and a no-subcommand group like
 * "status" resolves to status (with any trailing tokens as `extra`). When no
 * prefix matches a registry entry, fall back to the legacy shaping (bare token
 * for length 1, "<group>.<verb>" for length ≥ 2) so the unknown-command /
 * nearest-match / group-usage-fallback paths downstream behave as before.
 */
export function resolveCommand(leading) {
  if (leading.length === 0) return { commandName: null, extra: [] };
  const names = new Set(COMMAND_REGISTRY.map((e) => e.name));
  for (let n = leading.length; n >= 1; n -= 1) {
    const candidate = leading.slice(0, n).join('.');
    if (names.has(candidate)) return { commandName: candidate, extra: leading.slice(n) };
  }
  if (leading.length === 1) return { commandName: leading[0], extra: [] };
  return { commandName: `${leading[0]}.${leading[1]}`, extra: leading.slice(2) };
}

/**
 * Parse the flag section of argv into a {name: value} map plus a stripped
 * `json` flag. Returns {flags, json} on success or {error} (never throws) —
 * the {field, reason, command} shape validate-args.mjs already uses.
 */
export function parseFlags(tokens) {
  const flags = {};
  let json = false;
  for (let i = 0; i < tokens.length; i += 1) {
    const tok = tokens[i];
    if (!tok.startsWith('--')) {
      return { error: { field: null, reason: `unexpected argument "${tok}"`, command: null } };
    }
    const eq = tok.indexOf('=');
    const name = eq === -1 ? tok.slice(2) : tok.slice(2, eq);
    let value;
    if (eq !== -1) {
      value = tok.slice(eq + 1);
    } else if (FLAG_ALONE_BOOLEANS.has(name)) {
      value = true;
    } else {
      value = tokens[i + 1];
      if (value === undefined) {
        return { error: { field: name, reason: `flag --${name} requires a value`, command: null } };
      }
      i += 1;
    }
    if (name === 'json') {
      json = true;
      continue;
    }
    flags[name] = value;
  }
  return { flags, json };
}

// ─── nearest-match suggestion (unknown command → suggestion, never a bare
// not-found) — plain Levenshtein edit distance over registry names. ────────

function levenshtein(a, b) {
  const m = a.length;
  const n = b.length;
  const dp = Array.from({ length: m + 1 }, (_, i) => [i, ...Array(n).fill(0)]);
  for (let j = 0; j <= n; j += 1) dp[0][j] = j;
  for (let i = 1; i <= m; i += 1) {
    for (let j = 1; j <= n; j += 1) {
      dp[i][j] =
        a[i - 1] === b[j - 1]
          ? dp[i - 1][j - 1]
          : 1 + Math.min(dp[i - 1][j - 1], dp[i - 1][j], dp[i][j - 1]);
    }
  }
  return dp[m][n];
}

export function nearestCommandName(name, names = COMMAND_REGISTRY.map((e) => e.name)) {
  let best = null;
  let bestDist = Infinity;
  for (const candidate of names) {
    const dist = levenshtein(String(name || ''), candidate);
    if (dist < bestDist) {
      bestDist = dist;
      best = candidate;
    }
  }
  return best;
}

// ─── deprecated redirect (D2/D3 scope gap closed, validating iteration 1) ──
// No registry entry is deprecated today (all `deprecated: null`); the DISPATCH
// LOGIC must exist regardless, exercised in tests via a synthetic entry.

export function deprecatedRedirect(entry) {
  if (!entry || !entry.deprecated) return null;
  const since = entry.deprecated.since ?? null;
  const useInstead = entry.deprecated.use_instead ?? null;
  const message = `"${entry.name}" is deprecated${since ? ` since ${since}` : ''}; use "${useInstead}" instead.`;
  return {
    result: { ok: false, deprecated: true, since, use_instead: useInstead, message },
    text: `"${entry.name}" is deprecated${since ? ` since ${since}` : ''} — use "${useInstead}" instead.`,
    exitCode: 1,
  };
}

// ─── manifest content-hash tracking (drift over time) ──────────────────────
// bee.mjs runs as a fresh process per invocation with no built-in session
// concept, so the "last seen" hash is persisted to a small state file:
// <root>/.bee/manifest-hash.json ({ hash, checked_at }) — sibling to the
// other runtime-generated .bee/ files (reservations.json, decisions.jsonl).

export function computeManifestHash(registry = COMMAND_REGISTRY, schemaVersion = SCHEMA_VERSION) {
  const payload = JSON.stringify({ schema_version: schemaVersion, commands: registry });
  return crypto.createHash('sha256').update(payload).digest('hex');
}

function manifestHashStatePath(root) {
  return cacheFilePath(root, 'manifest-hash.json');
}
// Legacy location (pre-#11): the drift cache used to sit directly in .bee/ root.
function legacyManifestHashStatePath(root) {
  return path.join(root, '.bee', 'manifest-hash.json');
}

/** Compare the current registry hash against the last-persisted one, then
 * persist the current hash. Returns {manifest_changed, hint} — hint is only
 * meaningful when manifest_changed is true.
 *
 * `skipWrite` (codex-native-runtime-v2 D11): doctor is read-only FOR REAL —
 * zero writes anywhere, including this cache. main() passes skipWrite:true
 * only for the doctor route, which never even attempts the write, so it
 * cannot fail even on an unwritable cache dir. For every OTHER (mutating)
 * command the write still runs, but is now wrapped best-effort: a write
 * failure (e.g. a read-only sandbox) must never crash drift detection — the
 * comparison above already completed off the successfully-read prior hash,
 * so drift checking itself is never weakened, only the persistence step. */
function checkManifestDrift(root, { skipWrite = false } = {}) {
  const current = computeManifestHash();
  const stateFile = manifestHashStatePath(root);
  // Prefer the new .bee/cache/ hash; fall back to a legacy root file once so the
  // first post-#11 call doesn't spuriously report "manifest changed".
  const prior = readJson(stateFile, null) || readJson(legacyManifestHashStatePath(root), null);
  const priorHash = prior && typeof prior.hash === 'string' ? prior.hash : null;
  if (!skipWrite) {
    try {
      writeJsonAtomic(stateFile, { hash: current, checked_at: new Date().toISOString() });
      removeFileIfExists(legacyManifestHashStatePath(root));
    } catch {
      // best-effort persistence only; see doc comment above.
    }
  }
  if (priorHash && priorHash !== current) {
    return {
      manifest_changed: true,
      hint: 'Command registry content changed since the last bee.mjs call — re-run "bee --help --json" to refresh the manifest.',
    };
  }
  return { manifest_changed: false, hint: null };
}

// ─── --help / --help --json: D3 tool-schema-shaped manifest ────────────────

function toManifestEntries(entries) {
  return entries.map(({ name, invoke, description, parameters, examples, deprecated }) => ({
    name,
    invoke,
    description,
    parameters,
    examples,
    deprecated,
  }));
}

function publicManifestEntries() {
  return toManifestEntries(COMMAND_REGISTRY);
}

function renderHelpText(entries = publicManifestEntries()) {
  const lines = [`bee — unified CLI dispatcher (schema_version ${SCHEMA_VERSION})`, ''];
  for (const entry of entries) {
    lines.push(entry.invoke);
    lines.push(`    ${entry.description}`);
    const required = entry.parameters?.required || [];
    if (required.length) lines.push(`    required: ${required.map((r) => `--${r}`).join(', ')}`);
    if (entry.deprecated) {
      lines.push(`    DEPRECATED since ${entry.deprecated.since} — use "${entry.deprecated.use_instead}" instead.`);
    }
    lines.push('');
  }
  return `${lines.join('\n').trimEnd()}\n`;
}

function handleHelp(json) {
  if (json) {
    const manifest = { schema_version: SCHEMA_VERSION, commands: publicManifestEntries() };
    process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
  } else {
    process.stdout.write(renderHelpText());
  }
  return 0;
}

// ─── response emission (stdout's top-level JSON/text shape is ALWAYS the
// bare result — byte-identical to the original CLIs, parity, D5 — regardless
// of drift. P1 fix (review-phase-1.md): a prior version nested the result
// under {manifest_changed, manifest_changed_hint, result} on drift, which
// unpredictably reshaped every data command's output for exactly one call.
// The drift signal now only ever reaches stderr, never stdout, so a
// machine consumer's parsing of stdout never has to account for it.) ───────

function emit({ result, text, exitCode = 0 }, useJson, drift) {
  if (drift && drift.manifest_changed) {
    process.stderr.write(`manifest_changed: true — ${drift.hint}\n`);
  }
  if (useJson) {
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  } else {
    process.stdout.write(`${text}\n`);
  }
  return exitCode;
}

function emitError(message, useJson) {
  if (useJson) process.stdout.write(`${JSON.stringify({ error: message })}\n`);
  else process.stderr.write(`${message}\n`);
  return 1;
}

// ─── main ───────────────────────────────────────────────────────────────────

export async function main(argv) {
  if (argv[0] === '--help') {
    return handleHelp(argv.includes('--json'));
  }

  const { leading, rest } = splitCommandTokens(argv);
  const { commandName, extra } = resolveCommand(leading);
  const jsonRequested = rest.some((t) => t === '--json' || t.startsWith('--json='));

  // Group/command-scoped --help (GH #23): "bee <group> --help" or "bee
  // <group> <verb> --help" renders help filtered to just that group/command,
  // reusing the same publicManifestEntries/renderHelpText shapes as top-level
  // --help. Only fires when commandName resolves to at least one registry
  // entry (itself or a "<commandName>." prefix) — an unrecognized group falls
  // through unchanged to the existing GROUP_USAGE_FALLBACKS / nearest-match
  // error path below, byte-exact (DA5 bijection probe never sends --help).
  if (commandName && rest.includes('--help')) {
    const filtered = COMMAND_REGISTRY.filter(
      (e) => e.name === commandName || e.name.startsWith(`${commandName}.`),
    );
    if (filtered.length > 0) {
      const entries = toManifestEntries(filtered);
      if (jsonRequested) {
        const manifest = { schema_version: SCHEMA_VERSION, commands: entries };
        process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
      } else {
        process.stdout.write(renderHelpText(entries));
      }
      return 0;
    }
  }

  if (!commandName) {
    return emit(
      {
        result: { ok: false, error: { field: null, reason: 'no command given', command: null } },
        text: 'No command given. Try "bee --help".',
        exitCode: 1,
      },
      jsonRequested,
      null,
    );
  }

  let root;
  try {
    root = findRepoRoot(process.cwd());
    if (!root) {
      throw new Error(
        'No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding.',
      );
    }
  } catch (error) {
    return emitError(error instanceof Error ? error.message : String(error), jsonRequested);
  }

  // doctor is read-only FOR REAL (codex-native-runtime-v2 D11): bypass the
  // pre-routing cache write entirely rather than merely best-effort for this
  // one route, since "zero writes" is the command's own contract, not a
  // side effect of a hostile sandbox.
  const drift = checkManifestDrift(root, { skipWrite: commandName === 'doctor' });
  const entry = COMMAND_REGISTRY.find((e) => e.name === commandName);

  if (!entry) {
    // Group-usage fallback (du-1): a leading group token that resolves to no
    // registry entry (bare group, unknown verb, or unknown nested action)
    // emits that group's legacy "Use:" line byte-exact on stderr — the shim
    // always supplies the group token, so the generic no-command path can
    // never fire for helper calls. The full `leading` tokens (not `extra`) are
    // passed so the fallback can reconstruct the attempted verb/sub-action.
    const group = commandName.includes('.') ? commandName.split('.')[0] : commandName;
    const fallback = GROUP_USAGE_FALLBACKS[group];
    if (fallback) {
      return emitError(fallback(leading), jsonRequested);
    }
    const suggestion = nearestCommandName(commandName);
    return emit(
      {
        result: {
          ok: false,
          error: { field: null, reason: `unknown command "${commandName}"`, command: null },
          suggestion,
        },
        text: `Unknown command "${commandName}". Did you mean "${suggestion}"?`,
        exitCode: 1,
      },
      jsonRequested,
      drift,
    );
  }

  // A resolved entry with leftover leading tokens is a stray argument (e.g.
  // "cells ready foo") — refuse before dispatch. Ordered after the group
  // fallback so a nested-action miss ("state worker shave") reaches the
  // fallback's richer legacy message instead of this generic one.
  if (extra.length > 0) {
    return emit(
      {
        result: {
          ok: false,
          error: { field: null, reason: `unexpected argument "${extra[0]}"`, command: commandName },
        },
        text: `Unexpected argument "${extra[0]}" after "${commandName}".`,
        exitCode: 1,
      },
      jsonRequested,
      null,
    );
  }

  const redirect = deprecatedRedirect(entry);
  if (redirect) return emit(redirect, jsonRequested, drift);

  const parsed = parseFlags(rest);
  if (parsed.error) {
    const reason = parsed.error.reason;
    const field = parsed.error.field;
    return emit(
      {
        result: { ok: false, error: { ...parsed.error, command: commandName } },
        text: `Invalid call to "${commandName}": ${reason}${field ? ` (--${field})` : ''}.`,
        exitCode: 1,
      },
      jsonRequested,
      drift,
    );
  }

  // After a successful parse, the authoritative "was --json requested" signal
  // is parsed.json, NOT the pre-parse rest-scan (jsonRequested): a non-boolean
  // flag can consume the "--json" token as its value (e.g. the `worker prune
  // --dryrun --json` typo, where --dryrun eats --json), in which case --json is
  // NOT a real flag and errors must go to stderr — byte-parity with the legacy
  // helpers, which read json only from their own parsed args.
  const useJson = parsed.json;

  // i54-closeout D3: resolve knowledge.context's --lane shorthand into
  // --budget before validate() sees the flags — see
  // resolveKnowledgeContextLaneBudget's own comment for why this must run
  // here rather than inside the handler.
  resolveKnowledgeContextLaneBudget(commandName, parsed.flags);

  const validation = validate(entry, parsed.flags);
  if (!validation.ok) {
    const { command } = validation.error;
    // ce-1: list EVERY problem, not just the first — `error` stays
    // first-problem-shaped in `result` for machine consumers (test_bee_cli.mjs:325
    // pin), the human-facing `text` gets every problem plus a runnable
    // Example line. Channel is unchanged: this is the generic validate()
    // path, which has always emitted via `emit()` (stdout — DB3 leaves this
    // path alone; the three handler-owned verbs never reach here).
    const problems = Array.isArray(validation.problems) && validation.problems.length > 0
      ? validation.problems
      : [validation.error];
    const detail = problems.map((p) => `${p.reason}${p.field ? ` (--${p.field})` : ''}`).join('; ');
    const example = exampleFor(command);
    const exampleLine = example ? ` Example: ${example}` : '';
    return emit(
      {
        result: { ok: false, error: validation.error, problems: validation.problems },
        text: `Invalid call to "${command}": ${detail}.${exampleLine}`,
        exitCode: 1,
      },
      useJson,
      drift,
    );
  }

  const handler = HANDLERS[commandName];
  try {
    // reservations.reserve/release/sweep (D2) run their read-check-write body
    // under withStoreLock, which is async — `handler` may return a plain
    // value or a Promise; `await` resolves either uniformly and still routes
    // a rejection (e.g. LockBusyError) into this same catch, unchanged.
    const response = await handler(root, parsed.flags);
    return emit(response, useJson, drift);
  } catch (error) {
    return emitError(error instanceof Error ? error.message : String(error), useJson);
  }
}

// Guard direct execution vs. import: spawning `bee.mjs` (the real CLI usage,
// and how tests exercise the full dispatch path) runs main(); importing named
// exports for direct unit tests (nearestCommandName, deprecatedRedirect,
// computeManifestHash, parseFlags, ...) must never trigger it as a side effect.
const isDirectRun = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;
if (isDirectRun) {
  // work-visibility D3 (decision 4439bd7e): every direct invocation is
  // wall-timed and fail-open logged, stderr-only — stdout stays byte-identical
  // for every verb (--json consumers and tests never see a "[bee]" line).
  // `cmd` is resolved the SAME way main() resolves it (splitCommandTokens +
  // resolveCommand), computed here — not inside main() — so the guard can
  // time the whole call including argv parsing; 'unknown' covers every
  // resolution miss (no command given, --help's flag-only argv, ...),
  // mirroring "resolved command name, 'unknown' when resolution failed".
  const directArgv = process.argv.slice(2);
  const t0 = Date.now();
  let timingCmd = 'unknown';
  try {
    const { leading } = splitCommandTokens(directArgv);
    const { commandName } = resolveCommand(leading);
    if (commandName) timingCmd = commandName.split('.').join(' ');
  } catch {
    // resolution failure never blocks the run — cmd stays 'unknown'
  }
  const recordTiming = (ok) => {
    const ms = Date.now() - t0;
    // Fail-open append, mirroring hooks/adapter.mjs's appendHookLog: a
    // logging failure (unwritable/missing .bee/logs, no repo root, ...)
    // never changes the command's outcome, exit code, or output.
    try {
      const timingRoot = findRepoRoot(process.cwd()) || process.cwd();
      const logsDir = path.join(timingRoot, '.bee', 'logs');
      fs.mkdirSync(logsDir, { recursive: true });
      fs.appendFileSync(
        path.join(logsDir, 'timings.jsonl'),
        `${JSON.stringify({ ts: new Date().toISOString(), cmd: timingCmd, ms, ok })}\n`,
      );
    } catch {
      // fail-open: never break (or flip) a command over timing-log I/O
    }
    process.stderr.write(`[bee] ${timingCmd} ${ms}ms\n`);
  };
  main(directArgv).then(
    (code) => {
      process.exitCode = code;
      recordTiming(!code);
    },
    (error) => {
      process.stderr.write(`${error instanceof Error ? (error.stack || error.message) : String(error)}\n`);
      process.exitCode = 1;
      recordTiming(false);
    },
  );
}
