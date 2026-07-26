# rust-port-9 — bee-write-guard port 1 of 3 (core write checks)

**Status:** [DONE]

**Outcome:** Ported the write-guard CORE spine (gate/intake, reservations, cross-session + cross-worktree holds, worktree containment) onto the queen-bee hook runtime as `queen-bee hook write-guard`, with deny reasons byte-identical to the frozen mjs source; proven by the new `writeguard_core` conformance corpus (20/20 green) driving sha256-verified seeded copies of `bee-write-guard.mjs` as the node oracle. Bash path stays rust-port-11; read side/apply_patch/AskUserQuestion stay rust-port-12; the port is dark until the flip slice (no wiring edits).

**Files:**
- `crates/bee-core/src/guards.rs` (new — checkWrite core verdict engine)
- `crates/bee-core/src/state.rs` (lane path/read + resolve_pipeline)
- `crates/bee-core/src/config.rs` (merged raw config value incl. config.local.json overlay)
- `crates/bee-core/src/holds.rs` (normalize_path visibility + leading-`./` strip parity fix)
- `crates/bee-core/src/lib.rs`, `crates/bee-core/Cargo.toml` (regex-lite), `crates/Cargo.lock`
- `crates/queen-bee/src/hooks/write_guard.rs` (new — containment + hook main flow, fail-open wrapper)
- `crates/queen-bee/src/hooks/mod.rs` (dispatch entry)
- `crates/queen-bee/tests/writeguard_core.rs` (new — conformance corpus, cell-mandated target)

**Verify:** `cargo test --manifest-path crates/Cargo.toml -p queen-bee --test writeguard_core` — 20 passed, 0 failed. Full trace + verification evidence: `.bee/cells/rust-port-9.json`.
