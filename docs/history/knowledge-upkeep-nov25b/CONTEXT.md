# knowledge-upkeep-nov25b — brief

Lane: docs. Class: docs.

## Asked

Run the knowledge upkeep pass (`bee-knowledge-upkeep`) on the bundle.

## Found

`bee upkeep due --kind knowledge` reported a run due: 12 suspect concepts
loaded 5 or more times. No concept had a wrong report. The pass picked the
five suspect concepts loaded most that the last run did not check:
`areas/decision-memory/audit-log.md`, `areas/bee-herding/claude-session-transport.md`,
`areas/bee-herding/thin-peer-boot.md`,
`areas/doctrine-layer/situation-layer-class-index-session-group-and-parity-fence.md`,
`areas/workflow-state/close-usage-record.md`. One read-only reader per concept
checked each claim against the concept's own sources. All five drifted.

## Will write

Revise in place, inside `docs/knowledge/` only, the claims the sources prove
stale:

- `audit-log.md`: the closed kind set has 23 kinds; only `gate-bypass` and the
  `*-deferral` kinds are refused as decision tags and hidden from the doors.
- `claude-session-transport.md`: the thin flags in the spawn argv; any dispatch
  kind takes the Bash payload; the `steer_to` fallback name.
- `thin-peer-boot.md`: the headline says five env variables, as the body does.
- `situation-layer-...-parity-fence.md`: the compiled-in fallback is per class
  file; LEADER does not name the session index; the fallback line sits under
  the situations; the session list names the two autopilot rows.
- `close-usage-record.md`: close's own bookkeeping commit lands
  `.bee/usage/<feature>.json`; the worktree split stays an open gap.

Each revised concept gets `bee knowledge confirm`.

## Not in scope

Product code. The read-only worker-guard refused three plain `rg` searches
because their pattern text held a word it matches (`Paseo`,
`native_fallback_reason`, `uat door`); that defect is recorded as a mistake
for a bee fix, not fixed here.
