---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: bee-rs is clippy-clean and CI denies new warnings

## Summary

bee-rs carries 1039 clippy findings. Clippy's own machine fixes take 892;
the other 147 are fixed by hand, except two signature-design lints that one
workspace table allows. Then clippy runs with `-D warnings` locally and in
CI, so a new warning turns the build red. Behavior does not change; the full
test suite is the pin and it is green on the unchanged tree.

Mode: `small` — mechanical, behavior-preserving; the suite is the net.

## Requirements (from CONTEXT.md)

- D1: zero findings under `-- -D warnings` on the default targets; commands.test and ci.yml gain `-- -D warnings`.
- D2: `too_many_arguments` and `result_large_err` allowed once in `[workspace.lints.clippy]`; nothing else allowed; no item-level allow.
- C1: contract clippy-warnings-clean.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | 1039 findings, 892 machine-applicable | ran | cargo clippy --manifest-path packages/bee-rs/Cargo.toml --message-format json | `1039 {'auto': 892, 'manual': 147}` |
| 2 | The pin is green on the unchanged tree | ran | bee test at main 6f5d329 lineage (clippy-in-test head 766fdec95) | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml (137.0s)` |

## Approach

Situation: `skills/bee-planning/playbooks/refactor/refactoring.md`.

Pin: the existing suite (bin 5229 tests plus the integration targets), green
before any move. No new structure is named: this is lint cleanup, not a
reshape. Two code cells in series, because the manual pass edits what the
machine pass leaves, then one tester cell.

- cwc-1 (code): the machine pass only, one commit, so its diff reads as
  clippy's own suggestions.
- cwc-2 (code): the 147 manual findings, the workspace lint table (D2), and
  the `-D warnings` flip in commands.test and ci.yml.
- cwc-3 (test): a separate worker checks the contract on the result.

Rejected: one cell for both passes (a judgment edit hides in 892 mechanical
ones); `--all-targets` (commands.test checks default targets); item-level
allows (D2).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "cwc-1 runs the machine pass; cwc-2 fixes the rest and flips -D warnings."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "cwc-3 verifies the contract on the result."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "The concept line rides cwc-2."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Small lane; the tester cell is the independent check. Review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane; both claims ran this session."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Small lane; rejected paths named."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "No behavior change for any host."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "The full suite pins behavior; CI red on a new warning is the intended effect."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The clippy count is the map."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "All cells have a configured role."},
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
| A machine fix changes behavior | Full suite after each code cell; the tester reads the diff for semantic edits |
| Deleting dead_code breaks a test-only caller | Contract: no caller in any target; the suite compiles every target |
| A future toolchain adds a lint and reds CI | Accepted: a real finding, fixed as a fix-first cell |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cwc-1 | Apply clippy's machine-applicable fixes across bee-rs | bee-rs sources | — | about 892 findings gone, suite green | lib and bin tests |
| cwc-2 | Fix the remaining clippy findings and deny warnings | bee-rs sources, Cargo.toml, config, ci.yml, concept | cwc-1 | clippy -D warnings exits 0 | clippy, lib and bin tests, proof_gate |
| cwc-3 | Verify bee-rs is clippy-clean with behavior unchanged | none (report) | cwc-2 | contract checks pass | clippy, lib and bin tests, proof_gate |

```json
[
{"id":"cwc-1","feature":"clippy-warnings-clean","title":"Apply clippy's machine-applicable fixes across bee-rs","lane":"small","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --lib --bins",
"must_haves":{"truths":["cargo clippy --fix applied clippy's machine-applicable suggestions to the default targets of bee and fleet","no edit other than clippy's own suggestions is in the commit","cargo test --release --lib --bins is green with the same test count as before"]},
"decisions":["cf6ce5e3-b8dc-4973-b61a-094234fd3b21"],
"files":["packages/bee-rs/crates/bee/src/**","packages/bee-rs/crates/fleet/src/**"],
"read_first":["docs/history/clippy-warnings-clean/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Run from the worktree root: PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo clippy --fix --allow-dirty --allow-no-vcs --manifest-path packages/bee-rs/Cargo.toml (default targets only, no --all-targets). Repeat until it applies nothing more. Do not hand-edit any file. Run cargo build to confirm it compiles; if a machine fix breaks the build or a test, revert that one file's change and name it in the report for cwc-2. Count the remaining findings with cargo clippy --manifest-path packages/bee-rs/Cargo.toml --message-format short and put the count by lint in the report. Commit once (cell trailer). Proof: the verify command; quote the test counts before (5229 bin tests at the base) and after."},
{"id":"cwc-2","feature":"clippy-warnings-clean","title":"Fix the remaining clippy findings and deny warnings","lane":"small","role":"code","deps":["cwc-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --lib --bins && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test proof_gate",
"must_haves":{"truths":["cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings exits 0","packages/bee-rs/Cargo.toml allows exactly too_many_arguments and result_large_err in [workspace.lints.clippy] and both member crates opt in","no #[allow(clippy::...)] attribute and no code comment is added","the clippy entry of commands.test and the ci.yml clippy step end in -- -D warnings","cargo test --release --lib --bins and the proof_gate tests are green"]},
"decisions":["cf6ce5e3-b8dc-4973-b61a-094234fd3b21","e88c22c7-3bac-4130-a385-6709e7267d2e","4a500653-5751-44cf-9cd9-7186c25a056e"],
"files":["packages/bee-rs/crates/bee/src/**","packages/bee-rs/crates/fleet/src/**","packages/bee-rs/Cargo.toml","packages/bee-rs/crates/bee/Cargo.toml","packages/bee-rs/crates/fleet/Cargo.toml",".bee/config.json",".github/workflows/ci.yml","packages/bee-rs/crates/bee/tests/proof_gate.rs","docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md"],
"read_first":["docs/history/clippy-warnings-clean/CONTEXT.md","packages/bee-rs/Cargo.toml"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md"],
"action":"Implement CONTEXT.md D1 and D2 on top of cwc-1. (1) packages/bee-rs/Cargo.toml gains [workspace.lints.clippy] too_many_arguments = \"allow\" and result_large_err = \"allow\"; crates/bee/Cargo.toml and crates/fleet/Cargo.toml gain [lints] workspace = true. Nothing else is allowed. (2) Fix every remaining finding of cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings in code, behavior unchanged: type_complexity with a type alias; large_enum_variant by boxing the large variant; doc lints by fixing the doc text indentation; dead_code: delete the item only when rg shows no caller in any target (tests included), otherwise make it used where it is meant to be or gate it with cfg(test) if only tests use it; private_interfaces by aligning visibility; the rest as clippy suggests. No #[allow] attribute on any item and no new code comment. (3) .bee/config.json: the clippy entry of commands.test ends in ' -- -D warnings'; .github/workflows/ci.yml clippy step runs cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings. Keep proof_gate green (adjust its expectations only if it pins the old clippy text). (4) docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md: the commands.test sentence says clippy runs with -D warnings and the two workspace allows, citing clippy-warnings-clean D1/D2 (never raw ids). Run bee knowledge check. Commit once (cell trailer). Proof: the verify command; quote the counts."},
{"id":"cwc-3","feature":"clippy-warnings-clean","title":"Verify bee-rs is clippy-clean with behavior unchanged","lane":"small","role":"test","deps":["cwc-2"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --lib --bins && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test proof_gate",
"must_haves":{"truths":["clippy -D warnings exits 0 on the default targets","the lib and bin test count is not lower than the base 5229 bin tests","the only workspace clippy allows are too_many_arguments and result_large_err and no item-level clippy allow was added","no code comment was added by cwc-1 or cwc-2","every deleted dead_code item has no caller in any target","the report under docs/history/clippy-warnings-clean/reports/ lists each check with its result"]},
"decisions":["4a500653-5751-44cf-9cd9-7186c25a056e","cf6ce5e3-b8dc-4973-b61a-094234fd3b21","e88c22c7-3bac-4130-a385-6709e7267d2e"],
"files":["docs/history/clippy-warnings-clean/reports/cwc-3.md"],
"read_first":["docs/history/clippy-warnings-clean/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract clippy-warnings-clean (decision 4a500653) on the cwc-1 and cwc-2 commits. Run the verify command. Diff the two commits against the feature base: rg the added lines for '#[allow(clippy' and for new comment lines (// or ///, not removals); list every deleted fn, struct, const or enum and rg the whole crate, tests included, for each name; read the non-mechanical hunks of cwc-2 for any behavior change (changed condition, changed output text, changed order). Compare the bin test count with the base 5229. Write docs/history/clippy-warnings-clean/reports/cwc-3.md with each check, the command and the result. Edit no source file. If a check fails, return BLOCKED with the evidence. Commit the report once (cell trailer)."}
]
```
