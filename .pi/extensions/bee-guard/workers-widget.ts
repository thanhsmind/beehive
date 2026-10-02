import { readdirSync } from "node:fs"
import path from "node:path"
import { isDirectory, mainCheckoutRoot } from "./locate.ts"
import { carriedInboxTokens, loadRelocationCarry, readJsonObject } from "./result-inbox.ts"

export const IN_FLIGHT_WORKERS_WIDGET_KEY = "bee-workers"

/**
 * Derives the short suffix of a job id (e.g. "4153108-1" from "job-1789892858126-4153108-1",
 * or "100" from "job-100"). Used as fallback when marker carries neither seat nor cell_id.
 */
export function shortJobSuffix(jobId: string): string {
  const parts = jobId.split("-")
  if (parts.length > 2 && /^\d{10,}$/.test(parts[1])) {
    return parts.slice(2).join("-")
  }
  if (parts.length > 1) {
    return parts.slice(1).join("-")
  }
  return jobId.length > 8 ? jobId.slice(-8) : jobId
}

/**
 * Renders a row for one in-flight worker marker.
 * Rules:
 * - When marker carries seat and cell_id: "<seat> · <cell_id>"
 * - When marker carries seat only: "<seat>"
 * - When marker carries cell_id only: "<cell_id>"
 * - When neither field is present: fall back to the job id's short suffix
 * - Always prefixed with the progress tick glyph for in-flight work: "▸ " (D5)
 */
export function formatWorkerRow(marker: { seat?: string; cell_id?: string; job_id?: string }): string {
  const seat = typeof marker.seat === "string" ? marker.seat.trim() : ""
  const cellId = typeof marker.cell_id === "string" ? marker.cell_id.trim() : ""
  const jobId = typeof marker.job_id === "string" ? marker.job_id.trim() : ""

  let label = ""
  if (seat && cellId) {
    label = `${seat} · ${cellId}`
  } else if (seat) {
    label = seat
  } else if (cellId) {
    label = cellId
  } else if (jobId) {
    label = shortJobSuffix(jobId)
  } else {
    label = "worker"
  }

  return `▸ ${label}`
}

/**
 * Reads pending in-flight worker markers from .bee/result-inbox/<token>/ and carried tokens.
 * Advisory posture: an absent or unreadable inbox returns an empty list and never throws.
 */
export function getInFlightMarkers(
  directory: string,
  token: string,
): Array<{ job_id?: string; seat?: string; cell_id?: string }> {
  try {
    const mainRoot = mainCheckoutRoot(directory)
    loadRelocationCarry(mainRoot)

    const carried = carriedInboxTokens.get(token) ?? []
    const tokens = [token, ...carried.filter((t) => t !== token)]

    const candidates: Array<{ dir: string; name: string }> = []
    for (const tok of tokens) {
      const dir = path.join(mainRoot, ".bee", "result-inbox", tok)
      if (!isDirectory(dir)) continue
      let names: string[]
      try {
        names = readdirSync(dir)
      } catch {
        continue
      }
      for (const name of names) {
        if (name.endsWith(".json")) {
          candidates.push({ dir, name })
        }
      }
    }

    if (candidates.length === 0) return []

    candidates.sort((a, b) => a.name.localeCompare(b.name))

    const list: Array<{ job_id?: string; seat?: string; cell_id?: string }> = []
    for (const { dir, name } of candidates) {
      const markerPath = path.join(dir, name)
      const marker = readJsonObject(markerPath)
      if (!marker || typeof marker !== "object") continue
      const job_id = typeof marker.job_id === "string" ? marker.job_id : name.replace(/\.json$/, "")
      const seat = typeof marker.seat === "string" ? marker.seat : undefined
      const cell_id = typeof marker.cell_id === "string" ? marker.cell_id : undefined
      list.push({ job_id, seat, cell_id })
    }
    return list
  } catch {
    return []
  }
}

/**
 * Factory for Pi's ui.setWidget(<key>, factory, { placement: "belowEditor" }).
 * Returns a component with render(width) returning rows and invalidate().
 * Takes no input (D4).
 */
export function createInFlightWorkersWidget(rows: string[]) {
  const component = {
    render: (_width?: number) => rows,
    invalidate: () => {},
  }
  const factory = (_tui?: any, _theme?: any) => component
  ;(factory as any).lines = rows
  ;(factory as any).component = component
  return factory
}

/**
 * Updates the in-flight workers widget:
 * - If in-flight markers exist: sets widget with placement "belowEditor".
 * - If no in-flight markers exist: clears widget via setWidget(key, undefined) (D3).
 * - Advisory posture: never throws.
 */
export function refreshInFlightWorkersWidget(
  ctx: any,
  directory: string | null,
  token: string | null,
): void {
  try {
    if (!ctx?.ui || typeof ctx.ui.setWidget !== "function") return
    if (!directory || !token) {
      ctx.ui.setWidget(IN_FLIGHT_WORKERS_WIDGET_KEY, undefined)
      return
    }

    const markers = getInFlightMarkers(directory, token)
    if (markers.length === 0) {
      ctx.ui.setWidget(IN_FLIGHT_WORKERS_WIDGET_KEY, undefined)
      return
    }

    const rows = markers.map(formatWorkerRow)
    const factory = createInFlightWorkersWidget(rows)
    ctx.ui.setWidget(IN_FLIGHT_WORKERS_WIDGET_KEY, factory, { placement: "belowEditor" })
  } catch (err: any) {
    console.error(`bee in-flight-workers widget (advisory): ${err?.message ?? err}`)
  }
}

