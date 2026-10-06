import cp from "node:child_process"
import { existsSync, mkdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs"
import path from "node:path"
import { resolveBeeBinary } from "./locate.ts"
import { sendLeaderMessage } from "./result-inbox.ts"
import { state } from "./state.ts"

export const HEARTBEAT_NAME = "bee-leader"
export const DEFAULT_CRON = "*/30 * * * *"
export const STALE_MARKER_MS = 120000

export interface PaseoSettings {
  command: string
  cron: string
  broker_tick_secs: number
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
  const defaults: PaseoSettings = { command: "paseo", cron: DEFAULT_CRON, broker_tick_secs: 30 }
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
    const rawSecs = paseo?.broker_tick_secs
    const broker_tick_secs =
      typeof rawSecs === "number" && Number.isInteger(rawSecs) && rawSecs > 0
        ? rawSecs
        : 30
    return { command, cron, broker_tick_secs }
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
        let isStale = false
        try {
          const stat = statSync(marker)
          let hasScheduleId = false
          let scheduleId: string | null = null
          let markerCron: string | null = null
          let parsedMarker: Record<string, any> = {}
          try {
            const raw = readFileSync(marker, "utf8")
            const candidate = JSON.parse(raw)
            if (candidate && typeof candidate === "object" && !Array.isArray(candidate)) {
              parsedMarker = candidate
            }
            const rawId = parsedMarker.schedule_id ?? parsedMarker.id ?? parsedMarker.Id ?? parsedMarker.ID
            if (rawId !== undefined && rawId !== null && String(rawId).length > 0) {
              hasScheduleId = true
              scheduleId = String(rawId)
            }
            if (typeof parsedMarker.cron === "string") {
              markerCron = parsedMarker.cron
            }
          } catch {}
          if (hasScheduleId && scheduleId) {
            if (markerCron !== settings.cron) {
              try {
                await run(settings.command, ["heartbeat", "delete", scheduleId])
              } catch (delErr: any) {
                const reason = delErr?.message ? String(delErr.message) : String(delErr)
                parsedMarker.delete_failed = reason
                try {
                  writeFileSync(marker, JSON.stringify(parsedMarker, null, 2))
                } catch {}
                return { created: false, reason }
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
              const rawNewId = parsed.id ?? parsed.Id ?? parsed.ID
              if (rawNewId === undefined || rawNewId === null || String(rawNewId).length === 0) {
                throw new Error("missing schedule id in output")
              }
              const id = String(rawNewId)
              const markerData = {
                agent_id: agentId,
                schedule_id: id,
                cron: settings.cron,
                created_at: new Date().toISOString(),
              }
              writeFileSync(marker, JSON.stringify(markerData, null, 2))
              return { created: true, id }
            }
            return { created: false, reason: "exists" }
          }
          const ageMs = Date.now() - stat.mtimeMs
          if (ageMs < STALE_MARKER_MS) {
            return { created: false, reason: "exists" }
          }
          isStale = true
        } catch (innerErr: any) {
          if (innerErr?.message && innerErr.message !== "exists") {
            throw innerErr
          }
          return { created: false, reason: "exists" }
        }

        if (isStale) {
          try {
            unlinkSync(marker)
          } catch {}
          try {
            writeFileSync(marker, "{}", { flag: "wx" })
            createdExclusiveFile = true
          } catch (retryErr: any) {
            if (retryErr?.code === "EEXIST") {
              return { created: false, reason: "exists" }
            }
            return { created: false, reason: retryErr?.message ? String(retryErr.message) : String(retryErr) }
          }
        }
      } else {
        return { created: false, reason: err?.message ? String(err.message) : String(err) }
      }
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
    const base = err?.message ? String(err.message) : String(err)
    return { created: false, reason: `${base} — FIX: set herding.paseo.command to the npm @getpaseo/cli paseo binary` }
  }
}

export async function deleteHeartbeat(
  mainRoot: string,
  agentId: string,
  settings: PaseoSettings,
  run: (command: string, args: string[]) => Promise<string>,
): Promise<void> {
  try {
    const marker = heartbeatMarkerPath(mainRoot, agentId)
    if (!existsSync(marker)) {
      return
    }
    let parsed: Record<string, any> = {}
    try {
      const raw = readFileSync(marker, "utf8")
      const candidate = JSON.parse(raw)
      if (candidate && typeof candidate === "object" && !Array.isArray(candidate)) {
        parsed = candidate
      }
    } catch {
      return
    }
    const rawId = parsed.schedule_id ?? parsed.id ?? parsed.Id ?? parsed.ID
    if (rawId === undefined || rawId === null || String(rawId).length === 0) {
      try {
        unlinkSync(marker)
      } catch {}
      return
    }
    const scheduleId = String(rawId)
    try {
      await run(settings.command, ["heartbeat", "delete", scheduleId])
      try {
        unlinkSync(marker)
      } catch {}
    } catch (err: any) {
      parsed.delete_failed = err?.message ? String(err.message) : String(err)
      try {
        writeFileSync(marker, JSON.stringify(parsed, null, 2))
      } catch {}
    }
  } catch {}
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

export let tickRunning = false

export function setTickRunning(value: boolean): void {
  tickRunning = Boolean(value)
}

export function isTickRunning(): boolean {
  return tickRunning
}

export const BROKER_TIMER_SLOT = Symbol.for("bee.pi.broker-timer")
let activeBrokerTimer: ReturnType<typeof setInterval> | null = null
let pendingNewsText: string | null = null

export function stopBrokerTimer(): void {
  const scope = globalThis as any
  for (const timer of [activeBrokerTimer, scope[BROKER_TIMER_SLOT]]) {
    if (!timer) continue
    try {
      clearInterval(timer)
    } catch {}
  }
  activeBrokerTimer = null
  scope[BROKER_TIMER_SLOT] = null
  pendingNewsText = null
}

export interface BrokerTimerDeps {
  run?: (command: string, args: string[], options?: any) => Promise<string>
  sendLeaderMessage?: (pi: any, text: string) => Promise<boolean>
  state?: { turnStartPending: boolean; selfBusy?: boolean }
}

export function startBrokerTimer(
  pi: any,
  directory: string,
  settings?: PaseoSettings,
  deps?: BrokerTimerDeps | ((command: string, args: string[], options?: any) => Promise<string>),
): void {
  stopBrokerTimer()
  if (!directory) {
    return
  }
  const resolvedSettings = settings ?? readPaseoSettings(directory)
  const intervalSecs =
    typeof resolvedSettings?.broker_tick_secs === "number" &&
    Number.isInteger(resolvedSettings.broker_tick_secs) &&
    resolvedSettings.broker_tick_secs > 0
      ? resolvedSettings.broker_tick_secs
      : 30
  const intervalMs = intervalSecs * 1000

  const runFn =
    typeof deps === "function"
      ? deps
      : typeof deps?.run === "function"
        ? deps.run
        : null

  const sendFn =
    typeof (deps as BrokerTimerDeps)?.sendLeaderMessage === "function"
      ? (deps as BrokerTimerDeps).sendLeaderMessage!
      : sendLeaderMessage

  const stateRef = (deps as BrokerTimerDeps)?.state ?? state

  const timer = setInterval(() => {
    if (tickRunning) {
      return
    }
    setTickRunning(true)
    void Promise.resolve()
      .then(async () => {
        const turnPending = Boolean(stateRef.turnStartPending && !stateRef.selfBusy)
        if (pendingNewsText && !turnPending) {
          const toSend = pendingNewsText
          pendingNewsText = null
          await sendFn(pi, toSend)
        }
        let stdout = ""
        if (runFn) {
          stdout = await runFn("bee", ["herding", "broker", "tick", "--json"], {
            cwd: directory,
            timeout: 120000,
          })
        } else {
          const beeBinary = resolveBeeBinary(directory)
          if (!beeBinary) {
            return
          }
          stdout = await new Promise<string>((resolve, reject) => {
            const child = cp.execFile(
              beeBinary,
              ["herding", "broker", "tick", "--json"],
              { cwd: directory, timeout: 120000, encoding: "utf8" },
              (error, out) => {
                if (error) {
                  reject(error)
                } else {
                  resolve(String(out ?? ""))
                }
              },
            )
            child.stdin?.on("error", () => {})
            child.stdin?.end()
          })
        }
        const tick = parseTick(stdout)
        if (tick && tick.news) {
          const text = heartbeatText(tick)
          const nowPending = Boolean(stateRef.turnStartPending && !stateRef.selfBusy)
          if (nowPending) {
            pendingNewsText = text
          } else {
            await sendFn(pi, text)
          }
        }
      })
      .catch((err: any) => {
        console.error(`bee broker tick (timer): ${err?.message ?? err}`)
      })
      .finally(() => {
        setTickRunning(false)
      })
  }, intervalMs)

  if (typeof (timer as any)?.unref === "function") {
    ;(timer as any).unref()
  }
  activeBrokerTimer = timer
  ;(globalThis as any)[BROKER_TIMER_SLOT] = timer
}
