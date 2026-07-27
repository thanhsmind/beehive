# rust-port-19 — Grow the D5 fixture to exercise git ancestry, review candidates, transcript root

Status: [DONE]

## Outcome

Grew `crates/queen-bench/src/fixture.rs`'s `generate()` to exercise the two `status` cost blocks the pinned-size-only fixture previously measured as zero (Panel BLOCKER repair, validation decision 2026-07-26):

- **Git ancestry**: a real git repo (>= 50 linear commits + 2 tags) initialized co-located with `.bee/` — the fixture still resolves as its own bee root (proven: `resolveRootsCore`'s git-root fallback branch resolves `storeRoot === workRoot ===` the fixture itself, asserted indirectly via `onboarding.installed === true` on a real `bee.mjs status --json` read).
- **Review candidates**: a `.bee/review-candidates.jsonl` ledger (>= 60 entries) + 3 `.bee/reviews/` sessions engineered so `deriveCandidateStatus` hits all four `CANDIDATE_STATUSES` on a real read — `unreviewed` (no covering session), `in review` (open session), `reviewed` (approved session pinned at the git tip, 0 commits since), `review stale` (both via a real `rev-list --count > 0` AND via a candidate head that isn't a real git object, forcing a genuinely unresolvable `merge-base --is-ancestor`).
- **Crash-candidate transcript**: one heartbeat-stale, non-clean-end session under `.bee/sessions/`, with its transcript written under an INJECTED `recovery.transcript_roots` directory (never `$HOME/.claude/projects`) at the exact encoded-project-dir layout `detectCrashCandidates` resolves through (`recovery.mjs`, mirroring `perf.mjs`'s `encodeProjectDir`), sized >= 300 KB so `readTranscriptTail` genuinely reads past its 256 KB default window.

Added 3 new pinned floors (`REVIEW_CANDIDATES_FLOOR_COUNT` >= 60, `GIT_COMMITS_FLOOR_COUNT` >= 50, `TRANSCRIPT_TAIL_FLOOR_BYTES` >= 300 KB) — `generate()` refuses below any of them, same as the 4 pre-existing size floors (7/7 refusal cases now, all independently proven in both a unit test and `--self-test`).

`--self-test` now also asserts **non-triviality**: a real `node .bee/bin/bee.mjs status --json` run against the generated fixture must report `review.candidates.total > 0` (non-degraded) and `recovery.candidates` non-empty (non-degraded) — a fixture that still measures nothing fails `--self-test`, per the cell's must-have.

Verified end-to-end: `cargo build --release --manifest-path crates/Cargo.toml && cargo run --release --manifest-path crates/Cargo.toml -p queen-bench -- --self-test` → PASS, plus `cargo test -p queen-bench` (4 passed, including a new `encode_project_dir_mirrors_mjs` unit test proving byte-for-byte parity with `perf.mjs`'s `encodeProjectDir`).

## Files touched

`crates/queen-bench/src/fixture.rs`, `crates/queen-bench/src/selftest.rs`, `crates/queen-bench/src/main.rs` (new `--review-candidates`/`--git-commits`/`--transcript-tail-bytes` CLI flags), `crates/queen-bench/Cargo.toml` (added `serde_json = "1"`, already pinned at that version elsewhere in the workspace, for `selftest.rs`'s JSON assertions), `crates/Cargo.lock`.

## Deviations

None beyond the planned scope. Added `serde_json` as a `queen-bench`-local dependency (not touching the root `crates/Cargo.toml` members list) to parse the real `bee.mjs status --json` output robustly for the non-triviality assertion, rather than fragile substring matching.

Full cell definition/trace: `.bee/cells/rust-port-19.json`.
