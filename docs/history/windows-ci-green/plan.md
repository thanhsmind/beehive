# Plan: Windows CI green

## Summary

Two tests fail only on Windows. One builds its expected path without the
canonicalization the product applies, so on Windows the short name `RUNNER~1`
meets the long name `runneradmin` and they never match. The other fails with an
empty error, so nobody can see why.

This fixes the first outright and makes the second show its real error. The
branch's own Windows run then names that cause, and it gets fixed from evidence.

Mode: `small` — no risk flags; test code only, two files.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The worktree test fails on Windows because the two sides spell the same folder differently. | ran | `gh run view 35449141547 --log-failed` | `left: String("Call EnterWorktree with path=C:\\Users\\runneradmin\\AppData\\Local\\Temp\\.tmpKNPkzw\\main--wt--hwr-test-caller.")` / `right: String("Call EnterWorktree with path=C:\\Users\\RUNNER~1\\AppData\\Local\\Temp\\.tmpKNPkzw\\main--wt--hwr-test-caller.")` |
| 2 | The product canonicalizes with `dunce`, which resolves 8.3 short names. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:38` | `pub(crate) fn canonical_path_str(path: &Path) -> Result<String, String> { dunce::canonicalize(path)` |
| 3 | The same test file already builds expected paths with that function. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:5896` | `assert_eq!(enter["sourceCwd"], json!(canonical_path_str(&src).unwrap()));` |
| 4 | The failing test builds its expected path without it. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:6494` | `let target_str = p(&created.worktree_root);` |
| 5 | The release-dirt test hides its own cause. | ran | `gh run view 35449141547 --log-failed` | `release-dirt.sh failed: ` followed by an empty line — stderr empty, exit code and stdout not printed |
| 6 | `bash` resolves to a working shell on the Windows runner, so the launcher is not the cause. | ran | `gh run view 35449141547 --log-failed` | neither `tests.rs:11219`'s release-authorization test nor any of the five `statusline_contract.rs` tests appears in the failure list, and all of them call `Command::new("bash")` |
| 7 | CRLF is not the cause. | read | `.gitattributes:15` | `* text=auto eol=lf` |

## Approach

One cell. The path fix follows the file's existing idiom, and the diagnostics
change is a message-only edit, so the two sit together safely. After it lands, the
branch is pushed and the Windows run is read for the release-dirt cause.

SMALLER PATH check: fix only the worktree test and leave release-dirt red? That
leaves the workflow red and still unexplained. FAIL — both are in scope.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `win-1` | Canonicalize the worktree test's expected paths, and make the release-dirt test show its real error | two test files | — | The worktree test agrees with the product on Windows, and the release-dirt failure prints something a person can act on | the full bin suite green on Linux; the Windows run is read after the branch push |

```json
[
  {
    "id": "win-1",
    "feature": "windows-ci-green",
    "title": "Canonicalize the worktree test's expected paths, and make the release-dirt test show its real error",
    "lane": "small",
    "role": "test",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D2"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "docs/history/windows-ci-green/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Two edits, both in test code. FIRST, in packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs, inside `fn enter_and_merge_and_new_carry_session_runtime_and_instruction_with_injected_caller`: the expected paths are built with `p(...)`, which does not canonicalize, while the product builds the real value with `canonical_path_str` (verbs/worktree/handlers.rs:38), which runs dunce::canonicalize and so turns the Windows 8.3 short name RUNNER~1 into the long name runneradmin. Replace BOTH expectation sites in that test — `let target_str = p(&created.worktree_root);` and `let main_str = p(&main);` — with `canonical_path_str(&...).unwrap()`. This is the idiom the same file already uses; search it for `canonical_path_str(&` and match those lines exactly. Do not change the product, and do not change what the assertions check. SECOND, in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs, inside `fn test_release_dirt_script_filters_only_bee_paths`: both script runs assert success with a message that prints only stderr, and on Windows stderr was empty, so the failure named no cause. Change BOTH assertion messages — the one on `out` and the one on `out_clean` — to print the exit status, stdout and stderr together, labelled, so the next Windows run shows what actually happened. Change only the message: the condition stays `status.success()`. Do NOT touch scripts/release-dirt.sh: its cause is unknown, and changing a script the release path uses on a guess is out of scope for this cell.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee",
    "must_haves": {
      "truths": [
        "Both expected paths in the worktree test are built with canonical_path_str",
        "Both release-dirt assertion messages print the exit status, stdout and stderr",
        "No assertion condition changed, only how the expectations are built and what the messages print",
        "The full bin suite stays green on Linux"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "substantive": "both expectation sites use canonical_path_str, matching the file's own idiom"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "both script-run failure messages carry exit status, stdout and stderr"}
      ],
      "key_links": ["the expected path goes through the product's own canonicalizer rather than a second implementation of it"],
      "prohibitions": [
        "No change to scripts/release-dirt.sh",
        "No change to any product source file",
        "No weakening of an assertion condition to make a test pass"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| # | Scenario | Pass when |
|---|---|---|
| 1 | Worktree test on Linux | still passes |
| 2 | Worktree test on Windows (branch run) | passes: both sides now carry the long name |
| 3 | Release-dirt test on Windows (branch run) | fails, but now prints exit code, stdout and stderr |
| 4 | Full bin suite on Linux | green |

## Open Questions

- Why does `release-dirt.sh` fail on Windows? Answered by row 3 of the matrix, not in advance.
