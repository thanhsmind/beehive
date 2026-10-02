export const state: {
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
} = {
  cachedPreamble: null,
  preambleInjected: false,
  sessionInitRun: false,
  drainTimer: null,
  drainToken: null,
  drainDirectory: null,
  activeDrainCtx: null,
  drainInFlight: false,
  turnStartPending: false,
  selfBusy: false,
  activeTransition: false,
  transitionTeardownOccurred: false,
  cachedLimitsText: undefined,
  lastLimitsFetchTime: 0,
  lastLimitsModelKey: "",
  isFetchingLimits: false,
  dispatchCounter: 0,
  workerSteerDrainInFlight: false,
}
