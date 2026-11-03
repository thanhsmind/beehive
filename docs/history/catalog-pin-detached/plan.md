---
mode: tiny
# approved_gate2: <unset until approval>
---

# Plan: Catalog pin for --detached

## Summary

Bump the flag-vocabulary pin from 207 to 208 so the full suite is green again
and the 2.41.0 release can run.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: one test constant and
its reason comment; the release test gate re-runs the full suite anyway.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The pin test fails at 208 vs 207 on main | ran | `scripts/release.sh 2.41.0` (deploy worker job-1789564292319-476079-1) | `assertion left == right failed at crates/bee/src/catalog.rs:787:9 (left: 208, right: 207)` |
| 2 | The pin constant is 207 | read | `packages/bee-rs/crates/bee/src/catalog.rs:782` | `const PINNED_FLAG_COUNT: usize = 207;` |

## Cells (current slice)

```json
[
  {
    "id": "cpd-1",
    "feature": "catalog-pin-detached",
    "title": "Pin the flag vocabulary at 208 for --detached",
    "lane": "tiny",
    "role": "code",
    "deps": [],
    "decisions": ["cf50327a-104e-4e44-97b7-d52fd850494b"],
    "files": ["packages/bee-rs/crates/bee/src/catalog.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/catalog.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In catalog.rs test distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision, bump PINNED_FLAG_COUNT from 207 to 208 and add a comment entry '207 -> 208 (harness-worktree-relocation hwr-2)' saying worktree.merge gained --detached and no existing flag name means skip the caller-session relocation check (per D1).",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee distinct_flag_vocabulary_is_pinned",
    "must_haves": {"truths": ["the flag vocabulary pin test passes at 208"], "artifacts": [], "key_links": [], "prohibitions": ["Do not rename or remove any flag"]},
    "behavior_change": false
  }
]
```
