promote proposal for work item "traceable-runs" (docs/history/traceable-runs/CONTEXT.md + docs/history/traceable-runs/plan.md) — 9 capped cell(s): trun-1, trun-2, trun-3, trun-4, trun-5, trun-6, trun-7, trun-8, trun-9
anchor: history — docs/history/traceable-runs/CONTEXT.md, docs/history/traceable-runs/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/traceable-runs/delivery.md

---
type: bee.delivery
title: traceable-runs — delivery
description: "Delivery record proposed by bee knowledge promote for work item traceable-runs: 9 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-08-14
bee:
  id: traceable-runs-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [docs/history/traceable-runs/CONTEXT.md, docs/history/traceable-runs/plan.md]
  sources: [docs/history/traceable-runs/CONTEXT.md, docs/history/traceable-runs/plan.md, .bee/cells/trun-1.json, .bee/cells/trun-2.json, .bee/cells/trun-3.json, .bee/cells/trun-4.json, .bee/cells/trun-5.json, .bee/cells/trun-6.json, .bee/cells/trun-7.json, .bee/cells/trun-8.json, .bee/cells/trun-9.json]
---

# traceable-runs — Delivery

## What shipped

- **trun-1** — done (5 file(s) changed)
- **trun-2** — bee state gate records who approved a gate, under what bypass level, and why; a new feature record already seeds every gate pending (2 file(s) changed)
- **trun-3** — done (3 file(s) changed)
- **trun-4** — Auto-commit .bee and this feature's docs/history/ before WORKTREE_MERGE_MAIN_DIRTY refuses, closing the merge deadlock (5 file(s) changed)
- **trun-5** — Split GATE_ALLOWED_PREFIXES into GATE_ALLOWED_PREFIXES_GATED (docs/history/ only) and GATE_ALLOWED_PREFIXES_INTAKE (unchanged, blanket docs/); gated-phase boundary and its 'Allowed now' message use the gated list, the idle/terminal intake gate and the git-bookkeeping arm use the intake list, and hook_local.rs's worktree-first exemption stays on the intake list to preserve its exact prior behavior (5 file(s) changed)
- **trun-6** — Doctrine now sends every file-touching request through a brief and an approval at every lane (4 file(s) changed)
- **trun-7** — Add a persisted run_state field to the workflow record (closed vocabulary: shaping/awaiting-approval/running/blocked/done), derived from status/gates/cell-counts and written on every create/update; extend apply_workflow_d1_fields so it reaches .bee/state.json; expose it in bee status --json beside gate_records; no new cell status value. (5 file(s) changed)
- **trun-8** — New deferred_queue.rs verb module (add/list/claim/release/complete) wired into verbs/mod.rs's dispatch chain; claim exclusivity proven with a real multi-process race test plus a falsifiability negative control in tests/concurrency.rs. (3 file(s) changed)
- **trun-9** — Wired all scribing-debt scans through the shared queue rule so completing a scribe record clears the debt on every reporting surface; added queue-branch coverage (9 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **trun-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **trun-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **trun-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

- **trun-1** — Also edited verbs/state_group/tests.rs (outside the cell's declared files): two byte-level assertions on gates_patch_from_record's output (via write_through_projection) needed the new state field to keep the full declared suite green; minimal, mechanical, required by the in-scope gates_patch_from_record fix.
- **trun-2** — The cell's action text points at default_gates() in state_group/policy.rs (lines 390/506) as the seam that seeds pending gates at start-feature. Confirmed (per trun-1's own note, reread against the code) that default_gates() only resets the PROJECTED .bee/state.json approved_gates boolean, never the D3 record's state/actor/at/reason/bypass_level fields. The real seam is default_gate_entry() in workflow_store/record.rs, already landed by trun-1: ensure_workflow_record_for_feature's primary path (run_start_feature) passes gates: None, so a brand-new record already gets state:"pending" on every gate for free. Left policy.rs untouched; added a_new_feature_record_seeds_every_gate_as_pending (state_group/tests.rs) as the proof instead of a code change.
- **trun-2** — The actor/at/reason/bypass_level trace never reaches gates_patch_from_record (workflow_store/handoff.rs) or write_through_projection (state_group/ledger.rs) — both are outside this cell's declared files. Instead, run_gate_body issues a second, still-lock-held update_workflow_assuming_lock_with patch (using only functions already imported into set_gate.rs) that stamps the trace onto exactly the touched gate name(s) after the existing write_through_projection call lands approved/state. This avoids widening the cell's file scope while keeping the write inside the same workflow lock.
- **trun-2** — The action text says to regenerate the CLI help/registry payload if it is generated from a declaration. packages/bee-rs/crates/bee/src/generated/registry_payload.json is NOT in this cell's declared files, and its generator (scripts/export_registry_payload.mjs, per its own regen comment) no longer exists in this repo (Node runtime removed, per src/main.rs's own header). No cli_shape/registry validation gates a native verb before try_native — run_gate's own keys_known allowlist (extended in this cell) is what the real CLI enforces, so behavior is correct and fully tested without this file. The registry_payload.json entry for `state gate` still lacks --actor/--bypass-level/--reason in its documented parameters (bee --help --json), which is a real but doc-only gap — flagging it for the orchestrator rather than hand-editing a 184KB generated file outside this cell's scope.
- **trun-4** — git add -A -- <pathspecs> (unlike git status) fails outright with 'pathspec did not match any files' when a root matches nothing at all -- the ordinary case for docs/history/<feature> on a worktree that never wrote there. commit_main_bookkeeping filters pathspecs to ones that exist on disk or are tracked (git ls-files) before add/commit, discovered via a red test and fixed before green.
- **trun-5** — Retargeted two existing tests that pinned the old shared-list behavior rather than deleting them: gated_phase_denies_source_until_execution_approved now asserts docs/history/ stays allowed (was docs/plan.md); apply_patch_gate_policy_denies_source_allows_docs now asserts docs/history/ stays allowed and adds an assertion that blanket docs/ now refuses. Both retargets are RED-FIRST per the cell brief, not silent edits.
- **trun-8** — Split tests by kind rather than the guessed deferred_queue_tests.rs: fold/lifecycle/dual-condition-stale unit tests stay inline in deferred_queue.rs (capture.rs/backlog.rs's own convention); the multi-process claim race and its negative control live in tests/concurrency.rs, the crate's established home for real-OS-process race proofs.

## Provenance

Proposed by `bee knowledge promote --work traceable-runs` from 9 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/traceable-runs/CONTEXT.md`, `docs/history/traceable-runs/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "traceable-runs" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-14T11:01:42.113Z), the work item declares no bee.areas.

area workflow-state:
  - [trun-1] done — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/trun-1.json)
  - [trun-2] bee state gate records who approved a gate, under what bypass level, and why; a new feature record already seeds every gate pending — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/trun-2.json)
  - [trun-3] done — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/trun-3.json)
  - [trun-4] Auto-commit .bee and this feature's docs/history/ before WORKTREE_MERGE_MAIN_DIRTY refuses, closing the merge deadlock — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/trun-4.json)
  - [trun-5] Split GATE_ALLOWED_PREFIXES into GATE_ALLOWED_PREFIXES_GATED (docs/history/ only) and GATE_ALLOWED_PREFIXES_INTAKE (unchanged, blanket docs/); gated-phase boundary and its 'Allowed now' message use the gated list, the idle/terminal intake gate and the git-bookkeeping arm use the intake list, and hook_local.rs's worktree-first exemption stays on the intake list to preserve its exact prior behavior — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/trun-5.json)
  - [trun-6] Doctrine now sends every file-touching request through a brief and an approval at every lane — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/trun-6.json)
  - [trun-7] Add a persisted run_state field to the workflow record (closed vocabulary: shaping/awaiting-approval/running/blocked/done), derived from status/gates/cell-counts and written on every create/update; extend apply_workflow_d1_fields so it reaches .bee/state.json; expose it in bee status --json beside gate_records; no new cell status value. — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/trun-7.json)
  - [trun-8] New deferred_queue.rs verb module (add/list/claim/release/complete) wired into verbs/mod.rs's dispatch chain; claim exclusivity proven with a real multi-process race test plus a falsifiability negative control in tests/concurrency.rs. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/trun-8.json)
  - [trun-9] Wired all scribing-debt scans through the shared queue rule so completing a scribe record clears the debt on every reporting surface; added queue-branch coverage — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/trun-9.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell trun-1 — save as docs/knowledge/patterns/traceable-runs-trun-1-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-1 — pitfall candidate
description: "Pitfall candidate mined from cell trun-1's capped trace: Also edited verbs/state_group/tests.rs (outside the cell's declared files): two byte-level assertions on gates_patch_from_record's output (via write_through_pr…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-1-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-1.json]
  polarity: pitfall
---

# traceable-runs cell trun-1 — pitfall candidate

## What the cell did

done

## Recorded evidence (verbatim from .bee/cells/trun-1.json)

- **deviation** — Also edited verbs/state_group/tests.rs (outside the cell's declared files): two byte-level assertions on gates_patch_from_record's output (via write_through_projection) needed the new state field to keep the full declared suite green; minimal, mechanical, required by the in-scope gates_patch_from_record fix.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell trun-2 — save as docs/knowledge/patterns/traceable-runs-trun-2-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-2 — pitfall candidate
description: "Pitfall candidate mined from cell trun-2's capped trace: The cell's action text points at default_gates() in state_group/policy.rs (lines 390/506) as the seam that seeds pending gates at start-feature. Confirmed (per…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-2-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-2.json]
  polarity: pitfall
---

# traceable-runs cell trun-2 — pitfall candidate

## What the cell did

bee state gate records who approved a gate, under what bypass level, and why; a new feature record already seeds every gate pending

## Recorded evidence (verbatim from .bee/cells/trun-2.json)

- **deviation** — The cell's action text points at default_gates() in state_group/policy.rs (lines 390/506) as the seam that seeds pending gates at start-feature. Confirmed (per trun-1's own note, reread against the code) that default_gates() only resets the PROJECTED .bee/state.json approved_gates boolean, never the D3 record's state/actor/at/reason/bypass_level fields. The real seam is default_gate_entry() in workflow_store/record.rs, already landed by trun-1: ensure_workflow_record_for_feature's primary path (run_start_feature) passes gates: None, so a brand-new record already gets state:"pending" on every gate for free. Left policy.rs untouched; added a_new_feature_record_seeds_every_gate_as_pending (state_group/tests.rs) as the proof instead of a code change.
- **deviation** — The actor/at/reason/bypass_level trace never reaches gates_patch_from_record (workflow_store/handoff.rs) or write_through_projection (state_group/ledger.rs) — both are outside this cell's declared files. Instead, run_gate_body issues a second, still-lock-held update_workflow_assuming_lock_with patch (using only functions already imported into set_gate.rs) that stamps the trace onto exactly the touched gate name(s) after the existing write_through_projection call lands approved/state. This avoids widening the cell's file scope while keeping the write inside the same workflow lock.
- **deviation** — The action text says to regenerate the CLI help/registry payload if it is generated from a declaration. packages/bee-rs/crates/bee/src/generated/registry_payload.json is NOT in this cell's declared files, and its generator (scripts/export_registry_payload.mjs, per its own regen comment) no longer exists in this repo (Node runtime removed, per src/main.rs's own header). No cli_shape/registry validation gates a native verb before try_native — run_gate's own keys_known allowlist (extended in this cell) is what the real CLI enforces, so behavior is correct and fully tested without this file. The registry_payload.json entry for `state gate` still lacks --actor/--bypass-level/--reason in its documented parameters (bee --help --json), which is a real but doc-only gap — flagging it for the orchestrator rather than hand-editing a 184KB generated file outside this cell's scope.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell trun-4 — save as docs/knowledge/patterns/traceable-runs-trun-4-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-4 — pitfall candidate
description: "Pitfall candidate mined from cell trun-4's capped trace: git add -A -- <pathspecs> (unlike git status) fails outright with 'pathspec did not match any files' when a root matches nothing at all -- the ordinary case fo…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-4-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-4.json]
  polarity: pitfall
---

# traceable-runs cell trun-4 — pitfall candidate

## What the cell did

Auto-commit .bee and this feature's docs/history/ before WORKTREE_MERGE_MAIN_DIRTY refuses, closing the merge deadlock

## Recorded evidence (verbatim from .bee/cells/trun-4.json)

- **deviation** — git add -A -- <pathspecs> (unlike git status) fails outright with 'pathspec did not match any files' when a root matches nothing at all -- the ordinary case for docs/history/<feature> on a worktree that never wrote there. commit_main_bookkeeping filters pathspecs to ones that exist on disk or are tracked (git ls-files) before add/commit, discovered via a red test and fixed before green.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell trun-5 — save as docs/knowledge/patterns/traceable-runs-trun-5-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-5 — pitfall candidate
description: "Pitfall candidate mined from cell trun-5's capped trace: Retargeted two existing tests that pinned the old shared-list behavior rather than deleting them: gated_phase_denies_source_until_execution_approved now assert…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-5-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-5.json]
  polarity: pitfall
---

# traceable-runs cell trun-5 — pitfall candidate

## What the cell did

Split GATE_ALLOWED_PREFIXES into GATE_ALLOWED_PREFIXES_GATED (docs/history/ only) and GATE_ALLOWED_PREFIXES_INTAKE (unchanged, blanket docs/); gated-phase boundary and its 'Allowed now' message use the gated list, the idle/terminal intake gate and the git-bookkeeping arm use the intake list, and hook_local.rs's worktree-first exemption stays on the intake list to preserve its exact prior behavior

## Recorded evidence (verbatim from .bee/cells/trun-5.json)

- **deviation** — Retargeted two existing tests that pinned the old shared-list behavior rather than deleting them: gated_phase_denies_source_until_execution_approved now asserts docs/history/ stays allowed (was docs/plan.md); apply_patch_gate_policy_denies_source_allows_docs now asserts docs/history/ stays allowed and adds an assertion that blanket docs/ now refuses. Both retargets are RED-FIRST per the cell brief, not silent edits.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell trun-8 — save as docs/knowledge/patterns/traceable-runs-trun-8-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-8 — pitfall candidate
description: "Pitfall candidate mined from cell trun-8's capped trace: Split tests by kind rather than the guessed deferred_queue_tests.rs: fold/lifecycle/dual-condition-stale unit tests stay inline in deferred_queue.rs (capture.r…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-8-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-8.json]
  polarity: pitfall
---

# traceable-runs cell trun-8 — pitfall candidate

## What the cell did

New deferred_queue.rs verb module (add/list/claim/release/complete) wired into verbs/mod.rs's dispatch chain; claim exclusivity proven with a real multi-process race test plus a falsifiability negative control in tests/concurrency.rs.

## Recorded evidence (verbatim from .bee/cells/trun-8.json)

- **deviation** — Split tests by kind rather than the guessed deferred_queue_tests.rs: fold/lifecycle/dual-condition-stale unit tests stay inline in deferred_queue.rs (capture.rs/backlog.rs's own convention); the multi-process claim race and its negative control live in tests/concurrency.rs, the crate's established home for real-OS-process race proofs.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell trun-9 — save as docs/knowledge/patterns/traceable-runs-trun-9-pitfall.md

---
type: bee.pattern
title: traceable-runs cell trun-9 — pitfall candidate
description: "Pitfall candidate mined from cell trun-9's capped trace: trun-9: completing a scribe queue record does not clear the debt the session preamble reports (preamble reads an unreconciled second copy of scribing_debt), an…"
timestamp: 2026-08-14
bee:
  id: traceable-runs-trun-9-pitfall
  lifecycle: draft
  areas: [workflow-state]
  sources: [.bee/cells/trun-9.json]
  polarity: pitfall
---

# traceable-runs cell trun-9 — pitfall candidate

## What the cell did

Wired all scribing-debt scans through the shared queue rule so completing a scribe record clears the debt on every reporting surface; added queue-branch coverage

## Recorded evidence (verbatim from .bee/cells/trun-9.json)

- **failure_signature** — trun-9: completing a scribe queue record does not clear the debt the session preamble reports (preamble reads an unreconciled second copy of scribing_debt), and no test anywhere exercises the new deferred_debt_cleared reconciliation rule or either enqueue path
- **failure_signature** — trun-9 rework: the preamble and nudge copies of scribing_debt are reconciled, but verbs/status_full/cells.rs:436/:467 remain unreconciled with the deferred queue, so completing a scribe record clears the preamble and close's door while `bee status --json` and `bee orient` keep reporting the same debt

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 9 capped cell(s) mined, 1 delivery draft, 9 area bullet(s), 6 pattern candidate(s), 0 file(s) written.