# Tool call thrift — Context

**Feature slug:** tool-call-thrift
**Date:** 2026-11-21
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Make the leader spend fewer tool calls, and make each call return less
text. Five parts: a batching rule in the leader doctrine, a brief help
card, refusals that print the verb's flags, a preamble line that stops
asking for a help read before each verb, and a leader read guard that
also catches a multi-file `cat`. A sixth part is doctrine only: start a
cell with the existing claim-and-prepare fold. It ends at the leader;
worker prompts, model roles and the handoff mark do not change.

## What was asked

After leader-context-thrift shipped, the owner asked whether token use
fell and how much rework the new sessions had. The leader measured it,
the owner said the cost comes from too many tool calls and asked for a
better way, the leader proposed four changes, and the owner said to do
all of them.

## Measurements

Source: `message.usage` and the tool_use/tool_result pairs of six leader
transcripts (f2ee5669, 4c124e29, b1444ff5, 838f06b2, 01699838,
b1703487), 2026-11-21.

- 1023 tool calls in 828 tool turns; 135 turns (16 percent) ran more
  than one tool.
- Input tokens: cache read 151.6M of 153.1M. Weighted cost (cache read
  0.1, cache write 1.25, output 5): cache read 77 percent, output 13,
  cache write 9.
- Help reads: 109 calls, 313.7k chars, 2.9k chars each.
- bee bookkeeping verbs (cells, dispatch, state, herding, worktree):
  about 270 calls, 26 percent.
- Whole-file reads (cat, Read, wc): 57 calls, 5k to 10k chars each.
- The help of cells finish alone is 6.6k chars.

## Locked Decisions

| ID | Decision | Store id |
|----|----------|----------|
| D0 | Scope, approved by the owner: the parts in the boundary above. | a681eeb7 |
| D1 | The leader doctrine Token efficiency block states that every tool call re-reads the whole session; independent commands go in one Bash call or as parallel calls in one message; a lone short command needs a reason (its output decides the next command). | 4982d4a5 |
| D2 | The native help path accepts `--brief` with `--help` for one verb or a group: per verb the usage line, the one-line summary and one line per flag (name, `*` when required, type, description cut at 80 chars with an ellipsis); no details paragraph, no header note. Without `--brief` the output is byte-identical. | 12391e63 |
| C1 | Test contract help-brief: the brief card of cells finish names every flag once on its own line, has no details line, is under 2500 bytes; the full help of one verb and of a group is unchanged. | e5e7d363 |
| D3 | The router's missing-required and unknown-flag refusals and the CLI-shape guard denial append the verb's brief flag card from the D2 renderer; the correction line points at `--help --brief` instead of `--help --json`. | 3876c870 |
| C2 | Test contract refusal-card: cells finish with no `--id` is refused by the router with the `--id` card line in the text; an unknown flag is refused with the card; the shape guard denial for the same call carries the card and points at `--help --brief`. | 6f1ae265 |
| D4 | The preamble Command surface line drops the read-help-before-first-use instruction: run the verb, a wrong shape is refused before it runs with the flags listed, the brief card shows them up front. Skills that say to read help before use follow the same line. | 6fb5c82e |
| C3 | Test contract preamble-command-surface: the rendered Command surface block no longer holds "FIRST use" and names `--help --brief`; the preamble stays inside its byte budget. | e7428371 |
| D5 | The leader doctrine names one start per cell: dispatch prepare with `--claim` for one cell, dispatch wave for a wave; no separate claim and reserve before prepare. | a2d1736f |
| D6 | The leader read guard also refuses a Bash command whose only segment is `cat` of two or more files whose summed line count passes `leader_read_max_lines`; one-file `cat`, ranged and piped reads, workers and the 0 switch stay as leader-context-thrift D4 set them. | 94d78600 |
| C4 | Test contract leader-read-multi-cat: a leader's bare `cat` of two files summing past the limit is refused; a sum at the limit passes; a worker passes; limit 0 passes; a piped multi-file `cat` passes; one-file behavior is unchanged. | 3a2194d7 |

### Agent's Discretion

- Where the brief renderer lives inside the help module, as long as the
  refusals in D3 call the same function.
- How the multi-file `cat` parser shares code with the one-file parser.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/rust-runtime/command-surface.md | suspect | home of the help surface rules |
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | home of the leader token rules |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/help.rs` | native help: `try_native` (:51), `render_help_text` (:610), `flags_detail_lines` (:562) |
| `packages/bee-rs/crates/bee/src/router.rs` | missing-required (:458) and unknown-flag (:486) refusals |
| `packages/bee-rs/crates/bee/src/hooks/cli_shape.rs` | `render_denial` (:628), correction line (:647) |
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` | the Command surface line (:161) |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/leader_read.rs` | `cat_operand`, `unranged_target`, `leader_read_refusal` |
| `packages/bee/LEADER.md` | the leader doctrine source; `.bee/LEADER.md` is its regen output |
| `packages/bee/playbooks/leader-detail.md` | § Token efficiency detail |

## Out of scope

- The default `--help` output and `--help --json`.
- A new verb that folds the cell lifecycle; the fold already exists in
  dispatch prepare `--claim` and dispatch wave.
- Worker prompts, model roles, the handoff mark, the preamble budget size.
