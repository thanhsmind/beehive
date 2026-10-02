import { execFile } from "node:child_process"
import { resolveBeeBinary } from "./locate.ts"

export interface ExecBeeResult {
  stdout: string
  stderr: string
  exitCode: number
}

export const BEE_CLI_TIMEOUT_MS = 30_000
export const DEFAULT_MERGE_QUEUE_WAIT_MS = 180_000
export const POST_EXIT_MERGE_MARGIN_MS = 60_000
export const MIN_POST_EXIT_TIMEOUT_MS = 30_000
export const NODE_MAX_TIMER_TIMEOUT_MS = 2_147_483_647 // 2**31 - 1, Node.js maximum setTimeout delay

export function derivePostExitTimeoutMs(queueWaitMs?: number | null): number {
  const waitMs =
    typeof queueWaitMs === "number" && Number.isFinite(queueWaitMs) && queueWaitMs >= 0
      ? queueWaitMs
      : DEFAULT_MERGE_QUEUE_WAIT_MS
  const derived = Math.ceil(waitMs + POST_EXIT_MERGE_MARGIN_MS)
  return Math.min(Math.max(derived, MIN_POST_EXIT_TIMEOUT_MS), NODE_MAX_TIMER_TIMEOUT_MS)
}

export function execBeeCli(
  directory: string,
  args: string[],
  sessionId?: string,
  timeoutMs: number = BEE_CLI_TIMEOUT_MS,
): Promise<ExecBeeResult> {
  return new Promise((resolve) => {
    const beeBinary = resolveBeeBinary(directory)
    if (!beeBinary) {
      return resolve({
        stdout: "",
        stderr: "bee binary not found in this project or its main worktree",
        exitCode: 127,
      })
    }
    const env = { ...process.env }
    if (sessionId) {
      env.PI_SESSION_ID = sessionId
    }
    env.BEE_EXEC_TIMEOUT_MS = String(timeoutMs)
    const child = execFile(
      beeBinary,
      args,
      {
        cwd: directory,
        env,
        timeout: timeoutMs,
        maxBuffer: 10 * 1024 * 1024,
        encoding: "utf8",
      },
      (error, stdout, stderr) => {
        let exitCode = 0
        let errDetail = ""
        if (error) {
          exitCode =
            typeof (error as any).code === "number"
              ? (error as any).code
              : (error as any).status || 1
          if (exitCode === 0) exitCode = 1
          errDetail =
            (error as any).killed && (error as any).signal === "SIGTERM"
              ? `bee CLI timed out after ${timeoutMs}ms`
              : error.message || String(error)
        }
        const stderrStr = String(stderr || "")
        resolve({
          stdout: String(stdout || ""),
          stderr: stderrStr.length > 0 ? stderrStr : errDetail,
          exitCode,
        })
      },
    )
    child.stdin?.end()
  })
}

