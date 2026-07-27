# rust-port-8 — bee-core guard-support readers + command-registry JSON bridge + tokenize port

**Status:** [DONE]
**Worker:** Stuart

Outcome: added the guard-support piece of the D3 storage-compat contract to `bee-core` — read-only, zero-subprocess readers for `.bee/config.json` (hooks toggles, `gate_bypass` level, full `models` map incl. `CONFIGURABLE_SLOTS`/advisor), the projected `.bee/state.json`, cells listing, the legacy `.bee/reservations.json` projection plus sharded lease records (`.bee/runtime/leases/{cells,paths}/*.json`), and the `checkWrite` support set guards.mjs imports: worktree-holds (`find_foreign_holds`/`holds_store_corrupt`, with a ported `paths_overlap` and a small `Date.parse`-equivalent for TTL/staleness math), workspace-store records, and claims (`read_session`/`heartbeat_stale`). Every struct round-trips unknown fields via serde flatten (D3), matching rust-port-5's `fsutil` pattern. New `scripts/dump_command_registry.mjs` (promoted from the proven spike) snapshots all 116 `COMMAND_REGISTRY` entries to `.bee/cache/command-registry.json` with an embedded `source_sha256`; the registry loader flags a typed `Stale` result on drift rather than silently trusting an out-of-date snapshot. `tokenize_command` is an exact port oracle-diffed against BOTH frozen mjs copies (`tokenize-command.mjs` and `guards.mjs`'s `tokenize`) over a 30-case corpus (heredocs, redirects, env prefixes, subshells, quotes/escapes, chain separators), also proving the two mjs copies still agree with each other.

Files touched:

- `crates/bee-core/src/{tokenize,registry,config,state,cells,reservations,holds,workspace,claims,jsdate}.rs` (new)
- `crates/bee-core/tests/guard_support.rs` (new — the single integration target for this cell)
- `crates/bee-core/tests/support/tokenize_oracle.mjs` (new)
- `crates/bee-core/src/lib.rs` (module wiring)
- `scripts/dump_command_registry.mjs` (new)
- `.gitignore` (`.bee/cache/` line, outside the managed BEE fence)

Verify: `node scripts/dump_command_registry.mjs && cargo test --manifest-path crates/Cargo.toml -p bee-core --test guard_support` — 25 passed, 0 failed.

Full trace, deviations, and verification evidence: `.bee/cells/rust-port-8.json`.
