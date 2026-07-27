# rust-port-5 — bee-core fsutil semantics + unknown-field round-trip property tests

Status: [DONE]

## Outcome

Ported `.bee/bin/lib/fsutil.mjs` into `bee-core::fsutil`: `ensure_dir`, `read_json`/`read_jsonl` (tolerant, BOM-aware, corrupt-line-skipping), `write_json_atomic` (temp file + same-dir rename), `append_jsonl` (compact-line append, propagating I/O errors — see deviation below). Every "matches the mjs reader" claim is proven by a file-based node driver (`tests/support/mjs_oracle.mjs`) running the real `.bee/bin/lib/fsutil.mjs` in a child process and diffing its output against Rust, never a reimplementation guess.

Red-first (high-risk lane, `behavior_change: true`): a naive non-atomic `write_json_atomic` (plain truncating `fs::write`, no temp+rename) let a seeded-target concurrency test (6 writer threads × 60 iters racing a polling reader) observe an empty file mid-write, red 3/3 runs. A naive `read_jsonl` that unwrapped each line's parse instead of skipping corrupt ones panicked against the oracle's 8-case corrupt-fixture table. Both fixed to the real port and reran green.

The oracle then caught a genuine, unplanned divergence on a green implementation: mjs's `readJsonl` tolerates a leading BOM landing on a jsonl line as a side effect of ECMAScript's `trim()` WhiteSpace set including U+FEFF — Rust's Unicode `White_Space` property does not. Fixed with a `js_trim()` helper matching that exact ECMAScript set, reconfirmed against the real mjs reader.

All three `must_haves.truths` verified:
- unknown JSON fields survive a Rust read-modify-write of an mjs-written record (table-driven, 3 payload shapes; also reread by the real mjs reader afterward — cross-runtime interleaving, D3).
- `write_json_atomic`'s crash-safety property proven via genuine concurrent renames (not a hand-simulated kill) — `file` is only ever mutated by the single `rename` syscall.
- `read_jsonl` matches the real mjs reader byte-for-byte on 8 corrupt/truncated jsonl fixture shapes.

## Deviation

The cell's action text tags `appendJsonl` as "fail-open"; the actual `.bee/bin/lib/fsutil.mjs` source has no try/catch around it and every real caller (`decisions.mjs`, `perf.mjs`, `cells.mjs`, `reviews.mjs`, `capture.mjs`, `compaction.mjs`) relies on it propagating write failures. The "fail-open" *telemetry* pattern used for `.bee/logs/contention.jsonl` is implemented by `lock.mjs`'s own `appendContentionTelemetry`, which deliberately bypasses fsutil's `appendJsonl` for that reason. Ported `append_jsonl` to match the actual frozen source (propagating `io::Result`), not the shorthand description — documented in the module doc comment so it isn't "fixed" back by accident. No truth in `must_haves` tested error-swallowing behavior, so this is a documentation-level deviation, not a functional gap.

## Files touched

`crates/bee-core/Cargo.toml`, `crates/bee-core/src/lib.rs`, `crates/bee-core/src/fsutil.rs`, `crates/bee-core/tests/fsutil_oracle.rs`, `crates/bee-core/tests/support/mjs_oracle.mjs`, `crates/Cargo.lock`.

Commit: `d087e2e` — one commit, cell id `rust-port-5` in the message.

No edits to `.bee/bin/lib/fsutil.mjs` (frozen, D1) or `crates/Cargo.toml` (members final, rust-port-1). Oracle tests only ever operate on `tempfile::tempdir()` paths, never the repo's live `.bee/` store.

Full cell definition/trace (incl. `verification_evidence`): `.bee/cells/rust-port-5.json`.
