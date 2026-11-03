import fs from "node:fs"
import os from "node:os"
import path from "node:path"
import { state } from "./state.ts"

// ─── active-branch model usage statusline ──────────────────────────────────

export function formatTokens(n: number): string {
  if (n >= 1e6) {
    return (n / 1e6).toFixed(1) + "m"
  }
  if (n >= 1e3) {
    return Math.round(n / 1e3) + "k"
  }
  return String(Math.round(n))
}

export function positiveNum(val: unknown): number {
  return typeof val === "number" && Number.isFinite(val) && val > 0 ? val : 0
}

export interface ModelUsageSummary {
  provider: string
  model: string
  newTokens: number
  cachedTokens: number
}

export function aggregateModelUsage(branch: unknown): ModelUsageSummary[] {
  if (!Array.isArray(branch)) return []
  const map = new Map<string, ModelUsageSummary>()

  for (const entry of branch) {
    if (!entry || typeof entry !== "object") continue
    const msg = (entry as any).message && typeof (entry as any).message === "object"
      ? (entry as any).message
      : entry

    if (msg.role !== "assistant") continue

    const provider = typeof msg.provider === "string" ? msg.provider.trim() : ""
    const model = typeof msg.model === "string" ? msg.model.trim() : ""
    if (!provider || !model) continue

    const usage = msg.usage
    let input = 0
    let output = 0
    let cacheWrite = 0
    let cacheRead = 0

    if (usage && typeof usage === "object") {
      input = positiveNum(usage.input ?? usage.input_tokens ?? usage.inputTokens)
      output = positiveNum(usage.output ?? usage.output_tokens ?? usage.outputTokens)
      cacheWrite = positiveNum(
        usage.cacheWrite ??
        usage.cache_write ??
        usage.cacheCreationInputTokens ??
        usage.cache_creation_input_tokens,
      )
      if (usage.cache_creation && typeof usage.cache_creation === "object") {
        cacheWrite += positiveNum(usage.cache_creation.ephemeral_5m_input_tokens)
        cacheWrite += positiveNum(usage.cache_creation.ephemeral_1h_input_tokens)
      }
      cacheRead = positiveNum(
        usage.cacheRead ??
        usage.cache_read ??
        usage.cacheReadInputTokens ??
        usage.cache_read_input_tokens,
      )
    }

    const newTokens = input + output + cacheWrite
    const cachedTokens = cacheRead

    const key = `${provider}/${model}`
    const existing = map.get(key)
    if (existing) {
      existing.newTokens += newTokens
      existing.cachedTokens += cachedTokens
    } else {
      map.set(key, {
        provider,
        model,
        newTokens,
        cachedTokens,
      })
    }
  }

  return Array.from(map.values()).filter(
    (row) => row.newTokens > 0 || row.cachedTokens > 0,
  )
}

export function formatModelUsage(summaries: ModelUsageSummary[], limitsText?: string): string | undefined {
  const usageText = summaries.length > 0
    ? summaries
        .map(
          (s) =>
            `${s.provider}/${s.model} ${formatTokens(s.newTokens)} new/${formatTokens(s.cachedTokens)} cached`,
        )
        .join(" · ")
    : undefined

  if (usageText && limitsText) return `${usageText} · ${limitsText}`
  return usageText || limitsText
}

export const MODEL_USAGE_STATUS_KEY = "model-usage"
export const LIMITS_CACHE_TTL_MS = 45_000


export function formatRemainingTime(resetTime?: string): string | null {
  if (!resetTime) return null
  const ts = Date.parse(resetTime)
  if (!Number.isFinite(ts)) return null
  const delta = ts - Date.now()
  if (delta <= 0) return "now"

  const totalMin = Math.round(delta / 60_000)
  const days = Math.floor(totalMin / (60 * 24))
  const hours = Math.floor((totalMin % (60 * 24)) / 60)
  const mins = totalMin % 60

  if (days > 0) return `${days}d${hours > 0 ? ` ${hours}h` : ""}`
  if (hours > 0) return `${hours}h${mins > 0 ? ` ${mins}m` : ""}`
  return `${mins}m`
}

export function getAntigravityAuth(): { token: string; projectId?: string } | null {
  try {
    const authPath = path.join(os.homedir(), ".pi", "agent", "auth.json")
    if (!fs.existsSync(authPath)) return null
    const raw = fs.readFileSync(authPath, "utf8")
    const auth = JSON.parse(raw)
    const cred = auth.antigravity
    if (!cred || !cred.access) return null
    return { token: cred.access, projectId: cred.projectId }
  } catch {
    return null
  }
}

export async function fetchAntigravityLimits(modelId: string): Promise<string | undefined> {
  const creds = getAntigravityAuth()
  if (!creds) return undefined

  try {
    const modPath = path.join(
      os.homedir(),
      ".pi",
      "agent",
      "npm",
      "node_modules",
      "@heyhuynhgiabuu",
      "pi-oauth-antigravity",
      "dist",
      "usage",
      "usage.js",
    )
    if (fs.existsSync(modPath)) {
      const { fetchAccountUsage } = require(modPath)
      const rawKey = JSON.stringify({ token: creds.token, projectId: creds.projectId })
      const usage = await fetchAccountUsage(rawKey)
      if (usage && Array.isArray(usage.groups) && usage.groups.length > 0) {
        const isGemini = /gemini/i.test(modelId)
        let group = usage.groups.find((g: any) =>
          isGemini ? /gemini/i.test(g.displayName) : /claude|gpt/i.test(g.displayName),
        )
        if (!group) group = usage.groups[0]

        const buckets = Array.isArray(group.buckets) ? group.buckets : []
        const b5h = buckets.find(
          (b: any) => b.window === "5h" || /5\s*h/i.test(b.displayName) || /5h/i.test(b.bucketId),
        )
        const bwk = buckets.find(
          (b: any) =>
            b.window === "weekly" || /week/i.test(b.displayName) || /week/i.test(b.bucketId),
        )

        const parts: string[] = []
        if (b5h && typeof b5h.remainingFraction === "number") {
          const pct = Math.round(b5h.remainingFraction * 100)
          const reset = b5h.remainingFraction < 1 ? formatRemainingTime(b5h.resetTime) : null
          parts.push(`5h ${pct}%${reset ? ` (${reset})` : ""}`)
        }
        if (bwk && typeof bwk.remainingFraction === "number") {
          const pct = Math.round(bwk.remainingFraction * 100)
          const reset = bwk.remainingFraction < 1 ? formatRemainingTime(bwk.resetTime) : null
          parts.push(`wk ${pct}%${reset ? ` (${reset})` : ""}`)
        }

        if (parts.length > 0) return parts.join(" · ")
      }
    }
  } catch {
    // Non-fatal advisory check
  }
  return undefined
}

export function refreshModelUsageStatus(ctx: any): void {
  try {
    if (!ctx?.ui || typeof ctx.ui.setStatus !== "function") return
    if (!ctx?.sessionManager || typeof ctx.sessionManager.getBranch !== "function") return
    const branch = ctx.sessionManager.getBranch()
    const summaries = aggregateModelUsage(branch)
    const initialText = formatModelUsage(summaries, state.cachedLimitsText)
    ctx.ui.setStatus(MODEL_USAGE_STATUS_KEY, initialText)

    const model = ctx.model
    const provider = typeof model?.provider === "string" ? model.provider.trim() : ""
    const modelId = typeof model?.id === "string" ? model.id.trim() : ""

    if (provider === "antigravity") {
      const now = Date.now()
      const key = `${provider}:${modelId}`
      if (key !== state.lastLimitsModelKey || now - state.lastLimitsFetchTime >= LIMITS_CACHE_TTL_MS) {
        if (!state.isFetchingLimits) {
          state.isFetchingLimits = true
          fetchAntigravityLimits(modelId)
            .then((freshLimits) => {
              state.cachedLimitsText = freshLimits
              state.lastLimitsFetchTime = Date.now()
              state.lastLimitsModelKey = key
              const updatedText = formatModelUsage(summaries, state.cachedLimitsText)
              ctx.ui.setStatus(MODEL_USAGE_STATUS_KEY, updatedText)
            })
            .catch(() => {})
            .finally(() => {
              state.isFetchingLimits = false
            })
        }
      }
    } else if (state.cachedLimitsText !== undefined) {
      state.cachedLimitsText = undefined
      state.lastLimitsModelKey = ""
      const updatedText = formatModelUsage(summaries, undefined)
      ctx.ui.setStatus(MODEL_USAGE_STATUS_KEY, updatedText)
    }
  } catch (err: any) {
    console.error(`bee model-usage status (advisory): ${err?.message ?? err}`)
  }
}

