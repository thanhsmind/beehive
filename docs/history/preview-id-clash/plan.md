---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: a reused cell id is refused at gate preview

## Summary

A plan that reuses a cell id held by another feature's cell, live or
archived, passes the gate and fails only at `cells add`. Move that refusal to
`bee gate --preview`, the first door that reads the packet, so the rename
happens before approval.

Mode: `small` — one check in one function, one test file.

## Requirements (from CONTEXT.md)

- D1: preview refuses a packet id held by a live or archived cell of another feature.
- C1: contract preview-id-clash.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The preview parses the packet then writes gate_preview with no id check | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:541 | `let cells = match parse_plan_packets(&text) {` |
| 2 | One helper finds an id among live then archived cells | read | packages/bee-rs/crates/bee/src/verbs/cells/util.rs:254 | `pub(crate) fn resolve_cell_file(root: &Path, id: &str) -> Option<PathBuf> {` |
| 3 | The preview door is one function | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:500 | `pub(crate) fn run_gate_preview_body(root: &Path, flags: &Flags, _use_json: bool) -> R2<Out> {` |

## Approach

Situation: `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

Red first: the tester writes the archived-clash case and shows it fails on the
base before the fix lands, then the full contract green on the fix. The check
reads the cell store from the same root `cells add` writes to, so a preview
run from a granted worktree sees main's cells.

Rejected: a warning only (a warned clash still reaches the gate); keeping the
refusal at `cells add` alone (already there, after approval).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "pic-1 adds the preview check."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "pic-2 writes the contract tests red then green."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "The leader adds one line to the gates concept."},
    {"stage": "slice-judge", "classification": "not-applicable", "role": "review", "reason": "Small lane; review stays user-invoked."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Small lane."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "One new refusal with a named fix."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Same-feature ids pass, so re-previews are unchanged."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Fix-at debt is owner-requested."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "The leader wrote the plan."},
    {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "The anchors are known."},
    {"stage": "fact-extraction", "classification": "not-applicable", "role": "extraction", "reason": "No extraction stage."},
    {"stage": "generation-fallback", "classification": "not-applicable", "role": "generation", "reason": "Both cells have a configured role."},
    {"stage": "writing", "classification": "not-applicable", "role": "writer", "reason": "No prose deliverable."},
    {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "Small lane."},
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
| A re-preview of a slice whose cells were already added refuses itself | Same-feature ids pass; a contract case pins it |
| A preview from a worktree reads the worktree's empty cell store | The check reads the root `cells add` writes to |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pic-1 | Refuse a reused cell id at gate preview | plan_packets.rs | — | the refusal names id and feature | gate_preview tests |
| pic-2 | Prove the preview id-clash refusal red then green | tests/gate_preview.rs, report | pic-1 | contract cases pass | gate_preview tests |

```json
[
{"id":"pic-1","feature":"preview-id-clash","title":"Refuse a reused cell id at gate preview","lane":"small","role":"code","deps":[],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test gate_preview",
"must_haves":{"truths":["run_gate_preview_body refuses before writing gate_preview when a packet cell id resolves to a live or archived cell whose feature differs from the previewed feature","the refusal text names the id, the holding feature, whether that cell is live or archived, and tells the reader to rename the id in plan.md","an id held by a cell of the same feature does not refuse","the existing gate_preview tests still pass"]},
"decisions":["e03c4bf8-1e6b-4550-958c-f5afe300cd6a"],
"files":["packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"],
"read_first":["docs/history/preview-id-clash/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Implement CONTEXT.md D1 in packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs run_gate_preview_body (line 500). After parse_plan_packets (line 541) and before gate_preview is inserted or written, for each packet cell with a string id, look it up with crate::verbs::cells::resolve_cell_file (verbs/cells/util.rs:254) against the cell-store root that `cells add` uses (see how verbs/cells/mod.rs:242 resolves it with resolve_root_serving_granted; use that root, falling back to the preview's root). Read the found file's feature field; if it differs from the previewed feature, collect a problem. If any problem, return Out::Thrown with one line per clash, shaped: gate preview: cell id \"<id>\" is already held by the <live|archived> cell of feature \"<f>\". FIX: rename the id in plan.md, then re-run bee gate --preview. A cell file is archived when its path is not cells_dir/<id>.json. Nothing is written on refusal. No code comment. Commit once (cell trailer). Proof: the verify command."},
{"id":"pic-2","feature":"preview-id-clash","title":"Prove the preview id-clash refusal red then green","lane":"small","role":"test","deps":["pic-1"],
"verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test gate_preview",
"must_haves":{"truths":["a test previews a packet whose id is held by an archived cell of another feature and asserts a non-zero exit, the id and the holding feature in the output, and no gate_preview on the record","a test does the same for a live cell of another feature","a test shows an id held by a cell of the same feature still previews","the archived-clash test was run against the pic-1 parent commit and failed there, and the report quotes that failure","the report under docs/history/preview-id-clash/reports/ quotes each run's result line"]},
"decisions":["fd9a503f-a07c-4099-8e71-f48c5a26d9b3","e03c4bf8-1e6b-4550-958c-f5afe300cd6a"],
"files":["packages/bee-rs/crates/bee/tests/gate_preview.rs","docs/history/preview-id-clash/reports/pic-2.md"],
"read_first":["docs/history/preview-id-clash/CONTEXT.md"],
"affects_skills":[],"affects_specs":[],
"action":"Check contract preview-id-clash (decision fd9a503f). In packages/bee-rs/crates/bee/tests/gate_preview.rs, using its fixture helpers, add tests: (1) an archived cell file .bee/cells/archive/<other-feature>/<id>.json with feature <other-feature> plus a plan.md packet naming that id: gate --preview exits non-zero, output names the id and the other feature, and the record has no gate_preview; (2) the same with a live .bee/cells/<id>.json of another feature; (3) a live cell of the SAME feature with that id: preview succeeds. Red proof: check out the pic-1 parent version of plan_packets.rs into the tree, run test (1), see it fail, restore plan_packets.rs to HEAD and confirm git status shows it clean. Then run the verify green. Edit no production code. Write docs/history/preview-id-clash/reports/pic-2.md with each run and its result line. Commit the test file and report once (cell trailer)."}
]
```
