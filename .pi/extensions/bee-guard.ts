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
// codex-subagent-audit and chain-nudge are NOT wired here either, for the same
// reason they are not wired on the OpenCode belt: codex-subagent-audit is
// Codex-specific, and chain-nudge needs subagent-dispatch identity that no Pi
// event carries.

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
import { execFileSync } from "node:child_process"
import { existsSync, statSync } from "node:fs"
import path from "node:path"

const BINARY_NAMES = ["bee", "bee.exe"]

// ─── store + binary discovery (re-run on EVERY call, never cached) ──────────

/** Every root this project's Pi session might find a bee store under, in
 * priority order: the project directory first, then (for a linked worktree
 * with no vendored store of its own) the main worktree root via
 * `git rev-parse --git-common-dir`. Mirrors the shell fallback chain in
 * packages/bee/hooks/claude-hooks.json. */
function candidateRoots(directory: string): string[] {
  const roots = [directory]
  try {
    const commonDir = execFileSync(
      "git",
      ["-C", directory, "rev-parse", "--path-format=absolute", "--git-common-dir"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] },
    ).trim()
    if (commonDir) {
      // commonDir is "<main-worktree-root>/.git" for a linked worktree.
      roots.push(path.dirname(commonDir))
    }
  } catch {
    // not a git repo, or git unavailable — the direct-project root above is
    // all there is.
  }
  return roots
}

function isDirectory(candidate: string): boolean {
  try {
    return statSync(candidate).isDirectory()
  } catch {
    return false
  }
}

/** The passivity check, per call. True when a `.bee` DIRECTORY exists at the
 * project root or at the main worktree root — the cheapest honest "is this a
 * bee repo" signal, and the one that flips the moment an in-session
 * `bee onboard` creates the store (no `/reload` needed). The binary is a
 * SEPARATE question, deliberately: directory present + binary missing is an
 * undecidable bee repo, not a bee-less one. */
function beeStorePresent(directory: string): boolean {
  return candidateRoots(directory).some((root) => isDirectory(path.join(root, ".bee")))
}

/** The first bee binary that exists across the same roots, or null. Mirrors
 * candidateRoots' priority order exactly. */
function resolveBeeBinary(directory: string): string | null {
  for (const root of candidateRoots(directory)) {
    for (const name of BINARY_NAMES) {
      const candidate = path.join(root, ".bee", "bin", name)
      if (existsSync(candidate)) return candidate
    }
  }
  return null
}

// ─── the blocking surface (tool_call): fail CLOSED ─────────────────────────

type Verdict = { block: true; reason: string } | undefined

function block(reason: string): Verdict {
  return { block: true, reason }
}

/** Runs a BLOCKING bee hook and turns its verdict into either a plain allow
 * (`undefined`, Pi's "no opinion" return) or Pi's documented block object.
 * Every undecidable outcome on this path blocks — D3, fail closed:
 *   - `.bee` present but no binary  -> block
 *   - exit 2 (bee's DENY verdict)   -> block, carrying bee's stderr reason
 *     verbatim (this is also how a `@@BEE_PRIVACY@@` secret-read marker
 *     reaches the human: untouched, inside this reason string)
 *   - spawn failure / crash / any other non-zero exit -> block
 *   - exit 0 with `permissionDecision: "ask"` -> block (Pi's tool_call return
 *     is two-valued — block or allow, no ask primitive — so treating "ask" as
 *     allow would silently drop write-guard's dominant enforcement path; its
 *     own comment at write_guard/main.rs:389-394 is explicit that the verdict
 *     there is "ask, never allow")
 *   - exit 0 with non-empty stdout that will not parse -> block
 * Empty stdout, and exit-0 JSON with no `hookSpecificOutput`, are ordinary
 * allows. */
function runBlockingHook(
  directory: string,
  hookName: "write-guard",
  payload: Record<string, unknown>,
  toolInput: Record<string, unknown>,
  passthrough: boolean,
): Verdict {
  const beeBinary = resolveBeeBinary(directory)
  if (!beeBinary) {
    return block(
      "bee guard could not find the bee binary (.bee/bin/bee) in this project or its main worktree, " +
        "but this repo has a .bee store — blocking rather than letting a call through unchecked. " +
        "FIX: run `bee onboard --apply` (or vendor .bee/bin/bee) and retry.",
    )
  }

  let stdout: string
  try {
    stdout = execFileSync(beeBinary, ["hook", hookName], {
      input: JSON.stringify(payload),
      encoding: "utf8",
      cwd: directory,
    })
  } catch (err: any) {
    if (err?.status === 2) {
      const reason = (err.stderr ?? "").toString().trim()
      return block(reason || `bee ${hookName} denied this call.`)
    }
    const detail = (err?.stderr ?? err?.message ?? String(err)).toString().trim()
    return block(
      `bee ${hookName} did not return a verdict (${detail || "no output"}) — ` +
        "blocking rather than allowing an unchecked call.",
    )
  }

  const text = stdout.trim()
  if (text.length === 0) return undefined // ordinary allow — no verdict JSON

  let parsed: any
  try {
    parsed = JSON.parse(text)
  } catch {
    return block(
      `bee ${hookName} returned an exit-0 verdict this extension could not parse (${text}) — ` +
        "blocking rather than allowing an unchecked call.",
    )
  }

  const hso = parsed?.hookSpecificOutput as Record<string, unknown> | undefined
  if (!hso) return undefined // exit-0 JSON with nothing to apply

  // Checked BEFORE any repair below: "ask" is bee's own never-allow verdict.
  if (hso.permissionDecision === "ask") {
    const reason =
      (hso.permissionDecisionReason as string | undefined) ||
      (hso.additionalContext as string | undefined) ||
      `bee ${hookName} requires confirmation for this call.`
    return block(`bee ${hookName}: ${reason}`)
  }

  // A repair verdict (`updatedInput`) is emitted in bee's OWN field-name space
  // — the space of a PASS-THROUGH mapping, never of a translated one. Pi does
  // document in-place mutation of `event.input` (docs/extensions.md
  // "tool_call": "Mutations to event.input affect the actual tool execution"),
  // so a repair CAN land here — but only where this file forwarded the input
  // verbatim. On a field-TRANSLATED tool the repair would land in the wrong
  // field names, and letting the call run unrepaired is exactly the silent
  // bypass this belt exists to close: undecidable, therefore blocked (D3).
  // No Pi built-in maps pass-through today (Pi has no AskUserQuestion and no
  // Task tool), so neither branch can fire on a built-in — both are here so a
  // future repair path is caught, never dropped.
  if (hso.updatedInput && typeof hso.updatedInput === "object" && !Array.isArray(hso.updatedInput)) {
    if (!passthrough) {
      return block(
        `bee ${hookName} returned a repair for a tool whose arguments this belt translates by ` +
          "field name, so the repair cannot be applied in Pi's own field space — " +
          "blocking rather than running the call unrepaired.",
      )
    }
    Object.assign(toolInput, hso.updatedInput as Record<string, unknown>)
  }

  // additionalContext (a repair note, or a bare reservation warning with
  // neither a repair nor an ask) must reach a human, not be dropped. Pi's
  // tool_call return carries no text-injection surface, so stderr is the
  // surface — same choice the OpenCode belt makes.
  if (typeof hso.additionalContext === "string" && hso.additionalContext.length > 0) {
    console.error(`bee ${hookName}: ${hso.additionalContext}`)
  }

  return undefined
}

// ─── the advisory surfaces: swallow everything, never throw ────────────────

/** Runs an ADVISORY bee hook. Fail-open BY DESIGN, on both sides: bee's own
 * `hooks/mod.rs::emit_undecidable` already resolves an undecidable payload to
 * exit 0, and this wrapper additionally swallows every PI-SIDE failure
 * (missing binary, spawn error, non-zero exit, a native panic) by logging to
 * `console.error` and returning null. A broken bee install degrades the
 * digest/state-sync surfaces silently rather than aborting the session — the
 * opposite failure direction from runBlockingHook above, and the two must
 * never be swapped. Passive (silent, no log) when the repo has no `.bee`
 * store at all. */
function runAdvisoryHook(
  directory: string,
  hookName: string,
  payload: Record<string, unknown>,
): string | null {
  if (!beeStorePresent(directory)) return null // not a bee repo — feel nothing
  const beeBinary = resolveBeeBinary(directory)
  if (!beeBinary) {
    console.error(
      `bee ${hookName} (advisory): .bee store present but no bee binary (.bee/bin/bee) — skipped. ` +
        "FIX: run `bee onboard --apply` (or vendor .bee/bin/bee).",
    )
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

// ─── tool -> hook mapping (the only "rule" in this file) ───────────────────

type MappedCall = {
  hook: "write-guard"
  tool_name: string
  tool_input: Record<string, unknown>
  /** true only when tool_input is the caller's own input forwarded verbatim —
   * the one shape a bee `updatedInput` repair can be applied onto. */
  passthrough: boolean
}

/** Pi 0.84.3's COMPLETE built-in tool registry, enumerated from the installed
 * binary rather than guessed. Two independent version-matched anchors agree on
 * the same eight names:
 *   - docs/settings.md `defaultTools`: "Available built-ins are `read`,
 *     `bash`, `powershell`, `edit`, `write`, `grep`, `find`, and `ls`";
 *   - docs/extensions.md "Overriding Built-in Tools": the same eight;
 * and each argument shape below was read off the binary's own typebox schemas
 * (`bashSchema` {command,timeout}, `readSchema` {path,offset,limit},
 * `writeSchema` {path,content}, `editSchema` {path,edits[]}, `grepSchema`
 * {pattern,path,glob,ignoreCase,literal,context,limit}, `findSchema`
 * {pattern,path,limit}, `lsSchema` {path,limit}; `powershell` shares the shell
 * tool's schema with `bash`).
 *
 * Kept as a NAMED LIST, not a switch default, because the fail-safe below
 * depends on knowing exactly which names are enumerated. */
const PI_BUILTIN_TOOLS = [
  "bash",
  "powershell",
  "read",
  "write",
  "edit",
  "grep",
  "find",
  "ls",
] as const

/** Field names a custom tool might carry a write target under, in probe
 * order — used ONLY by the fail-safe route below. */
const PATH_FIELDS = [
  "file_path",
  "filePath",
  "path",
  "file",
  "target",
  "destination",
  "dest",
  "output",
  "outputPath",
]

function firstString(input: any, keys: string[]): string | undefined {
  if (!input || typeof input !== "object") return undefined
  for (const key of keys) {
    const value = input[key]
    if (typeof value === "string" && value.length > 0) return value
  }
  return undefined
}

/** Which Pi tool bee's blocking hook sees as what, and the field-name
 * translation into the PreToolUse shape bee already reads
 * (packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs).
 *
 * This function NEVER returns null. A name outside PI_BUILTIN_TOOLS — a custom
 * `pi.registerTool` tool from a sibling extension, a tool added by a future Pi
 * release, an override of a built-in — is routed to write-guard as a
 * WRITE-CAPABLE call, because a tool this file does not recognise is a
 * TypeScript-side allow otherwise, and that is the one bypass this file exists
 * to close. bee still owns the verdict; the fail-safe only decides which
 * SHAPE the unknown call is presented in, never whether it passes.
 *
 * bee's write-capable set is `Edit | Write | MultiEdit | Bash | apply_patch`
 * (write_guard/main.rs:64-67), so the fail-safe picks between the two shapes
 * that carry a target: a `command` string routes as Bash, everything else
 * routes as Write with the first path-shaped field found, and the raw input
 * rides along untouched so no field is hidden from a future detector. */
function mapToolCall(tool: string, input: any): MappedCall {
  const args = (input && typeof input === "object" ? input : {}) as Record<string, unknown>

  switch (tool) {
    case "bash":
    case "powershell":
      // Both shell tools share `{ command, timeout }`; bee reads `command`.
      return {
        hook: "write-guard",
        tool_name: "Bash",
        tool_input: { command: args.command },
        passthrough: false,
      }

    case "write":
      return {
        hook: "write-guard",
        tool_name: "Write",
        tool_input: { file_path: args.path, content: args.content },
        passthrough: false,
      }

    case "edit":
      // Pi's `edit` takes an ARRAY of replacements (`edits: [{oldText,
      // newText}]`), which is Claude's MultiEdit shape, not its single-edit
      // Edit shape — MultiEdit is the honest name here, and bee treats all
      // three write-tool names identically (write_guard/main.rs:64, reading
      // only `file_path`). The entries are translated into bee's own field
      // names so a future detector reading them finds the shape it expects.
      return {
        hook: "write-guard",
        tool_name: "MultiEdit",
        tool_input: {
          file_path: args.path,
          edits: Array.isArray(args.edits)
            ? args.edits.map((edit: any) => ({
                old_string: edit?.oldText,
                new_string: edit?.newText,
              }))
            : undefined,
        },
        passthrough: false,
      }

    case "read":
      // path -> file_path. offset/limit are forwarded ONLY when Pi actually
      // supplied them: bee's "unbounded read" size-denial check
      // (write_guard/main.rs:121-124) fires only when tool_name === "Read" AND
      // tool_input has NEITHER an "offset" NOR a "limit" key. JSON.stringify
      // drops an `undefined` property outright, so an omitted Pi argument
      // reads on bee's side as truly absent, not merely falsy — the
      // presence/absence signal survives the translation exactly.
      return {
        hook: "write-guard",
        tool_name: "Read",
        tool_input: { file_path: args.path, offset: args.offset, limit: args.limit },
        passthrough: false,
      }

    case "grep":
      // bee's read-guard resolves the target from tool_input.file_path OR
      // .path (write_guard/main.rs:110) — Pi's grep already uses "path" for
      // the same purpose, so no rename is needed for the one field bee reads.
      // Pi's `glob` filter is Claude Grep's `include`; both ride along unread
      // but harmless.
      return {
        hook: "write-guard",
        tool_name: "Grep",
        tool_input: { path: args.path, pattern: args.pattern, include: args.glob },
        passthrough: false,
      }

    case "find":
      return {
        hook: "write-guard",
        tool_name: "Glob",
        tool_input: { path: args.path, pattern: args.pattern },
        passthrough: false,
      }

    case "ls":
      return {
        hook: "write-guard",
        tool_name: "Glob",
        tool_input: { path: args.path },
        passthrough: false,
      }

    default: {
      // FAIL-SAFE. Never a silent allow: bee decides, on the write-capable
      // shape that best fits the unknown arguments.
      const command = firstString(args, ["command"])
      if (command !== undefined) {
        return {
          hook: "write-guard",
          tool_name: "Bash",
          tool_input: { command },
          passthrough: false,
        }
      }
      return {
        hook: "write-guard",
        tool_name: "Write",
        tool_input: { ...args, file_path: firstString(args, PATH_FIELDS) ?? "" },
        passthrough: false,
      }
    }
  }
}

// ─── advisory session state (process-lifetime only, never persisted) ───────

// D8 (CONTEXT.md): the full session preamble is fetched ONCE per session_start
// and injected ONCE, on the first turn of the session; every turn after that
// carries only `bee hook prompt-context`'s own per-turn delta. `/reload` fires
// a second session_start with reason "reload" against the SAME session — it
// must not re-run session-init (which registers the acting session and may
// adopt a handoff already claimed), so a reload keeps whatever this instance
// already fetched. A genuinely new session (`new`/`resume`/`fork`) resets the
// pair, because it IS a new session.
let cachedPreamble: string | null = null
let preambleInjected = false
let sessionInitRun = false

/** Pi's session_start reasons mapped onto the SessionStart `source` values
 * bee's session-init reads. Only "startup" and "clear" are in bee's
 * ADOPT_SOURCES (session_init.rs:64), so only a genuinely fresh session
 * boundary can adopt a handoff — a resume, a fork, and a reload never do,
 * which is exactly AGENTS.md's rule ("a resumed or compacted session never
 * adopts"). */
function sessionSource(reason: string | undefined): string {
  switch (reason) {
    case "new":
      return "clear"
    case "resume":
    case "fork":
    case "reload":
      return "resume"
    default:
      return "startup"
  }
}

function sessionIdOf(ctx: any): string | undefined {
  try {
    const id = ctx?.sessionManager?.getSessionId?.()
    return typeof id === "string" && id.length > 0 ? id : undefined
  } catch {
    return undefined
  }
}

function directoryOf(ctx: any): string {
  const cwd = ctx?.cwd
  return typeof cwd === "string" && cwd.length > 0 ? cwd : process.cwd()
}

// ─── the belt ──────────────────────────────────────────────────────────────

export default function (pi: ExtensionAPI) {
  // ── BLOCKING: write-guard on every tool call. Fail CLOSED. ───────────────
  pi.on("tool_call", (async (event: any, ctx: any) => {
    const directory = directoryOf(ctx)
    // Passivity, per call: no .bee store anywhere -> this is not a bee repo,
    // and a Pi session here must feel nothing at all.
    if (!beeStorePresent(directory)) return undefined

    const mapped = mapToolCall(String(event?.toolName ?? ""), event?.input)
    return runBlockingHook(
      directory,
      mapped.hook,
      {
        hook_event_name: "PreToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_input: mapped.tool_input,
      },
      // The repair target is Pi's own mutable `event.input` — only ever
      // written for a pass-through mapping (see runBlockingHook).
      (event?.input ?? {}) as Record<string, unknown>,
      mapped.passthrough,
    )
  }) as any)

  // ── ADVISORY: session-init, ONCE per session, cached (D8). ───────────────
  pi.on("session_start", (async (event: any, ctx: any) => {
    try {
      const reason = event?.reason as string | undefined
      if (reason === "reload" && sessionInitRun) return // /reload is idempotent
      if (reason !== "reload") {
        cachedPreamble = null
        preambleInjected = false
        sessionInitRun = false
      }
      const directory = directoryOf(ctx)
      const text = runAdvisoryHook(directory, "session-init", {
        hook_event_name: "SessionStart",
        session_id: sessionIdOf(ctx),
        source: sessionSource(reason),
        cwd: directory,
      })
      sessionInitRun = true
      if (text) cachedPreamble = text
    } catch (err: any) {
      console.error(`bee session-init (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: the per-turn context feed (D8). The cached preamble rides
  // the FIRST turn only; `bee hook prompt-context` runs unchanged on every
  // turn and is the per-turn delta. Never throws, never blocks a turn. ──────
  pi.on("before_agent_start", (async (event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      const parts: string[] = []

      if (cachedPreamble && !preambleInjected) {
        preambleInjected = true
        parts.push(cachedPreamble)
      }

      const delta = runAdvisoryHook(directory, "prompt-context", {
        hook_event_name: "UserPromptSubmit",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        prompt: typeof event?.prompt === "string" ? event.prompt : "",
      })
      if (delta) parts.push(delta)

      if (parts.length === 0) return undefined
      const base = typeof event?.systemPrompt === "string" ? event.systemPrompt : ""
      return { systemPrompt: `${base}\n\n${parts.join("\n\n")}` }
    } catch (err: any) {
      console.error(`bee prompt-context (advisory): ${err?.message ?? err}`)
      return undefined
    }
  }) as any)

  // ── ADVISORY: state-sync after every tool result. Returns nothing, so the
  // result itself is never modified. ───────────────────────────────────────
  pi.on("tool_result", (async (event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      runAdvisoryHook(directory, "state-sync", {
        hook_event_name: "PostToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapToolCall(String(event?.toolName ?? ""), event?.input).tool_name,
      })
    } catch (err: any) {
      console.error(`bee state-sync (advisory): ${err?.message ?? err}`)
    }
    return undefined
  }) as any)

  // ── ADVISORY: the turn-end waiting mark. `agent_settled` is Pi's own
  // "nothing will continue automatically" signal (docs/extensions.md:569) —
  // the Stop analog, and session-close's Stop path is what sets the
  // `turn-end` waiting mark (session_close/mod.rs:260-309). Any continuation
  // nudge session-close can emit on Stop has no Pi enforcement equivalent
  // here (nothing on this event can force the session to keep going); it is
  // logged, never enforced. ────────────────────────────────────────────────
  pi.on("agent_settled", (async (_event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      runAdvisoryHook(directory, "session-close", {
        hook_event_name: "Stop",
        session_id: sessionIdOf(ctx),
        cwd: directory,
      })
    } catch (err: any) {
      console.error(`bee session-close (advisory): ${err?.message ?? err}`)
    }
  }) as any)
}

// Exported for the belt parity/contract suite (pi_plugin_contracts.rs), which
// derives this belt's rows from this source rather than a hand list.
export { PI_BUILTIN_TOOLS, mapToolCall }
