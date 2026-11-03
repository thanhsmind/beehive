/** Pi's session_start reasons mapped onto the SessionStart `source` values
 * bee's session-init reads. Only "startup" and "clear" are in bee's
 * ADOPT_SOURCES (session_init.rs:64), so only a genuinely fresh session
 * boundary can adopt a handoff — a resume, a fork, and a reload never do,
 * which is exactly AGENTS.md's rule ("a resumed or compacted session never
 * adopts"). */
export function sessionSource(reason: string | undefined): string {
  switch (reason) {
    case "new":
      return "clear"
    case "resume":
    case "fork":
    case "reload":
      return "resume"
    default:
      return "startup"
  }
}

export function sessionIdOf(ctx: any): string | undefined {
  try {
    const id = ctx?.sessionManager?.getSessionId?.() ?? ctx?.sessionId
    return typeof id === "string" && id.length > 0 ? id : undefined
  } catch {
    return undefined
  }
}

export function directoryOf(ctx: any): string {
  const cwd = ctx?.cwd
  return typeof cwd === "string" && cwd.length > 0 ? cwd : process.cwd()
}

