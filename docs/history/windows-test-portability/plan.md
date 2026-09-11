---
artifact_contract: bee-plan/v1
mode: small
---

# Plan: windows-test-portability

## Summary

The Windows CI workflow is red on every run since commit `c5519899` (2026-09-06).
Thirteen tests fail on Windows only. The release binaries are not affected.
Each failure is a test that does not match how the product behaves on Windows.
This plan changes tests only. No product code changes.

Mode: `small`. Flags: none. Three test files change.
Why this is the least workflow that protects the work: the product behavior is
correct and stays as it is. Only the tests change so that they hold on every host.

## Requirements

- R1: The 11 onboard statusline tests pass on Windows.
- R2: `verbs::drivers::tests::prompt_skew_names_the_offending_file_and_its_remedy` passes on Windows.
- R3: `verbs::worktree::tests::enter_worktree_core_verifies_grant_and_git_link_with_zero_mutation` passes on Windows.
- R4: The PowerShell statusline skip in `onboard/mod.rs:396` stays. It is correct
  because `statusline-command.sh` is a bash script (Chesterton's fence).
- R5: The Linux suite stays green.

## Load-bearing claims

| # | Claim | Label | Anchor |
|---|-------|-------|--------|
| 1 | Onboarding skips the statusline when the host shell is PowerShell | read | `packages/bee-rs/crates/bee/src/onboard/mod.rs:396` |
| 2 | Without `host_shell` in `.bee/config.json`, the host decides, and Windows means PowerShell | read | `packages/bee-rs/crates/bee/src/onboard/merge.rs:48-58` |
| 3 | The onboard test fixture writes no `.bee/config.json`, so on Windows every statusline expectation fails | read | `packages/bee-rs/crates/bee/src/onboard/tests.rs:45-100` |
| 4 | The skew message prints the OS path, with `\` on Windows; the test looks for `/` | ran | Windows run 34573359675, `verbs/drivers/tests.rs:4739` |
| 5 | `worktreeRoot` comes from git (long path `runneradmin`), the test's path from the temp dir (short path `RUNNER~1`) | ran | Windows run 34573359675, `verbs/worktree/tests.rs:5987` |

## Cells

### Cell 1 — onboard statusline tests hold on a PowerShell host (`onboard/tests.rs`)

- The statusline section (from `// ── the default status display (4cac0774)`) pins the
  fixture repo to a POSIX host: a helper writes `.bee/config.json` `{"host_shell":"posix"}`.
  Tests in that section that write their own `.bee/config.json` keep `host_shell: "posix"` in it.
- `plan_on_an_empty_repo_lists_the_whole_install` and
  `apply_on_an_empty_repo_then_reapply_is_a_no_op` read
  `super::merge::host_shell_is_powershell(&fx.repo)` (the one source of the rule).
  On a PowerShell host they expect no statusline items and no `statusline` managed key.
- Red first: pin `{"host_shell":"powershell"}` for a local run on Linux. The same tests
  fail for the same reason. Then apply the fix and see them green.

### Cell 2 — path tests hold on Windows (`verbs/drivers/tests.rs`, `verbs/worktree/tests.rs`)

- Drivers: compare the message after `\` becomes `/`.
- Worktree: compare `worktreeRoot` and `created.worktree_root` after
  `dunce::canonicalize` of both, so the short and long forms of one folder are equal.

The two cells touch disjoint files and run in parallel.

## Proof

- Local: `cargo test -p bee` over the changed modules on Linux — green.
- Cell 1 also has a red-then-green proof on Linux through the PowerShell pin.
- Cell 2 changes are Windows-only behavior. Their final proof is the Windows CI run after merge.
