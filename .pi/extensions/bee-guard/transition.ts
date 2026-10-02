import { existsSync, readdirSync, readFileSync, realpathSync, rmSync, statSync } from "node:fs"
import path from "node:path"
import { isDirectory, mainCheckoutRoot } from "./locate.ts"
import { state } from "./state.ts"
import { derivePostExitTimeoutMs, execBeeCli } from "./bee-cli.ts"
import { directoryOf, sessionIdOf } from "./session.ts"
import { carriedInboxTokens, recordCarry, saveRelocationCarry } from "./result-inbox.ts"

export interface SessionTransitionContinuation {
  operation: "merge-worktree"
  noCleanup: boolean
  skipUat: boolean
  queueWaitMs: number | null
}

export interface SessionTransitionIntent {
  schemaVersion: 1
  operation: "enter-worktree" | "exit-worktree-before-merge" | "exit-worktree"
  sourceCwd: string
  targetCwd: string
  worktreeId: string
  feature: string | null
  piSessionId: string | null
  continuation: SessionTransitionContinuation | null
}

export interface PendingRelocation {
  transition: SessionTransitionIntent
  sessionId: string
  sourceCwd: string
}

export const pendingTransitions = new Map<string, PendingRelocation>()
export const pendingRelocationTokens = new Map<string, string>()

export function canonicalizePath(p: string): string {
  if (!p || typeof p !== "string") {
    throw new Error("Path must be a non-empty string")
  }
  return realpathSync(p)
}

export function tokenizeArgv(raw: string): string[] {
  if (raw.includes("\0")) {
    throw new Error("Command argument contains illegal NUL byte")
  }
  const tokens: string[] = []
  let current = ""
  let inSingle = false
  let inDouble = false
  let escaped = false

  for (let i = 0; i < raw.length; i++) {
    const ch = raw[i]
    if (escaped) {
      current += ch
      escaped = false
      continue
    }
    if (ch === "\\") {
      if (inSingle) {
        current += ch
      } else {
        escaped = true
      }
      continue
    }
    if (ch === "'" && !inDouble) {
      inSingle = !inSingle
      continue
    }
    if (ch === '"' && !inSingle) {
      inDouble = !inDouble
      continue
    }
    if (/\s/.test(ch) && !inSingle && !inDouble) {
      if (current.length > 0) {
        tokens.push(current)
        current = ""
      }
      continue
    }
    current += ch
  }
  if (escaped || inSingle || inDouble) {
    throw new Error("Command argument contains unclosed quote or dangling escape")
  }
  if (current.length > 0) {
    tokens.push(current)
  }
  return tokens
}


export const MARKER_REGEX = /@@BEE_SESSION_TRANSITION@@\s+([^\r\n]+)\r?\n?/g

export const EXPECTED_TRANSITION_KEYS = [
  "continuation",
  "feature",
  "operation",
  "piSessionId",
  "schemaVersion",
  "sourceCwd",
  "targetCwd",
  "worktreeId",
]

export const EXPECTED_CONTINUATION_KEYS = [
  "noCleanup",
  "operation",
  "queueWaitMs",
  "skipUat",
]

export function hasExactKeys(obj: Record<string, unknown>, expectedSortedKeys: string[]): boolean {
  const keys = Object.keys(obj).sort()
  if (keys.length !== expectedSortedKeys.length) return false
  for (let i = 0; i < keys.length; i++) {
    if (keys[i] !== expectedSortedKeys[i]) return false
  }
  return true
}

export function validateTransitionIntent(
  rawJson: string,
  ctx: any,
): SessionTransitionIntent | null {
  let parsed: any
  try {
    parsed = JSON.parse(rawJson)
  } catch {
    return null
  }

  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return null
  }
  if (!hasExactKeys(parsed, EXPECTED_TRANSITION_KEYS)) {
    return null
  }
  if (parsed.schemaVersion !== 1) {
    return null
  }
  if (
    parsed.operation !== "enter-worktree" &&
    parsed.operation !== "exit-worktree-before-merge" &&
    parsed.operation !== "exit-worktree"
  ) {
    return null
  }
  if (typeof parsed.sourceCwd !== "string" || typeof parsed.targetCwd !== "string") {
    return null
  }
  if (!path.isAbsolute(parsed.sourceCwd) || !path.isAbsolute(parsed.targetCwd)) {
    return null
  }

  if (typeof parsed.worktreeId !== "string" || parsed.worktreeId.trim().length === 0) {
    return null
  }

  if (parsed.feature !== null && (typeof parsed.feature !== "string" || parsed.feature.trim().length === 0)) {
    return null
  }

  const activeSessionId = sessionIdOf(ctx)
  if (!activeSessionId || typeof parsed.piSessionId !== "string" || parsed.piSessionId !== activeSessionId) {
    return null
  }

  if (parsed.operation === "enter-worktree" || parsed.operation === "exit-worktree") {
    if (parsed.continuation !== null) {
      return null
    }
  } else if (parsed.operation === "exit-worktree-before-merge") {
    const cont = parsed.continuation
    if (!cont || typeof cont !== "object" || Array.isArray(cont)) {
      return null
    }
    if (!hasExactKeys(cont, EXPECTED_CONTINUATION_KEYS)) {
      return null
    }
    if (cont.operation !== "merge-worktree") {
      return null
    }
    if (typeof cont.noCleanup !== "boolean") {
      return null
    }
    if (typeof cont.skipUat !== "boolean") {
      return null
    }
    if (
      cont.queueWaitMs !== null &&
      (typeof cont.queueWaitMs !== "number" || !Number.isFinite(cont.queueWaitMs) || cont.queueWaitMs < 0)
    ) {
      return null
    }
  }

  let canonSource: string
  let canonTarget: string
  let canonCtxCwd: string
  try {
    canonSource = canonicalizePath(parsed.sourceCwd)
    canonTarget = canonicalizePath(parsed.targetCwd)
    canonCtxCwd = canonicalizePath(ctx.cwd)
  } catch {
    return null
  }

  if (parsed.sourceCwd !== canonSource || parsed.targetCwd !== canonTarget) {
    return null
  }

  if (canonSource !== canonCtxCwd) {
    return null
  }
  if (canonTarget === canonSource) {
    return null
  }
  if (!existsSync(parsed.targetCwd) || !isDirectory(parsed.targetCwd)) {
    return null
  }

  return parsed as SessionTransitionIntent
}

export function processTextForMarkers(
  text: string,
  ctx: any,
): { newText: string; validIntent: SessionTransitionIntent | null; modified: boolean } {
  if (!text.includes("@@BEE_SESSION_TRANSITION@@")) {
    return { newText: text, validIntent: null, modified: false }
  }

  let validIntent: SessionTransitionIntent | null = null
  let modified = false

  const newText = text.replace(MARKER_REGEX, (match, jsonStr) => {
    const validated = validateTransitionIntent(jsonStr.trim(), ctx)
    if (validated && !validIntent) {
      validIntent = validated
      modified = true
      return ""
    }
    return match
  })

  return { newText, validIntent, modified }
}

export function extractAndCaptureTransitionMarker(
  event: any,
  ctx: any,
): { content: any } | undefined {
  let anyModified = false
  let capturedIntent: SessionTransitionIntent | null = null

  if (Array.isArray(event?.content)) {
    for (const block of event.content) {
      if (block && typeof block.text === "string") {
        const { newText, validIntent, modified } = processTextForMarkers(block.text, ctx)
        if (modified) {
          block.text = newText
          anyModified = true
          if (validIntent && !capturedIntent) capturedIntent = validIntent
        }
      }
    }
  } else if (typeof event?.content === "string") {
    const { newText, validIntent, modified } = processTextForMarkers(event.content, ctx)
    if (modified) {
      event.content = newText
      anyModified = true
      if (validIntent && !capturedIntent) capturedIntent = validIntent
    }
  }

  if (typeof event?.text === "string") {
    const { newText, validIntent, modified } = processTextForMarkers(event.text, ctx)
    if (modified) {
      event.text = newText
      anyModified = true
      if (validIntent && !capturedIntent) capturedIntent = validIntent
    }
  }

  if (typeof event?.output === "string") {
    const { newText, validIntent, modified } = processTextForMarkers(event.output, ctx)
    if (modified) {
      event.output = newText
      anyModified = true
      if (validIntent && !capturedIntent) capturedIntent = validIntent
    }
  }

  if (!capturedIntent) {
    return undefined
  }

  const activeSessionId = sessionIdOf(ctx)
  if (!activeSessionId) return undefined

  const token = `reloc-${Date.now()}-${Math.random().toString(36).slice(2)}`
  const directory = directoryOf(ctx)
  pendingTransitions.set(token, {
    transition: capturedIntent,
    sessionId: activeSessionId,
    sourceCwd: directory,
  })
  pendingRelocationTokens.set(activeSessionId, token)

  const patchContent = Array.isArray(event?.content)
    ? event.content
    : typeof event?.content === "string"
      ? [{ type: "text", text: event.content }]
      : typeof event?.text === "string"
        ? [{ type: "text", text: event.text }]
        : typeof event?.output === "string"
          ? [{ type: "text", text: event.output }]
          : event?.content

  return { content: patchContent }
}

export async function revalidateDeferredIntent(
  ctx: any,
  intent: SessionTransitionIntent,
): Promise<SessionTransitionIntent | null> {
  if (!intent || typeof intent !== "object") return null
  if (intent.schemaVersion !== 1) return null

  const directory = directoryOf(ctx)
  let args: string[] = []

  if (intent.operation === "enter-worktree") {
    if (!intent.worktreeId) return null
    args = ["worktree", "enter", "--id", intent.worktreeId, "--json"]
  } else if (intent.operation === "exit-worktree") {
    if (!intent.worktreeId) return null
    args = ["worktree", "exit", "--id", intent.worktreeId, "--json"]
  } else if (intent.operation === "exit-worktree-before-merge") {
    args = ["worktree", "merge", "--json"]
    if (intent.worktreeId) {
      args.push("--id", intent.worktreeId)
    }
    if (intent.continuation?.noCleanup) {
      args.push("--no-cleanup")
    }
    if (intent.continuation?.skipUat) {
      args.push("--skip-uat")
    }
    if (intent.continuation?.queueWaitMs != null) {
      args.push("--queue-wait-ms", String(intent.continuation.queueWaitMs))
    }
  } else {
    return null
  }

  const result = await execBeeCli(directory, args, sessionIdOf(ctx))
  if (result.exitCode !== 0) {
    return null
  }

  let parsed: any
  try {
    parsed = JSON.parse(result.stdout)
  } catch {
    return null
  }

  const verified = parsed?.sessionTransition
  if (!verified || typeof verified !== "object") return null

  const validatedVerified = validateTransitionIntent(JSON.stringify(verified), ctx)
  if (!validatedVerified) return null

  if (validatedVerified.schemaVersion !== intent.schemaVersion) return null
  if (validatedVerified.operation !== intent.operation) return null

  if (
    typeof validatedVerified.worktreeId !== "string" ||
    typeof intent.worktreeId !== "string" ||
    validatedVerified.worktreeId.trim().length === 0 ||
    validatedVerified.worktreeId !== intent.worktreeId
  ) {
    return null
  }

  if ((validatedVerified.feature ?? null) !== (intent.feature ?? null)) return null
  if ((validatedVerified.piSessionId ?? null) !== (intent.piSessionId ?? null)) return null

  let canonVerifiedSource: string, canonIntentSource: string, canonCtxCwd: string
  let canonVerifiedTarget: string, canonIntentTarget: string
  try {
    canonVerifiedSource = canonicalizePath(validatedVerified.sourceCwd)
    canonIntentSource = canonicalizePath(intent.sourceCwd)
    canonVerifiedTarget = canonicalizePath(validatedVerified.targetCwd)
    canonIntentTarget = canonicalizePath(intent.targetCwd)
    canonCtxCwd = canonicalizePath(ctx.cwd)
  } catch {
    return null
  }
  if (canonVerifiedSource !== canonIntentSource) return null
  if (canonVerifiedSource !== canonCtxCwd) return null
  if (canonVerifiedTarget !== canonIntentTarget) return null

  if (intent.operation === "exit-worktree-before-merge") {
    const vCont = validatedVerified.continuation
    const iCont = intent.continuation
    if (!vCont || !iCont) return null
    if (vCont.operation !== iCont.operation) return null
    if (vCont.noCleanup !== iCont.noCleanup) return null
    if (vCont.skipUat !== iCont.skipUat) return null
    if (vCont.queueWaitMs !== iCont.queueWaitMs) return null
  } else {
    if (validatedVerified.continuation !== null || intent.continuation !== null) return null
  }

  return validatedVerified
}

export async function handlePostExitMerge(replacedCtx: any, intent: SessionTransitionIntent): Promise<void> {
  const continuation = intent.continuation
  const mergeArgs = ["worktree", "merge"]
  if (intent.worktreeId) {
    mergeArgs.push("--id", intent.worktreeId)
  }
  if (continuation?.noCleanup) {
    mergeArgs.push("--no-cleanup")
  }
  if (continuation?.skipUat) {
    mergeArgs.push("--skip-uat")
  }
  if (continuation?.queueWaitMs != null) {
    mergeArgs.push("--queue-wait-ms", String(continuation.queueWaitMs))
  }

  const directory = replacedCtx.cwd || intent.targetCwd
  const timeoutMs = derivePostExitTimeoutMs(continuation?.queueWaitMs)
  const result = await execBeeCli(directory, mergeArgs, sessionIdOf(replacedCtx), timeoutMs)
  if (result.exitCode === 0) {
    const out = result.stdout.trim() || result.stderr.trim()
    replacedCtx.ui?.notify?.(`Merge succeeded: ${out || `Merged worktree ${intent.worktreeId || ""}`}`, "info")
  } else {
    const err = result.stderr.trim() || result.stdout.trim()
    const worktreeId = intent.worktreeId || ""
    const reentryCmd = `/bee-worktree-enter --id ${worktreeId}`
    replacedCtx.ui?.notify?.(
      `Merge refused: ${err}\nTo return to the worktree, run: ${reentryCmd}`,
      "error",
    )
  }
}

export async function performSessionTransition(ctx: any, intent: SessionTransitionIntent): Promise<boolean> {
  if (state.activeTransition) {
    ctx.ui?.notify?.("Session transition refused: another transition is in progress", "warning")
    return false
  }
  state.activeTransition = true
  state.transitionTeardownOccurred = false

  let forkedSessionFile: string | null = null
  try {
    if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
      ctx.ui?.notify?.("Session transition refused: agent turn is currently active", "error")
      return false
    }

    const currentSessionId = sessionIdOf(ctx)
    if (
      !currentSessionId ||
      typeof intent?.piSessionId !== "string" ||
      intent.piSessionId !== currentSessionId
    ) {
      ctx.ui?.notify?.(
        "Session transition refused: current session ID does not match intent session ID",
        "error",
      )
      return false
    }

    if (!intent || typeof intent !== "object") {
      ctx.ui?.notify?.("Session transition refused: intent is missing or invalid", "error")
      return false
    }
    if (intent.schemaVersion !== 1) {
      ctx.ui?.notify?.("Session transition refused: unsupported schemaVersion", "error")
      return false
    }
    if (
      intent.operation !== "enter-worktree" &&
      intent.operation !== "exit-worktree-before-merge" &&
      intent.operation !== "exit-worktree"
    ) {
      ctx.ui?.notify?.("Session transition refused: unknown operation", "error")
      return false
    }
    if (typeof intent.sourceCwd !== "string" || typeof intent.targetCwd !== "string") {
      ctx.ui?.notify?.("Session transition refused: sourceCwd or targetCwd missing", "error")
      return false
    }
    if (!path.isAbsolute(intent.sourceCwd) || !path.isAbsolute(intent.targetCwd)) {
      ctx.ui?.notify?.("Session transition refused: sourceCwd and targetCwd must be absolute", "error")
      return false
    }

    let canonSource: string
    let canonTarget: string
    let canonCtxCwd: string
    try {
      canonSource = canonicalizePath(intent.sourceCwd)
      canonTarget = canonicalizePath(intent.targetCwd)
      canonCtxCwd = canonicalizePath(ctx.cwd)
    } catch (err: any) {
      ctx.ui?.notify?.(
        `Session transition refused: path canonicalization failed (${err?.message ?? err})`,
        "error",
      )
      return false
    }

    if (intent.sourceCwd !== canonSource || intent.targetCwd !== canonTarget) {
      ctx.ui?.notify?.("Session transition refused: sourceCwd or targetCwd is not canonical", "error")
      return false
    }

    if (canonSource !== canonCtxCwd) {
      ctx.ui?.notify?.(`Session transition refused: sourceCwd (${canonSource}) does not match current directory (${canonCtxCwd})`, "error")
      return false
    }

    if (canonTarget === canonSource) {
      ctx.ui?.notify?.("Session transition refused: target directory is identical to source directory", "error")
      return false
    }

    if (!existsSync(intent.targetCwd) || !isDirectory(intent.targetCwd)) {
      ctx.ui?.notify?.(`Session transition refused: target directory does not exist: ${intent.targetCwd}`, "error")
      return false
    }

    const validatedIntent = validateTransitionIntent(JSON.stringify(intent), ctx)
    if (!validatedIntent) {
      ctx.ui?.notify?.("Session transition refused: intent is missing or invalid", "error")
      return false
    }

    const sm = ctx?.sessionManager
    if (!sm) {
      ctx.ui?.notify?.("Session transition refused: no sessionManager available", "error")
      return false
    }
    if (typeof sm.isPersisted === "function" && !sm.isPersisted()) {
      ctx.ui?.notify?.("Session transition refused: current session is in-memory and cannot be forked", "error")
      return false
    }
    const sourceSessionFile =
      typeof sm.getSessionFile === "function" ? sm.getSessionFile() : sm.sessionFile
    if (!sourceSessionFile || typeof sourceSessionFile !== "string" || !existsSync(sourceSessionFile)) {
      ctx.ui?.notify?.("Session transition refused: source session file does not exist on disk", "error")
      return false
    }
    try {
      if (statSync(sourceSessionFile).size === 0) {
        ctx.ui?.notify?.("Session transition refused: source session file is empty", "error")
        return false
      }
    } catch {
      ctx.ui?.notify?.("Session transition refused: unable to inspect source session file", "error")
      return false
    }

    const SessionManagerConstructor = sm.constructor
    if (!SessionManagerConstructor || typeof SessionManagerConstructor.forkFrom !== "function") {
      ctx.ui?.notify?.("Session transition refused: SessionManager.forkFrom is not available", "error")
      return false
    }

    let forkedManager: any
    try {
      forkedManager = await SessionManagerConstructor.forkFrom(sourceSessionFile, intent.targetCwd)
    } catch (forkErr: any) {
      ctx.ui?.notify?.(`Session transition failed during fork: ${forkErr?.message ?? forkErr}`, "error")
      return false
    }

    forkedSessionFile =
      typeof forkedManager?.getSessionFile === "function"
        ? forkedManager.getSessionFile()
        : forkedManager?.sessionFile
    if (!forkedSessionFile || !existsSync(forkedSessionFile)) {
      ctx.ui?.notify?.("Session transition failed: forked session file was not created", "error")
      return false
    }

    let switchResult: any
    let reboundSessionFrom: string | null = null
    let reboundSessionTo: string | null = null
    let carryRecordedNewSession: string | null = null
    const mainRoot = mainCheckoutRoot(intent.sourceCwd)

    try {
      switchResult = await ctx.switchSession(forkedSessionFile, {
        withSession: async (replacedCtx: any) => {
          let newSessionId = sessionIdOf(replacedCtx)
          if (!newSessionId && forkedSessionFile) {
            try {
              const firstLine = readFileSync(forkedSessionFile, "utf8").split("\n")[0]
              const parsed = JSON.parse(firstLine)
              if (typeof parsed?.id === "string") newSessionId = parsed.id
            } catch {}
          }

          let carriedJobsCount = 0
          let reboundClaimsCount = 0

          if (newSessionId) {
            const carriedTokens = recordCarry(mainRoot, currentSessionId, newSessionId)
            carryRecordedNewSession = newSessionId
            for (const tok of carriedTokens) {
              const tokDir = path.join(mainRoot, ".bee", "result-inbox", tok)
              if (isDirectory(tokDir)) {
                try {
                  const names = readdirSync(tokDir)
                  carriedJobsCount += names.filter((n) => n.endsWith(".json")).length
                } catch {}
              }
            }

            // The main checkout, never `intent.targetCwd`: `cells rebind-session`
            // reads the shared control plane and REFUSES inside a granted feature
            // worktree, so a worktree cwd turns every rebind into a warning and a
            // no-op (reproduced live, 2026-09-16).
            const rebindResult = await execBeeCli(
              mainRoot,
              ["cells", "rebind-session", "--from", currentSessionId, "--to", newSessionId, "--json"],
              newSessionId,
            )
            if (rebindResult.exitCode === 0) {
              reboundSessionFrom = currentSessionId
              reboundSessionTo = newSessionId
              try {
                const parsed = JSON.parse(rebindResult.stdout)
                if (Array.isArray(parsed?.rebound)) {
                  reboundClaimsCount = parsed.rebound.length
                }
              } catch {}
            } else {
              const errDetail =
                rebindResult.stderr.trim() || rebindResult.stdout.trim() || "unknown error"
              replacedCtx.ui?.notify?.(
                `Warning: Failed to rebind cell claims from session ${currentSessionId} to ${newSessionId}: ${errDetail}. ` +
                  `To rebind manually, run: bee cells rebind-session --from ${currentSessionId} --to ${newSessionId}`,
                "warning",
              )
            }
          }

          const details: string[] = []
          if (reboundClaimsCount > 0) {
            details.push(`rebound ${reboundClaimsCount} ${reboundClaimsCount === 1 ? "claim" : "claims"}`)
          }
          if (carriedJobsCount > 0) {
            details.push(`carried ${carriedJobsCount} ${carriedJobsCount === 1 ? "job" : "jobs"}`)
          }
          const detailSuffix = details.length > 0 ? ` — ${details.join(", ")}` : ""

          if (intent.operation === "exit-worktree-before-merge") {
            if (details.length > 0) {
              replacedCtx.ui?.notify?.(
                `Relocated session before merge (${intent.targetCwd})${detailSuffix}`,
                "info",
              )
            }
            await handlePostExitMerge(replacedCtx, intent)
          } else if (intent.operation === "exit-worktree") {
            const worktreeId = intent.worktreeId || ""
            const reentryCmd = `/bee-worktree-enter --id ${worktreeId}`
            replacedCtx.ui?.notify?.(
              `Relocated session to main (${intent.targetCwd})${detailSuffix}. The worktree was kept. To return, run: ${reentryCmd}`,
              "info",
            )
          } else {
            replacedCtx.ui?.notify?.(
              `Relocated session to worktree ${intent.worktreeId || ""} (${intent.targetCwd})${detailSuffix}`,
              "info",
            )
          }
        },
      })
    } catch (switchErr: any) {
      if (reboundSessionFrom && reboundSessionTo) {
        try {
          await execBeeCli(
            mainRoot,
            ["cells", "rebind-session", "--from", reboundSessionTo, "--to", reboundSessionFrom, "--json"],
            currentSessionId,
          )
        } catch {}
      }
      if (carryRecordedNewSession) {
        carriedInboxTokens.delete(carryRecordedNewSession)
        saveRelocationCarry(mainRoot)
      }
      if (!state.transitionTeardownOccurred) {
        if (forkedSessionFile && existsSync(forkedSessionFile)) {
          try {
            rmSync(forkedSessionFile, { force: true })
          } catch {}
        }
      }
      const switchErrMsg = switchErr?.message ?? switchErr
      console.error(
        state.transitionTeardownOccurred
          ? `bee session switch failed after teardown: ${switchErrMsg}`
          : `bee session switch failed: ${switchErrMsg}`,
      )
      try {
        ctx.ui?.notify?.(
          state.transitionTeardownOccurred
            ? `Session switch failed after teardown: ${switchErrMsg}`
            : `Session switch failed: ${switchErrMsg}`,
          "error",
        )
      } catch (notifyErr: any) {
        console.error(
          `bee UI notification failed after session switch error: ${notifyErr?.message ?? notifyErr}`,
        )
      }
      return false
    }

    if (switchResult && switchResult.cancelled) {
      if (reboundSessionFrom && reboundSessionTo) {
        try {
          await execBeeCli(
            mainRoot,
            ["cells", "rebind-session", "--from", reboundSessionTo, "--to", reboundSessionFrom, "--json"],
            currentSessionId,
          )
        } catch {}
      }
      if (carryRecordedNewSession) {
        carriedInboxTokens.delete(carryRecordedNewSession)
        saveRelocationCarry(mainRoot)
      }
      if (!state.transitionTeardownOccurred && forkedSessionFile && existsSync(forkedSessionFile)) {
        try {
          rmSync(forkedSessionFile, { force: true })
        } catch {}
      }
      try {
        ctx.ui?.notify?.("Session switch was cancelled", "info")
      } catch (notifyErr: any) {
        console.error(
          `bee UI notification failed after session cancellation: ${notifyErr?.message ?? notifyErr}`,
        )
      }
      return false
    }

    return true
  } finally {
    state.activeTransition = false
    state.transitionTeardownOccurred = false
  }
}

