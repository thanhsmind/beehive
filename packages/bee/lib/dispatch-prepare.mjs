// dispatch-prepare.mjs — `bee dispatch prepare`, one source of truth for
// every bee-owned dispatch payload (g22-1, GH #22 P0-3).
//
// Builds the exact envelope a caller hands to the Agent tool / spawn_agent
// tool / an external cli executor, PLUS a small "economics" record (which
// tier was requested, which channel/enforcement mechanism carries it, and
// whether the effective model is verifiably pinned) — so a worker dispatch
// never has to hand-assemble the marker/model-param/subagent_type shape
// dispatch-guard.mjs (the enforcement side) is going to judge. Two sides,
// one vocabulary: this module imports PINNED_AGENT_TYPE from
// lib/dispatch-guard.mjs rather than re-deriving its own copy, and every
// [bee-tier: <t>] marker this module writes uses the same anchored-at-start
// convention dispatch-guard.mjs's ANCHORED_TIER_MARKER_RE checks.
//
// PURPOSE MAP (advisor A1, binding):
//   kind cell               -> resolveTier(root, 'generation', runtime, {for:'cell'})
//   kind gather              -> resolveTier(root, 'generation', runtime, {for:'gather'})
//   kind reviewer            -> resolveTier(root, 'review',     runtime, {for:'gather'})
//   kind advisor             -> resolveAdvisor(root, runtime) — NEVER a bare
//                                resolveTier(root, 'advisor', ...) call, which
//                                would silently coerce to 'generation'
//                                (state.mjs CONFIGURABLE_SLOTS comment, :1247).
//
// A cli-shaped resolution for kind 'cell' is a typed refusal
// ({type:'refused', reason:'cli_tier_gather_only', ...}, state.mjs resolveTier)
// — prepare returns that refusal VERBATIM and never builds a payload around
// it (advisor A1: "prepare NEVER routes around a refusal"). A cli-shaped
// resolution for gather/reviewer/advisor is a legitimate external-executor
// dispatch (External Executors, bee-swarming/references/swarming-reference.md)
// and gets its own Bash-shaped payload, below.

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { resolveTier, resolveAdvisor } from './state.mjs';
import { readCell } from './cells.mjs';
import {
  PINNED_AGENT_TYPE,
  deriveEconomics,
  NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
} from './dispatch-guard.mjs';

export const DISPATCH_RUNTIMES = ['codex', 'claude'];
export const DISPATCH_KINDS = ['cell', 'gather', 'reviewer', 'advisor'];

// The tier/slot name embedded in the [bee-tier: <t>] marker and recorded as
// economics.logical_tier. cell/gather both resolve the 'generation' slot;
// reviewer resolves 'review'. advisor has no resolveTier slot at all (it is
// deliberately excluded from CONFIGURABLE_SLOTS) — 'advisor' is a label, not
// a token the CLAUDE branch's ANCHORED_TIER_MARKER_RE recognizes (R1: that
// regex stays byte-unchanged), so a claude advisor-kind payload still never
// passes evaluateClaudeDispatch's marker branches. The CODEX branch's own
// ANCHORED_CODEX_TIER_MARKER_RE (native-transport R1) does recognize
// `advisor` — a confirmed-native codex advisor payload is expected to, and
// must, pass evaluateDispatch's codex branch (dispatch-prepare's own golden
// row, native-transport cnt-3).
function slotForKind(kind) {
  if (kind === 'cell' || kind === 'gather') return 'generation';
  if (kind === 'reviewer') return 'review';
  return 'advisor';
}

function purposeForKind(kind) {
  return kind === 'cell' ? { for: 'cell' } : { for: 'gather' };
}

// hardening-7 — claim-ownership guard for `dispatch prepare --kind cell`.
// Mirrors msh-4's audited-door pattern (cells.mjs checkClaimOwnership /
// guardClaimOwnership) but on a DIFFERENT axis: msh-4 compares the live
// claim file's `session` against the caller's resolved session; this checks
// the CELL RECORD's own `status`/`trace.worker` against the caller-supplied
// `--worker` — dispatch prepare has no session concept and never touches the
// claims.mjs store, so it reads exactly the fields readCell already
// returned, nothing more. Never throws (a foreign/unclaimed cell is a
// legitimate refusal a caller can rescue with --force-ownership, same
// "throws only on a malformed CALL" discipline prepareDispatch's own
// docstring states for every other branch).
function checkCellClaimOwnership(cell, worker) {
  if (cell.status !== 'claimed') {
    return {
      ok: false,
      code: 'not_claimed',
      status: cell.status,
      owner: null,
      reason: `cell "${cell.id}" is "${cell.status}", not "claimed" — dispatch prepare requires a claimed cell (run bee.mjs cells claim or cells claim-next first). Pass --force-ownership to override (audited).`,
    };
  }
  const owner = cell.trace && typeof cell.trace.worker === 'string' ? cell.trace.worker : null;
  if (owner !== worker) {
    return {
      ok: false,
      code: 'not_owner',
      status: cell.status,
      owner,
      reason: `cell "${cell.id}" is claimed by worker "${owner || '(unknown)'}" — "${worker}" does not own this claim. Pass --force-ownership to override (audited).`,
    };
  }
  return { ok: true, code: null, status: cell.status, owner };
}

// Template-consistent minimal prompt bodies (advisor spec: "for cell, render
// from the Worker Prompt Template shape ... for gather/reviewer/advisor,
// template-consistent minimal shapes"). Cell context comes from the loaded
// cell; gather/reviewer/advisor get a goal + paths + digest contract shape —
// the caller fills in the exact paths/question before dispatch.
//
// hardening-1-7-10 (D7): the reservation identity rendered into the prompt is
// the CALLER-supplied, validated `worker` name (the same name
// checkCellClaimOwnership above just checked against the cell's own
// trace.worker) — never the synthetic `prepare-<cell.id>` nickname this used
// to render. That placeholder never matched any reservation a real worker
// would take out (reservations are keyed by agent name, not by cell id), so
// a worker following this prompt verbatim would reserve files under an
// identity nobody else could recognize as theirs. `worker` is required
// whenever `kind === 'cell'` (prepareDispatch already throws before this is
// called if it is missing), so this is always a real, trimmed name here.
function cellPromptBody(cell, worker) {
  return [
    `Nickname (reservation identity): ${worker}`,
    `Assigned cell id: ${cell.id}`,
    `Feature: ${cell.feature}`,
    '',
    `Title: ${cell.title || '(untitled)'}`,
    `Action: ${cell.action || '(no action recorded)'}`,
    '',
    'Inputs — read these; nothing else will be provided:',
    `- docs/history/${cell.feature}/CONTEXT.md`,
    `- docs/history/${cell.feature}/plan.md`,
    '',
    'Contract:',
    '- Load the bee-executing skill immediately and follow its loop exactly.',
    '- Execute only the assigned cell. Do not select or accept other work.',
    '- Reserve every file before writing, under your nickname.',
    '- Return exactly one final status token: [DONE], [BLOCKED], [HANDOFF], or [NOOP].',
    '',
    'Startup:',
    '1. Read AGENTS.md.',
    '2. Run node .bee/bin/bee.mjs status --json',
    `3. Run node .bee/bin/bee.mjs cells show --id ${cell.id}`,
    '4. Reserve, implement, verify, cap, release, report.',
  ].join('\n');
}

const GATHER_SHAPED_GOAL = {
  gather: 'Gather: locate and digest the requested paths/facts. Read-only — never write, never edit, never run a mutating command.',
  reviewer: 'Review: check the given claim/diff against the repo. Read-only; may run read-only commands (tests, linters, the configured verify) to check evidence.',
  advisor: 'Advisor consult: produce an independent digest/opinion on the given question. Read-only.',
};

function gatherShapedPromptBody(kind) {
  return [
    GATHER_SHAPED_GOAL[kind] || `${kind}: read-only task.`,
    '',
    'Paths: <caller fills in the exact files/paths to read>',
    '',
    'Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.',
  ].join('\n');
}

function promptBodyFor(kind, cell, worker) {
  return kind === 'cell' ? cellPromptBody(cell, worker) : gatherShapedPromptBody(kind);
}

// PREPARE-TIME RECORD (advisor R2): one line per prepared dispatch, appended
// to the SAME .bee/logs/dispatch.jsonl the guard's own enforcement audit
// writes to, distinguished by source:'prepare' — no correlation with the
// guard's later enforcement line is attempted (a different dispatch_id/ts,
// on purpose: this is "what was asked for", the guard's line is "what was
// allowed/denied"). Fail-open like every other bee log write: a log failure
// never blocks prepare from returning its payload.
function appendPrepareRecord(root, record) {
  try {
    const logsDir = path.join(root, '.bee', 'logs');
    fs.mkdirSync(logsDir, { recursive: true });
    fs.appendFileSync(
      path.join(logsDir, 'dispatch.jsonl'),
      `${JSON.stringify({ ts: new Date().toISOString(), source: 'prepare', ...record })}\n`,
    );
  } catch {
    // fail-open — the prepare record is an audit convenience, never a blocker
  }
}

/**
 * prepareDispatch(root, {runtime, kind, cell, classification}) -> the payload
 * envelope, or a typed refusal ({ok:false, ...}). Throws only on a malformed
 * CALL (bad runtime/kind, missing/unknown --cell for kind 'cell') — never on
 * a legitimate cli-shaped, unconfigured-advisor, or native-unavailable
 * resolution, which are typed refusals returned to the caller, not
 * exceptions.
 *
 * `classification` (codex-native-transport D1/D3/R3-R5, binding) is the
 * caller-supplied verdict of `readNativeTransportClassification(root)` —
 * this lib module deliberately never imports or calls that reader itself
 * (it lives in bee.mjs, the bin layer; a lib module reaching back into bin
 * would invert the repo's bin->lib import direction). bee.mjs's own
 * `dispatch prepare` handler is the one production caller that reads the
 * live probe and passes its `.classification` string through; every other
 * caller (including every test in this repo) that omits `classification`
 * gets exactly D3's documented "unprobed/unknown ⇒ native_budget_only"
 * behavior — which, for a non-native-shaped slot, is simply inert (only
 * `resolved.type === 'native'` ever reads this parameter at all), so every
 * existing budget-only/model/cli/refused caller stays byte-identical.
 *
 * `worker` (hardening-7, required when `kind === 'cell'`) names the caller
 * requesting the dispatch; it is checked against the loaded cell's own
 * `status`/`trace.worker` (checkCellClaimOwnership, above) so `prepare`
 * refuses to build a payload for a cell nobody claimed, or that another
 * worker currently owns — a dispatch payload is authority to act on a
 * cell, and prepare must never hand that out to a caller who doesn't
 * (yet) hold the claim. `forceOwnership` (--force-ownership) bypasses the
 * refusal and appends an audited `ownership_override` entry to the same
 * prepare-time record every dispatch already writes (mirrors msh-4's
 * "force always leaves an audit line" discipline). Missing `worker` on a
 * `kind: 'cell'` call is a malformed CALL (throws), same as missing
 * `cell`; a claimed-elsewhere or unclaimed cell is a legitimate refusal
 * (typed, not thrown) a caller can retry after claiming, or override.
 */
export function prepareDispatch(root, { runtime, kind, cell: cellId, worker, forceOwnership = false, classification } = {}) {
  if (!DISPATCH_RUNTIMES.includes(runtime)) {
    throw new Error(`dispatch prepare: --runtime must be one of ${DISPATCH_RUNTIMES.join('|')}, got "${runtime}".`);
  }
  if (!DISPATCH_KINDS.includes(kind)) {
    throw new Error(`dispatch prepare: --kind must be one of ${DISPATCH_KINDS.join('|')}, got "${kind}".`);
  }

  let cell = null;
  let ownershipOverride = null;
  let resolvedWorker = null;
  if (kind === 'cell') {
    if (!cellId) {
      throw new Error('dispatch prepare: --cell is required when --kind cell.');
    }
    cell = readCell(root, cellId);
    if (!cell) {
      throw new Error(`dispatch prepare: cell "${cellId}" not found.`);
    }
    if (typeof worker !== 'string' || !worker.trim()) {
      throw new Error('dispatch prepare: --worker is required when --kind cell.');
    }
    const trimmedWorker = worker.trim();
    resolvedWorker = trimmedWorker;
    const ownership = checkCellClaimOwnership(cell, trimmedWorker);
    if (!ownership.ok && !forceOwnership) {
      return {
        ok: false,
        type: 'refused',
        reason: 'claim_ownership',
        code: ownership.code,
        status: ownership.status,
        owner: ownership.owner,
        fix: ownership.reason,
      };
    }
    if (forceOwnership) {
      // hardening-7 (msh-4 mirror): logs whether or not there was actually a
      // conflict to bypass — "force always leaves an audit line", never
      // conditional on whether it turned out to be needed.
      //
      // hardening-1-7-10 (D7): `transferred` is ALWAYS false here — this is
      // advisory-only, on purpose. cells.mjs's claims.mjs exposes
      // adoptClaim(root, cellId, newSessionId), a real transfer primitive,
      // but it operates on a DIFFERENT ownership axis: the SESSION-based
      // claims-store file cells.mjs's own checkClaimOwnership reads. This
      // function's ownership check (checkCellClaimOwnership, above) is on
      // the CELL RECORD's own trace.worker string — a plain name, no session
      // concept, never touching the claims.mjs store at all (see this
      // module's own docstring). Calling adoptClaim here would transfer the
      // wrong record and silently leave cell.trace.worker exactly as it was,
      // which would be worse than doing nothing — a caller reading
      // "transferred" would believe the cell's real owner changed when it
      // did not. There is no simple, correct transfer primitive on this
      // axis (it would mean a new cells.mjs mutator to rewrite trace.worker
      // on an already-claimed cell, an architectural addition out of scope
      // for this cell), so `forceOwnership` stays a bypass of THIS
      // function's own refusal only: the caller may build and use the
      // payload, but the cell's actual claim ownership (trace.worker) is
      // untouched by this call.
      ownershipOverride = {
        forced_by: trimmedWorker,
        bypassed: !ownership.ok,
        code: ownership.ok ? null : ownership.code,
        owner_bypassed: ownership.ok ? null : ownership.owner,
        status_bypassed: ownership.ok ? null : ownership.status,
        transferred: false,
        note: 'advisory bypass only — cell.trace.worker (the actual claim owner) was NOT transferred; no correct transfer primitive exists on this ownership axis (see comment above).',
      };
    }
  }

  const tierToken = slotForKind(kind);
  let resolved;
  if (kind === 'advisor') {
    resolved = resolveAdvisor(root, runtime);
    if (resolved == null) {
      return {
        ok: false,
        reason: 'advisor_not_configured',
        fix: `set models.${runtime}.advisor in .bee/config.json to enable an advisor consult (resolveAdvisor never falls back to another tier).`,
      };
    }
  } else {
    resolved = resolveTier(root, tierToken, runtime, purposeForKind(kind));
    if (resolved.type === 'refused') {
      // advisor A1: prepare NEVER routes around a refusal — surfaced verbatim,
      // never coerced into a payload.
      return { ok: false, type: 'refused', reason: resolved.reason, slot: resolved.slot, fix: resolved.fix };
    }
  }

  const promptBody = promptBodyFor(kind, cell, resolvedWorker);
  const requestedModel = resolved.type === 'model' ? resolved.model : null;
  const pinnedType = PINNED_AGENT_TYPE[tierToken] || 'general-purpose';

  let tool;
  let payload;
  let channel;
  // Native-override-only extras (codex-native-transport D1/D3a, R5): never
  // populated on any other path, so every non-native envelope/log line below
  // stays byte-identical to what it was before this branch existed.
  let refusal = null;
  let nativeConfirmed = false;
  let envelopeExtra = {};

  if (resolved.type === 'native') {
    // Native V2 model-override routing (D1/D5/D7, native-transport R3/R5):
    // `resolved` here is state.mjs's {type:'native', model, effort?,
    // fork_turns, agent_type, fallback?} — a CONFIG-time decision (a slot is
    // shaped {kind:'native',...}). Whether the client can actually accept an
    // override spawn is a separate RUNTIME fact — `classification`, gated
    // strictly on the reader this module never calls directly (see the
    // docstring above). D1: a native route that is requested but
    // unavailable/refused reports its reason and falls back to CLI only when
    // config explicitly permits it — silent native->CLI switching is
    // forbidden, and so is silently downgrading to a marker-only budget spawn.
    nativeConfirmed = classification === NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE;
    if (nativeConfirmed) {
      tool = 'spawn_agent';
      payload = {
        agent_type: resolved.agent_type || 'worker',
        // Marker at the very start of message — the exact anchored position
        // every other codex spawn_agent payload uses (D5: the marker anchor
        // never moves for a native-override payload either).
        message: `[bee-tier: ${tierToken}]\n${promptBody}`,
        model: resolved.model,
        // E2/D2: a full-history fork rejects model overrides — 'none' is a
        // VALIDITY precondition for an override spawn, never merely context
        // hygiene, so this is hardcoded rather than trusted to whatever
        // resolved.fork_turns happens to carry.
        fork_turns: 'none',
      };
      if (resolved.effort != null) {
        payload.reasoning_effort = resolved.effort;
      }
      channel = 'codex-native';
      envelopeExtra = { transport: 'native-override' };
    } else if (resolved.fallback && resolved.fallback.type === 'cli' && typeof resolved.fallback.command === 'string' && resolved.fallback.command) {
      // D1 explicit-only fallback + D3a coupling (decision c0cba64e): only
      // ever the slot's OWN configured fallback command — this branch is the
      // one legitimate route to CLI from a native slot; nothing here invents
      // a command from anywhere else, and a classification of
      // external_cli_only is treated identically to native_budget_only (both
      // are simply "not confirmed native_model_override").
      tool = 'Bash';
      payload = { command: resolved.fallback.command, stdin: promptBody };
      channel = 'cli-exec';
      envelopeExtra = { fallback_reason: 'native_unavailable' };
    } else {
      // No confirmed override and no explicit fallback configured on this
      // slot: D1's "never silent" — a typed refusal naming the classification
      // that blocked it, never an invented CLI command, never a silent
      // downgrade to a marker-only budget spawn (D3a coupling).
      refusal = {
        ok: false,
        type: 'refused',
        reason: 'native_unavailable',
        detail: classification || NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
      };
    }
  } else if (resolved.type === 'cli') {
    // External-executor dispatch (swarming-reference.md "External Executors"):
    // never an Agent/spawn_agent tool call — an in-family subagent cannot BE
    // the external CLI. The prompt is carried on stdin, matching the
    // promptVia:'stdin' convention documented on cli-shaped config slots.
    tool = 'Bash';
    payload = { command: resolved.command, stdin: promptBody };
    channel = 'cli-exec';
  } else if (runtime === 'codex') {
    tool = 'spawn_agent';
    // Live-probed codex 0.145.0 schema (i54-closeout D1, validation-canary):
    // {task_name, message} are REQUIRED and `agent_type` does not exist in
    // the server-governed schema, so the ordinary emit is the doc-canonical
    // {task_name, message, fork_turns} shape swarming-reference.md teaches.
    // `model`/`reasoning_effort` exist in that schema but the override path
    // is rejected end-to-end on 0.145.0 (`native_budget_only`) — the
    // ordinary path never attaches them (R18: emit only what was observed
    // accepted; overrides ride the confirmed-native branch above only).
    payload = {
      // A stable, followup_task-addressable name: the cell id for cell
      // dispatches, the kind otherwise (the caller may rename before spawn).
      task_name: cell ? cell.id : `bee-${kind}`,
      // Marker at the very start of message — the exact position
      // dispatch-guard.mjs's evaluateDispatch checks (ANCHORED_TIER_MARKER_RE).
      message: `[bee-tier: ${tierToken}]\n${promptBody}`,
      // ORCH-02 isolation guarantee: never fork the parent history for
      // routine dispatches (swarming-reference.md "Isolation guarantee").
      fork_turns: 'none',
    };
    channel = 'codex-native';
    // Codex's Multi-Agent V2 spawn_agent DOES accept a per-agent model
    // override (model/reasoning_effort/fork_turns) — real and catalog-
    // validated, but hidden from the visible tool schema by default
    // (hide_spawn_agent_metadata=true, E1/E6, codex-native-transport). This
    // branch is the path taken whenever no confirmed native override applies
    // (no native slot configured for this tier, or one is configured but
    // `classification` above did not confirm override acceptance on this
    // host): the tier is enforced as a read budget + output cap stated in
    // the prompt, never a structural param — exactly the same budget-only
    // shape this branch has always produced.
  } else {
    tool = 'Agent';
    payload = {
      subagent_type: pinnedType,
      prompt: `[bee-tier: ${tierToken}]\n${promptBody}`,
      description: `${kind} (${requestedModel || tierToken})`,
    };
    if (resolved.type === 'model') {
      payload.model = resolved.model;
    }
    channel = 'claude-agent';
  }

  if (refusal) {
    return refusal;
  }

  // Shared derivation (g22-2, GH #22 P1-6 D3; extended native-transport R5):
  // the honest pinned/unverified/inherited-or-unknown/native-requested split
  // now lives ONCE in dispatch-guard.mjs's deriveEconomics, so this module's
  // economics block and the enforcement hook's dispatch-log economics can
  // never independently drift. A structural `model` param exists here ONLY
  // on the claude-agent channel when resolved.type === 'model' (the exact
  // condition, above, that set payload.model) — a confirmed native override
  // carries its own structural `model` field but through the SEPARATE
  // `nativeConfirmed` flag, never through `paramModel` (that stays a
  // claude-agent-only concept); codex-native's budget-only spawn has no
  // model field at all, and cli-exec's Bash payload names its own model
  // outside this vocabulary.
  const paramModel = channel === 'claude-agent' && resolved.type === 'model' ? resolved.model : null;
  const economics = deriveEconomics({ channel, tier: tierToken, paramModel, resolved, nativeConfirmed });

  const dispatch_id = crypto.randomUUID();

  appendPrepareRecord(root, {
    dispatch_id,
    kind,
    cell: cell ? cell.id : null,
    runtime,
    ...(envelopeExtra.fallback_reason ? { native_fallback_reason: envelopeExtra.fallback_reason, native_classification: classification || null } : {}),
    ...(envelopeExtra.transport ? { native_classification: classification || null } : {}),
    ...(ownershipOverride ? { ownership_override: ownershipOverride } : {}),
    ...economics,
  });

  return {
    tool,
    payload,
    dispatch_id,
    economics,
    ...envelopeExtra,
    // hardening-1-7-10 (D7): surfaced to the CALLER, not only logged into
    // .bee/logs/dispatch.jsonl via appendPrepareRecord below — a caller
    // that passed --force-ownership must be able to see, from the returned
    // envelope itself, that ownership was bypassed for THIS call only and
    // never actually transferred (ownershipOverride.transferred is always
    // false; see the comment where it is built, above).
    ...(ownershipOverride ? { ownership_override: ownershipOverride } : {}),
  };
}
