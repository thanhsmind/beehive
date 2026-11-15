---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: The onboard detector finds manifests in subdirectories

## Summary

The onboard convention detectors read only root files, so a monorepo such as
beehive (`packages/bee-rs/Cargo.toml`) gets no clippy offer. The detectors
now look at the root, then one and two directories down, and a nested
manifest names its location in the proposed command.

Mode: `small` — one file of code plus one concept; output is advisory text.

## Requirements (from CONTEXT.md)

- D1: root, depth 1, depth 2; sorted per depth; first match that satisfies the convention; skip hidden dirs and node_modules, target, vendor, dist, build; script manifests stay root-only.
- D2: nested commands name the location; root output byte-identical; marker words unchanged.
- C1: tests per contract nested-manifest-detect.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The clippy convention reads the root Cargo.toml only | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:225 | `("Cargo.toml", "", "cargo clippy -- -D warnings` |
| 2 | The convention loop joins the file name to the root | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:203 | `let text = std::fs::read_to_string(root.join(file)).ok();` |
| 3 | The test conventions check root markers only | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:131 | `exists(&root.join("go.mod"))` |

## Approach

Situation: `skills/bee-planning/playbooks/feature/feature.md`.

architect skipped: one private helper in one file; the data shape is a
relative manifest path, and D1 fixes the search order.

Two cells in series on the same file: mmd-1 (code) adds the bounded
manifest finder, uses it in both detectors, and syncs the concept; mmd-2
(test), a separate worker, verifies mmd-1 and adds the contract tests.

Rejected: an unbounded walk (slow, finds member crates first); parsing
workspace members (one parser per ecosystem); nested package.json scripts.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "mmd-1 writes the Rust detector change."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "mmd-2 verifies mmd-1 and adds the contract tests."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "The one concept line rides mmd-1."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Small lane; the tester cell is the independent check. Review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane; every claim row was read this session."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Small lane; rejected paths named."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Root output is byte-identical; only nested repos gain text."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Advisory text only; no door, guard or state write."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "The owner asked for this directly."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The leader read the one file directly."},
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
| A root repo's notice text changes | D2 keeps root text byte for byte; existing static_check tests stay green |
| The walk is slow on a big repo | Two levels only, skip list for vendored trees |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| mmd-1 | Find convention manifests up to two directories down | notices.rs, host-project-artifacts concept | — | beehive gets a clippy offer naming packages/bee-rs/Cargo.toml | notices tests |
| mmd-2 | Test nested manifest detection | notices.rs | mmd-1 | contract tests green | notices tests |

```json
[
{"id":"mmd-1","feature":"monorepo-manifest-detect","title":"Find convention manifests up to two directories down","lane":"small","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard::notices",
"must_haves":{"truths":["detect_static_checks on the beehive repo root returns cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings with source packages/bee-rs/Cargo.toml","a repo with a root Cargo.toml still gets cargo clippy -- -D warnings with source Cargo.toml","every existing onboard::notices test stays green"]},
"decisions":["ac0a62df-abf0-47e6-bb1d-c7055a1c9cb2","2b3c81e2-7531-4728-9efb-0a2eba4cff58"],
"files":["packages/bee-rs/crates/bee/src/onboard/notices.rs","docs/knowledge/areas/onboarding/host-project-artifacts.md"],
"read_first":["docs/history/monorepo-manifest-detect/CONTEXT.md","packages/bee-rs/crates/bee/src/onboard/notices.rs"],
"affects_skills":[],"affects_specs":["docs/knowledge/areas/onboarding/host-project-artifacts.md"],
"action":"Implement CONTEXT.md D1 and D2 in packages/bee-rs/crates/bee/src/onboard/notices.rs. Add one private finder that returns the candidate manifest paths relative to root for a file name (or a suffix such as .csproj): root first, then depth 1, then depth 2, each depth sorted by relative path, skipping directories whose name starts with '.' and node_modules, target, vendor, dist, build. In static_check_for, take the first candidate whose text satisfies the table (empty table = any) and build the command from its location: root keeps the exact current text; nested gives cargo clippy --manifest-path <rel> -- -D warnings, ruff check <dir>, go -C <dir> vet ./..., mypy <dir>; source = rel path; marker unchanged. In detect_commands conventions: pyproject -> pytest or pytest <dir>, csproj -> dotnet test (root, source = file name as today) or dotnet test <rel csproj>, go.mod -> go test ./... or go -C <dir> test ./...; source = rel path. Use forward slashes in rel paths. package.json, Makefile, composer.json stay root-only. Sync docs/knowledge/areas/onboarding/host-project-artifacts.md: the sentence that says detect_static_checks reads the project's own root files now says root then up to two directories down, cite docs/history/monorepo-manifest-detect/CONTEXT.md D1/D2 (never raw ids). Run bee knowledge check. No code comments. Do not add new tests; mmd-2 owns them. Proof: the verify command; quote the counts."},
{"id":"mmd-2","feature":"monorepo-manifest-detect","title":"Test nested manifest detection","lane":"small","role":"test","deps":["mmd-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard::notices",
"must_haves":{"truths":["a nested packages/app/Cargo.toml gives cargo clippy --manifest-path packages/app/Cargo.toml -- -D warnings","a root manifest beats a nested one","a manifest three directories down or under node_modules, target or a hidden directory is not found","nested go.mod gives go -C <dir> test ./... from detect_commands","commands_notices names the nested clippy command when commands.test lacks clippy"]},
"decisions":["641b2eb3-79a0-4aa8-bcb0-956dbbfdadc5","ac0a62df-abf0-47e6-bb1d-c7055a1c9cb2","2b3c81e2-7531-4728-9efb-0a2eba4cff58"],
"files":["packages/bee-rs/crates/bee/src/onboard/notices.rs"],
"read_first":["docs/history/monorepo-manifest-detect/CONTEXT.md","packages/bee-rs/crates/bee/src/onboard/notices.rs"],
"affects_skills":[],"affects_specs":[],
"action":"Verify the mmd-1 commit against CONTEXT.md D1 and D2, then add tests to the tests module of packages/bee-rs/crates/bee/src/onboard/notices.rs that check every clause of contract nested-manifest-detect (decision 641b2eb3). Test names carry static_check (trust-layers C1) or nested_manifest. Use temp-dir repos and the existing static_check_repo helper where it fits. Assert observable output (command, source) only. If mmd-1 breaks the contract, return BLOCKED with the failing output instead of changing the production code. No code comments. Proof: the verify command; quote the counts and name the new tests."}
]
```
