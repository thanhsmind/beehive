import type { ExtensionAPI } from "@earendil-works/pi-coding-agent"
import { directoryOf, sessionIdOf } from "./session.ts"
import { execBeeCli } from "./bee-cli.ts"
import {
  pendingTransitions,
  performSessionTransition,
  revalidateDeferredIntent,
  tokenizeArgv,
} from "./transition.ts"
import { runAdvisoryHook } from "./hooks.ts"
import type { Belt } from "./events.ts"

export function registerCommands(pi: ExtensionAPI, belt: Belt): void {
  // ── Worktree session relocation commands (pwsr-2) ──────────────────────────

  pi.registerCommand("bee-worktree-new", {
    description: "Create and enter a new bee worktree",
    handler: async (args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      let tokens: string[]
      try {
        tokens = tokenizeArgv(args)
      } catch (err: any) {
        ctx.ui?.notify?.(`Command argument error: ${err?.message ?? err}`, "error")
        return
      }
      const beeArgs = ["worktree", "new", ...tokens]
      if (!beeArgs.includes("--json")) {
        beeArgs.push("--json")
      }
      const directory = directoryOf(ctx)
      const result = await execBeeCli(directory, beeArgs, sessionIdOf(ctx))
      if (result.exitCode !== 0) {
        ctx.ui?.notify?.(result.stderr.trim() || result.stdout.trim() || "bee worktree new failed", "error")
        return
      }
      let parsed: any
      try {
        parsed = JSON.parse(result.stdout)
      } catch {
        ctx.ui?.notify?.("Failed to parse bee output as JSON", "error")
        return
      }
      if (parsed?.sessionTransition) {
        await performSessionTransition(ctx, parsed.sessionTransition)
      } else {
        ctx.ui?.notify?.(
          "bee version mismatch: worktree new succeeded but output lacked sessionTransition intent. Upgrade bee to enable session relocation.",
          "error",
        )
      }
    },
  })

  pi.registerCommand("bee-worktree-enter", {
    description: "Enter an existing bee worktree",
    handler: async (args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      let tokens: string[]
      try {
        tokens = tokenizeArgv(args)
      } catch (err: any) {
        ctx.ui?.notify?.(`Command argument error: ${err?.message ?? err}`, "error")
        return
      }
      const beeArgs = ["worktree", "enter", ...tokens]
      if (!beeArgs.includes("--json")) {
        beeArgs.push("--json")
      }
      const directory = directoryOf(ctx)
      const result = await execBeeCli(directory, beeArgs, sessionIdOf(ctx))
      if (result.exitCode !== 0) {
        ctx.ui?.notify?.(result.stderr.trim() || result.stdout.trim() || "bee worktree enter failed", "error")
        return
      }
      let parsed: any
      try {
        parsed = JSON.parse(result.stdout)
      } catch {
        ctx.ui?.notify?.("Failed to parse bee output as JSON", "error")
        return
      }
      if (parsed?.sessionTransition) {
        await performSessionTransition(ctx, parsed.sessionTransition)
      } else {
        ctx.ui?.notify?.(
          "bee version mismatch: worktree enter succeeded but output lacked sessionTransition intent. Upgrade bee to enable session relocation.",
          "error",
        )
      }
    },
  })

  pi.registerCommand("bee-worktree-exit", {
    description: "Exit current bee worktree back to main",
    handler: async (args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      let tokens: string[]
      try {
        tokens = tokenizeArgv(args)
      } catch (err: any) {
        ctx.ui?.notify?.(`Command argument error: ${err?.message ?? err}`, "error")
        return
      }
      const beeArgs = ["worktree", "exit", ...tokens]
      if (!beeArgs.includes("--json")) {
        beeArgs.push("--json")
      }
      const directory = directoryOf(ctx)
      const result = await execBeeCli(directory, beeArgs, sessionIdOf(ctx))
      if (result.exitCode !== 0) {
        ctx.ui?.notify?.(result.stderr.trim() || result.stdout.trim() || "bee worktree exit failed", "error")
        return
      }
      let parsed: any
      try {
        parsed = JSON.parse(result.stdout)
      } catch {
        ctx.ui?.notify?.("Failed to parse bee output as JSON", "error")
        return
      }
      if (parsed?.sessionTransition) {
        await performSessionTransition(ctx, parsed.sessionTransition)
      } else {
        ctx.ui?.notify?.(
          "bee version mismatch: worktree exit succeeded but output lacked sessionTransition intent. Upgrade bee to enable session relocation.",
          "error",
        )
      }
    },
  })

  pi.registerCommand("bee-worktree-merge", {
    description: "Merge current bee worktree back to main",
    handler: async (args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      let tokens: string[]
      try {
        tokens = tokenizeArgv(args)
      } catch (err: any) {
        ctx.ui?.notify?.(`Command argument error: ${err?.message ?? err}`, "error")
        return
      }
      const beeArgs = ["worktree", "merge", ...tokens]
      if (!beeArgs.includes("--json")) {
        beeArgs.push("--json")
      }
      const directory = directoryOf(ctx)
      const result = await execBeeCli(directory, beeArgs, sessionIdOf(ctx))
      if (result.exitCode !== 0) {
        ctx.ui?.notify?.(result.stderr.trim() || result.stdout.trim() || "bee worktree merge failed", "error")
        return
      }
      let parsed: any
      try {
        parsed = JSON.parse(result.stdout)
      } catch {
        const out = result.stdout.trim() || result.stderr.trim()
        ctx.ui?.notify?.(`Merge succeeded: ${out || "Merged worktree into main"}`, "info")
        return
      }
      if (parsed?.sessionTransition) {
        await performSessionTransition(ctx, parsed.sessionTransition)
      } else {
        const out = result.stdout.trim()
        ctx.ui?.notify?.(`Merge succeeded: ${out || "Merged worktree into main"}`, "info")
      }
    },
  })

  pi.registerCommand("bee-worktree-relocate", {
    description: "Internal session relocation command for bee worktree transitions",
    handler: async (args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      const token = args.trim()
      if (!token) return
      const pending = pendingTransitions.get(token)
      if (!pending) return
      pendingTransitions.delete(token)

      const curSessionId = sessionIdOf(ctx)
      if (
        !curSessionId ||
        !pending.sessionId ||
        pending.sessionId !== curSessionId ||
        typeof pending.transition?.piSessionId !== "string" ||
        pending.transition.piSessionId !== curSessionId
      ) {
        ctx.ui?.notify?.(
          "Session transition refused: session ID mismatch on private command",
          "error",
        )
        return
      }

      const verifiedTransition = await revalidateDeferredIntent(ctx, pending.transition)
      if (!verifiedTransition) {
        ctx.ui?.notify?.(
          "Session transition refused: deferred transition intent failed authenticity validation",
          "error",
        )
        return
      }

      await performSessionTransition(ctx, verifiedTransition)
    },
  })

  pi.registerCommand("bee-tools-reopen", {
    description: "Restore the full tool set after stage narrowing",
    handler: async (_args: string, ctx: any) => {
      if (typeof ctx?.isIdle === "function" && !ctx.isIdle()) {
        ctx.ui?.notify?.("Command refused: agent turn is currently active", "error")
        return
      }
      try {
        belt.toolsReopened = true
        let toolsToRestore = belt.fullToolSet
        if ((!toolsToRestore || toolsToRestore.length === 0) && typeof (pi as any).getAllTools === "function") {
          try {
            const all = (pi as any).getAllTools()
            if (Array.isArray(all) && all.length > 0) {
              toolsToRestore = all
                .map((t: any) => (typeof t === "string" ? t : t?.name))
                .filter(Boolean)
            }
          } catch {}
        }
        if (toolsToRestore && toolsToRestore.length > 0 && typeof (pi as any).setActiveTools === "function") {
          (pi as any).setActiveTools(toolsToRestore)
          ctx.ui?.notify?.(`Restored full tool set (${toolsToRestore.join(", ")})`, "info")
          if (typeof (pi as any).sendMessage === "function") {
            try {
              await (pi as any).sendMessage({
                customType: "bee-stage-tools",
                content: `Notice: Full tool set restored (${toolsToRestore.join(", ")}).`,
                display: true,
                details: { restoredTools: toolsToRestore },
              })
            } catch {}
          }
        } else {
          ctx.ui?.notify?.("No stored tool set to restore", "info")
        }
      } catch (err: any) {
        ctx.ui?.notify?.(`Failed to restore tools: ${err?.message ?? err}`, "error")
      }
    },
  })

  pi.registerCommand("bee-obligation-skip", {
    description: "Skip an obligation by key so it produces no continuation",
    handler: async (args: string, ctx: any) => {
      const key = String(args ?? "").trim()
      if (!key) {
        if (ctx?.hasUI !== false && typeof ctx?.ui?.notify === "function") {
          ctx.ui.notify("Usage: /bee-obligation-skip <key>", "error")
        }
        return
      }
      try {
        const directory = directoryOf(ctx)
        runAdvisoryHook(directory, "session-close", {
          hook_event_name: "Stop",
          session_id: sessionIdOf(ctx),
          cwd: directory,
          skip_key: key,
        })
        if (ctx?.hasUI !== false && typeof ctx?.ui?.notify === "function") {
          ctx.ui.notify(`Skipped obligation: ${key}`, "info")
        }
      } catch (err: any) {
        console.error(`bee obligation skip (advisory): ${err?.message ?? err}`)
      }
    },
  })

}
