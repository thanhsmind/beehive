---
artifact_contract: bee-plan/v2
mode: tiny
---

# Plan: the release bee binary stops carrying the fake backend

## Summary

`FakeBackend` compiles into every build of the fleet crate. Gate it behind a
`test-support` feature that tests turn on through dev-dependencies, so a
release build of bee omits it.

Mode: `tiny` — two manifests and one module line; no behavior change.

## Requirements (from CONTEXT.md)

- D1: gate `fake` under `cfg(any(test, feature = "test-support"))` and enable the feature for fleet's and bee's tests.
- C1: contract fake-backend-gated.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The fake module is declared with no gate | read | packages/bee-rs/crates/fleet/src/backend.rs:12 | `pub mod fake;` |
| 2 | bee depends on fleet through the workspace entry | read | packages/bee-rs/crates/bee/Cargo.toml:11 | `fleet.workspace = true` |
| 3 | The workspace uses resolver 2, so dev-dependency features stay out of release builds | read | packages/bee-rs/Cargo.toml:2 | `resolver = "2"` |

## Approach

Situation: `skills/bee-planning/playbooks/refactor/refactoring.md`.

Pin first: the existing fleet and bee tests that use FakeBackend are the
record. One code cell makes the change; a separate test cell proves the tests
still run and that a release bee binary holds no FakeBackend symbol.

Rejected: bare `cfg(test)` (hides the fake from tests under `tests/` and from
the bee crate's tests).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "tch-3 makes the change."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "tch-4 proves tests and the release artifact."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "No behavior a concept describes changes."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Tiny lane; review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Tiny lane."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Tiny lane."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "No user-facing behavior change."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Test-support gating only."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Backlog item is owner-requested."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The anchors are known."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Both cells have a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "No prose deliverable."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Tiny lane."},
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
| A test target loses the fake and fails to compile | tch-4 runs the whole workspace test build |
| The feature leaks into the release binary through unification | tch-4 inspects the release artifact for the FakeBackend symbol |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| tch-3 | Gate the fleet fake backend behind test-support | fleet/src/backend.rs, fleet/Cargo.toml, bee/Cargo.toml | — | fleet and bee tests compile and pass | cargo test -p fleet -p bee build |
| tch-4 | Prove tests run and the release binary lacks the fake | report | tch-3 | no FakeBackend in release bee | release build symbol check |

```json
[
{"id":"tch-3","feature":"test-ci-hygiene","title":"Gate the fleet fake backend behind test-support","lane":"tiny","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p fleet",
"must_haves":{"truths":["fleet declares a test-support feature with no default","backend.rs declares pub mod fake under cfg(any(test, feature = test-support))","fleet and bee enable test-support only through dev-dependencies","cargo test -p fleet passes including tests/choreography.rs"]},
"decisions":["c597c5b7-1a53-4ee6-9e26-f554e30ab125","b0bcc68f-2daa-4d9a-8c8c-b8f7b870fe20"],
"files":["packages/bee-rs/crates/fleet/src/backend.rs","packages/bee-rs/crates/fleet/Cargo.toml","packages/bee-rs/crates/bee/Cargo.toml"],
"read_first":["docs/history/test-ci-hygiene/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D1. In packages/bee-rs/crates/fleet/Cargo.toml add [features] test-support = [] and [dev-dependencies] fleet = { path = \".\", features = [\"test-support\"] } so fleet's tests/ targets see the fake. In packages/bee-rs/crates/fleet/src/backend.rs change line 12 to #[cfg(any(test, feature = \"test-support\"))] pub mod fake;. In packages/bee-rs/crates/bee/Cargo.toml add [dev-dependencies] fleet = { workspace = true, features = [\"test-support\"] } (merge into an existing dev-dependencies table if there is one). Doc comments that name fake stay as they are. No code comments added, no behavior change. Run the verify, then cargo test --release -p bee --no-run to confirm bee's tests still compile. Commit once (cell trailer)."},
{"id":"tch-4","feature":"test-ci-hygiene","title":"Prove tests run and the release binary lacks the fake","lane":"tiny","role":"test","deps":["tch-3"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p fleet",
"must_haves":{"truths":["a release build of the bee binary contains no FakeBackend symbol","cargo build -p fleet without features fails to resolve fleet::backend::fake","fleet choreography tests and bee wave tests run and pass","the report under docs/history/test-ci-hygiene/reports/ quotes each result"]},
"decisions":["b0bcc68f-2daa-4d9a-8c8c-b8f7b870fe20","c597c5b7-1a53-4ee6-9e26-f554e30ab125"],
"files":["docs/history/test-ci-hygiene/reports/tch-4.md"],
"read_first":["docs/history/test-ci-hygiene/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract fake-backend-gated (decision b0bcc68f) on the tch-3 commit. Run 1: cargo build --release -p bee, then search the binary for FakeBackend with strings or rg -a -c FakeBackend on target/release/bee, expect zero. Run 2: confirm a non-test consumer cannot name the fake, for example cargo rustc -p fleet --lib --release with a temporary check is not allowed, so instead read cargo tree -p bee -e features and confirm test-support is absent from the normal bee build. Run 3: cargo test --release -p fleet -p bee -- choreography wave and quote the passing counts. Edit no source. Write docs/history/test-ci-hygiene/reports/tch-4.md with each command and its result verbatim. If a check fails, return BLOCKED with the evidence. Commit only the report (cell trailer)."}
]
```
