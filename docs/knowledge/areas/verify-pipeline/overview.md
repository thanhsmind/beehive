---
type: bee.area
title: "Verify Pipeline — purpose, test execution, and proof discipline"
description: "The verification pipeline that proves correctness: running declared test commands, managing concurrency, ensuring hermetic runs, and validating skill pointer integrity."
timestamp: 2026-08-22
bee:
  id: verify-pipeline-overview
  lifecycle: active
  areas: [verify-pipeline]
  required_context: []
  decisions: ["contention-split D1-D6 (decision 1ce777d9)", "verify-scoping D1/D2 (decisions e39d3f89, 20534ea9)", "412e9b3a (commands.test is the one declared test path)"]
  sources: ["docs/history/test-economy/CONTEXT.md", "docs/specs/verify-pipeline.md#R1", "docs/specs/verify-pipeline.md#R2"]
  authoritative_for: "verify-pipeline: purpose, test execution, and proof discipline"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/test_runner.rs]
  owns.skills: []
  owns.tests: [packages/bee-rs/crates/bee/tests/proof_gate.rs, packages/bee-rs/crates/bee/tests/concurrency.rs]
---

# Verify Pipeline — Purpose, Test Execution, and Proof Discipline

## Purpose

The verify pipeline guarantees that code changes, documentation, and skill definitions
satisfy their declared invariants before landing. It provides hermetic, contention-free
verification across worktrees and enforces proof-of-correctness gates across the bee
lifecycle.

## How this area is split

- Test execution and hermetic runs: `concurrency-and-hermetic-runs.md`.
- Skill reference and pointer verification: `skill-reference-pointer-integrity.md`.
- Historical suite discovery and caching: `suite-topology-and-discovery.md`, `suite-result-cache.md`.

## Entry Points & Triggers

- **Cell verify** — executed during `bee cells finish` and cell completion checks.
- **Merge verification** — runs declared test suites during `bee worktree merge`.
- **Knowledge verification** — runs `bee knowledge check` and `bee knowledge index --check`.

## Data Dictionary

| Element | Meaning |
|---|---|
| declared test command | Configured project test command in `.bee/config.json` (`commands.test`). |
| proof line | Triple `<command> — <result> — <scope reason>` recorded on cell completion. |
| pointer integrity | Verification ensuring all doc and skill cross-references resolve. |

## Actors & Access

- **Orchestrator and workers** — execute scoped verification and record proof lines.
- **CI / merge gate** — executes full project verification prior to integration.

## Business Rules

- A red verification result refuses cell completion and merge unconditionally.
- Proof lines must name fresh command output and valid scope justifications.

## Pointers (implementation)

- Test runner invocation: `packages/bee-rs/crates/bee/src/verbs/test_runner.rs`.
- Proof gate tests: `packages/bee-rs/crates/bee/tests/proof_gate.rs`.
- Concurrency tests: `packages/bee-rs/crates/bee/tests/concurrency.rs`.

## Concepts in this area

- [Verify Pipeline — concurrency safety and hermetic runs](concurrency-and-hermetic-runs.md) — Keeping whole-tree regeneration lock-serialized and atomic-swapped, keeping every child suite hermetic to session identity, and the deterministic race/isolation proofs that back both claims.

## Patterns in this area

- [A test can assert, as intended behavior, the very escape a fix is dispatched to remove](../../patterns/20260806-a-test-can-assert-the-very-escape-a-fix-is-dispatched-to-remove.md) — A guard scoped to one departure path had a test asserting it never fires on any other path — the escape itself, written down as a promise — so closing the hole meant inverting that test's assertion, and a worker that treats a green test as a specification would have concluded the fix was wrong.
- [A platform job red on a build error is an unknown number of failures](../../patterns/20260821-a-platform-job-red-on-a-build-error-is-an-unknown-number-of-failures.md) — A compile error on one platform hides every test behind it, so the visible failure count is one and the real one is unknown until the build is green
- [A guard deleted with its runtime is a guard removed](../../patterns/20260825-a-guard-deleted-with-its-runtime-is-a-guard-removed.md) — A guard deleted with its runtime is a guard removed
- [A rule living in N places needs one test that reads all N](../../patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md) — When one command or value must be identical in several files, discipline and docs will not keep them synced — one test that reads every copy and diffs them will, and it catches the drift before it exists.
- [A wrapper on PATH is not the binary a derivation reads](../../patterns/20260826-a-wrapper-on-path-is-not-the-binary-a-derivation-reads.md) — A test that derives facts from an installed binary's own bytes silently derives nothing when PATH resolves to a wrapper script or a versionless shim
- [Verify strings are authored, not just read — two traps](../../patterns/20260708-verify-strings-are-authored-not-just-read.md) — A cell’s verify command must be dry-run once before it reaches a worker, not reviewed as prose
- [A frozen assertion can encode the defect it guards — the worker must stop, not rewrite](../../patterns/20260710-a-frozen-assertion-can-encode-the-defect-it.md) — The worker must stop, not rewrite
- [A non-exposure invariant needs a test on every output surface it crosses](../../patterns/20260710-a-non-exposure-invariant-needs-a-test-on.md) — A non-exposure invariant needs a test on every output surface it crosses
- [A NUL byte in a source file makes grep silently match nothing](../../patterns/20260710-a-nul-byte-in-a-source-file-makes.md) — A NUL byte in a source file makes grep silently match nothing
- [Scope an incident-born check to the defect class, never the first location](../../patterns/20260710-scope-an-incident-born-check-to-the-defect.md) — Scope an incident-born check to the defect class, never the first location
- [A removal is verified by its invariants, not the names it deletes](../../patterns/20260711-a-removal-is-verified-by-its-invariants-not.md) — A removal is verified by its invariants, not the names it deletes
- [A reviewer's cited line is a sample of a class — sweep the diff before re-review](../../patterns/20260711-a-reviewers-cited-line-is-a-sample-of.md) — Sweep the diff before re-review
- [Pre-code gates filter spec defects; only diff review catches implementation defects](../../patterns/20260711-pre-code-gates-filter-spec-defects-only-diff.md) — Pre-code gates filter spec defects; only diff review catches implementation defects
- [Dry-run negative-grep verifies against their own fixtures](../../patterns/20260712-dry-run-negative-grep-verifies-against-their-own.md) — Dry-run negative-grep verifies against their own fixtures
- [Empty child-process output can be a sandbox denial, not a regression](../../patterns/20260712-empty-child-process-output-can-be-a-sandbox.md) — Empty child-process output can be a sandbox denial, not a regression
- [Async assertions under a non-awaiting runner pass vacuously](../../patterns/20260714-async-assertions-under-a-non-awaiting-runner-pass.md) — Async assertions under a non-awaiting runner pass vacuously
- [A freeze fixture's wrapper verify must assert a printed sentinel, not a filename or bare exit](../../patterns/20260715-a-freeze-fixtures-wrapper-verify-must-assert-a.md) — A freeze fixture's wrapper verify must assert a printed sentinel, not a filename or bare exit
- [A cell that changes a shared mutator surface re-runs the sibling suites of that surface — its own new suite is not enough](../../patterns/20260720-a-cell-that-changes-a-shared-mutator-surface.md) — A cell touching a shared guard/dispatch surface must re-run every sibling suite that exercises it, not just its own new suite
- [Local green is worthless if suites inherit harness env — hermeticity must be structural, and the release gate is the exact-tag CI](../../patterns/20260721-local-green-is-worthless-if-suites-inherit-harness.md) — Hermeticity must be structural, and the release gate is the exact-tag CI
- [Race tests assert structure, never scheduler luck — and a race harness that hides the child's stderr is itself a bug](../../patterns/20260721-race-tests-assert-structure-never-scheduler-luck.md) — And a race harness that hides the child's stderr is itself a bug
- [A scan scope set from assumption passes green while hiding the very bug it was built to catch](../../patterns/20260723-a-scan-scope-set-from-assumption-passes-green-while-hiding-the-bug.md) — Three times in one session a hand-chosen scan scope was one directory too narrow and reported clean. The scope is the finding, not the hits — derive it by measurement before you trust a green.
- [A system contradicting itself hides in the seam, because each half passes inspection alone](../../patterns/20260723-a-system-contradicting-itself-hides-in-the-seam.md) — One mechanism produces what another must consume, reclaim or check — and the contradiction survives every green run, because no unit test owns a handoff.
- [Clearing a red by widening the threshold is not the same act as correcting what is measured — prove which one you did](../../patterns/20260723-clearing-a-red-by-widening-the-threshold-is-not-fixing-the-check.md) — Both clear the red, both read identically in a commit summary, and only one leaves the guard able to detect what it exists to detect. A negative control is what separates them.
- [A red-first proof whose oracle can be fed by live-environment detection proves nothing about the code under test](../../patterns/20260724-red-first-oracle-fed-by-live-environment.md) — Reverting a version-pin constant stayed green locally because the machine's live CLI version equals the new pin — the assertion string was satisfied by live detection, not by the constant. A local red-first that can be fed by the environment is not a red floor; require an environment-independent proof.
- [A verify command that is a binary invocation is unowned by CI and rots silently](../../patterns/20260726-a-binary-invocation-verify-is-unowned-by-ci.md) — One cell's verify was a parity binary run; the scheduled build ran only the compiler and the test suite, so nothing re-executed it. A later cell grew the shared fixture, the binary's safety check refused the new shape, and that verify stayed red across an entire slice while build, suite and branch status all stayed green.
- [A harness that inherits ambient environment shrinks its own proof without ever going red](../../patterns/20260726-a-harness-that-inherits-ambient-env-shrinks-its-proof.md) — A parity harness inherited the launching shell's session identifier, which resolved to a session absent from the fixture, so the branch that serializes a full lane record never executed on any developer machine — and a real key-order break lived there. Both legs read the same environment, so correctness was never at risk; only the coverage silently shrank.
- [An empty-collection fall-through silently replaces the branch a test's name claims](../../patterns/20260726-an-empty-collection-fallthrough-replaces-the-branch-a-test-names.md) — A test named for a synchronization step's authoritative rebuild seeded no work records, so every case took the no-records shortcut; the assertions were real, the oracle was real, and the branch the cell existed to port was never executed. The removal proof — neutralize the seeding and require the test to fail — is what separates a named branch from an exercised one.
- [An impacted-test run computed after the commit selects nothing, and caps false-green](../../patterns/20260727-an-impacted-run-computed-after-the-commit-selects-nothing.md) — A worker committed its change, then ran run_verify.mjs --impacted-from-git, which diffed against the now-clean tree and saw only leftover uncommitted bookkeeping files — reporting 0 suites and capping the cell verify_passed:true while the change was in fact red.
- [A derivation the tooling already computes is worthless where doctrine forbids it](../../patterns/20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed.md) — The impact registry already maps every source file to the suites that consume it, but a cell's verify is doctrinally forbidden from using it — so sibling-suite selection fell back to human memory, and two cells editing the same file made opposite guesses.
- [A scan set built from the git index crashes the very gate it feeds](../../patterns/20260728-a-scan-set-from-the-git-index-crashes-the-gate-that-guards-it.md) — A coverage gate listed its inputs with git ls-files — the index, not the working tree — so a deferred deletion left it reading a file that no longer existed; the ENOENT killed the process and took every assertion behind it into silence.
- [A contract written by one unit and read by another owes a row that crosses the seam](../../patterns/20260729-a-contract-written-by-one-unit-and-read-by-another-owes-a-row-that-crosses-the-seam.md) — A contract written by one unit and read by another owes a row that crosses the seam
- [A test selector that matches nothing reports green](../../patterns/20260729-a-selector-that-matches-nothing-reports-green.md) — A test selector that matches nothing reports green
- [A test run under a configuration must assert the configuration is live](../../patterns/20260729-a-test-run-under-a-configuration-must-assert-the-configuration-is-live.md) — A test run under a configuration must assert the configuration is live
- [An advisory check that is wrong survives, because it cannot cost anything](../../patterns/20260729-an-advisory-check-that-is-wrong-survives-because-it-cannot-cost-anything.md) — A false-positive matcher reported a reachable pointer as missing for months; two workers in one session read the warning, wrote it off as pre-existing noise, and moved on — a blocking check with the same defect would have been fixed the day it landed.
- [A guard test that has never failed proves only that the guard agrees with itself](../../patterns/lane-model-diversity-lmd-2-pitfall.md) — Falsify a new contract test with a temporary reverted mutation on both the refusal and admit arms before trusting it
