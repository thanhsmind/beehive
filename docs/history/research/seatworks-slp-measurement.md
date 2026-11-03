---
artifact_contract: bee-research/v1
topic: seatworks-slp-measurement
depth: standard
date: 2026-10-03
---

# Small-model measurement for SLP on Pi (slice 4)

## Bottom Line

- **Result:** on one tiny task, every run finished the whole bee workflow on
  both versions — 8 of 8 runs put an executable `scripts/hello.sh` on main
  with a capped cell. This task cannot tell the versions apart on success.
- **Speed:** with the small model, the median run took 330 s after the change
  and 560 s before. Three runs per version, with a wide spread (326–722 s
  after, 461–574 s before). Treat this as a hint, not a result.
- **Guard denials did not fall:** 19 after, 15 before, for the small model.
  The refusals that slices 2 and 3 rewrote never fired on this task. The
  denials that did fire are mostly shell-shape guards no slice touched.
- **What reached the model:** the new per-turn `run:` line reached the small
  model in 2 of 3 runs after the change (15 and 7 sightings).
- **Biggest remaining friction:** the write guard's shell-shape refusals —
  "a `cd` earlier in this compound command" and "unexpanded shell syntax" —
  16 of 52 denials across all runs, on both versions and both models.
- **Confidence:** high that both versions complete this task; low on any
  difference between them. Suggested next step: a task set that exercises the
  rewritten refusals, and a slice on the shell-shape guard texts.

## Setup

| Item | Value |
|---|---|
| Before | bee at `adf52d027` (release 2.48.0 plus bookkeeping), built from a clean archive |
| After | bee at main `13e6ba0f4` (slices 1–3 and the refusal follow-ups merged) |
| Pi | 1.0.0, headless: `pi -p --mode json --session-dir …` |
| Small model | `openrouter/~deepseek/deepseek-flash-latest` (the direct `deepseek-flash` route returned `402 Insufficient Balance`) |
| Stronger model | `openai-codex/gpt-5.6-luna` |
| Sandbox | a fresh git repo per run, onboarded by that version's own binary, `.bee/bin/bee` vendored, `gate_bypass: "full"` (as in all five mined transcripts) |
| Task | "Add a shell script scripts/hello.sh that prints "hello from bee" and make it executable. … follow AGENTS.md exactly, from orient to a merged and capped cell. Do not ask questions; the gate bypass is on." |
| Runs | small model: 3 per version; stronger model: 1 per version; run in parallel, one sandbox each |

The harness is in `docs/history/research/seatworks-slp-measurement/`:
`measure-run.sh` (one run), `measure-analyze.py` (the table below),
`measure-denials.py` (denial classes), `measure-results.json` (raw rows). The
scripts carry this session's scratchpad paths; change `S` to rerun them.

## Results

| Run | Tool calls | bee calls | Guard denials | Hand reads of bee state | Hand writes of bee state | Capped cells | Script on main | Seconds |
|---|---|---|---|---|---|---|---|---|
| after, small, 1 | 102 | 54 | 7 | 5 | 0 | 1 | yes | 330 |
| after, small, 2 | 91 | 46 | 7 | 2 | 0 | 1 | yes | 326 |
| after, small, 3 | 123 | 57 | 5 | 3 | 0 | 1 | yes | 722 |
| before, small, 1 | 98 | 51 | 2 | 8 | 0 | 1 | yes | 574 |
| before, small, 2 | 127 | 68 | 11 | 5 | 0 | 1 | yes | 560 |
| before, small, 3 | 98 | 51 | 2 | 2 | 1 | 1 | yes | 461 |
| after, stronger | 104 | 78 | 10 | 0 | 0 | 1 | yes | 644 |
| before, stronger | 120 | 87 | 8 | 0 | 0 | 1 | yes | 788 |

Definitions: a guard denial is a tool error whose text names a guard, a
denial or a refusal. A hand read or write of bee state is a shell command that
reads or changes `.bee/state*`, `.bee/lanes`, `.bee/claims`, `.bee/cells` or
`.bee/runtime` directly instead of through a bee verb.

Denial classes, all runs:

| Class | After | Before | Touched by slices 1–3 |
|---|---|---|---|
| "a `cd` earlier in this compound command" | 4 | 8 | no |
| unexpanded `$VAR` in a write target | 3 | 1 | no |
| CLI-shape guard | 3 | 2 | no (already named a fix) |
| control-plane verb refused inside a worktree | 5 | 4 | no |
| containment (main path from a worktree) | 4 | 3 | yes (slice 2) |
| concurrent-worker git guard | 2 | 1 | no |
| intake gate / worktree-first / other | 8 | 4 | partly |

After a denial, the very next call succeeded 21 of 29 times after the change
and 20 of 23 before.

## What this does and does not show

- It shows both versions carry a small model through a full gated feature on
  a tiny task, with no lost result and no duplicate dispatch.
- It does not show that slices 1–3 help: the task never hit the rewritten
  refusals, never ran a herded cell worker, and never had a second lane — the
  three situations the transcripts failed on.
- The stronger model was not more efficient on this task: it made more bee
  calls and took longer on both versions.
- Hand reads of bee state fell slightly with the change (median 3 vs 5) and
  hand writes went from one to zero; three runs per side cannot separate this
  from noise.
- Not measured: cost per run, and herded worker behavior (the small model ran
  every cell inline, which the tiny lane allows).

## Follow-ups

- A task set that forces the rewritten refusals: a second live lane, a cell
  already claimed by the same session, a deployment dispatch, a preview
  mismatch, and a small-lane cell that must be dispatched to a herded worker.
- The shell-shape guard texts are now the largest measured friction; give
  them the same runnable-fix treatment as pi-slp-operations D1.

## Source Pack

- Runs and transcripts: this session's scratchpad `measure/<run>/stdout.jsonl`
  (not committed; the derived rows are in `measure-results.json`).
- Prior evidence: `docs/history/research/seatworks-slp-pi-small-models.md`
  ("Transcript evidence").
