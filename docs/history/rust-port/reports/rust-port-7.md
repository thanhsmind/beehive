# rust-port-7 — queen-bee hook runtime: adapter port + D7b conformance rig + trivial hooks

**Status:** [DONE]

**Outcome:** Ported the shared hook runtime adapter (`.bee/bin/hooks/adapter.mjs`) and the two trivial hooks (`tools-logger`, `codex-subagent-audit`) into `crates/queen-bee`, wired behind `queen-bee hook <name>`, plus the D7b `hook_conformance` rig (11 test functions covering all 7 required fixture classes) proving both against the real, frozen mjs hooks via seeded, sha256-verified temp roots. Ran a genuine scoped red-first pass (deliberately inverted `hook_enabled`, observed 3 reported failures, reverted, reran green) before capping.

**Files touched:**
- `crates/queen-bee/Cargo.toml`
- `crates/queen-bee/src/main.rs`
- `crates/queen-bee/src/lib.rs`
- `crates/queen-bee/src/adapter.rs`
- `crates/queen-bee/src/hookconfig.rs`
- `crates/queen-bee/src/hooks/mod.rs`
- `crates/queen-bee/src/hooks/tools_logger.rs`
- `crates/queen-bee/src/hooks/codex_subagent_audit.rs`
- `crates/queen-bee/tests/hook_conformance.rs`
- `crates/queen-bee/tests/support/adapter_encoding_oracle.mjs`
- `crates/Cargo.lock` (dependency resolution only)

**Deviations:** see `.bee/cells/rust-port-7.json` trace (`deviations`) — codex-subagent-audit is not gated by the hooks-enabled toggle in the frozen source (ported faithfully to match reality, not the cell text's generalization); `[lib]` target added to queen-bee's own Cargo.toml for direct adapter-function testing.

Full trace/evidence: `.bee/cells/rust-port-7.json`.
