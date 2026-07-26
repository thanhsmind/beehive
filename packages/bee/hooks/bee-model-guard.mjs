#!/usr/bin/env node
// bee-model-guard: PreToolUse (Agent|Task) — Claude projection only
// (hooks/catalog.mjs ALLOWED_DIFFERENCES: Codex does not expose collaboration
// spawn through PreToolUse).
// Enforces explicit-tier transport on subagent dispatch (decision 0023, plan
// docs/history/model-tier-guard/plan.md D1/D2, advisor-and-orchestration
// 2A-iii B4/B5/AO5): every Agent/Task dispatch must carry either a valid
// tool_input.model or a case-insensitive [bee-tier: <tier>] marker ANCHORED to
// a reserved position: the first non-whitespace token of tool_input.prompt, or
// the start of tool_input.description (leading whitespace allowed either way).
// A marker occurring anywhere else (embedded after other prompt text,
// mid-description) never satisfies the transport — that would let quoted plan
// text, user content, or retrieved docs forge the tier with no real decision
// made (review-findings.md P1-1).
//
// The declared tier is read BEFORE the model param is judged (2A-iii closes the
// old short-circuit that accepted ANY non-empty model string and logged it as a
// legitimate transport): (1) marker + param must AGREE — the param strictly
// equals the model the marker's tier resolves to, and a tier with no model name
// (ceiling/budget/cli) must carry no param; (2) a bare param with no marker must
// be a MEMBER of the models configured across the claude tier slots (config is
// the sole authority — no hardcoded allowlist; an unconfigured repo fail-opens);
// (3) a marker whose tier is cli-shaped is denied and routed to the external
// executor (an in-family subagent cannot BE the external CLI); (4) a bare
// dispatch (no param, no anchored marker) silently inherits the most expensive
// session model, so it is denied.
// (0, W3/AO5/AO10) BEFORE any of the above: a marker naming generation/
// extraction/review paired with subagent_type "general-purpose" is denied
// outright — those three tiers each have a rendered bee agent type
// (bee-gather/bee-extract/bee-review) and general-purpose carries no pinned
// identity; "ceiling" has no rendered agent and stays exempt.
// Deny = exit 2 with the reason (rule + FIX line) on stderr, and a
// {hook:'model-guard',event:'deny',...} line appended to .bee/logs/hooks.jsonl.
// Input/root/crash-logging go through the shared runtime adapter
// (hooks/adapter.mjs, cell codex-parity-3): stdin is normalized before any
// property access and root discovery lives inside the fail-open boundary.
// Everything else is fail-open: exit 0 (crashes logged to .bee/logs/hooks.jsonl).
// Deny only — this hook never auto-injects or rewrites the model param.

import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl, appendHookLog } from "./adapter.mjs";

const HOOK_NAME = "model-guard";
// Codex-native collaboration spawn (codex-native-runtime-v2 D4): Codex exposes
// agent spawns through PreToolUse as tool_name "spawn_agent" (spike codex-cli
// 0.144.4, capability-matrix row D1), with tool_input.agent_type "worker" and
// the task text in tool_input.message. This is an ISOLATED branch, never a
// member of DISPATCH_TOOLS — the Claude enforcement reads model/tier/
// subagent_type and emits Claude-specific model-param remediation, which would
// misread a real Codex spawn message as a bare Claude dispatch. Both branches'
// actual decision logic now lives in lib/dispatch-guard.mjs's evaluateDispatch
// (g22-1, GH #22 P0-3 PRECONDITION REFACTOR) — this hook is a thin wrapper:
// parse stdin payload -> evaluateDispatch -> the same exit codes, stderr
// messages, and logDispatch/logDeny audit lines as before the extraction.
const CODEX_SPAWN_TOOL = "spawn_agent";
const DISPATCH_TOOLS = new Set(["Agent", "Task"]);

function logDeny(root, toolName, toolInput) {
  // appendHookLog is itself fail-open: a log failure never blocks the deny.
  appendHookLog(root, {
    ts: new Date().toISOString(),
    hook: HOOK_NAME,
    event: "deny",
    tool_name: toolName,
    tool_input_keys: Object.keys(toolInput),
  });
}

// Dispatch economics (g22-2, GH #22 P1-6 D3 + advisor R3): the same honest
// channel/logical/requested/effective split dispatch-prepare.mjs's economics
// block carries, derived here through the ONE shared function
// (dispatch-guard.mjs's deriveEconomics) rather than a second hand-rolled
// copy. `dispatchGuard`/`stateLib` are the already-imported modules from
// main() below — this never does its own dynamic import, and never resolves
// a tier the caller did not already tell us about. `resolved` is looked up
// fresh (resolveTier is a read, not a mutation) ONLY when a tier is present,
// so a bare/unmarked dispatch (no tier at all) never pays for a lookup it
// cannot use. `paramModel` is exactly `model` — the structural param this
// hook already extracted from tool_input — so a claude-agent dispatch that
// carries a real param is 'pinned' and one that doesn't (marker-only, relying
// on the prompt budget) is 'unverified'; codex-native is ALWAYS
// 'inherited-or-unknown' regardless of tier/model (0.144.4 has no per-agent
// model selection to verify). Fails open to `null` fields on any resolution
// error — economics are an audit convenience, never a blocker.
function deriveDispatchEconomics(root, dispatchGuard, stateLib, toolName, isCodexSpawn, model, tier) {
  try {
    const channel = isCodexSpawn ? "codex-native" : "claude-agent";
    const runtime = isCodexSpawn ? "codex" : "claude";
    const resolved = tier ? stateLib.resolveTier(root, tier, runtime) : null;
    return dispatchGuard.deriveEconomics({ channel, tier: tier || null, paramModel: model || null, resolved });
  } catch {
    return null;
  }
}

// Dispatch audit log (P22, feature dispatch-log): one line per evaluated
// Agent/Task/spawn_agent dispatch — allowed or denied — so the resolved
// model/tier is auditable independent of what the UI shows. Fail-open like
// logDeny: a log failure never changes the guard's decision or exit code.
// `economics` (g22-2) is ADDITIVE ONLY — every field this function already
// wrote (transport/model/tier/subagent_type/description) stays byte-for-byte
// the same; economics is spread in alongside them, never over them, and is
// entirely absent (not even null-valued keys) when the caller has none to
// give (denied dispatches where nothing was derivable).
function logDispatch(root, toolName, toolInput, transport, model, tier, economics) {
  try {
    const logsDir = path.join(root, ".bee", "logs");
    fs.mkdirSync(logsDir, { recursive: true });
    const description =
      typeof toolInput.description === "string" ? toolInput.description.slice(0, 120) : "";
    fs.appendFileSync(
      path.join(logsDir, "dispatch.jsonl"),
      `${JSON.stringify({
        ts: new Date().toISOString(),
        tool: toolName,
        transport,
        model: model || null,
        tier: tier || null,
        subagent_type: typeof toolInput.subagent_type === "string" ? toolInput.subagent_type : null,
        description,
        ...(economics || {}),
      })}\n`,
    );
  } catch {
    // fail-open — auditing never blocks a dispatch
  }
}

// A single deny exit: record the rejected dispatch in the audit log (with the
// transport label that names WHY it was rejected), append the deny event, and
// write the reason to stderr for Claude Code to feed back on PreToolUse exit 2.
// Both log calls are fail-open — a log failure never changes the exit code.
function denyWith(root, toolName, toolInput, reason, transport, model, tier, economics) {
  logDispatch(root, toolName, toolInput, transport, model || null, tier || null, economics);
  logDeny(root, toolName, toolInput);
  process.stderr.write(reason);
  return 2;
}

async function main() {
  // FAIL-OPEN PRECISION (P1-2, now owned by hooks/adapter.mjs): the adapter
  // normalizes the payload before ANY property access — a top-level null or
  // array payload, or a non-string cwd, can never reach a `.` access or
  // path.resolve — and resolves the repo root inside its own fail-open
  // boundary.
  const ctx = await readHookContext(HOOK_NAME);
  const payload = ctx.payload;
  const root = ctx.root;
  if (!root) {
    return 0;
  }

  try {
    if (!fs.existsSync(path.join(root, ".bee", "bin", "lib", "state.mjs"))) {
      return 0;
    }

    const toolName = payload.tool_name || payload.toolName || "";
    // Codex spawn recognition keys on the AUTHORITATIVE field payload.tool_name
    // ONLY — a top-level `toolName` alias is not the observed Codex envelope and
    // fails open. The Claude dispatch set (Agent|Task) is evaluated exactly as
    // before; this line adds the Codex tool without changing that behavior.
    const isCodexSpawn = payload.tool_name === CODEX_SPAWN_TOOL;
    if (!isCodexSpawn && !DISPATCH_TOOLS.has(toolName)) {
      return 0;
    }

    const stateLib = await import(libModuleUrl(root, "state.mjs"));
    if (!stateLib.hookEnabled(root, HOOK_NAME)) {
      return 0;
    }

    // The decision core (PRECONDITION REFACTOR, g22-1): dynamically imported
    // from the ROOT's own .bee/bin/lib/, exactly like state.mjs above — never
    // a hooks-side vendored copy, so this always evaluates against the
    // installed repo's own dispatch-guard.mjs. `toolInput` is exactly what
    // evaluateDispatch expects as its second argument for both branches: the
    // Claude Agent/Task tool_input object, or the Codex spawn_agent tool_input
    // object ({agent_type, message, ...}) — never a further-wrapped payload.
    const dispatchGuard = await import(libModuleUrl(root, "dispatch-guard.mjs"));
    const toolInput = payload.tool_input;
    const result = dispatchGuard.evaluateDispatch(toolName, toolInput, root);

    // transport === null: evaluateDispatch had no opinion (wrong tool, or a
    // malformed/absent/unobserved tool_input shape) — never log, exit 0
    // silently, byte-identical to the pre-refactor early returns.
    if (result.transport === null) {
      return 0;
    }
    // Economics (g22-2): derived from the SAME decision fields evaluateDispatch
    // already returned, for both allow and deny (advisor R3: "denied dispatches
    // get the fields too where derivable"). A resolution failure fails open to
    // null, never blocking or altering the allow/deny outcome itself.
    const economics = deriveDispatchEconomics(
      root,
      dispatchGuard,
      stateLib,
      toolName,
      isCodexSpawn,
      result.model,
      result.tier,
    );
    if (result.decision === "deny") {
      return denyWith(root, toolName, toolInput, result.reason, result.transport, result.model, result.tier, economics);
    }
    // Allow: log the evaluated transport exactly as every allow branch did
    // before the refactor (model-param / marker / codex-spawn-marker).
    logDispatch(root, toolName, toolInput, result.transport, result.model, result.tier, economics);
    return 0;
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
}

process.exitCode = await main();
