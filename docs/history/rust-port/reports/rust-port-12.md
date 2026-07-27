# rust-port-12 — bee-write-guard port 3 of 3 (read side, apply_patch, AskUserQuestion)

[DONE] Ported `checkRead` (privacy marker + scout-dir denies), the large-read
size guard, Codex `apply_patch` target-by-target provability, and
`AskUserQuestion` schema validation (incl. ask-guard-autofix) to
`bee-core`/`queen-bee`, closing the write-guard port's third and final split.
21/21 conformance fixtures pass against the live node oracle
(`bee-write-guard.mjs`), red-first proven (11/21 genuinely failed against the
pre-cell fall-open code).

Files touched:
- `crates/bee-core/src/guards.rs` — `check_read` + `check_ask_user_question` (guards.mjs ports)
- `crates/queen-bee/src/hooks/write_guard.rs` — read-side dispatch, read-size guard, apply_patch target extraction/wiring, AskUserQuestion dispatch (bee-write-guard.mjs hook-level ports)
- `crates/queen-bee/tests/writeguard_read.rs` — new D7b conformance corpus (21 fixtures)

Full trace/evidence: `.bee/cells/rust-port-12.json`.
