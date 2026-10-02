// bee's Pi enforcement belt. This file holds ZERO guard rules of its own —
// every allow/deny/advisory verdict comes from `.bee/bin/bee hook <name>`, the
// exact same brain the Claude, Codex and OpenCode belts call
// (packages/bee/hooks/claude-hooks.json, .opencode/plugins/bee-guard.ts).
// This file only:
//
//   1. locates the bee STORE and the bee BINARY (mirrors claude-hooks.json's
//      project-then-main-worktree search, since a linked worktree checkout has
//      no vendored .bee/bin/bee of its own) — re-checked on EVERY call, never
//      once at load time, so an in-session `bee onboard` starts guarding
//      without a `/reload` (D1/CONTEXT.md "Agent's Discretion" passivity rule);
//   2. maps every Pi tool this extension routes onto the JSON shape the target
//      bee hook already reads on stdin — a field-name translation for the
//      enumerated built-ins, and a FAIL-SAFE write-capable route for every
//      name outside that map (see mapToolCall below); and
//   3. picks ONE of exactly two failure policies per surface, never a third:
//        BLOCKING (`tool_call` only) — write-guard. bee's documented DENY
//        verdict (exit code 2, reason on stderr) becomes Pi's documented block
//        return, `{ block: true, reason }` (Pi 0.84.3 docs/extensions.md
//        "tool_call"). Deny, crash, missing binary, an "ask" verdict and an
//        unparseable exit-0 verdict ALL block — D3 fail CLOSED, all the way
//        through.
//        ADVISORY (session_start, before_agent_start, tool_result,
//        agent_settled) — these NEVER throw and never block: a missing binary,
//        a spawn error, a crash, or any exit code is swallowed and logged to
//        stderr for a human to notice, never surfaced as a session-ending
//        exception. This is bee's own posture too — hooks/mod.rs's
//        `emit_undecidable` already resolves bee's OWN could-not-decide
//        outcome to exit 0 (fail-open BY DESIGN); this file's advisory wrapper
//        additionally swallows failures ON THE PI SIDE that never even reach
//        that native fail-open path. A fail-open host swallows a fail-CLOSED
//        throw right back into an allow — the one failure mode the BLOCKING
//        path above must never have, and exactly why the two policies never
//        mix on the same call (pattern 20260714).
//   4. drains this session's RESULT INBOX (pi-result-mailbox D4/D5/D6) — the
//      one surface here that is not a hook call at all. A detached
//      `bee herding run --inbox-session <token>` leaves a pending marker for
//      the orchestrator session whose id is that token; this file polls for
//      the marker's finished result and injects a HEADER into the session.
//      It is ADVISORY-class like everything in (3) above: it never throws and
//      never blocks, and its timer is created inside `session_start` (never at
//      load time) and `.unref()`d, so a host that merely imports this file can
//      still exit. See "the result-inbox drain" below.
//
// PASSIVITY (CONTEXT.md, Agent's Discretion): a repo with no `.bee` DIRECTORY
// at the project root or at the main worktree root is not a bee repo — every
// handler here returns without running anything and without printing anything.
// A `.bee` directory PRESENT with the binary MISSING is the opposite case: the
// repo IS bee-managed and the guard cannot decide, so the blocking path blocks
// (D3) and the advisory path logs. Both checks run per call, never cached.
//
// model-guard is a NAMED EXCLUSION on this belt — n/a — Pi has NO native
// subagent surface: no Agent tool, no Task tool, no subagent_type parameter
// anywhere in its built-in tool registry (store decision 7f9c8518; plan.md
// "model-guard is a NAMED EXCLUSION on Pi"). Every worker dispatch from a Pi
// session routes through the herding transport instead, which is a bee CLI
// call (`bee herding run`) and therefore already covered by write-guard on the
// `bash` tool. A model-guard row wired here would be a vacuous name-match that
// can never fire — the exclusion is asserted BY NAME in the belt parity test.
//
// codex-subagent-audit is a NAMED EXCLUSION on this belt — Codex-specific (same reason it is not wired on the OpenCode belt).
// chain-nudge is a NAMED EXCLUSION on this belt — needs subagent-dispatch identity that no Pi event carries (same reason it is not wired on the OpenCode belt).
//
// ── activity hook mapping across Claude lifecycle rows ─────────────────────
// The Claude manifest (packages/bee/hooks/claude-hooks.json) fires activity
// on eight distinct lifecycle events. The Pi belt maps each row onto the
// closest honest Pi lifecycle carrier and passes the original Claude event
// name in hook_event_name (activity is a state machine keyed on Claude names):
//   1. UserPromptSubmit    -> before_agent_start (session_id, prompt, cwd) and ui_prompt_end
//   2. PreToolUse          -> tool_execution_start (session_id, tool_name, tool_use_id, cwd)
//   3. PostToolUse         -> tool_result when !isError (session_id, tool_name, tool_use_id, cwd)
//   4. PostToolUseFailure  -> tool_result when isError (session_id, tool_name, tool_use_id, cwd)
//   5. PermissionRequest   -> NAMED EXCLUSION: Pi 0.84–0.85 has no interactive permission prompt event
//   6. Notification        -> ui_prompt_start (session_id, cwd; notification_type: agent_needs_input)
//   7. Stop                -> agent_settled (session_id, cwd)
//   8. SessionEnd          -> session_shutdown when reason is not "reload" (session_id, cwd, reason)
//
// ── the Pi range this belt supports ────────────────────────────────────────
// FLOOR: Pi 0.84.4. This belt registers `ui_prompt_start` and `ui_prompt_end`
// (rows 6 and 1 above), and Pi added both events in 0.84.4, so the belt cannot
// run on 0.84.3.
// CEILING: Pi 1.0.0, proven by live run 20261002-135348-2965648, a bee workflow
// driven end to end on it (.bee/verify/verify-app/features/pi-hat-wave.md).
//
// A version named anywhere else in this file records which Pi docs or binary
// were READ for the fact beside it. That is a different claim from the range
// above, it is still true as written, and it is deliberately left alone
// (docs/history/pi-stage-dispatch/plan.md). Do not "fix" those to match this
// block.
//
// The keep/adapt/delete disposition for every Pi extension-API change across
// this range, and the list of Pi surfaces an upgrade can break, live in
// docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md.

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
import { registerEvents, type Belt } from "./events.ts"
import { registerCommands } from "./commands.ts"
import {
  executeVerdictTool,
  VERDICT_TOOL_NAME,
  VERDICT_TOOL_PARAMETERS,
  verdictTool,
} from "./tool-verdict.ts"
import { beeDispatchTool, beeAdvisorTool } from "./tool-dispatch.ts"
import {
  BEE_STEER_TOOL_NAME,
  BEE_STEER_TOOL_PARAMETERS,
  beeSteerTool,
  executeBeeSteerTool,
} from "./tool-steer.ts"
import { mapToolCall, PI_BUILTIN_TOOLS } from "./tool-map.ts"
import {
  DEFAULT_MERGE_QUEUE_WAIT_MS,
  derivePostExitTimeoutMs,
  NODE_MAX_TIMER_TIMEOUT_MS,
  POST_EXIT_MERGE_MARGIN_MS,
} from "./bee-cli.ts"
import { validateTransitionIntent } from "./transition.ts"
import {
  createInFlightWorkersWidget,
  formatWorkerRow,
  getInFlightMarkers,
  IN_FLIGHT_WORKERS_WIDGET_KEY,
  refreshInFlightWorkersWidget,
  shortJobSuffix,
} from "./workers-widget.ts"

// ─── the belt ──────────────────────────────────────────────────────────────

export default function (pi: ExtensionAPI) {
  const belt: Belt = {
    fullToolSet: null,
    lastStage: null,
    toolsReopened: false,
  }

  registerEvents(pi, belt)
  registerCommands(pi, belt)

  if (typeof (pi as any).registerTool === "function") {
    ;(pi as any).registerTool(verdictTool)
    ;(pi as any).registerTool(beeDispatchTool)
    ;(pi as any).registerTool(beeAdvisorTool)
    ;(pi as any).registerTool(beeSteerTool)
  }
}

// Exported for the belt parity/contract suite (pi_plugin_contracts.rs), which
// derives this belt's rows from this source rather than a hand list.
export {
  PI_BUILTIN_TOOLS,
  mapToolCall,
  derivePostExitTimeoutMs,
  DEFAULT_MERGE_QUEUE_WAIT_MS,
  POST_EXIT_MERGE_MARGIN_MS,
  NODE_MAX_TIMER_TIMEOUT_MS,
  validateTransitionIntent,
  VERDICT_TOOL_NAME,
  VERDICT_TOOL_PARAMETERS,
  executeVerdictTool,
  verdictTool,
  BEE_STEER_TOOL_NAME,
  BEE_STEER_TOOL_PARAMETERS,
  executeBeeSteerTool,
  beeSteerTool,
  IN_FLIGHT_WORKERS_WIDGET_KEY,
  shortJobSuffix,
  formatWorkerRow,
  createInFlightWorkersWidget,
  getInFlightMarkers,
  refreshInFlightWorkersWidget,
}
