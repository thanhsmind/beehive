# Plan: refusal-followups

## Summary
The "lane is mid-flight" refusal of `bee state start-feature` names no
command. It now names `bee state session bind --lane <feature>`, like the
live-workflow refusal next to it (pi-slp-operations D1). One more test pins
the CLAIMED fix when neither side has a session.

Mode: `tiny` — 0 risk flags. One product file, text only.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rfu-1 | Name the bind command in the lane-mid-flight refusal | policy.rs, state_group/tests.rs, cells/tests.rs | — | the refusal names `bee state session bind --lane <feature>` | state_group and cells tests |

```json
[
  {
    "id": "rfu-1",
    "feature": "refusal-followups",
    "lane": "tiny",
    "role": "code",
    "change_class": "refactor",
    "title": "Name the bind command in the lane-mid-flight refusal",
    "deps": [],
    "decisions": ["D1", "75db2633-545a-47a3-8daa-5e492eed6568", "a18203fc-18a0-4461-bc47-bb147395d171"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1: in packages/bee-rs/crates/bee/src/verbs/state_group/policy.rs replace `FIX: finish or explicitly wind down that lane first, then retry.` in the lane-mid-flight startFeature refusal with `FIX: bee state session bind --lane {feature} --session-id <session id>.`; keep the text before FIX and the trigger unchanged. Add a test in packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs that a mid-flight lane refusal names that command. In packages/bee-rs/crates/bee/src/verbs/cells/tests.rs add a case where both caller and holder are sessionless and the CLAIMED reason names `FIX: bee cells claim-next.`. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group && cargo test --release -p bee --bin bee verbs::cells",
    "must_haves": {
      "truths": ["the lane-mid-flight startFeature refusal names bee state session bind --lane with the feature", "a CLAIMED refusal with both sides sessionless names bee cells claim-next"]
    }
  }
]
```
