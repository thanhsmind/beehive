# Token spend cuts — Context

**Feature slug:** token-spend-cuts
**Date:** 2026-11-21
**Scope:** Standard
**Domain types:** RUN, CONFIG

## What was asked

Spec drop sup-20261121-7c4e from waggledance. The owner read the
survey-platform transcripts of 2026-11-20 and 2026-11-21 (bee 2.112.0) and
approved five cuts. Total spend was 71 effM over 93 sessions; 65 percent was
cache read, so context size times call count. Two leader sessions ran about
4.5 hours each with 12 merges each and no compaction, and peaked at 556k and
425k tokens. Opus was 77 percent of spend. Read was 41 percent of tool output.
Workers start near 20k tokens before work. The semantic judge passed 16 of 16
and changed nothing.

Owner's words, items 1 and 2: hand off and open a new session after merge;
the compact threshold is configurable, compact past 300k tokens. Move test to
sonnet, small design may use sonnet, and make the role descriptions fit every
project. Items 3, 4 and 5: as suggested.

## What will be done

1. A leader hands off at the next clean stop once its context reaches the
   mark (default 300k), and after every green worktree merge.
2. Test and small design cells run on sonnet; hard ones are escalated by the
   plan. Shipped role text stays stack-neutral; the owner gets the exact edit
   for their user-global config.
3. Worker templates tell workers to locate with `rg -n` and read ranges.
4. The worker brief drops the `docs/knowledge/index.md` fallback and the
   knowledge pack budget halves.
5. Small cells (under 30 changed lines) skip the judge on a standard lane, and
   the tester returns the judge block with its test report.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | One leader session per feature. `handoff_context_tokens` default rises from 150000 to 300000; 0 still turns it off. At or past the mark the prompt-context line tells the leader to hand off at the next clean stop (a capped cell or a merged worktree), never mid-cell. A green `bee worktree merge` ends its next action with the fresh-leader-session handoff. Supersedes context-mark-advisory D1 and D2. | 40c00f86 |
| D2 | Test and design cells run on their role model (sonnet in the shipped template). A hard test cell or a substantial design cell is marked with `bee cells escalate` at plan time. Shipped role descriptions name no stack. The owner edits the user-global team keys. | b9049c94 |
| D3 | Every worker agent template carries one read rule: `rg -n` first, then a Read range with offset and limit; no `cat`, `head`, `tail` or `sed` dumps; a whole file only when the work edits most of it or it is short. | ff5721fd |
| D4 | The learned-context block never falls back to `docs/knowledge/index.md`. Knowledge pack budgets: tiny 4000, small 6000, standard 10000, high-risk 15000, default 10000 tokens. The knowledge A/B holdout is unchanged. | ea4e544e |
| D5 | On a standard lane the judge-debt door skips a capped behavior_change cell whose cap commit changed fewer than 30 lines. High-risk still owes every verdict. A code cell's tester returns the judge block too; the leader records it with `bee cells judge-record`. | 7e452d6b |
| C1 | Test contract handoff-mark-directive. | 56a0eec4 |
| C2 | Test contract knowledge-pack-budget. | ce42c69f |
| C3 | Test contract judge-small-diff. | 3e94e127 |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | the context mark line and its default |
| docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | suspect | roles, escalation flag, effort ladder |
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | suspect | judge-debt door and judge tiers |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` | `compute_handoff_warning`, the mark and its line |
| `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs` | the green merge next action |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | `lane_budget`, `bundle_learned_lines` |
| `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` | `judge_debt` and the judge-debt door |
| `packages/bee/agents/` | the worker agent templates |
| `packages/bee/playbooks/handoff.md` | rule agents-context-handoff-65 |

## Out of scope

The knowledge A/B arms. Host-side automatic compaction. Changing the owner's
user-global config file: its team keys are user-edited.
