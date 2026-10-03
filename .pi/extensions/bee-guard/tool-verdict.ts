import { existsSync, readdirSync, renameSync, writeFileSync } from "node:fs"
import path from "node:path"
import { candidateRoots, isDirectory, mainCheckoutRoot } from "./locate.ts"
import { directoryOf } from "./session.ts"

// ─── verdict terminating tool (D6 amended by 6b7e8f49) ──────────────────────

/** Resolves the bee store directory (.bee) across candidateRoots. */
export function resolveBeeStore(directory: string): string | null {
  for (const root of candidateRoots(directory)) {
    const candidate = path.join(root, ".bee")
    if (isDirectory(candidate)) return candidate
  }
  return null
}

export const VERDICT_TOOL_NAME = "verdict"

export const VERDICT_TOOL_PARAMETERS = {
  type: "object",
  properties: {
    status: {
      type: "string",
      enum: ["done", "blocked"],
      description: "Final status of the work: done or blocked",
    },
    summary: {
      type: "string",
      description: "One-line summary of what happened or why the worker blocked",
    },
    files_changed: {
      type: "array",
      items: { type: "string" },
      description: "Paths of files changed or created",
    },
    proof: {
      type: "string",
      description: "Command or evidence backing the outcome",
    },
    options: {
      type: "array",
      items: { type: "string" },
      description: "Optional ways forward when blocked with a choice",
    },
    leaning: {
      type: "string",
      description: "Optional preferred option, repeated word for word",
    },
    report_path: {
      type: "string",
      description: "Optional path to the full report markdown file",
    },
    dissent: {
      type: "object",
      properties: {
        claim: { type: "string" },
        alternative: { type: "string" },
        severity: { type: "string", enum: ["blocker", "consider"] },
      },
      required: ["claim", "alternative", "severity"],
      description: "Optional structured disagreement with the task",
    },
  },
  required: ["status", "summary", "files_changed", "proof"],
}

export async function executeVerdictTool(
  _toolCallId: string,
  params: any,
  _signal?: any,
  _onUpdate?: any,
  ctx?: any,
) {
  if (!params || typeof params !== "object") {
    throw new Error("Verdict parameters must be an object")
  }
  const required = ["status", "summary", "files_changed", "proof"]
  for (const field of required) {
    if (params[field] === undefined || params[field] === null) {
      throw new Error(`Missing required field: ${field}`)
    }
  }
  if (params.status !== "done" && params.status !== "blocked") {
    throw new Error(`Invalid status: ${params.status} (expected 'done' or 'blocked')`)
  }
  if (typeof params.summary !== "string") {
    throw new Error("Field 'summary' must be a string")
  }
  if (!Array.isArray(params.files_changed)) {
    throw new Error("Field 'files_changed' must be an array of strings")
  }
  if (typeof params.proof !== "string") {
    throw new Error("Field 'proof' must be a string")
  }
  if (params.status === "done" && params.proof.trim().length === 0) {
    throw new Error("Verdict with status 'done' requires proof. Fix: run the proof and pass `<command> — <result> — <scope reason>`.")
  }

  const directory = directoryOf(ctx)
  const mainRoot = mainCheckoutRoot(directory)
  const store = path.join(mainRoot, ".bee")
  if (!isDirectory(store)) {
    throw new Error("No .bee store found to record verdict")
  }

  const jobId = typeof process.env.BEE_HERDING_JOB_ID === "string" ? process.env.BEE_HERDING_JOB_ID.trim() : ""
  if (!jobId) {
    throw new Error("The verdict tool works only inside a bee herding job. Fix: finish with a normal final message.")
  }

  const mailboxRoot = path.join(store, "mailbox")
  const mailboxDir = path.join(mailboxRoot, jobId)
  if (!isDirectory(mailboxDir)) {
    throw new Error(`Job mailbox directory not found for job: ${jobId}`)
  }

  let round = 1
  try {
    const names = readdirSync(mailboxDir)
    let maxRound = 0
    for (const name of names) {
      const m = /^(?:result|brief|ack)-(\d+)\.(?:json|txt)$/.exec(name)
      if (m) {
        const r = Number.parseInt(m[1], 10)
        if (Number.isFinite(r) && r > maxRound) maxRound = r
      }
    }
    if (maxRound > 0) round = maxRound
  } catch {}

  const finalFile = path.join(mailboxDir, `result-${round}.json`)
  if (existsSync(finalFile)) {
    throw new Error(`result-${round}.json already exists in the job mailbox. Fix: the result for this round is already recorded, end the turn.`)
  }

  const resultPayload: Record<string, unknown> = {
    status: params.status,
    summary: params.summary,
    files_changed: params.files_changed,
    proof: params.proof,
  }
  if (Array.isArray(params.options)) {
    resultPayload.options = params.options
  }
  if (typeof params.leaning === "string" && params.leaning.length > 0) {
    resultPayload.leaning = params.leaning
  }
  if (typeof params.report_path === "string" && params.report_path.length > 0) {
    resultPayload.report_path = params.report_path
  }
  if (params.dissent && typeof params.dissent === "object") {
    resultPayload.dissent = params.dissent
  }

  const tmpFile = path.join(mailboxDir, `result-${round}.json.tmp`)
  try {
    writeFileSync(tmpFile, JSON.stringify(resultPayload, null, 2) + "\n", "utf8")
    renameSync(tmpFile, finalFile)
  } catch (err: any) {
    throw new Error(`Failed to write verdict result-${round}.json: ${err?.message ?? err}`)
  }

  return {
    content: [{ type: "text", text: `Verdict recorded: ${params.status} (${params.summary})` }],
    details: resultPayload,
    terminate: true,
  }
}

export const verdictTool = {
  name: VERDICT_TOOL_NAME,
  label: VERDICT_TOOL_NAME,
  description:
    "Record the structured worker verdict (status, summary, files_changed, proof) and conclude execution.",
  promptSnippet: "Emit a final structured verdict to conclude execution",
  promptGuidelines: [
    "Use verdict as your final action when finishing or blocking on assigned work.",
    "After calling verdict, do not emit another assistant response in the same turn.",
  ],
  parameters: VERDICT_TOOL_PARAMETERS,
  execute: executeVerdictTool,
}

