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
| 6 | **(corrected)** On Windows the bare name `bash` resolves to the WSL launcher, not Git Bash — this IS the release-dirt cause. The original row claimed the opposite from the fact that other `bash` tests were absent from the failure list; none of them ever ran bash on Windows (the five in `statusline_contract.rs` are `#[cfg(unix)]`, and the release-authorization test returns early on every platform because it looks for `packages/scripts/release.sh`). | ran | `gh run view 35453818440 --log-failed` | stdout, decoded from UTF-16: `Windows Subsystem for Linux has no installed distributions` |
| 7 | CRLF is not the cause. | read | `.gitattributes:15` | `* text=auto eol=lf` |

## Approach

One cell. The path fix follows the file's existing idiom, and the diagnostics
change is a message-only edit, so the two sit together safely. After it lands, the
branch is pushed and the Windows run is read for the release-dirt cause.

SMALLER PATH check: fix only the worktree test and leave release-dirt red? That
leaves the workflow red and still unexplained. FAIL — both are in scope.

## Cells — current slice (preview)

Slice 3. The branch run 35454713330 proved win-2: release-dirt passed on Windows. The worktree test then failed one assertion later (line 6535): the new-worktree instruction also carries the raw path. win-2 traced only the failing line, not the later ones that share its expected value.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `win-3` | Match the new-worktree instruction path to what the product emits | one test file | — | Windows workflow green | full bin suite green on Linux; the Windows run on the branch |

```json
[
  {
    "id": "win-3",
    "feature": "windows-ci-green",
    "title": "Match the new-worktree instruction path to what the product emits",
    "lane": "tiny",
    "role": "test",
    "status": "open",
    "deps": [],
    "decisions": [
      "D1",
      "D3",
      "00967a93-511b-46bb-8b0e-f06e9bef8623"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs inside `fn enter_and_merge_and_new_carry_session_runtime_and_instruction_with_injected_caller`, section 3 (new_worktree_transition_result_and_text) expects `/move {target_str}`, where target_str is canonical. The product builds that instruction from `p(&created.worktree_root)` (handlers.rs, new_worktree_transition_result_and_text), the raw path, so Windows run 35454713330 failed at line 6535 with RUNNER~1 vs runneradmin. Build that one expectation from `p(&created.worktree_root)`. Leave section 1 and section 4 on target_str: both go through enter_worktree_core, which emits the canonical path. No product change.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee",
    "must_haves": {
      "truths": [
        "The new-worktree expectation matches the raw path the product emits",
        "The enter expectations stay canonical",
        "The full bin suite stays green on Linux"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
          "substantive": "section 3 expectation built from p(&created.worktree_root)"
        }
      ],
      "key_links": [
        "each expectation mirrors the path builder of the product function it asserts"
      ],
      "prohibitions": [
        "No product source change",
        "No weakening of an assertion condition"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

### Capped packet, slice 2 (record only)

```text
[
  {
    "id": "win-2",
    "feature": "windows-ci-green",
    "title": "Run release-dirt under a real Windows bash, and match the merge path to what the product emits",
    "lane": "small",
    "role": "test",
    "status": "open",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/shell.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "docs/history/windows-ci-green/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Two corrections, both in test code, both proven by the branch's own Windows run 35453818440. FIRST, in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs inside `fn test_release_dirt_script_filters_only_bee_paths`: both script runs call `Command::new(\"bash\")`, and on Windows that bare name resolves to the WSL launcher in C:\\Windows\\System32 \u2014 the run printed \"Windows Subsystem for Linux has no installed distributions\" in UTF-16 on stdout. The product already solves exactly this: read the header comment of packages/bee-rs/crates/bee/src/shell.rs, which describes this failure, and use `crate::shell::command()`, which pins the child PATH to a real Win32 bash. BUT use it on Windows ONLY. On every other platform `crate::shell::command()` returns /bin/sh, not bash (see `fn resolve` in shell.rs), and release-dirt.sh needs real bash \u2014 it uses [[ ]], ${x:0:2}, process substitution and read -d ''. On Ubuntu /bin/sh is dash and the script would break. So: on Windows build the command from `crate::shell::command()` (expect it to be present there, with a message naming the missing Win32 bash), elsewhere keep `Command::new(\"bash\")`. Put a one-line comment beside the branch saying why, citing shell.rs. Keep the diagnostic assertion messages from win-1 as they are. SECOND, in packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs inside `fn enter_and_merge_and_new_carry_session_runtime_and_instruction_with_injected_caller`: win-1 changed BOTH expectation sites to canonical_path_str. The Windows run proved only the ENTER site needed it \u2014 the test now passes the enter assertion and fails the merge one at line 6517, because the product emits the merge instruction with the unconverted short path (RUNNER~1) while the enter instruction is canonical. Revert ONLY the merge site: `let main_str = canonical_path_str(&main).unwrap();` goes back to `let main_str = p(&main);`. Leave the enter site canonical. Do not change the product to make the two consistent; that inconsistency is filed separately. Do not touch scripts/release-dirt.sh.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee",
    "must_haves": {
      "truths": [
        "On Windows the release-dirt script runs through crate::shell::command(), so it reaches a real Win32 bash",
        "On every other platform the release-dirt script still runs under bash, not /bin/sh",
        "The worktree test's enter expectation stays canonical and its merge expectation matches the raw path the product emits",
        "The full bin suite stays green on Linux"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
          "substantive": "a Windows-only branch to crate::shell::command() with a comment citing shell.rs"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
          "substantive": "the merge site reverted to p(&main); the enter site unchanged"
        }
      ],
      "key_links": [
        "the Windows bash comes from the product's own resolver, not a second implementation of it"
      ],
      "prohibitions": [
        "No use of crate::shell::command() off Windows, where it is /bin/sh",
        "No change to scripts/release-dirt.sh or any product source file",
        "No weakening of an assertion condition"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

### Capped packet, slice 1 (record only)

```text
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
