# pstack Rust checks — Context

**Feature slug:** pstack-rust-checks
**Date:** 2026-11-14
**Scope:** Standard
**Domain types:** RUN

## What was asked

The owner asked to finish the two open pstack items. This feature is the
second: the five Rust checks that `pstack-craft-adoption` named in its
Slice 2 and never built (`docs/history/pstack-craft-adoption/plan.md:149`,
ranked in `docs/history/research/pstack-craft-study.md` rows 9-13). A
read-only gather on 2026-11-14 confirmed that none of the five exists; a
patch-id check exists only at merge time, computed from the commit.

## What will be done

bee gets a `bee plan check` verb, an Upstream cells block in the worker
prompt, a patch-id stored at cap and read at merge, a dead-pointer warning
at `bee close`, and a `--rolling` refill for `bee dispatch wave --run`.
Every new check is advisory or additive: none adds a new refusal.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | New read-only verb `bee plan check [--feature <slug>] [--json]`, schema `bee-plan-check/v1` `{ok, feature, mode, findings:[{check, detail}]}`. Checks: (a) sections: for mode standard or high-risk, Summary, Requirements, Load-bearing claims, Approach, Role assignments, Risk map, Cells present and in that order; other modes need Summary, Load-bearing claims, Cells; (b) playbook: plan.md cites `skills/bee-planning/playbooks/<route class>.md`; (c) decisions: every D-number of the CONTEXT.md Locked Decisions table appears in plan.md; (d) evidence: each `read` claims row has its verbatim evidence, outer backticks stripped, as a byte substring of the anchored line or line range, or of the whole file with no line. Exit 0 clean, 10 with findings. `bee gate --preview` prints the same findings as warning lines. Nothing new refuses. | c9ae60cb |
| D2 | Upstream trace relay: `dispatch prepare --kind cell` renders an Upstream cells block for each capped dependency: id, title, outcome, commit, files (at most 20, then a count), deviations (each cut to 300 characters). A cell with no deps renders byte-identically to today. The block is outside the knowledge holdout. | 03da9d9b |
| D3 | Patch-id proof currency: a cap records the stable patch-id of its report commit as `trace.report.patch_id` when the commit resolves. The carried-proof check of `bee worktree merge` uses the stored `patch_id` first and computes it from the commit only when none is stored. Found in the branch set reads carried; not found reads stale. The merge preflight stays advisory. | 77258729 |
| D4 | `bee close` warns on dead proof pointers: each token of a capped cell's proof line that names a repo-relative path (has a slash, is not a URL or a flag) is checked under the repo root; each missing path prints one warning line with cell and path, and the JSON gains `dead_proof_pointers [{cell, path}]`. It never refuses. | 682e3531 |
| D5 | Rolling refill: `bee dispatch wave --run --rolling --limit N` keeps at most N herding runs live; when one ends, bee recomputes the schedule, claims and prepares the next ready cell and starts it, until none is ready and none is live. Native Agent-tool items return under `native` once and are never refilled. `--rolling` without `--run` refuses, naming `--run`. Without `--rolling`, wave behaves as before. | 806c6687 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/workflow-state/gates.md | suspect | the claims-table refusal and gate preview that D1 sits beside |
| docs/knowledge/areas/workflow-state/dispatch.md | suspect | the cell prompt blocks D2 extends |
| docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md | suspect | the carried-proof patch-id check D3 changes |
| docs/knowledge/areas/workflow-state/merge-readiness.md | suspect | the close proof door D4 adds a warning to |
| docs/knowledge/areas/bee-herding/waves-and-occupancy.md | suspect | the wave run D5 extends |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs` | the claims-table parse D1 reuses |
| `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs` | `run_gate_preview_body`, where D1's warnings print |
| `packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs` | the newest small verb group D1 mirrors |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | the cell prompt (D2) and `run_dispatch_wave` (D5) |
| `packages/bee/prompts/worker-cell.md` | the worker prompt template D2 extends |
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | where a cap writes `trace.report` (D3) |
| `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs` | `commit_patch_id` and `split_carried_proof` (D3) |
| `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` | the close proof door (D4) |
| `packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs` | the parallel herding run (D5) |

## Out of scope

New refusals of any kind. Each check here reports; a later feature may
promote one to a door once it has run clean on real plans.
