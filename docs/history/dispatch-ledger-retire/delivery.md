# dispatch-ledger-retire — delivery

## What changed

- `cap_cell_from_flags` in `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` calls `reconcile_capped_workers` once after the cell guard drops, so a capped cell's `workers[]` row reads `capped`. A failure warns on stderr and never fails the cap. Every cap door shares this function (D1).
- The prune keep set is unchanged. It already skips a worker whose cell file is capped, and keeps one whose cell file is missing or corrupt on purpose. A row-status skip was rejected because it would prune where that fail-open is intended (D1, plan).
- `bee dispatch wave --run` adds `empty_wave: true` with a one-line reason when at least one worker ran and every one returned nothing. A wave with zero rows is not flagged (D2).
- Each `bee dispatch wave` entry carries `rebuild`, the `bee dispatch prepare` command that rebuilds its payload (D3).
- `skills/bee-swarming/SKILL.md` states the `transport_ready` / `transport_reason` check. `bee herding status` prints a scope note: its line is the calling session's own pane, not worker availability, and it names the herdr and tmux variables (D4).
- The `budget-overrun` signal, its `estimate_minutes` reader, the `overrun-2x-estimate` counter and the prompt lines are gone from the supervisor (D5).
- Tests: capped-row keep set (dlr-1t), the wave rebuild command (dlr-2t), the closed signal sets and old-row skip on read (dlr-3t).
- Knowledge: `docs/knowledge/areas/workflow-state/dispatch.md`, `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` and `docs/knowledge/areas/bee-herding/presence-wake-reports-and-earned-autonomy.md`.

## Proof

- Red-first tests per cell; dlr-3t proved red with the signal re-added.
- dlr-5 ran the docs parity and pointer checks: no errors, only existing warnings.
- Cell verify for this cell: `.bee/bin/bee knowledge check`.

## Known limits

- Rows archived as `running` before this change stay `running`; the change is forward-only.
- `bee herding wave` and fleet `WaveResult` do not carry `empty_wave`.
- An estimate input for cells is a new PBI.
