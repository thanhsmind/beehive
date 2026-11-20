---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: bee test warns when a shared cargo target can swap its binaries

## Summary

With `CARGO_TARGET_DIR` shared across checkouts, a worker building another
checkout can replace binaries under a running suite and turn it falsely red.
The owner kept the shared target (CONTEXT.md D1); `bee test` now says so
before it runs, naming the dir and the other checkouts.

Mode: `small` — one warning in one verb, one ledger helper.

## Requirements (from CONTEXT.md)

- D1: warn, and still run, when the target is shared and a live worker runs in another checkout.
- C1: contract shared-target-warning.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | bee test runs from one function | read | packages/bee-rs/crates/bee/src/verbs/test_runner.rs:80 | `fn run(use_json: bool, t0: Instant) -> Option<ExitCode> {` |
| 2 | The ledger folds waves into rows | read | packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:227 | `fn fold_waves_by_wave_id(root: &Path) -> Vec<WaveRow> {` |
| 3 | Each ledger worker records its checkout | read | packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:82 | `pub(crate) worktree: String,` |
| 4 | The owner's shell shares one target | ran | rg -n CARGO_TARGET_DIR ~/.bashrc | `export CARGO_TARGET_DIR="$HOME/.cache/cargo-target"` |

## Approach

Situation: `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

A pure function decides the warning from the variable, the checkout and the
live workers' checkouts; a ledger helper lists live workers' checkouts the
same way occupancy's fallback does (unresolved workers in waves younger than
`DEFAULT_STALE_AFTER_MS`). The tester pins both directions red-first.

Rejected: per-worktree target dirs (owner chose option 2); refusing to run
(the owner wants a warning, not a stop).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "stw-1 adds the warning."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "stw-2 writes the contract tests red then green."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "The leader adds one paragraph to the suite concept."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Small lane; review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "The owner chose between two options."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "A warning only; the run is unchanged."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Warning only."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Owner-requested."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The anchors are known."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Both cells have a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "No prose deliverable."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Small lane."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| An unresolved ledger row from a finished worker warns falsely | Same stale window as occupancy's fallback; the text says the ledger counts the worker live |
| A worktree run reads its own empty ledger | The helper reads the control root's ledger |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| stw-1 | Warn in bee test when a shared cargo target meets a live worker elsewhere | test_runner.rs, wave_ledger.rs | — | warning before the run | existing test_runner and wave_ledger tests |
| stw-2 | Prove the shared target warning red then green | test_runner.rs tests, report | stw-1 | contract cases pass | shared_target filter |

```json
[
{"id":"stw-1","feature":"shared-target-warning","title":"Warn in bee test when a shared cargo target meets a live worker elsewhere","lane":"small","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::test_runner herding::wave_ledger",
"must_haves":{"truths":["a pure function in test_runner.rs takes the CARGO_TARGET_DIR value, the tested checkout path and the live workers' checkout paths and returns the warning text or None","it returns None when the value is unset or empty, when the target resolves inside the checkout, or when every live worker checkout is the tested checkout","the warning names the target dir and each other checkout and says a red may be false","run() prints the warning to stderr before the suite runs, adds shared_target_warning to the --json result, and still runs and records the suite","a pub(crate) helper in wave_ledger.rs lists the worktree of every unresolved worker in waves younger than DEFAULT_STALE_AFTER_MS, read from the control root"]},
"decisions":["881a704e-f4a2-4756-ad74-75de5ad7178e"],
"files":["packages/bee-rs/crates/bee/src/verbs/test_runner.rs","packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"],
"read_first":["docs/history/shared-target-warning/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D1. In packages/bee-rs/crates/bee/src/herding/wave_ledger.rs add pub(crate) fn live_worker_worktrees(root: &Path, now_ms: i64, stale_after_ms: i64) -> Vec<String> over fold_waves_by_wave_id (line 227) and is_live (line 208): the worktree of each worker with outcome None in a live row. In packages/bee-rs/crates/bee/src/verbs/test_runner.rs add a pure fn shared_target_warning(target_dir: Option<&str>, checkout: &Path, live_worktrees: &[String]) -> Option<String>: None when target_dir is None or blank, when the target (canonicalized when it exists, else lexical) is inside the canonical checkout, or when no live worktree differs from the checkout after canonicalizing both; otherwise one line shaped: warning: CARGO_TARGET_DIR <dir> is shared, and the wave ledger counts a live worker in <checkout list>; a build there can replace binaries under this suite, so a red here may be false. FIX: wait for those workers, or re-run with a private CARGO_TARGET_DIR. In run() (line 80), when commands are declared and before run_declared_tests, read CARGO_TARGET_DIR from the environment, read the live worktrees from the control root (resolve it the way bee herding occupancy finds the ledger; a linked worktree reads main's .bee/wave-ledger.jsonl), print the warning to stderr if Some, and insert shared_target_warning into the result map. The suite still runs. Write no tests (stw-2 owns them) and no code comment. Commit once (cell trailer). Proof: the verify command."},
{"id":"stw-2","feature":"shared-target-warning","title":"Prove the shared target warning red then green","lane":"small","role":"test","deps":["stw-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- shared_target",
"must_haves":{"truths":["a test shows shared_target_warning returns a line naming the dir and the other checkout when the target is outside the checkout and a live worker is elsewhere","tests show None for an unset value, a blank value, a target inside the checkout, and live workers only in the same checkout","a test shows live_worker_worktrees lists unresolved workers of a fresh wave and skips resolved workers and waves older than the stale window","every new test name contains shared_target","a test was run against the stw-1 parent commit and failed or did not compile there, and the report quotes it","the report under docs/history/shared-target-warning/reports/ quotes each run's result line"]},
"decisions":["6fd09277-23f5-4d22-a97a-02e14bc0738d","881a704e-f4a2-4756-ad74-75de5ad7178e"],
"files":["packages/bee-rs/crates/bee/src/verbs/test_runner.rs","packages/bee-rs/crates/bee/src/herding/wave_ledger.rs","docs/history/shared-target-warning/reports/stw-2.md"],
"read_first":["docs/history/shared-target-warning/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract shared-target-warning (decision 6fd09277) on the stw-1 commit. Add tests to the existing tests modules: test_runner.rs (mod tests near line 423) for shared_target_warning, all five cases, using tempdirs for real paths; wave_ledger.rs tests for live_worker_worktrees with a fixture .bee/wave-ledger.jsonl (fresh unresolved, fresh resolved, stale unresolved). Every test name contains shared_target. Red proof: run the verify with the production files at the stw-1 parent (git show <parent>:<path> into the tree), record the failure or compile error, restore both files to HEAD, confirm git status shows them clean apart from your test edits. Then run the verify green. Edit no production code outside the tests modules. Write docs/history/shared-target-warning/reports/stw-2.md with each run and its result line. Commit once (cell trailer)."}
]
```
