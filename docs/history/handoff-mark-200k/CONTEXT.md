# Handoff mark 200k — Context

**Feature slug:** handoff-mark-200k
**Date:** 2026-11-21
**Shaping session:** complete
**Scope:** Small
**Domain types:** RUN

## Feature Boundary

Lower the default context mark at which a leader hands off, from 300000
to 200000 tokens. It ends at the default value, the tests that pin it,
the handoff playbook sentence and the token economy concept. The line
text, the clean-stop rule and the 0 switch do not change.

## What was asked

The owner asked which token savings are still open. The leader measured
the reread-thrift session and gave two options for the mark. The owner
chose 200k.

Evidence (`bee report usage` on that session, and its transcript):

- Total 55.0M tokens: the leader 30.4M, 14 workers 24.3M.
- Leader: 153 calls, context from 27.5k to 296k tokens, mean 198k. The
  fixed start context is 14 percent of the leader total.
- Replay of the same context curve with a reset at the mark, restart at
  60k plus 450k of re-read cost: 200k gives 17.9M (one handoff), 150k
  gives 16.0M (two handoffs), 250k gives 23.0M. One run; the spread is
  not measured.
- The beehive config held `handoff_context_tokens: 0`, so the mark was
  off in this repo. It is now 200000 (main, commit 7716adac1).

## Locked Decisions

| ID | Decision | Store id |
|----|----------|----------|
| D1 | The default of `handoff_context_tokens` falls from 300000 to 200000. The key, the directive line text, the clean-stop rule and the 0 switch stay as token-spend-cuts D1 set them. The beehive project config moves from 0 to 200000. | 7bc2853d (supersedes 40c00f86) |
| C1 | Test contract handoff-mark-directive: with the key absent, 200000 context tokens or more adds exactly one line that names the size and the 200k mark; 199999 adds none; 0 adds none at any size; a configured number wins over the default. | 03ca8fc1 (supersedes 56a0eec4) |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | home of the mark rule; states the 300000 default |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` | `compute_handoff_warning` holds the default; its test module pins the mark |
| `packages/bee/playbooks/handoff.md` | the rule sentence that names the default mark |
