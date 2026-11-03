---
mode: tiny
---

# Plan: Windows cfg gate for the backlog fail-open test

## Summary

Gate one unix-only test behind `#[cfg(unix)]` so the Windows CI job on
`main` compiles again.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one attribute on one
test, matching two neighbours in the same file; the Windows CI run after the
push is the deterministic net.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The Windows job fails to compile the test binary at the ungated import | ran | GitHub run 35715737480, `gh run view --log-failed` | `error[E0433]: cannot find \`unix\` in \`os\`` at `crates\bee\src\verbs\drivers\close.rs:5283:22` |
| 2 | The test has no cfg gate while its neighbours do | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:5281-5283` | `#[test]` directly above `fn an_unwritable_backlog_warns_and_the_close_still_passes()`, then `use std::os::unix::fs::PermissionsExt;`; `close.rs:4242` reads `#[cfg(unix)]` |

## Cells (current slice)

```json
[
  {
    "id": "wcu-1",
    "feature": "windows-cfg-unix",
    "title": "Gate the backlog fail-open test behind cfg(unix)",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": ["0f2e5d4e-e11f-42aa-ae01-b3689a2a2fe4"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In close.rs, add #[cfg(unix)] on the line above #[test] for an_unwritable_backlog_warns_and_the_close_still_passes (line 5281), matching gpgsign_true_with_a_failing_signer_still_lands_the_bookkeeping_commit at 4242 (per D1). Change nothing else.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee an_unwritable_backlog_warns",
    "must_haves": {"truths": ["the test carries #[cfg(unix)] and still passes on Linux"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change the test body", "Do not touch any other test"]},
    "behavior_change": false
  }
]
```
