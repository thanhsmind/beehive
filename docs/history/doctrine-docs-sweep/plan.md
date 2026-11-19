---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: doctrine and docs sweep

## Summary

Seven doctrine and reference changes from the 2026-11-19 backlog sweep, each
one docs cell, and one checker that runs the parity and pointer tests over all
seven.

Mode: `standard` (docs lane) — skills, doctrine and reference files only; the
review expertise split keeps the dispatched file name so no code changes.

## Requirements (from CONTEXT.md)

- D1 to D7, one cell each (ddx-1 to ddx-7); ddx-8 checks them.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Dispatch prepare injects the review expertise by its path | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1332 | `.bee/expertise/review.md :: finding quality and severity calibration` |
| 2 | bee-how loads the whole trace-and-provenance file | read | skills/bee-how/SKILL.md:18 | `trace-and-provenance` |
| 3 | Wayfinding asks the whole frontier in one round | read | skills/bee-wayfinding/CREATION-LOG.md:48 | `ask the whole frontier in one round, numbered` |

## Approach

Situation: `skills/bee-planning/playbooks/docs/docs.md`.

Seven independent docs cells run in one wave; skills-touching cells carry the
wave-barrier regen acknowledgement and the leader runs `bee dev regen` once at
wave close. ddx-8 checks all seven.

Rejected: moving the reviewer side to a new file name (it would need a
prepare.rs change and a tester for it).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {
      "stage": "implementation",
      "classification": "not-applicable",
      "role": "code",
      "reason": "No code cell; ddx-6 keeps prepare.rs unchanged."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "required",
      "role": "test",
      "reason": "ddx-8 checks every docs cell."
    },
    {
      "stage": "docs",
      "classification": "required",
      "role": "docs",
      "reason": "ddx-1 to ddx-7 write the doctrine and references."
    },
    {
      "stage": "slice-judge",
      "classification": "not-applicable",
      "role": "review",
      "reason": "Review stays user-invoked."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "not-applicable",
      "role": "hat-facts-gaps",
      "reason": "Docs lane."
    },
    {
      "stage": "hat-alternatives",
      "classification": "not-applicable",
      "role": "hat-alternatives",
      "reason": "Docs lane."
    },
    {
      "stage": "hat-user-impact",
      "classification": "not-applicable",
      "role": "hat-user-impact",
      "reason": "Docs lane."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Docs lane."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Owner-approved backlog items."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "No release in this feature."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The leader wrote the plan."
    },
    {
      "stage": "read-only-gather",
      "classification": "not-applicable",
      "role": "read",
      "reason": "Referrers found by the leader."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No extraction stage."
    },
    {
      "stage": "generation-fallback",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Every cell has a configured role."
    },
    {
      "stage": "writing",
      "classification": "not-applicable",
      "role": "writer",
      "reason": "The docs role writes."
    },
    {
      "stage": "generic-advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "No consult needed."
    },
    {
      "stage": "supervision",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "Attended session."
    },
    {
      "stage": "blind-lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "No blind lanes."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "No blind lanes."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "No blind lanes."
    },
    {
      "stage": "design-ui",
      "classification": "not-applicable",
      "role": "design",
      "reason": "No rendered UI."
    }
  ]
}
```

## Risk map

| Risk | Guard |
|---|---|
| A split file leaves a dangling pointer | pointer_integrity in ddx-5, ddx-6 and ddx-8 |
| Rendered skill trees drift | one `bee dev regen` at wave close, then release-manifest --check in ddx-8 |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ddx-1 | Write the caller's tutorial first for a shared API, and add a steer recipe page | 3 file(s) | — | see truths | docs verify |
| ddx-2 | Check four trust preconditions before an unattended loop | 2 file(s) | — | see truths | docs verify |
| ddx-3 | Make the project leader triage every proposed PBI itself | 4 file(s) | — | see truths | docs verify |
| ddx-4 | Give question batching one policy and one home | 3 file(s) | — | see truths | docs verify |
| ddx-5 | Split trace-and-provenance into a trace file and a provenance file | 12 file(s) | — | see truths | docs verify |
| ddx-6 | Split the review expertise into a reviewer side and an author side | 11 file(s) | — | see truths | docs verify |
| ddx-7 | Cover every ui command the bee-design core references name | 1 file(s) | — | see truths | docs verify |
| ddx-8 | Check the doctrine and docs sweep for parity and pointers | 1 file(s) | ddx-1, ddx-2, ddx-3, ddx-4, ddx-5, ddx-6, ddx-7 | see truths | test verify |

```json
[
{"id": "ddx-1", "feature": "doctrine-docs-sweep", "title": "Write the caller's tutorial first for a shared API, and add a steer recipe page", "lane": "standard", "role": "docs", "deps": [], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["skills/bee-planning/playbooks/feature/feature.md has a numbered step that, for a package or API other code calls, writes the caller's tutorial first and checks the result against it", "skills/bee-planning/playbooks/session/steer-recipes.md maps one-line steer prompts to bee situations and principles", "skills/bee-planning/playbooks/session/index.md links steer-recipes.md"], "why": "pstack guide G4 asks for both; neither exists.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["skills/bee-planning/playbooks/feature/feature.md", "skills/bee-planning/playbooks/session/steer-recipes.md", "skills/bee-planning/playbooks/session/index.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-planning/playbooks/feature/feature.md", "skills/bee-planning/playbooks/session/steer-recipes.md", "skills/bee-planning/playbooks/session/index.md"], "affects_specs": [], "action": "Implement CONTEXT.md D1. Source: docs/history/research/pstack-guide-distill.md section G4. Keep the steer page short: a table of one-line prompts, each with the situation file or principle slug it routes to. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-2", "feature": "doctrine-docs-sweep", "title": "Check four trust preconditions before an unattended loop", "lane": "standard", "role": "docs", "deps": [], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["skills/bee-planning/playbooks/session/autonomous-run.md checks four preconditions before the loop runs: the task was done once by hand, the agent has the tools, every stage can stop the line, repeated failures already became checks", "skills/bee-herding/references/role-bootstrap.md has the same four-precondition check as a pre-flight step"], "why": "pstack guide G5 asks for a trust checklist; neither file has one.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["skills/bee-planning/playbooks/session/autonomous-run.md", "skills/bee-herding/references/role-bootstrap.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-planning/playbooks/session/autonomous-run.md", "skills/bee-herding/references/role-bootstrap.md"], "affects_specs": [], "action": "Implement CONTEXT.md D2. Source: docs/history/research/pstack-guide-distill.md section G5. In role-bootstrap.md add it as a new numbered pre-flight step after the bypass-level check, and renumber the following steps. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-3", "feature": "doctrine-docs-sweep", "title": "Make the project leader triage every proposed PBI itself", "lane": "standard", "role": "docs", "deps": [], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["packages/bee/LEADER.md states that the project leader triages every proposed PBI itself (build, park or drop) and that only a money, publishing or irreversible call goes to the owner, decision-ready", "docs/knowledge/areas/decision-memory/owner-brief-contract.md row 5 says the Your call paragraph carries the leader's verdict, not an open choice", "skills/bee-shaping/SKILL.md first triage says the leader decides and records the verdict"], "why": "Decision 7942f87c settled leader-owned triage; the shipped doctrine still offers the owner open choices.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["7942f87c-ae78-474e-b525-85eba044f723", "2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["packages/bee/LEADER.md", "packages/bee/AGENTS.block.md", "docs/knowledge/areas/decision-memory/owner-brief-contract.md", "skills/bee-shaping/SKILL.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-shaping/SKILL.md"], "affects_specs": [], "action": "Implement CONTEXT.md D3. Read decision 7942f87c (`.bee/bin/bee decisions search --text 7942f87c`) and docs/discovery/spec-drops/sup-20261117-a7d2.md. Edit packages/bee/AGENTS.block.md only if it holds the triage rule text; if the rule lives only in LEADER.md, leave AGENTS.block.md unchanged and say so. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-4", "feature": "doctrine-docs-sweep", "title": "Give question batching one policy and one home", "lane": "standard", "role": "docs", "deps": [], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["skills/bee-hive/references/routing-and-contracts.md section Question Format states the policy: one question at a time when an answer can reframe the scope; a numbered batch in one round when the questions are independent", "skills/bee-shaping/references/shaping-reference.md and skills/bee-wayfinding/SKILL.md point to that section and carry no conflicting batching rule"], "why": "Shaping says a batch buries the reframe; wayfinding says ask the whole frontier at once.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["a0401ac2-4611-4ef6-b59d-1cc540bba9b5", "2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["skills/bee-hive/references/routing-and-contracts.md", "skills/bee-shaping/references/shaping-reference.md", "skills/bee-wayfinding/SKILL.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-hive/references/routing-and-contracts.md", "skills/bee-shaping/references/shaping-reference.md", "skills/bee-wayfinding/SKILL.md"], "affects_specs": [], "action": "Implement CONTEXT.md D4 and the decision named in the CONTEXT. Keep each pointer one line. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-5", "feature": "doctrine-docs-sweep", "title": "Split trace-and-provenance into a trace file and a provenance file", "lane": "standard", "role": "docs", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity && .bee/bin/bee knowledge check", "must_haves": {"truths": ["skills/bee-researching/references/trace-and-provenance.md no longer exists and trace.md and provenance.md hold its two halves", "skills/bee-how/SKILL.md loads only trace.md and skills/bee-why/SKILL.md loads only provenance.md", "no file outside docs/history and CREATION-LOG.md files names trace-and-provenance.md", "the pointer_integrity test passes"], "why": "bee-how and bee-why each load a file half of which they never use.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["skills/bee-researching/references/trace-and-provenance.md", "skills/bee-researching/references/trace.md", "skills/bee-researching/references/provenance.md", "skills/bee-researching/SKILL.md", "skills/bee-how/SKILL.md", "skills/bee-why/SKILL.md", "skills/bee-planning/references/design-sketch.md", "skills/bee-recall/SKILL.md", "skills/bee-blast-radius/SKILL.md", "skills/bee-teach/SKILL.md", "skills/bee-planning/playbooks/research/investigation.md", "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-researching/references/trace-and-provenance.md", "skills/bee-researching/references/trace.md", "skills/bee-researching/references/provenance.md", "skills/bee-researching/SKILL.md", "skills/bee-how/SKILL.md", "skills/bee-why/SKILL.md", "skills/bee-planning/references/design-sketch.md", "skills/bee-recall/SKILL.md", "skills/bee-blast-radius/SKILL.md", "skills/bee-teach/SKILL.md", "skills/bee-planning/playbooks/research/investigation.md"], "affects_specs": [], "action": "Implement CONTEXT.md D5. Split the file by its own sections; delete the old file. Repoint every referrer in this cell's list to the half it uses (both halves if it uses both). Leave CREATION-LOG.md files and docs/history untouched. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-6", "feature": "doctrine-docs-sweep", "title": "Split the review expertise into a reviewer side and an author side", "lane": "standard", "role": "docs", "deps": [], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity && .bee/bin/bee knowledge check", "must_haves": {"truths": [".bee/expertise/review.md holds only the reviewer side (how to review, where to look, what a finding is, severity calibration, the rubric)", ".bee/expertise/review-author.md holds the author side (asking for a review, receiving a finding, reviewing your own work)", "every referrer that cites an author-side section points to review-author.md", "the pointer_integrity test passes"], "why": "A dispatched reviewer loads the author-side half it never uses.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": [".bee/expertise/review.md", ".bee/expertise/review-author.md", ".bee/expertise/INDEX.md", "skills/bee-planning/references/plan-artifact.md", "skills/bee-hive/references/gates-and-delegation.md", "skills/bee-reviewing/SKILL.md", "skills/bee-reviewing/references/reviewing-reference.md", "skills/bee-principle-verify-before-reporting/SKILL.md", "skills/bee-principle-minimize-reader-load/SKILL.md", "docs/handbook/stages/reviewing.md", "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": ["skills/bee-planning/references/plan-artifact.md", "skills/bee-hive/references/gates-and-delegation.md", "skills/bee-reviewing/SKILL.md", "skills/bee-reviewing/references/reviewing-reference.md", "skills/bee-principle-verify-before-reporting/SKILL.md", "skills/bee-principle-minimize-reader-load/SKILL.md"], "affects_specs": [], "action": "Implement CONTEXT.md D6. Keep the reviewer side in review.md so dispatch prepare (packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs near 1332) keeps injecting the right file with no code change. Move the sections headed Asking for a review, Receiving a finding and Reviewing your own work into review-author.md. Add review-author.md to .bee/expertise/INDEX.md. Repoint only referrers that cite the author side. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-7", "feature": "doctrine-docs-sweep", "title": "Cover every ui command the bee-design core references name", "lane": "standard", "role": "docs", "deps": [], "verify": ".bee/bin/bee knowledge check", "must_haves": {"truths": ["packages/bee/design/bee-design/references/contract.md covers ui vr gate, ui vr diff, ui vr accept, ui ds soul check, ui ds soul init and ui ds a11y", "every ui command named in packages/bee/design/bee-design/references is covered by contract.md"], "why": "Core references name ui commands the contract leaves out.", "out_of_scope": ["files outside this cell's list"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["packages/bee/design/bee-design/references/contract.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Implement CONTEXT.md D7. First run rg -o over packages/bee/design/bee-design/references for every `ui <words>` command, then add each missing one to contract.md in the existing entry shape (read the source references for its flags and outputs; do not invent behavior; where a reference gives no detail, say so in the entry). Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it.", "regen_obligation_ack": "wave-barrier"},
{"id": "ddx-8", "feature": "doctrine-docs-sweep", "title": "Check the doctrine and docs sweep for parity and pointers", "lane": "standard", "role": "test", "deps": ["ddx-1", "ddx-2", "ddx-3", "ddx-4", "ddx-5", "ddx-6", "ddx-7"], "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test agents_block_render_parity --test rule_index_parity --test class_playbook_parity && .bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check", "must_haves": {"truths": ["the verify command passes", "each truth of ddx-1 through ddx-7 is checked against the files and marked held or not held in the report", "the report under docs/history/doctrine-docs-sweep/reports/ddx-8.md quotes each command and its result line"], "why": "A docs cell never certifies itself.", "out_of_scope": ["editing any checked file"]}, "decisions": ["2f624012-edb2-4c56-9634-e2d75b6a973c"], "files": ["docs/history/doctrine-docs-sweep/reports/ddx-8.md"], "read_first": ["docs/history/doctrine-docs-sweep/CONTEXT.md"], "affects_skills": [], "affects_specs": [], "action": "Check every truth of cells ddx-1 to ddx-7 (bee cells show --id <id> --json) against the committed files, without reading their cap reports. Run the verify. Write the report. Edit no other file. Return BLOCKED with evidence when a truth does not hold. Write no code comment. Run only the cell verify. Commit once with `git commit -- <your files>` (git add may be refused) and the cell trailer. The wave leader runs bee dev regen once at wave close; do not run it."}
]
```
