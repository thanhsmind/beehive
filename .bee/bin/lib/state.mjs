// state.mjs — repo root discovery, runtime state, config, gates.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { readJson, writeJsonAtomic } from './fsutil.mjs';
// Leaf-module imports only — no cycle: claims.mjs and reservations.mjs import
// only fsutil/lock.mjs/node builtins (unlike cells.mjs, which imports THIS
// file) and never each other or this file, so composing both here stays
// cycle-free (msh-5).
import { readSession, readClaim, isClaimActive, claimsDir, adoptClaim, activeWorkers, heartbeatStale } from './claims.mjs';
import { pathsOverlap, listReservations } from './reservations.mjs';
import { readGrants, decideWorktreeStore, createFeatureWorktree } from './worktree-store.mjs';
// workspace-store.mjs (multisession-native-19) is structurally isolated —
// imports only fsutil.mjs/lock.mjs (its own header's C4 guarantee) — so
// importing it here, same as workflow-store.mjs above, creates no cycle.
// multisession-native-20 (D3) uses it to resolve write-policy ownership at
// this file's own write-capable entry point (startFeature).
import { registerWorkspace, attachWorkspace } from './workspace-store.mjs';
// D6 — startFeature's single read-check-write body runs inside this lock
// (CLI verbs WAIT normally: no maxAttempts override here, unlike the hook-
// driven touch path in claims.mjs/reservations.mjs).
import { withStoreLock } from './lock.mjs';
// decisions.mjs imports only node builtins + fsutil (no cycle back to this file);
// advisorRefAnchors reads the newest active decision id through it (AO13).
import { activeDecisions } from './decisions.mjs';
// workflow-store.mjs (multisession-native-5) is a session/state-free LEAF —
// see its own header's C4 structural guarantee — so importing it here creates
// no cycle: it never imports this file back. startFeature (below) uses it to
// create/seed workflow records alongside its existing legacy-store writes
// (D1, multisession-native-6).
import { createWorkflow, listWorkflows, updateWorkflow } from './workflow-store.mjs';

export const BEE_VERSION = '1.20.3';

export const GATE_NAMES = ['context', 'shape', 'execution', 'review'];

// The phase enum (02-architecture state model). 'compounding-complete' is the
// one blessed terminal alias written at feature close (07-contracts, hook 6).
// Anything else is agent drift — bee_status flags it (decision 0004).
export const PHASES = [
  'idle',
  'exploring',
  'planning',
  'swarming',
  'reviewing',
  'scribing',
  'compounding',
  'grooming',
];
export const KNOWN_PHASES = [...PHASES, 'compounding-complete'];

export function isKnownPhase(phase) {
  return KNOWN_PHASES.includes(phase);
}

// D13 (validation-diet) — a phase value a pre-cut repo's file still carries
// (removed from PHASES/KNOWN_PHASES by the merge of the standalone
// 'validating' stage into 'planning') is coerced, at read, to the phase that
// now carries its role — never re-derived per call site. Mirrors the
// isKnownPhase(phase) ? phase : 'idle' read-coercion precedent at
// ensureWorkflowRecordForFeature below, but preserves the record's INTENT
// (mid-execution-gate work, which 'planning' now carries under the merged
// gate) rather than resetting it to 'idle'.
//
// Applied at readState/readStateStrict (the default store) AND at
// laneRecordFrom (the single merge point shared by readLane AND
// readLaneStrict, so the lane path gets it for free rather than as a second
// copy) — the two read surfaces D13 names. This is why both bee.mjs's
// `state set` pre-mutation isKnownPhase(state.phase) refusal (resolveMutation
// Target reads through readStateStrict/readLaneStrict) and the write guard's
// resolvePipeline-sourced phase (guards.mjs, via readState/readLane) see the
// coerced value automatically: neither module re-implements the mapping.
const LEGACY_PHASE_COERCIONS = { validating: 'planning' };

function coerceLegacyPhase(phase) {
  return Object.prototype.hasOwnProperty.call(LEGACY_PHASE_COERCIONS, phase)
    ? LEGACY_PHASE_COERCIONS[phase]
    : phase;
}

// chain-integrity D1-REVISED — the chain's tail is guarded at the DOOR, not by
// phase name. The enum check above is the ONLY thing that used to stand between
// `swarming` and `compounding-complete`, so a hand-typed close asserted that
// scribing AND compounding had run when neither had.
//
// Why not "compounding only from scribing": nothing in bee ever sets phase
// `scribing` (zero hits repo-wide) — bee-scribing goes straight to `state
// scribing-run`, which produces `compounding` directly. That rule would have
// made `compounding` unreachable. So instead:
//   - `compounding` is not settable at all; only a real scribing run yields it
//   - `scribing-run` demands a phase where execution has actually happened
//   - `compounding-complete` demands `compounding`, a FRESH recorded
//     compounding run (compounding-gate D2, checked right here — no
//     cells.mjs needed, see below), and (in bee.mjs) zero scribing debt
// Everything else stays permissive: every backward move (hive law 5 needs them)
// and `idle`, the de-facto abandon verb.
//
// PURE by necessity: cells.mjs already imports this file, so the scribing-debt
// half of the rule cannot live here — it lives at the bee.mjs choke point. The
// compounding-run freshness half (compounding-gate D2, cell cg-1) reads only
// fields already present on the record passed in (last_compounding_run,
// last_scribing_run) — no cells.mjs access needed — so unlike scribing debt it
// stays right here, pure, alongside the phase-name check it complements.
export const SCRIBING_RUN_FROM = ['swarming', 'reviewing', 'scribing'];

// compounding-gate D1 (cell cg-1) — the compounding-run recorder's own phase
// door, same shape as SCRIBING_RUN_FROM/checkScribingRunPhase below but for
// the sibling verb `state compounding-run`. Unlike scribing-run, a
// compounding run does NOT produce a phase transition — bee-compounding runs
// entirely INSIDE phase "compounding" (stamped by the prior scribing run), so
// the only legal phase to record from is that one phase itself, not a set of
// predecessors.
export const COMPOUNDING_RUN_FROM = ['compounding'];

export function checkPhaseTransition(from, to, record, opts = {}) {
  const current = from || 'idle';
  const waiveCompounding = opts && opts.waiveCompounding === true;
  if (to === 'compounding') {
    return {
      ok: false,
      reason:
        'set: phase "compounding" is not settable directly — it is produced only by RECORDING a real scribing run, never by asserting one. FIX: run `bee state scribing-run --feature <f> --areas "<a,b>" --next-action "<n>"`, which stamps last_scribing_run and advances the phase for you.',
    };
  }
  if (to === 'compounding-complete' && current !== 'compounding') {
    return {
      ok: false,
      reason:
        `set: phase "compounding-complete" may only be entered from "compounding" (current: "${current}"). That name asserts scribing ran, compounding ran, AND the compounding run was RECORDED — not merely asserted; setting it from "${current}" claims work that did not happen and shuts the intake gate on a feature that never closed. FIX: close the chain in order — bee-scribing (\`state scribing-run\`), then bee-compounding (\`state compounding-run\`).`,
    };
  }
  if (to === 'compounding-complete') {
    // compounding-gate D2 (cell cg-1) — the tail guard's SECOND half. The
    // check above only catches a hand-typed close asserting a PHASE that
    // never happened; this one catches a hand-typed close asserting a
    // COMPOUNDING RUN that never happened, even from the correct phase.
    // "Fresh" means a last_compounding_run exists, names the SAME feature as
    // last_scribing_run, and was stamped at or after it — a compounding run
    // left over from a previous scribing pass on this same feature does not
    // count.
    const rec = record || {};
    const run = rec.last_compounding_run;
    const scribing = rec.last_scribing_run;
    const runAtMs = run && typeof run.at === 'string' ? Date.parse(run.at) : NaN;
    const scribingAtMs = scribing && typeof scribing.at === 'string' ? Date.parse(scribing.at) : NaN;
    const fresh = Boolean(
      run &&
        scribing &&
        Number.isFinite(runAtMs) &&
        Number.isFinite(scribingAtMs) &&
        runAtMs >= scribingAtMs &&
        run.feature === scribing.feature,
    );
    if (!fresh && !waiveCompounding) {
      const featureName = (scribing && scribing.feature) || rec.feature || '<f>';
      return {
        ok: false,
        reason:
          `set: phase "compounding-complete" refused — no fresh compounding run recorded for feature "${featureName}" (last_compounding_run must exist, with an "at" timestamp at or after last_scribing_run.at, for the same feature). FIX: run \`bee state compounding-run --feature ${featureName} --learnings <path>\`, then retry. If compounding genuinely needs no separate recorded run here, retry with --waive-compounding — it is permitted, but it logs a decision naming the feature.`,
      };
    }
    return { ok: true, waivedCompounding: !fresh && waiveCompounding };
  }
  return { ok: true, waivedCompounding: false };
}

export function checkScribingRunPhase(from) {
  const current = from || 'idle';
  if (SCRIBING_RUN_FROM.includes(current)) return { ok: true };
  return {
    ok: false,
    reason:
      `scribing-run: refused from phase "${current}" — a scribing run records the spec sync for work that has been EXECUTED. Legal from: ${SCRIBING_RUN_FROM.join(', ')}. FIX: if execution really is done, the phase should say so; if it is not, there is nothing to scribe yet.`,
  };
}

// compounding-gate D1 (cell cg-1) — mirrors checkScribingRunPhase exactly,
// for the compounding-run recorder's own phase door (COMPOUNDING_RUN_FROM
// above).
export function checkCompoundingRunPhase(from) {
  const current = from || 'idle';
  if (COMPOUNDING_RUN_FROM.includes(current)) return { ok: true };
  return {
    ok: false,
    reason:
      `compounding-run: refused from phase "${current}" — a compounding run records the durable-learnings sync for a feature whose scribing has already run. Legal from: ${COMPOUNDING_RUN_FROM.join(', ')}. FIX: if compounding really is underway, the phase should already say so (\`state scribing-run\` is what produces it); if it is not, there is nothing to compound yet.`,
  };
}

// Host-project standard commands (docs/09 item 1, decision D1): the record is
// the primitive — .bee/config.json `commands`, no init.sh, no second location.
// onboard_bee.mjs keeps its OWN copy of this exact array (commandsNotices'
// "any standard command recorded?" check) — test_onboard_bee.mjs asserts the
// two stay byte-identical, so this stays the 4 core lifecycle keys only.
export const COMMAND_KEYS = ['setup', 'start', 'test', 'verify'];

// worktree_companion_start/_end/_mount (worktree-companion-hook): the SAME
// per-repo `commands` record, extended for `worktree new --with-companion` /
// `worktree merge` to compose with a project-configured nested-repo session
// tool (fgos, or anything else) without bee's own code ever naming one. bee
// never assumes what the companion tool is — only that `_start` prints JSON
// `{worktreePath, sessionId?}` to stdout, `_end` accepts a literal `<id>`
// token to substitute, and `_mount` is a relative path inside the worktree.
// Deliberately a SEPARATE list from COMMAND_KEYS above, not merged into it:
// these are optional feature config, not "standard lifecycle commands" —
// onboarding's notice (has the host project recorded setup/test/verify?)
// must not read a companion-only config as satisfying that check.
export const WORKTREE_COMPANION_COMMAND_KEYS = ['worktree_companion_start', 'worktree_companion_end', 'worktree_companion_mount'];

function normalizeCommands(raw) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return {};
  const commands = {};
  for (const key of [...COMMAND_KEYS, ...WORKTREE_COMPANION_COMMAND_KEYS]) {
    if (typeof raw[key] === 'string' && raw[key].trim()) commands[key] = raw[key].trim();
  }
  return commands;
}

// no-test-repos D1 (decision 55b951e1) — a repo declares itself no-test by
// setting commands.verify (or commands.test) to this exact sentinel string.
// Absence keeps its existing meaning (not-captured-yet, unchanged nag) and
// empty strings still normalize away above — the sentinel is a VALUE a
// command key is allowed to hold, never a new key or a new normalization
// branch, so normalizeCommands above needs no change: it already keeps
// 'none' exactly like it keeps any other non-empty string.
export const NO_TEST_SENTINEL = 'none';

export function isNoTestCommand(value) {
  return value === NO_TEST_SENTINEL;
}

// D1/D2 — either commands.verify or commands.test carrying the sentinel
// declares the repo no-test (either alone is sufficient to flip every gate
// this feature touches: inject.mjs's preamble line, cells.mjs's add/update/
// cap waiver). verify wins for MESSAGING (inject.mjs's preamble line names
// commands.verify specifically) but this predicate itself does not care
// which key carries it.
export function isNoTestRepo(config) {
  const commands = (config && config.commands) || {};
  return isNoTestCommand(commands.verify) || isNoTestCommand(commands.test);
}

const DEFAULT_HOOKS = {
  'session-init': true,
  'prompt-context': true,
  'write-guard': true,
  'state-sync': true,
  'chain-nudge': true,
  'session-close': true,
  'tools-logger': true,
};

// Decision 0012 — model tiers, runtime-keyed. bee is dual-runtime, and each
// runtime names its models differently, so the map is keyed by runtime first,
// then tier. `extraction` = cheapest capable, `generation` = mid, `ceiling` =
// the strongest (kept scarce — the orchestrator's own model). A null value
// means "this runtime cannot select a per-agent model" → the tier is enforced
// via read budgets + output caps in the worker prompt instead (Codex today).
// Cells can be tiered at any of these; `ceiling` is a concept ("keep it on the
// session model"), not a configured value (decision 0015).
export const MODEL_TIERS = ['extraction', 'generation', 'ceiling'];
// Only these two are configured — the CHEAPER tiers you downgrade workers to.
// The ceiling is never configured: it is always the session/orchestrator model,
// so it has no entry and resolves to "inherit the session model".
export const CONFIGURABLE_TIERS = ['extraction', 'generation'];
// Decision 0021 (P16) — `review` is a configurable ROLE beside the tiers: the
// model that reviews what generation implemented (reviewing specialists,
// fresh-eyes, plan-checker). Independent reviewer > self-review; a review slot
// stronger than generation catches what the implementer's own model misses.
// null → falls back to the generation tier.
export const CONFIGURABLE_SLOTS = [...CONFIGURABLE_TIERS, 'review'];
// Decision D2 (advisor feature) — `advisor` is normalized alongside the
// configurable slots but is deliberately NOT one of them: CONFIGURABLE_SLOTS
// stays exactly [extraction, generation, review] so resolveTier's slot gate
// and its review-falls-back-to-generation semantics never apply to it
// (decision 0015 collision avoided — the ceiling tier stays unconfigured and
// `advisor` is not a tier either). Only normalizeModels loops this extended
// list; resolveAdvisor (below, beside resolveTier) is the sole reader.
const MODEL_NORMALIZE_SLOTS = [...CONFIGURABLE_SLOTS, 'advisor'];
// Decision 0021 (P17) — per-slot reasoning effort, applied where the runtime
// has a per-agent effort switch; ignored (recorded only) where it does not.
export const EFFORT_LEVELS = ['low', 'medium', 'high', 'xhigh', 'max'];
export const RUNTIMES = ['claude', 'codex'];
const DEFAULT_MODELS = {
  // Claude Code Agent tool accepts short model names: haiku | sonnet | opus | fable.
  // The all-Claude default role split (owner, 2026-07-10): session model
  // orchestrates (ceiling), opus reviews, sonnet implements, haiku extracts —
  // every slot editable per repo to whatever models the user actually has.
  claude: { extraction: 'haiku', generation: 'sonnet', review: 'opus' },
  // Codex has no per-agent model selection today → null tiers = budget/cap fallback.
  // Set real model ids here if your runtime supports switching (e.g. generation: 'gpt-5').
  codex: { extraction: null, generation: null, review: null },
};

// Decisions 0019/0021 (P14/P16/P17) — a configurable slot value is one of:
//   "model-name"                       → the runtime's per-agent model switch
//   null                               → budget/cap fallback (no per-agent
//     switch); for the `review` slot: fall back to the generation tier
//   { model: "...", effort: "..." }    → model + reasoning effort, applied
//     where the runtime has a per-agent effort switch (invalid efforts drop)
//   { kind: "cli", command: "..." }    → an EXTERNAL executor: a separate CLI
//     process (codex exec, a GLM/Kimi CLI, ...) dispatched by the orchestrator
//     under the same bee-executing contract; effort rides inside the command.
// Invalid shapes are ignored (the default for that slot stays).
function normalizeTierValue(value) {
  if (typeof value === 'string' && value.trim()) return value.trim();
  if (value === null) return null;
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    if (value.kind === 'cli' && typeof value.command === 'string' && value.command.trim()) {
      return { kind: 'cli', command: value.command.trim() };
    }
    // Native V2 model-override leaf (D2, codex-native-transport): a per-agent
    // model switch that rides on codex spawn_agent metadata. Preserved through
    // normalize (a shape normalizeTierValue does not keep is stripped, which
    // would make the resolver branch dead code). Invalid fork_turns/effort are
    // silently dropped here exactly as an invalid effort is on {model,effort};
    // validateModelsConfig is the loud layer that flags them.
    if (value.kind === 'native' && typeof value.model === 'string' && value.model.trim()) {
      const out = { kind: 'native', model: value.model.trim() };
      if (typeof value.effort === 'string' && EFFORT_LEVELS.includes(value.effort.trim())) {
        out.effort = value.effort.trim();
      }
      if (typeof value.fork_turns === 'string' && value.fork_turns.trim() === 'none') {
        out.fork_turns = 'none';
      }
      if (typeof value.agent_type === 'string' && value.agent_type.trim()) {
        out.agent_type = value.agent_type.trim();
      }
      return out;
    }
    // Explicit-fallback composite (D1/D2): a native primary plus an opt-in cli
    // fallback. The fallback is kept ONLY when fallback_policy is exactly
    // 'explicit-only' — a composite missing that policy string normalizes to the
    // native primary alone, so no silent native->cli fallback can ever surface.
    if (
      value.primary &&
      typeof value.primary === 'object' &&
      !Array.isArray(value.primary) &&
      value.primary.kind === 'native' &&
      typeof value.primary.model === 'string' &&
      value.primary.model.trim()
    ) {
      const out = { primary: normalizeTierValue(value.primary) };
      if (value.fallback_policy === 'explicit-only') {
        out.fallback_policy = 'explicit-only';
        if (
          value.fallback &&
          typeof value.fallback === 'object' &&
          !Array.isArray(value.fallback) &&
          value.fallback.kind === 'cli' &&
          typeof value.fallback.command === 'string' &&
          value.fallback.command.trim()
        ) {
          out.fallback = { kind: 'cli', command: value.fallback.command.trim() };
        }
      }
      return out;
    }
    if (value.kind === undefined && typeof value.model === 'string' && value.model.trim()) {
      const out = { model: value.model.trim() };
      if (typeof value.effort === 'string' && EFFORT_LEVELS.includes(value.effort.trim())) {
        out.effort = value.effort.trim();
      }
      return out;
    }
  }
  return undefined;
}

// Decision 8cd4c84e / D2b (P18, evolving loop) — dogfood_repos: the foreign repos
// whose ALREADY-WRITTEN .bee/feedback-digest.json bee's evolving loop consumes.
// Each entry normalizes to { path, label }: a bare string is the path (label
// defaults to its basename), or an explicit { path, label } object. Absent key,
// or any other shape, → [] / skipped — never thrown. Every path is path.resolve()d
// THEN fs.realpath()ed here (critical pattern [20260708]: an MSYS /tmp string must
// never reach a node fs API unresolved), and a path that does not exist or is
// unreadable is WARNED and SKIPPED — one dead dogfood repo must never break the
// bee repo's own session. Mirrors normalizeCommands / normalizeModels: a
// single parse path lives in readConfig, nowhere else.
function normalizeDogfoodRepos(raw) {
  if (!Array.isArray(raw)) return [];
  const out = [];
  for (const item of raw) {
    let rawPath = null;
    let label = null;
    if (typeof item === 'string') {
      rawPath = item;
    } else if (item && typeof item === 'object' && !Array.isArray(item) && typeof item.path === 'string') {
      rawPath = item.path;
      if (typeof item.label === 'string' && item.label.trim()) label = item.label.trim();
    } else {
      continue; // any other shape is ignored, never thrown
    }
    if (typeof rawPath !== 'string' || !rawPath.trim()) continue;
    const resolved = path.resolve(rawPath.trim());
    let real;
    try {
      real = fs.realpathSync(resolved);
    } catch (err) {
      // A missing or unreadable dogfood repo is warned and skipped, never thrown.
      // Inside a hook run (BEE_HOOK_CONTEXT set once by hooks/adapter.mjs's
      // readHookContext, before any lib import) the warning is suppressed —
      // it would otherwise prefix every hook's unrelated stdout/stderr — but
      // the entry is still skipped identically. Plain CLI runs (status,
      // onboarding — no adapter) never set the flag and keep the warning.
      if (!process.env.BEE_HOOK_CONTEXT) {
        console.warn(
          `dogfood_repos: skipping "${rawPath}" — ${err && err.code ? err.code : err} (dead or unreadable repo; the bee session continues)`,
        );
      }
      continue;
    }
    out.push({ path: real, label: label || path.basename(resolved) });
  }
  return out;
}

function normalizeModels(raw) {
  const out = {
    claude: { ...DEFAULT_MODELS.claude },
    codex: { ...DEFAULT_MODELS.codex },
  };
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    for (const rt of RUNTIMES) {
      const src = raw[rt];
      if (!src || typeof src !== 'object' || Array.isArray(src)) continue;
      for (const slot of MODEL_NORMALIZE_SLOTS) {
        const value = normalizeTierValue(src[slot]);
        if (value !== undefined) out[rt][slot] = value;
      }
    }
  }
  return out;
}

// config-validate (ao-2ai-1) — a BLOCKLIST OF KNOWN-BAD auto-approve /
// sandbox-bypass flags, NOT a positive read-only guarantee: a command string
// free of every alias below is not proven safe (env-var injection, shell
// wrappers, and provider-specific aliases not yet enumerated here are all out
// of string-inspection reach). This closure only ever grows.
export const UNSAFE_CLI_FLAGS = [
  '--yolo',
  '--dangerously-skip-permissions',
  '--dangerously-bypass-approvals-and-sandbox',
  '--full-auto',
  '-s danger-full-access',
  '--sandbox danger-full-access', // long-form equivalent of `-s danger-full-access`
];

// The slots a models.<runtime>.* value may occupy, validated (mirrors
// MODEL_NORMALIZE_SLOTS above — kept as a separate literal so this validator
// has no dependency on normalizeModels' internal constant name).
const MODEL_VALIDATE_SLOTS = [...CONFIGURABLE_SLOTS, 'advisor'];

// ao-2b-2/AO8 — ADVICE-CLASS slots (advisor, review — every runtime) must
// run read-only: their output is data consulted by a worker or gate, never
// code that edits the repo. This is a SECOND, NARROWER blocklist layered on
// top of UNSAFE_CLI_FLAGS above (which already denies auto-approve flags on
// every cli slot, generation/extraction included) — the same honest framing
// applies: a command string free of every token below is not proven
// read-only, only free of these known write-granting sandbox aliases.
// 'danger-full-access' is deliberately bare (no leading '-s '/'--sandbox ')
// so it also catches the '=' spelling and any future flag-name prefix;
// UNSAFE_CLI_FLAGS' '-s danger-full-access' row still fires alongside it on
// an advice-class slot — both codes are expected together (B6/B7 + this).
export const ADVICE_CLASS_SLOTS = ['advisor', 'review'];
export const ADVICE_CLASS_WRITABLE_TOKENS = [
  '-s workspace-write',
  '--sandbox workspace-write',
  '--sandbox=workspace-write',
  'danger-full-access',
];

/**
 * validateModelsConfig (ao-2ai-1) — loudly flags malformed / prompt-less /
 * unsafe cli-tier config that normalizeTierValue today silently reverts to
 * the seeded default for (a malformed cli value -> normalizeTierValue
 * returns undefined -> normalizeModels never overwrites -> the seeded
 * 'sonnet' stays, no error). Reads the RAW parsed .bee/config.json (never the
 * normalized readConfig() output — normalizeTierValue strips any field this
 * validator needs to inspect, like a declared prompt-transport indicator) so
 * it can see exactly what a human or an agent actually wrote. NEVER throws —
 * a null/wrong-type config, or a malformed models/runtime/slot shape, is
 * itself reported as a problem row, never an exception; this is also read
 * from `bee status` on every session, so it must degrade to a warning, not a
 * crash. Returns an array of { code, runtime, slot, message } rows, empty
 * when nothing is wrong.
 *
 * Checks, per runtime/slot:
 *   (a) a value that looks like a cli executor (carries `kind` and/or
 *       `command`) but is missing `kind:'cli'` or a non-empty `command`
 *       string — today this silently reverts to the seeded default (W-e).
 *   (b) a valid cli value with no explicit prompt-transport indicator
 *       (`promptVia`, a non-empty string) — a prompt-less cli worker starts
 *       with no task (B2). Deliberately does NOT sniff the command string
 *       (e.g. a trailing "-") for a stdin convention: the config must
 *       DECLARE its transport, never leave it inferred.
 *   (c) a cli `command` containing any UNSAFE_CLI_FLAGS alias (B6/B7).
 *   (d) an ADVICE-CLASS slot (advisor, review — every runtime; AO8) whose
 *       `command` contains any ADVICE_CLASS_WRITABLE_TOKENS alias — advice
 *       slots must run read-only, so a write-granting sandbox mode is
 *       refused here even when it is not on the universal UNSAFE_CLI_FLAGS
 *       list. generation/extraction cli slots are NOT advice-class and are
 *       untouched by this check.
 */
export function validateModelsConfig(config) {
  const problems = [];
  // `undefined` means "no config was read at all" (e.g. .bee/config.json
  // does not exist yet) — every other config reader in this file treats a
  // missing file as silently "use defaults", never a warning; callers reading
  // from disk should pass `undefined` as their readJson fallback so a fresh
  // repo with no config.json produces zero problems here. `null` (or any
  // other non-object) is different: something WAS read/passed and it is not
  // usable — that is the malformed-input case this validator must never
  // throw on but must still report.
  if (config === undefined) return problems;
  if (config === null || typeof config !== 'object' || Array.isArray(config)) {
    problems.push({
      code: 'config-malformed',
      runtime: null,
      slot: null,
      message: '.bee/config.json content is null or not an object — models config cannot be validated; defaults apply.',
    });
    return problems;
  }
  const models = config.models;
  if (models === undefined) return problems; // no models key at all — nothing configured, not an error
  if (models === null || typeof models !== 'object' || Array.isArray(models)) {
    problems.push({
      code: 'config-malformed',
      runtime: null,
      slot: null,
      message: '`models` in .bee/config.json is present but not an object — ignored; defaults apply.',
    });
    return problems;
  }
  for (const rt of RUNTIMES) {
    const src = models[rt];
    if (src === undefined) continue;
    if (src === null || typeof src !== 'object' || Array.isArray(src)) {
      problems.push({
        code: 'runtime-malformed',
        runtime: rt,
        slot: null,
        message: `models.${rt} is present but not an object — ignored; defaults apply.`,
      });
      continue;
    }
    for (const slot of MODEL_VALIDATE_SLOTS) {
      const value = src[slot];
      if (value === undefined || value === null) continue; // unset is fine
      if (typeof value === 'string') continue; // plain model name
      if (typeof value !== 'object' || Array.isArray(value)) {
        problems.push({
          code: 'slot-value-malformed',
          runtime: rt,
          slot,
          message: `models.${rt}.${slot} is not a string, object, or null — ignored; defaults apply.`,
        });
        continue;
      }
      // Native V2 model-override shapes (D2, codex-native-transport) — detected
      // BEFORE looksLikeCli (ADVISOR-R2 Δ1). {kind:'native'} carries a `kind`
      // key so it would be mis-flagged cli-malformed; the composite carries no
      // top-level kind/command/model so it would be mis-flagged
      // model-shape-malformed. Each gets its own reject codes.
      const isComposite = 'primary' in value || 'fallback' in value || 'fallback_policy' in value;
      if (isComposite) {
        const primary = value.primary;
        const primaryOk =
          primary &&
          typeof primary === 'object' &&
          !Array.isArray(primary) &&
          primary.kind === 'native' &&
          typeof primary.model === 'string' &&
          primary.model.trim().length > 0;
        if (!primaryOk) {
          problems.push({
            code: 'composite-primary-malformed',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} is a composite (primary/fallback) but its primary is not a valid native override {kind:"native", model} — ignored; today this silently reverts to the seeded default (D2).`,
          });
          continue;
        }
        if (primary.fork_turns !== undefined && primary.fork_turns !== 'none') {
          problems.push({
            code: 'native-fork-turns-unknown',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} composite primary has fork_turns:${JSON.stringify(primary.fork_turns)} — only "none" is valid; a full-history fork rejects model overrides (E2/D2).`,
          });
        }
        if (value.fallback_policy !== 'explicit-only') {
          problems.push({
            code: 'composite-fallback-policy-missing',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} is a composite but has no fallback_policy:"explicit-only" — its cli fallback is silently dropped and no fallback is ever taken; silent native->cli fallback is forbidden (D1). Set fallback_policy:"explicit-only" to opt in.`,
          });
          continue;
        }
        const fb = value.fallback;
        const fbOk =
          fb &&
          typeof fb === 'object' &&
          !Array.isArray(fb) &&
          fb.kind === 'cli' &&
          typeof fb.command === 'string' &&
          fb.command.trim().length > 0;
        if (!fbOk) {
          problems.push({
            code: 'composite-fallback-malformed',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} composite declares fallback_policy:"explicit-only" but its fallback is not a valid cli executor {kind:"cli", command} — the fallback is silently dropped; fix or remove it (D2).`,
          });
        }
        continue;
      }
      if (value.kind === 'native') {
        const modelOk = typeof value.model === 'string' && value.model.trim().length > 0;
        if (!modelOk) {
          problems.push({
            code: 'native-model-missing',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} is a native override (kind:"native") but has no non-empty model — the exact catalog model id is required; today this silently reverts to the seeded default (D2).`,
          });
          continue;
        }
        if (value.fork_turns !== undefined && value.fork_turns !== 'none') {
          problems.push({
            code: 'native-fork-turns-unknown',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} native override has fork_turns:${JSON.stringify(value.fork_turns)} — only "none" is valid; a full-history fork rejects model overrides (E2/D2).`,
          });
        }
        continue;
      }
      const looksLikeCli = 'kind' in value || 'command' in value;
      if (looksLikeCli) {
        const kindOk = value.kind === 'cli';
        const commandOk = typeof value.command === 'string' && value.command.trim().length > 0;
        if (!kindOk || !commandOk) {
          problems.push({
            code: 'cli-malformed',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} looks like a cli executor but is missing kind:"cli" or a non-empty command — today this silently reverts to the seeded default; fix or remove it (W-e).`,
          });
          continue; // malformed shape: nothing further to check on it
        }
        const transportOk = typeof value.promptVia === 'string' && value.promptVia.trim().length > 0;
        if (!transportOk) {
          problems.push({
            code: 'cli-prompt-transport-missing',
            runtime: rt,
            slot,
            message: `models.${rt}.${slot} is a cli executor with no declared prompt transport — set promptVia (e.g. "stdin") so the prompt reliably reaches it; never inferred from the command string (B2).`,
          });
        }
        for (const flag of UNSAFE_CLI_FLAGS) {
          if (value.command.includes(flag)) {
            problems.push({
              code: 'cli-unsafe-flag',
              runtime: rt,
              slot,
              flag,
              message: `models.${rt}.${slot} command contains "${flag}" — a known auto-approve/sandbox-bypass flag; remove it (B6/B7). This is a blocklist of KNOWN-BAD flags, not a positive read-only guarantee.`,
            });
          }
        }
        if (ADVICE_CLASS_SLOTS.includes(slot)) {
          for (const token of ADVICE_CLASS_WRITABLE_TOKENS) {
            if (value.command.includes(token)) {
              problems.push({
                code: 'cli-advice-slot-writable',
                runtime: rt,
                slot,
                flag: token,
                message: `models.${rt}.${slot} is an advice-class cli slot (advisor/review must run read-only, AO8) and its command contains "${token}" — a known write-granting sandbox token; remove it. This is a blocklist of KNOWN write-granting tokens, not a positive read-only guarantee.`,
              });
            }
          }
        }
        continue;
      }
      if (typeof value.model !== 'string' || !value.model.trim()) {
        problems.push({
          code: 'model-shape-malformed',
          runtime: rt,
          slot,
          message: `models.${rt}.${slot} is an object but neither a valid cli executor nor a valid {model} shape — ignored; today this silently reverts to the seeded default.`,
        });
      }
    }
  }
  return problems;
}

// W3 pinned agent files (ao-3b-1) — the tier each rendered .claude/agents/
// bee-*.md belongs to. "ceiling" is deliberately absent: it has no rendered
// agent (it IS the session model).
const AGENT_FILE_TIER = {
  'bee-gather': 'generation',
  'bee-extract': 'extraction',
  'bee-review': 'review',
};

// Extracts the `model:` value out of an agent file's YAML frontmatter with a
// plain regex (no YAML parser dependency — the templates only ever emit a
// flat `key: value` block, ao-3b-1). Never throws: a read failure (missing
// file, permission) reports {found:false}; a present-but-unparseable file
// reports {found:true, model:undefined} so the caller can flag it as
// malformed rather than silently skip it.
function readAgentFileModel(file) {
  let raw;
  try {
    raw = fs.readFileSync(file, 'utf8');
  } catch {
    return { found: false, model: null };
  }
  const frontmatter = /^---\r?\n([\s\S]*?)\r?\n---/.exec(raw);
  const modelLine = frontmatter ? /^model:\s*(.+)$/m.exec(frontmatter[1]) : null;
  return { found: true, model: modelLine ? modelLine[1].trim() : undefined };
}

/**
 * validateAgentFilesDrift (ao-3b-2, AO12) — advisory-only check that each
 * rendered `.claude/agents/bee-*.md`'s `model:` frontmatter still matches
 * the model currently configured for its tier. Deliberately a SEPARATE,
 * root-taking helper: validateModelsConfig above must stay PURE (no root, no
 * fs — it is read from inside bee-model-guard's fail-open hot path, and a
 * throw there is swallowed by the hook's catch, a refusal that refuses
 * nothing). This helper is called only from hosts that already have root
 * (`bee status`, `bee config validate`) — never from a hook.
 *
 * NEVER throws: an absent agent file is clean (nothing rendered yet, or the
 * tier is cli-shaped/unconfigured and onboarding correctly skipped it —
 * AO10/AO11); a present file with unparseable frontmatter is reported as its
 * own problem code, never an exception. Advisory only — this never refuses
 * anything; the dispatch itself is already protected by the guard's
 * marker+param equality rule (AO5), independent of this check.
 *
 * `rawConfig` is the SAME raw `.bee/config.json` payload validateModelsConfig
 * takes (the readRawConfigForValidation undefined/null/object contract) —
 * normalized here through the same normalizeModels() readConfig itself uses,
 * so the comparison is against the EFFECTIVE model (seeded defaults applied),
 * never the raw shape, and never a second disk read of config.json.
 */
export function validateAgentFilesDrift(root, rawConfig) {
  const problems = [];
  const rawModels =
    rawConfig && typeof rawConfig === 'object' && !Array.isArray(rawConfig) ? rawConfig.models : undefined;
  const models = normalizeModels(rawModels);
  for (const [agentName, slot] of Object.entries(AGENT_FILE_TIER)) {
    const file = path.join(root, '.claude', 'agents', `${agentName}.md`);
    const { found, model: fileModel } = readAgentFileModel(file);
    if (!found) continue; // absent is clean — nothing rendered, or a cli/null slot correctly skipped (AO10/AO11)
    if (fileModel === undefined) {
      problems.push({
        code: 'agent-file-malformed',
        agent: agentName,
        slot,
        message: `.claude/agents/${agentName}.md has no readable "model:" frontmatter line — cannot check drift; re-run onboarding to re-render it.`,
      });
      continue;
    }
    let value = models.claude ? models.claude[slot] : null;
    if (value == null && slot === 'review') {
      value = models.claude ? models.claude.generation : null; // review falls back to generation
    }
    let expected = null;
    if (typeof value === 'string') expected = value;
    else if (value && typeof value === 'object' && typeof value.model === 'string') expected = value.model;
    // value.kind === 'cli', or value == null (budget/unconfigured), leaves
    // expected === null: the slot now names no model at all, so ANY model
    // string still in the file is drift (the file should be removed at the
    // next onboarding sync, but that is onboarding's job, not this
    // advisory's — it still surfaces the mismatch).
    if (expected === null) {
      problems.push({
        code: 'agent-file-drift',
        agent: agentName,
        slot,
        message: `.claude/agents/${agentName}.md declares model: "${fileModel}" but the ${slot} slot is now cli-shaped or unconfigured (no model name) — re-run onboarding to remove the stale file.`,
      });
    } else if (expected !== fileModel) {
      problems.push({
        code: 'agent-file-drift',
        agent: agentName,
        slot,
        message: `.claude/agents/${agentName}.md declares model: "${fileModel}" but the configured ${slot} model is "${expected}" — re-run onboarding to re-render it.`,
      });
    }
  }
  return problems;
}

/**
 * Walk up from startDir looking for `.bee/onboarding.json`; if none found
 * anywhere up the tree, walk up again for the first `.git`; else null.
 * (Onboarding marker wins over .git even when .git is closer to startDir.)
 */
export class WorktreeLinkInvalidError extends Error {
  constructor(message) {
    super(message);
    this.name = 'WorktreeLinkInvalidError';
    this.code = 'WORKTREE_LINK_INVALID';
  }
}

function readGitdirFile(file, base) {
  try {
    let raw = fs.readFileSync(file, 'utf8').trim();
    if (!raw) return null;
    if (raw.startsWith('gitdir:')) raw = raw.slice('gitdir:'.length).trim();
    return path.resolve(base, raw.replace(/\\/g, path.sep));
  } catch {
    return null;
  }
}

function locateGitRoot(start) {
  let dir = path.resolve(start || process.cwd());
  while (true) {
    const marker = path.join(dir, '.git');
    if (fs.existsSync(marker)) return { workRoot: dir, marker };
    const parent = path.dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
}

/**
 * Resolve the physical checkout and the single coordination-store root.
 * `resolveRoots` is a compat wrapper (byte-identical shape/throw semantics)
 * around `resolveRootsCore` below — `resolveContext` (multisession-native D2)
 * is built from the SAME core, so there is exactly one git-worktree walk-up
 * classification in this codebase, not two.
 */
function resolveRootsCore(startDir) {
  // A fixture/project may live below an unrelated ancestor that happens to
  // contain a .git directory (for example /tmp/.git). Prefer the nearest
  // onboarding marker when the current directory itself is not a Git root;
  // linked worktrees still have a .git file at their own root and continue
  // through the bidirectional validation below.
  let nearest = path.resolve(startDir || process.cwd());
  while (true) {
    if (fs.existsSync(path.join(nearest, '.bee', 'onboarding.json')) && !fs.existsSync(path.join(nearest, '.git'))) {
      return { storeRoot: nearest, workRoot: nearest, worktreeResolution: 'ordinary' };
    }
    const parent = path.dirname(nearest);
    if (parent === nearest) break;
    nearest = parent;
  }
  const located = locateGitRoot(startDir);
  if (!located) {
    let dir = path.resolve(startDir || process.cwd());
    while (true) {
      if (fs.existsSync(path.join(dir, '.bee', 'onboarding.json'))) {
        return { storeRoot: dir, workRoot: dir, worktreeResolution: 'ordinary' };
      }
      const parent = path.dirname(dir);
      if (parent === dir) break;
      dir = parent;
    }
    return { storeRoot: null, workRoot: null, worktreeResolution: 'ordinary' };
  }
  const { workRoot, marker } = located;
  if (!fs.statSync(marker).isFile()) {
    return { storeRoot: workRoot, workRoot, worktreeResolution: 'ordinary' };
  }

  const gitdir = readGitdirFile(marker, workRoot);
  const invalid = (reason) => {
    throw new WorktreeLinkInvalidError(`${reason} (${marker})`);
  };
  if (!gitdir) return invalid('linked worktree gitdir is missing or malformed');
  const worktreesRoot = path.resolve(gitdir, '..');
  const commonGitDir = path.resolve(worktreesRoot, '..');
  if (path.basename(commonGitDir) !== '.git' || path.basename(worktreesRoot) !== 'worktrees') {
    return invalid('linked worktree gitdir is outside the expected .git/worktrees namespace');
  }
  const id = path.basename(gitdir);
  if (!id || id === '.' || id === '..') return invalid('linked worktree id is empty');
  const reverse = readGitdirFile(path.join(gitdir, 'gitdir'), gitdir);
  if (!reverse || path.resolve(reverse) !== path.resolve(marker)) {
    return invalid('linked worktree reverse gitdir pointer is missing or mismatched');
  }
  const mainRoot = path.dirname(commonGitDir);
  // Opt-in per-worktree store (worktree-feature-parallelism): a worktree whose
  // git-verified id is registered in the MAIN store's grant registry resolves
  // to its own local store; an unregistered id resolves to main (P40 default,
  // byte-for-byte). Grants are read only from the main store, never from
  // anything the worktree claims about itself.
  const storeRoot = readGrants(path.join(mainRoot, '.bee'))[id] === true ? workRoot : mainRoot;
  return { storeRoot, workRoot, worktreeResolution: 'linked-valid', id, mainRoot, worktreeRoot: workRoot };
}

/** Compat wrapper: byte-identical to the original resolveRoots — every
 * existing caller keeps working unchanged. Delegates to the same core
 * resolveContext is built from (see resolveContext below); this function
 * carries no resolution logic of its own anymore. */
export function resolveRoots(startDir) {
  return resolveRootsCore(startDir);
}

/**
 * resolveContext(cwd) — the control-plane/data-plane split (multisession-
 * native D2): `{ projectRoot, controlRoot, workspaceRoot, localRuntimeRoot,
 * gitCommonDir, workspaceId, worktreeId }`. Built on the SAME classification
 * resolveRoots has always computed (resolveRootsCore above) — one canonical
 * git-worktree resolution, never a second copy of the walk-up-and-validate
 * logic (advisor digest slice4, binding condition 5: resolveContext is THE
 * single git-common-dir resolver; see herding.mjs's resolveHerdingMainRoot
 * for the one call site left independent, and why — comment there).
 *
 * - workspaceRoot: the physical checkout (today's resolveRoots `workRoot`).
 * - gitCommonDir: the real `.git` COMMON directory — the checkout's own
 *   `.git` dir when ordinary, the MAIN checkout's `.git` dir when a linked
 *   worktree; null when no git root is reachable at all (the same
 *   onboarding-marker-only fallback resolveRoots already tolerates).
 * - controlRoot: `mainRoot` itself (msn-18a: least-churn reading of D2) —
 *   mainRoot is the MAIN checkout regardless of worktree registration
 *   (control plane is shared across ALL worktrees, not opt-in). Coordination
 *   stores resolve as `<controlRoot>/.bee/...`, exactly the paths the leaf
 *   modules (claims.mjs sessionsDir/claimsDir, workflow-store.mjs
 *   workflowsDir) already build from a bare root — so main-checkout callers
 *   are byte-identical (controlRoot === workspaceRoot === mainRoot there) and
 *   a linked worktree's coordination reads/writes land in the SAME physical
 *   `<mainRoot>/.bee/...` tree main itself uses, with zero path-shape change
 *   to those leaf modules. (An earlier draft nested this under
 *   `mainRoot/.bee/runtime/control`, a store shape nothing else in the
 *   codebase reads or writes — corrected here before any caller shipped
 *   against it.)
 * - localRuntimeRoot: `<workspaceRoot>/.bee/runtime/local` — always the
 *   CURRENT physical checkout, never the main checkout (per-checkout caches
 *   that never need sharing).
 * - projectRoot: today's product-root concept (resolveProductRoot below),
 *   kept distinct from workspaceRoot for the repo-divorce topology (#14).
 * - workspaceId / worktreeId: the MAIN checkout is always `('main', null)`.
 *   A linked worktree always reports its git-verified id as `worktreeId`
 *   (it IS a worktree, registered or not). `workspaceId` only becomes that
 *   same id once the worktree has been REGISTERED as its own workspace
 *   (today's `bee worktree register`/`new` — decideWorktreeStore's
 *   'linked-valid-granted'); an unregistered linked worktree still reports
 *   workspaceId 'main', matching the existing P40 store-fallback default
 *   (worktree-store.mjs decideWorktreeStore/readGrants — the "existing
 *   worktree registry" this field derives from).
 *
 * Returns every field null when nothing is resolvable at all (no git root,
 * no onboarding marker anywhere up the tree) — the same "give up" case
 * resolveRoots has always had (storeRoot/workRoot both null).
 */
export function resolveContext(cwd) {
  const core = resolveRootsCore(cwd);
  if (!core.workRoot) {
    return {
      projectRoot: null,
      controlRoot: null,
      workspaceRoot: null,
      localRuntimeRoot: null,
      gitCommonDir: null,
      workspaceId: null,
      worktreeId: null,
    };
  }

  const workspaceRoot = core.workRoot;
  const isLinked = core.worktreeResolution === 'linked-valid';
  const mainRoot = isLinked ? core.mainRoot : workspaceRoot;

  const ordinaryGitDir = path.join(workspaceRoot, '.git');
  const gitCommonDir = isLinked
    ? path.join(mainRoot, '.git')
    : fs.existsSync(ordinaryGitDir) && fs.statSync(ordinaryGitDir).isDirectory()
      ? ordinaryGitDir
      : null;

  // Reuse the NEW decision layer (worktree-store.mjs) instead of re-deriving
  // "is this worktree registered" by hand — this is decideWorktreeStore's
  // first production call site (previously wired nowhere, per that module's
  // own header comment).
  const classification = {
    kind: core.worktreeResolution,
    id: core.id,
    mainRoot,
    worktreeRoot: isLinked ? core.worktreeRoot : workspaceRoot,
  };
  const grants = readGrants(path.join(mainRoot, '.bee'));
  const decision = decideWorktreeStore(classification, { grants });
  const workspaceId = decision.ok && decision.kind === 'linked-valid-granted' ? decision.id : 'main';
  const worktreeId = isLinked ? core.id : null;

  return {
    projectRoot: resolveProductRoot(workspaceRoot),
    controlRoot: mainRoot,
    workspaceRoot,
    localRuntimeRoot: path.join(workspaceRoot, '.bee', 'runtime', 'local'),
    gitCommonDir,
    workspaceId,
    worktreeId,
  };
}

export function findRepoRoot(startDir) {
  const roots = resolveRoots(startDir);
  return roots.storeRoot;
}

/**
 * controlRootFor(root) — resolveContext(root).controlRoot, with a fallback
 * to `root` itself when nothing is resolvable at all (no git root, no
 * onboarding marker reachable — resolveContext's own "give up" case) so a
 * caller never has to null-check before using the result. This is how
 * state.mjs re-roots its OWN claims/sessions/workflow-store call sites
 * (msn-18a) onto the shared control plane, so a `root` that happens to be a
 * linked worktree's checkout still reaches the SAME store main uses.
 * Main/solo repos: byte-identical to `root` (controlRoot === workspaceRoot
 * there).
 *
 * Exported as of msn-18b: cells.mjs/recovery.mjs/compaction.mjs import this
 * directly (none of them import state.mjs in a way that cycles back here —
 * verified against every module state.mjs itself imports). reservations.mjs
 * CANNOT import this (state.mjs imports reservations.mjs directly — see this
 * file's own `import { pathsOverlap, listReservations } from
 * './reservations.mjs'` above — so the reverse import would cycle); it
 * carries its own minimal, cycle-safe replica instead (see reservations.mjs
 * module header for why, and advisor-digest-slice4 binding condition 6 for
 * the "single git-common-dir resolver, or a tracking note" precedent this
 * follows).
 */
export function controlRootFor(root) {
  return resolveContext(root).controlRoot ?? root;
}

// resolveProductRoot — where the host project's PRODUCT docs live: docs/backlog.md,
// docs/specs/, the product README. This is normally the bee root, but can differ
// when `.bee/` sits in a "workshop" root while the product is an independent nested
// repo one directory down (GitHub #14: the repo-divorce topology). `.bee/config.json`
// `product_root` (a path relative to the bee root, or absolute) overrides it.
//
// Contract:
//   - unset / empty  -> the bee root itself. ZERO behavior change for every ordinary
//                        single-root repo (the freeze this whole change is built on).
//   - set + exists   -> the resolved product directory.
//   - set + missing  -> a LOUD stderr warning (breaking exactly the silent-null outage
//                        #14 is about) and the resolved (non-existent) path is returned,
//                        so product-doc reads find nothing WITH a reason, never crash.
// Runtime state (.bee/*) and bee's own workshop docs (docs/history/*, learnings) never
// route through this — only the product's own docs do.
export function resolveProductRoot(root) {
  const configured = readConfig(root).product_root;
  if (configured === undefined || configured === null || configured === '') return root;
  if (typeof configured !== 'string') {
    console.warn(
      `bee: .bee/config.json product_root must be a string path (got ${typeof configured}); ignoring it and using the bee root.`,
    );
    return root;
  }
  const resolved = path.isAbsolute(configured) ? configured : path.resolve(root, configured);
  let isDir = false;
  try {
    isDir = fs.statSync(resolved).isDirectory();
  } catch {
    isDir = false;
  }
  if (!isDir) {
    console.warn(
      `bee: config product_root "${configured}" -> "${resolved}" is not an existing directory; product-doc reads (docs/backlog.md, docs/specs/) will find nothing until you fix .bee/config.json product_root. (GitHub #14)`,
    );
  }
  return resolved;
}

// cacheFilePath — the home for bee's DERIVED / rebuildable scratch files (inject
// dedup cache, the perf-open session marker, the manifest-hash drift cache). They
// live under `.bee/cache/` (already gitignored + excluded from managed hashing)
// instead of cluttering the `.bee/` root beside durable state (GitHub #11).
export function cacheFilePath(root, name) {
  return path.join(root, '.bee', 'cache', name);
}

export function defaultState() {
  return {
    schema_version: '1.0',
    phase: 'idle',
    feature: null,
    mode: null,
    approved_gates: { context: false, shape: false, execution: false, review: false },
    workers: [],
    summary: '',
    // codex-loop (advisor #54): a fresh/idle record must NOT order a re-route.
    // 'Invoke bee-hive.' here is emitted by the prompt reminder on every turn of an
    // idle session, which reads as 'there is workflow waiting' when there is none —
    // the terminal-points-back-to-hive loop. Idle states what is true: nothing is
    // active, and the next move belongs to the user.
    next_action: 'No active bee work — awaiting a user request.',
  };
}

export function statePath(root) {
  return path.join(root, '.bee', 'state.json');
}

export function readState(root) {
  const state = readJson(statePath(root), null);
  if (!state || typeof state !== 'object' || Array.isArray(state)) return defaultState();
  const merged = { ...defaultState(), ...state };
  merged.approved_gates = { ...defaultState().approved_gates, ...(state.approved_gates || {}) };
  merged.phase = coerceLegacyPhase(merged.phase); // D13: legacy 'validating' -> 'planning'
  return merged;
}

// readStateStrict — the CLI-only sibling of readState (review P1-1). readState
// stays fail-open on purpose: hooks (bee-state-sync, bee-write-guard) and
// `bee.mjs status` read it constantly and must never throw on a corrupt file
// mid-session, so its "present-but-unparseable -> defaultState()" shape is
// untouched here and must stay untouched. readStateStrict is for `bee.mjs
// state`'s mutation verbs only, which is the one place a fail-open read is
// actively harmful: every verb re-reads-then-writes, so a corrupt file
// silently read as defaults gets written straight back as a fresh skeleton —
// gates reset, workers emptied, feature nulled, exit 0, no trace anything was
// lost. readStateStrict instead distinguishes:
//   - absent state.json           -> defaultState() (same as readState; a
//     first mutation on a fresh repo must still be able to create the file)
//   - present but unparseable, or -> throws, so the caller (bee.mjs state's
//     present but not a JSON object    main()) prints the message to stderr and
//                                       exits non-zero with the file untouched
export function readStateStrict(root) {
  const file = statePath(root);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') return defaultState();
    throw new Error(
      `readStateStrict: could not read "${file}" (${err && err.code ? err.code : err}). ` +
        'The bee CLI refuses to rebuild state from defaults when it cannot read the existing file — that could ' +
        'silently clobber real state (gates, workers, feature). ' +
        `FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error(
      `readStateStrict: "${file}" exists but is not valid JSON. ` +
        'The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently ' +
        'clobber real state (gates, workers, feature) while reporting success. ' +
        `FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error(
      `readStateStrict: "${file}" exists but is not a JSON object (found ${Array.isArray(parsed) ? 'an array' : typeof parsed}). ` +
        'The bee CLI refuses to rebuild state from defaults over a present-but-corrupt file — that would silently ' +
        'clobber real state (gates, workers, feature) while reporting success. ' +
        `FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  const merged = { ...defaultState(), ...parsed };
  merged.approved_gates = { ...defaultState().approved_gates, ...(parsed.approved_gates || {}) };
  merged.phase = coerceLegacyPhase(merged.phase); // D13: legacy 'validating' -> 'planning'
  return merged;
}

export function writeState(root, state) {
  writeJsonAtomic(statePath(root), state);
  return state;
}

export function gateApproved(state, gateName) {
  return Boolean(state && state.approved_gates && state.approved_gates[gateName] === true);
}

// ─── handoff kinds (fresh-session-handoff fsh-9, D1) ────────────────────────
// Two kinds, one file (.bee/HANDOFF.json): 'planned-next' (previous cell
// capped with a green verify, next cell already claimed by the writer —
// the only kind a fresh session may act on without confirmation) and 'pause'
// (today's mid-flight-interruption meaning — surface and WAIT, never
// auto-resume). readHandoff stays the fail-open DISPLAY read (unchanged
// shape/behavior for every existing caller) but now normalizes `kind` for
// display: a record with a missing or unknown kind reads as 'pause' — the
// fail-safe that keeps every handoff written before this cell, and any
// record some future bug corrupts, on the safe (surface-and-wait) side.
// writeHandoff/adoptHandoff below are the CLI-owned guarded mutators (hive
// law 12) — HANDOFF.json had no CLI writer before this cell.
export const HANDOFF_KINDS = ['planned-next', 'pause'];

// Exported (multisession-native-15, condition G) so state-projection.mjs's
// rebuildHandoffProjection can rely on the SAME normalization this module's
// own readHandoff uses, rather than reimplementing it — single source of
// truth for the missing/unknown -> 'pause' fail-safe.
export function normalizeHandoffKind(kind) {
  return kind === 'planned-next' ? 'planned-next' : 'pause';
}

export function handoffPath(root) {
  return path.join(root, '.bee', 'HANDOFF.json');
}

export function readHandoff(root) {
  const handoff = readJson(handoffPath(root), null);
  if (!handoff || typeof handoff !== 'object' || Array.isArray(handoff)) return handoff;
  return { ...handoff, kind: normalizeHandoffKind(handoff.kind) };
}

/**
 * writeHandoff — the strict, guarded CLI-owned writer (mirrors this file's
 * own readStateStrict/readLaneStrict throw-on-refusal convention, hence
 * "(strict)" rather than claims.mjs's typed-return convention). --kind is
 * never guessed: the caller must say 'pause' or 'planned-next' explicitly.
 *
 * 'pause' keeps today's free-form shape (whatever fields the caller passes —
 * cell/files/done/remaining/next_action/phase/feature/mode, unchanged) plus
 * the kind and a written_at stamp. No new precondition: this is the same
 * "surface and WAIT" record as always, now CLI-written instead of prose-Write.
 *
 * 'planned-next' is where D1's preconditions live (in the verb, not prose):
 * refuses, with zero mutation, unless BOTH hold —
 *   (a) the named previous_cell's OWN cell record (read directly, not through
 *       lib/cells.mjs — that module imports THIS file, so importing it back
 *       here would cycle; mirrors the existing listAllCellsForStart pattern
 *       below) has status 'capped' and trace.verify_passed === true (the
 *       real cap precondition field capCell/recordVerify enforce);
 *   (b) the named next_cell already has a claim (claims.mjs, the
 *       cross-session primitive) OWNED by the given writer_session — the
 *       "carried claim" that survives the writing session's own /clear.
 * On success the record also stores writer_session/previous_cell/next_cell
 * alongside kind and written_at, so adoptHandoff below has everything it
 * needs without re-deriving anything.
 *
 * DEPRECATION NOTE (multisession-native-24, dated 2026-07-25, advisor
 * consult slice 5 condition E): this direct-file writer stays for exactly
 * ONE more release as the C1 no-workflow-records legacy fallback — the sole
 * production caller is handleStateHandoffWrite in bee.mjs, and ONLY on the
 * branch where resolveHandoffWorkflowId(root, ...) resolves to null (a repo
 * with zero workflow records anywhere). Every repo with at least one live
 * workflow record routes through writeMailboxHandoff instead, and the legacy
 * .bee/HANDOFF.json there is a rebuildHandoffProjection-owned DISPLAY
 * projection, never written here. Removal condition: once no supported repo
 * can lack workflow records (every startFeature call already seeds one via
 * seedLegacyWorkflows — this fallback exists only for a repo that predates
 * multisession-native or was never onboarded through it), this function and
 * its C1 branch in bee.mjs retire together. Until then, writeHandoff plus
 * adoptHandoff's own rmSync below are the ONLY direct legacy-file writers
 * besides rebuildHandoffProjection (state-projection.mjs) — enforced by a
 * grep-audit test (test_state.mjs).
 */
export function writeHandoff(root, input = {}) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) {
    throw new Error('writeHandoff: an object record is required.');
  }
  if (input.kind !== 'planned-next' && input.kind !== 'pause') {
    throw new Error(
      `writeHandoff: --kind must be "planned-next" or "pause" (got ${JSON.stringify(input.kind)}) — D1 requires an explicit kind, never guessed. FIX: pass one of the two handoff kinds.`,
    );
  }
  const now = new Date().toISOString();

  if (input.kind === 'pause') {
    const { kind: _kind, ...rest } = input;
    const record = { ...rest, kind: 'pause', written_at: now };
    writeJsonAtomic(handoffPath(root), record);
    return record;
  }

  // planned-next: every precondition is READ before the single write below
  // (fails closed, zero mutations on refusal — mirrors startFeature's C1
  // discipline elsewhere in this file).
  const writerSession = typeof input.writer_session === 'string' ? input.writer_session.trim() : '';
  const previousCell = typeof input.previous_cell === 'string' ? input.previous_cell.trim() : '';
  const nextCell = typeof input.next_cell === 'string' ? input.next_cell.trim() : '';
  if (!writerSession || !previousCell || !nextCell) {
    throw new Error(
      'writeHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.',
    );
  }

  const previous = readJson(path.join(root, '.bee', 'cells', `${previousCell}.json`), null);
  if (!previous || previous.status !== 'capped' || previous.trace?.verify_passed !== true) {
    throw new Error(
      `writeHandoff: refused — previous cell "${previousCell}" is not capped with a passing verify (found status "${previous?.status ?? 'missing'}", verify_passed ${JSON.stringify(previous?.trace?.verify_passed ?? null)}). A planned-next handoff may only follow a green-verified cap. FIX: cap "${previousCell}" with a recorded passing verify first (bee.mjs cells verify then cap), then retry.`,
    );
  }

  const claim = readClaim(controlRootFor(root), nextCell);
  if (!claim || claim.session !== writerSession) {
    throw new Error(
      `writeHandoff: refused — next cell "${nextCell}" has no claim owned by writer session "${writerSession}" (found ${claim ? `owner "${claim.session}"` : 'no claim'}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim "${nextCell}" as session "${writerSession}" first (claims.mjs claimCellFile), then retry.`,
    );
  }

  const record = {
    ...input,
    kind: 'planned-next',
    writer_session: writerSession,
    previous_cell: previousCell,
    next_cell: nextCell,
    written_at: now,
  };
  writeJsonAtomic(handoffPath(root), record);
  return record;
}

/**
 * adoptHandoff — transfers the carried claim to `sessionId`, then clears the
 * handoff. PRECISION (validation-s4 panel W5): this is CLEAR-AFTER-ADOPT with
 * idempotent recovery, NOT a transaction spanning the claim store and
 * HANDOFF.json — there is no cross-file atomicity here. A crash landing
 * exactly between the two steps (claim adopted, handoff not yet cleared)
 * self-heals on the NEXT adoptHandoff call: re-reading the still-present
 * handoff and re-adopting its next_cell is a BENIGN SELF-ADOPT when the claim
 * already belongs to `sessionId` (adoptClaim has no "already owned" special
 * case — it happily rewrites session -> the same session, refreshing
 * timestamps), and the clear then goes through. Never claim this function is
 * atomic across the two files; it is idempotent, which is what makes the
 * crash window harmless.
 *
 * Typed-failure contract (mirrors claims.mjs, the module this wraps): every
 * refusal returns { ok:false, code, reason } and NEVER throws — a missing
 * handoff, a pause-kind handoff (never auto-resumed — D1's hard boundary),
 * or a failed underlying adoptClaim (propagated as-is) all leave BOTH the
 * claim and the handoff untouched. Only a genuinely bad `sessionId` argument
 * throws (requireId inside adoptClaim), matching claims.mjs's own bad-
 * argument convention.
 *
 * DEPRECATION NOTE (multisession-native-24, dated 2026-07-25): same status
 * and removal condition as writeHandoff's own note above — this is its C1
 * no-workflow-records fallback counterpart, called only from
 * handleStateHandoffAdopt in bee.mjs when resolveHandoffWorkflowId resolves
 * to null. Every live-workflow repo instead calls adoptMailboxHandoff.
 */
export function adoptHandoff(root, sessionId) {
  const handoff = readHandoff(root);
  if (!handoff) {
    return { ok: false, code: 'NO_HANDOFF', reason: 'no .bee/HANDOFF.json to adopt.' };
  }
  if (handoff.kind !== 'planned-next') {
    return {
      ok: false,
      code: 'NOT_PLANNED_NEXT',
      reason: `handoff kind "${handoff.kind}" is not "planned-next" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).`,
    };
  }
  const nextCell = typeof handoff.next_cell === 'string' ? handoff.next_cell.trim() : '';
  if (!nextCell) {
    return { ok: false, code: 'MALFORMED', reason: 'planned-next handoff has no next_cell to adopt.' };
  }

  const adopted = adoptClaim(controlRootFor(root), nextCell, sessionId);
  if (!adopted.ok) {
    // adoptClaim's own typed failure (GATE_HELD / NOT_FOUND) — propagate
    // as-is; neither the claim nor the handoff has been touched by this call.
    return adopted;
  }

  fs.rmSync(handoffPath(root), { force: true });
  return { ok: true, claim: adopted.claim, previous_owner: adopted.previous_owner, next_cell: nextCell };
}

// ─── handoff mailboxes per workflow (multisession-native-15, D5, issue #56
// 3.7/mục 10, advisor consult slice 3 condition G) ──────────────────────────
// Legacy .bee/HANDOFF.json above is ONE mailbox for the whole repo
// (readHandoff/writeHandoff/adoptHandoff, left completely unchanged — still
// the byte-identical path for a repo with no workflow records at all, C1).
// A repo with at least one workflow record (workflow-store.mjs,
// multisession-native-5+) gets a MAILBOX PER WORKFLOW instead:
// .bee/runtime/handoffs/<workflow-id>/<seq>.json, seq zero-padded and
// monotonic per mailbox. bee.mjs resolves WHICH workflow a given `state
// handoff write/show/adopt` call targets (bound session -> its workflow;
// --lane names it; the default record's own live workflow as the last
// resort; null when none of those resolve, which is this module's signal to
// fall back to the single legacy file above) — this module only knows how
// to read/write a mailbox once it has been handed a concrete workflow id.
//
// Each mailbox slot is further scoped by `target_role` (free-form,
// caller-supplied, default null meaning "the unscoped default recipient"):
// writing a NEW record for a given (workflow, target_role) pair marks the
// previous OPEN record for that SAME (workflow, target_role) 'cleared' in
// the same write — the mailbox's answer to the single legacy file's
// implicit overwrite-on-rewrite semantics, so a caller who never names a
// role keeps exactly the old "one active handoff" behavior. A DIFFERENT
// target_role (e.g. a reviewer's handoff alongside an implementer's) gets
// its own independent slot and is never touched by this auto-clear — this
// is what keeps a reviewer handoff from clobbering an implementer handoff
// (D9 invariant, must_have truth), while every record — cleared or not —
// stays on disk under its own seq for audit history (never deleted).
//
// adoptMailboxHandoff mirrors adoptHandoff's own two-step
// (transfer-claim-then-clear) idempotent-recovery contract: a crash between
// the steps leaves the record status 'adopted' (not yet 'cleared'), and the
// next adopt call for that mailbox+role self-heals by finding that same
// 'adopted' record and skipping straight to the clear step (never re-runs
// adoptClaim once status is already 'adopted' — no double fence_epoch bump
// on recovery).
const HANDOFF_SEQ_WIDTH = 4;
export const HANDOFF_MAILBOX_STATUSES = ['open', 'adopted', 'cleared'];

function requireHandoffWorkflowId(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error('handoff mailbox: workflow id is required.');
  }
  const id = value.trim();
  if (/[\\/]/.test(id) || id.includes('..')) {
    throw new Error(`handoff mailbox: workflow id "${id}" must be a plain id (no path separators).`);
  }
  return id;
}

function normalizeTargetRole(role) {
  return typeof role === 'string' && role.trim() ? role.trim() : null;
}

export function handoffMailboxDir(root, workflowId) {
  return path.join(root, '.bee', 'runtime', 'handoffs', requireHandoffWorkflowId(workflowId));
}

function handoffRecordPath(root, workflowId, seq) {
  return path.join(handoffMailboxDir(root, workflowId), `${String(seq).padStart(HANDOFF_SEQ_WIDTH, '0')}.json`);
}

/**
 * listHandoffMailbox(root, workflowId) — every record in a workflow's
 * mailbox, oldest to newest by seq. Fail-open (missing/unreadable dir ->
 * []), matching listWorkflows/readLane's own convention for a store that may
 * not exist yet. A record missing/unparseable is skipped, never thrown for
 * the whole listing. `kind` is normalized the SAME way readHandoff
 * normalizes it (missing/unknown -> 'pause', G's fail-safe) so a caller
 * never has to special-case a raw on-disk value. `seq` is derived from the
 * filename and attached in-memory only — never persisted inside the record
 * body itself (the filename is the one source of truth for it).
 */
export function listHandoffMailbox(root, workflowId) {
  let entries;
  try {
    entries = fs.readdirSync(handoffMailboxDir(root, workflowId));
  } catch {
    return [];
  }
  const records = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const seq = Number.parseInt(entry.slice(0, -'.json'.length), 10);
    if (!Number.isFinite(seq)) continue;
    const raw = readJson(handoffRecordPath(root, workflowId, seq), null);
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) continue;
    records.push({ ...raw, kind: normalizeHandoffKind(raw.kind), seq });
  }
  records.sort((a, b) => a.seq - b.seq);
  return records;
}

/**
 * newestOpenHandoffMailboxRecord(root, workflowId, targetRole) — the newest
 * 'open' record for this workflow, scoped to `targetRole` (null = the
 * default/unscoped slot). Never returns an 'adopted' or 'cleared' record.
 */
export function newestOpenHandoffMailboxRecord(root, workflowId, targetRole = null) {
  const role = normalizeTargetRole(targetRole);
  const records = listHandoffMailbox(root, workflowId);
  for (let i = records.length - 1; i >= 0; i--) {
    const record = records[i];
    if (record.status !== 'open') continue;
    if (normalizeTargetRole(record.target_role) !== role) continue;
    return record;
  }
  return null;
}

// nextHandoffSeq — the highest existing seq + 1 for this mailbox (1 when
// empty). Only ever called from inside withStoreLock('handoff:<id>', ...)
// below, so no locking of its own.
function nextHandoffSeq(root, workflowId) {
  const records = listHandoffMailbox(root, workflowId);
  return records.length === 0 ? 1 : records[records.length - 1].seq + 1;
}

/**
 * writeMailboxHandoff(root, workflowId, input) — the mailbox-per-workflow
 * sibling of writeHandoff above. SAME kind/precondition contract (D1,
 * unchanged): --kind must be 'pause' or 'planned-next' explicitly; a
 * planned-next record requires the previous cell capped with a passing
 * verify AND the next cell's claim owned by the writer session. msn-12's
 * fence_epoch is snapshotted onto the new record's `claim_epoch` field at
 * write time (informational/audit — adoptMailboxHandoff below is what
 * actually moves the claim and re-stamps this field with the bumped value).
 * Every precondition is READ before the single write (fails closed, zero
 * mutation on refusal — same discipline as writeHandoff).
 *
 * `from_session` is the mailbox schema's field name (D5's issue #56 shape);
 * `writer_session` is kept as an alias on the SAME record (both fields, same
 * value, for a planned-next handoff) so a caller written against the legacy
 * `writer_session` name keeps working — a deliberate compatibility
 * duplication, not schema drift.
 *
 * Runs under `withStoreLock(root, 'handoff:<workflowId>')` so two concurrent
 * writers to the SAME workflow's mailbox can never generate the same seq.
 * Writing a new record auto-clears the previous OPEN record for the SAME
 * (workflow, target_role) pair — see this section's header comment for why.
 */
export function writeMailboxHandoff(root, workflowId, input = {}) {
  const wfId = requireHandoffWorkflowId(workflowId);
  if (!input || typeof input !== 'object' || Array.isArray(input)) {
    throw new Error('writeMailboxHandoff: an object record is required.');
  }
  if (input.kind !== 'planned-next' && input.kind !== 'pause') {
    throw new Error(
      `writeMailboxHandoff: --kind must be "planned-next" or "pause" (got ${JSON.stringify(input.kind)}) — D1 requires an explicit kind, never guessed. FIX: pass one of the two handoff kinds.`,
    );
  }
  const role = normalizeTargetRole(input.target_role);
  const now = new Date().toISOString();

  let fields;
  if (input.kind === 'pause') {
    const { kind: _kind, target_role: _role, from_session: rawFromSession, ...rest } = input;
    const fromSession = typeof rawFromSession === 'string' && rawFromSession.trim() ? rawFromSession.trim() : null;
    fields = { ...rest, kind: 'pause', from_session: fromSession };
  } else {
    const writerSession = typeof input.writer_session === 'string' ? input.writer_session.trim() : '';
    const previousCell = typeof input.previous_cell === 'string' ? input.previous_cell.trim() : '';
    const nextCell = typeof input.next_cell === 'string' ? input.next_cell.trim() : '';
    if (!writerSession || !previousCell || !nextCell) {
      throw new Error(
        'writeMailboxHandoff: a planned-next handoff requires non-empty writer_session, previous_cell, and next_cell (D1) — FIX: pass all three.',
      );
    }
    const previous = readJson(path.join(root, '.bee', 'cells', `${previousCell}.json`), null);
    if (!previous || previous.status !== 'capped' || previous.trace?.verify_passed !== true) {
      throw new Error(
        `writeMailboxHandoff: refused — previous cell "${previousCell}" is not capped with a passing verify (found status "${previous?.status ?? 'missing'}", verify_passed ${JSON.stringify(previous?.trace?.verify_passed ?? null)}). A planned-next handoff may only follow a green-verified cap. FIX: cap "${previousCell}" with a recorded passing verify first (bee.mjs cells verify then cap), then retry.`,
      );
    }
    const claim = readClaim(controlRootFor(root), nextCell);
    if (!claim || claim.session !== writerSession) {
      throw new Error(
        `writeMailboxHandoff: refused — next cell "${nextCell}" has no claim owned by writer session "${writerSession}" (found ${claim ? `owner "${claim.session}"` : 'no claim'}). The next cell must already be claimed by the writing session before a planned-next handoff carries it. FIX: claim "${nextCell}" as session "${writerSession}" first (claims.mjs claimCellFile), then retry.`,
      );
    }
    const {
      kind: _kind,
      target_role: _role,
      writer_session: _ws,
      from_session: _fs,
      previous_cell: _pc,
      next_cell: _nc,
      ...rest
    } = input;
    fields = {
      ...rest,
      kind: 'planned-next',
      writer_session: writerSession,
      from_session: writerSession,
      previous_cell: previousCell,
      next_cell: nextCell,
      claim_epoch: Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : 1,
    };
  }

  return withStoreLock(root, `handoff:${wfId}`, () => {
    const seq = nextHandoffSeq(root, wfId);
    // Auto-clear the previous open record for this SAME (workflow, role) —
    // see the section header. Never touches a different role's record.
    const prior = newestOpenHandoffMailboxRecord(root, wfId, role);
    if (prior) {
      const { seq: priorSeq, ...priorRest } = prior;
      writeJsonAtomic(handoffRecordPath(root, wfId, priorSeq), {
        ...priorRest,
        status: 'cleared',
        cleared_at: now,
      });
    }
    const record = {
      id: `${wfId}-${String(seq).padStart(HANDOFF_SEQ_WIDTH, '0')}`,
      workflow_id: wfId,
      target_role: role,
      status: 'open',
      written_at: now,
      ...fields,
    };
    writeJsonAtomic(handoffRecordPath(root, wfId, seq), record);
    return { ...record, seq };
  });
}

/**
 * adoptMailboxHandoff(root, workflowId, sessionId, { targetRole } = {}) —
 * the mailbox-per-workflow sibling of adoptHandoff above. Finds the newest
 * record for `(workflowId, targetRole)` that is still 'open' OR already
 * 'adopted' (the self-heal case — see this section's header) and:
 *   - refuses NO_HANDOFF if none exists,
 *   - refuses NOT_PLANNED_NEXT if the found record's kind isn't
 *     'planned-next' (a pause handoff is never adopted, D1),
 *   - otherwise transfers the carried claim (adoptClaim, which stamps the
 *     bumped fence_epoch per msn-12) — skipped when the record is already
 *     'adopted' (self-heal, the claim already moved) — then marks the
 *     record 'cleared'.
 * Never throws (mirrors adoptHandoff/claims.mjs's typed-failure contract
 * for a REFUSAL); a malformed `sessionId` still throws via adoptClaim's own
 * requireId, same as adoptHandoff.
 */
export function adoptMailboxHandoff(root, workflowId, sessionId, { targetRole = null } = {}) {
  const wfId = requireHandoffWorkflowId(workflowId);
  const role = normalizeTargetRole(targetRole);
  return withStoreLock(root, `handoff:${wfId}`, () => {
    const records = listHandoffMailbox(root, wfId);
    let candidate = null;
    for (let i = records.length - 1; i >= 0; i--) {
      const record = records[i];
      if (normalizeTargetRole(record.target_role) !== role) continue;
      if (record.status === 'open' || record.status === 'adopted') {
        candidate = record;
        break;
      }
    }
    if (!candidate) {
      return {
        ok: false,
        code: 'NO_HANDOFF',
        reason: `no open handoff in workflow "${wfId}"'s mailbox${role ? ` (role "${role}")` : ''} to adopt.`,
      };
    }
    if (candidate.kind !== 'planned-next') {
      return {
        ok: false,
        code: 'NOT_PLANNED_NEXT',
        reason: `handoff kind "${candidate.kind}" is not "planned-next" — a pause handoff is never adopted, it must be surfaced and WAITED on (D1).`,
      };
    }
    const nextCell = typeof candidate.next_cell === 'string' ? candidate.next_cell.trim() : '';
    if (!nextCell) {
      return { ok: false, code: 'MALFORMED', reason: 'planned-next handoff has no next_cell to adopt.' };
    }
    const { seq, ...rest } = candidate;
    let claim;
    let previousOwner = candidate.adopted_previous_owner ?? null;
    if (candidate.status === 'open') {
      const adopted = adoptClaim(controlRootFor(root), nextCell, sessionId);
      if (!adopted.ok) return adopted; // untouched, propagate typed refusal as-is
      claim = adopted.claim;
      previousOwner = adopted.previous_owner;
      writeJsonAtomic(handoffRecordPath(root, wfId, seq), {
        ...rest,
        status: 'adopted',
        claim_epoch: adopted.claim.fence_epoch,
        adopted_by: sessionId,
        adopted_at: new Date().toISOString(),
        adopted_previous_owner: previousOwner,
      });
    } else {
      // Self-heal: the claim already moved on a prior (crashed-before-clear)
      // call — never re-adopt (would double-bump fence_epoch for nothing).
      claim = readClaim(controlRootFor(root), nextCell);
    }
    writeJsonAtomic(handoffRecordPath(root, wfId, seq), {
      ...rest,
      status: 'cleared',
      claim_epoch: claim && Number.isFinite(claim.fence_epoch) ? claim.fence_epoch : candidate.claim_epoch,
      adopted_by: sessionId,
      adopted_at: candidate.adopted_at ?? new Date().toISOString(),
      adopted_previous_owner: previousOwner,
      cleared_at: new Date().toISOString(),
    });
    return { ok: true, claim, previous_owner: previousOwner, next_cell: nextCell, workflow_id: wfId, seq };
  });
}

// ─── lanes: per-feature pipeline records beside the default state.json ──────
// (fresh-session-handoff fsh-3, decisions D2/D4). A lane record carries the
// same core as the default record — feature, mode, phase (closed vocabulary),
// approved_gates (all four), summary, next_action — plus created_at, and lives
// at .bee/lanes/<feature>.json. Lanes are ADDITIVE: a repo with no .bee/lanes/
// and no bound session behaves byte-identically to the single-pipeline model
// (D4 zero-lane parity). state.json remains the DEFAULT lane, authoritative
// whenever no session→lane binding says otherwise.

export function lanesDir(root) {
  return path.join(root, '.bee', 'lanes');
}

// Mirrors claims.mjs requireId: the feature becomes a filename under
// .bee/lanes/, so path separators and '..' are bad arguments (throw), while
// spaces and unicode are ordinary feature names.
function requireLaneFeature(value) {
  if (typeof value !== 'string' || !value.trim()) {
    throw new Error('lane feature is required.');
  }
  const feature = value.trim();
  if (/[\\/]/.test(feature) || feature.includes('..')) {
    throw new Error('lane feature must be a plain id (no path separators).');
  }
  return feature;
}

export function lanePath(root, feature) {
  return path.join(lanesDir(root), `${requireLaneFeature(feature)}.json`);
}

function defaultLaneRecord(feature) {
  return {
    schema_version: '1.0',
    feature,
    mode: null,
    phase: 'idle',
    approved_gates: { context: false, shape: false, execution: false, review: false },
    summary: '',
    next_action: '',
    created_at: null,
  };
}

// null when the parsed content is not a lane record for THIS feature: not a
// JSON object, or a record whose feature field names another feature (a
// mismatched record is corrupt, never trusted — mirrors readSession's id check).
function laneRecordFrom(feature, parsed) {
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return null;
  if (parsed.feature !== feature) return null;
  const merged = { ...defaultLaneRecord(feature), ...parsed };
  merged.approved_gates = { ...defaultLaneRecord(feature).approved_gates, ...(parsed.approved_gates || {}) };
  merged.phase = coerceLegacyPhase(merged.phase); // D13: legacy 'validating' -> 'planning', shared by readLane + readLaneStrict
  return merged;
}

// readLane — fail-open DISPLAY read, the lane sibling of readState's fail-open
// discipline with one deliberate difference: there is no per-feature default
// to fall back to, so "missing" is null, and "present but corrupt" is WARNED
// and skipped (null) — a display surface keeps rendering the healthy lanes,
// and a corrupt record is never guessed at (mutations go through
// readLaneStrict, which refuses loudly instead).
export function readLane(root, feature) {
  let file;
  try {
    file = lanePath(root, feature); // fail-open: a malformed name reads as "no lane"
  } catch {
    return null;
  }
  if (!fs.existsSync(file)) return null;
  const record = laneRecordFrom(String(feature).trim(), readJson(file, null));
  if (!record) {
    console.warn(
      `readLane: skipping corrupt lane record "${path.relative(root, file)}" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}").`,
    );
    return null;
  }
  return record;
}

// readLaneStrict — the mutation sibling (mirrors the readStateStrict
// discipline): a missing lane reads as null (creation is the caller's explicit
// move — a lane is never implicitly defaulted into existence), while a
// present-but-unreadable/corrupt record THROWS with the file untouched, so no
// lane mutation can silently clobber real lane state (gates, phase).
export function readLaneStrict(root, feature) {
  const id = requireLaneFeature(feature); // bad names throw, matching claims.mjs requireId
  const file = lanePath(root, id);
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch (err) {
    if (err && err.code === 'ENOENT') return null;
    throw new Error(
      `readLaneStrict: could not read lane record "${file}" (${err && err.code ? err.code : err}). The bee CLI refuses to mutate a lane it cannot read — that could silently clobber real lane state (gates, phase). FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  let parsed;
  let parseFailed = false;
  try {
    parsed = JSON.parse(text);
  } catch {
    parseFailed = true;
  }
  const record = parseFailed ? null : laneRecordFrom(id, parsed);
  if (!record) {
    throw new Error(
      `readLaneStrict: lane record "${file}" exists but is corrupt (not a JSON object naming feature "${id}"). The bee CLI refuses to rebuild a lane from defaults over a present-but-corrupt file — that would silently clobber real lane state (gates, phase) while reporting success. FIX: inspect/restore the file (e.g. "git checkout -- ${path.relative(root, file)}"), then retry.`,
    );
  }
  return record;
}

export function writeLane(root, lane) {
  if (!lane || typeof lane !== 'object' || Array.isArray(lane)) {
    throw new Error('writeLane: a lane record object is required.');
  }
  writeJsonAtomic(lanePath(root, lane.feature), lane);
  return lane;
}

export function removeLane(root, feature) {
  fs.rmSync(lanePath(root, feature), { force: true });
}

/** Fail-open enumeration for display: corrupt records are warned and skipped by readLane. */
export function listLanes(root) {
  let entries;
  try {
    entries = fs.readdirSync(lanesDir(root));
  } catch {
    return [];
  }
  const lanes = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const record = readLane(root, entry.slice(0, -'.json'.length));
    if (record) lanes.push(record);
  }
  return lanes;
}

/**
 * resolvePipeline — the ONE reader seam between a session and its pipeline
 * record (D2/D4): session record → bound lane → default state.json. Resolution
 * NEVER guesses and never scans: no sessionId, no session record, or no lane
 * binding all mean the default pipeline view. A binding that names a lane
 * which is invalid, missing, or corrupt is a TYPED refusal
 * ({ ok:false, code, reason, feature }) — silently falling back to the default
 * there would point a bound session at the wrong pipeline's gates.
 * Returns { ok:true, source:'default'|'lane', feature?, record } on success.
 *
 * multisession-native-10 (session→workflow binding): `session.lane` is kept
 * as the field name (an intentional alias — no session-record schema break),
 * but what it names is now, in effect, a WORKFLOW-backed feature: since
 * msn-7, every write to `.bee/lanes/<feature>.json` routes through the live
 * workflow record naming that feature (state-projection.mjs's "record
 * wins"), so reading the lane file here already reflects the workflow
 * record's current content — no extra I/O or extra lock is added on this
 * hot hook-driven read path (bee-prompt-context.mjs's try-once/no-wait
 * discipline is unchanged by this cell). The refusal codes below
 * (LANE_INVALID/LANE_MISSING/LANE_CORRUPT) are unchanged in name and
 * trigger, but now, in substance, ARE the missing/corrupt-workflow refusal
 * for a bound session — never a silent fallback to the default pipeline.
 */
export function resolvePipeline(root, { sessionId = null } = {}) {
  const defaults = () => ({ ok: true, source: 'default', record: readState(root) });
  if (typeof sessionId !== 'string' || !sessionId.trim()) return defaults();
  // Sessions and lanes are coordination-store state (msn-18a): resolved
  // through controlRootFor so a `root` that is a linked worktree's own
  // checkout still finds the SAME session/lane records main sees — a
  // worktree-bound session must resolve its lane, not hard-deny on it
  // (advisor-digest-slice4 binding condition 2). Main/solo repos:
  // controlRootFor(root) === root, byte-identical.
  const controlRoot = controlRootFor(root);
  const session = readSession(controlRoot, sessionId);
  if (!session) return defaults();
  const bound = typeof session.lane === 'string' ? session.lane.trim() : '';
  if (!bound) return defaults();
  let file;
  try {
    file = lanePath(controlRoot, bound);
  } catch (err) {
    return {
      ok: false,
      code: 'LANE_INVALID',
      feature: bound,
      reason: `session "${session.id}" is bound to lane "${bound}", which is not a valid lane name (${err instanceof Error ? err.message : err}) — never guessed back to the default pipeline. FIX: rebind or unbind the session (claims.mjs bindSessionLane/unbindSessionLane).`,
    };
  }
  if (!fs.existsSync(file)) {
    return {
      ok: false,
      code: 'LANE_MISSING',
      feature: bound,
      reason: `session "${session.id}" is bound to lane "${bound}" but ${path.relative(controlRoot, file)} does not exist — resolution never guesses back to the default pipeline. FIX: start the lane (startFeature with lane mode) or unbind the session.`,
    };
  }
  const record = readLane(controlRoot, bound);
  if (!record) {
    return {
      ok: false,
      code: 'LANE_CORRUPT',
      feature: bound,
      reason: `session "${session.id}" is bound to lane "${bound}" but its record is corrupt — display never guesses and mutations must refuse. FIX: inspect/restore ${path.relative(controlRoot, file)}, then retry.`,
    };
  }
  return { ok: true, source: 'lane', feature: bound, record };
}

export function readOnboarding(root) {
  return readJson(path.join(root, '.bee', 'onboarding.json'), null);
}

// config.local.json path (hardening-8 config overlay, D-local-config-overlay)
// — a per-machine, gitignored sibling of the TRACKED .bee/config.json. Lets a
// machine-specific value (today: dogfood_repos absolute paths) live outside
// the tracked file so a shared repo's config.json never commits one
// contributor's local filesystem layout.
export function localConfigPath(root) {
  return path.join(root, '.bee', 'config.local.json');
}

// mergeConfigOverlay — deep-merge `overlay` OVER `base` (overlay wins on every
// conflict). Plain objects merge key-by-key, recursively; arrays REPLACE
// wholesale (never concatenated/deep-merged element-by-element — an overlay
// listing 2 dogfood_repos must fully replace the tracked list's 3, never
// interleave them). A non-object/non-array overlay value (string, number,
// boolean, null) simply replaces the base value at that key. `base` is never
// mutated — every level that changes is a fresh object/array.
export function mergeConfigOverlay(base, overlay) {
  if (Array.isArray(overlay)) return overlay.slice();
  if (overlay && typeof overlay === 'object') {
    const baseObj = base && typeof base === 'object' && !Array.isArray(base) ? base : {};
    const out = { ...baseObj };
    for (const [key, value] of Object.entries(overlay)) {
      if (
        value &&
        typeof value === 'object' &&
        !Array.isArray(value) &&
        baseObj[key] &&
        typeof baseObj[key] === 'object' &&
        !Array.isArray(baseObj[key])
      ) {
        out[key] = mergeConfigOverlay(baseObj[key], value);
      } else if (Array.isArray(value)) {
        out[key] = value.slice();
      } else {
        out[key] = value;
      }
    }
    return out;
  }
  // overlay is not an object/array (undefined, null, or a scalar at the root)
  // — nothing to merge; base passes through untouched.
  return base;
}

export function readConfig(root) {
  const raw = readJson(path.join(root, '.bee', 'config.json'), null);
  const tracked = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : {};
  // Overlay wins; an absent/malformed overlay file leaves `config` exactly
  // the tracked object — byte-identical to pre-overlay readConfig for every
  // repo that has never written .bee/config.local.json (D4 zero-overlay
  // parity, mirrors the lanes/worktree additive-by-default discipline
  // elsewhere in this file).
  const overlayRaw = readJson(localConfigPath(root), null);
  const overlay = overlayRaw && typeof overlayRaw === 'object' && !Array.isArray(overlayRaw) ? overlayRaw : null;
  const config = overlay ? mergeConfigOverlay(tracked, overlay) : tracked;
  // Advisor mode is removed in full (D1, reverses decisions 0013/0015). A
  // stale `advisor` key left over in a repo's .bee/config.json must be
  // TOLERATED, never thrown on — but it must not flow through this spread
  // into the parsed result (a bare `...config` would let it ride through
  // untouched). Destructure it out here; onboard_bee.mjs/`bee.mjs status` warn
  // (never error) when the raw file still carries the key.
  const { advisor: _staleAdvisor, ...rest } = config;
  return {
    ...rest,
    hooks: { ...DEFAULT_HOOKS, ...(config.hooks || {}) },
    lanes: config.lanes || {},
    capabilities: config.capabilities || {},
    commands: normalizeCommands(config.commands),
    models: normalizeModels(config.models),
    dogfood_repos: normalizeDogfoodRepos(config.dogfood_repos),
  };
}

export function hookEnabled(root, name) {
  const config = readConfig(root);
  return config.hooks[name] !== false;
}

// ── Local-only config namespaces (D2, intake-gate-git-exemption) ───────────
// guards.* / hooks.* are machine-local safety toggles (idle_gate, per-hook
// enable/disable, ...). `bee config set/unset` routes these two namespaces
// to the gitignored overlay ONLY — never to the tracked, team-shared
// .bee/config.json — so a temporary local safety lift is structurally
// incapable of reaching a teammate. This is the exact defect behind
// incident a7d2069 (corrected in 63a41e0): the documented escape hatch
// (`bee config set --key guards.idle_gate --value false`) wrote into the
// tracked file, and a commit staged before the gate was restored shipped
// the intake gate disabled to everyone. Read precedence is UNCHANGED —
// readConfig() above already lets the overlay win over the tracked value
// for every namespace, this one included; only the WRITE destination moves.
export const LOCAL_ONLY_CONFIG_NAMESPACES = ['guards', 'hooks'];

export function isLocalOnlyConfigKey(keyPath) {
  const top = String(keyPath).split('.')[0];
  return LOCAL_ONLY_CONFIG_NAMESPACES.includes(top);
}

// One-line warning surfaced by `bee config get/set/unset` when a guards.*/
// hooks.* key is still found sitting in the TRACKED config.json (e.g. from
// before this routing existed, or hand-edited back in). The value keeps
// working — read precedence is unchanged — but per CONTEXT D2's open
// question this is NEVER auto-migrated or auto-edited; a human moves it
// deliberately.
export function trackedLocalOnlyKeyWarning(keyPath) {
  return `config: "${keyPath}" is set in the TRACKED .bee/config.json — guards.*/hooks.* values are machine-local only now (.bee/config.local.json). It still works (read precedence unchanged), but this CLI will never write to or remove it from the tracked file again; move it yourself.`;
}

// ── Gate-bypass autopilot levels (total-autopilot, decision dcf01d7b) ────────
// The `.bee/config.json` `gate_bypass` value normalizes into one of four
// levels. Backward compatible: the legacy boolean `true` maps to 'normal' (the
// original tiny/small/standard bypass); `false`/absent/unknown map to 'off'.
//   off    — every gate stops for the human (default; also plain `false`)
//   normal — Gates 1-3 auto-approved for tiny/small/standard non-hard-gate
//            work; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop
//   full   — ALL Gates 1-3 auto-approved incl. high-risk/hard-gate work; only
//            secret-file reads and a review P1 finding still stop
//   total  — ZERO stops: every gate at every lane, secret-file reads, Gate 4
//            UAT, and review P1 findings auto-proceed; no human checkpoint left
export const BYPASS_LEVELS = ['off', 'normal', 'full', 'total'];

export function bypassLevel(root) {
  const raw = readConfig(root).gate_bypass;
  if (raw === 'total') return 'total';
  if (raw === 'full') return 'full';
  if (raw === true || raw === 'on' || raw === 'normal') return 'normal';
  return 'off';
}

// One canonical loud banner per active level, shared by `bee status` and the
// session preamble so bypass reads identically wherever it is surfaced. 'off'
// renders no banner (empty string) — callers omit the line entirely.
export function bypassBanner(level) {
  switch (level) {
    case 'total':
      return '⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-bypass-gate off';
    case 'full':
      return '⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-3 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-bypass-gate off';
    case 'normal':
      return '⚡ GATE BYPASS: NORMAL — Gates 1-3 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop. Turn off: bee-bypass-gate off';
    default:
      return '';
  }
}

// ── Ship visibility (spec #81 P1, sv-1 scope) ───────────────────────────────
// `.bee/config.json` `ship_visibility` surfaces cap-time visibility behavior
// (routing-and-contracts.md "Ship visibility"). This cell (sv-1) only wires
// the config key through to `status --json` and the session preamble; the
// full contract's three-value default (draft-pr/push-only/off with remote
// detection) is out of scope here — sv-1 recognizes exactly 'off' (default,
// absent) and 'draft-pr'. An unrecognized value normalizes to 'off' with a
// one-line stderr warning, never a crash — same discipline as bypassLevel
// above, plus the warn-don't-throw precedent at STALE_ADVISOR_KEY_WARNING
// below.
export const SHIP_VISIBILITY_VALUES = ['off', 'draft-pr'];

export function shipVisibility(root) {
  const raw = readConfig(root).ship_visibility;
  if (raw === undefined || raw === null) return 'off';
  if (SHIP_VISIBILITY_VALUES.includes(raw)) return raw;
  process.stderr.write(
    `config: unrecognized ship_visibility "${raw}" in .bee/config.json — normalized to "off". Allowed: ${SHIP_VISIBILITY_VALUES.join(', ')}.\n`,
  );
  return 'off';
}

// D1 — one shared warning line for both surfacers (`bee.mjs status` and
// onboard_bee.mjs) so a stale `advisor` key reads identically wherever it is
// noticed. Warn only, never error: readConfig above already tolerates the key.
// Names the TOP-LEVEL key explicitly (advisor feature, D2 open question) so
// it cannot be misread as covering the new models.<runtime>.advisor slot,
// which is a different, still-valid config path resolved by resolveAdvisor.
export const STALE_ADVISOR_KEY_WARNING =
  'advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)';

export function hasStaleAdvisorKey(root) {
  const raw = readJson(path.join(root, '.bee', 'config.json'), null);
  return Boolean(raw && typeof raw === 'object' && !Array.isArray(raw) && 'advisor' in raw);
}

/**
 * Resolve tier → model name for a runtime (decision 0012). Returns the
 * configured model, or null when the runtime cannot switch models per agent
 * (caller then enforces the tier via read budget + output cap in the prompt).
 * Unknown runtime falls back to 'claude'; unknown tier to 'generation'.
 */
export function modelForTier(root, tier, runtime = 'claude') {
  // The ceiling tier is never configured — it is always the session/orchestrator
  // model (decision 0015). null means "inherit the session model" (omit the
  // subagent model param). Only generation/extraction resolve to a pinned model.
  // A cli-executor tier (decision 0019) has no model NAME — callers that can
  // dispatch externally should use resolveTier(); here it degrades to null.
  const resolved = resolveTier(root, tier, runtime);
  return resolved.type === 'model' ? resolved.model : null;
}

/**
 * Typed slot resolution (decisions 0019/0021), purpose-scoped (AO12/B1,
 * plan.md 2A-ii). `slot` is a tier (extraction/generation/ceiling) or the
 * `review` role. `purpose` is an OPTIONAL 4th param shaped `{for:'gather'|
 * 'cell'}`, DEFAULT `'cell'` — the fail-safe side: every existing 3-arg
 * call, a missing/null `purpose`, and ANY malformed/unknown `purpose` value
 * (a string, a number, an array, `{for:'banana'}`, etc.) all resolve as
 * cell-execution. Only an explicit `{for:'gather'}` is treated as a
 * read-only gather. Returns one of:
 *   { type: 'inherit' }                — ceiling: omit the model param, the
 *     worker inherits the session model (decision 0015)
 *   { type: 'model', model, effort? }  — spawn a subagent with this model
 *     (and per-agent reasoning effort where the runtime supports it)
 *   { type: 'budget' }                 — no per-agent switch: enforce the tier
 *     as a read budget + output cap in the worker prompt
 *   { type: 'cli', command }           — dispatch an EXTERNAL executor process
 *     (protocol: bee-swarming reference, External Executors section) — ONLY
 *     when the resolved slot value is cli-shaped AND purpose is explicitly
 *     `{for:'gather'}`.
 *   { type: 'refused', reason: 'cli_tier_gather_only', slot, fix } — the
 *     resolved slot value is cli-shaped but purpose is 'cell' (default,
 *     explicit, or malformed). This is a RETURNED TYPE, never a throw — a
 *     cli tier resolving for cell execution is not yet proven safe (Discovery-2:
 *     an external CLI's cwd is not the repo root, so the execute contract's
 *     reserve/verify/cap helpers would misfire silently); it stays refused
 *     until a cell-execution dogfood is green (plan 2A/W9). `modelForTier`
 *     (bee-model-guard's fail-open hot path, :133) calls resolveTier with no
 *     purpose and keeps returning `null` for cli exactly as before this
 *     change — a refused type is not a 'model' type, so its `resolved.type
 *     === 'model'` check still falls through to null, zero behavior change.
 * The refusal applies to EVERY slot including 'review' — a bare 3-arg
 * resolve of a cli-shaped review slot refuses too; the external-reviewer
 * dispatch stays reachable via `{for:'gather'}` (the routing prose that
 * teaches callers the 4-arg form moves to 2A-iii). Non-cli slot values
 * (inherit/model/budget) ignore purpose entirely — byte-identical results
 * with or without it. A null `review` slot still falls back to the
 * generation tier (decision 0021) BEFORE the cli/purpose check.
 */
export function resolveTier(root, slot, runtime = 'claude', purpose) {
  if (slot === 'ceiling') return { type: 'inherit' };
  const { models } = readConfig(root);
  const rt = RUNTIMES.includes(runtime) ? runtime : 'claude';
  const s = CONFIGURABLE_SLOTS.includes(slot) ? slot : 'generation';
  let value = models[rt] ? models[rt][s] : null;
  if (value == null && s === 'review') {
    value = models[rt] ? models[rt].generation : null; // review falls back to generation
  }
  if (value == null) return { type: 'budget' };
  if (typeof value === 'string') return { type: 'model', model: value };
  if (value.kind === 'cli') {
    const forGather =
      purpose != null &&
      typeof purpose === 'object' &&
      !Array.isArray(purpose) &&
      purpose.for === 'gather';
    if (!forGather) {
      return {
        type: 'refused',
        reason: 'cli_tier_gather_only',
        slot: s,
        fix: 'declare {for:"gather"} for a read-only gather; cli cell execution stays refused until a cell-execution dogfood is green (plan 2A/W9)',
      };
    }
    return { type: 'cli', command: value.command };
  }
  // Native V2 model-override (D2, codex-native-transport) — inserted BEFORE the
  // generic value.model branch (plan cnt-1 note) so a native leaf is never
  // mis-read as a plain {model} shape. normalize guarantees a valid model here.
  if (value.kind === 'native') return nativeResolved(value);
  if (value.primary && typeof value.primary === 'object' && !Array.isArray(value.primary)) {
    const resolved = nativeResolved(value.primary);
    if (value.fallback_policy === 'explicit-only' && value.fallback && value.fallback.kind === 'cli' && typeof value.fallback.command === 'string') {
      resolved.fallback = { type: 'cli', command: value.fallback.command };
    }
    return resolved;
  }
  if (typeof value.model === 'string') {
    return value.effort
      ? { type: 'model', model: value.model, effort: value.effort }
      : { type: 'model', model: value.model };
  }
  return { type: 'budget' };
}

/**
 * Resolve the advisor slot (decision D2, advisor feature) for a runtime:
 * `models.<runtime>.advisor`, the `review`-slot shape reused for a new
 * purpose. Unlike resolveTier, this NEVER returns a budget type and NEVER
 * falls back to another tier — null unambiguously means "no advisor" (unset,
 * invalid, or a cli shape missing its command), which is exactly what a
 * degenerate-consult check (D2/D3) needs to skip straight to `[BLOCKED]`.
 * Deliberately NOT routed through resolveTier: `advisor` is not in
 * CONFIGURABLE_SLOTS (decision 0015 collision avoided), so resolveTier would
 * silently coerce an unrecognized slot to 'generation' — the one behavior
 * this function must never exhibit.
 */
export function resolveAdvisor(root, runtime = 'claude') {
  const { models } = readConfig(root);
  const rt = RUNTIMES.includes(runtime) ? runtime : 'claude';
  const value = models[rt] ? models[rt].advisor : undefined;
  if (value == null) return null; // unset, absent runtime, or explicit null -> no advisor
  if (typeof value === 'string') return { type: 'model', model: value };
  if (value.kind === 'cli') return { type: 'cli', command: value.command };
  // Native V2 model-override + explicit-fallback composite (D2), inserted BEFORE
  // the generic value.model branch (plan cnt-1 note). An invalid native shape
  // is stripped by normalize before it reaches here, so resolveAdvisor never
  // returns a bogus native — it falls through to null, exactly like a cli slot
  // missing its command.
  if (value.kind === 'native') return nativeResolved(value);
  if (value.primary && typeof value.primary === 'object' && !Array.isArray(value.primary)) {
    const resolved = nativeResolved(value.primary);
    if (value.fallback_policy === 'explicit-only' && value.fallback && value.fallback.kind === 'cli' && typeof value.fallback.command === 'string') {
      resolved.fallback = { type: 'cli', command: value.fallback.command };
    }
    return resolved;
  }
  if (typeof value.model === 'string') {
    return value.effort
      ? { type: 'model', model: value.model, effort: value.effort }
      : { type: 'model', model: value.model };
  }
  return null;
}

// Shared resolution of a normalized native V2 model-override leaf
// ({kind:'native', model, effort?, fork_turns?, agent_type?}) into a resolved
// {type:'native', ...} record (D2). Called by resolveTier and resolveAdvisor;
// a function declaration so it is hoisted above both. normalize has already
// trimmed the fields and dropped any invalid ones, so this only applies the
// resolved defaults: fork_turns:'none' (overrides require it, E2) and
// agent_type:'worker'.
function nativeResolved(value) {
  const out = { type: 'native', model: value.model };
  if (value.effort != null) out.effort = value.effort;
  out.fork_turns = value.fork_turns != null ? value.fork_turns : 'none';
  out.agent_type = value.agent_type != null ? value.agent_type : 'worker';
  return out;
}

// ─── advisor_ref staleness anchors + check (AO3/AO13, Slice 4) ──────────────
// Zero precedent: no gate anywhere checked a precondition before this. The
// advisor_ref field records that a real advisor consult happened for the
// SELECTED (default or lane) record; Gate 3 refuses high-risk execution
// approval when the ref is missing or stale. Staleness is NEVER a TTL — AO13
// bans invented time numbers. Both helpers are pure reads and never throw on a
// missing artifact: a missing plan.md hashes to the absent sentinel, so the
// only failure mode is "stale", never a crash on the gate's hot path.

export const ADVISOR_PLAN_ABSENT_SENTINEL = 'absent';

function advisorPlanPath(root, feature) {
  return path.join(root, 'docs', 'history', String(feature ?? ''), 'plan.md');
}

// The verb stamps these itself at record time; the caller never supplies them.
// `feature` is the SELECTED record's feature (a lane's own feature, not the
// default record's — checker M1), so the plan.md hashed is THAT feature's plan.
export function advisorRefAnchors(root, feature) {
  let newest_decision_id = null;
  try {
    const active = activeDecisions(root, { recent: 1 });
    newest_decision_id = active.length ? active[0].id : null;
  } catch {
    newest_decision_id = null;
  }
  let plan_sha256 = ADVISOR_PLAN_ABSENT_SENTINEL;
  try {
    const planFile = advisorPlanPath(root, feature);
    if (fs.existsSync(planFile)) {
      plan_sha256 = crypto.createHash('sha256').update(fs.readFileSync(planFile)).digest('hex');
    }
  } catch {
    plan_sha256 = ADVISOR_PLAN_ABSENT_SENTINEL;
  }
  return { feature: feature ?? null, newest_decision_id, plan_sha256 };
}

// AO13 verbatim: an advisor_ref is stale if ANY of — its feature differs from
// state.feature; the newest active decision id changed since the consult;
// sha256(plan.md) changed; or the ref predates the most recent revocation of
// the execution gate. A malformed/missing ref (non-object, array, or null)
// reads as missing — stale, never a throw.
export function advisorRefStale(root, ref, state) {
  if (!ref || typeof ref !== 'object' || Array.isArray(ref)) {
    return { stale: true, reasons: ['no advisor_ref recorded'] };
  }
  const reasons = [];
  const feature = state && state.feature != null ? state.feature : null;
  const anchors = advisorRefAnchors(root, feature);
  if (ref.feature !== anchors.feature) {
    reasons.push(`feature changed since the consult (ref "${ref.feature}" ≠ current "${anchors.feature}")`);
  }
  if (ref.newest_decision_id !== anchors.newest_decision_id) {
    reasons.push(
      `a new decision was logged since the consult (ref "${ref.newest_decision_id}" ≠ current "${anchors.newest_decision_id}")`,
    );
  }
  if (ref.plan_sha256 !== anchors.plan_sha256) {
    reasons.push('plan.md changed since the consult (sha256 mismatch)');
  }
  const revokedAt = state && state.gate_revoked_at ? state.gate_revoked_at.execution : undefined;
  if (revokedAt && (!ref.consulted_at || String(ref.consulted_at) < String(revokedAt))) {
    reasons.push(
      `the consult predates the most recent execution-gate revocation (consulted ${ref.consulted_at ?? 'never'}, revoked ${revokedAt})`,
    );
  }
  return { stale: reasons.length > 0, reasons };
}

// ─── startFeature: guarded atomic feature start (decision D2, plan.md test ──
// matrix row 5 / codex-runtime-parity). ONE atomic operation for beginning a
// new feature so a new feature can never inherit approvals or silently step
// over abandoned work (plan-review.md P1: "feature start could clear evidence
// of active work"). Fails closed — every precondition is read BEFORE any
// write, so a refusal makes ZERO mutations — unless ALL of the following hold:
//   - the CURRENT phase is 'idle' or the terminal alias 'compounding-complete'
//     (a mid-flight prior feature must finish or be explicitly wound down —
//     never silently stepped over by starting a new one)
//   - no .bee/HANDOFF.json exists (a paused session must resume/close first)
//   - no OTHER live worker remains (D6, multisession-native-8: derived from
//     live-heartbeat sessions x claims via claims.mjs activeWorkers — never
//     the hand-mutated state.workers array; the calling session's OWN
//     heartbeat is excluded, C3, so a solo starter is never blocked by
//     itself. A worker with a stale heartbeat drops out on its own; there is
//     no "clear it first" step to take anymore.)
//   - no active (unreleased, unexpired) reservation exists
//   - no cell anywhere has status 'claimed' (live worker state; only cap/drop
//     end a claim)
//   - no cell belonging to the PRIOR feature (state.feature) is in a
//     nonterminal status (open/claimed/blocked) — capped and dropped are the
//     only terminal statuses. An abandoned cell must first be resolved through
//     the EXISTING drop verb (bee.mjs cells drop --id ID --reason R) —
//     startFeature never auto-clears workers/cells/reservations as cleanup
//     (P1 repair, plan-review.md).
// On success, exactly one atomic write sets feature/mode/phase, resets ALL
// FOUR gates to false, and refreshes summary/next_action.
//
// Self-contained by design: cells.mjs already imports readState/gateApproved/
// MODEL_TIERS from this module, so this function reads .bee/cells/*.json
// directly (small local helper below) rather than importing lib/cells.mjs,
// avoiding a state.mjs <-> cells.mjs import cycle. (The pure pathsOverlap
// predicate IS imported from reservations.mjs — that module imports only
// fsutil/lock/claims/lease-store, no cycle back to this file.)
//
// multisession-native-16 (advisor consult slice 3, condition C, BINDING):
// listActiveReservationsForStart used to read `.bee/reservations.json`
// directly. That file is now a rebuildable PROJECTION (msn-16 demoted
// reservations.mjs's storage to lease-store.mjs's sharded per-resource lease
// files) that is only ever refreshed on demand — reading it here would open
// exactly the "lazy-projection gap" condition C calls out: reserve() could
// succeed (a live lease acquired) while the stale projection file still
// showed no conflict, letting startFeature green-light a colliding start.
// Condition C's two options were (a) synchronously rebuild the projection
// inside every reserve/release call, or (b) migrate this direct reader onto
// the shim's own list API — (b) was chosen: listReservations() reads
// lease-store's live files directly, so this precondition always sees
// reserve()'s true, current state with no rebuild step, and reserve/release/
// renew/sweep never pay for a synchronous whole-projection rewrite on their
// own hot path (this cell's other binding requirement).
//
// LANE MODE (fresh-session-handoff fsh-3, validated Q4): startFeature with
// { lane: true } starts the feature AS a lane record under .bee/lanes/ while
// the default pipeline and every other lane stay byte-untouched. The default
// (non-lane) path below keeps today's byte-identical semantics apart from
// F5's handoff rescoping (multisession-native-6 — see startFeature's own
// header comment). Lane-scoped preconditions — attribution DERIVED from
// existing fields, never new ones:
//   (a) nonterminal cells whose cell.feature equals THIS lane's feature block;
//   (b) the global HANDOFF blocks a lane start only when its feature field
//       names this lane's feature (F5: the default start now matches this
//       exact per-feature scoping instead of any-handoff-blocks);
//   (c) an OTHER live worker (D6, multisession-native-8 derived view — never
//       state.workers) blocks only when its current claimed cell derives to
//       this lane's feature (worker session → claim → cell.feature); the
//       calling session's own heartbeat is excluded (C3), same as (e) below;
//   (d) global holds check: when the caller declares intended paths, any
//       overlap with ANOTHER session's active holds — claimed cells' files or
//       active reservations — refuses (own-session claims and expired holds
//       never block; no declared paths, no check);
//   (e) (multisession-native-6) the shared workflow-precondition layer —
//       checkNoLiveWorkflowForFeature/checkNoSameFeatureClaimedCells — same
//       as the default path; normally shadowed by (a)'s broader scan but
//       closes the one gap (a) cannot see: a live workflow record for this
//       feature with no lane file of its own (e.g. it was started via the
//       default path, or already dropped).

// ─── review-p1-fixes p2-1 — THE DEBT CORE, shared by every door ────────────
//
// A second reviewer walked straight through the previous round's fix, and the
// reason was structural, not a missed line: each door carried its OWN copy of
// "does this feature still owe proof?", each keyed on its own ALLOWLIST of
// origin phases. An allowlist of origins loses this game by construction —
// every new phase and every new entry point is a fresh door somebody has to
// remember, and the reviewer's repro (cap `--feature-verify-pending` in
// `compounding`, walk out through `compounding -> compounding-complete`, then
// `state start-feature --feature beta`) simply used two of the doors nobody
// had written down yet. The rule is inverted here, and stated exactly once:
//
//     Debt is cleared by EVIDENCE, never by the absence of evidence.
//
// Three consequences the callers inherit instead of re-implementing:
//
//   (1) The guarded set is DERIVED, not enumerated. Debt lives on CELLS, and
//       the cell store is phase-independent: a cell can hold a pending cap or
//       an unfinished test obligation in ANY phase the record occupies, so
//       EVERY departure is guarded (isDebtGuardedDeparture below), the sole
//       exception being a literal no-op re-set. A phase added to KNOWN_PHASES
//       tomorrow inherits the guard instead of escaping it.
//
//   (2) An UNREADABLE cell store is UNKNOWN debt, never zero debt. Both guards
//       used to read through cells.mjs's listCells — which swallows every fs
//       error and returns [] — behind a further `catch { return; }`, so
//       `chmod 000 .bee/cells` read as "this feature owes nothing" and opened
//       both doors at once. readFeatureCellsStrict refuses instead, naming the
//       store. A store that does not EXIST is the one honest zero: a directory
//       that was never created holds no cells, which is evidence, not absence.
//
//   (3) Every door asks the SAME question. The computation lives here once;
//       callers own only their refusal head (bee.mjs's two phase doors and the
//       swap door, startFeature's outgoing-feature door below), and share the
//       FIX tails exported alongside it. Two doors cannot drift apart when
//       there is only one answer to drift from.
//
// It lives in state.mjs, not cells.mjs, for the cycle reason this file already
// documents at listAllCellsForStart: cells.mjs imports state.mjs, so state.mjs
// can never import it back — and startFeature below is itself one of the doors
// that must ask.

/**
 * readFeatureCellsStrict(root, feature) — the cell reader that FAILS CLOSED.
 * Returns the cell objects belonging to `feature` (all cells when `feature` is
 * null). Throws, naming the store and what is wrong with it, when the store
 * exists but cannot be read in full — an unreadable file, a malformed file, or
 * an unreadable directory. A MISSING directory returns [] (see (2) above).
 */
export function readFeatureCellsStrict(root, feature = null) {
  const dir = path.join(root, '.bee', 'cells');
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch (err) {
    if (err && err.code === 'ENOENT') return []; // no store ⇒ zero cells ⇒ zero debt
    throw new Error(cellStoreUnreadableMessage(dir, `the directory itself could not be read (${(err && err.code) || (err && err.message) || 'unknown error'})`));
  }
  const cells = [];
  const unreadable = [];
  for (const entry of entries) {
    if (entry.isDirectory()) continue; // `archive` (or any dir) is never a cell — mirrors cells.mjs listCells
    if (!entry.name.endsWith('.json')) continue;
    const file = path.join(dir, entry.name);
    let raw;
    try {
      raw = fs.readFileSync(file, 'utf8');
    } catch (err) {
      unreadable.push(`${entry.name} (${(err && err.code) || 'unreadable'})`);
      continue;
    }
    let cell;
    try {
      cell = JSON.parse(raw);
    } catch {
      unreadable.push(`${entry.name} (malformed JSON)`);
      continue;
    }
    if (!cell || typeof cell !== 'object' || Array.isArray(cell)) {
      unreadable.push(`${entry.name} (not a cell object)`);
      continue;
    }
    if (feature && cell.feature !== feature) continue;
    cells.push(cell);
  }
  if (unreadable.length > 0) {
    throw new Error(
      cellStoreUnreadableMessage(dir, `${unreadable.length} cell file(s) could not be read: ${unreadable.join(', ')}`),
    );
  }
  return cells;
}

function cellStoreUnreadableMessage(dir, detail) {
  return (
    `refused — the cell store at "${dir}" is UNREADABLE: ${detail}.\n` +
    'bee cannot prove this feature owes no unpaid debt (pending feature-verify caps, unfinished test cells) from a store it cannot read, and an unreadable store is UNKNOWN debt, never zero debt — debt is cleared by evidence, never by the absence of evidence.\n' +
    'FIX: repair or restore the cell store (check permissions on the directory and each named file, and that every cell file holds valid JSON), then retry. There is no bypass level and no waiver flag for this: passing here would mean asserting a fact bee just failed to read.'
  );
}

/**
 * isDebtGuardedDeparture(fromPhase, targetPhase) — the DERIVED guarded set.
 * True for every phase change; false only for a literal no-op re-set. An
 * absent/unknown `fromPhase` counts as a departure (fail closed): a record too
 * damaged to name its own phase is not evidence that its cells owe nothing.
 */
export function isDebtGuardedDeparture(fromPhase, targetPhase) {
  return String(targetPhase) !== String(fromPhase);
}

// Mirror of cells.mjs's deriveChangeClass (four lines, same cycle reason as
// listAllCellsForStart below). Kept behaviour-identical deliberately: an
// explicit `change_class` wins, the ONLY derivation is
// behavior_change:true ⇒ 'behavior', everything else is unclassified.
function cellChangeClass(cell) {
  if (!cell || typeof cell !== 'object') return null;
  if (typeof cell.change_class === 'string' && cell.change_class) return cell.change_class;
  return cell.behavior_change === true ? 'behavior' : null;
}

/**
 * featureVerifyDebt(root, record, feature) — main-verifies D3's debt question.
 * Returns null when the door is clear (no pending caps, or a fresh green
 * record already covers them), otherwise `{pending, recordState}`. Throws when
 * the cell store is unreadable.
 *
 * Takes the feature as a PARAMETER rather than reading record.feature: the
 * swap and start-feature doors exist precisely because the record's feature is
 * about to be replaced, and the debt they must weigh is the OUTGOING one's.
 */
export function featureVerifyDebt(root, record, feature) {
  if (!feature) return null; // no feature ⇒ nothing keyed to hold open
  const cells = readFeatureCellsStrict(root, feature);
  const pending = [];
  const unrecorded = [];
  let newestOwedCapMs = null;
  for (const cell of cells) {
    if (!cell || cell.status !== 'capped') continue;
    const trace = cell.trace || {};
    // worker-conformance D11/D12 — TWO markers, ONE debt.
    //
    //   feature_verify: "pending"  proof deliberately RELOCATED to the feature
    //                              boundary (main-verifies D1);
    //   proof: "unrecorded"        proof never recorded AT ALL — a cap that
    //                              asserted a pass with no output and no
    //                              verification_evidence, stamped by capCell
    //                              after its whole refusal chain has run (D14).
    //
    // This door cannot tell them apart and must not try: in both cases the one
    // thing that can still prove the cell is the feature-level green run, so
    // both arm it. The second marker is what keeps D1 survivable — once the
    // per-cell evidence doors stop refusing, an unproven cap carries no
    // "pending" marker, and a door armed on "pending" alone would let a
    // feature close with zero tests executed anywhere.
    const relocated = trace.feature_verify === 'pending';
    const unproven = trace.proof === 'unrecorded';
    if (!relocated && !unproven) continue;
    pending.push(cell.id);
    if (unproven) unrecorded.push(cell.id);
    // …and the freshness clock runs over the UNION of both, which is the
    // load-bearing half (D12): reading it over the relocated caps alone lets a
    // green record newer than the newest PENDING cap but older than a newer
    // UNRECORDED cap look fresh, and open the door on a cell that run never
    // covered.
    const cappedMs = Date.parse(trace.capped_at);
    if (Number.isFinite(cappedMs) && (newestOwedCapMs === null || cappedMs > newestOwedCapMs)) {
      newestOwedCapMs = cappedMs;
    }
  }
  if (pending.length === 0) return null; // no proof owed at the boundary ⇒ door untouched
  const raw = record && record.feature_verify;
  const rec = raw && typeof raw === 'object' && !Array.isArray(raw) ? raw : null;
  const sameFeature = rec && (!rec.feature || rec.feature === feature);
  const green = rec && sameFeature && rec.result === 'green';
  const recAtMs = rec ? Date.parse(rec.at) : NaN;
  const fresh = Number.isFinite(recAtMs) && (newestOwedCapMs === null || recAtMs > newestOwedCapMs);
  if (green && fresh) return null;
  let recordState;
  if (!rec) {
    recordState = 'NO feature-verify record exists';
  } else if (!sameFeature) {
    recordState = `the recorded feature-verify names feature "${rec.feature}", not "${feature}"`;
  } else if (rec.result !== 'green') {
    recordState = `the recorded feature-verify is "${rec.result}" — a red record documents the failure, it never satisfies this door (main-verifies D2)`;
  } else {
    recordState = `the recorded GREEN feature-verify (at ${rec.at}) is STALE — not newer than the newest cap still owing proof (pending OR unrecorded), so at least one capped cell was never covered by it`;
  }
  return { pending, unrecorded, recordState };
}

/**
 * testCellDebt(root, feature) — slice-tail-test-batching P4's debt question.
 * Returns null when the feature owes no test coverage, otherwise
 * `{kind, offenders, cappedBehaviorBearing}` where kind is:
 *   'missing'   — capped behavior/api cells exist and NO test cell does at all;
 *   'not-green' — a test cell exists but is not capped, or capped red.
 * Throws when the cell store is unreadable.
 */
export function testCellDebt(root, feature) {
  if (!feature) return null; // no feature ⇒ nothing to hold open
  const cells = readFeatureCellsStrict(root, feature);
  const offenders = [];
  // fs-3 — the OTHER half of the same rule. The original guard only counted
  // test cells that EXIST, so a feature that never emitted one at all left
  // swarming clean: the guarantee rested on planning REMEMBERING to schedule
  // the cell, which is prose, not machine. (Observed live on flow-speedup
  // itself: two capped behavior cells, zero test cells, swarming → scribing
  // with no complaint.) These two collect the debt the 'missing' kind reads.
  let testCellCount = 0;
  const cappedBehaviorBearing = [];
  for (const cell of cells) {
    if (!cell) continue;
    const changeClass = cellChangeClass(cell);
    if (changeClass === 'test') {
      // worker-conformance wc-2c — a DROPPED test cell is WITHDRAWN work: it
      // owes nothing, and it stands in for nothing. Before this skip the
      // branch read one dropped cell two contradictory ways at once — it
      // counted toward testCellCount (suppressing the 'missing' kind below)
      // AND landed in offenders through the `!== 'capped'` arm as "not
      // capped". Found live on this feature's own close-door.
      //
      // The ORDER is the whole guard against this becoming an escape hatch:
      // skipping BEFORE the counter increments means a feature that drops its
      // only test cell falls through to 'missing' rather than passing clean,
      // so dropping the cell is never cheaper than writing it.
      //
      // Only 'dropped' — 'open', 'claimed' and 'blocked' are genuinely
      // undischarged work somebody still owes, and must keep refusing.
      if (cell.status === 'dropped') continue;
      testCellCount += 1;
      const trace = cell.trace || {};
      if (cell.status !== 'capped') {
        offenders.push(`${cell.id} (status: ${cell.status || 'unknown'} — not capped)`);
      } else if (trace.verify_passed === false) {
        offenders.push(`${cell.id} (capped with a FAILING recorded verify)`);
      } else if (trace.proof === 'unrecorded') {
        // worker-conformance D11 — the third way a test cell fails to
        // discharge this debt, and the one D1 creates. `verify_passed: false`
        // is a test cell that ran and lost; this is a test cell that asserted
        // a pass with nothing to show for it — no verify output and no
        // verification_evidence (D14). Decision 0004's rule outlives the door
        // that used to enforce it per-cell: an assertion is not evidence, so
        // an asserted pass cannot be the coverage this door is holding out for.
        offenders.push(`${cell.id} (capped with NO recorded proof — trace.proof: "unrecorded"; an asserted pass is not evidence)`);
      }
      continue;
    }
    // The trigger is a CAPPED behavior/api cell, and ONLY those two classes:
    //
    //  · CAPPED, not merely present — bootstrap. A feature whose behavior
    //    cells are all still open has authored no shipped behavior yet, so it
    //    owes no tests yet; blocking there would wall in every feature at the
    //    moment it starts. Debt begins when behavior is capped, which is
    //    exactly when P1 let that cell cap WITHOUT authoring tests.
    //  · behavior/api ONLY — a docs-only, refactor-only or formatting-only
    //    feature never authored behavior, and must never be asked for a test
    //    cell it has nothing to write. refactor/formatting already cap on
    //    `suite-green` (the EXISTING suite), and a docs cell resolves to no
    //    class at all through cellChangeClass, so all three fall through here
    //    and the branch below stays silent for them.
    //  · and only when the cell TOUCHED CODE (user law, 2026-07-27: "test chỉ
    //    dành cho code"). A behavior cell whose entire recorded file set is
    //    instruction/knowledge text — skills/, docs/, plans/, .bee/, or bare
    //    .md — changed what the agent reads, not what the machine runs; a test
    //    cell has nothing executable to cover there. An empty or missing file
    //    list stays CONSERVATIVE (counts as code) so an unrecorded diff can
    //    never launder real behavior past the debt.
    if ((changeClass === 'behavior' || changeClass === 'api') && cell.status === 'capped') {
      const recorded = [
        ...(Array.isArray(cell.files) ? cell.files : []),
        ...(Array.isArray(cell.trace && cell.trace.files_changed) ? cell.trace.files_changed : []),
      ];
      const nonCode = (f) =>
        typeof f === 'string' &&
        (f.startsWith('skills/') || f.startsWith('docs/') || f.startsWith('plans/') || f.startsWith('.bee/') || f.endsWith('.md'));
      const touchedCode = recorded.length === 0 || recorded.some((f) => !nonCode(f));
      if (touchedCode) cappedBehaviorBearing.push(`${cell.id} (${changeClass})`);
    }
  }
  // Order preserved from the original guard: the "no test cell at all" branch
  // is only reached when nothing else is outstanding.
  if (offenders.length === 0 && testCellCount === 0 && cappedBehaviorBearing.length > 0) {
    return { kind: 'missing', offenders, cappedBehaviorBearing };
  }
  if (offenders.length === 0) return null;
  return { kind: 'not-green', offenders, cappedBehaviorBearing };
}

// The shared FIX tails — the remedy never depends on which door refused, so
// the text does not either. Every door imports these rather than growing its
// own copy (the drift that produced this cell in the first place).
export const FEATURE_VERIFY_FIX_TAIL =
  'Those cells owe their proof at the feature boundary by one of two roads: capped through `cells cap --feature-verify-pending` (main-verifies D1), which relocates per-cell proof to ONE feature-level verify over the feature\'s whole diff; or capped with trace.proof "unrecorded" (worker-conformance D11/D12), meaning no verify output and no verification evidence was recorded at all. Either way the same single run discharges them, and this door is what keeps that honest (D3).\n' +
  'FIX: run the feature verify (the impacted suite over the feature\'s whole diff), capture its output to a file, record it — `bee state feature-verify record --command "<cmd>" --output-file <file> --result green` — then retry. A red result is recordable (it documents the failure) but never opens this door: open fix cells in this same feature (never un-cap, main-verifies D5), re-verify, and record the green.\n' +
  'This is a mechanical precondition, not a human gate: no gate_bypass level (including "total") and no headless run lifts it, and there is no waiver flag.';

export function testCellDebtFixTail(kind, feature) {
  if (kind === 'missing') {
    return (
      'Slice-tail test batching (spec #80/#85 P1/P4) let each of those cap on the EXISTING suite staying green, with no new test authored — that trade is only safe because a `change_class: "test"` cell at the slice tail owes the coverage instead. This feature never scheduled one, so that coverage is owed by nobody.\n' +
      "WHAT IS MISSING: one cell with `change_class: \"test\"`, in this same feature, authoring consolidated coverage (happy path, edge cases, error paths) over the slice's NET behavior — not per-cell internals.\n" +
      `FIX: \`bee cells add --id <id> --feature ${feature} --change-class test ...\`, then execute it, record a passing verify, and cap it — \`bee cells verify --id <id> ... --passed true\` then \`bee cells cap --id <id> ...\`. A feature that genuinely authored no behavior (docs-only, refactor-only, formatting-only) never reaches this refusal: only a CAPPED behavior/api cell creates the debt.\n` +
      'This is a mechanical precondition, not a human gate: no gate_bypass level (including "total") and no headless run lifts it, and there is no waiver flag.'
    );
  }
  return (
    "Slice-tail test batching (spec #80/#85 P4) deferred this slice's test AUTHORING to that cell; leaving now would ship behavior whose tests were never written, never passed, or never actually run (a cap marked trace.proof \"unrecorded\" asserted a pass with no output and no evidence — worker-conformance D11).\n" +
    'FIX: execute the test cell (happy path, edge cases, error paths over the slice\'s net behavior), record a passing verify, and cap it — `bee cells verify --id <id> ... --passed true` then `bee cells cap --id <id> ...`. If its suite exposed a regression in an already-capped cell, open fix cells in this same feature and cap the test cell green after; capped cells are never un-capped.\n' +
    'This is a mechanical precondition, not a human gate: no gate_bypass level (including "total") and no headless run lifts it, and there is no waiver flag.'
  );
}

// ─── guard-completion gc-1 — ONE QUESTION, ASKED BY EVERY DOOR ─────────────
//
// Round THREE on this surface, and the first two both looked correct:
//
//   round 1 — guarded the phase door, missed the feature-swap door;
//   round 2 — guarded the swap door for feature-verify debt and moved the
//             computations up here (the DEBT CORE above) so no two doors
//             could compute the same debt differently.
//
// A reviewer walked straight through round 2 anyway, because the knowledge
// that actually escaped was never the computation — it was the LIST. Each door
// still assembled its own sequence of questions by hand, and the swap door's
// sequence was {feature-verify, scribing}: it never asked about test-cell debt.
// So a feature holding a capped `change_class: "behavior"` cell and NO test
// cell at all was refused by the phase door (EXIT 1) and by start-feature
// (EXIT 1), while `state set --feature beta --owner swarming
// --waive-scribing-debt` walked out with it (EXIT 0). One shared answer is
// worth nothing while every caller still chooses which parts of it to hear.
//
// So the doors stop knowing what debt IS. They ask ONE question —
// guardFeatureDebt — and the answer knows the whole set:
//
//   · a new debt kind is ONE entry in FEATURE_DEBT_KINDS below. Every door
//     inherits it the instant it is added, with no door edited, and the
//     door × kind matrix in test_bee_cli.mjs (which iterates this same array
//     rather than hand-listing pairs) fails until that kind has a fixture, so
//     the coverage is inherited too;
//   · a new DOOR gets the whole set by construction — the only thing it can
//     supply is its own refusal head, never a subset of the questions;
//   · a door that tried to ask a subset would have to re-import the detectors
//     and rebuild the message composition by hand. That is precisely the shape
//     that produced three rounds of this bug, and test_bee_cli.mjs pins it
//     shut structurally (no per-kind detector call survives in bee.mjs).
//
// SCOPE — the UNWAIVABLE debt set: mechanical preconditions that no
// gate_bypass level (including "total") and no waiver flag lift. Scribing debt
// is deliberately NOT a member: it is waivable (--waive-scribing-debt), and it
// is computed in cells.mjs, which imports this file and therefore cannot be
// read from here. Every door runs this guard BEFORE its own scribing check for
// exactly that reason — a waiver for the waivable debt must never be the thing
// that carries an unwaivable one through (the reviewer's repro was that exact
// command).

/**
 * FEATURE_DEBT_KINDS — the whole unwaivable debt set, in refusal order.
 * Each kind owns four things and nothing else: how the debt is DETECTED, how
 * it is NAMED in a refusal, what it means to ABANDON it (doors that walk away
 * from the feature rather than merely moving its phase), and its FIX tail.
 * Nothing here knows which door is asking; no door knows what is in here.
 */
export const FEATURE_DEBT_KINDS = [
  {
    id: 'feature-verify',
    label: 'pending feature-level verify (main-verifies D1/D3)',
    detect: (root, record, feature) => featureVerifyDebt(root, record, feature),
    owed: (debt) => {
      // Both roads are named, and the unrecorded ones are named TWICE — the
      // reader needs to know which cells never recorded proof at all, because
      // their FIX reads differently from a deliberate relocation's.
      const unrecordedNote =
        debt.unrecorded && debt.unrecorded.length
          ? ` — of those, ${debt.unrecorded.join(', ')} recorded NO proof at all (trace.proof: "unrecorded")`
          : '';
      return `has ${debt.pending.length} capped cell(s) awaiting the feature-level verify (trace.feature_verify: "pending", or trace.proof: "unrecorded"): ${debt.pending.join(', ')}${unrecordedNote}, and ${debt.recordState}`;
    },
    abandonment: (feature, incoming) =>
      `Walking away to "${incoming}" leaves "${feature}"'s ONE feature-level verify unrun, and every reader of these markers keys on the record's feature: once the swap lands, nothing reads those pending caps again. The relocated proof would not be deferred, it would be destroyed.`,
    fixTail: () => FEATURE_VERIFY_FIX_TAIL,
  },
  {
    id: 'test-cell',
    label: 'consolidated slice-tail test coverage (spec #80/#85 P1/P4)',
    detect: (root, record, feature) => testCellDebt(root, feature),
    owed: (debt) =>
      debt.kind === 'missing'
        ? `has ${debt.cappedBehaviorBearing.length} capped behavior/api cell(s) and NO consolidated test cell at all: ${debt.cappedBehaviorBearing.join(', ')}`
        : `has ${debt.offenders.length} consolidated test cell(s) not green: ${debt.offenders.join(', ')}`,
    abandonment: (feature, incoming) =>
      `Walking away to "${incoming}" leaves that coverage owed by nobody: every reader of these cells keys on the record's feature, so once "${incoming}" is the active feature, "${feature}"'s behavior is never asked for its tests again.`,
    fixTail: (debt, feature) => testCellDebtFixTail(debt.kind, feature),
  },
];

/**
 * guardFeatureDebt(root, record, feature, {subject, abandonedFor}) — THE door
 * call. Every door in the codebase makes exactly this one call and supplies
 * only its own refusal head:
 *
 *   subject      the refusal up to (but not including) the debt clause, e.g.
 *                `set: refusing to leave phase "x" for "y" — feature "f"`.
 *                The kind appends `has …` and the FIX tail.
 *   abandonedFor the incoming feature when this door WALKS AWAY from `feature`
 *                (swap, start-feature) rather than moving its phase; adds each
 *                kind's abandonment paragraph. null for phase doors.
 *
 * Throws on the first standing debt, and throws (never passes) when the cell
 * store is unreadable — unknown debt is never zero debt. Returns nothing.
 */
export function guardFeatureDebt(root, record, feature, { subject, abandonedFor = null } = {}) {
  if (!feature) return; // no feature ⇒ nothing keyed to hold open (idle, or a record with none)
  for (const kind of FEATURE_DEBT_KINDS) {
    const debt = kind.detect(root, record, feature);
    if (!debt) continue;
    const abandonNote = abandonedFor ? `${kind.abandonment(feature, abandonedFor)}\n` : '';
    throw new Error(`${subject} ${kind.owed(debt)}.\n${abandonNote}${kind.fixTail(debt, feature)}`);
  }
}

/**
 * guardOutgoingFeatureDebt(root, record, outgoing, incoming) — startFeature's
 * door. Owns its refusal head; asks the whole debt set through the one call.
 */
function guardOutgoingFeatureDebt(root, record, outgoing, incoming) {
  guardFeatureDebt(root, record, outgoing, {
    subject: `startFeature: refused — starting "${incoming}" abandons feature "${outgoing}", which`,
    abandonedFor: incoming,
  });
}

function listAllCellsForStart(root) {
  const dir = path.join(root, '.bee', 'cells');
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return [];
  }
  const cells = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    const cell = readJson(path.join(dir, entry), null);
    if (!cell || typeof cell !== 'object' || Array.isArray(cell)) continue;
    cells.push(cell);
  }
  return cells;
}

function listActiveReservationsForStart(root) {
  // msn-16 condition C: reads THROUGH the shim (lease-store-backed), never
  // the legacy `.bee/reservations.json` projection — see the header comment
  // above this function for why. listReservations' own activeOnly filter
  // applies the exact same "unreleased AND not TTL-expired" semantics this
  // function used to implement by hand against the raw file.
  return listReservations(root, { activeOnly: true });
}

// ─── C1 (advisor consult slice 2, binding, the seed) — legacy-to-workflow
// materialization ───────────────────────────────────────────────────────────
// Workflow records (msn-5, workflow-store.mjs) become the new unit of
// coordination state (D1), but until msn-7 rebuilds .bee/state.json and
// .bee/lanes/*.json as READ-ONLY projections, the legacy stores stay the live
// writers. That leaves exactly one dangerous moment: the very FIRST workflow
// record ever created in a repo that already has a live legacy pipeline (a
// non-idle default record, or a non-terminal lane) — if nothing captures that
// pipeline into a workflow record now, a later reader that trusts workflow
// records as the source of truth (msn-7+) would see it as if it never
// existed. C1 closes that gap: before this repo's first-ever workflow record
// is created, materialize every live legacy record into one.
//
// Idempotent two ways: (1) gated on listWorkflows(root).workflows.length===0,
// so this only ever runs once per repo — once ANY workflow record exists, it
// is a no-op on every later call; (2) even within one run, each candidate is
// matched against "does a workflow already carry this feature at a
// non-closed status" before creating another, so the default record and a
// lane that happen to share a feature name can never double-materialize it.
function legacyGatesToWorkflowGates(approvedGates) {
  const gates = {};
  for (const name of GATE_NAMES) {
    gates[name] = { approved: Boolean(approvedGates && approvedGates[name] === true), approved_for_plan_rev: null };
  }
  return gates;
}

async function seedLegacyWorkflows(root) {
  // Workflow records are control-plane (msn-18a) — resolved through
  // controlRootFor so seeding from a worktree lands in the SAME workflow
  // store main reads, never a worktree-local one.
  const controlRoot = controlRootFor(root);
  const { workflows: existing } = listWorkflows(controlRoot);
  if (existing.length > 0) return; // never re-seed once ANY workflow record exists

  // workflow-lifecycle wl-1: the per-candidate "does this feature already have
  // a live record?" test is no longer kept here as a local array scan — it is
  // ensureWorkflowRecordForFeature's own idempotency, re-read from disk, so
  // seeding and starting share ONE definition of "already recorded". The
  // zero-records gate above still makes this whole function a once-per-repo
  // operation.
  //
  // (a) the legacy default record. "NON-idle" per C1: phase outside
  // idle/compounding-complete, OR a gate approved, OR a feature recorded —
  // read with the fail-open readState (never throw here; a corrupt default
  // record must never block an unrelated feature's start from seeding).
  const legacy = readState(root);
  const legacyLive =
    Boolean(legacy.feature) ||
    (legacy.phase && legacy.phase !== 'idle' && legacy.phase !== 'compounding-complete') ||
    Object.values(legacy.approved_gates || {}).some((v) => v === true);
  if (legacyLive && legacy.feature) {
    await ensureWorkflowRecordForFeature(root, {
      feature: legacy.feature,
      phase: legacy.phase,
      mode: legacy.mode,
      gates: legacyGatesToWorkflowGates(legacy.approved_gates),
      summary: legacy.summary,
      next_action: legacy.next_action,
      status: 'active',
    });
  }

  // (b) every non-terminal lane (same phase test as (a) minus the gates/
  // feature disjuncts — a lane file always carries its feature in the
  // filename, so "recorded but idle" never arises the way it can for the
  // singleton default record).
  for (const lane of listLanes(root)) {
    if (!lane || !lane.feature) continue;
    const laneLive = lane.phase && lane.phase !== 'idle' && lane.phase !== 'compounding-complete';
    if (!laneLive) continue;
    await ensureWorkflowRecordForFeature(root, {
      feature: lane.feature,
      phase: lane.phase,
      mode: lane.mode,
      gates: legacyGatesToWorkflowGates(lane.approved_gates),
      summary: lane.summary,
      next_action: lane.next_action,
      status: 'active',
    });
  }
}

// checkNoLiveWorkflowForFeature — the workflow-record sibling of the lane
// path's own-feature nonterminal-cell check: refuses if a NON-CLOSED workflow
// record already names this feature, naming the conflicting workflow in the
// message (must_have: "precondition failures name the conflicting
// workflow"). Read-only — runs before any write, so a refusal here leaves
// zero mutations, same discipline as every other precondition in this file.
function checkNoLiveWorkflowForFeature(root, feature) {
  // Workflow records are control-plane (msn-18a) — see seedLegacyWorkflows.
  const { workflows } = listWorkflows(controlRootFor(root));
  const conflict = workflows.find((wf) => wf.feature === feature && wf.status !== 'closed');
  if (conflict) {
    throw new Error(
      `startFeature: refused — a live workflow already exists for feature "${feature}" (workflow ${conflict.id}, phase "${conflict.phase}", status "${conflict.status}"). FIX: close or resolve that workflow before starting a new one for the same feature.`,
    );
  }
}

// checkNoSameFeatureClaimedCells — the workflow-precondition layer's
// "claimed-cell check scoped to the same feature's cells" (advisor consult
// slice 2). Reuses the caller's already-fetched cell list — no extra read.
// For the default path this is a genuinely NEW check (today's own "any cell
// anywhere claimed" check is feature-blind); for the lane path it is
// additive to, and normally shadowed by, lane check (a)'s broader
// open/claimed/blocked scan — kept here anyway so both paths share one
// workflow-precondition seam instead of diverging.
function checkNoSameFeatureClaimedCells(feature, cells) {
  const claimed = cells.filter((cell) => cell.feature === feature && cell.status === 'claimed');
  if (claimed.length > 0) {
    throw new Error(
      `startFeature: refused — feature "${feature}" already has claimed cell(s): ${claimed
        .map((c) => c.id)
        .join(', ')}. FIX: cap or drop them first (bee.mjs cells cap / bee.mjs cells drop).`,
    );
  }
}

// ─── workflow-lifecycle wl-1: the ONE record-creation seam + close-by-feature
// ────────────────────────────────────────────────────────────────────────────
// DEFECT this closes (5 live drift incidents, 2026-07-28). Sessions kept
// reverting mid-run to a previously CLOSED feature. The chain: a stopping
// subagent fires the state-sync hook -> rebuildStateProjection's
// idle-bootstrap branch picks the NEWEST `status: 'active'` workflow record
// (state-projection.mjs) -> and stale records stayed active forever, because
//   (a) some feature-start paths never created a workflow record at all
//       (inspection of .bee/runtime/workflows/ found records for only 4 of the
//       ~10 features that ran that day: main-verifies, foundation-fixes,
//       parallel-default, test-batching-finish and ship-visibility-config had
//       none), and
//   (b) fx-1's close reached only records that EXIST and only for the
//       OUTGOING feature named on the legacy default record — so a feature
//       whose record was never created could never be closed either (starting
//       status-diet left skill-token-diet active).
// The two exports below are the fix's public shape; ensureWorkflowRecordForFeature
// is the single creation seam every start path in this file routes through, so
// "which paths create a record" can never again drift path-by-path.
//
// AUDIT — every path in THIS file that starts, materializes, or adopts a
// feature, and what it now does about a workflow record:
//   1. startFeature, DEFAULT branch          -> ensureWorkflowRecordForFeature,
//      then closeWorkflowsForFeature({keepFeature}).
//   2. startFeature, LANE branch (startLane) -> ensureWorkflowRecordForFeature
//      (the same post-lock call — one seam, both branches); closes nothing, by
//      design (lanes are deliberately concurrent; startLane is byte-untouched).
//   3. startFeature, --isolate REDIRECT      -> returns before any write to
//      this root at all (a new checkout is created instead), so there is no
//      pipeline here to record; the redirected-to checkout starts normally.
//   4. seedLegacyWorkflows (C1 materialization) -> ensureWorkflowRecordForFeature
//      for the live legacy default record and every non-terminal lane, gates
//      carried over. Not a "start", but it is the other creator of records, so
//      it shares the seam rather than open-coding its own idempotency test.
//   5. adoptHandoff / adoptMailboxHandoff    -> NO record work, deliberately.
//      Neither writes `feature`, `phase`, or gates: they transfer a CELL claim
//      and clear a mailbox slot. adoptMailboxHandoff is keyed BY workflow id,
//      so a record provably exists; adoptHandoff is the C1 fallback that runs
//      only in a repo with zero workflow records, whose next real pipeline
//      write goes through 1/2/4 above.
//   6. writeState / writeLane                -> raw persistence primitives, not
//      start paths; every caller that means "start" goes through startFeature.
// The remaining creator OUTSIDE this file is the CLI's own `state set --feature`
// (bee.mjs) — the path that produced hole (a) in practice — which is why the two
// contracted exports above exist: it calls them rather than growing its own copy.

/**
 * listWorkflowRecords(root) — INTERFACE CONTRACT (workflow-lifecycle wl-1/wl-2,
 * frozen so the library and CLI cells could be implemented in parallel).
 * Returns a plain ARRAY of the full workflow records (not listWorkflows'
 * `{workflows, skipped}` envelope), resolved through controlRootFor so a caller
 * inside a worktree reads the SAME control-plane store main reads (msn-18a).
 * Fail-open exactly like listWorkflows: an unreadable record is skipped and
 * warned about, never thrown for the whole listing.
 */
export function listWorkflowRecords(root) {
  const { workflows } = listWorkflows(controlRootFor(root));
  return workflows;
}

/**
 * closeWorkflowsForFeature(root, { keepFeature }) — INTERFACE CONTRACT
 * (workflow-lifecycle wl-1/wl-2, frozen). Closes every LIVE workflow record
 * (`status !== 'closed'` — this file's own definition of live, the same one
 * checkNoLiveWorkflowForFeature uses) whose `feature` differs from
 * `keepFeature`, and returns the array of closed `{id, feature}`.
 *
 * Close BY FEATURE, never by record presence: hole (b) above existed because
 * the old close asked "does the outgoing feature have a record?" — a question
 * whose answer is "no" exactly when the drift is worst. This asks the only
 * question that stays true either way: "is any OTHER feature still live?".
 * `keepFeature: null` closes every live record (a full wind-down).
 *
 * Idempotent — a second call finds nothing left to close and returns []. Lock-
 * safe — each close is one updateWorkflow call taking that record's OWN
 * `workflow:<id>` lock, sequentially, never a shared lock and never nested
 * inside the 'state' lock (call it only from OUTSIDE that lock, exactly as
 * startFeature does below; C4's sessions/workflow isolation is unaffected —
 * this touches no session record).
 */
export async function closeWorkflowsForFeature(root, { keepFeature = null } = {}) {
  const controlRoot = controlRootFor(root);
  const keep = typeof keepFeature === 'string' && keepFeature.trim() ? keepFeature.trim() : null;
  const closed = [];
  for (const wf of listWorkflows(controlRoot).workflows) {
    if (!wf || wf.status === 'closed') continue;
    if (keep !== null && wf.feature === keep) continue;
    await updateWorkflow(controlRoot, wf.id, { status: 'closed' });
    closed.push({ id: wf.id, feature: wf.feature });
  }
  return closed;
}

/**
 * ensureWorkflowRecordForFeature(root, {...}) — the SINGLE workflow-record
 * creation seam for every path that starts or adopts a feature. Returns
 * `{ record, created }`.
 *
 * Idempotent by FEATURE: when a live (`status !== 'closed'`) record already
 * names this feature it is returned untouched with `created: false` — never a
 * second record for one feature, and never a silent overwrite of the live
 * one's phase/gates (a caller that means to MOVE a live record uses
 * updateWorkflow, which is what bee.mjs's own state verbs already do).
 * Otherwise one fresh record is created via createWorkflow under its own
 * `workflow:<id>` lock.
 *
 * Callers must be OUTSIDE the 'state' lock (createWorkflow takes
 * `workflow:<id>`; the two are never held together — see startFeature's header).
 */
export async function ensureWorkflowRecordForFeature(
  root,
  { feature, phase = 'idle', mode = null, summary = '', next_action = '', status = 'active', gates = undefined } = {},
) {
  if (typeof feature !== 'string' || !feature.trim()) {
    throw new Error('ensureWorkflowRecordForFeature: a non-empty feature slug is required.');
  }
  const featureTrimmed = feature.trim();
  const controlRoot = controlRootFor(root);
  const live = listWorkflows(controlRoot).workflows.find((wf) => wf.feature === featureTrimmed && wf.status !== 'closed');
  if (live) return { record: live, created: false };
  const record = await createWorkflow(controlRoot, {
    feature: featureTrimmed,
    phase: isKnownPhase(phase) ? phase : 'idle',
    mode: mode == null ? null : String(mode),
    status,
    summary: typeof summary === 'string' ? summary : '',
    next_action: typeof next_action === 'string' ? next_action : '',
    ...(gates === undefined ? {} : { gates }),
  });
  return { record, created: true };
}

// D6 — async: the whole precondition-read-through-write body below runs
// inside withStoreLock('state', ...) so a concurrent startFeature/set/gate/
// worker/scribing-run CLI invocation can no longer race this function's
// read-check-write into a lost update. Every caller now awaits it (bee.mjs's
// handleStateStartFeature; direct unit callers in test_lib.mjs) — this is
// the same async-ification msh-3 already did for reservations.mjs's
// reserve/release/sweepExpired. CLI verbs WAIT normally (no maxAttempts
// override): only the hook-driven heartbeat/lease-renewal touch path
// (claims.mjs/reservations.mjs) uses try-once mode.
//
// D1/C4 (multisession-native-6): three steps, in order, NONE nested inside
// another's lock. (1) seedLegacyWorkflows (C1) runs first, entirely before
// this call's own legacy write, so it only ever materializes pipelines that
// were ALREADY live — never the feature/lane this very call is about to
// write (which would otherwise end up with two workflow records for one
// start). (2) the legacy precondition-read-through-write body below (default
// record OR lane, chosen by `lane`) is UNCHANGED in its own lock ('state',
// exactly as before) except for two additions: F5's per-feature HANDOFF
// scoping on the default path, and the two workflow-precondition checks
// (checkNoLiveWorkflowForFeature/checkNoSameFeatureClaimedCells) shared with
// the lane path. (3) once that body resolves and the 'state' lock is
// released, startFeature creates THIS feature's fresh workflow record under
// its OWN lock (workflow:<id>, via createWorkflow -> withWorkflowLock) —
// never nested inside 'state', and never touching any session record, so
// 'sessions' and 'workflow:<id>' are structurally never held together (C4).
// Steps (1)-(3) are three separate transactions, so a crash between them can
// leave the legacy write landed without its workflow record (or vice versa)
// — accepted for this transitional slice; msn-7/10 make workflow records the
// sole source of truth and retire the legacy write entirely.
export async function startFeature(
  root,
  { feature, mode = null, phase = 'exploring', lane = false, sessionId = null, paths = [], isolate = false } = {},
) {
  if (typeof feature !== 'string' || !feature.trim()) {
    throw new Error('startFeature: a non-empty --feature slug is required.');
  }
  const featureTrimmed = feature.trim();
  const phaseValue = String(phase);
  if (!isKnownPhase(phaseValue)) {
    throw new Error(
      `startFeature: invalid phase "${phaseValue}" — not in the known-phase enum (isKnownPhase). FIX: use one of ${KNOWN_PHASES.join(', ')}.`,
    );
  }

  // D1/C1/C4 (multisession-native-6): seed BEFORE this call's own legacy
  // write happens (not after) — seeding must see the world as it was before
  // THIS start, never re-materialize the very feature/lane this same call is
  // about to write, which would otherwise hand it two workflow records for
  // one start. Seeding itself never touches 'state' or 'workflow:<id>' for
  // the feature being started here — only prior, unrelated live records.
  await seedLegacyWorkflows(root);

  // C3 (multisession-native-8): computed once, shared by both branches below
  // — the derived-workers precondition on EITHER path excludes exactly this
  // id from "other" live workers, so a solo caller's own heartbeat never
  // blocks its own start.
  const sessionIdTrimmed = typeof sessionId === 'string' && sessionId.trim() ? sessionId.trim() : null;

  // multisession-native-20 (D3): this file's own write-capable entry point —
  // resolved BEFORE the 'state' lock is ever acquired (same "never nested"
  // discipline seedLegacyWorkflows above already follows: applyWritePolicy
  // takes its own, separate locks — workspace:<id>/worktree-admin — that must
  // never be held simultaneously with 'state'). Scoped to the DEFAULT
  // (non-lane) path only: CONTEXT.md's "Scope boundaries" locks lanes-as-UX
  // unchanged, and lanes are bee's existing, already-coordinated concurrent-
  // same-checkout mechanism (many sessions on many features, deliberately
  // supported — see the lane precondition checks a few lines below, which
  // this cell leaves byte-untouched). The literal "another session is
  // active, wait" refusal D3 describes replacing is the DEFAULT path's own
  // `others.length > 0` activeWorkers check further down — this is that
  // replacement. A successful isolate-create returns here directly, before
  // this call's own legacy write, never touching root's pipeline at all —
  // the caller (bee.mjs's handleStateStartFeature) checks `.redirect` and
  // must not proceed with its own follow-up projection rebuild when true.
  if (!lane) {
    const policy = await applyWritePolicy(root, {
      sessionId: sessionIdTrimmed,
      paths: Array.isArray(paths) ? paths : [],
      isolate: isolate === true,
      feature: featureTrimmed,
      verbHint: `state start-feature --feature ${featureTrimmed}`,
    });
    if (policy.redirect) {
      return policy;
    }
  }

  const legacyRecord = await withStoreLock(root, 'state', () => {
    if (lane) {
      return startLane(root, {
        feature: requireLaneFeature(feature),
        mode,
        phase: phaseValue,
        sessionId: sessionIdTrimmed,
        paths,
      });
    }

    // Re-read immediately before any check (C1) — every read below happens
    // before the single write at the end, so a refusal leaves zero mutations.
    const state = readStateStrict(root);

    if (state.phase !== 'idle' && state.phase !== 'compounding-complete') {
      throw new Error(
        `startFeature: refused — current phase is "${state.phase}", not idle or the terminal alias "compounding-complete". A prior feature must finish or be explicitly wound down before a new feature starts. FIX: resume/close the current feature through its normal chain, or drop its remaining cells (bee.mjs cells drop), then retry.`,
      );
    }

    // F5 (advisor consult slice 2, binding): scoped per-feature, mirroring
    // the lane path's own handoff.feature check in startLane below — a
    // handoff naming a DIFFERENT feature no longer blocks this start.
    // Residual coupling: both paths still read the SAME single
    // .bee/HANDOFF.json (one mailbox for the whole repo) until slice 3's
    // per-workflow mailboxes (D5, msn-15) land — two DIFFERENT features can
    // still only have one paused handoff on file at a time; they just no
    // longer block each OTHER's starts the way any-handoff-blocks used to.
    const handoff = readHandoff(root);
    if (handoff && handoff.feature === featureTrimmed) {
      throw new Error(
        `startFeature: refused — .bee/HANDOFF.json names feature "${featureTrimmed}"; its paused work must resume or close before this feature restarts. FIX: resume the handoff (or explicitly delete HANDOFF.json once its work is truly abandoned), then retry.`,
      );
    }

    // multisession-native-20 (D3) RETIRES this precondition's old shape.
    // Pre-msn-20 (D6, multisession-native-8) this refused whenever ANY other
    // session merely had a live heartbeat — regardless of whether it had
    // ever actually written anything — with a bare "wait" message. D3
    // explicitly describes replacing exactly that "another session is
    // active, wait" answer with the isolate-or-consent flow: applyWritePolicy
    // above (this file's own write-capable entry point, run before the
    // 'state' lock) already covers the real case worth blocking — another
    // session that has ACTUALLY become this workspace's live write owner —
    // with a strictly better UX (refuse naming the exact --isolate
    // one-liner, or auto-isolate with consent) instead of a bare wait. A
    // session that merely has an open heartbeat but has never itself
    // attempted a write here is no longer, on its own, a reason to refuse.

    const activeReservations = listActiveReservationsForStart(root);
    if (activeReservations.length > 0) {
      throw new Error(
        `startFeature: refused — ${activeReservations.length} active reservation(s) remain (${activeReservations
          .map((r) => `${r.agent}:${r.path}`)
          .join(', ')}). FIX: release them first (bee.mjs reservations release).`,
      );
    }

    const cells = listAllCellsForStart(root);
    const claimed = cells.filter((cell) => cell.status === 'claimed');
    if (claimed.length > 0) {
      throw new Error(
        `startFeature: refused — claimed cell(s) remain: ${claimed.map((c) => c.id).join(', ')}. FIX: cap or drop them first (bee.mjs cells cap / bee.mjs cells drop).`,
      );
    }

    const priorFeature = state.feature;
    if (priorFeature) {
      const nonterminal = cells.filter(
        (cell) =>
          cell.feature === priorFeature &&
          (cell.status === 'open' || cell.status === 'claimed' || cell.status === 'blocked'),
      );
      if (nonterminal.length > 0) {
        throw new Error(
          `startFeature: refused — prior feature "${priorFeature}" has nonterminal cell(s): ${nonterminal
            .map((c) => `${c.id}(${c.status})`)
            .join(', ')}. An abandoned cell must first be resolved through the existing drop verb (bee.mjs cells drop --id ID --reason R) — startFeature never auto-clears cells as cleanup. FIX: cap or drop each listed cell, then retry.`,
        );
      }
    }

    // review-p1-fixes p2-1 — the THIRD door, and the one the delta re-review's
    // repro actually escaped through. startFeature already refused over the
    // prior feature's NONTERMINAL cells; it said nothing about the debt its
    // TERMINAL ones still carry, so a feature could close with pending
    // feature-verify caps (or an unfinished consolidated test cell) and the
    // very next `state start-feature` would overwrite `feature`, close the
    // outgoing workflow records (closeWorkflowsForFeature, below), and be done.
    // Every reader of trace.feature_verify keys on the record's feature: once
    // the name is replaced, nothing reads those pending caps again — the
    // relocated proof is not deferred, it is destroyed. CLOSING MUST NOT BE
    // THE THING THAT ERASES THE OBLIGATION.
    //
    // Restarting the SAME feature is not an abandonment (the readers still key
    // on that name), so only a genuine change of feature asks. Placed inside
    // the precondition block, before the single write, so a refusal still
    // makes ZERO mutations — and, like its bee.mjs siblings, it is a mechanical
    // precondition: no gate_bypass level (including "total") and no waiver
    // flag lifts it.
    if (priorFeature && priorFeature !== featureTrimmed) {
      guardOutgoingFeatureDebt(root, state, priorFeature, featureTrimmed);
    }

    // New workflow-precondition layer (advisor consult slice 2): re-scoped
    // to LIVE workflow records / this feature's own claimed cells, additive
    // to every legacy check above.
    checkNoLiveWorkflowForFeature(root, featureTrimmed);
    checkNoSameFeatureClaimedCells(featureTrimmed, cells);

    // All preconditions hold — ONE atomic write: feature/mode/phase, reset all
    // four gates, refreshed summary/next_action. A new feature never inherits
    // approvals from whatever came before it.
    state.feature = featureTrimmed;
    state.mode = mode == null ? null : String(mode);
    state.phase = phaseValue;
    state.approved_gates = { context: false, shape: false, execution: false, review: false };
    state.summary = `Feature "${state.feature}" started at phase "${phaseValue}".`;
    // codex-loop (advisor #54): start-feature IS the routing decision — telling the
    // agent to invoke hive again right after it is what re-opened the idle->hive loop
    // one step later. Hand off forward to the phase that was just entered.
    state.next_action = `Continue bee-${phaseValue} for "${state.feature}".`;
    writeState(root, state);
    return state;
  });

  // D1/C4 (multisession-native-6): workflow record creation happens OUTSIDE
  // the 'state' lock — see the block comment above this function (C1 already
  // seeded above, before the legacy write).
  //
  // multisession-native-7: summary/next_action are carried over from the
  // legacy write's own computed text (`legacyRecord`, either the default
  // record's or startLane's) rather than left at createWorkflow's empty-
  // string defaults — a lane projection rebuilt from this record (see
  // handleStateStartFeature in bee.mjs) must show the same descriptive text
  // startLane already wrote, not a blank one.
  // Workflow records are control-plane (msn-18a) — see seedLegacyWorkflows.
  //
  // workflow-lifecycle wl-1: routed through ensureWorkflowRecordForFeature,
  // the ONE creation seam, so BOTH branches above (default and lane) and every
  // future start path create their record through identical logic instead of
  // each open-coding a createWorkflow call. Idempotent by feature, so a
  // half-completed earlier start that already left a live record for this
  // feature is adopted rather than duplicated (checkNoLiveWorkflowForFeature
  // normally refuses before ever reaching here; this is the belt to its braces).
  await ensureWorkflowRecordForFeature(root, {
    feature: featureTrimmed,
    phase: phaseValue,
    mode,
    status: 'active',
    summary: legacyRecord.summary,
    next_action: legacyRecord.next_action,
  });

  // Close the OUTGOING work — D1 (foundation-fixes), widened by
  // workflow-lifecycle wl-1 from "the prior feature's records" to "every OTHER
  // live record". The old shape asked whether the legacy default record still
  // NAMED the outgoing feature and whether that feature happened to have a
  // record; when either answer was no (the two holes documented on
  // closeWorkflowsForFeature above), a stale `status: 'active'` record survived
  // and the projection's idle-bootstrap picker later resurrected it mid-session.
  // Closing by feature — keep exactly the feature being started, close the rest
  // — is true regardless of what the outgoing side left behind.
  //
  // Still scoped to the DEFAULT (non-lane) path: a lane start closes nothing,
  // because lanes are bee's deliberately-concurrent mechanism and startLane's
  // own logic stays byte-untouched. The residual, accepted by design: a DEFAULT
  // start closes a live LANE feature's record too — the default record is the
  // whole-repo pipeline, and the picker it feeds must resolve to exactly one
  // active workflow.
  //
  // Runs AFTER the new record exists (never before) so a crash between the two
  // leaves the new record live plus a to-be-closed stale one — never a moment
  // with zero active workflows for a feature genuinely still starting.
  if (!lane) {
    await closeWorkflowsForFeature(root, { keepFeature: featureTrimmed });
  }

  return legacyRecord;
}

// ─── write-policy resolution (multisession-native-20, D3, advisor-digest-
// slice4 condition 6) ───────────────────────────────────────────────────────
// Three modes, resolved fresh on every call at each write-capable entry
// point (startFeature above, and bee.mjs's cells claim/claim-next):
//
//   - 'observe'         (config.guards.write_policy === 'observe'): this
//     workspace's ownership machinery is never consulted at all — the
//     unlimited-concurrent, read/analyze/review mode. An explicit opt-OUT of
//     this whole feature, byte-identical to pre-msn-20 behavior.
//   - 'shared-disjoint' (config.guards.write_policy === 'shared-disjoint'):
//     opt-in. Ownership is never claimed; instead an EXACT-path lease (the
//     existing reservations.mjs/lease-store.mjs machinery, msn-11/16 — never
//     a broad/glob reservation) must already be held by the acting session
//     over every declared path, or the call refuses LEASE_REQUIRED naming
//     what is missing. Nothing here ACQUIRES the lease on the caller's
//     behalf — "mandatory before write" is a precondition, not an
//     auto-grant (bee.mjs reservations reserve remains the one CLI writer).
//   - default ('isolated'): the workspace registry's single write owner
//     (workspace-store.mjs, msn-19) decides. A solo caller (nobody else
//     live) always becomes owner — byte-identical to today, the must_have
//     prohibition this cell is bound by. A caller that finds a DIFFERENT,
//     LIVE session already owning the workspace either (a) refuses, naming
//     the exact one-liner to retry with (surfaced in full once per session,
//     a shorter repeat after — never spammed), or (b) with explicit
//     --isolate / config.guards.auto_isolate=true, creates a fresh feature
//     worktree (worktree-store.mjs createFeatureWorktree, which already
//     registers its own workspace record per msn-19) and answers with the
//     new checkout's path plus a loud one-line disk-cost disclosure —
//     condition 6's (a)/(b). The ORIGINAL write-capable call is never
//     silently redirected into the new checkout by this function itself
//     (a different process, possibly a different physical machine, cannot
//     be redirected mid-call) — callers check `.redirect` and stop short of
//     performing their own write when it is true, surfacing `.text` instead.
//
// Sessionless callers (no sessionId — the legacy, pre-multisession-native
// calling convention some tests and sessionless CLI invocations still use)
// skip ownership entirely under the default mode: there is no session
// identity to own a workspace with, so this returns `workspace: 'unbound'`
// and never touches workspace-store.mjs — zero new writes, zero new
// behavior for a caller this cell was never asked to change (prohibition:
// "no silent default change for solo single-session repos").
export class WritePolicyRefusalError extends Error {
  constructor(code, message, details) {
    super(message);
    this.name = 'WritePolicyRefusalError';
    this.type = 'refused';
    this.code = code;
    if (details && typeof details === 'object' && details.holder !== undefined) this.holder = details.holder;
  }
}

function resolveWritePolicyMode(config) {
  const configured = config && config.guards && typeof config.guards.write_policy === 'string' ? config.guards.write_policy.trim() : '';
  if (configured === 'observe') return 'observe';
  if (configured === 'shared-disjoint') return 'shared-disjoint';
  return 'isolated';
}

function writePolicyIsolateOneLiner(verbHint) {
  const verb = typeof verbHint === 'string' && verbHint.trim() ? verbHint.trim() : '<verb>';
  return (
    `bee.mjs ${verb} --isolate (creates a fresh worktree for this session), or set ` +
    'guards.auto_isolate to true in .bee/config.json to always auto-isolate on contention.'
  );
}

// "Surfaced once per session" (condition 6a) — a small per-session marker
// under the control plane, checked/stamped ONLY on the refusal path (never
// on a successful isolate-create, never on ordinary ownership). Corrupt or
// unreadable is treated as "not yet shown" (fail-open toward the FULLER
// message — the safer direction when the marker itself cannot be trusted).
function isolateNoticeMarkerPath(controlRoot, sessionId) {
  const safeId = String(sessionId).replace(/[\\/]/g, '_');
  return path.join(controlRoot, '.bee', 'runtime', 'notices', 'isolate', `${safeId}.json`);
}

function hasIsolateNoticeShown(controlRoot, sessionId) {
  return fs.existsSync(isolateNoticeMarkerPath(controlRoot, sessionId));
}

function markIsolateNoticeShown(controlRoot, sessionId) {
  try {
    writeJsonAtomic(isolateNoticeMarkerPath(controlRoot, sessionId), { session_id: sessionId, shown_at: new Date().toISOString() });
  } catch {
    // best-effort — a failed stamp only means the next refusal shows the
    // fuller message again, never a functional failure of the refusal itself.
  }
}

/**
 * applyWritePolicy(root, { sessionId, paths, isolate, feature, verbHint, now,
 * enforceIsolation }) — resolve and, where the mode requires it, ENFORCE this
 * cell's write policy for one write-capable call. Returns `{ ok: true, mode,
 * ... }` when the caller may proceed with its own write; throws
 * WritePolicyRefusalError (typed, never a silent deny) when it may not. A
 * successful isolate-create returns `{ ok: true, mode: 'isolated', workspace:
 * 'isolated-created', redirect: true, worktreeRoot, workspaceId, text,
 * costDisclosure }` — the caller's own write must NOT proceed in `root` when
 * `redirect` is true; surface `.text` instead (must_have: "with
 * --isolate/config: worktree created, workspace registered, caller told the
 * path").
 *
 * `enforceIsolation` (default true): whether the DEFAULT ('isolated') mode's
 * single-write-owner check actually blocks/redirects a second live session,
 * as opposed to treating 'isolated' as a no-op passthrough (byte-identical
 * to `workspace: 'unbound'`) — 'observe' and 'shared-disjoint' are UNCHANGED
 * either way, since they never depend on this flag. `false` is bee.mjs's own
 * choice for `cells claim`/`claim-next` (never state start-feature's default
 * path, where this cell's own single-write-owner truths are proven): those
 * two verbs ARE the mechanism bee's existing, already-coordinated concurrent
 * swarm model uses (multiple LIVE sessions legitimately claiming DIFFERENT
 * cells of the SAME feature in the SAME checkout — bee-swarming, reservations
 * D3, claims.mjs — the exact pattern CONTEXT.md's "Scope boundaries" locks as
 * unchanged "for the user", same reasoning this file's own lane path is
 * exempted by). Blocking a swarm worker's claim on workspace ownership would
 * not replace an unwanted "another session is active, wait" — it would BE
 * one, reintroduced under a new name, against the one entry point bee's own
 * README calls out as normal multi-session operation.
 */
export async function applyWritePolicy(
  root,
  { sessionId = null, paths = [], isolate = false, feature = null, verbHint = null, now = Date.now(), enforceIsolation = true } = {},
) {
  const config = readConfig(root);
  const mode = resolveWritePolicyMode(config);

  if (mode === 'observe') {
    return { ok: true, mode: 'observe' };
  }

  const controlRoot = controlRootFor(root);

  if (mode === 'shared-disjoint') {
    const declared = Array.isArray(paths) ? paths.filter((p) => typeof p === 'string' && p.trim()) : [];
    if (declared.length === 0) {
      return { ok: true, mode: 'shared-disjoint', leased: [] };
    }
    const active = listReservations(controlRoot, { activeOnly: true, now });
    const missing = declared.filter(
      (p) =>
        !active.some(
          (r) => sessionId && r.session === sessionId && !String(r.path).endsWith('*') && pathsOverlap(r.path, p),
        ),
    );
    if (missing.length > 0) {
      throw new WritePolicyRefusalError(
        'LEASE_REQUIRED',
        `bee write-policy (shared-disjoint): no exact-path lease held for: ${missing.join(', ')}. ` +
          'A broad/glob reservation never satisfies shared-disjoint — an exact-path lease is mandatory before write. ' +
          `FIX: bee.mjs reservations reserve --agent <worker> --cell <id> --path <path>${
            sessionId ? ` --session-id ${sessionId}` : ''
          } for each path, then retry.`,
      );
    }
    return { ok: true, mode: 'shared-disjoint', leased: declared };
  }

  // isolated (default). No session identity => nothing to own a workspace
  // with — the legacy sessionless calling convention proceeds untouched.
  // enforceIsolation === false (cells claim/claim-next): same no-op — see
  // this function's own header for why those two verbs opt out.
  if (!sessionId || !enforceIsolation) {
    return { ok: true, mode: 'isolated', workspace: 'unbound' };
  }

  const ctx = resolveContext(root);
  const workspaceId = (ctx && ctx.workspaceId) || 'main';
  const workspaceRoot = (ctx && ctx.workspaceRoot) || root;
  await registerWorkspace(controlRoot, {
    id: workspaceId,
    type: ctx && ctx.worktreeId ? 'worktree' : 'main',
    root: workspaceRoot,
  });

  // Production-shaped isOwnerLive (test_workspace_store.mjs precedent,
  // msn-19): built from claims.mjs's own readSession + heartbeatStale rather
  // than imported into workspace-store.mjs itself (structural isolation,
  // that module's own header).
  const isOwnerLive = (ownerSessionId, nowMs) => {
    const session = readSession(controlRoot, ownerSessionId);
    return session ? !heartbeatStale(session, nowMs) : false;
  };

  const attach = await attachWorkspace(controlRoot, workspaceId, sessionId, { now, isOwnerLive });
  if (attach.role === 'owner') {
    return { ok: true, mode: 'isolated', workspace: 'owner', reclaimed: Boolean(attach.reclaimed) };
  }

  // blocked: a DIFFERENT, live session already owns this workspace's writes.
  const autoIsolate = isolate === true || (config.guards && config.guards.auto_isolate === true);
  if (!autoIsolate) {
    const alreadyShown = hasIsolateNoticeShown(controlRoot, sessionId);
    if (!alreadyShown) markIsolateNoticeShown(controlRoot, sessionId);
    const oneLiner = writePolicyIsolateOneLiner(verbHint);
    throw new WritePolicyRefusalError(
      'WORKSPACE_ISOLATION_REQUIRED',
      alreadyShown
        ? `bee write-policy: workspace "${workspaceId}" is still write-owned by session "${attach.write_owner_session}". FIX: ${oneLiner}`
        : `bee write-policy: a second write-capable session defaults to isolation, not a wait — workspace "${workspaceId}" ` +
          `already has a live write owner (session "${attach.write_owner_session}"). Bee never writes into the same ` +
          `checkout as a live owner. FIX: ${oneLiner}`,
      { holder: attach.write_owner_session },
    );
  }

  // Consented (--isolate) or configured (guards.auto_isolate) — create a
  // fresh feature worktree. createFeatureWorktree already registers its own
  // workspace record (msn-19, worktree-store.mjs's own registerWorkspace
  // call) — this call attaches the ACTING session as that new workspace's
  // first (and, since it was just created, uncontested) write owner.
  const isolateFeature = (typeof feature === 'string' && feature.trim()) || `session-${sessionId}`;
  const created = await createFeatureWorktree(controlRoot, { feature: isolateFeature });
  await attachWorkspace(controlRoot, created.id, sessionId, { now });
  const costDisclosure = `[bee cost] Isolated worktree created — a FULL working-tree copy at ${created.worktreeRoot} (disk cost scales with repo size).`;
  return {
    ok: true,
    mode: 'isolated',
    workspace: 'isolated-created',
    redirect: true,
    worktreeRoot: created.worktreeRoot,
    workspaceId: created.id,
    branch: created.branch,
    costDisclosure,
    text: `${costDisclosure}\nOpen your next session with cwd=${created.worktreeRoot} to continue "${isolateFeature}" there — this checkout stays untouched.`,
  };
}

// Active claim holds by ANOTHER session whose claimed cell's files overlap the
// declared paths (startFeature lane precondition (d)). Claims name a cell, not
// paths, so the held paths are DERIVED from the claimed cell's files list —
// same derivation discipline as the handoff/worker attribution above.
function listClaimHoldsForStart(root, sessionId, cellById, declared) {
  // Claims are control-plane (msn-18a) — resolved through controlRootFor so
  // a start from a worktree checks holds against the SAME claim store main
  // reads, never a worktree-local one.
  const controlRoot = controlRootFor(root);
  let entries;
  try {
    entries = fs.readdirSync(claimsDir(controlRoot));
  } catch {
    return [];
  }
  const nowMs = Date.now();
  const holds = [];
  for (const entry of entries) {
    if (!entry.endsWith('.json')) continue;
    let claim;
    try {
      claim = readClaim(controlRoot, entry.slice(0, -'.json'.length));
    } catch {
      continue; // a filename that is not a plain cell id is no claim of ours
    }
    if (!claim || !isClaimActive(claim, nowMs)) continue;
    if (sessionId && claim.session === sessionId) continue; // own holds never block
    const cell = cellById.get(claim.cell);
    const files = cell && Array.isArray(cell.files) ? cell.files : [];
    for (const file of files) {
      if (declared.some((declaredPath) => pathsOverlap(file, declaredPath))) {
        holds.push({ session: claim.session, cell: claim.cell, path: file });
        break;
      }
    }
  }
  return holds;
}

// startLane — the lane-mode body of startFeature (never called directly; the
// exported startFeature validates feature/phase first). Fails closed exactly
// like the default path: every precondition is read BEFORE the single write,
// so a refusal makes ZERO mutations — to this lane, to the default record, and
// to every other lane.
function startLane(root, { feature, mode, phase, sessionId, paths }) {
  // A corrupt existing lane record refuses loudly with the file untouched.
  const existing = readLaneStrict(root, feature);
  if (existing && existing.phase !== 'idle' && existing.phase !== 'compounding-complete') {
    throw new Error(
      `startFeature: refused — lane "${feature}" is mid-flight at phase "${existing.phase}", not idle or the terminal alias "compounding-complete". FIX: finish or explicitly wind down that lane first, then retry.`,
    );
  }

  const cells = listAllCellsForStart(root);

  // (a) nonterminal cells of THIS lane's feature block; other features' never do.
  const nonterminal = cells.filter(
    (cell) =>
      cell.feature === feature &&
      (cell.status === 'open' || cell.status === 'claimed' || cell.status === 'blocked'),
  );
  if (nonterminal.length > 0) {
    throw new Error(
      `startFeature: refused — feature "${feature}" already has nonterminal cell(s): ${nonterminal
        .map((c) => `${c.id}(${c.status})`)
        .join(', ')}. An abandoned cell must first be resolved through the existing drop verb (bee.mjs cells drop --id ID --reason R). FIX: cap or drop each listed cell, then retry.`,
    );
  }

  // (b) the global handoff blocks a LANE start only when it names this feature.
  const handoff = readHandoff(root);
  if (handoff && handoff.feature === feature) {
    throw new Error(
      `startFeature: refused — .bee/HANDOFF.json names feature "${feature}"; its paused work must resume or close before this lane restarts. FIX: resume the handoff (or explicitly delete HANDOFF.json once its work is truly abandoned), then retry.`,
    );
  }

  // (c) D6 (multisession-native-8): an OTHER live worker (derived from live
  // heartbeat sessions x claims, claims.mjs activeWorkers — never
  // state.workers) blocks only when its currently claimed cell derives to
  // this feature. excludeSessionId (C3) means the calling session's own
  // heartbeat is never "another" worker.
  const cellById = new Map(cells.map((cell) => [cell.id, cell]));
  const laneWorkers = activeWorkers(root, { excludeSessionId: sessionId }).filter((worker) => {
    const cell = worker.cell ? cellById.get(worker.cell) : null;
    return Boolean(cell && cell.feature === feature);
  });
  if (laneWorkers.length > 0) {
    throw new Error(
      `startFeature: refused — active worker session(s) on feature "${feature}": ${laneWorkers
        .map((w) => `${w.session_id}(${w.cell})`)
        .join(', ')}. FIX: wait for the session's heartbeat to go stale, or have it cap/drop its claimed cell, then retry.`,
    );
  }

  // (d) declared intended paths vs ANOTHER session's active holds.
  const declared = (Array.isArray(paths) ? paths : [paths])
    .filter((p) => typeof p === 'string' && p.trim())
    .map((p) => p.trim());
  if (declared.length > 0) {
    const reservationHolds = listActiveReservationsForStart(root).filter((reservation) =>
      declared.some((declaredPath) => pathsOverlap(reservation.path, declaredPath)),
    );
    if (reservationHolds.length > 0) {
      throw new Error(
        `startFeature: refused — declared path(s) overlap active reservation hold(s): ${reservationHolds
          .map((r) => `${r.agent}:${r.path}`)
          .join(', ')}. FIX: wait for release/expiry (bee.mjs reservations release), or start the lane over non-overlapping paths.`,
      );
    }
    const claimHolds = listClaimHoldsForStart(root, sessionId, cellById, declared);
    if (claimHolds.length > 0) {
      throw new Error(
        `startFeature: refused — declared path(s) overlap file(s) of cell(s) claimed by another session: ${claimHolds
          .map((h) => `${h.session}:${h.cell}(${h.path})`)
          .join(', ')}. FIX: wait for the claim to release or expire, or start the lane over non-overlapping paths.`,
      );
    }
  }

  // (e) the shared workflow-precondition layer (multisession-native-6).
  checkNoLiveWorkflowForFeature(root, feature);
  checkNoSameFeatureClaimedCells(feature, cells);

  // All preconditions hold — ONE atomic write to this lane's record: feature/
  // mode/phase, ALL FOUR gates reset (spec R1 applied per lane), refreshed
  // summary/next_action. created_at survives a restart; the default record and
  // every other lane stay byte-identical.
  const record = {
    schema_version: '1.0',
    feature,
    mode: mode == null ? null : String(mode),
    phase,
    approved_gates: { context: false, shape: false, execution: false, review: false },
    summary: `Feature "${feature}" started at phase "${phase}" (lane).`,
    next_action: `Continue bee-${phase} for "${feature}" (lane).`,
    created_at:
      existing && typeof existing.created_at === 'string' && existing.created_at
        ? existing.created_at
        : new Date().toISOString(),
  };
  writeLane(root, record);
  return record;
}
