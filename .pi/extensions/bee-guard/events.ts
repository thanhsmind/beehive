import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
import cp, { execFile } from "node:child_process"
import { rmSync } from "node:fs"
import { directoryOf, sessionIdOf, sessionSource } from "./session.ts"
import { beeStorePresent, resolveBeeBinary } from "./locate.ts"
import { mapToolCall, BEE_STAGE_TOOLS } from "./tool-map.ts"
import { runBlockingHook, runAdvisoryHook } from "./hooks.ts"
import { refreshModelUsageStatus } from "./model-usage.ts"
import {
  forcedContinuationSessions,
  inFlightClaims,
  promptDepths,
  startResultDrain,
} from "./result-inbox.ts"
import {
  extractAndCaptureTransitionMarker,
  pendingRelocationTokens,
  pendingTransitions,
  performSessionTransition,
  processTextForMarkers,
  revalidateDeferredIntent,
} from "./transition.ts"
import { drainWorkerSteer } from "./tool-steer.ts"
import { state } from "./state.ts"

// ─── the belt ──────────────────────────────────────────────────────────────

export interface Belt {
  fullToolSet: string[] | null
  lastStage: string | null
  toolsReopened: boolean
}

export function registerEvents(pi: ExtensionAPI, belt: Belt): void {
  // ── BLOCKING: write-guard on every tool call. Fail CLOSED. ───────────────
  pi.on("tool_call", (async (event: any, ctx: any) => {
    const directory = directoryOf(ctx)
    // Passivity, per call: no .bee store anywhere -> this is not a bee repo,
    // and a Pi session here must feel nothing at all.
    if (!beeStorePresent(directory)) return undefined

    const mapped = mapToolCall(String(event?.toolName ?? ""), event?.input)
    if (mapped.hook === null) return undefined
    return runBlockingHook(
      directory,
      mapped.hook,
      {
        hook_event_name: "PreToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_input: mapped.tool_input,
        bee_runtime: "pi",
        tools_reopened: belt.toolsReopened,
      },
      // The repair target is Pi's own mutable `event.input` — only ever
      // written for a pass-through mapping (see runBlockingHook).
      (event?.input ?? {}) as Record<string, unknown>,
      mapped.passthrough,
    )
  }) as any)

  // ── ADVISORY: session-init, ONCE per session, cached (D8). ───────────────
  pi.on("session_start", (async (event: any, ctx: any) => {
    refreshModelUsageStatus(ctx)
    try {
      const reason = event?.reason as string | undefined
      if (reason === "reload" && state.sessionInitRun) return // /reload is idempotent
      if (reason !== "reload") {
        state.cachedPreamble = null
        state.preambleInjected = false
        state.sessionInitRun = false
        const activeSessionId = sessionIdOf(ctx) ?? ""
        promptDepths.delete(activeSessionId)
      }
      const directory = directoryOf(ctx)
      // D4: the result-inbox drain arms HERE and only here — never at module
      // load. Its own try, so a drain that cannot start never costs the
      // session its preamble.
      try {
        startResultDrain(pi, directory, sessionIdOf(ctx), ctx)
      } catch (err: any) {
        console.error(`bee result-inbox (advisory) could not start: ${err?.message ?? err}`)
      }
      const text = runAdvisoryHook(directory, "session-init", {
        hook_event_name: "SessionStart",
        session_id: sessionIdOf(ctx),
        source: sessionSource(reason),
        cwd: directory,
        runtime: "pi",
      })
      state.sessionInitRun = true
      if (text) state.cachedPreamble = text
    } catch (err: any) {
      console.error(`bee session-init (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: the per-turn context feed (D8). The cached preamble rides
  // the FIRST turn only; `bee hook prompt-context` runs unchanged on every
  // turn and is the per-turn delta. Never throws, never blocks a turn. ──────
  pi.on("before_agent_start", (async (event: any, ctx: any) => {
    // D4/F1: a turn has begun. The latch that kept a burst from opening a
    // second overlapping plain turn is released, and every result drained from
    // here until `agent_settled` is STEERED into this running turn instead of
    // starting one of its own. Before the try: neither assignment can throw,
    // and the busy fact must never depend on the hook call below.
    state.turnStartPending = false
    state.selfBusy = true
    if (ctx) state.activeDrainCtx = ctx
    try {
      const directory = directoryOf(ctx)
      const parts: string[] = []

      if (state.cachedPreamble && !state.preambleInjected) {
        state.preambleInjected = true
        parts.push(state.cachedPreamble)
      }

      const delta = runAdvisoryHook(directory, "prompt-context", {
        hook_event_name: "UserPromptSubmit",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        prompt: typeof event?.prompt === "string" ? event.prompt : "",
      })
      if (delta) parts.push(delta)

      try {
        runAdvisoryHook(directory, "activity", {
          hook_event_name: "UserPromptSubmit",
          session_id: sessionIdOf(ctx),
          cwd: directory,
          prompt: typeof event?.prompt === "string" ? event.prompt : "",
        })
      } catch (err: any) {
        console.error(`bee activity (advisory): ${err?.message ?? err}`)
      }

      if (parts.length === 0) return undefined
      const base = typeof event?.systemPrompt === "string" ? event.systemPrompt : ""
      return { systemPrompt: `${base}\n\n${parts.join("\n\n")}` }
    } catch (err: any) {
      console.error(`bee prompt-context (advisory): ${err?.message ?? err}`)
      return undefined
    }
  }) as any)

  // ── ADVISORY: pre-tool activity. Maps to PreToolUse in activity.
  // Records working activity before the blocking tool_call path runs.
  pi.on("tool_execution_start", (async (event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      const mapped = mapToolCall(String(event?.toolName ?? ""), event?.args)
      runAdvisoryHook(directory, "activity", {
        hook_event_name: "PreToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_use_id: typeof event?.toolCallId === "string" ? event.toolCallId : undefined,
      })
    } catch (err: any) {
      console.error(`bee activity tool_execution_start (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: UI prompt start maps to Notification:agent_needs_input.
  // Tracks nested UI prompt depth. Only the outermost start enters waiting_input.
  pi.on("ui_prompt_start", (async (_event: any, ctx: any) => {
    try {
      const activeSessionId = sessionIdOf(ctx) ?? ""
      const currentDepth = promptDepths.get(activeSessionId) ?? 0
      promptDepths.set(activeSessionId, currentDepth + 1)
      if (currentDepth === 0) {
        const directory = directoryOf(ctx)
        runAdvisoryHook(directory, "activity", {
          hook_event_name: "Notification",
          notification_type: "agent_needs_input",
          session_id: sessionIdOf(ctx),
          cwd: directory,
        })
      }
    } catch (err: any) {
      console.error(`bee activity ui_prompt_start (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: UI prompt end maps to UserPromptSubmit without prompt text.
  // Ends the waiting_input span and returns activity to working. Unmatched
  // ends are ignored. Inner ends decrement depth without ending the wait span.
  pi.on("ui_prompt_end", (async (_event: any, ctx: any) => {
    try {
      const activeSessionId = sessionIdOf(ctx) ?? ""
      const currentDepth = promptDepths.get(activeSessionId) ?? 0
      if (currentDepth <= 0) {
        return // unmatched end is ignored
      }
      if (currentDepth === 1) {
        promptDepths.delete(activeSessionId)
        const directory = directoryOf(ctx)
        runAdvisoryHook(directory, "activity", {
          hook_event_name: "UserPromptSubmit",
          session_id: sessionIdOf(ctx),
          cwd: directory,
        })
      } else {
        promptDepths.set(activeSessionId, currentDepth - 1)
      }
    } catch (err: any) {
      console.error(`bee activity ui_prompt_end (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  pi.on("input", (async (event: any, ctx: any) => {
    try {
      const text = typeof event?.text === "string" ? event.text : ""
      const behavior = typeof event?.streamingBehavior === "string" ? event.streamingBehavior : ""
      const isSteerOrFollowUp = behavior === "steer" || behavior === "followUp" || behavior === "follow_up"
      if (isSteerOrFollowUp && text.trim().length > 0) {
        const directory = directoryOf(ctx)
        const beeBinary = resolveBeeBinary(directory)
        if (beeBinary && beeStorePresent(directory)) {
          const payload = JSON.stringify({
            hook_event_name: "SessionClose",
            session_id: sessionIdOf(ctx),
            cwd: directory,
            record_scope_input: text,
          })
          const child = cp.execFile(
            beeBinary,
            ["hook", "session-close"],
            { cwd: directory, timeout: 5000 },
            () => {},
          )
          child.stdin?.on("error", () => {})
          child.stdin?.end(payload)
        }
      }
    } catch {}
    return { action: "continue" }
  }) as any)

  // ── ADVISORY: state-sync, tools-logger, and activity after every tool result.
  // Strips and captures session transition markers from successful shell results. ──
  pi.on("tool_result", (async (event: any, ctx: any) => {
    const directory = directoryOf(ctx)
    const mapped = mapToolCall(String(event?.toolName ?? ""), event?.input)
    try {
      runAdvisoryHook(directory, "state-sync", {
        hook_event_name: "PostToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
      })
    } catch (err: any) {
      console.error(`bee state-sync (advisory): ${err?.message ?? err}`)
    }
    try {
      runAdvisoryHook(directory, "tools-logger", {
        hook_event_name: "PostToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
      })
    } catch (err: any) {
      console.error(`bee tools-logger (advisory): ${err?.message ?? err}`)
    }
    try {
      const isError = Boolean(event?.isError)
      runAdvisoryHook(directory, "activity", {
        hook_event_name: isError ? "PostToolUseFailure" : "PostToolUse",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_use_id: typeof event?.toolCallId === "string" ? event.toolCallId : undefined,
      })
    } catch (err: any) {
      console.error(`bee activity (advisory): ${err?.message ?? err}`)
    }
    let patch: { content: any } | undefined
    try {
      const isError = Boolean(event?.isError)
      if (!isError) {
        const toolName = String(event?.toolName ?? "").toLowerCase()
        if (toolName === "bash" || toolName === "powershell") {
          patch = extractAndCaptureTransitionMarker(event, ctx)
        }
      }
    } catch (err: any) {
      console.error(`bee transition marker capture (advisory): ${err?.message ?? err}`)
    }
    return patch
  }) as any)

  pi.on("agent_before_settle", (async (event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      const raw = runAdvisoryHook(directory, "session-close", {
        hook_event_name: "Stop",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        obligations_only: true,
      })
      if (!raw || typeof raw !== "string" || raw.trim().length === 0) {
        return undefined
      }
      let parsed: any = null
      try {
        parsed = JSON.parse(raw.trim())
      } catch {
        return undefined
      }
      const obligations = Array.isArray(parsed?.obligations) ? parsed.obligations : []
      if (obligations.length === 0) {
        return undefined
      }
      for (const o of obligations) {
        if (typeof o?.user_notice === "string" && ctx?.hasUI !== false && typeof ctx?.ui?.notify === "function") {
          try {
            ctx.ui.notify(o.user_notice, "warning")
          } catch (err: any) {
            console.error(`bee settle obligation notify (advisory): ${err?.message ?? err}`)
          }
        }
      }
      const activeSessionId = sessionIdOf(ctx) ?? ""
      if (activeSessionId) {
        forcedContinuationSessions.add(activeSessionId)
      }
      const existingEntries = Array.isArray(event?.entries) ? event.entries : []
      const obligationEntries = obligations.map((o: any) => ({
        type: "custom_message",
        customType: "bee-obligation",
        content: o.message,
        display: true,
        details: {
          key: o.key,
          kind: o.kind,
          cell: o.cell,
        },
      }))
      return {
        entries: [...existingEntries, ...obligationEntries],
        continue: true,
      }
    } catch (err: any) {
      console.error(`bee agent_before_settle (advisory): ${err?.message ?? err}`)
      return undefined
    }
  }) as any)

  // ── ADVISORY: the turn-end waiting mark and continuation nudge.
  // `agent_settled` is Pi's own "nothing will continue automatically" signal
  // (docs/extensions.md:569) — the Stop analog, where session-close sets the
  // `turn-end` waiting mark (session_close/mod.rs:260-309). When session-close
  // emits a block verdict (decision: "block"), we enforce continuation by
  // injecting the reason into the session via `pi.sendUserMessage` (pib-3).
  // Advisory nudges (systemMessage) do not trigger injection. ───────────────
  pi.on("agent_settled", (async (_event: any, ctx: any) => {
    // D4/F2: the turn is over, so the claims injected INTO it are consumed
    // now — not when `sendUserMessage` returned. A claim still on disk after a
    // crash is reclaimed at the next `session_start`, which is what makes this
    // channel at-least-once rather than at-most-once.
    const activeSessionId = sessionIdOf(ctx) ?? ""
    const hadForcedContinuation = forcedContinuationSessions.has(activeSessionId)
    forcedContinuationSessions.delete(activeSessionId)
    try {
      promptDepths.delete(activeSessionId)
      state.selfBusy = false
      state.turnStartPending = false
      for (const processing of inFlightClaims) rmSync(processing, { force: true })
      inFlightClaims.clear()
    } catch (err: any) {
      console.error(`bee result-inbox (advisory): could not consume a claim: ${err?.message ?? err}`)
    }

    try {
      if (activeSessionId && pendingRelocationTokens.has(activeSessionId)) {
        const token = pendingRelocationTokens.get(activeSessionId)!
        pendingRelocationTokens.delete(activeSessionId)
        setTimeout(async () => {
          try {
            if (typeof pi?.sendUserMessage === "function") {
              await pi.sendUserMessage(`/bee-worktree-relocate ${token}`, {
                expandPromptTemplates: true,
              })
            }
          } catch (err: any) {
            console.error(`bee relocation dispatch (advisory): ${err?.message ?? err}`)
          }
        }, 0)
      }
    } catch (err: any) {
      console.error(`bee relocation (advisory): ${err?.message ?? err}`)
    }
    let sessionCloseVerdict: string | null = null
    try {
      const directory = directoryOf(ctx)
      try {
        runAdvisoryHook(directory, "state-sync", {
          hook_event_name: "Stop",
          session_id: sessionIdOf(ctx),
          cwd: directory,
        })
      } catch (err: any) {
        console.error(`bee state-sync (advisory): ${err?.message ?? err}`)
      }
      const rawVerdict = runAdvisoryHook(directory, "session-close", {
        hook_event_name: "Stop",
        session_id: sessionIdOf(ctx),
        cwd: directory,
      })
      sessionCloseVerdict = rawVerdict
      // Gated continuation nudge (Epic C / pib-3): session-close emits a block
      // verdict {"decision":"block","reason":"..."} when maybe_bypass_block in
      // hooks/session_close/nudges.rs triggers (e.g., gate_bypass in planning mode).
      // Advisory nudges emit {"systemMessage":"..."} and must NOT inject a turn.
      //
      // Deduplication: no client-side repeat guard is created here. The Rust hook
      // already deduplicates bypass nudges in nudges.rs:425-430 via
      // should_inject(root, "bypass-stop-net", &hash) with a 30-minute window.
      if (typeof rawVerdict === "string" && rawVerdict.trim().length > 0) {
        try {
          const parsed = JSON.parse(rawVerdict.trim())
          if (
            !hadForcedContinuation &&
            parsed &&
            parsed.decision === "block" &&
            typeof parsed.reason === "string" &&
            parsed.reason.trim().length > 0
          ) {
            if (typeof pi?.sendUserMessage === "function") {
              state.turnStartPending = true
              try {
                await pi.sendUserMessage(parsed.reason)
              } catch (injectErr: any) {
                state.turnStartPending = false
                console.error(`bee: failed to inject continuation nudge into session: ${injectErr?.message ?? injectErr}`)
              }
            }
          }
        } catch {
          // Non-JSON or unparseable output on advisory hook is ignored
        }
      }
      try {
        runAdvisoryHook(directory, "activity", {
          hook_event_name: "Stop",
          session_id: sessionIdOf(ctx),
          cwd: directory,
        })
      } catch (err: any) {
        console.error(`bee activity (advisory): ${err?.message ?? err}`)
      }
    } catch (err: any) {
      console.error(`bee session-close (advisory): ${err?.message ?? err}`)
    }

    // ── ADVISORY: close guard (D5, D13).
    // When a session settles with a claimed cell that was never capped, warn into
    // the visible session transcript naming that cell and the verb to run (D5, D13).
    // This is warn-only: the session still ends, nothing blocks or delays settling.
    try {
      const directory = directoryOf(ctx)
      const raw = sessionCloseVerdict ?? runAdvisoryHook(directory, "session-close", {
        hook_event_name: "Stop",
        session_id: sessionIdOf(ctx),
        cwd: directory,
      })
      if (typeof raw === "string" && raw.trim().length > 0) {
        try {
          const parsed = JSON.parse(raw.trim())
          const msg = typeof parsed?.systemMessage === "string" ? parsed.systemMessage : ""
          let cells = ""
          if (Array.isArray(parsed?.claimed_cells) && parsed.claimed_cells.length > 0) {
            cells = parsed.claimed_cells.join(", ")
          } else if (msg) {
            const match = /Claimed-but-uncapped cells:\s*([^\n]+)/.exec(msg)
            if (match && match[1]) {
              cells = match[1].replace(/\.$/, "").trim()
            }
          }
          if (cells) {
            const warningNotice = `Warning: session settled with claimed uncapped cell(s): ${cells}. Run \`bee cells finish\` (or \`bee cells release\`) to resolve.`
            if (typeof (pi as any).sendMessage === "function") {
              try {
                await (pi as any).sendMessage(
                  {
                    customType: "bee-close-warning",
                    content: warningNotice,
                    display: true,
                    details: {
                      cells,
                      verb: "bee cells finish",
                    },
                  },
                  {
                    deliverAs: "nextTurn",
                    triggerTurn: false,
                  },
                )
              } catch (sendErr: any) {
                console.error(`bee close-guard transcript warning (advisory): ${sendErr?.message ?? sendErr}`)
              }
            }
            if (ctx?.hasUI !== false && typeof ctx?.ui?.notify === "function") {
              try {
                ctx.ui.notify(warningNotice, "warning")
              } catch (notifyErr: any) {
                console.error(`bee close-guard notify (advisory): ${notifyErr?.message ?? notifyErr}`)
              }
            }
          }
        } catch {
          // Non-JSON or unparseable output on advisory hook is ignored
        }
      }
    } catch (err: any) {
      console.error(`bee close-guard (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: per-stage active tool narrowing (D4, D12). ──────────────────
  pi.on("turn_start", (async (event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      await drainWorkerSteer(pi, directory)
      if (!beeStorePresent(directory)) return undefined

      // Capture all known/registered tools before any narrowing occurs so the
      // full set can be restored by the re-open command.
      if (!belt.fullToolSet && typeof (pi as any).getAllTools === "function") {
        try {
          const all = (pi as any).getAllTools()
          if (Array.isArray(all) && all.length > 0) {
            belt.fullToolSet = all
              .map((t: any) => (typeof t === "string" ? t : t?.name))
              .filter(Boolean)
          }
        } catch {}
      }
      if (!belt.fullToolSet && typeof (pi as any).getActiveTools === "function") {
        try {
          const active = (pi as any).getActiveTools()
          if (Array.isArray(active) && active.length > 0) {
            belt.fullToolSet = [...active]
          }
        } catch {}
      }

      const raw = runAdvisoryHook(directory, "stage-tools", {
        hook_event_name: "TurnStart",
        session_id: sessionIdOf(ctx),
        cwd: directory,
        turn_index: typeof event?.turnIndex === "number" ? event.turnIndex : undefined,
      })
      if (!raw) return undefined

      let parsed: any
      try {
        parsed = JSON.parse(raw.trim())
      } catch {
        return undefined
      }

      if (!parsed || typeof parsed !== "object") return undefined
      const allowed = Array.isArray(parsed.allowed_tools)
        ? parsed.allowed_tools
        : Array.isArray(parsed.allowedTools)
          ? parsed.allowedTools
          : Array.isArray(parsed.tools)
            ? parsed.tools
            : null

      if (!allowed) return undefined

      const stage =
        typeof parsed.stage === "string"
          ? parsed.stage
          : typeof parsed.stage_name === "string"
            ? parsed.stage_name
            : ""

      if (belt.lastStage !== null && stage !== belt.lastStage) {
        belt.toolsReopened = false
      }
      belt.lastStage = stage

      if (belt.toolsReopened) return undefined

      let currentActive: string[] = []
      if (typeof (pi as any).getActiveTools === "function") {
        currentActive = (pi as any).getActiveTools()
      } else if (belt.fullToolSet) {
        currentActive = [...belt.fullToolSet]
      }

      const allowedSet = new Set(allowed)
      const keeps = (t: string) => (BEE_STAGE_TOOLS.has(t) ? allowedSet.has(t) : currentActive.includes(t))
      const pool = [...new Set([...(belt.fullToolSet ?? []), ...currentActive])]
      const targetActive = pool.filter(keeps)
      const removedTools = currentActive.filter((t) => !keeps(t))
      const addedTools = targetActive.filter((t) => !currentActive.includes(t))

      if (removedTools.length + addedTools.length > 0 && typeof (pi as any).setActiveTools === "function") {
        (pi as any).setActiveTools(targetActive)

        if (typeof parsed.user_message === "string" && typeof ctx?.ui?.notify === "function") {
          ctx.ui.notify(parsed.user_message, "info")
        }

        if (typeof parsed.model_message === "string" && typeof (pi as any).sendMessage === "function") {
          try {
            await (pi as any).sendMessage({
              customType: "bee-stage-tools",
              content: parsed.model_message,
              display: true,
              details: {
                stage: stage || null,
                removedTools,
                allowedTools: targetActive,
              },
            })
          } catch (err: any) {
            console.error(`bee stage-tools model notice (advisory): ${err?.message ?? err}`)
          }
        }
      }
    } catch (err: any) {
      console.error(`bee stage-tools (advisory): ${err?.message ?? err}`)
    }
  }) as any)

  // ── ADVISORY: active-branch model usage statusline refresh ─────────────────
  pi.on("turn_end", (async (_event: any, ctx: any) => {
    if (ctx) state.activeDrainCtx = ctx
    refreshModelUsageStatus(ctx)
    try {
      const directory = directoryOf(ctx)
      await drainWorkerSteer(pi, directory)
    } catch {}
  }) as any)

  pi.on("session_tree", (async (_event: any, ctx: any) => {
    refreshModelUsageStatus(ctx)
  }) as any)

  // ── ADVISORY: pre-compact session check (b1a26071). Must return nothing /
  // undefined: Pi treats any returned object as a custom compaction or cancel,
  // which would silently drop or abort the user's /compact. ─────────────────
  pi.on("session_before_compact", (async (_event: any, ctx: any) => {
    try {
      const directory = directoryOf(ctx)
      runAdvisoryHook(directory, "session-close", {
        hook_event_name: "PreCompact",
        session_id: sessionIdOf(ctx),
        cwd: directory,
      })
    } catch (err: any) {
      console.error(`bee session-close pre-compact (advisory): ${err?.message ?? err}`)
    }
    return undefined
  }) as any)

  // ── ADVISORY: session shutdown handler. Closes the session record and marks
  // activity state as exited on all reasons except /reload (which continues the
  // same session and is treated as idempotent by session_start above).
  // Closing on reload would mark a live session as dead and drop its worktree
  // hold prematurely, so the handler exits early on "reload" and terminates on
  // everything else.
  pi.on("session_shutdown", (async (event: any, ctx: any) => {
    try {
      const reason = event?.reason as string | undefined
      // Precedent & reason mapping:
      // 1. session_start above treats "reload" as the same session continuing.
      //    Therefore "reload" returns early and neither session-close nor activity runs.
      // 2. All other reasons (including undefined/default) terminate the active session.
      // 3. In Claude's vocabulary, "resume" means transcript resumption of the SAME session,
      //    so activity.rs deliberately ignores reason: "resume" (map_event returns None).
      //    In Pi, "resume" means switching away to another session file, so THIS session is ending.
      //    To prevent activity.rs from silently skipping the exit transition on Pi's "resume",
      //    all terminating Pi reasons map to a Claude-shaped exit reason ("quit").
      if (reason === "resume" && state.activeTransition) {
        state.transitionTeardownOccurred = true
      }
      if (reason === "reload") return undefined
      const directory = directoryOf(ctx)
      try {
        runAdvisoryHook(directory, "session-close", {
          hook_event_name: "SessionEnd",
          session_id: sessionIdOf(ctx),
          cwd: directory,
        })
      } catch (err: any) {
        console.error(`bee session-close shutdown (advisory): ${err?.message ?? err}`)
      }
      try {
        runAdvisoryHook(directory, "activity", {
          hook_event_name: "SessionEnd",
          session_id: sessionIdOf(ctx),
          cwd: directory,
          reason: "quit",
        })
      } catch (err: any) {
        console.error(`bee activity shutdown (advisory): ${err?.message ?? err}`)
      }
    } catch (err: any) {
      console.error(`bee session_shutdown (advisory): ${err?.message ?? err}`)
    }
    const activeSessionId = sessionIdOf(ctx) ?? ""
    promptDepths.delete(activeSessionId)
    pendingTransitions.clear()
    pendingRelocationTokens.clear()
    return undefined
  }) as any)

}
