---
type: research
status: closed
claimed-by:
blocked-by: none
---

## Question

Across docs/knowledge/, skills/, and .bee decisions: which rules
currently live in more than one file with differing wording? Produce a
list of duplicated rules with their file set, so ticket 001 can pick a
home policy from real cases, not guesses.

## Answer

Research digest (read-only sweep, 2026-08-22).

### Area → code/skill path mapping: does not exist

Frontmatter keys over 109 area concepts: `id, lifecycle, areas,
required_context, decisions, sources, authoritative_for, tags`. No
`owns:` / `paths:`. `authoritative_for` is a prose topic string, never a
path list — zero hits for `packages/`, `skills/`, `scripts/` inside it.
Code and skill paths appear only in free-text "Pointers
(implementation)" body sections, unchecked by any schema. Nothing
declares which skill or crate an area governs.

### Duplicated rules (12 cases)

| # | Rule | Sites | Verdict |
|---|---|---|---|
| 1 | Delegation threshold (when a mechanical step delegates down-tier) | skills/bee-hive/references/gates-and-delegation.md:127 (">3 files"); docs/knowledge/areas/doctrine-layer/delegation-threshold.md:42-46 ("historical heuristic, not pinned"); AGENTS.md:109-113 (no number) | CONTRADICT — knowledge retires the number the skill still states |
| 2 | Write-guard allowlist for docs-lane writes | skills/bee-hive/references/routing-and-contracts.md:167 (`docs/` blanket) vs packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:33-43 (gated phase = `docs/history/` only) | DRIFT vs code; no knowledge concept carries the list |
| 3 | Never build on red | AGENTS.md:95; skills/bee-hive/SKILL.md:108; gates-and-delegation.md:118; skills/bee-swarming/SKILL.md:150; swarming-reference.md:273-277; areas/doctrine-layer/unenforced-obedience.md:51-56 | DRIFT — five scopes |
| 4 | Cap records proof line; close/merge check it, run nothing | AGENTS.md:92-97 + 8 skill/knowledge sites | AGREE, ~9 copies; decision 5a6a1e17 records it going stale in 3 surfaces before |
| 5 | "never zero X workers" | AGENTS.md:120 (execution); areas/doctrine-layer/helper-classes-and-transports.md:49 (I/O); gates-and-delegation.md:131 (ceremony) | DRIFT — same phrase, three subjects |
| 6 | Close every task with a capture line or "nothing settled" | AGENTS.md:155; bee-hive SKILL.md:54,110; routing-and-contracts.md:56,167,175; bee-capturing SKILL.md:43 | AGREE |
| 7 | ~65% context → HANDOFF.json | AGENTS.md:205 + 5 sites | AGREE; only swarming-reference.md:520 carries the schema |
| 8 | Gates never self-approved; gate_bypass the one exception | AGENTS.md:31-33; gates-and-delegation.md:74,110; bee-hive SKILL.md:101; bee-herding README.md:34 + role-dispatch.md:34-40 | DRIFT — herding requires full/total, inverts the opt-in framing, no cross-ref |
| 9 | One commit per cell, id as trailer | AGENTS.md:206-209; worker-details.md:147-151 | AGREE |
| 10 | Review is user-invoked, never automatic | AGENTS.md:23,236 + 4 sites | AGREE |
| 11 | Close on ONE next action | AGENTS.md:162-166; routing-and-contracts.md:222-223; the-communication-contract.md:48,123 | AGREE |
| 12 | Worktree-first | AGENTS.md:41-46 + 5 sites | AGREE on rule, DRIFT on exemption list (each site a different set) |

### Worst artifact

docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md:100-115
and :185-195 — three stacked in-place amendment notes (07-27 → 07-31 →
08-18), each reversing the last. The live rule is readable only by
taking the last note. Same file decision 5a6a1e17 already corrected once.

### Intended source of truth

AGENTS.md reads as the standing sheet for rules 3–12: always loaded,
terse, cited by skills. docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:29-31
declares the boundary ("anything the router restates is paid twice") —
declared but unenforced: no field, no check. For code behavior (#2)
the crate is truth and the prose is a stale copy; `authoritative_for`
cannot say so because it holds no paths.
