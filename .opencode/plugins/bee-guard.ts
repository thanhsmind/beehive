// bee's OpenCode enforcement belt. This file holds ZERO guard rules of its
// own — every allow/deny/advisory verdict comes from `.bee/bin/bee hook
// <name>`, the exact same brain the Claude and Codex hook belts call
// (packages/bee/hooks/claude-hooks.json). This file only:
//
//   1. locates the bee binary (mirrors claude-hooks.json's project-then-
//      main-worktree search, since a linked worktree checkout has no
//      vendored .bee/bin/bee of its own);
//   2. maps every OpenCode tool/event this plugin routes onto the JSON shape
//      the target bee hook already reads on stdin — sometimes a field-name
//      translation (write/edit/bash/read), sometimes a verbatim pass-through
//      (question/task) — see mapToolCall below and discovery.md's field-
//      shape tables; and
//   3. picks ONE of exactly two failure policies per surface, never a third:
//        BLOCKING (tool.execute.before only) — write-guard and model-guard.
//        bee's documented DENY verdict (exit code 2, reason on stderr)
//        becomes a thrown Error, the only documented block path for
//        `tool.execute.before` (no abort field exists on `output`).
//        ADVISORY (every other surface) — session-init/prompt-context via
//        chat.message, state-sync/session-close via `event`, tools-logger
//        via tool.execute.after. These NEVER throw: a missing binary, a
//        spawn error, a crash, or any exit code is swallowed and logged to
//        stderr for a human to notice, never surfaced as a session-ending
//        exception. This is bee's own posture too — hooks/mod.rs's
//        `emit_undecidable` already resolves bee's OWN could-not-decide
//        outcome to exit 0 (fail-open BY DESIGN); this file's advisory
//        wrapper additionally swallows failures ON THE OPENCODE SIDE (a
//        crashed spawn, a missing binary) that never even reach that native
//        fail-open path. A fail-open host swallows a fail-CLOSED throw right
//        back into an allow — that is the one failure mode the BLOCKING path
//        above must never have, and it is exactly why the two policies never
//        mix on the same call.
//
// codex-subagent-audit is a NAMED EXCLUSION — n/a — codex-specific (no
// OpenCode session ever carries Codex SubagentStart/SubagentStop evidence).
// chain-nudge is NOT wired in this file — plan.md's Approach section names
// `event: session.idle` as its surface, but it depends on subagent-dispatch
// identity (agent_name/subagent_type of a JUST-COMPLETED bee-managed
// worker) that a bare `session.idle` event does not carry on its own,
// unlike Claude's SubagentStop which fires scoped to the exact subagent
// that stopped. Wiring it correctly is E4/S4's job (worker dispatch
// parity), not this guard-parity cell's — named gap, see discovery.md.
//
// Full implemented hook -> OpenCode surface table, with failure policy and
// any payload-contract gap, is recorded in
// docs/history/opencode-support/discovery.md.

import type { Plugin } from "@opencode-ai/plugin"
import { execFileSync } from "node:child_process"
import { existsSync } from "node:fs"
import { randomUUID } from "node:crypto"
import path from "node:path"

const BINARY_NAMES = ["bee", "bee.exe"]

/** Every path this project's OpenCode session might find a bee binary at,
 * in priority order. Mirrors the shell fallback chain in
 * packages/bee/hooks/claude-hooks.json: project .bee/bin first, then (for a
 * linked worktree with no vendored binary of its own) the main worktree's
 * .bee/bin via `git rev-parse --git-common-dir`. */
function candidateBinaries(directory: string): string[] {
  const candidates = BINARY_NAMES.map((name) => path.join(directory, ".bee", "bin", name))
  try {
    const commonDir = execFileSync(
      "git",
      ["-C", directory, "rev-parse", "--path-format=absolute", "--git-common-dir"],
      { encoding: "utf8" },
    ).trim()
    if (commonDir) {
      // commonDir is "<main-worktree-root>/.git" for a linked worktree.
      const mainRoot = path.dirname(commonDir)
      for (const name of BINARY_NAMES) candidates.push(path.join(mainRoot, ".bee", "bin", name))
    }
  } catch {
    // not a git repo, or git unavailable — the direct-project candidates
    // above are all there is; resolveBeeBinary throws below if none exist.
  }
  return candidates
}

function resolveBeeBinary(directory: string): string {
  for (const candidate of candidateBinaries(directory)) {
    if (existsSync(candidate)) return candidate
  }
  throw new Error(
    "bee guard could not find the bee binary (.bee/bin/bee) in this project or its main worktree — " +
      "denying rather than letting a call through unchecked. " +
      "FIX: run `bee onboard --apply` (or vendor .bee/bin/bee) and retry.",
  )
}

// ─── blocking hooks (tool.execute.before): throw on deny, fail CLOSED ──────

/** Runs a BLOCKING bee hook and turns its verdict into either a normal
 * return (allow) or a thrown Error (deny, or any undecidable OpenCode-side
 * failure — missing binary, spawn error, crash, unexpected exit code). This
 * is the one function in this file allowed to throw; every advisory call
 * below goes through runAdvisoryHook instead, which never does.
 *
 * A verdict-carrying exit 0 can also carry a repair on stdout (write-guard's
 * AskUserQuestion header auto-fix; model-guard's dispatch-label/subagent_type
 * auto-fix — both emitted as `hookSpecificOutput.updatedInput` JSON). This
 * function does not parse or apply that repair back into `output.args` —
 * named gap, see discovery.md; the call is still let through unmodified,
 * exactly as an ordinary allow would be. */
function runBlockingHook(
  directory: string,
  hookName: "write-guard" | "model-guard",
  payload: Record<string, unknown>,
): void {
  const beeBinary = resolveBeeBinary(directory) // throws (fail-closed) if unresolved

  try {
    execFileSync(beeBinary, ["hook", hookName], {
      input: JSON.stringify(payload),
      encoding: "utf8",
      cwd: directory,
    })
    // exit 0 — allow. Nothing to mutate; output.args passes through as-is.
  } catch (err: any) {
    // execFileSync throws on any non-zero exit or a spawn failure.
    // Exit code 2 is bee's documented DENY verdict — surface its stderr
    // reason as the block (this is also how a `@@BEE_PRIVACY@@` secret-read
    // marker reaches the human: verbatim, inside this thrown message).
    // Anything else (spawn failure, signal kill, unexpected exit code, a
    // native panic inside bee) is undecidable, and undecidable is
    // fail-closed here too, never fail-open.
    if (err?.status === 2) {
      const reason = (err.stderr ?? "").toString().trim()
      throw new Error(reason || `bee ${hookName} denied this call.`)
    }
    const detail = (err?.stderr ?? err?.message ?? String(err)).toString().trim()
    throw new Error(
      `bee ${hookName} did not return a verdict (${detail || "no output"}) — ` +
        "denying rather than allowing an unchecked call.",
    )
  }
}

// ─── advisory hooks: swallow everything, never throw ────────────────────────

/** Runs an ADVISORY bee hook. Fail-open BY DESIGN, on both sides:
 * - bee's own native `hooks/mod.rs::emit_undecidable` already resolves an
 *   undecidable payload to exit 0 with a stderr diagnostic — never a
 *   throw-shaped exit from these hooks in the first place.
 * - this wrapper additionally swallows every OPENCODE-SIDE failure (missing
 *   binary, spawn error, non-zero exit, a native panic) by logging to
 *   `console.error` and returning null, so a broken bee install degrades
 *   the digest/logging/state-sync surfaces silently rather than aborting
 *   the session — the opposite failure direction from runBlockingHook
 *   above, and the two must never be swapped. */
function runAdvisoryHook(directory: string, hookName: string, payload: Record<string, unknown>): string | null {
  let beeBinary: string
  try {
    beeBinary = resolveBeeBinary(directory)
  } catch (err: any) {
    console.error(`bee ${hookName} (advisory): ${err?.message ?? err}`)
    return null
  }
  try {
    const stdout = execFileSync(beeBinary, ["hook", hookName], {
      input: JSON.stringify(payload),
      encoding: "utf8",
      cwd: directory,
    })
    const text = stdout.trim()
    return text.length > 0 ? text : null
  } catch (err: any) {
    const detail = (err?.stderr ?? err?.message ?? String(err)).toString().trim()
    console.error(`bee ${hookName} (advisory) did not complete cleanly: ${detail || "no output"}`)
    return null
  }
}

// ─── tool -> hook mapping (the only "rule" in this file) ────────────────────

type MappedCall = { hook: "write-guard" | "model-guard"; tool_name: string; tool_input: Record<string, unknown> }

/** Which OpenCode tools bee's blocking hooks care about, and the field-name
 * translation into the PreToolUse shape bee already reads
 * (packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs,
 * packages/bee-rs/crates/bee/src/hooks/model_guard.rs). Every actual
 * allow/deny decision still comes from the bee binary.
 *
 * This must cover every write-capable and read-shaped tool the installed
 * OpenCode binary registers, not just the ones a given session happens to
 * expose to a model — a registered tool that falls through to `default` is
 * a TypeScript-side allow, which is the one bypass this file exists to
 * close. See discovery.md's tool-registry and field-shape tables for how
 * this list was enumerated. */
function mapToolCall(tool: string, args: any): MappedCall | null {
  switch (tool) {
    case "write":
      return {
        hook: "write-guard",
        tool_name: "Write",
        tool_input: { file_path: args?.filePath, content: args?.content },
      }
    case "edit":
      return {
        hook: "write-guard",
        tool_name: "Edit",
        tool_input: { file_path: args?.filePath, old_string: args?.oldString, new_string: args?.newString },
      }
    case "bash":
      return { hook: "write-guard", tool_name: "Bash", tool_input: { command: args?.command } }
    case "apply_patch":
      // OpenCode's built-in apply_patch tool args shape is
      // `{ patchText: string }` (confirmed against the installed
      // opencode-ai@1.18.16 binary's own PatchSchema — see discovery.md).
      // bee's write-guard reads the patch body from tool_input.input,
      // .patch, or .command (detectors.rs apply_patch_text) — "patch" is
      // the field name that carries it here.
      return { hook: "write-guard", tool_name: "apply_patch", tool_input: { patch: args?.patchText } }
    case "read":
      // filePath -> file_path. offset/limit are forwarded ONLY when the
      // model actually supplied them: bee's own "unbounded read" size-denial
      // check (write_guard/main.rs ~121-124) only fires when tool_name ===
      // "Read" AND tool_input has NEITHER an "offset" NOR a "limit" key —
      // JSON.stringify drops an `undefined` property outright, so an
      // omitted OpenCode arg reads on bee's side as truly absent, not merely
      // falsy, preserving that presence/absence signal exactly.
      return {
        hook: "write-guard",
        tool_name: "Read",
        tool_input: { file_path: args?.filePath, offset: args?.offset, limit: args?.limit },
      }
    case "grep":
      // bee's read-guard resolves the target from tool_input.file_path OR
      // .path (write_guard/main.rs:110) — OpenCode's grep args already use
      // "path" for the same purpose, so no rename is needed for the one
      // field bee actually reads; pattern/include ride along unread but
      // harmless (kept for a future guard that might want them).
      return {
        hook: "write-guard",
        tool_name: "Grep",
        tool_input: { path: args?.path, pattern: args?.pattern, include: args?.include },
      }
    case "glob":
      return {
        hook: "write-guard",
        tool_name: "Glob",
        tool_input: { path: args?.path, pattern: args?.pattern },
      }
    case "question":
      // OpenCode's question args, `{ questions: [{ question, header,
      // options: [{ label, description }], multiple }] }`, already match
      // bee's AskUserQuestion tool_input shape field-for-field
      // (write_guard/detectors.rs check_ask_user_question reads exactly
      // these keys) — passed through untouched, no translation needed.
      return { hook: "write-guard", tool_name: "AskUserQuestion", tool_input: args ?? {} }
    case "task":
      // Same story: OpenCode's task args, `{ description, prompt,
      // subagent_type, task_id?, command?, background? }`, already match
      // the three fields model-guard's evaluate_claude_dispatch reads
      // (model_guard.rs:516-524 — model/tier-marker/subagent_type) —
      // passed through untouched. See discovery.md for the one real
      // payload-contract gap this mapping still carries: model-guard's
      // model-param equality check is hardcoded to `models.claude`
      // (model_guard.rs:570), not a `models.opencode` key that does not
      // exist yet (E4/S4).
      return { hook: "model-guard", tool_name: "Task", tool_input: args ?? {} }
    default:
      return null // not a guarded tool — no rule here, just routing
  }
}

// ─── advisory session state (process-lifetime only, never persisted) ───────

/** session-init only wants to run once per session (it registers/refreshes
 * the acting session and may adopt a handoff — running it on every message
 * would re-adopt a handoff already claimed). Tracked in-memory only: this
 * Set resets on every OpenCode server process restart, which just means the
 * next chat.message re-runs session-init once more — harmless, since
 * session-init's own native logic is itself idempotent (heartbeat-or-create,
 * ADOPT_SOURCES-gated adoption). */
const sessionInitRun = new Set<string>()

export default (async ({ directory }) => {
  return {
    // ── BLOCKING: write-guard (write/edit/bash/apply_patch/read/grep/glob/
    // question) + model-guard (task) — throw on deny, fail CLOSED. ─────────
    "tool.execute.before": async (input, output) => {
      const mapped = mapToolCall(input.tool, output.args)
      if (!mapped) return
      runBlockingHook(directory, mapped.hook, {
        hook_event_name: "PreToolUse",
        session_id: input.sessionID,
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_input: mapped.tool_input,
      })
    },

    // ── ADVISORY: session-init (once per session) + prompt-context (every
    // message), injected as a synthetic text part — the "preamble digest"
    // plan.md's Approach section names for this surface. Fail-open: never
    // throws, and an empty digest injects nothing. ─────────────────────────
    "chat.message": async (input, output) => {
      const sessionID = input.sessionID
      const digestParts: string[] = []

      if (!sessionInitRun.has(sessionID)) {
        sessionInitRun.add(sessionID)
        const initText = runAdvisoryHook(directory, "session-init", {
          hook_event_name: "SessionStart",
          session_id: sessionID,
          // OpenCode's chat.message carries no startup/resume/clear/compact
          // distinction (verified against @opencode-ai/plugin@1.18.16's
          // type declarations) — "startup" is the closest single value,
          // and it is one of the two ADOPT_SOURCES session-init checks
          // (session_init.rs:64), so handoff adoption still works on the
          // first message of a session.
          source: "startup",
          cwd: directory,
        })
        if (initText) digestParts.push(initText)
      }

      const promptText = runAdvisoryHook(directory, "prompt-context", {
        hook_event_name: "UserPromptSubmit",
        session_id: sessionID,
        cwd: directory,
      })
      if (promptText) digestParts.push(promptText)

      if (digestParts.length === 0) return

      output.parts.push({
        // OpenCode's own part-id schema requires a "prt"-prefixed string
        // (confirmed live: an unprefixed UUID here throws
        // `SchemaError: Expected a string starting with "prt"` inside
        // Session.updatePart and fails the whole chat.message call) — this
        // mirrors that prefix convention, not documented in the plugin
        // type declarations themselves.
        id: `prt_${randomUUID().replace(/-/g, "")}`,
        sessionID,
        messageID: input.messageID ?? output.message.id,
        type: "text",
        text: digestParts.join("\n\n"),
        synthetic: true,
      } as any)
    },

    // ── ADVISORY: state-sync (file.edited, session.idle) + session-close
    // (session.idle, session.deleted). Fail-open: never throws. ────────────
    "event": async (input) => {
      const event = input.event as { type?: string; properties?: Record<string, unknown> } | undefined
      const type = event?.type
      if (!type) return

      if (type === "file.edited") {
        // EventFileEdited.properties is `{ file: string }` only — verified
        // against @opencode-ai/sdk@1.18.16's type declarations, this event
        // carries NO sessionID at all. state-sync still runs: its
        // session-heartbeat-renewal half is skipped natively (no
        // `session_id` in the payload -> get_session_id returns None,
        // state_sync.rs:131-136), but the state-count rebuild half
        // (.bee/state.json refresh) still fires — named gap, see
        // discovery.md.
        runAdvisoryHook(directory, "state-sync", {
          hook_event_name: "PostToolUse",
          cwd: directory,
        })
        return
      }

      if (type === "session.idle") {
        const sessionID = (event?.properties as { sessionID?: string } | undefined)?.sessionID
        runAdvisoryHook(directory, "state-sync", {
          hook_event_name: "SubagentStop",
          session_id: sessionID,
          cwd: directory,
        })
        // session-close's native run_inner delegates wholesale on
        // ctx.event === "PreCompact" (session_close/mod.rs:107) — OpenCode
        // has no compaction-boundary analog wired in this cell, so this
        // call always uses "Stop", the fully-native path. Any
        // `{"decision":"block",...}` continuation-nudge session-close can
        // emit on Stop has no OpenCode enforcement equivalent on
        // session.idle (nothing here can force the session to keep going);
        // it is only logged, never enforced — see discovery.md.
        runAdvisoryHook(directory, "session-close", {
          hook_event_name: "Stop",
          session_id: sessionID,
          cwd: directory,
        })
        return
      }

      if (type === "session.deleted") {
        const sessionID = (event?.properties as { info?: { id?: string } } | undefined)?.info?.id
        runAdvisoryHook(directory, "session-close", {
          hook_event_name: "Stop",
          session_id: sessionID,
          cwd: directory,
        })
      }
    },

    // ── ADVISORY: tools-logger — observe-only, every tool call. Fail-open:
    // never throws. ──────────────────────────────────────────────────────
    "tool.execute.after": async (input) => {
      // Reuse mapToolCall's translation so bee's tools.jsonl records the
      // same tool_name the Claude/Codex belts would (e.g. "Write", "Task"),
      // falling back to OpenCode's own raw tool id for the tools this file
      // does not otherwise gate (webfetch, websearch, todowrite, skill,
      // invalid) — tools-logger has no matcher restriction in the Claude
      // belt (claude-hooks.json's PostToolUse entry with no "matcher" key),
      // so every tool call is logged here too.
      const mapped = mapToolCall(input.tool, input.args)
      runAdvisoryHook(directory, "tools-logger", {
        hook_event_name: "PostToolUse",
        session_id: input.sessionID,
        tool_name: mapped?.tool_name ?? input.tool,
        cwd: directory,
        // agent_id/agent_type/duration_ms/tool_status: not obtainable from
        // OpenCode's tool.execute.after payload (input: {tool, sessionID,
        // callID, args}; output: {title, output, metadata}) — omitted
        // rather than guessed. tools_logger.rs already treats all four as
        // optional (tools_logger.rs:20-35), so this degrades gracefully:
        // named gap, see discovery.md.
      })
    },
  }
}) satisfies Plugin
