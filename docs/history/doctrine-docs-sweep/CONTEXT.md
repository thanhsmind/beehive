# Doctrine and docs sweep — Context

**Feature slug:** doctrine-docs-sweep
**Date:** 2026-11-19
**Backlog items:** p-00af3fb7, p-fbfa8325, p-0dfb7aa9, p-6c3e4223, p-77bfcb18, p-b97e839a, p-982de3cf
**Shaping session:** complete under gate bypass full (owner said go on the 2026-11-19 backlog sweep)
**Scope:** Standard (docs lane)
**Domain types:** READ

## Feature Boundary

Seven doctrine and reference changes in `skills/`, `packages/bee/` and
`.bee/expertise/`. No Rust code changes except where a test pins a moved file
path.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `skills/bee-planning/playbooks/feature/feature.md` gains a step: for a package or API that other code calls, write the caller's tutorial first and check the result against it. A short page `skills/bee-planning/playbooks/session/steer-recipes.md` maps one-line steer prompts to bee situations and principles, and the session index links it. | p-00af3fb7; docs/history/research/pstack-guide-distill.md G4. |
| D2 | `skills/bee-planning/playbooks/session/autonomous-run.md` and the bee-herding bootstrap protocol (`skills/bee-herding/references/role-bootstrap.md`) check four preconditions before an unattended loop: the task was done once by hand, the agent has the tools, every stage can stop the line, repeated failures already became checks. | p-fbfa8325; pstack guide G5. |
| D3 | The shipped leader doctrine (`packages/bee/LEADER.md` and the AGENTS block source) states that the project leader triages every proposed PBI itself (build, park, or drop) and that only a money, publishing or irreversible call goes to the owner, decision-ready. The owner-brief contract (`docs/knowledge/areas/decision-memory/owner-brief-contract.md`) and bee-shaping first triage say the "Your call" paragraph carries the leader's verdict, not an open choice. | p-0dfb7aa9; decision 7942f87c; spec drop sup-20261117-a7d2. |
| D4 | Question-batching policy, one home in `skills/bee-hive/references/routing-and-contracts.md` § Question Format: ask one question at a time when an answer can reframe the scope; ask a numbered batch in one round when the questions are independent of each other (a wayfinding frontier). Shaping and wayfinding point to that home; the conflicting lines in `skills/bee-shaping/references/shaping-reference.md` and the wayfinding docs are aligned. The policy is logged with `bee decisions log`. | p-6c3e4223. Leader call per decision 7942f87c. Both current rules are each right for their case; the split is by dependency. |
| D5 | `skills/bee-researching/references/trace-and-provenance.md` splits into `trace.md` and `provenance.md`. bee-how loads only trace, bee-why only provenance; every referrer is repointed. | p-77bfcb18. |
| D6 | `.bee/expertise/review.md` splits into a reviewer-side file and an author-side file (asking for a review, receiving a finding, reviewing your own work). A dispatched reviewer loads only the reviewer-side file; every referrer is repointed. | p-b97e839a. |
| D7 | `packages/bee/design/bee-design/references/contract.md` covers every `ui` command that core references name: `ui vr gate|diff|accept`, `ui ds soul check|init`, `ui ds a11y`, and any other found by a search of the references. | p-982de3cf. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `skills/bee-planning/playbooks/feature/feature.md`, `skills/bee-planning/playbooks/session/` | D1, D2. |
| `skills/bee-herding/references/role-bootstrap.md` | D2. |
| `packages/bee/LEADER.md`, `packages/bee/AGENTS.block.md` | D3. |
| `docs/knowledge/areas/decision-memory/owner-brief-contract.md` | D3. |
| `skills/bee-hive/references/routing-and-contracts.md`, `skills/bee-shaping/references/shaping-reference.md` | D4. |
| `skills/bee-researching/references/trace-and-provenance.md`, `skills/bee-how/SKILL.md`, `skills/bee-why/SKILL.md` | D5. |
| `.bee/expertise/review.md` | D6. |
| `packages/bee/design/bee-design/references/contract.md` | D7. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
