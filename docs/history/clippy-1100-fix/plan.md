---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: main is clippy-clean on the CI toolchain

## Summary

CI went red on main after release 2.108.0: CI clippy is 1.100.0 and flags
`needless_bool` in `is_empty_fleet`, a lint the local 1.98.0 does not have.
Fix the lint and the unused `fleet` dependency the same toolchain reports,
and prove it with clippy 1.100.0 installed beside the default.

Mode: `small` — two lines of code, one manifest line; the existing tests pin the function.

## Requirements (from CONTEXT.md)

- D1: `!all_dry_run`; drop `serde` from fleet; prove with `cargo +1.100.0 clippy -- -D warnings`.
- C1: contract clippy-1100-fix.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The flagged guard returns a bool literal | read | packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:368 | `if all_dry_run {` |
| 2 | fleet declares serde | read | packages/bee-rs/crates/fleet/Cargo.toml:12 | `serde.workspace = true` |
| 3 | 1.100.0 reports exactly these two | ran | cargo +1.100.0 clippy --manifest-path packages/bee-rs/Cargo.toml --message-format short | `warning: unused dependency` |

## Approach

Situation: `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

Red first: CI run 43789475981 is the failing record, reproduced locally by
`cargo +1.100.0 clippy -- -D warnings`. The truth table of `is_empty_fleet`
is already pinned by the tests at wave_ledger.rs:795-874. One code cell, one
tester cell.

Rejected: pinning the toolchain here (release-binaries adds cross targets to
`stable`; a pin needs its own change); `#[allow(clippy::needless_bool)]`
(clippy-warnings-clean D2 bans item allows).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "c11-1 fixes the lint and the manifest."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "c11-2 verifies on both toolchains."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "No behavior a concept describes changes."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Small lane; review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Small lane."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "No behavior change."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Existing tests pin the function."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Red CI on main is fix-first."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The CI log is the map."},
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
| The rewrite flips the function's result | Tests at wave_ledger.rs:795-874 pin every arm |
| Removing serde breaks a fleet derive | 1.100.0 says it is unused; the build proves it |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| c11-1 | Make bee-rs clippy-clean on rustc 1.100 | wave_ledger.rs, fleet/Cargo.toml, Cargo.lock | — | clippy 1.100 -D warnings exits 0 | clippy on both toolchains, wave_ledger tests |
| c11-2 | Verify clippy 1.100 cleanliness and the empty-fleet truth table | report | c11-1 | contract checks pass | clippy on both toolchains, full suite |

```json
[
{"id":"c11-1","feature":"clippy-1100-fix","title":"Make bee-rs clippy-clean on rustc 1.100","lane":"small","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo +1.100.0 clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- wave_ledger",
"must_haves":{"truths":["cargo +1.100.0 clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings exits 0 with no warning line","cargo clippy on the default toolchain still exits 0 under -D warnings","the wave_ledger tests pass"]},
"decisions":["7bb9fb4c-36e5-4486-bbc3-81240abd9d1f"],
"files":["packages/bee-rs/crates/bee/src/herding/wave_ledger.rs","packages/bee-rs/crates/fleet/Cargo.toml","packages/bee-rs/Cargo.lock"],
"read_first":["docs/history/clippy-1100-fix/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D1. In packages/bee-rs/crates/bee/src/herding/wave_ledger.rs is_empty_fleet (line 359), replace the final if all_dry_run { return false; } true with !all_dry_run (or the expression clippy 1.100 suggests), same result for every input. In packages/bee-rs/crates/fleet/Cargo.toml remove the serde dependency line; keep serde_json; let cargo update Cargo.lock. Use a private CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-r100 for the +1.100.0 runs. No #[allow], no code comment. Commit once (cell trailer). Proof: the verify command; quote its output lines."},
{"id":"c11-2","feature":"clippy-1100-fix","title":"Verify clippy 1.100 cleanliness and the empty-fleet truth table","lane":"small","role":"test","deps":["c11-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo +1.100.0 clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
"full_suite_reason":"Removing a crate dependency can break any target that builds fleet; the full suite is the related test.",
"must_haves":{"truths":["clippy 1.100.0 -D warnings exits 0 with no warning line","default-toolchain clippy -D warnings exits 0","the full cargo test is green","the existing is_empty_fleet tests cover every arm of its truth table, or a new test adds the missing arm","the report under docs/history/clippy-1100-fix/reports/ lists each check with its result"]},
"decisions":["e6d916ce-9dfe-435d-abb6-14c3a4ec0bce","7bb9fb4c-36e5-4486-bbc3-81240abd9d1f"],
"files":["docs/history/clippy-1100-fix/reports/c11-2.md","packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"],
"read_first":["docs/history/clippy-1100-fix/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract clippy-1100-fix (decision e6d916ce) on the c11-1 commit. Run the verify (CARGO_TARGET_DIR=/home/thanhsmind/.cache/cargo-target-r100 for the +1.100.0 run). Read the is_empty_fleet tests in wave_ledger.rs (around 795-874) and map them to the truth table; add a test in that tests module only for an arm no test covers. Edit no production code. Write docs/history/clippy-1100-fix/reports/c11-2.md with each check, command and result. If a check fails, return BLOCKED with the evidence. Commit once (cell trailer)."}
]
```
