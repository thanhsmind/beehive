---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Make Windows CI green on main

## Summary

Windows CI has failed on main for every run on 2026-11-07, the 2.83.0
release commit included. The same 9 tests fail each time; Linux passes all
of them. Each failure is a test or fixture that assumes a Unix detail: a
backslash path pasted raw into JSON, git line-ending conversion, the
binary name without `.exe`, or a shell and LF output. This plan fixes each
at its cause.

Mode: `small` — 1 risk flag: cross-platform.
Why this is the least workflow that protects the work: the changes sit in
test fixtures, with a product change only where the product is wrong on
Windows; Linux tests pin that Linux does not change, and Windows CI on the
pushed branch is the live proof.

## Requirements

- D1 (`6a122360`): a test that splices a temp path into JSON escapes it as a JSON string.
- D2 (`6a122360`): a test that compares git-written file bytes turns off `core.autocrlf` in its fixture repo.
- D3 (`6a122360`): the vendored binary name is platform-specific (`.bee/bin/bee.exe` on Windows); onboard tests expect the platform name.
- D4 (`6a122360`): `merge_full_suite` and `output_spill` fixtures run on Windows; a product change only where the product is wrong on Windows.
- D5 (`eeaff7c5`, plan rev 1): the spill test pins cli-output-spill D3 on both platforms: unix spills, windows passes through unchanged.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The same tests fail on the release commit and on e118d3ac5 | ran | gh run view 42338862370 --log-failed | result: FAILED. 4587 passed; 4 failed |
| 2 | The steer_to fixture pastes the raw cwd into JSON | read | packages/bee-rs/crates/bee/src/herding/run.rs:13688 | listing.replace("CWD", &opts.cwd.display().to_string()) |
| 3 | The worktree restore test sees CRLF | ran | gh run view 42342606992 --log-failed | left: "main conflict\r\n" |
| 4 | Onboard plans the Windows binary name | ran | gh run view 42342606992 --log-failed | left: String(".bee/bin/bee.exe") |
| 5 | The merge door tests fail at bee test | ran | gh run view 42342606992 --log-failed | bee test itself ran the suite |
| 6 | The spill test finds no marker line | ran | gh run view 42342606992 --log-failed | exactly one marker line |

Red evidence is Windows CI only; these tests are green on Linux, so each
cell's red is the CI log above, and its green is Windows CI on the pushed
branch.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wc7-1 to wc7-4 change test fixtures and, where needed, Windows product paths."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each cell; Windows CI on the branch is the live proof."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader records the capture."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Small lane: no consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Small lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Small lane: no hat wave."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Small lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Small lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Cells (current slice)

All four cells touch disjoint files and run in parallel.

```json
[
  {
    "id": "wc7-1",
    "feature": "windows-ci-1107",
    "title": "Escape the temp path in the steer_to listing fixture and pin LF in the worktree restore test",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "6a122360-92d4-44a5-a2d1-417e2b05fac6"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/windows-ci-1107/plan.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "test-fixture-only change; the two named tests pin it on both platforms",
    "action": "Two Windows-only failures from CI run 42342606992. (1) Per D1: herding::run::tests::claude_session_paseo_claude_run_records_the_one_matching_session_as_steer_to fails on Windows with steer_to Null because paseo_claude_run_with_listing (run.rs about line 13688) replaces CWD in a JSON listing with the raw cwd display string; a Windows path holds backslashes, so the listing is not valid JSON. Replace CWD with the cwd encoded as a JSON string body (serde_json::to_string of the display string, outer quotes stripped), so every test that uses the helper gets a valid listing on every platform. Check sibling helpers in the same test module that splice CWD the same way and fix them the same way. (2) Per D2: verbs::worktree::tests::uncommitted_bee_store_changes_on_main_with_conflict_restores_main (tests.rs about line 2729) reads \"main conflict\\r\\n\" on Windows; the fixture repo inherits core.autocrlf. Turn off core.autocrlf in the fixture repo that test uses (prefer the shared fixture init if it is the one place, so every worktree test is line-ending stable). Change test code only. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- claude_session_paseo_claude_run uncommitted_bee_store_changes_on_main",
    "must_haves": {
      "truths": [
        "the steer_to listing fixture is valid JSON when the cwd holds backslashes",
        "the worktree restore test does not depend on core.autocrlf"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "the listing fixture JSON-escapes the cwd"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "substantive": "the fixture repo sets core.autocrlf false"}
      ],
      "key_links": ["paseo_claude_run_with_listing feeds the steer_to tests"],
      "prohibitions": ["No product code change", "No code comments"]
    }
  },
  {
    "id": "wc7-2",
    "feature": "windows-ci-1107",
    "title": "Expect the platform binary name in the vendored-binary onboard tests",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D3", "6a122360-92d4-44a5-a2d1-417e2b05fac6"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs"
    ],
    "read_first": [
      "docs/history/windows-ci-1107/plan.md",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "Windows expectation in two onboard tests; a plan.rs change only if the plan is wrong on Windows",
    "action": "Two Windows-only failures from CI run 42342606992. (1) onboard::tests::vendored_binary_planned_when_missing_and_hooks_expect_it asserts the copy_binary path is \".bee/bin/bee\"; on Windows the plan names \".bee/bin/bee.exe\", which is the correct vendored name there (hooks try bee then bee.exe). Make the test expect the platform name (\".bee/bin/bee\" plus std::env::consts::EXE_SUFFIX). (2) onboard::tests::hook_wiring_does_not_depend_on_a_vendored_binary_and_never_stacks writes a fake .bee/bin/bee.exe (\"MZ\") and asserts the re-plan has 0 items; on Windows it has 1. Find which item it is (most likely copy_binary, because on Windows bee.exe is the vendored name and its bytes differ from the current binary). The test's claim is that vendoring never re-wires hooks: keep that claim, and assert it on the hook items only, or vendor the fake at the name that is NOT the platform's vendored path, whichever keeps the test honest on both platforms. If the item is a hook re-wire, that is a product bug in onboard/plan.rs; fix it there instead. Linux behaviour unchanged. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard::tests::",
    "must_haves": {
      "truths": [
        "the vendored-binary test expects the platform binary name",
        "the never-stacks test still proves vendoring does not re-wire hooks"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/tests.rs", "substantive": "platform-aware expectations in both tests"}
      ],
      "key_links": ["compute_plan emits copy_binary"],
      "prohibitions": ["No weakening of the no-re-wire claim", "No code comments"]
    }
  },
  {
    "id": "wc7-3",
    "feature": "windows-ci-1107",
    "title": "Run the merge full-suite door tests on Windows",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D4", "6a122360-92d4-44a5-a2d1-417e2b05fac6"],
    "files": [
      "packages/bee-rs/crates/bee/tests/merge_full_suite.rs"
    ],
    "read_first": [
      "docs/history/windows-ci-1107/plan.md",
      "packages/bee-rs/crates/bee/tests/merge_full_suite.rs",
      "docs/history/full-suite-once/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "Windows fixture for an integration test file; product files only by named deviation",
    "action": "Four Windows-only failures in tests/merge_full_suite.rs from CI run 42342606992: a_fresh_green_record_merges_and_the_door_runs_no_test panics 'bee test itself ran the suite' (line about 162); a_code_feature_with_no_test_record_is_refused_naming_bee_test_in_the_worktree, a_red_record_is_refused and a_later_docs_only_commit_keeps_the_record_fresh (WORKTREE_MERGE_WORKTREE_DIRTY) follow. Find why the fixture's declared test command does not run or does not write its marker on Windows (a sh-only command, a path with backslashes in a shell string, LF assumptions, or files the fixture writes into the worktree that git then sees as dirty, e.g. through core.autocrlf), and make the fixture run on Windows: a command form that runs under the shell bee uses on Windows, and core.autocrlf false in the fixture repos if line endings make the tree dirty. If bee test or worktree merge itself is wrong on Windows, stop and report [BLOCKED] naming the product file and line instead of editing it. Linux behaviour unchanged. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test merge_full_suite",
    "must_haves": {
      "truths": [
        "the fixture's test command runs under the shell bee uses on each platform",
        "the fixture worktree is clean after its own commits on Windows"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/merge_full_suite.rs", "substantive": "platform-correct test command and line-ending stable fixture"}
      ],
      "key_links": ["bee test writes .bee/logs/test-results.json that worktree merge reads"],
      "prohibitions": ["No product code change without a [BLOCKED] report", "No code comments"]
    }
  },
  {
    "id": "wc7-4",
    "feature": "windows-ci-1107",
    "title": "Pin the unix-only output spill in its test on both platforms",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D4", "D5", "6a122360-92d4-44a5-a2d1-417e2b05fac6", "eeaff7c5-e3ae-4692-856e-b805f43fd199"],
    "files": [
      "packages/bee-rs/crates/bee/tests/output_spill.rs"
    ],
    "read_first": [
      "docs/history/windows-ci-1107/plan.md",
      "packages/bee-rs/crates/bee/tests/output_spill.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "Windows fixture for one integration test; product files only by named deviation",
    "action": "Per D5: oversized_allowlisted_output_spills_to_a_file in tests/output_spill.rs fails on Windows because the spill capture is cfg(unix) only by cli-output-spill D3 (output_spill.rs:8-19). Split the assertion by platform: under cfg(unix) keep today\u2019s spill assertions unchanged; under cfg(not(unix)) assert the same oversized allowlisted run exits like the unwrapped run, prints the full output unchanged, prints no \u0027[bee] output spilled:\u0027 marker and leaves no spill file. Change test code only. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test output_spill",
    "must_haves": {
      "truths": [
        "on unix an oversized allowlisted run spills to a file with one marker", "on windows the same run passes through unchanged with no marker and no spill file"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/output_spill.rs", "substantive": "cfg-split assertions pinning D3"}
      ],
      "key_links": ["the spill wraps the read-verb output"],
      "prohibitions": ["No product code change", "No code comments"]
    }
  }
]
```
