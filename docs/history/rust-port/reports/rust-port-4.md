# rust-port-4 — D7a CLI parity harness (bee-parity)

**Status:** [DONE]

**Outcome:** Built the std-only `bee-parity` crate's `--self-check`: clones one queen-bench-generated fixture into three temp roots outside the repo tree, runs the same `node .bee/bin/bee.mjs status --json` through a shared runner, and proves the rig's own detection power — self-parity (leg A vs B) diffs clean with both legs exiting 0, and a third leg with one seeded mutation must diff non-clean or the self-check fails. Root-resolution safety (structural + empirical fixture-signature check) is asserted before every invocation. Verify passed, exit 0.

**Files touched:**
- `crates/bee-parity/src/main.rs`
- `crates/bee-parity/src/normalize.rs`
- `crates/bee-parity/src/differ.rs`
- `crates/bee-parity/src/clone.rs`
- `crates/bee-parity/src/rootsafety.rs`
- `crates/bee-parity/src/mutate.rs`
- `crates/bee-parity/src/runner.rs`

Full trace/evidence: `.bee/cells/rust-port-4.json`
