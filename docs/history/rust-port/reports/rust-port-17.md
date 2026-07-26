# rust-port-17 — bee-chain-nudge + bee-state-sync hooks incl. worktree-holds renewal, conformance

**Status:** [DONE]

**Outcome:** Ported the two remaining heavy hooks onto the rust-port-7 hook runtime — `bee-chain-nudge` (`crates/queen-bee/src/hooks/chain_nudge.rs`, read-only SubagentStop advisory: worker-collect nudge, review-synthesis nudge, scribing-debt warning) and `bee-state-sync` (`crates/queen-bee/src/hooks/state_sync.rs`, silent throttled heartbeat + reservation-lease + cross-worktree-hold renewal, then the locked `rebuildStateProjection` call from rust-port-16 with LOCK_BUSY skipping silently). Backing write paths were added to `bee-core` since they didn't exist yet: `claims::heartbeat_session`/`renew_claim_ttl` (the one write path `heartbeatTouch` composes), `reservations::renew_holds_by_session` (the one lease-store write path the hook needs), and `holds::renew_holds` (the cross-worktree-holds renewal path, added to the previously read-only `holds.rs`). The new `heavyhooks_conformance` target (`crates/queen-bee/tests/heavyhooks_conformance.rs`, 16 tests) proves byte-identical advisory JSON / silence for chain-nudge across trigger and silence classes, and side-effect parity for state-sync (state.json rebuild, heartbeat/claim/lease/hold renewal, throttle no-op, LOCK_BUSY skip, crash fail-open) against the real frozen mjs oracles on seeded fixture roots — fixtures seeded via a new tracked driver, `tests/support/heavyhooks_fixture.mjs`, which builds authentic session/claim/lease/hold records through the real `claims.mjs`/`lease-store.mjs`/`worktree-holds.mjs` functions rather than hand-guessed shapes.

**Deviations (disclosed):** (1) only the one write path each hook's call site exercises was ported from claims.mjs/reservations.mjs+lease-store.mjs/worktree-holds.mjs — every other mutating function in those modules (adopt/release/sweep, epoch fencing, lease acquire/release, mirror/release/sweep) stays unported, out of scope. (2) a real byte-compatibility bug was caught by this cell's own oracle diff during development and fixed before capping: the first `heartbeat_session`/`renew_claim_ttl` draft round-tripped through bee-core's typed `Session`/`Claim` structs, which serialize `Option` fields as `null` when absent — the mjs source's own plain-object writes never introduce a key the file didn't already have. Fixed by patching the raw `serde_json::Value` in place for both writes. (3) `chain_nudge.rs` uses the hook's own bare root everywhere (matching bee-chain-nudge.mjs's literal usage, which never references `ctx.controlRoot`); `state_sync.rs` mirrors bee-state-sync.mjs's mixed usage (heartbeat/hold-renewal take the control root, lease-renewal/rebuild take the bare root, each per the mjs source's own literal call).

**Files:**
- `crates/queen-bee/src/hooks/chain_nudge.rs` (new)
- `crates/queen-bee/src/hooks/state_sync.rs` (new)
- `crates/queen-bee/src/hooks/mod.rs` (registers the two new hook names)
- `crates/bee-core/src/claims.rs` (extended — heartbeat/claim-TTL write paths)
- `crates/bee-core/src/reservations.rs` (extended — lease renewal-by-session)
- `crates/bee-core/src/holds.rs` (extended — cross-worktree hold renewal, the module's first write path)
- `crates/queen-bee/tests/heavyhooks_conformance.rs` (new — this cell's mandated single integration target, 16 tests)
- `crates/queen-bee/tests/support/heavyhooks_fixture.mjs` (new — node fixture driver)

**Verify:** `cargo test --manifest-path crates/Cargo.toml -p queen-bee --test heavyhooks_conformance` — 16 passed, 0 failed. No regressions: `cargo test -p bee-core` (172 passed) and `cargo test -p queen-bee` (117 passed) both green; `cargo build --workspace` clean. Full trace + verification evidence: `.bee/cells/rust-port-17.json`.
