# rust-port-10 — bee-model-guard port: dispatch evaluation + economics audit line + conformance

**Status:** [DONE]
**Worker:** Jerry

Outcome: ported `.bee/bin/lib/dispatch-guard.mjs`'s `evaluateDispatch` onto `bee_core::dispatch_guard` (all seven deny classes — `codex-spawn-unmarked`, `generic-type-denied`, `param-tier-mismatch`, `param-on-nameless-tier`, `param-not-configured`, `cli-tier-denied`, `bare-denied` — plus the `codex-spawn-marker`/`model-param`/`marker` allow transports, deny reasons copied verbatim), backed by a new `bee_core::config` tier-resolution layer (`normalize_models`/`resolve_tier`/`resolve_advisor`/`model_for_tier`) that extends rust-port-8's raw config reader with state.mjs's `DEFAULT_MODELS` defaulting and `normalizeTierValue` shape validation, including advisor-slot resolution, plus a pure `derive_economics` port. `queen_bee::hooks::model_guard` is a thin wrapper turning the verdict into the hook exit/stderr/audit-log contract: one `.bee/logs/dispatch.jsonl` economics line per evaluated dispatch (fail-open), reusing rust-port-9's `write_guard::run_fail_open` for the crash boundary rather than a second hand-rolled wrapper.

Files touched:

- `crates/bee-core/src/config.rs` (tier-resolution additions + inline unit tests)
- `crates/bee-core/src/dispatch_guard.rs` (new)
- `crates/bee-core/src/lib.rs` (module wiring)
- `crates/queen-bee/src/hooks/model_guard.rs` (new)
- `crates/queen-bee/src/hooks/mod.rs` (hook dispatch wiring)
- `crates/queen-bee/tests/modelguard_conformance.rs` (new — the single integration target for this cell)

Verify: `cargo test --manifest-path crates/Cargo.toml -p queen-bee --test modelguard_conformance` — 13 passed, 0 failed, covering all seven deny classes, allow twins, dispatch.jsonl line-shape parity (ts-normalized), both a node-side and a rust-side crash fixture, rig self-check, negative control, and a diff-detector meta-proof.

Full trace, deviations, and verification evidence: `.bee/cells/rust-port-10.json`.
