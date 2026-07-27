# rust-port-18

[DONE] — release-profile fail-open fix: `[profile.release]` no longer sets `panic = "abort"` (was silently defeating `catch_unwind`-based fail-open in the shipped binary); `panic = "unwind"` set explicitly with a comment explaining why. Added a fixture-only, env-gated crash seam (`crash_seam_panic_if_armed`, inert without `BEE_QUEEN_BEE_CRASH_SEAM=<hook-name>`) so a new conformance target could force a genuine unwind inside the actual `target/release/queen-bee` binary. Red-first proved the pre-fix binary aborts (no exit code, no crash line) instead of failing open; post-fix it's green (7/7). Binary size cost: +96,520 bytes (~+10.3%, 934,544 -> 1,031,064).

Files touched:
- `crates/Cargo.toml` (`[profile.release]`: `panic = "abort"` -> `panic = "unwind"`, documented; workspace `members` untouched)
- `crates/queen-bee/src/hooks/write_guard.rs` (new `crash_seam_panic_if_armed` fn; wired into `run()`'s `run_fail_open` closure)
- `crates/queen-bee/src/hooks/model_guard.rs` (wired the same seam into `run()`'s `run_fail_open` closure)
- `crates/queen-bee/tests/release_failopen.rs` (new, 7 tests)

Full trace and verification evidence (red-first output, post-fix output, evidence JSON): `.bee/cells/rust-port-18.json`.

Existing suites (hook_conformance, writeguard_core, writeguard_bash, writeguard_read, modelguard_conformance) re-run green after the change: 84 passed, 0 failed — no regressions.

Reservations released. One commit, cell id `rust-port-18` in the message.
