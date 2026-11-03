// ─── advisory session state (process-lifetime only, never persisted) ───────

export interface BeltState {
  cachedPreamble: string | null
  preambleInjected: boolean
  sessionInitRun: boolean
  drainTimer: ReturnType<typeof setInterval> | null
  drainToken: string | null
  drainDirectory: string | null
  activeDrainCtx: any
  drainInFlight: boolean
  turnStartPending: boolean
  selfBusy: boolean
  activeTransition: boolean
  transitionTeardownOccurred: boolean
  cachedLimitsText: string | undefined
  lastLimitsFetchTime: number
  lastLimitsModelKey: string
  isFetchingLimits: boolean
  dispatchCounter: number
  workerSteerDrainInFlight: boolean
  inFlightClaims: Set<string>
  forcedContinuationSessions: Set<string>
  settleOutcomes: Map<string, string>
  heldClaims: Set<string>
  promptDepths: Map<string, number>
  paseoLeader: boolean | null
  paseoAgentId: string | null
}

export const STATE_SLOT = Symbol.for("bee.pi.state")

const defaults = (): BeltState => ({
  // D8 (CONTEXT.md): the full session preamble is fetched ONCE per session_start
  // and injected ONCE, on the first turn of the session; every turn after that
  // carries only `bee hook prompt-context`'s own per-turn delta. `/reload` fires
  // a second session_start with reason "reload" against the SAME session — it
  // must not re-run session-init (which registers the acting session and may
  // adopt a handoff already claimed), so a reload keeps whatever this instance
  // already fetched. A genuinely new session (`new`/`resume`/`fork`) resets the
  // pair, because it IS a new session.
  cachedPreamble: null,
  preambleInjected: false,
  sessionInitRun: false,
  /** The timer, its session token and its directory. Module-lifetime only — the
   * inbox on disk is the state that survives, never these. */
  drainTimer: null,
  drainToken: null,
  drainDirectory: null,
  activeDrainCtx: null,
  /** Re-entrancy guard: a tick that is still awaiting an injection never starts a
   * second one. */
  drainInFlight: false,
  /** F1 (pi-peer service.ts:233-236): set BEFORE a non-steer injection and
   * released at `before_agent_start`, so a burst cannot open two overlapping
   * plain user turns in the gap before the host starts the first one. */
  turnStartPending: false,
  /** Busy state, own-session only: `before_agent_start` sets it, `agent_settled`
   * clears it, and a `session_start` resets it — a missed settle can never wedge
   * delivery. */
  selfBusy: false,
  activeTransition: false,
  transitionTeardownOccurred: false,
  cachedLimitsText: undefined,
  lastLimitsFetchTime: 0,
  lastLimitsModelKey: "",
  isFetchingLimits: false,
  dispatchCounter: 0,
  workerSteerDrainInFlight: false,
  inFlightClaims: new Set(),
  forcedContinuationSessions: new Set(),
  settleOutcomes: new Map(),
  heldClaims: new Set(),
  promptDepths: new Map(),
  paseoLeader: null,
  paseoAgentId: null,
})

const slot = ((globalThis as any)[STATE_SLOT] ??= {}) as Partial<BeltState>
for (const [key, value] of Object.entries(defaults())) {
  if (!(key in slot)) (slot as any)[key] = value
}

export const state = slot as BeltState

export function beltEnv(): NodeJS.ProcessEnv {
  return state.paseoLeader && state.paseoAgentId
    ? { ...process.env, PASEO_AGENT_ID: state.paseoAgentId }
    : { ...process.env }
}
