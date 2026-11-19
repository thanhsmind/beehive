---
artifact_contract: bee-plan/v2
mode: tiny
---

# Plan: a stale vendored prompt turns the suite red

## Summary

Every prompt parity check today loops over a hand-written name list, and a
worker once narrowed one to let a stale `.bee/bin/prompts/` copy pass.
`peer-boot.md` has no vendored check at all. Add one test whose file set
comes from the directory itself.

Mode: `tiny` — one new test in one file; no production code changes.

## Requirements (from CONTEXT.md)

- D1: directory-driven parity test between `packages/bee/prompts/` and `.bee/bin/prompts/`.
- C1: contract vendored-prompt-parity.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | C4 compares the source tree with the embedded bytes only, over a hand list | read | packages/bee-rs/crates/bee/src/devtools/prompts.rs:256 | `fn c4_embedded_prompts_match_disk() {` |
| 2 | The runtime skew probe takes one name at a time | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:222 | `pub(crate) fn prompt_skew(root: &Path, name: &str) -> Option<PromptSkew> {` |
| 3 | The vendored tree holds five prompts, peer-boot among them | ran | git ls-files .bee/bin/prompts | `.bee/bin/prompts/peer-boot.md` |

## Approach

Situation: `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

The repro is a stale vendored copy that the suite does not catch. The tester
shows the new test red on a changed vendored file and on an extra vendored
file, restores the tree, and shows it green. One code cell, one tester cell.

Rejected: adding peer-boot to `PROMPT_NAMES` (it is not a dispatch prompt, and
a list can be narrowed again).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "vpg-1 writes the test."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "vpg-2 proves it red then green."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "No behavior a concept describes changes."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Tiny lane; review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Tiny lane."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Tiny lane."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "No behavior change."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Test-only change."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Fix-at debt is owner-requested."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The anchors are known."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Both cells have a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "No prose deliverable."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Tiny lane."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind lanes."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind lanes."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind lanes."},
    {"stage": "design-ui", "classification": "not-applicable", "role": "design", "reason": "No rendered UI."}
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| The test passes vacuously on an empty directory | It asserts at least one source prompt was compared |
| CRLF checkouts on Windows go red | Both sides normalize CRLF to LF before comparing |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| vpg-1 | Pin vendored prompts to their source files | devtools/prompts.rs | — | the new test passes | vendored_prompts filter |
| vpg-2 | Prove the vendored prompt parity test red then green | report | vpg-1 | red on stale copy, green restored | vendored_prompts filter |

```json
[
{"id":"vpg-1","feature":"vendored-prompt-parity-guard","title":"Pin vendored prompts to their source files","lane":"tiny","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- vendored_prompts",
"must_haves":{"truths":["a test named vendored_prompts_match_source_file_for_file exists in the tests module of packages/bee-rs/crates/bee/src/devtools/prompts.rs","the test takes its file set from reading the packages/bee/prompts directory, with no hand-written name list","the test fails when .bee/bin/prompts holds a file the source tree lacks","the verify command runs the new test and it passes"]},
"decisions":["39cbf005-f763-408f-9456-81a8c8acf5fb","a9aedf60-73d2-4b4e-9ed6-baef61e458ad"],
"files":["packages/bee-rs/crates/bee/src/devtools/prompts.rs"],
"read_first":["docs/history/vendored-prompt-parity-guard/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D1. In the tests module of packages/bee-rs/crates/bee/src/devtools/prompts.rs, beside c4_embedded_prompts_match_disk (line 256), add #[test] fn vendored_prompts_match_source_file_for_file. Use the module's repo_root(). Read every *.md under packages/bee/prompts with std::fs::read_dir; for each, read .bee/bin/prompts/<same name>, fail with a message naming the missing or differing vendored path and the remedy `bee dev regen`; compare text after replacing \\r\\n with \\n on both sides. Then read_dir .bee/bin/prompts and fail on any *.md with no source twin, naming it. Assert at least one file was compared. No hand-written list of names, no code comment, no production code change. Commit once (cell trailer). Proof: the verify command; quote the line that shows the new test ran and passed."},
{"id":"vpg-2","feature":"vendored-prompt-parity-guard","title":"Prove the vendored prompt parity test red then green","lane":"tiny","role":"test","deps":["vpg-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- vendored_prompts",
"must_haves":{"truths":["with one line appended to .bee/bin/prompts/peer-boot.md the test fails and its message names peer-boot.md","with an extra file .bee/bin/prompts/stray.md the test fails and its message names stray.md","after restoring the tree the test passes and git status shows .bee/bin/prompts clean","the report under docs/history/vendored-prompt-parity-guard/reports/ quotes each run's result line"]},
"decisions":["a9aedf60-73d2-4b4e-9ed6-baef61e458ad","39cbf005-f763-408f-9456-81a8c8acf5fb"],
"files":["docs/history/vendored-prompt-parity-guard/reports/vpg-2.md"],
"read_first":["docs/history/vendored-prompt-parity-guard/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract vendored-prompt-parity (decision a9aedf60) on the vpg-1 commit. Run 1: append one line to .bee/bin/prompts/peer-boot.md, run the verify, expect FAILED naming peer-boot.md, then restore the file with a checkout of HEAD. Run 2: create .bee/bin/prompts/stray.md, run the verify, expect FAILED naming stray.md, then delete it. Run 3: run the verify on the clean tree, expect pass. Confirm .bee/bin/prompts is clean in git status. Edit no source. Write docs/history/vendored-prompt-parity-guard/reports/vpg-2.md with each run, its command and its result line verbatim. If a check fails, return BLOCKED with the evidence. Commit only the report (cell trailer)."}
]
```
