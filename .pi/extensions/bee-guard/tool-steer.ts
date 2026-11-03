import cp, { execFile } from "node:child_process"
import { readdirSync, readFileSync, renameSync, statSync } from "node:fs"
import path from "node:path"
import { isDirectory, mainCheckoutRoot, resolveBeeBinary } from "./locate.ts"
import { directoryOf } from "./session.ts"
import { beltEnv, state } from "./state.ts"

export function findRunningJobs(mailboxRoot: string): string[] {
  if (!isDirectory(mailboxRoot)) return []
  let entries: string[]
  try {
    entries = readdirSync(mailboxRoot)
  } catch {
    return []
  }
  const running: string[] = []
  for (const name of entries) {
    const jobDir = path.join(mailboxRoot, name)
    if (!isDirectory(jobDir)) continue
    let files: string[]
    try {
      files = readdirSync(jobDir)
    } catch {
      continue
    }
    let maxBriefRound = 0
    let maxResultRound = 0
    for (const f of files) {
      const mb = /^brief-(\d+)\.txt$/.exec(f)
      if (mb) {
        const r = Number.parseInt(mb[1], 10)
        if (Number.isFinite(r) && r > maxBriefRound) maxBriefRound = r
      }
      const mr = /^result-(\d+)\.json$/.exec(f)
      if (mr) {
        const r = Number.parseInt(mr[1], 10)
        if (Number.isFinite(r) && r > maxResultRound) maxResultRound = r
      }
    }
    if (maxBriefRound > 0 && maxBriefRound > maxResultRound) {
      running.push(name)
    }
  }
  running.sort()
  return running
}

export const BEE_STEER_TOOL_NAME = "bee_steer"

export const BEE_STEER_TOOL_PARAMETERS = {
  type: "object",
  properties: {
    job_id: {
      type: "string",
      description: "Optional ID of the running herding job to steer. If omitted, targets the single running job.",
    },
    text: {
      type: "string",
      description: "Steer guidance text to relay to the running worker.",
    },
  },
  required: ["text"],
}

export async function executeBeeSteerTool(
  _toolCallId: string,
  params: any,
  _signal?: any,
  _onUpdate?: any,
  ctx?: any,
) {
  if (!params || typeof params !== "object") {
    throw new Error("Parameters must be an object")
  }
  if (typeof params.text !== "string" || params.text.trim().length === 0) {
    throw new Error("Field 'text' must be a non-empty string")
  }
  const text = params.text

  const directory = directoryOf(ctx)
  const mainRoot = mainCheckoutRoot(directory)
  const mailboxRoot = path.join(mainRoot, ".bee", "mailbox")

  let jobId = typeof params.job_id === "string" ? params.job_id.trim() : ""
  if (!jobId) {
    const running = findRunningJobs(mailboxRoot)
    if (running.length === 0) {
      throw new Error("Zero running jobs found (no running jobs to steer)")
    }
    if (running.length > 1) {
      throw new Error(`Multiple running jobs found (${running.join(", ")}): specify job_id to steer`)
    }
    jobId = running[0]
  }

  const beeBinary = resolveBeeBinary(directory)
  if (!beeBinary) {
    throw new Error("bee_steer: bee binary not found in this project or its main worktree")
  }

  const result = await new Promise<{ stdout: string; stderr: string }>((resolve, reject) => {
    const child = cp.execFile(
      beeBinary,
      ["herding", "steer", jobId, "--text", text, "--json"],
      { cwd: directory, env: beltEnv() },
      (error, stdout, stderr) => {
        if (error) {
          reject(new Error(stderr?.toString()?.trim() || stdout?.toString()?.trim() || error.message))
        } else {
          resolve({ stdout: stdout.toString(), stderr: stderr.toString() })
        }
      },
    )
    child.stdin?.end()
  })

  return {
    content: [{ type: "text", text: result.stdout.trim() || `Steer relayed to worker ${jobId}` }],
    details: { job_id: jobId, text },
  }
}

export const beeSteerTool = {
  name: BEE_STEER_TOOL_NAME,
  label: BEE_STEER_TOOL_NAME,
  description: "Relay guidance text to a running bee worker.",
  promptSnippet: "Relay guidance text to a running worker",
  promptGuidelines: [
    "Use bee_steer to send mid-run corrections or extra context to an active worker.",
  ],
  parameters: BEE_STEER_TOOL_PARAMETERS,
  execute: executeBeeSteerTool,
}


export async function drainWorkerSteer(pi: any, directory: string): Promise<void> {
  const jobId = typeof process.env.BEE_HERDING_JOB_ID === "string" ? process.env.BEE_HERDING_JOB_ID.trim() : ""
  if (!jobId) return
  if (state.workerSteerDrainInFlight) return
  state.workerSteerDrainInFlight = true
  try {
    const mainRoot = mainCheckoutRoot(directory)
    const mailboxDir = path.join(mainRoot, ".bee", "mailbox", jobId)
    if (!isDirectory(mailboxDir)) return
    let names: string[]
    try {
      names = readdirSync(mailboxDir)
    } catch {
      return
    }
    const steerFiles: Array<{ name: string; n: number }> = []
    for (const name of names) {
      const m = /^steer-(\d+)\.json$/.exec(name)
      if (m) {
        steerFiles.push({ name, n: Number.parseInt(m[1], 10) })
      }
    }
    steerFiles.sort((a, b) => a.n - b.n)
    const prefix = "Steer from your leader (relayed mid-run; context only, the cell and its gates are unchanged):"
    for (const item of steerFiles) {
      const filePath = path.join(mailboxDir, item.name)
      try {
        const stat = statSync(filePath)
        if (stat.size > 8192) {
          console.error(`bee steer drain (advisory): skipping oversized steer file ${item.name} (${stat.size} bytes > 8192)`)
          continue
        }
        const raw = readFileSync(filePath, "utf8")
        let parsed: any
        try {
          parsed = JSON.parse(raw)
        } catch (err: any) {
          console.error(`bee steer drain (advisory): skipping malformed steer file ${item.name}: ${err?.message ?? err}`)
          continue
        }
        if (
          !parsed ||
          typeof parsed !== "object" ||
          typeof parsed.n !== "number" ||
          typeof parsed.text !== "string" ||
          typeof parsed.at !== "string"
        ) {
          console.error(`bee steer drain (advisory): skipping invalid steer file ${item.name} (missing n, text, or at)`)
          continue
        }
        const deliveredPath = `${filePath}.delivered`
        try {
          renameSync(filePath, deliveredPath)
        } catch {
          continue
        }
        await pi.sendUserMessage(`${prefix}\n\n${parsed.text}`, { deliverAs: "steer" })
      } catch (err: any) {
        console.error(`bee steer drain (advisory): failed to drain ${item.name}: ${err?.message ?? err}`)
      }
    }
  } finally {
    state.workerSteerDrainInFlight = false
  }
}

