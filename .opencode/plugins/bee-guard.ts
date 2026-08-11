// bee's OpenCode enforcement belt. This file holds ZERO guard rules of its
// own — every allow/deny verdict comes from `.bee/bin/bee hook write-guard`,
// the exact same brain the Claude and Codex hook belts call
// (packages/bee/hooks/claude-hooks.json). This file only:
//
//   1. locates the bee binary (mirrors claude-hooks.json's project-then-
//      main-worktree search, since a linked worktree checkout has no
//      vendored .bee/bin/bee of its own);
//   2. maps OpenCode's write/edit/bash `tool.execute.before` payloads onto
//      the JSON shape bee's write-guard hook already reads on stdin;
//   3. turns bee's documented DENY verdict (exit code 2, reason on stderr)
//      into a thrown Error — the only documented block path for
//      `tool.execute.before` (no abort field exists on `output`); and
//   4. turns every OTHER failure to reach a verdict — missing binary, a
//      spawn error, a crash, a signal kill, an unexpected exit code — into a
//      thrown Error too. Undecidable is fail-closed here, never a silent
//      allow: a fail-open host would swallow a fail-closed throw right back
//      into an allow, which is the one failure mode this file must never
//      have.
//
// Proof (deny + allow transcripts, skills discovery, AGENTS.md load) lives
// in docs/history/opencode-support/discovery.md.

import type { Plugin } from "@opencode-ai/plugin"
import { execFileSync } from "node:child_process"
import { existsSync } from "node:fs"
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
    "bee write-guard could not find the bee binary (.bee/bin/bee) in this project or its main worktree — " +
      "denying rather than letting a write through unchecked. " +
      "FIX: run `bee onboard --apply` (or vendor .bee/bin/bee) and retry.",
  )
}

type MappedCall = { tool_name: string; tool_input: Record<string, unknown> }

/** The only "rule" in this file: which OpenCode tools bee's write-guard
 * cares about, and the field-name translation into the PreToolUse shape bee
 * already reads (packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs).
 * Every actual allow/deny decision still comes from the bee binary. */
function mapToolCall(tool: string, args: any): MappedCall | null {
  switch (tool) {
    case "write":
      return { tool_name: "Write", tool_input: { file_path: args?.filePath, content: args?.content } }
    case "edit":
      return {
        tool_name: "Edit",
        tool_input: { file_path: args?.filePath, old_string: args?.oldString, new_string: args?.newString },
      }
    case "bash":
      return { tool_name: "Bash", tool_input: { command: args?.command } }
    default:
      return null // not a write-capable tool bee's write-guard gates — no rule here, just routing
  }
}

export default (async ({ directory }) => {
  return {
    "tool.execute.before": async (input, output) => {
      const mapped = mapToolCall(input.tool, output.args)
      if (!mapped) return

      const beeBinary = resolveBeeBinary(directory) // throws (fail-closed) if unresolved

      const payload = JSON.stringify({
        hook_event_name: "PreToolUse",
        session_id: input.sessionID,
        cwd: directory,
        tool_name: mapped.tool_name,
        tool_input: mapped.tool_input,
      })

      try {
        execFileSync(beeBinary, ["hook", "write-guard"], {
          input: payload,
          encoding: "utf8",
          cwd: directory,
        })
        // exit 0 — allow. Nothing to mutate; output.args passes through as-is.
      } catch (err: any) {
        // execFileSync throws on any non-zero exit or a spawn failure.
        // Exit code 2 is bee's documented DENY verdict — surface its stderr
        // reason as the block. Anything else (spawn failure, signal kill,
        // unexpected exit code, a native panic inside bee) is undecidable,
        // and undecidable is fail-closed here too, never fail-open.
        if (err?.status === 2) {
          const reason = (err.stderr ?? "").toString().trim()
          throw new Error(reason || "bee write-guard denied this write.")
        }
        const detail = (err?.stderr ?? err?.message ?? String(err)).toString().trim()
        throw new Error(
          `bee write-guard did not return a verdict (${detail || "no output"}) — ` +
            "denying rather than allowing an unchecked write.",
        )
      }
    },
  }
}) satisfies Plugin
