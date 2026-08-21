# herding-prompt-stall — work shape

Route: class `bugfix` · lane `standard` · flags `external-systems`,
`covered-contract-change` · product files 1.

Locked context and the three decisions: `docs/history/herding-prompt-stall/CONTEXT.md`.

## What is being built

`bee herding run` reads herdr's agent lifecycle state at three points and
misreads it at all three. This slice fixes all three in the one file that
holds them, and syncs the knowledge layer that describes the old behavior.

| Wait point | Today | After |
|---|---|---|
| `wait_for_agent_ready` (`run.rs:811`) | accepts `idle` only | accepts `idle` or `done` (D2); `blocked` fails fast (D3) |
| `deliver_pointer` (`run.rs:783`) | 30 blind sends, receipt = a bee-observed transition into `working` | one send through `herdr agent prompt --wait --until working --timeout`; herdr's `agent_prompt_stalled` is the failure (D1); `blocked` fails fast (D3) |
| `decide_poll` (`run.rs:604`) | `blocked` is invisible; the pane burns the idle timeout | `blocked` ends the wait at once with a named remedy (D3) |

The single behavioral thread: **bee stops interpreting a raw lifecycle sample
taken inside the agent's boot window, and uses herdr's own settle-aware verbs
instead.** herdr already carries the settle logic (`agent prompt --wait`,
`agent wait --until`) and already classifies an approval or question UI as
`blocked`. bee was reimplementing the first badly and ignoring the second.

## Why this size

One product file (`packages/bee-rs/crates/bee/src/herding/run.rs`) holds all
three wait points, the `Herdr` trait they call through, and the `FakeHerdr`
every test injects. Splitting them into separate cells would serialize three
workers on one file for no parallelism gain. The docs cell touches no code and
runs beside it.

## Cost if the shape is wrong

The blast radius is every `bee herding run` dispatch. Getting the receipt
wrong in the other direction — refusing a delivery that did land — turns a
working dispatch into a `spawn_failed`. That is why the delivery cell keeps the
result-file escape (`result_present()`): an ultra-fast round that finishes
before any status poll still counts as delivered.

## Slice 1 (the whole feature)

**hps-1 — `run.rs`: read herdr's lifecycle state the way herdr defines it.**
`packages/bee-rs/crates/bee/src/herding/run.rs`. Widen the `Herdr` trait's
`agent_prompt` to carry herdr's `--wait --until <state> --timeout <ms>` and add
the typed `agent_prompt_stalled` outcome; teach `RealHerdr` to pass those flags
and classify the stall; then apply D1, D2 and D3 at the three wait points
above. Retire the baseline/transition receipt. Keep the `result_present()`
escape, the deny codes, and the pane-lifecycle rules unchanged.
Tests: the existing `deliver_pointer` and `wait_for_agent_ready` cases that pin
the old semantics are rewritten, plus new cases for a `done` pane accepted as
ready, a `blocked` pane refused fast at each of the three points, and a
stalled prompt surfaced as a delivery failure rather than 30 resends.

**hps-2 — knowledge and config docs carry the new contract.**
`docs/knowledge/areas/bee-herding/*`, `skills/bee-herding/references/operational-invariants.md`,
`.bee/config-sample.json`. Record herdr's lifecycle-state contract (why a
never-focused pane reports `done`, what `blocked` means, what
`agent_prompt_stalled` means) and replace every statement of the retired
idle-only gate and transition receipt. No code.

## Verification

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release
--manifest-path packages/bee-rs/Cargo.toml herding` for hps-1 — the herding
suite is the scope the change touches. hps-2 is a docs cell: pointer/parity
check over the edited files.

Beyond the suite, the acceptance proof is the reproduction itself: three
concurrent `bee herding run --agent agy-flash` into one worktree, all three
reaching a written result — the probe that produced this finding.
