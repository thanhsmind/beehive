# pi-1-6-upgrade — CONTEXT

Owner ask (2026-11-24): pi is updated; make beehive run better on the new
pi. mise already holds pi 1.6.0, the npm latest. A 1.4.0 to 1.6.0 read of
the shipped CHANGELOG and docs found no belt break and no new extension API
that bee needs. Two defects surfaced in the live check and are the work.

Root causes:

- The belt's `session_shutdown` handler stops the broker timer but not the
  result-inbox drain timer. Pi invalidates every captured ctx about 3 s after
  `session_shutdown`; the drain interval then refreshes the in-flight
  workers widget through the stale ctx and prints the advisory "This
  extension ctx is stale after session replacement or reload" once per tick.
  It shows whenever pi lives past that window, for example after provider
  429 retries. Measured live with a probe extension on 1.6.0 and 1.4.0.
- `bee doctor`'s `runtime_pi` install hint names the retired
  `@mariozechner/pi-coding-agent`, whose newest version 0.73.1 is below the
  belt floor 0.84.4.

## Locked decisions

- D1 (`d9fa372a`). The `session_shutdown` handler calls `stopDrainTimer()`
  beside `stopBrokerTimer()` for every reason except `reload`. A relocation
  keeps delivery because the next `session_start` runs `startResultDrain`.
- D2 (`8344cfa9`). The install hint reads
  `npm install -g @earendil-works/pi-coding-agent`.
- D3 (`045dd245`). The belt header CEILING and the capability-audit page
  move to Pi 1.6.0, and the page gains a 1.4.0 to 1.6.0 audit section built
  from decision `6025a77e`.

## Out of scope

- Any use of the 1.5.0 `blocked` custom-UI option, `Usage.cost.totalFromProvider`
  or `pi.exec` `maxBuffer`: the belt draws no dialog, tracks no cost and
  spawns through node.
- The floor stays 0.84.4.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md | suspect | holds the version range and per-version audit |
| docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md | suspect | says the drain needs a live Pi session |

### Code

| Path | What it decides |
|------|-----------------|
| `.pi/extensions/bee-guard/events.ts` | the session_shutdown teardown |
| `.pi/extensions/bee-guard/result-inbox.ts` | startResultDrain and stopDrainTimer |
| `.pi/extensions/bee-guard/index.ts` | the belt header range |
| `packages/bee-rs/crates/bee/src/doctor.rs` | the runtime install hints |
