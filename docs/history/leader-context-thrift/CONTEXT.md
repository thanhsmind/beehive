# Leader context thrift — Context

**Feature slug:** leader-context-thrift
**Date:** 2026-11-21
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Keep the leader's context small, so each call re-reads less. Six parts:
an earlier handoff mark, a handoff record that carries what a fresh
leader needs, a cold-read probe of that record, a guard that sends large
file reads to a read worker, a dispatch output that leaves the worker
prompt in a file, and a report that measures the result. It ends at the
leader's own context; worker prompts, model roles and the preamble
budget do not change.

## What was asked

The owner read a post on graph engineering (specialist nodes, routing in
code, scoped state per node) and asked whether bee could use it to spend
fewer tokens. The leader measured this repo's transcripts; the owner
asked to go deeper on keeping the leader context small, asked why the
same content is read again and again, asked whether a handoff hurts the
model's reasoning or the cache cost, and then said to build it.

## Measurements

Source: `message.usage` of every transcript under this project's Claude
projects directory, 2026-11-21; 207 leader sessions, 146 herded worker
sessions, 108 subagents. Weights for cost share: Opus and Fable 5, Sonnet
3, Haiku 1; cache read 0.1, cache write 1.25, output 5 (assumed list
ratios, not checked per model).

- Leader 90.1 percent of weighted cost; subagents 7.1; herded workers 2.8.
- Leader mean context per call 208k; cache read is 81 percent of leader
  cost. 68 percent of leader input comes from calls above 200k context.
- Peak context per leader session: median 106k, p90 382k; 67 sessions
  pass 200k.
- Attributed leader re-read (size times later calls, chars/4 tokens,
  covers about 58 percent of measured input): file and search results
  about 40 percent, results over 2k tokens 24 percent, the leader's own
  tool arguments 26 percent (Write 9.4, of which 31 percent are worker
  briefs), bee command output about 12 percent (`dispatch prepare` 2.3).
- Fixed first-call prefix: median 26.7k; about 9k of it is already cached
  at a new session's first call (283 of 353 sessions read cache there).
- Replay with a reset at the mark and a 35k restart: 200k cuts 47
  percent of leader input (94 handoffs), 160k 55 (141), 120k 64 (234),
  80k 73 (496). An upper bound: the successor re-reads nothing.
- Handoff cost: about 35k written at the 1h cache rate (2x), pays back
  after about 6 calls at 150k. Mid-session cache expiry: 46 rewrites,
  10M tokens, 24 percent of all leader cache writes.
- Leader tool errors fall from 14.7 percent below 50k to 2.8 percent
  above 400k, but 34 to 50 percent of them are guard refusals and 22 to
  27 percent user denials: a warm-up cost per session, not a reasoning
  measure. Handoff-started sessions in the sample: too few to measure.

## Locked Decisions

| ID | Decision | Store id |
|----|----------|----------|
| D1 | The default of `handoff_context_tokens` falls from 200000 to 120000, and the beehive project config moves from 200000 to 120000. The key, the directive line text, the clean-stop rule and the 0 switch stay as token-spend-cuts D1 set them. | 985cc944 (supersedes 7bc2853d) |
| C1 | Test contract handoff-mark-directive: with the key absent, 120000 context tokens or more adds exactly one line that names the size and the 120k mark; 119999 adds none; 0 adds none at any size; a configured number wins over the default. | 38d13294 (supersedes 03ca8fc1) |
| D2 | `bee state handoff write` takes `--verified`, `--tree`, `--key-files`, `--gotchas`, `--standing-orders`, `--ruled-out` on both kinds, stored under snake_case keys; key_files splits on commas, verified, gotchas, standing_orders and ruled_out split on semicolons, tree stays a string. `bee state handoff show` and the preamble HANDOFF block print each present field on its own line, capped at 300 characters. | 6d462761 |
| C2 | Test contract handoff-record-fields: each D2 flag reads back through `bee state handoff show`, split as D2 says, for both kinds; no flag stores no key; the preamble block prints a present field and omits an absent one. | 7f7519bd |
| D3 | Before a leader releases its session after a handoff write, it runs one cold-read probe: a read-role gather whose brief holds only the `bee state handoff show` output and asks for the next action and the ruled-out paths. A miss means rewrite the record and probe once more. The handoff playbook and the swarming reference state the step. | 4945350f |
| D4 | In a leader session the guard refuses a Read with no limit on a file longer than `leader_read_max_lines` (default 400, 0 off), and a Bash command whose only segment is `cat` of one such file. The refusal names offset and limit and a read gather through `bee dispatch prepare`. Workers, short files and ranged reads are unchanged. | c1d2947a |
| C3 | Test contract leader-read-cap: a 401-line unranged Read in a leader session is refused with both remedies named; 400 lines passes; a ranged Read passes; a worker passes; 0 passes every size; a bare `cat` of 401 lines is refused, a piped or ranged one passes. | 6d4f74aa |
| D5 | `bee dispatch prepare --prompt-out <path>` writes the prompt to the path on the herding and cli-exec arms, drops payload stdin and returns a command that reads `--task-file <path>`; prompt_sha256 is unchanged; the Agent arm ignores it. bee-swarming and the dispatch door line tell the leader to pass it. | 1fef0cf9 |
| C4 | Test contract dispatch-prompt-out: the file holds the prompt whose sha256 equals prompt_sha256, stdin is absent and the command names `--task-file` with the path; without the flag the output is byte-identical. | e01214ca |
| D6 | `bee report context --since <iso>` prints, for leader sessions (transcripts no mailbox job claims), session count, calls, mean and p90 context per call, the input share above the handoff mark, and for handoff-started sessions the count of `cells reopen` and `mailbox reflect` commands in their first 40 calls; it takes `--json`. | 2d8d37a5 |
| C5 | Test contract report-context: on two leader fixtures and one claimed worker fixture, only the leaders count, the mean comes from message usage, above-mark calls land in the share, and a `cells reopen` inside a handoff-started session's first 40 calls is counted. | 70dda218 |

### Agent's Discretion

- The module that holds the D4 guard and how it reads the line count.
- The split of `bee report context` into a verb file and a pure aggregation
  function, as long as C5 runs on fixtures.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | home of the handoff mark rule and the token budget |
| docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md | suspect | the read role the D3 probe and D4 remedy dispatch on |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` | `compute_handoff_warning` holds the default mark; its tests pin it |
| `packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs` | `write_mailbox_handoff` builds the untyped handoff record |
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs` | `handoff_block_lines` renders the HANDOFF block |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` | the Read-tool guard path a leader read cap joins |
| `packages/bee-rs/crates/bee/src/hooks/mod.rs` | `herding_worker_marker_set` tells a worker from a leader |
| `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | the dispatch envelope that inlines the worker prompt |
| `packages/bee/playbooks/handoff.md` | the handoff rule text and default mark |

## Out of scope

- The preamble byte budget and its drop order.
- Model roles and the team config.
- A hard stop at the mark; the line stays a directive at the next clean stop.
- Worker-side read rules; workers already carry their own search rule.
