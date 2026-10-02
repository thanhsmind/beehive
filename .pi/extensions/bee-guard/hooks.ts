import { execFileSync } from "node:child_process"
import { beeStorePresent, resolveBeeBinary } from "./locate.ts"

export type Verdict = { block: true; reason: string } | undefined

export function block(reason: string): Verdict {
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
export function runBlockingHook(
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
export function runAdvisoryHook(
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

