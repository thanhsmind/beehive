# Context: release-red-2-76-0

The 2.76.0 release stopped at its test gate. `scripts/release.sh 2.76.0`
ran the full declared suite and three tests in `-p bee --bin bee` went
red. All three come from the worker-status-digest merge (`c4f951657`).
Nothing was tagged or pushed.

## Locked decisions

- **D1** (`56d30fdd`) — Keep `bee herding notify --step` and raise
  `PINNED_FLAG_COUNT` from 235 to 236. worker-status-digest D2
  locks `--step` by name. The nearest existing name,
  `--phase`, means a bee workflow phase, so reuse gives one word two
  meanings.
- **D2** (`b0d85440`) — Pin the wsdg-1 truth line of
  `docs/history/worker-status-digest/plan.md` as a
  `KNOWN_HISTORICAL_EXCEPTIONS` entry. The plan is gate-approved
  history and is not rewritten.
- **D3** (`6a7ba50e`) — Re-cut `GOLDEN_NATIVE_CELL_PROMPT` once, so it
  carries the progress-note line that wsdg-2 added to `worker-cell.md`.

## Out of scope

No product behavior change. No edit to `scripts/release.sh`, to
`worker-cell.md`, or to any `docs/history/worker-status-digest/` file.
