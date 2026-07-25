// dispatch-guard.mjs — the pure decision core for bee's dispatch-transport
// enforcement (g22-1, GH #22 P0-3 PRECONDITION REFACTOR / advisor R1).
//
// Extracted verbatim (byte-for-byte decision logic, zero behavior change)
// from hooks/bee-model-guard.mjs so it can be shared by two callers that both
// need the SAME allow/deny judgment without duplicating it:
//   - hooks/bee-model-guard.mjs (PreToolUse enforcement): dynamically imports
//     this module the same way it dynamically imports state.mjs — from the
//     ROOT's own .bee/bin/lib/, never a hooks-side vendored copy — and turns
//     evaluateDispatch()'s decision into the deny exit code / stderr / audit
//     log lines it always has.
//   - lib/dispatch-prepare.mjs (`bee dispatch prepare`): builds a payload,
//     then re-runs it through evaluateDispatch() itself as a self-check
//     before handing the payload back (the ALLOW/DENY the hook would render
//     is computed once and trusted by both sides — one source of truth,
//     never two copies of the same judgment call).
//
// This module does ZERO I/O — no fs writes, no stderr, no logging. Every
// decision is a pure function of (toolName, toolInput, root); `root` is only
// used to call resolveTier()/modelForTier(), which read .bee/config.json
// (a read, not a mutation). Callers own all side effects.

import { resolveTier, resolveAdvisor, modelForTier, CONFIGURABLE_SLOTS } from './state.mjs';

// Codex-native collaboration spawn (codex-native-runtime-v2 D4; schema
// re-probed live on codex-cli 0.145.0 — i54-closeout D1,
// docs/history/i54-closeout/reports/validation-canary.md): Codex exposes
// agent spawns through PreToolUse as tool_name "spawn_agent". The 0.145.0
// server-governed schema is {task_name (required), message (required),
// fork_turns?, model?, reasoning_effort?} — `agent_type` does not exist in
// it; the legacy 0.144.4 spike shape was {agent_type: "worker", message}.
// The guard therefore keys on the TOOL NAME plus the message field, never on
// agent_type/task_name presence (both shapes get the same verdict). This is
// an ISOLATED branch, never mixed with the Claude Agent/Task rules below.
export const CODEX_SPAWN_TOOL = 'spawn_agent';
// Legacy 0.144.4 vocabulary — no longer part of any guard verdict (0.145.0
// has no agent_type field); kept exported for external readers of the old ABI.
export const CODEX_SPAWN_WORKER_TYPE = 'worker';

export const DISPATCH_TOOLS = new Set(['Agent', 'Task']);

// Native transport capability classification (codex-native-transport D3,
// advisor R3 — binding). PURE evidence -> classification mapping, zero I/O:
// the version+config-scoped probe record this classification is read from
// (validity legs, live re-checks, `codex features list` calls) lives in
// bee.mjs (readNativeTransportClassification / writeNativeTransportProbe,
// mirroring the g22-3 doctor-attest pattern) — this function only ever
// judges the evidence object it is handed, so it stays as pure as every
// other export in this file.
export const NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE = 'native_model_override';
export const NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY = 'native_budget_only';
export const NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY = 'external_cli_only';

/**
 * classifyNativeTransport(evidence) — D3a (authoritative, decision
 * c0cba64e): classification is schema/behavior evidence, never version
 * inference. `evidence` shape (all optional):
 *   { multi_agent: boolean, multi_agent_v2: boolean, override_spawn_accepted: boolean }
 * `multi_agent`/`multi_agent_v2` observed via the same `codex features list`
 * read; `override_spawn_accepted` observed by the g22-6 canary harness's
 * accepted-override-spawn probe under an isolated CODEX_HOME (D4: bee never
 * enables the flags on the user's real config, only inside the canary's own
 * per-run copy).
 *
 *   external_cli_only    <=> multi_agent === false (positive evidence the
 *                            base spawn transport is OFF — the ONLY
 *                            external trigger)
 *   native_model_override <=> multi_agent !== false AND multi_agent_v2 ===
 *                            true AND override_spawn_accepted === true
 *   native_budget_only    <=> everything else (v2 off, override not
 *                            accepted, partial/absent/unknown evidence) —
 *                            the feature stays inert until proven on the
 *                            host's actual build.
 */
export function classifyNativeTransport(evidence) {
  if (!evidence || typeof evidence !== 'object') {
    return NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY;
  }
  if (evidence.multi_agent === false) {
    return NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY;
  }
  if (evidence.multi_agent_v2 === true && evidence.override_spawn_accepted === true) {
    return NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE;
  }
  return NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY;
}

// Anchored to the start of the string (leading whitespace allowed): the
// marker must be the first thing in prompt/description/message, never merely
// present somewhere inside it (P1-1 — no 500-char scan window, no mid-text
// match).
export const ANCHORED_TIER_MARKER_RE = /^\s*\[bee-tier:\s*(ceiling|generation|extraction|review)\]/i;

// Codex-branch-only marker regex (codex-native-transport R1, cnt-3/cnt-4
// split — binding): additionally recognizes an `advisor` marker. `advisor`
// is a native-transport-only slot label (dispatch-prepare.mjs's
// slotForKind), never a CONFIGURABLE_SLOTS member — the claude branch's own
// ANCHORED_TIER_MARKER_RE above is deliberately left byte-unchanged (R1: "claude
// branch regex untouched"), so a claude dispatch can never forge an advisor
// tier through this widened set, and resolveTier's generation-coercion trap
// (state.mjs CONFIGURABLE_SLOTS comment) stays exactly as guarded as before.
export const ANCHORED_CODEX_TIER_MARKER_RE = /^\s*\[bee-tier:\s*(ceiling|generation|extraction|review|advisor)\]/i;

// W3 pinned-type rule (plan.md Slice 3B item 3, AO5/AO10/AO11): the three
// model-backed tiers each get a rendered bee agent definition
// (.claude/agents/bee-*.md) — "ceiling" deliberately has none, it IS the
// session model.
export const PINNED_AGENT_TYPE = {
  generation: 'bee-gather',
  extraction: 'bee-extract',
  review: 'bee-review',
};

function startsWithTierMarker(text, re = ANCHORED_TIER_MARKER_RE) {
  if (typeof text !== 'string') {
    return null;
  }
  const match = re.exec(text);
  return match ? match[1].toLowerCase() : null;
}

function markerTier(toolInput) {
  return (
    startsWithTierMarker(toolInput.description) || startsWithTierMarker(toolInput.prompt) || null
  );
}

// The set of model NAMES resolvable from the claude runtime's configured tier
// slots (extraction/generation/review). cli-shaped and null slots resolve to
// no model name and contribute nothing. This is the membership authority for
// a bare `model` param (B5) — config is the sole source, never a hardcoded
// allowlist. An empty set means the repo configures no model tier: the
// caller fail-opens (allow) exactly as before this slice.
function configuredModelSet(root) {
  const models = new Set();
  for (const slot of CONFIGURABLE_SLOTS) {
    const m = modelForTier(root, slot, 'claude');
    if (typeof m === 'string' && m.trim()) {
      models.add(m.trim());
    }
  }
  // Fold the advisor slot into the union (cnt-7, advisor-digest R2). advisor is
  // deliberately NOT a CONFIGURABLE_SLOTS member (state.mjs — decision 0015
  // collision avoided), so the loop above never sees it; yet `bee dispatch
  // prepare --runtime claude --kind advisor` emits {model: <advisor model>}
  // through the SAME resolveAdvisor resolver, and the guard must recognize
  // prepare's own payload or it denies bee's own advisor dispatches
  // ('param-not-configured' — the live prepare/guard asymmetry this closes).
  // Only a resolved {type:'model'} advisor contributes its model NAME; a
  // cli/native/null advisor resolves to no name and adds nothing, exactly like
  // a cli or null tier slot above — this widens the allowlist by the advisor
  // slot's own configured model and nothing more.
  const advisor = resolveAdvisor(root, 'claude');
  if (advisor && advisor.type === 'model' && typeof advisor.model === 'string' && advisor.model.trim()) {
    models.add(advisor.model.trim());
  }
  return models;
}

// A neutral "no opinion" result: transport === null tells every caller never
// to log anything — this shape is not a dispatch to evaluate at all (wrong
// tool, unobserved envelope, absent/malformed tool_input).
function noOpinion() {
  return { decision: 'allow', transport: null, reason: null, tier: null, model: null, subagentType: null };
}

function allowResult(transport, { tier = null, model = null, subagentType = null } = {}) {
  return { decision: 'allow', transport, reason: null, tier, model, subagentType };
}

function denyResult(reason, transport, { tier = null, model = null, subagentType = null } = {}) {
  return { decision: 'deny', transport, reason, tier, model, subagentType };
}

// Codex-native spawn guard (codex-native-runtime-v2 D4, decision 0023 parity;
// widened by i54-closeout D1). Triggered by tool_name "spawn_agent" for EVERY
// payload carrying a non-empty string message — the live codex 0.145.0 probe
// (validation-canary) observed the server-governed schema {task_name,
// message} (required) with no agent_type field, and the doc-canonical shape
// swarming-reference.md teaches ({task_name, message, fork_turns}) must get
// a real verdict, never a silent noOpinion. The legacy 0.144.4 spike shape
// ({agent_type: "worker", message}) is evaluated identically: the verdict
// keys on the tool name plus the message, NEVER on agent_type/task_name
// presence. The authoritative task field is MESSAGE, not prompt, and the
// [bee-tier: <tier>] marker must anchor to the START of message (leading
// whitespace allowed). Only a missing/empty/non-string message stays a
// no-opinion (there is no task text to judge); the unmarked-spawn deny is
// never weakened.
//
// D6 route-check gap (codex-native-transport, decision 350f1e82, bound to
// cnt-4): CONTEXT.md's D6 calls for this function to validate an override-
// carrying spawn's model/reasoning_effort/fork_turns against the configured
// route once such a spawn is observed. That route-check is INTENTIONALLY
// ABSENT here — this function never reads toolInput.model/reasoning_effort/
// fork_turns at all — because the PreToolUse envelope it would validate
// (V3) is terminal-UNOBSERVED on both codex builds probed to date: on
// 0.144.4 the hook chain never fired for a successful override spawn (root
// cause open); on 0.144.6 the override tool schema itself is REFUSED at the
// API level before any spawn_agent call is attempted, so no envelope ever
// reaches tool execution to inspect (full evidence:
// docs/history/codex-native-transport/reports/probe-evidence.md). A spawn
// that carries override fields therefore passes through exactly like one
// that doesn't — evaluated on the anchored message marker only — by design: this is
// a defense-in-depth allow-hole (ADVISOR-R2 Δ3), not an oversight, and it
// stays this way until a codex build lets V3 be observed and this comment
// is replaced by the real route-check.
function evaluateCodexSpawn(toolInput) {
  if (!toolInput || typeof toolInput !== 'object' || Array.isArray(toolInput)) {
    return noOpinion();
  }
  const message = toolInput.message;
  if (typeof message !== 'string' || message === '') {
    return noOpinion();
  }
  const tier = startsWithTierMarker(message, ANCHORED_CODEX_TIER_MARKER_RE);
  if (tier) {
    return allowResult('codex-spawn-marker', { tier });
  }
  const reason =
    'bee-model-guard: every Codex spawn_agent needs an explicit tier — its ' +
    "message must OPEN with a [bee-tier: <tier>] marker (decision 0023 " +
    "parity, codex-native-runtime-v2 D4, i54-closeout D1). A marker anywhere but the " +
    "start of the message does not count, and a marker in any other field is ignored; " +
    "without one the spawned worker silently inherits the session model.\n" +
    "FIX: begin the spawn message with the marker, e.g. " +
    '"[bee-tier: generation] <task>" (tiers: ceiling/generation/extraction/review/advisor).';
  return denyResult(reason, 'codex-spawn-unmarked');
}

// The Claude Agent/Task dispatch rules (decision 0023, hardened per P1-1,
// AO5, W3/AO10/AO11) — every branch of the original bee-model-guard.mjs main(),
// unchanged.
function evaluateClaudeDispatch(rawToolInput, root) {
  if (!rawToolInput || typeof rawToolInput !== 'object' || Array.isArray(rawToolInput)) {
    return noOpinion();
  }
  const toolInput = rawToolInput;

  const modelParam =
    typeof toolInput.model === 'string' && toolInput.model.trim() ? toolInput.model.trim() : null;
  const tier = markerTier(toolInput);
  const subagentType = typeof toolInput.subagent_type === 'string' ? toolInput.subagent_type : null;

  // (0) Pinned-type rule (W3, AO5/AO10/AO11) — fires BEFORE every allow
  // branch below.
  if (tier && tier !== 'ceiling' && subagentType === 'general-purpose') {
    const pinnedType = PINNED_AGENT_TYPE[tier];
    const reason =
      `bee-model-guard: [bee-tier: ${tier}] must spawn its pinned agent type, not ` +
      'subagent_type: "general-purpose" — general-purpose carries no tier identity and ' +
      "would run under whatever runtime default is in effect, not the rendered bee agent " +
      "for this tier (AO5/AO10).\n" +
      `FIX: set subagent_type: "${pinnedType}" (bee's rendered agent for the ${tier} tier), ` +
      'or use "Explore" for a read-only gather that does not need the rendered agent.';
    return denyResult(reason, 'generic-type-denied', { tier, model: modelParam, subagentType });
  }

  // (1) Marker + model param — AO5 strict equality.
  if (tier && modelParam) {
    const resolved = resolveTier(root, tier, 'claude');
    if (resolved.type === 'model') {
      if (modelParam === resolved.model) {
        return allowResult('model-param', { tier, model: modelParam, subagentType });
      }
      const reason =
        `bee-model-guard: [bee-tier: ${tier}] resolves to model "${resolved.model}", but ` +
        `the dispatch carries model: "${modelParam}" — the tier label and the param ` +
        "disagree, so the dispatch would run on the param while the audit records the " +
        "tier (AO5: config is the authority, the model does not get a vote).\n" +
        `FIX: set model: "${resolved.model}" to match the ${tier} tier, or drop the ` +
        "marker and declare the tier whose configured model is the one you want.";
      return denyResult(reason, 'param-tier-mismatch', { tier, model: modelParam, subagentType });
    }
    // inherit (ceiling) / budget / refused (cli): the tier carries NO model
    // name, so a param bolted onto it can only lie in the audit trail.
    const reason =
      `bee-model-guard: [bee-tier: ${tier}] resolves to no model name` +
      (resolved.type === 'refused' ? ' (the slot is a cli executor)' : '') +
      `, but the dispatch carries model: "${modelParam}". The marker would record one ` +
      "thing in dispatch.jsonl while the subagent actually runs on the param.\n" +
      "FIX: drop the model param (the marker alone selects the tier), or drop the marker " +
      "and declare the tier whose configured model equals the param you intended.";
    return denyResult(reason, 'param-on-nameless-tier', { tier, model: modelParam, subagentType });
  }

  // (2) Model param, no marker — B5 membership against configured tier slots.
  if (modelParam) {
    const memberSet = configuredModelSet(root);
    if (memberSet.size === 0 || memberSet.has(modelParam)) {
      // Empty set = unconfigured repo -> fail-open allow (today's behavior).
      return allowResult('model-param', { tier: null, model: modelParam, subagentType });
    }
    const configured = [...memberSet].sort().join(', ');
    const reason =
      `bee-model-guard: model: "${modelParam}" is not a model configured for any claude ` +
      "tier — a param outside config selects an unaudited model and, for an up-dispatch, " +
      "hides ceiling scarcity (AO5/B5: config is the sole authority; there is no hardcoded " +
      "allowlist).\n" +
      `FIX: use one of the configured models (${configured}); or, for a session-model ` +
      "dispatch, add [bee-tier: ceiling] (ceiling = the session model) to the " +
      "prompt/description; or add this model to a configured tier slot in .bee/config.json.";
    return denyResult(reason, 'param-not-configured', { tier: null, model: modelParam, subagentType });
  }

  // (3) Marker, no param — B4(1)/W10.
  if (tier) {
    const resolved = resolveTier(root, tier, 'claude');
    if (resolved.type === 'refused') {
      // A cli-shaped slot: an in-family Agent/Task subagent cannot BE the
      // external CLI (it runs as its own process, not a spawned subagent).
      const reason =
        `bee-model-guard: [bee-tier: ${tier}] resolves to a cli executor, which an ` +
        "in-family Agent/Task subagent cannot be — a cli tier runs as an external process, " +
        "not a spawned subagent.\n" +
        "FIX: dispatch it through the external-executor gather path — a Bash call running " +
        "the configured command verbatim with the prompt on stdin (resolveTier(root, slot, " +
        "runtime, {for:'gather'}) returns {type:'cli', command}). Do not attach a model " +
        "param; the cli command names its own model.";
      return denyResult(reason, 'cli-tier-denied', { tier, model: null, subagentType });
    }
    // model / budget / inherit -> allow (today's behavior, resolution-backed).
    return allowResult('marker', { tier, model: null, subagentType });
  }

  // (4) Bare — deny (today's behavior), but resolve the generation slot for
  // the FIX so we never tell the agent to pass a model that does not exist.
  const genResolved = resolveTier(root, 'generation', 'claude');
  const bareFix =
    genResolved.type === 'model'
      ? `FIX: pass model: "${genResolved.model}" for the generation tier, or add ` +
        '[bee-tier: ceiling] (or another tier: generation/extraction/review) to the prompt/description.'
      : 'FIX: add [bee-tier: ceiling] (or another tier: generation/extraction/review) to the ' +
        'prompt/description; the generation tier is a cli executor or unconfigured, so run it ' +
        'through the external-executor gather path (a Bash call with the command verbatim and ' +
        'the prompt on stdin) rather than a model param.';
  const reason =
    'bee-model-guard: every Agent/Task dispatch needs an explicit tier — a `model` ' +
    'param or a `[bee-tier: <tier>]` marker in the prompt/description (decision 0023). ' +
    'A bare dispatch would silently inherit the most expensive session model.\n' +
    bareFix;
  return denyResult(reason, 'bare-denied', { tier: null, model: null, subagentType });
}

// ─── Dispatch economics (g22-2, GH #22 P1-6, D3 + advisor R3) ──────────────
// A single honest mapping from "what the dispatch declared" to "what we can
// actually prove about the model that ran" — shared by hooks/bee-model-
// guard.mjs's logDispatch (enforcement-time audit line) and
// dispatch-prepare.mjs's economics block (prepare-time record), so the two
// sides of the SAME judgment (advisor A2's "one vocabulary" principle,
// extended from decisions to economics) never carry two independently
// hand-rolled status computations that could drift apart.
//
// This is a pure function — zero I/O, zero fs/log access — matching the rest
// of this module's contract. Callers resolve `resolved` themselves (via
// resolveTier/resolveAdvisor, which DO read .bee/config.json) and hand the
// typed result in; deriveEconomics only ever maps already-known facts to the
// {logical_tier, requested_model, effective_model, effective_model_status,
// channel, enforcement} shape. Never call resolveTier from in here.
//
// CHANNEL is the transport family (claude-agent / codex-native / cli-exec) —
// deliberately a NEW vocabulary, not a reuse of the legacy `transport` field
// evaluateDispatch/logDispatch already emit (advisor R3: transport keeps its
// existing enforcement-label meaning — 'model-param'/'marker'/'bare-denied'/
// etc — untouched; channel is additive, never a replacement).
//
// The honest mapping (GH #22 P1-6 D3):
//   - claude-agent + a real structural `model` param  -> 'pinned'; the
//     effective model IS the param (we watched the caller pass it).
//   - claude-agent + tier/budget only, no param        -> 'unverified'; we
//     know what SHOULD run (requested_model, if the tier resolves to a named
//     model) but never observe a structural pin, so we cannot claim more.
//   - codex-native (spawn_agent), NOT a confirmed native override -> 'inherited-
//     or-unknown', ALWAYS — codex-cli 0.144.4 has no per-agent model selection
//     at all (P14/P17), so claiming 'pinned' or even 'unverified' here would
//     imply a verification path that does not exist. This status never
//     changes based on tier/model inputs; only a confirmed native override
//     (below) justifies moving it.
//   - codex-native + a CONFIRMED native V2 model override (codex-native-
//     transport D1/D7, native-transport R5) -> 'native-requested': the tool
//     call itself carries a structural model param AND the live capability
//     probe (readNativeTransportClassification) has classified this host as
//     native_model_override, so codex's own catalog validation is a real
//     acceptance signal — a stronger claim than 'inherited-or-unknown', but
//     still short of 'pinned': catalog-accepted is not runtime-confirmed
//     (D7 — effective_model stays null; a child's self-report is never
//     evidence). Keyed strictly on `nativeConfirmed` — the caller decides
//     confirmation from resolved.type==='native' + a classification-confirmed
//     probe (R5); this function itself never re-derives that judgment.
//   - cli-exec (external executor payloads)             -> 'unverified'; the
//     cli command names its own model in its own argv, so requested_model is
//     always null here (nothing in bee's config vocabulary to report) even
//     when the resolved slot happens to carry cli metadata.
function deriveEconomics({ channel, tier = null, paramModel = null, resolved = null, nativeConfirmed = false } = {}) {
  const isNativeConfirmed = channel === 'codex-native' && resolved != null && resolved.type === 'native' && nativeConfirmed === true;
  const resolvedModel = resolved && (resolved.type === 'model' || resolved.type === 'native') ? resolved.model : null;

  let enforcement;
  if (channel === 'cli-exec') {
    enforcement = 'cli-command';
  } else if (isNativeConfirmed) {
    // A confirmed native override carries the model as a REAL structural
    // field on the spawn_agent payload (dispatch-prepare.mjs's `model`),
    // never merely a prompt-stated read budget — 'prompt-budget' would
    // misdescribe it exactly the way it never has for the claude-agent
    // model-param case.
    enforcement = 'native-model-param';
  } else if (channel === 'codex-native') {
    enforcement = 'prompt-budget';
  } else {
    // claude-agent (or any unrecognized channel defaults to the claude shape):
    // a real param structurally pins the dispatch; its absence means the tier
    // marker alone is carrying the request as a prompt-stated budget.
    enforcement = paramModel ? 'model-param' : 'prompt-budget';
  }

  let effectiveModel = null;
  let effectiveModelStatus;
  if (isNativeConfirmed) {
    effectiveModelStatus = 'native-requested';
  } else if (channel === 'codex-native') {
    effectiveModelStatus = 'inherited-or-unknown';
  } else if (channel === 'cli-exec') {
    effectiveModelStatus = 'unverified';
  } else if (paramModel) {
    effectiveModel = paramModel;
    effectiveModelStatus = 'pinned';
  } else {
    effectiveModelStatus = 'unverified';
  }

  // cli-exec never reports a requested_model (the command names its own,
  // outside bee's config vocabulary); every other channel prefers the actual
  // param when present, else falls back to what config would have named
  // (a native slot's model counts here too — informational either way, D7).
  const requestedModel = channel === 'cli-exec' ? null : paramModel || resolvedModel || null;

  return {
    logical_tier: tier,
    requested_model: requestedModel,
    effective_model: effectiveModel,
    effective_model_status: effectiveModelStatus,
    channel,
    enforcement,
  };
}

export { deriveEconomics };

// D5 (self-correcting-loop) — the 'pinned' string deriveEconomics assigns to
// effective_model_status, exported so judge.mjs's deriveModelIndependence
// reuses the SAME vocabulary value rather than a second hand-rolled literal
// that could silently drift from deriveEconomics' actual output (the "one
// vocabulary" principle this module's dispatch-economics comment above
// already documents for transport/enforcement).
export const PINNED_MODEL_STATUS = 'pinned';

/**
 * evaluateDispatch(toolName, toolInput, root) — the single decision function
 * both the guard hook and `bee dispatch prepare` call. `toolInput` is exactly
 * what the hook would see as `payload.tool_input` (for Codex: the object
 * carrying `agent_type`/`message` directly, not a further-wrapped envelope).
 *
 * Returns { decision: 'allow'|'deny', transport, reason, tier, model,
 * subagentType }. `transport === null` means "no opinion" — the caller must
 * never log a dispatch line for it (wrong tool, or a malformed/absent
 * tool_input that never reached a real branch). Every other transport value
 * — allow or deny — is a real evaluated dispatch and the caller logs it.
 */
export function evaluateDispatch(toolName, toolInput, root) {
  if (toolName === CODEX_SPAWN_TOOL) {
    return evaluateCodexSpawn(toolInput);
  }
  if (DISPATCH_TOOLS.has(toolName)) {
    return evaluateClaudeDispatch(toolInput, root);
  }
  return noOpinion();
}
