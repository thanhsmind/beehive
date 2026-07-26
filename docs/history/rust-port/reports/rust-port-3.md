# rust-port-3 — bee-core D9 lock protocol port + node-rust interop conformance

**Status:** [DONE]
**Worker:** Dave (ceiling)

Outcome: the full D9 lock-file protocol now lives in `crates/bee-core/src/lock.rs` (path scheme, `{pid,session,ts,token}` body, 30s/1h staleness with pid-liveness probe, identity-verified rename takeover, transient-FS retry, hooks-never-wait try-once, fail-open contention.jsonl telemetry), proven equivalent to the frozen `.bee/bin/lib/lock.mjs` by 11 cross-runtime interop tests (real lock.mjs in node children via the file-based `tests/support/lock_driver.mjs`, per-test temp roots, backdated-ts staleness) plus 10 unit tests — including the two-simultaneous-holders negative test proven to fail red on a deliberate protocol violation. Sharded lease store deferred to Slice 3 (validation decision W3).

Files touched:

- `crates/bee-core/src/lock.rs` (new)
- `crates/bee-core/tests/lock_interop.rs` (new)
- `crates/bee-core/tests/support/lock_driver.mjs` (new)
- `crates/bee-core/src/lib.rs` (module wiring)
- `crates/bee-core/src/fsutil.rs` (`js_trim` → `pub(crate)`)
- `crates/bee-core/Cargo.toml` (+ sha2, getrandom, libc(unix))
- `crates/Cargo.lock`

Verify: `cargo test --manifest-path crates/Cargo.toml -p bee-core lock` — 21 passed, 0 failed.

Full trace, deviations, and verification evidence: `.bee/cells/rust-port-3.json`.
