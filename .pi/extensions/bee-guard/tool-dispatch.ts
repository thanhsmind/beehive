import { spawn } from "node:child_process"
import { resolveBeeBinary } from "./locate.ts"
import { state } from "./state.ts"
import { directoryOf, sessionIdOf } from "./session.ts"
import { usableInboxToken } from "./result-inbox.ts"
import { execBeeCli } from "./bee-cli.ts"
import { tokenizeArgv } from "./transition.ts"

export const HERDING_RUN_ARGV = [".bee/bin/bee", "herding", "run"]

export function flagArgs(params: any, keys: string[]): string[] {
  return keys.flatMap((key) => {
    const val = params?.[key]
    if (typeof val === "boolean") {
      return val ? [`--${key}`] : []
    }
    if (typeof val === "string" && val.length > 0) {
      return [`--${key}`, val]
    }
    return []
  })
}

export function notifySafely(ctx: any, message: string): void {
  try {
    ctx?.ui?.notify?.(message, "error")
  } catch {}
}

export async function runBeeDispatch(prepareArgs: string[], ctx: any) {
  const directory = directoryOf(ctx)
  const token = usableInboxToken(sessionIdOf(ctx))
  if (!token) {
    throw new Error("bee_dispatch needs this session's id to deliver the worker's result, and Pi gave none.")
  }
  const prepared = await execBeeCli(directory, ["dispatch", "prepare", "--runtime", "pi", ...prepareArgs, "--json"], token)
  const answer = prepared.stdout.trim()
  if (prepared.exitCode !== 0) throw new Error(prepared.stderr.trim() || answer)
  let parsed: any = null
  let argv: string[] = []
  try {
    parsed = JSON.parse(answer)
    if (parsed?.tool === "Bash") argv = tokenizeArgv(parsed.payload.command)
  } catch {}
  if (parsed?.ok === false) throw new Error(answer)
  if (!HERDING_RUN_ARGV.every((word, i) => argv[i] === word)) {
    throw new Error(`bee_dispatch refused: bee dispatch prepare did not return a \`${HERDING_RUN_ARGV.join(" ")}\` command — ${answer}`)
  }
  const beeBinary = resolveBeeBinary(directory)
  if (!beeBinary) throw new Error("bee_dispatch: bee binary not found in this project or its main worktree")
  state.dispatchCounter += 1
  const jobId = `job-${Date.now()}-${process.pid}-${state.dispatchCounter}`
  const child = spawn(beeBinary, [...argv.slice(1), "--inbox-session", token, "--job-id", jobId], {
    cwd: directory,
    detached: true,
    stdio: ["pipe", "ignore", "ignore"],
  })
  child.on("error", (err) => notifySafely(ctx, `bee_dispatch job ${jobId} did not start: ${err.message}`))
  const isCellJob = prepareArgs.includes("--cell") || (prepareArgs.indexOf("--kind") !== -1 && prepareArgs[prepareArgs.indexOf("--kind") + 1] === "cell")
  child.on("exit", (code) => {
    if (code) {
      const cellNote = isCellJob ? "; a done cell exits non-zero until the leader caps it." : "."
      notifySafely(ctx, `bee_dispatch job ${jobId} exited with code ${code}. Its result, if any, arrives in this session${cellNote}`)
    }
  })
  child.stdin?.on("error", () => {})
  child.stdin?.end(typeof parsed.payload.stdin === "string" ? parsed.payload.stdin : "")
  child.unref()
  return {
    content: [
      {
        type: "text",
        text: `Worker started (job ${jobId}). Its result comes back to this session when it finishes — keep working.`,
      },
    ],
    details: { job_id: jobId, outcome: "started" },
  }
}

export const beeDispatchTool = {
  name: "bee_dispatch",
  label: "bee_dispatch",
  description:
    "Start a bee worker for a claimed cell or a gather, reviewer or advisor job. Returns a job id at once; the worker's result arrives in this session when it finishes.",
  promptSnippet: "Hand a claimed cell or a side job to a bee worker",
  promptGuidelines: [
    "Use bee_dispatch to hand a claimed cell to a worker instead of writing its files yourself.",
    "A refusal from bee_dispatch is bee's answer; read its fix before you call again.",
  ],
  parameters: {
    type: "object",
    properties: {
      kind: { type: "string", enum: ["cell", "gather", "reviewer", "advisor"], description: "What the worker is for" },
      role: { type: "string", description: "Optional team role that names the job" },
      cell: { type: "string", description: "Cell id, required when kind is cell" },
      worker: { type: "string", description: "The worker name that holds the cell's claim, required when kind is cell" },
      purpose: { type: "string", description: "One line on what a non-cell job is for" },
      stage: { type: "string", description: "Optional lifecycle stage for the worker" },
      feature: { type: "string", description: "Optional feature name" },
      expertise: { type: "string", description: "Optional worker expertise or track" },
      claim: { type: "boolean", description: "Optional flag to claim the cell during dispatch" },
    },
    required: ["kind"],
  },
  execute: (_toolCallId: string, params: any, _signal?: any, _onUpdate?: any, ctx?: any) =>
    runBeeDispatch(flagArgs(params, ["kind", "role", "cell", "worker", "purpose", "stage", "feature", "expertise", "claim"]), ctx),
}

export const beeAdvisorTool = {
  name: "bee_advisor",
  label: "bee_advisor",
  description:
    "Ask a bee advisor seat a question. Returns a job id at once; the advisor's answer arrives in this session when it finishes.",
  promptSnippet: "Consult a bee advisor seat",
  promptGuidelines: ["Use bee_advisor when the workflow asks for an advisor consult or a hat seat."],
  parameters: {
    type: "object",
    properties: {
      role: { type: "string", description: "Advisor seat, for example advisor or hat-risks" },
      purpose: { type: "string", description: "One line on what the consult is for" },
      stage: { type: "string", description: "Optional lifecycle stage" },
      feature: { type: "string", description: "Optional feature name" },
    },
    required: ["role", "purpose"],
  },
  execute: (_toolCallId: string, params: any, _signal?: any, _onUpdate?: any, ctx?: any) =>
    runBeeDispatch(["--kind", "advisor", ...flagArgs(params, ["role", "purpose", "stage", "feature"])], ctx),
}

