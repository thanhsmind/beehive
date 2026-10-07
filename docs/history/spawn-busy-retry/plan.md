# Plan: spawn-busy-retry

## Summary

`verbs::feedback` and `verbs::report` tests fail now and then (8 of 120 runs
under 6 parallel loops; Linux CI run 37581495733). Each test writes a stub
`gh` and bee then runs it. While another test thread forks, the child holds a
write descriptor to the new file, and Linux refuses to run it
(`ETXTBSY`). Renaming the file does not help, because the inode is the same.
`bee doctor` already retries exactly this error for its version probe. This
plan moves that retry into one helper and uses it for the two `gh` spawns.

Mode: `small` — 0 risk flags (4 files: one helper and its three callers)
Why this is the least workflow that protects the work: one small helper, one
error kind retried, three call sites.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | doctor already retries ETXTBSY | read | packages/bee-rs/crates/bee/src/doctor.rs:1243 | const PROBE_ETXTBSY_ATTEMPTS: u32 = 10; |
| 2 | feedback spawns gh once | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1856 | let output = std::process::Command::new(gh) |
| 3 | report spawns gh once | read | packages/bee-rs/crates/bee/src/verbs/report.rs:552 | let child = Command::new(program) |
| 4 | The flake reproduces locally under load | ran | bash stress-fb.sh (6 loops x 20 runs) | failures=8/120 |

## Cells (current slice)

```json
[
  {
    "id": "sbr-1",
    "feature": "spawn-busy-retry",
    "title": "Retry ExecutableFileBusy on bee's external spawns through one helper",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": [
      "5d55579f-1c3a-4c82-abea-5de3c0d86ad5"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/fsutil.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
      "packages/bee-rs/crates/bee/src/verbs/report.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
      "packages/bee-rs/crates/bee/src/verbs/report.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Move the ETXTBSY retry loop in doctor.rs installed_binary_bee_version (doctor.rs:1243-1263, PROBE_ETXTBSY_ATTEMPTS/PROBE_ETXTBSY_DELAY_MS) into one pub(crate) helper in fsutil.rs that takes a closure returning io::Result<T> and retries only std::io::ErrorKind::ExecutableFileBusy (10 tries, 20 ms apart), returning the last result. Use it in doctor.rs (same behavior), in feedback.rs ingest_issues for the gh issue list .output() (feedback.rs:1856), and in report.rs gh() for the Command spawn (report.rs:552). The helper retries nothing else. Unit-test the helper: a closure that fails with ExecutableFileBusy twice then succeeds returns Ok after 3 calls; a NotFound error is returned at once after 1 call. No new code comments. Proof: the cell verify, plus the leader's stress loop over verbs::feedback and verbs::report (6 parallel loops x 20 runs), which failed 8/120 before.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fsutil doctor verbs::feedback verbs::report",
    "must_haves": {
      "truths": [
        "One helper owns the ExecutableFileBusy retry and doctor uses it",
        "The feedback gh issue list and the report gh calls retry ExecutableFileBusy",
        "No other spawn error is retried"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/fsutil.rs",
          "substantive": "retry helper for ExecutableFileBusy"
        }
      ],
      "key_links": [
        "verbs::feedback and verbs::report stable under parallel stress"
      ],
      "prohibitions": [
        "No new code comments",
        "No retry of any error other than ExecutableFileBusy"
      ]
    },
    "behavior_change": true
  }
]
```
