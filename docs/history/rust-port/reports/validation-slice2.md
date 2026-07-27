# rust-port — Validation Report (Slice 2: read spine + status + D5 proof)

Date: 2026-07-26 · Lane: high-risk · Verdict: **READY WITH CONSTRAINTS** (post-repair)

## Shape (8 cells after repair)

Waves: (13, 18, 19) → 14 → 16 → 20 → 15 → 17.

| Cell | Delivers |
|---|---|
| 13 | status readers A: backlog counts, capture queue, decisions-active, cells-derived helpers |
| **18** | **FIX-FIRST** — release-profile fail-open (panic=abort vs catch_unwind) + release-binary crash proof |
| **19** | **Grow the D5 fixture** — git ancestry, ≥60 review candidates + sessions, injected transcript root, new pinned floors |
| 14 | status readers B1 — review derivation via in-process gix (zero subprocess), divergence-class + panic-probe fixtures |
| 16 | workflow-store + state-projection port |
| 20 | status readers B2 — recovery tail, contention, onboarding/handoff/bypass/raw-config/controlRootFor, remaining state helpers |
| 15 | `queen-bee status` — byte-parity (json + text legs) via bee-parity + host-real 5 ms bench gate |
| 17 | chain-nudge + state-sync hooks incl. worktree-holds renewal |

## Evidence gathered

- Confirmed the 97 ms status cost: `buildReviewBlock` (bee.mjs:794 → :409-433) derives each candidate through `spawnSync git` (reviews.mjs:401), memoized per head/ref pair.
- `cargo test -p bee-core --test status_readers_a` → exit 101 with the available-targets list: the `--test <target>` verify form is confirmed non-vacuous.
- `cargo search gix` → 0.86.0 reachable; not currently a bee-core dependency.

## Panel BLOCKERs and disposition

- **B1 (proof hollow):** the pinned D5 fixture (`queen-bench/src/fixture.rs:126-169`) has no `.git`, no review candidates, no transcript root, so the node leg spawns zero git and reads zero transcript — the bench measured only the JSON-read third and neither cost D5 names. → **cell 19 grows the fixture** with new pinned floors and a non-triviality self-test (`bee.mjs status --json` must report a non-zero, non-degraded review block on a generated fixture). D5's never-shrink rule is preserved; the status gate is expected to go red first, and cell 15 returning `[BLOCKED]` with a profile is the honest outcome.
- **B2 (readers with no home):** `readOnboarding`, `readHandoff`, `bypassLevel`, `readRawConfigForValidation`, `controlRootFor` are called by `buildStatus` and were in no cell → folded into cell 20.
- **B3 (stale-binary green):** cell 15's verify now builds the workspace first — `bee-parity` and `queen-bench` resolve sibling binaries via `current_exe` and have no cargo dependency on `queen-bee`, so `cargo run -p …` alone would have benchmarked an old binary.
- **B4 (P1, found by the panel, own cell):** `[profile.release] panic = "abort"` while the fail-open contract uses `catch_unwind` (write_guard.rs:684, :950). Tests build the dev profile, so every fail-open green covers a profile the host never ships — a release binary would abort instead of exiting 0. → **cell 18, fix-first**, with an environment-independent red-first against the pre-fix profile and a proof that spawns the actual release binary.

## Warnings folded into cells

W5 gix divergence classes (shallow clone, packed-refs, detached HEAD, linked worktree, alternates, commit-graph, grafts, unknown-object tri-state) → named fixtures in cell 14, plus tri-state fidelity (unknown object ⇒ unresolved, never covered:false). W6 status non-determinism (clock-boundary reservation expiry, heartbeat staleness) and the hermetic transcript root → pinned/injected in cell 20, never allowlist widening. W7 JS reference-identity filter (bee.mjs:744-746) → named fixture in cell 20. W8 cell 14 overload → split 14/20. W9 cell 17's spurious dep on 15 → deps corrected to 16+20 (the hooks import nothing from the status command and must not be hostage to the 5 ms gate). W10 → D3 added to cell 15's decisions.

## Cold-pickup repairs

Cell 14's helper list pointed at the wrong modules (seven helpers are private to bee.mjs, not exported) → real homes anchored and the oracle split stated (import-style drivers for exported units; whole-command `bee.mjs status --json` sub-object diffing for the private ones). Cell 15 now names the harness files it must extend and states `--status-check` as a new arm that must not break `--self-check`. Cell 17 now names `hook_conformance.rs` as the rig of record (the rig is duplicated in-file, not a shared module). Cells 13/16 got their oracle-pattern and reuse anchors (`fsutil_oracle.rs`, `lock.rs`, `workspace.rs`, `lock_interop.rs`).

## Constraints carried into execution

1. Cell 18 runs first — until it caps, no fail-open claim covers the shipped artifact.
2. The D5 fixture may only grow. A red status gate is reported with its profile, never made green by shrinking the fixture or widening the volatile allowlist.
3. gix path is panic-free by construction and preserves node's tri-state ancestry answer.
4. Ported hooks stay dark until the dedicated flip slice.
