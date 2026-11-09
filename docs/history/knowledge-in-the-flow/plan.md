---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Knowledge in the feature flow

## Summary

Today only cell workers get knowledge; the leader shapes and plans blind,
and close leaves a proposal file nobody applies. After this, one command
gives the leader the feature's knowledge pack (the concepts and the code
they point at) at shaping and planning, the hat-wave advisors get the same
pack, every cell commits its own knowledge change and its cap records it, a
writer cell consolidates the feature's knowledge before the merge so main
gets code and knowledge together, close stops writing proposal files, and
the knowledge report compares features with and without the pack. This
repo's own backlog of 45 proposals, 50 capture stubs and 114 orphaned
scribing cells is drained once through that same flow.

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change, multi-domain, data-model
Why this is the least workflow that protects the work: it changes what close, merge, cap and dispatch print and record in every host repo, and existing tests assert the proposal file close writes today.

Class playbook: feature — `skills/bee-planning/references/planning-reference.md` ("Playbooks").

## Requirements (from CONTEXT.md)

- D1: Flow, not gate. No new gate, no mandatory knowledge-ref check. → every new line is a report; no verb gains a refusal (kitf-1, kitf-4, kitf-5).
- D2: One feature pack (knowledge + the code its concepts anchor) at shaping and planning, used by the interview, CONTEXT.md, plan.md claims and hat-wave briefs. → kitf-1 (verb), kitf-2 (hat wave), kitf-6 (skills).
- D3: One assembler, `learned_context_lines`. → kitf-1 extends it; the verb, the advisor prompt and the cell prompt all call it.
- D4: Each cell writes its knowledge delta in its own code commit. → kitf-6 (worker brief), kitf-5 (cap records it).
- D5: Close and merge only consolidate; a writer worker writes the delivery; knowledge lands in the same merge; the proposal backlog goes away. → kitf-4 (verb, close, merge), kitf-7 (this feature's own consolidation), kitf-6 (planning adds the writer cell).
- D6: No human review step for knowledge writes. → no review door anywhere in this plan.
- D7: Thin layer: anchor to code, never restate code; the writer trims. → consolidate prints the rules (kitf-4); kitf-6 and kitf-7 apply them.
- D8: Record pack paths in dispatch.jsonl; 10% holdout at feature level and cell level. → kitf-1 (pack row, feature arm), kitf-2 (advisor row), kitf-3 (report).
- D9: One-time drain of proposals, capture stubs, orphaned scribing. → kitf-8a, kitf-8b, kitf-8c.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The one assembler takes a cell-shaped value, so a feature can call it with a synthetic `{id, feature, lane}` | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1075 | `pub(crate) fn learned_context_lines(root: &Path, cell: &Value) -> D<Vec<String>> {` |
| 2 | The block is capped at 8 lines today | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:654 | `pub(crate) const LEARNED_CONTEXT_MAX_LINES: usize = 8;` |
| 3 | The holdout arm is a hash of an id string, reusable for a feature slug | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1118 | `pub(crate) fn knowledge_arm(root: &Path, cell_id: &str) -> &'static str {` |
| 4 | Only kind cell records an arm and learned paths today | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3293 | `if let (true, Some(c)) = (kind == "cell", &cell) {` |
| 5 | Close writes promote-proposals.md and enqueues a stub and a queue record | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3327-3330 | `match write_text_atomic(&root.join(&proposals_rel), &text) { / Ok(()) => { / enqueue_promote_stub(root, feature, &proposal, &proposals_rel); / enqueue_promote_deferred_record(root, feature, &proposals_rel);` |
| 6 | The report reads arms only from prepare rows of kind cell | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:343 | `if row.get("source").and_then(Value::as_str) != Some("prepare") \|\| row.get("kind").and_then(Value::as_str) != Some("cell") {` |
| 7 | Cap already records files_changed, so the delta is derivable without a new input | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:551 | `trace.insert("files_changed".into(), Value::Array(f.files_changed.clone()));` |
| 8 | Merge text output has one builder to extend | read | packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:1506 | `pub(crate) fn merge_text_lines(id: &str, main_root: &Path, answer: &MergeAnswer) -> Vec<String> {` |
| 9 | The worker prompt already has a conditional learned block | read | packages/bee/prompts/worker-cell.md:39 | `{{#if learned_context}}` |
| 10 | The advisor prompt is a separate template with its own conditionals | read | packages/bee/prompts/advisor.md:3 | `{{#if seat}}` |
| 11 | The advisor template is compiled into the binary | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:39 | `pub(crate) const PROMPT_ADVISOR: &str = include_str!("../../../../../../bee/prompts/advisor.md");` |
| 12 | The unapplied-proposal detector is separate from close, so old proposals still surface until drained | read | packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:263 | `pub(crate) fn unapplied_promote_proposals(root: &Path) -> Vec<UnappliedPromoteProposal> {` |
| 13 | This repo already records cell arms; most closed cells predate the holdout | ran | .bee/bin/bee knowledge report | `outcomes unrecorded: 1287 closed cell(s) with no recorded arm` |

## Discovery

Read the five concepts in CONTEXT.md's read set plus `docs/history/knowledge-freshness-mechanic/CONTEXT.md`
(anchors, cap sync door D4, pull-only D1) and `docs/history/knowledge-docs-in-the-flow/CONTEXT.md`
(read set D2, kept). Opened the anchors in the claims table. Ran `bee knowledge report`: 20
knowledge and 1 holdout cell recorded here. The verb registry is
`packages/bee-rs/crates/bee/src/generated/registry_payload.json`, routed by
`packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs`.

## Approach

Recommended path:

- Extend the one assembler (D3): `learned_context_lines` keeps its knowledge lines and appends the
  code lines its concepts anchor, so cells, the pack verb and the advisor prompt print the same pack
  (D2). Contract decision `cfc015c6` (contract:feature-pack).
- `bee knowledge pack` is the shaping and planning load; it records a `source: pack` row with the
  feature arm (D8). Skills call it at Explore and at planning; Lock fills `## Read set` from it.
- Advisor prepares render the pack (D2, hat-wave briefs) and record the arm (`b05930b2`, contract:advisor-pack).
- The report adds feature-level arms (`4cfda42e`, contract:report-feature-arms).
- Close stops writing proposals; `bee knowledge consolidate` gives the writer cell its brief; merge
  and close print a knowledge line, refusing nothing (`cde4316f`, contract:consolidate; D1, D5).
- Cap records `knowledge_delta` (`5faa858a`, contract:cap-knowledge-delta; D4).
- Planning adds a final writer cell to every code-touching feature, so consolidation is an ordinary
  step before merge (D1, D5). This feature's own kitf-7 is the first one.

Rejected alternatives:

- A merge refusal when no delivery exists — a new gate, against D1.
- Auto-writing proposals into the bundle at close — the knowledge would land after the merge, against D5.
- A second assembler for the pack verb — against D3.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Cell prompt bytes change for bundles with anchors | MEDIUM | kitf-1 | Existing learned-context tests stay green except the ones that assert the exact block on an anchored fixture, which gain the code lines; a bundle with no anchors renders byte-identically (`cargo test … -- learned_context`). |
| Close tests assert the proposal file | MEDIUM | kitf-4 | The proposal-write tests are rewritten to assert no file and the new line; the unapplied detector tests stay unchanged. |
| Drain writes to many area concepts | MEDIUM | kitf-8a/b/c | Serial cells, `bee knowledge check` green after each. |
| Merge output shape | LOW | kitf-4 | Merge tests assert the new line and JSON key. |

Waves: wave 1 = kitf-1, kitf-5, kitf-8a (no shared files). Wave 2 = kitf-2, kitf-3, kitf-4 (after
kitf-1: they share prepare.rs, knowledge tests, the registry and routing). kitf-8b after kitf-8a,
kitf-8c after kitf-8b (all write area concepts). Wave 3 = kitf-6 (after kitf-1, kitf-2, kitf-4, kitf-5:
skills name the verbs). Wave 4 = kitf-9 (after every code cell and kitf-6). Wave 5 = kitf-7 (after
everything, the consolidation of this feature).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "rust-verbs", "classification": "required", "role": "code", "reason": "kitf-1 to kitf-5 write Rust and their tests"},
    {"stage": "skill-and-prompt-edits", "classification": "required", "role": "docs", "reason": "kitf-6 edits skills, the worker prompt and regenerates the trees"},
    {"stage": "knowledge-writing", "classification": "required", "role": "writer", "reason": "kitf-7 consolidates this feature; kitf-8a/b/c drain the debt"},
    {"stage": "end-to-end-proof", "classification": "required", "role": "test", "reason": "kitf-9 writes the end-to-end integration test"},
    {"stage": "code-scout", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of more than three files", "reason": "I/O offload"},
    {"stage": "release", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "plan-drafting", "classification": "not-applicable", "role": "plan", "reason": "the leader drafted the plan inline"},
    {"stage": "fact-lookup", "classification": "conditional", "role": "extraction", "condition": "a single fact from a known location", "reason": "cheap reader"},
    {"stage": "fallback-writer", "classification": "not-applicable", "role": "generation", "reason": "every cell names its role"},
    {"stage": "cell-review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "high-risk-consult", "classification": "required", "role": "advisor", "reason": "high-risk lane; the hat wave synthesis is the advisor ref"},
    {"stage": "observer", "classification": "not-applicable", "role": "supervisor", "reason": "no unattended control loop"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "the shape has a precedent in the repo"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "the shape has a precedent in the repo"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "the shape has a precedent in the repo"},
    {"stage": "plan-check-facts", "classification": "required", "role": "hat-facts-gaps", "reason": "hat wave seat 1"},
    {"stage": "plan-check-risks", "classification": "required", "role": "hat-risks", "reason": "hat wave seat 4, high-risk"},
    {"stage": "plan-check-value", "classification": "required", "role": "hat-value", "reason": "hat wave seat 5, high-risk"},
    {"stage": "plan-check-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "hat wave seat 2"},
    {"stage": "plan-check-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "hat wave seat 3"},
    {"stage": "ui", "classification": "not-applicable", "role": "design", "reason": "no rendered UI"}
  ]
}
```

## Shape

Epic map. Feature outcome: knowledge moves with the feature from shaping to merge, with no new gate.
Repo-reality basis: claims 1-13.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Read | One pack at shaping, planning, hat wave, cells | D2, D3 | kitf-1, kitf-2 | feature_pack and advisor_pack tests |
| Measure | Feature arm, pack rows, report | D8 | kitf-1, kitf-3 | feature_outcomes tests |
| Write | Delta at cap, consolidation before merge, close without proposals | D4, D5, D7 | kitf-4, kitf-5, kitf-6, kitf-7 | knowledge_consolidat and knowledge_delta tests; parity |
| Drain | This repo's debt | D9 | kitf-8a, kitf-8b, kitf-8c | knowledge check; orient debt counts at 0 |
| Prove | End to end | Done criteria | kitf-9 | integration test green |

One slice; every cell below is in it.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| kitf-1 | Assemble one feature pack and load it with bee knowledge pack | prepare.rs, knowledge/pack.rs (new), knowledge/mod.rs, knowledge/routing.rs, registry_payload.json, drivers/tests.rs, knowledge/tests.rs | — | `bee knowledge pack --feature f --stage shaping` lists concepts and the code they anchor, and dispatch.jsonl gets a pack row with the feature arm | `cargo test … --bin bee -- feature_pack` |
| kitf-5 | Record the knowledge delta on every cap | cells/handlers_close.rs, cells/tests.rs | — | `bee cells cap` prints `knowledge delta: 2 file(s)` and the trace carries them | `cargo test … --bin bee -- knowledge_delta` |
| kitf-8a | Drain the unapplied promote proposals into the bundle | docs/knowledge/** | — | `bee orient` names no unapplied proposal | `bee knowledge check` |
| kitf-2 | Give hat-wave and advisor briefs the feature pack | prepare.rs, packages/bee/prompts/advisor.md, drivers/tests.rs | kitf-1 | an advisor prompt shows the Learned context block and its prepare row carries the arm | `cargo test … --bin bee -- advisor_pack` |
| kitf-3 | Compare features with and without the pack in the knowledge report | knowledge/report.rs, knowledge/tests.rs, tests/knowledge_outcomes.rs | kitf-1 | `bee knowledge report` prints feature outcome lines per arm | `cargo test … --bin bee -- feature_outcomes` |
| kitf-4 | Consolidate knowledge before merge and stop writing proposals at close | knowledge/consolidate.rs (new), knowledge/mod.rs, knowledge/routing.rs, registry_payload.json, drivers/close.rs, worktree/handlers.rs, worktree/phases.rs, worktree/tests.rs | kitf-1 | `bee knowledge consolidate` prints the writer's brief; merge prints a knowledge line; close prints consolidated or not and writes no proposal file | `cargo test … --bin bee -- knowledge_consolidat` |
| kitf-8b | Flush the pending capture stubs into the bundle | docs/knowledge/** | kitf-8a | `bee capture count` reads 0 | `bee knowledge check` |
| kitf-8c | Stamp the orphaned scribing debt through consolidation | docs/knowledge/** | kitf-8b | the preamble shows no orphaned scribing debt | `bee knowledge check` |
| kitf-6 | Teach shaping, planning, workers and capturing the pack and the delta | skills/*, packages/bee/prompts/worker-cell.md, rendered skill trees | kitf-1, kitf-2, kitf-4, kitf-5 | bee-shaping runs `bee knowledge pack` at Explore; workers commit their knowledge delta; planning adds the writer cell | `cargo test … --test rule_index_parity` |
| kitf-9 | Prove the flow end to end in a throwaway repo | tests/knowledge_in_the_flow_e2e.rs (new) | kitf-1 to kitf-6 | one test runs pack, cap with delta, consolidate, close with no proposal, report with both arm levels | `cargo test … --test knowledge_in_the_flow_e2e` |
| kitf-7 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, docs/history/knowledge-in-the-flow/delivery.md | all | the okf-profile and workflow-state concepts describe the pack, the delta and consolidation | `bee knowledge check` |

```json
[
  {
    "id": "kitf-1",
    "feature": "knowledge-in-the-flow",
    "title": "Assemble one feature pack and load it with bee knowledge pack",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D2", "D3", "D8", "cfc015c6"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/pack.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "docs/knowledge/areas/okf-profile/context-and-promote.md",
      "docs/knowledge/areas/workflow-state/dispatch.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision cfc015c6 (contract:feature-pack) exactly. (1) In prepare.rs extend `learned_context_lines` (prepare.rs:1075): keep the knowledge lines and their 8-line cap (LEARNED_CONTEXT_MAX_LINES, prepare.rs:654) byte-identical, then append at most 8 code lines `- code: <path> — anchored by <concept path>` built from the `bee.anchors` list of each listed concept in listed order: keep a path only when it exists under the bundle root's repo root, does not start with `docs/`, is not in the cell's read_first, and was not already listed; sanitize the path through the same one-line collapse the knowledge paths use. A bundle with no anchors renders exactly as before. (2) Add `feature_arm(root, feature)` beside `knowledge_arm` (prepare.rs:1118) hashing the string `feature:<slug>` with the same bucket rule and `knowledge_holdout_percent`. (3) Extend the cell prepare record (prepare.rs:3293 block) so `learned_context` keeps only knowledge paths and a new `pack_code` array holds the code paths. (4) Add `verbs/knowledge/pack.rs` with `bee knowledge pack --feature <f> --stage shaping|planning [--lane <l>] [--json]`: resolve the bundle root as prepare does for a feature with a granted worktree (the worktree checkout, else the repo root); compute the feature arm; for the knowledge arm call `learned_context_lines` with the value `{id: <feature>, feature: <feature>, lane: <lane or standard>}`; print the lines in text mode and `{feature, stage, knowledge_arm, knowledge:[{path,title}], code:[{path, anchored_by}], writes:[\".bee/logs/dispatch.jsonl\"]}` in JSON; for the holdout arm print empty lists and the one line `holdout: this feature loads no pack (measurement arm, knowledge_holdout_percent)`; append one dispatch.jsonl row through `append_prepare_record`'s file with `source: \"pack\"`, `kind: \"pack\"`, `feature`, `stage`, `knowledge_arm`, `learned_context`, `pack_code`; a stage outside shaping|planning refuses as a typed usage error; an absent or empty bundle prints an empty pack, exit 0. Register the verb in routing.rs and registry_payload.json beside `knowledge search`. (5) Tests whose fn names contain `feature_pack`: anchored concept adds code lines; anchor under docs/ or missing on disk or in read_first is skipped; bundle with no anchors is byte-identical; holdout feature prints empty pack and records arm holdout; pack row shape; bad stage refuses; cell prepare record carries pack_code. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- feature_pack",
    "must_haves": {
      "truths": [
        "bee knowledge pack --feature f --stage shaping lists the feature's knowledge lines followed by the code lines their concepts anchor",
        "a holdout feature gets an empty pack and its row records knowledge_arm holdout",
        "every pack call appends one dispatch.jsonl row with source pack, stage, feature, knowledge_arm, learned_context and pack_code",
        "a bundle whose concepts carry no anchors renders the cell Learned context block byte-identically to before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/pack.rs", "substantive": "the pack verb, its text and JSON output and its dispatch.jsonl row"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "learned_context_lines code lines, feature_arm, pack_code on the cell record"}
      ],
      "key_links": [
        "pack.rs calls learned_context_lines, never a second assembler"
      ],
      "prohibitions": [
        "No new refusal on any existing verb",
        "No file bodies in the pack output",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-5",
    "feature": "knowledge-in-the-flow",
    "title": "Record the knowledge delta on every cap",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D4", "5faa858a"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision 5faa858a (contract:cap-knowledge-delta). Where cap writes `trace.files_changed` (handlers_close.rs:551 `trace.insert(\"files_changed\".into(), Value::Array(f.files_changed.clone()));`) also write `trace.knowledge_delta`: the files_changed entries whose normalized path starts with `docs/knowledge/`, in order. The same holds on the `bee cells finish` path that reaches this code (`finish_cap_and_release`, handlers_close.rs:1448). The cap and finish text output gain one line `knowledge delta: N file(s)` or `knowledge delta: none`; the JSON result carries `knowledge_delta`. Nothing refuses (D1). Tests whose fn names contain `knowledge_delta`: a cap with two docs/knowledge files and one code file records two; a cap with none records an empty list and prints `knowledge delta: none`; finish records the same. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- knowledge_delta",
    "must_haves": {
      "truths": [
        "a capped cell's trace carries knowledge_delta listing its docs/knowledge files",
        "cap prints knowledge delta: none when the cell changed no docs/knowledge file and still caps"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "knowledge_delta on the trace and the output line"}
      ],
      "key_links": [
        "knowledge_delta is derived from files_changed, never a new flag"
      ],
      "prohibitions": [
        "No new cap refusal",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-8a",
    "feature": "knowledge-in-the-flow",
    "title": "Drain the unapplied promote proposals into the bundle",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": [],
    "decisions": ["D5", "D6", "D7", "D9"],
    "files": [
      "docs/knowledge/**"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "skills/bee-capturing/SKILL.md",
      "skills/bee-capturing/references/area-spec.md",
      "docs/knowledge/index.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Drain every unapplied promote proposal once (D9). Get the list from the main checkout: `.bee/bin/bee orient` names each unapplied `docs/history/<feature>/promote-proposals.md` and its `bee deferred-queue complete --kind promote --feature <feature>` line. For each feature, oldest first: read the proposal and the code it names; keep only what is still true in today's code and is not already in the bundle; write it into the existing area concept that owns it (extend, never add a near-duplicate), each kept bullet anchored to a code path that exists (add it to that concept's bee.anchors), never restating what the code shows (D7); a pitfall pattern becomes a docs/knowledge/patterns/ concept only when no pattern already covers it; no human review step (D6). Cite history D-numbers in area docs, never raw decision ids. Then run `.bee/bin/bee state scribing-run --feature <feature> --no-lane --areas \"<areas touched, or none>\" --next-action \"none\"` and `.bee/bin/bee deferred-queue complete --kind promote --feature <feature>` — the stamp is owed even when nothing was kept. Run `.bee/bin/bee knowledge index` after the edits, then `.bee/bin/bee knowledge check` must be green. Commit the docs/knowledge changes in one commit. No code comments.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "bee orient names no unapplied promote proposal",
        "bee knowledge check is green after the drain"
      ],
      "artifacts": [
        {"path": "docs/knowledge/index.md", "substantive": "regenerated index after the drain"}
      ],
      "key_links": [
        "every kept bullet sits in an existing area concept and anchors to an existing code path"
      ],
      "prohibitions": [
        "No near-duplicate concept beside an existing one",
        "No raw decision ids in area docs",
        "No edit outside docs/knowledge/"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "kitf-2",
    "feature": "knowledge-in-the-flow",
    "title": "Give hat-wave and advisor briefs the feature pack",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["kitf-1"],
    "decisions": ["D2", "D3", "D8", "b05930b2"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee/prompts/advisor.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "packages/bee/prompts/advisor.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision b05930b2 (contract:advisor-pack). In the non-cell render path of prepare.rs (the `render(&template, &[(\"brief\", …), (\"expertise\", …), (\"purpose\", …), (\"original_request\", …), (\"seat\", …)])` call before the worker-cell branch), for kind advisor only: resolve the feature (the --feature flag, else the bound lane feature already used for original_request), resolve the bundle root the way the cell path does, compute `feature_arm` (from kitf-1), and for the knowledge arm build the lines with `learned_context_lines(bundle_root, {id: feature, feature, lane})`; pass them as a `learned_context` var. In packages/bee/prompts/advisor.md add, after the original_request block, `{{#if learned_context}}` / `Learned context (machine-assembled feature pack — read before answering; prefer it over re-deriving):` / `{{learned_context}}` / `{{/if}}`. Gather and reviewer templates and their vars stay unchanged; every template var still has a supplied value. The advisor prepare record gains `knowledge_arm`, `learned_context` (knowledge paths) and `pack_code` (code paths); a holdout feature renders no block and records arm holdout with empty lists; with no resolvable feature render no block and record no arm. Tests whose fn names contain `advisor_pack`: hat-* role advisor prompt carries the block; holdout feature carries none; record fields; gather prompt unchanged. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- advisor_pack",
    "must_haves": {
      "truths": [
        "an advisor or hat-* dispatch for a knowledge-arm feature shows the Learned context block with the same lines bee knowledge pack prints",
        "the advisor prepare row records knowledge_arm, learned_context and pack_code",
        "gather and reviewer prompts render byte-identically to before"
      ],
      "artifacts": [
        {"path": "packages/bee/prompts/advisor.md", "substantive": "the conditional Learned context block"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "advisor learned_context var and record fields"}
      ],
      "key_links": [
        "the advisor block is built by learned_context_lines"
      ],
      "prohibitions": [
        "No change to gather.md or reviewer.md",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-3",
    "feature": "knowledge-in-the-flow",
    "title": "Compare features with and without the pack in the knowledge report",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["kitf-1"],
    "decisions": ["D8", "4cfda42e"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "packages/bee-rs/crates/bee/tests/knowledge_outcomes.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "docs/history/knowledge-outcome-measure/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision 4cfda42e (contract:report-feature-arms). Beside `build_outcomes` (report.rs:374) add `build_feature_outcomes(root)`: read dispatch.jsonl rows that carry `knowledge_arm` and `feature` and have `source: pack` or `kind: advisor`; a feature's arm is its first such row; its cells are the closed cells (the existing `closed_cells`) whose `feature` matches; count per arm features, cells and the four existing signals via `cell_signals`; `unrecorded_features` counts features of closed cells with no such row. Add `feature_outcomes` to the report JSON beside `outcomes`, and text lines `feature outcomes <arm>: N feature(s), M cell(s), B blocked, R reopened, X red round(s), W with mistakes` plus `feature outcomes unrecorded: K feature(s) with no recorded pack`. The cell-level `outcomes` block and its lines stay byte-identical. Tests whose fn names contain `feature_outcomes`, in knowledge/tests.rs and tests/knowledge_outcomes.rs. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- feature_outcomes",
    "must_haves": {
      "truths": [
        "bee knowledge report prints feature outcome lines for the knowledge and holdout arms",
        "the cell-level outcome lines are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs", "substantive": "build_feature_outcomes and its JSON and text output"}
      ],
      "key_links": [
        "feature arms come from pack and advisor rows in dispatch.jsonl"
      ],
      "prohibitions": [
        "No change to the cell-level outcomes block",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-4",
    "feature": "knowledge-in-the-flow",
    "title": "Consolidate knowledge before merge and stop writing proposals at close",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["kitf-1"],
    "decisions": ["D1", "D5", "D7", "cde4316f"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "docs/knowledge/areas/okf-profile/context-and-promote.md",
      "docs/knowledge/areas/workflow-state/deferred-work-queue.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision cde4316f (contract:consolidate). (1) Add verbs/knowledge/consolidate.rs with `bee knowledge consolidate --feature <f> [--json]`, read-only, `writes: []`: reuse promote's builder (`build_promotion` in promote.rs) for the area-update and pattern-candidate sections; list the docs/knowledge files the feature's capped cells changed (their trace.knowledge_delta when present, else files_changed under docs/knowledge/), live and archived cells both; name the target `docs/history/<f>/delivery.md` and whether it exists; print the thin-layer rules in three lines: every entry anchors to code (bee.anchors), never restate what the code shows, extend the owning concept instead of adding a near-duplicate. Register it in routing.rs and registry_payload.json. Tests live in consolidate.rs's own test module. (2) In close.rs replace the proposal write (close.rs:3323-3353): do not write promote-proposals.md, do not call enqueue_promote_stub or enqueue_promote_deferred_record; push `Knowledge consolidated for \"<f>\": docs/history/<f>/delivery.md` when that file exists in the feature's checkout, else `Knowledge not consolidated for \"<f>\" — run bee knowledge consolidate --feature <f>`; put `knowledge: {consolidated: bool, delivery: <path|null>}` in the close JSON; the exit code is unchanged. Delete the two enqueue functions only if nothing else calls them, and rewrite the close tests that asserted the proposal file to assert its absence and the new line. `unapplied_promote_proposals` stays as is. (3) In the merge: compute the docs/knowledge files in the merged range with `branch_changed_files` (merge.rs:252) before the merge, carry `knowledge: {files: N, delivery: <path|null>}` on the merge result, and in `merge_text_lines` (handlers.rs:1506) print `  knowledge: N docs/knowledge file(s) in this merge; delivery: <path|none>` on a successful merge. Nothing refuses (D1). Tests whose fn names contain `knowledge_consolidat` for the verb, close and merge. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- knowledge_consolidat",
    "must_haves": {
      "truths": [
        "bee knowledge consolidate --feature f prints the proposal sections, the feature's docs/knowledge files, the delivery path and the thin-layer rules, and writes nothing",
        "bee close writes no promote-proposals.md and prints whether the feature's knowledge was consolidated, with the same exit code as before",
        "bee worktree merge prints the knowledge line and carries the knowledge object in --json"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs", "substantive": "the read-only consolidate verb and its tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "substantive": "close without the proposal write, with the consolidated line"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs", "substantive": "the merge knowledge line"}
      ],
      "key_links": [
        "consolidate reuses build_promotion, never a second miner"
      ],
      "prohibitions": [
        "No new refusal in close or merge",
        "No change to unapplied_promote_proposals",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-8b",
    "feature": "knowledge-in-the-flow",
    "title": "Flush the pending capture stubs into the bundle",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": ["kitf-8a"],
    "decisions": ["D6", "D7", "D9"],
    "files": [
      "docs/knowledge/**"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/index.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Drain every pending capture stub once (D9). List them from the main checkout with `.bee/bin/bee capture list`, oldest first. For each stub: check the settlement against today's code; when still true and not already in the bundle, merge it into the area concept that owns it (extend, never duplicate), anchored to code (bee.anchors), never restating code (D7), no human review (D6), citing history D-numbers, never raw decision ids. Then `.bee/bin/bee capture flush --id <id> --into <concept path, or none>`. Run `.bee/bin/bee knowledge index`, then `.bee/bin/bee knowledge check` must be green. Commit the docs/knowledge changes in one commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "bee capture count reads 0 pending",
        "bee knowledge check is green after the flush"
      ],
      "artifacts": [
        {"path": "docs/knowledge/index.md", "substantive": "regenerated index after the flush"}
      ],
      "key_links": [
        "each flushed stub names the concept it landed in or none"
      ],
      "prohibitions": [
        "No raw decision ids in area docs",
        "No edit outside docs/knowledge/"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "kitf-8c",
    "feature": "knowledge-in-the-flow",
    "title": "Stamp the orphaned scribing debt through consolidation",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": ["kitf-8b"],
    "decisions": ["D6", "D7", "D9"],
    "files": [
      "docs/knowledge/**"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/index.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Drain the orphaned scribing debt once (D9). From the main checkout, `.bee/bin/bee status --json` lists it under scribing_debt.orphaned.features (feature and cell ids). For each feature: read its capped cells' traces (`.bee/cells/archive/<feature>/` or `.bee/cells/`) and its docs/history/<feature>/ files; keep only behavior still true in today's code and missing from the bundle; write it into the owning area concept, anchored, thin (D7), no review (D6), history D-numbers only. Then `.bee/bin/bee state scribing-run --feature <feature> --no-lane --areas \"<areas touched, or none>\" --next-action \"none\"` — owed even when nothing was kept. Run `.bee/bin/bee knowledge index`, then `.bee/bin/bee knowledge check` must be green. Commit the docs/knowledge changes in one commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "bee status --json shows scribing_debt.orphaned.count 0",
        "bee knowledge check is green after the drain"
      ],
      "artifacts": [
        {"path": "docs/knowledge/index.md", "substantive": "regenerated index after the drain"}
      ],
      "key_links": [
        "every orphaned feature has a scribing-run stamp"
      ],
      "prohibitions": [
        "No raw decision ids in area docs",
        "No edit outside docs/knowledge/"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "kitf-6",
    "feature": "knowledge-in-the-flow",
    "title": "Teach shaping, planning, workers and capturing the pack and the delta",
    "lane": "high-risk",
    "role": "docs",
    "status": "open",
    "deps": ["kitf-1", "kitf-2", "kitf-4", "kitf-5"],
    "decisions": ["D1", "D2", "D4", "D5", "D7"],
    "files": [
      "skills/bee-shaping/SKILL.md",
      "skills/bee-shaping/references/context-template.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-planning/references/planning-reference.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "packages/bee/prompts/worker-cell.md",
      ".agents/skills/*",
      ".claude/skills/*",
      ".codex/skills/*",
      ".bee/onboarding.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "skills/bee-shaping/SKILL.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      "packages/bee/prompts/worker-cell.md"
    ],
    "affects_skills": ["bee-shaping", "bee-planning", "bee-swarming", "bee-capturing", "bee-hive"],
    "affects_specs": [],
    "action": "Edit the sources in skills/ and packages/bee/prompts/, then run `bee dev regen` and commit the rendered trees with them. bee-shaping Explore: the scout runs `bee knowledge pack --feature <f> --stage shaping` first (replacing the separate `bee knowledge context` step; `bee knowledge search` stays for symptoms) and Lock fills `## Read set` from the pack's knowledge and code lines (context-template.md names that source). bee-planning: Research runs `bee knowledge pack --feature <f> --stage planning`, and load-bearing claims anchor into the pack's code paths where they apply; Prep adds a final `role: writer` consolidation cell to every code-touching standard or high-risk feature, depending on every other cell, whose action runs `bee knowledge consolidate --feature <f>`, writes the deduped knowledge and `docs/history/<f>/delivery.md`, and whose verify is `.bee/bin/bee knowledge check` (planning-reference.md carries the cell shape). gates-and-delegation.md Hat wave: seats are dispatched with `--feature <f>` so their brief carries the pack. worker-cell.md: one bullet telling the worker to write its knowledge delta (owning area concept update, a pitfall pattern when it hit one, bee.anchors for files it changed) in the same commit as its code, thin and anchored, no review step. bee-swarming: the same rule in the Execute contract by pointer to worker-cell.md. bee-capturing: close no longer writes proposals; consolidation happens in the writer cell before merge; old proposals drain once. State nowhere that any of this refuses (D1). Write plain global English. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity",
    "must_haves": {
      "truths": [
        "bee-shaping's Explore runs bee knowledge pack --stage shaping and Lock fills the read set from it",
        "bee-planning runs bee knowledge pack --stage planning and adds the writer consolidation cell",
        "the worker brief tells every cell to commit its knowledge delta with its code"
      ],
      "artifacts": [
        {"path": "skills/bee-shaping/SKILL.md", "substantive": "Explore and Lock use the pack"},
        {"path": "skills/bee-planning/SKILL.md", "substantive": "Research uses the pack; Prep adds the writer cell"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "the knowledge delta bullet"}
      ],
      "key_links": [
        "rendered skill trees match their sources after bee dev regen"
      ],
      "prohibitions": [
        "No wording that makes loading or writing knowledge a gate",
        "No hand edit of a rendered skill tree"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kitf-9",
    "feature": "knowledge-in-the-flow",
    "title": "Prove the flow end to end in a throwaway repo",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": ["kitf-1", "kitf-2", "kitf-3", "kitf-4", "kitf-5", "kitf-6"],
    "decisions": ["D2", "D4", "D5", "D8", "cfc015c6", "cde4316f", "5faa858a", "4cfda42e"],
    "files": [
      "packages/bee-rs/crates/bee/tests/knowledge_in_the_flow_e2e.rs"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "packages/bee-rs/crates/bee/tests/knowledge_outcomes.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Write one integration test file that drives the built bee binary in a throwaway onboarded repo the way tests/knowledge_outcomes.rs does: a bundle with one area concept anchored to a code file; a feature; `bee knowledge pack --stage shaping` and `--stage planning` print the concept and the anchored code path and append two pack rows; a cell that changes the code file and the concept caps with `knowledge delta: 1 file(s)`; `bee knowledge consolidate --feature` lists that concept and writes nothing; after a delivery.md is written, `bee close` prints `Knowledge consolidated` and leaves no promote-proposals.md; `bee knowledge report --json` carries the cell arm and the feature arm for that feature. Use the fixture holdout percent 0 so the arms are deterministic; add one case with 100 that shows the holdout feature gets an empty pack and the report counts it in holdout. Assert observable outputs only. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test knowledge_in_the_flow_e2e",
    "must_haves": {
      "truths": [
        "one test runs pack at shaping and planning, a capped cell with a knowledge delta, consolidate, close with no proposal file, and a report with cell and feature arms"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/knowledge_in_the_flow_e2e.rs", "substantive": "the end-to-end flow test"}
      ],
      "key_links": [
        "the test drives the real binary, not internal functions"
      ],
      "prohibitions": [
        "No assertion on internal call order",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "kitf-7",
    "feature": "knowledge-in-the-flow",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": ["kitf-1", "kitf-2", "kitf-3", "kitf-4", "kitf-5", "kitf-6", "kitf-8c", "kitf-9"],
    "decisions": ["D5", "D6", "D7"],
    "files": [
      "docs/knowledge/**",
      "docs/history/knowledge-in-the-flow/delivery.md"
    ],
    "read_first": [
      "docs/history/knowledge-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-in-the-flow/plan.md",
      "docs/knowledge/areas/okf-profile/context-and-promote.md",
      "docs/knowledge/areas/workflow-state/dispatch.md",
      "docs/knowledge/areas/workflow-state/deferred-work-queue.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the new writer step on this feature: build the worktree binary (`cargo build --release --manifest-path packages/bee-rs/Cargo.toml --bin bee`) and run its `bee knowledge consolidate --feature knowledge-in-the-flow`. Dedupe and trim the deltas the cells wrote and add what is missing: okf-profile context-and-promote (B5 now: close writes no proposal, consolidate is the writer's brief; the pack verb and the code lines of the one assembler), workflow-state dispatch (pack rows, advisor rows, feature arm), deferred-work-queue (close no longer enqueues promote records), the knowledge report concept (feature outcomes), the cells cap concept (knowledge_delta). Anchor each to the code it describes, never restate code (D7), cite this feature's CONTEXT.md D-numbers, never raw decision ids. Write docs/history/knowledge-in-the-flow/delivery.md: the concepts changed and why, one line each. Run `.bee/bin/bee knowledge index`, then `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "the okf-profile and workflow-state concepts describe the pack, the knowledge delta and consolidation, anchored to code",
        "docs/history/knowledge-in-the-flow/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {"path": "docs/history/knowledge-in-the-flow/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": [
        "the delivery lands in the same merge as the code"
      ],
      "prohibitions": [
        "No raw decision ids of this feature in area docs",
        "No restating of code"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

High-risk: the applicable edge dimensions of `skills/bee-planning/references/edge-dimensions.md`, one probe each.

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Empty input | Bundle absent or empty | kitf-1 | `bee knowledge pack` exits 0 with an empty pack |
| Boundary | 9 anchored code paths | kitf-1 | exactly 8 code lines |
| Invalid input | `--stage deploy` | kitf-1 | typed usage refusal, no row appended |
| Duplicate | Two concepts anchor the same file | kitf-1 | the file is listed once |
| Security | A newline in an anchor path | kitf-1 | the line is collapsed to one line |
| Backward compatibility | Bundle with no anchors | kitf-1 | cell Learned context bytes equal to before |
| Measurement | Holdout percent 100 and 0 | kitf-1, kitf-2, kitf-9 | arm holdout with empty pack; arm knowledge with the pack |
| State | Close with and without delivery.md | kitf-4 | the two lines; no promote-proposals.md either way; same exit code |
| Concurrency | Not applicable — every verb here appends one row or reads | — | — |
| Recovery | Merge with zero docs/knowledge files | kitf-4 | `knowledge: 0 docs/knowledge file(s) in this merge; delivery: none` |
| Regression | Cell-level report block | kitf-3 | text and JSON unchanged |
| End to end | The flow in one repo | kitf-9 | the integration test green |

A changed behavior row: close on main writes promote-proposals.md; close on head writes none (kitf-4 test).

## Open Questions

(none)

## Out of scope

- Draining host repos' debt (waggledance and the 11 host repos): each host drains through the shipped flow after it upgrades.
- A release: `scripts/release.sh` runs on the owner's request after this merges.
