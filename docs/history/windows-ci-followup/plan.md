---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Make the last two Windows tests pass

## Summary

Windows CI on main still fails two tests after issues-sweep-1023. The
mailbox check refuses any path that contains `~`, so a Windows short name
like `RUNNER~1` is always refused. The new union-merge test compares line
endings that Windows git changes. This fixes both.

Mode: `small` — 1 risk flag: cross-platform.
Why this is the least workflow that protects the work: one product line and
one test fixture, each proven by its existing test.

## Requirements

- D1 (`26d96839`): the mailbox check refuses a path that starts with `~`, not
  every path that contains `~`.
- D2 (`26d96839`): the union-merge test fixture turns off `core.autocrlf`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | the mailbox check rejects any `~` | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:296 | if target.is_empty() \|\| target.contains(['$', '~', '*', '?', '[']) { |
| 2 | Windows CI still denies mv into the mailbox | ran | gh run view 39916634170 --log-failed | expected allow: mv 'C:\Users\RUNNER~1 |
| 3 | the union test fixture inits git with default settings | read | packages/bee-rs/crates/bee/src/onboard/tests.rs:970 | git_ok(&fx.repo, &["init", "-q", "-b", "main", "."]); |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"wcf-1 changes one guard line and one test fixture."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride the cell; Windows CI is the live proof."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader queues the capture stub."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"The release is its own lane."},
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
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Small lane: no hat wave."}
  ]
}
```

## Cells (current slice)

```json
[
  {
    "id": "wcf-1",
    "feature": "windows-ci-followup",
    "title": "Judge Windows short-name paths in the mailbox check and fix the union test line endings",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "26d96839-efa6-4ccc-a170-aa0fc455680c"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/windows-ci-followup/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "one-character scope change in an existing refusal; the existing read_only_table allow and deny rows pin it",
    "action": "Two Windows-only failures from CI run 39916634170. (1) Per D1: in hooks/worker_guard.rs `fn in_mailbox` returns false when `target.contains(['$', '~', '*', '?', '['])`; on Windows the temp path holds an 8.3 short name (`C:\\Users\\RUNNER~1\\...`), so every mv into the mailbox is denied and `hooks::worker_guard::tests::read_only_table` fails. Tilde expansion only happens at the start of a word, so refuse a target that starts with `~` and keep refusing any `$`, `*`, `?` or `[` anywhere. Add rows to `read_only_table` (or a new test beside it): a mailbox path whose directory name holds `~` in the middle (build it with a real temp dir child named like `RUN~1`, created on disk) is allowed for mv into the mailbox, and `mv ~/a <mailbox>/b` is still denied; watch the middle-tilde row fail before the fix. (2) Per D2: in onboard/tests.rs `two_branches_appending_to_the_decision_log_merge_cleanly_after_onboarding` inits a git repo with default settings, and the Windows runner's core.autocrlf turns the merged file into CRLF, so the LF comparison fails. Right after the `init` call add `git_ok(&fx.repo, &[\"config\", \"core.autocrlf\", \"false\"]);`. No other changes. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee two_branches_appending",
    "must_haves": {
      "truths": [
        "a mailbox path with ~ in the middle of a directory name is judged by where it points",
        "a path that starts with ~ is still refused",
        "the union-merge test does not depend on core.autocrlf"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs", "substantive": "leading-tilde refusal and a middle-tilde allow test"}
      ],
      "key_links": ["in_mailbox feeds the mv and redirect checks"],
      "prohibitions": ["No other change to the read-only guard", "No code comments"]
    }
  }
]
```
