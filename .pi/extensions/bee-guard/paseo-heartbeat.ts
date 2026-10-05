import { existsSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from "node:fs"
import path from "node:path"

export const HEARTBEAT_NAME = "bee-leader"
export const DEFAULT_CRON = "*/5 * * * *"

export interface PaseoSettings {
  command: string
  cron: string
}

export interface TickResult {
  claimed: number
  notices_sent: number
  news: boolean
}

export type EnsureHeartbeatResult =
  | { created: true; id: string }
  | { created: false; reason: string }

function extractFirstJsonObject(text: string): Record<string, any> | null {
  if (typeof text !== "string") {
    return null
  }
  for (let s = 0; s < text.length; s++) {
    if (text[s] !== "{") {
      continue
    }
    let depth = 0
    let inString = false
    let escape = false
    for (let i = s; i < text.length; i++) {
      const char = text[i]
      if (escape) {
        escape = false
        continue
      }
      if (char === "\\") {
        if (inString) {
          escape = true
        }
        continue
      }
      if (char === '"') {
        inString = !inString
        continue
      }
      if (!inString) {
        if (char === "{") {
          depth++
        } else if (char === "}") {
          depth--
          if (depth === 0) {
            try {
              const candidate = text.slice(s, i + 1)
              const parsed = JSON.parse(candidate)
              if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
                return parsed
              }
            } catch {}
            break
          }
        }
      }
    }
  }
  return null
}

export function isPaseoLeader(env: Record<string, string | undefined> = process.env): boolean {
  if (!env || typeof env !== "object") {
    return false
  }
  const agentId = env.PASEO_AGENT_ID
  if (typeof agentId !== "string" || agentId.length === 0) {
    return false
  }
  const worker = env.BEE_HERDING_WORKER
  if (typeof worker === "string" && worker.length > 0) {
    return false
  }
  return true
}

export function isHeartbeatPrompt(text: unknown): boolean {
  if (typeof text !== "string") {
    return false
  }
  const trimmed = text.trimStart()
  return trimmed.startsWith("<paseo-system>") && trimmed.includes('Schedule "bee-leader" fired')
}

export function readPaseoSettings(mainRoot: string): PaseoSettings {
  const defaults: PaseoSettings = { command: "paseo", cron: DEFAULT_CRON }
  try {
    const configPath = path.join(mainRoot, ".bee", "config.json")
    if (!existsSync(configPath)) {
      return defaults
    }
    const raw = readFileSync(configPath, "utf8")
    const parsed = JSON.parse(raw)
    const paseo = parsed?.herding?.paseo
    const command = typeof paseo?.command === "string" && paseo.command.length > 0 ? paseo.command : "paseo"
    const cron = typeof paseo?.heartbeat_cron === "string" && paseo.heartbeat_cron.length > 0 ? paseo.heartbeat_cron : DEFAULT_CRON
    return { command, cron }
  } catch {
    return defaults
  }
}

export function heartbeatMarkerPath(mainRoot: string, agentId: string): string {
  return path.join(mainRoot, ".bee", "runtime", "paseo-heartbeat", `${agentId}.json`)
}

export async function ensureHeartbeat(
  mainRoot: string,
  agentId: string,
  settings: PaseoSettings,
  run: (command: string, args: string[]) => Promise<string>,
): Promise<EnsureHeartbeatResult> {
  let createdExclusiveFile = false
  let marker = ""
  try {
    marker = heartbeatMarkerPath(mainRoot, agentId)
    const markerDir = path.dirname(marker)
    mkdirSync(markerDir, { recursive: true })
    try {
      writeFileSync(marker, "{}", { flag: "wx" })
      createdExclusiveFile = true
    } catch (err: any) {
      if (err?.code === "EEXIST") {
        return { created: false, reason: "exists" }
      }
      return { created: false, reason: err?.message ? String(err.message) : String(err) }
    }

    const stdout = await run(settings.command, [
      "heartbeat",
      "create",
      "--cron",
      settings.cron,
      "--name",
      HEARTBEAT_NAME,
      "--json",
      "bee heartbeat",
    ])
    const parsed = extractFirstJsonObject(stdout)
    if (!parsed) {
      throw new Error("unparseable stdout")
    }
    const rawId = parsed.id ?? parsed.Id ?? parsed.ID
    if (rawId === undefined || rawId === null || String(rawId).length === 0) {
      throw new Error("missing schedule id in output")
    }
    const id = String(rawId)
    const markerData = {
      agent_id: agentId,
      schedule_id: id,
      cron: settings.cron,
      created_at: new Date().toISOString(),
    }
    writeFileSync(marker, JSON.stringify(markerData, null, 2))
    return { created: true, id }
  } catch (err: any) {
    if (createdExclusiveFile && marker) {
      try {
        if (existsSync(marker)) {
          unlinkSync(marker)
        }
      } catch {}
    }
    return { created: false, reason: err?.message ? String(err.message) : String(err) }
  }
}

export function parseTick(stdout: unknown): TickResult | null {
  if (typeof stdout !== "string") {
    return null
  }
  const parsed = extractFirstJsonObject(stdout)
  if (!parsed) {
    return null
  }
  const claimed = typeof parsed.claimed === "number" && Number.isFinite(parsed.claimed) ? parsed.claimed : 0
  const notices_sent = typeof parsed.notices_sent === "number" && Number.isFinite(parsed.notices_sent) ? parsed.notices_sent : 0
  return {
    claimed,
    notices_sent,
    news: claimed > 0 || notices_sent > 0,
  }
}

export function heartbeatText(tick: TickResult | null | undefined): string {
  if (tick && tick.news) {
    const claimed = typeof tick.claimed === "number" ? tick.claimed : 0
    const notices_sent = typeof tick.notices_sent === "number" ? tick.notices_sent : 0
    return `bee heartbeat: the broker routed ${claimed} question(s) and sent ${notices_sent} notice(s). Read them with bee orient and act.`
  }
  return "bee heartbeat: no news. Reply with the single word ok and do nothing else."
}
