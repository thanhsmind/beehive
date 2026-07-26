# rust-port-6 — CI-full-run visibility: test_rust_workspace.mjs wrapper suite

[DONE]

Added `scripts/tests/test_rust_workspace.mjs`, a wrapper suite auto-discovered by `run_verify.mjs`'s `test_*.mjs` glob (zero registry edits) that spawns `cargo build --release --manifest-path crates/Cargo.toml` then `cargo test --manifest-path crates/Cargo.toml`, mirrors cargo's exit status, and fails red with an actionable message when cargo is missing from PATH — never a silent skip. The header states the impacted-run blind spot: local `--impacted-from-git` runs will not select this suite until the D6 dependency graph lands in Slice 7.

Files touched: `scripts/tests/test_rust_workspace.mjs`

Full trace/evidence: `.bee/cells/rust-port-6.json`
