---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: dispatch-ledger-retire

## Summary
A finished cell now marks its dispatch row done the moment it closes, a wave whose workers all came back empty says so, each wave entry tells you the command that rebuilds its payload, the swarming doc states how a dispatch reads transport readiness, and the dead budget-overrun supervisor signal goes away.

Mode: `standard` — 2 risk flags: public-contracts, covered-contract-change
Why this is the least workflow that protects the work: five small decisions across four Rust files and two docs, each with a red test first; no new data shape.

## Requirements (from CONTEXT.md)
- D1: cap sets the worker row to `capped`, by one reconcile call after the cap's cell guard drops, warn-only on failure, covering every cap door (they all pass `cap_cell_from_flags`). The keep-set row-status skip is NOT added: the keep set already skips a capped cell file, and a row-status skip would prune where the code deliberately keeps (a missing or corrupt cell file). Recorded as a decision touching D1. The red test for the row status is real; the prune deadlock test is written and passes today, and is reported as such.
- D2: `bee dispatch wave --run` output carries `empty_wave: true` plus a one-line reason when it ran at least one worker and every one returned nothing (`outcome` and `exit` null, empty stdout). A wave with zero rows is not flagged. `bee herding wave` and fleet `WaveResult` are out of this change.
- D3: each `bee dispatch wave` entry carries the `bee dispatch prepare` command that rebuilds its payload.
- D4: bee-swarming states the `transport_ready` / `transport_reason` check; `bee herding status` text says it reports the calling session's own pane, not worker availability, and names the transport's env variables (herdr and tmux).
- D5: `budget-overrun` leaves both closed signal sets, the `estimate_minutes` reader and the `overrun-2x-estimate` counter that shares it, the supervisor prompt (every line and mirror) and the stale registry text.

## Load-bearing claims

Labels: `read` / `ran` / `guessed`; evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A row set to capped already exists as a function, so the cap path only needs a caller | read | packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:364 | pub(crate) fn reconcile_capped_workers(root: &Path) -> Result<usize, Err2> { |
| 2 | The only production caller of that function is the worktree merge, not cap | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:1164 | crate::verbs::state_group::reconcile_capped_workers(main_root) |
| 3 | The prune keep set already skips a worker row whose cell file is capped, so the deadlock half of D1 is already closed; D1 adds the row-status skip | read | packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:343 | let capped = is_cell_file_capped(&cells_dir.join(format!("{cell_id}.json")))?; |
| 4 | Registration hard-codes running | read | packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:170 | push_worker_record(workers, nickname, cell, role, Some("running")) |
| 5 | The wave verb is run_dispatch_wave | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4667 | pub(crate) fn run_dispatch_wave(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> { |
| 6 | The fleet wave result separates outcome classes | read | packages/bee-rs/crates/fleet/src/choreography.rs:30 | pub struct WaveResult { |
| 7 | The supervisor signal list names budget-overrun | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:307 | "budget-overrun", |
| 8 | The supervisor reads estimate_minutes | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:2682 | const ESTIMATE_FIELD: &str = "estimate_minutes"; |
| 9 | The supervisor prompt documents budget-overrun | read | skills/bee-herding/references/supervisor-prompt.md:85 | **budget-overrun** — a cell is running far past what it was estimated to |
| 10 | Cap doors share one function with main_root | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1678 | StoreRoots::main_root() |
| 11 | Dispatch wave rows are built in wave_run.rs, not from WaveResult | read | packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs:150 | fn row(id: &str, spawn: &Spawn, envelope_path: &Path) -> Value { |
| 12 | Wave worker name is w-<cell> | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4549 | pub(crate) fn auto_wave_worker_name(cell_id: &str) -> String { |
| 13 | The supervisor signal list has a second closed set | read | packages/bee-rs/crates/bee/src/verbs/supervisor.rs:321 | "budget-overrun", |
| 14 | A test pins budget-overrun as a known signal | read | packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:10847 | "budget-overrun" |

## Discovery
CONTEXT.md read set read at the lines above. Finding that moves the shape: `reconcile_capped_workers` and a prune skip on capped cell files already exist (workers.rs:364, :342), so D1's red test for the deadlock is expected to pass today; its red half is the row status staying `running` after cap. The test proves that honestly and the plan does not invent a failure.

## Approach
Playbook: `skills/bee-planning/playbooks/feature/feature.md` (steps 1-9, delegation by cell).

Cap calls `reconcile_capped_workers` once, after the cell guard drops, in `cap_cell_from_flags` (every door shares it); a failure warns and never fails the cap, mirroring phases.rs:1164 (D1). Rows already archived as `running` stay; the change is forward-only. `bee dispatch wave --run` gains `empty_wave` and a per-entry `rebuild` command in wave_run.rs and prepare.rs (D2, D3). Docs and the herding status text carry D4. D5 deletes the signal from both closed sets, the reader, the `overrun-2x-estimate` counter, the prompt lines and their mirrors (regenerated with `bee dev regen`), the registry text and the knowledge mention. Rejected: a new ledger table (the field and its tests exist; Chesterton's fence); a keep-set row-status skip (flips a deliberate fail-open).

Waves: dlr-1, dlr-2, dlr-3, dlr-4 run in parallel only where files are disjoint; dlr-2 and dlr-3 both touch drivers/tests.rs, so dlr-3 runs after dlr-2. Each tester cell follows its implementer.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {
      "stage": "code",
      "classification": "required",
      "role": "code",
      "reason": "dlr-1, dlr-2 and dlr-3 change Rust."
    },
    {
      "stage": "read",
      "classification": "not-applicable",
      "role": "read",
      "reason": "Not used by this feature."
    },
    {
      "stage": "test",
      "classification": "required",
      "role": "test",
      "reason": "A tester cell per code cell adds the new tests."
    },
    {
      "stage": "docs",
      "classification": "required",
      "role": "docs",
      "reason": "dlr-4 and dlr-5 edit the swarming contract and check it."
    },
    {
      "stage": "deploy",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "Not used by this feature."
    },
    {
      "stage": "plan",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "Not used by this feature."
    },
    {
      "stage": "extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "Not used by this feature."
    },
    {
      "stage": "generation",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Not used by this feature."
    },
    {
      "stage": "review",
      "classification": "not-applicable",
      "role": "review",
      "reason": "Not used by this feature."
    },
    {
      "stage": "advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "Not used by this feature."
    },
    {
      "stage": "supervisor",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "Not used by this feature."
    },
    {
      "stage": "lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "Not used by this feature."
    },
    {
      "stage": "lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "Not used by this feature."
    },
    {
      "stage": "lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "Not used by this feature."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "required",
      "role": "hat-facts-gaps",
      "reason": "The plan-step hat wave."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Not used by this feature."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Not used by this feature."
    },
    {
      "stage": "hat-alternatives",
      "classification": "required",
      "role": "hat-alternatives",
      "reason": "The plan-step hat wave."
    },
    {
      "stage": "hat-user-impact",
      "classification": "required",
      "role": "hat-user-impact",
      "reason": "The plan-step hat wave."
    },
    {
      "stage": "design",
      "classification": "not-applicable",
      "role": "design",
      "reason": "Not used by this feature."
    },
    {
      "stage": "writer",
      "classification": "required",
      "role": "writer",
      "reason": "dlr-6 consolidates knowledge."
    }
  ]
}
```

## Risk map

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| cap path calls reconcile | MEDIUM — cap runs under the cell lock; the call must not deadlock on the worker-state lock | dlr-1 | cap test: row status capped afterwards |
| wave output shape | LOW | dlr-2 | wave test on empty and non-empty |
| signal removal | LOW — closed set shrinks by one | dlr-3 | supervisor tests |

## Shape
Epic map: one capability per decision, current slice is all four.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dlr-1 | Mark a worker row capped at cap | state_group/workers.rs, cells/handlers_close.rs, state_group/tests.rs | — | a capped cell's row reads capped | state_group and cells tests |
| dlr-1t | Test the cap row status | cells/tests.rs, state_group/tests.rs | dlr-1 | row capped after cap; prune keeps a missing-cell row | the named tests |
| dlr-2 | Flag an empty wave and name each entry's rebuild command | drivers/wave_run.rs, drivers/prepare.rs, drivers/tests.rs | — | wave output shows empty_wave and rebuild | wave tests |
| dlr-2t | Test the wave flag | drivers/wave_run.rs tests | dlr-2 | empty, non-empty and zero-row waves | the named tests |
| dlr-3 | Remove the budget-overrun signal | supervisor.rs, drivers/tests.rs, catalog.rs, skills/bee-herding/references/supervisor-prompt.md and mirrors, registry text, knowledge doc | dlr-2 | the signal is no longer accepted | supervisor and catalog tests |
| dlr-3t | Test the closed signal sets | supervisor.rs tests | dlr-3 | budget-overrun refused; others still record | the named tests |
| dlr-4 | State the transport check | skills/bee-swarming/SKILL.md, herding.rs status text | — | docs name transport_ready; status names its scope | a status test and the doc parity check |
| dlr-5 | Docs parity and pointer check for dlr-4 | docs | dlr-4 | parity green | `bee knowledge check` |
| dlr-6 | Consolidate knowledge and write delivery.md | docs/knowledge, docs/history/dispatch-ledger-retire/delivery.md | dlr-1t, dlr-2t, dlr-3t, dlr-5 | areas match the code | `bee knowledge check` |
| dlr-7 | Check the consolidated knowledge and delivery record | docs/history/dispatch-ledger-retire/delivery.md | dlr-6 | knowledge check green | `bee knowledge check` |

```json
[
{"id": "dlr-1", "feature": "dispatch-ledger-retire", "title": "Mark a worker row capped at cap", "lane": "standard", "role": "code", "status": "open", "deps": [], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT D1 as refined in plan.md Requirements D1. In cap_cell_from_flags (handlers_close.rs), after the cell guard drops, call crate::verbs::state_group::reconcile_capped_workers(main_root); on Err warn and continue, mirroring worktree/phases.rs:1164-1173. Do not change read_prune_keep_set. Write the red test first: a capped cell's worker row reads `capped` after cap. Edit only existing tests the change forces.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- reconcile_capped cap", "must_haves": {"why": "A finished cell's dispatch row stops reading running, so the ledger stops growing stale.", "out_of_scope": ["the prune keep set", "archived running rows", "fleet WaveResult"], "truths": ["after cap the cell's workers[] row has status capped", "a failure inside reconcile leaves the cap green", "a cell with no row changes nothing"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-1t", "feature": "dispatch-ledger-retire", "title": "Test the cap row status and the keep-set fail-open", "lane": "standard", "role": "test", "status": "open", "deps": ["dlr-1"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs"], "affects_skills": [], "affects_specs": [], "action": "Judge existing coverage first (reconcile_capped_workers tests at state_group/tests.rs:1110-1176), author only the gap: cap through the real verb marks the row capped; a prune keep set still contains a row whose cell file is missing; a capped cell's files are not kept. Cite the existing tests by name in the cap proof.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- reconcile_capped prune keep_set cap", "must_haves": {"why": "The ledger fix needs proof from a worker that did not write it.", "out_of_scope": ["production code edits"], "truths": ["the named tests pass", "the missing-cell keep case is pinned"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-2", "feature": "dispatch-ledger-retire", "title": "Flag an empty wave and name each entry's rebuild command", "lane": "standard", "role": "code", "status": "open", "deps": [], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT D2 and D3 as refined in plan.md. In wave_run.rs finish_value add `empty_wave: true` and a one-line `empty_wave_reason` when at least one row ran and every row has outcome null, exit null and empty output; zero rows is not flagged. In the dispatch wave entries (prepare.rs run_dispatch_wave) add a `rebuild` string `.bee/bin/bee dispatch prepare --runtime <rt> --kind cell --cell <id> --worker w-<id> --json` using auto_wave_worker_name. Keep the worktree refusal. Red test first.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- wave", "must_haves": {"why": "A leader can tell a wave where nobody answered from one that launched nothing, and can rebuild any entry's payload.", "out_of_scope": ["bee herding wave", "fleet WaveResult"], "truths": ["all-empty wave carries empty_wave true", "a wave with one answer is not flagged", "a zero-row wave is not flagged", "every entry carries its rebuild command"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-2t", "feature": "dispatch-ledger-retire", "title": "Test the wave flag and rebuild command", "lane": "standard", "role": "test", "status": "open", "deps": ["dlr-2"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/drivers/wave_run.rs"], "affects_skills": [], "affects_specs": [], "action": "Judge existing wave tests (wave_run.rs:290) first and author the gap: empty, non-empty, zero-row, and rebuild text containing `--cell <id> --worker w-<id>`.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- wave", "must_haves": {"why": "An independent worker checks the wave output shape.", "out_of_scope": ["production code edits"], "truths": ["the named tests pass"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-3", "feature": "dispatch-ledger-retire", "title": "Remove the budget-overrun signal", "lane": "standard", "role": "code", "status": "open", "deps": ["dlr-2"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/supervisor.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "packages/bee-rs/crates/bee/src/catalog.rs", "skills/bee-herding/references/supervisor-prompt.md", "docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/supervisor.rs", "skills/bee-herding/references/supervisor-prompt.md"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT D5 as refined in plan.md. Remove budget-overrun from KNOWN_SIGNALS and INTERVENTION_SIGNALS, the ESTIMATE_FIELD reader, cell_estimate_minutes and the overrun-2x-estimate counter with its tests, every budget-overrun line in supervisor-prompt.md (:85, :168, :179, :201), the catalog.rs watch-word text and the drivers/tests.rs:10847 case, and the knowledge doc mention. Then run `bee dev regen` to refresh mirrors and the registry payload and commit its output. Read-side tolerance of old rows stays.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- supervisor signal catalog prompt && .bee/bin/bee dev release-manifest --check", "must_haves": {"why": "A signal that can never fire stops misleading the supervisor and its prompt.", "out_of_scope": ["adding an estimate input", "historical supervisor rows"], "truths": ["supervisor record refuses budget-overrun", "struggling-loop and same-region-resubmit still record", "no budget-overrun text remains in skills/ or the prompt mirrors"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-3t", "feature": "dispatch-ledger-retire", "title": "Test the closed signal sets", "lane": "standard", "role": "test", "status": "open", "deps": ["dlr-3"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["packages/bee-rs/crates/bee/src/verbs/supervisor.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "packages/bee-rs/crates/bee/src/verbs/supervisor.rs"], "affects_skills": [], "affects_specs": [], "action": "Judge existing signal tests first, author the gap: budget-overrun refused with the closed-set message; the other signals record; an old budget-overrun row on disk is dropped silently on read.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- supervisor signal", "must_haves": {"why": "An independent worker checks the closed set shrank and nothing else moved.", "out_of_scope": ["production code edits"], "truths": ["the named tests pass"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-4", "feature": "dispatch-ledger-retire", "title": "State the transport check in the swarming contract", "lane": "standard", "role": "docs", "status": "open", "deps": [], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["skills/bee-swarming/SKILL.md", "packages/bee-rs/crates/bee/src/verbs/../herding.rs"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "skills/bee-swarming/SKILL.md", "packages/bee-rs/crates/bee/src/verbs/../herding.rs"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT D4 as refined in plan.md. In bee-swarming's Delegation contract state that a dispatch reads transport_ready and transport_reason and that the fallback model applies only when the payload says transport_ready false. In the `bee herding status` no-job text (herding.rs near :1486) say it reports the calling session's own pane, not worker availability, and name the transport's env variables for herdr and tmux. Run `bee dev regen` and commit its output.", "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --no-fail-fast -- herding status && .bee/bin/bee dev release-manifest --check", "must_haves": {"why": "A leader reads one sentence instead of guessing whether dispatch has a transport.", "out_of_scope": ["new transport code", "tmux behavior changes"], "truths": ["the swarming doc names transport_ready and transport_reason", "herding status text states its scope"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-5", "feature": "dispatch-ledger-retire", "title": "Check dlr-4 docs parity and pointers", "lane": "standard", "role": "docs", "status": "open", "deps": ["dlr-4"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["skills/bee-swarming/SKILL.md"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "skills/bee-swarming/SKILL.md"], "affects_skills": [], "affects_specs": [], "action": "Run the parity and pointer checks for dlr-4 as a second worker: bee knowledge check and the release-manifest check; report mismatches only.", "verify": ".bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check", "must_haves": {"why": "A docs change needs a second pair of eyes.", "out_of_scope": ["edits"], "truths": ["both checks pass"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-6", "feature": "dispatch-ledger-retire", "title": "Consolidate knowledge and write delivery.md", "lane": "standard", "role": "writer", "status": "open", "deps": ["dlr-1t", "dlr-2t", "dlr-3t", "dlr-5"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["docs/knowledge/", "docs/history/dispatch-ledger-retire/delivery.md"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md", "docs/history/dispatch-ledger-retire/plan.md", "docs/history/dispatch-ledger-retire/plan.md"], "affects_skills": [], "affects_specs": [], "action": "Run `bee knowledge consolidate --feature dispatch-ledger-retire`, update the worker-ledger, dispatch wave and supervisor concepts to match the code, cite history D-numbers (not raw decision ids), and write docs/history/dispatch-ledger-retire/delivery.md.", "verify": ".bee/bin/bee knowledge check", "must_haves": {"why": "The state layer must match what shipped.", "out_of_scope": ["code edits"], "truths": ["knowledge check passes", "delivery.md exists"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments", "Never hand-edit .bee/*.json(l)"]}},
{"id": "dlr-7", "feature": "dispatch-ledger-retire", "title": "Check the consolidated knowledge and delivery record", "lane": "standard", "role": "test", "status": "open", "deps": ["dlr-6"], "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e"], "files": ["docs/history/dispatch-ledger-retire/delivery.md"], "read_first": ["docs/history/dispatch-ledger-retire/plan.md"], "affects_skills": [], "affects_specs": [], "action": "As a worker sharing no context with dlr-6, run bee knowledge check and confirm delivery.md exists and its claims match the capped cells' diffs; report mismatches only.", "verify": ".bee/bin/bee knowledge check", "must_haves": {"why": "A writer cell needs an independent check.", "out_of_scope": ["edits"], "truths": ["knowledge check passes", "delivery.md exists"], "artifacts": [], "key_links": [], "prohibitions": ["No code comments"]}},
{"id": "dlr-8", "feature": "dispatch-ledger-retire", "title": "Check the transport-check doc and the herding status message", "lane": "standard", "role": "test", "deps": ["dlr-4", "dlr-5"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding::status && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity", "must_haves": {"truths": ["skills/bee-swarming/SKILL.md states that a dispatch reads transport_ready and transport_reason and that the fallback model applies only when the payload says transport_ready false", "bee herding status without a built job says it does not answer availability and names HERDR_ENV and HERDR_PANE_ID", "with that message line reverted the herding status test fails, restored it passes", "the report under docs/history/dispatch-ledger-retire/reports/dlr-8.md quotes each check"], "why": "dlr-4 and dlr-5 have no test-role checker.", "out_of_scope": ["editing any checked file"]}, "decisions": ["649b82da-039f-4d04-94bc-d106be285a0e", "f5916b45-b4f3-4f75-92a9-2dfba2e0d6aa"], "files": ["docs/history/dispatch-ledger-retire/reports/dlr-8.md"], "read_first": ["docs/history/dispatch-ledger-retire/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Check CONTEXT.md D4 on commit 0d31c78f0 (dlr-4) without reading its cap report. Read skills/bee-swarming/SKILL.md and the herding status not-built message in packages/bee-rs/crates/bee/src/herding.rs. Run the verify (fix the filter to the real test names if it matches zero and say so). Mutation: revert the message line in a scratch edit, run, expect a failure, restore from HEAD, run, expect pass. Write the report. Edit no source. Commit only the report with git commit -- <path> (git add -N first) and the cell trailer."}
]
```

## Test matrix
- Cap: a capped cell's worker row reads `capped` (cells test). Pass when `workers[0].status == "capped"`. A cell with no row changes nothing. A reconcile failure leaves the cap green.
- Prune: a capped cell's row no longer keeps its files (passes today, written as a pin); a row whose cell file is missing is still kept. Pass when the keep set contains the missing-cell id.
- Wave: all rows empty gives `empty_wave: true` and a reason; one row with output gives no flag; zero rows gives no flag. Pass when the JSON key matches.
- Rebuild: every entry carries the prepare command with its cell id. Pass when it contains `--cell <id> --worker w-<id>`.
- Signals: `budget-overrun` is refused by `supervisor record`; `struggling-loop` still records. Pass when the refusal names the closed set.
- Same-scenario row: the cap scenario runs on main and on head; main leaves `running`, head gives `capped`.

## Open Questions
(none)

## Out of scope
An estimate input on cells.
