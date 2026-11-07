---
type: bee.delivery
title: slp-role-gaps — delivery
description: "Delivery record for slp-role-gaps: nine role-interaction gaps found by comparing bee with five reference multi-agent systems, shipped in 6 capped cells."
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding]
  required_context: [docs/history/slp-role-gaps/CONTEXT.md, docs/history/slp-role-gaps/plan.md]
  sources: [docs/history/slp-role-gaps/CONTEXT.md, docs/history/slp-role-gaps/plan.md, docs/history/slp-role-gaps/hat-wave-synthesis.md, docs/history/slp-role-gaps/judge-slice-1.md, docs/history/slp-role-gaps/judge-rig-7.md, docs/history/learnings/20261015-slp-role-gaps-flash-route-shared-worktree.md]
---

# slp-role-gaps — Delivery

## What shipped

- **rig-1** — `bee cells dissent` refuses a repeat dissent on a cell whose dissent already has a verdict, unless new evidence is given; the evidence is stored on the appended entry (D3).
- **rig-2** — the native finish report accepts five optional handback keys (reaction, skipped, unknowns, found, stopped), validates them, stores them on the trace and clears them on release (D1, D2).
- **rig-3** — herding results carry the same handback and the dissent's new evidence onto the run envelope; `herding.paseo.on_model_mismatch: refuse` stops a mismatched agent while the run polls; the envelope carries `model_mismatch` in both modes (D1, D2, D3, D9).
- **rig-5** — `bee herding status` reads a write job whose worktree stopped changing as `stalled` with reason `no_diff_progress`, and appends one message per stalled stretch; observe-only (D7).
- **rig-6** — the worker Result form names the optional keys, partial stop, dissent once and the four-layer final report; the swarming skill and the worker prompt point at it (D1, D2, D3, D8).
- **rig-7** — the advisor prompt asks for a recommendation, a confidence and a falsifier; blind-lane and hat synthesis score proposals with names hidden; the reviewer checks the exact head first (D4, D5, D6). Its reopen re-cut the advisor and reviewer dispatch goldens and pinned one plan span in the CLI-shape scan.

## Verify

- Each cell capped against its recorded verify; the slice judge (bee-review, opus) returned PASS for all six (`docs/history/slp-role-gaps/judge-slice-1.md`, `judge-rig-7.md`).
- Full declared suite after the rework: unit binary 4260 passed, 0 failed on re-run; all 40 other test binaries green. One test, `native_probe_freshness_checks_each_record_leg_against_an_isolated_executable`, failed once under the parallel run and passed alone — filed as a flake in the backlog.
- rig-6's note "every_template_carries_the_original_request_block_and_matches_disk stays red" was the vendored prompt copy; the wave-barrier `bee dev regen` and the re-vendored `.bee/bin/bee` cleared it.

## Deviations

- rig-1, rig-3, rig-5 were rerouted from role `code` (paseo-pi-flash, observed thinking off) to role `test` (opus): decisions `d933d1a2`, `64df01a4`, `5117fa99`.
- rig-3 absorbed the planned rig-4 (both edited the run envelope), after the plan-step hat wave.
- rig-1 kept `record_dissent`'s signature and added an evidence-taking sibling, so `herding/run.rs` stayed out of its scope.
- rig-3 keeps the five handback fields in one sub-struct with defaults, and records the configured model and thinking on the job record so the refusal and the envelope read one place.
- rig-5 shows `stall_reason` on the status line beside the `stalled` word, not only on the job record.

## Provenance

Built from the six capped cell traces and the plan-check and judge records named in `sources`. Lessons: `docs/history/learnings/20261015-slp-role-gaps-flash-route-shared-worktree.md`.
