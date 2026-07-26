# rust-port-21 — close the chain-nudge registered-worker coverage gap and two conformance-honesty residuals left by rust-port-17

**Status:** [DONE]

**Outcome:** Fix-first cell closing a proof gap the round-1 goal-check judge raised on rust-port-17 but which the orchestrator did not relay in that cell's rework round (coordinator's own note, not a worker miss). All three items are proof-only — zero production-behaviour changes.

1. **The real gap.** No fixture ever seeded `state.workers`, so `chain_nudge.rs`'s `is_registered_worker` check and the whole ported `worker_name()` fallback ladder (string entry, then object entries matched via `nickname`/`name`/`agent`/`worker`) were never diffed against the frozen `bee-chain-nudge.mjs` oracle — every nudge fixture up to this cell fired via `phase=swarming` only. Added `chain_nudge_worker_name_ladder_every_rung_matches_oracle` (table-driven, one row per rung, `phase=idle` throughout so only the registered-worker path can produce a nudge) and its non-triviality twin `chain_nudge_worker_name_ladder_no_match_stays_silent_twin_matches_oracle`. Also: no chain-nudge fixture ever pinned a `session_id`, so `state::resolve_pipeline` was never exercised beyond its `None` short-circuit. Added `chain_nudge_session_id_exercises_resolve_pipeline_lane_branch_matches_oracle` — a session bound to a lane (via a new `bind-lane` fixture op using the real `claims.mjs` `bindSessionLane`) whose own record carries `phase=swarming`, while the default `state.json` stays idle with no workers; only the resolved lane record can produce the nudge here.
2. **Honesty residuals.** Three same-runtime before/after no-op assertions (`state_sync_disabled_hook_makes_no_writes_on_either_runtime`, `state_sync_throttled_no_op_when_heartbeat_fresh_matches_oracle`, `state_sync_lock_busy_skips_state_rebuild_silently_on_both_runtimes`) compared parsed `serde_json::Value`s — blind to key order under this workspace's `preserve_order` feature (`IndexMap`'s `PartialEq` ignores order), and the LOCK_BUSY assertion's message literally claimed "byte-identical" while being exactly the opposite. Converted all three to raw-text comparison; the message now matches the check.
3. **Unasserted argument.** `model_guard.rs:57` passes its own hook name to `run_fail_open` (fixed in rust-port-17's rework), but `modelguard_conformance.rs`'s rust-side crash test called `run_fail_open` directly with a hand-supplied literal, so a regression at that exact call site would be caught by nothing. Added `crash_fail_open_both_runtimes_log_the_correct_hook_name`, driving the REAL `queen-bee hook model-guard` dispatch (a new `run_rust_with_crash_seam` helper) alongside the node oracle's own crash path, asserting the `hook` field cross-runtime.

**Red-first evidence** (defect reintroduced in real source or, for item 2, a scratch experiment on real session-shaped data; test run to failure; quoted; reverted; re-green):
- Worker ladder: narrowed `chain_nudge.rs`'s fallback list to `[nickname, name]` (dropping `agent`/`worker`) → `chain-nudge/worker-ladder/agent-rung: stdout diverged (node="...Worker \"Kevin\" returned..." rust="")`.
- resolve_pipeline: made the `Ok(pipeline)` arm also read `bee_state.phase` (ignoring the resolved lane record) → `stdout diverged (node="...A bee worker returned..." rust="")`.
- Byte-level vs parsed-Value: a scratch test (run, then removed) built two session-shaped texts with identical fields in different key order — `assert_eq!` on parsed `Value`s passed silently; `assert_eq!` on raw text failed exactly as expected.
- model-guard hook name: temporarily made `model_guard.rs:57` pass the literal `"write-guard"` → `assertion left == right failed: rust port crash line hook field, got ... "hook": String("write-guard") ... left: Some(String("write-guard")) right: Some(String("model-guard"))`.

All four reverted; full suites re-verified green after each.

**Files:**
- `crates/queen-bee/tests/heavyhooks_conformance.rs` (worker-ladder + session-id/lane fixtures; 3 assertions converted to byte-level)
- `crates/queen-bee/tests/modelguard_conformance.rs` (new cross-runtime model-guard crash hook-field test)
- `crates/queen-bee/tests/support/heavyhooks_fixture.mjs` (new `bind-lane` op)

No production files touched (`crates/queen-bee/src/` clean in the final diff).

**Verify:** `cargo test --manifest-path crates/Cargo.toml -p queen-bee --test heavyhooks_conformance && cargo test --manifest-path crates/Cargo.toml -p queen-bee --test modelguard_conformance` — 19 passed (was 16) and 14 passed (was 13), 0 failed. No regressions: `cargo test -p queen-bee` (121 passed across 10 suites) and `cargo test -p bee-core` (172 passed across 9 suites) both green; `cargo build --workspace` clean. Full trace + verification evidence: `.bee/cells/rust-port-21.json`.
