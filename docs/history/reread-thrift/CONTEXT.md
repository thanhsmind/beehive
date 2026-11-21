# Reread thrift — Context

**Feature slug:** reread-thrift
**Date:** 2026-11-21
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

Cut the tokens that a leader and its workers re-read on every model call:
the leader's context that never resets between features, the leader's raw
reads, the worker's unused listings, the worker's wide searches, the leader
doctrine file and the fixed text of the cell brief. It ends at bee's own
code, prompts and doctrine; it does not change paseo or the host.

## What was asked

The owner asked which points in the flow still waste tokens. The leader
measured the session that ran token-spend-cuts, leader-call-thrift and
wave-chain-thrift, and listed five points and one small one. The owner said
"Làm toàn bộ đi".

Evidence (`bee report usage` on that session, and its transcripts):

- Total 103.4M tokens: the opus leader 80.2M, 25 workers 23.2M.
- Leader: 334 calls, context from 27k to 388k tokens, 196 calls above 200k.
  A context reset at each of the two merges gives 40.6M in place of 76.0M.
- Leader context by re-read weight: raw shell reads 25 percent, thinking 21
  percent, bee verb output 10 percent.
- Worker: 471 calls, first-call context about 20k tokens. `rg` and `fd`
  output was 41 percent of worker tool output; the brief read was 20 percent.

## Locked Decisions

| ID | Decision | Store id |
|----|----------|----------|
| D1 | A successful `bee worktree merge` ends with one line that tells the leader to hand off before the next feature. `bee feature start` refuses with `handoff_due` when the calling session already merged a different feature, unless `--same-session` carries a reason, which goes to the audit log. A caller with no session id is not refused. | 6746c0d3 |
| D2 | `bee herding result --job <id>` prints a digest of the job's result file (status, summary, proof, files_changed, found, unknowns, skipped, report_path), at most 1500 characters, never the report body. bee-swarming tells the leader to read the digest, to open a report only on a red, a question or a blocked status, and to start a diff review with the stat view. | 5c368ef5 |
| D3 | Every claude launch that gets `THIN_PEER_CLAUDE_ENV` also gets `SLASH_COMMAND_TOOL_CHAR_BUDGET=1`. Where bee builds the claude argument list itself (claude-session spawn, built-in claude herding pane), a thin worker launch also carries `--disable-slash-commands` and `--disallowedTools Agent`. Other launches are unchanged. | 6d55d3dd |
| D4 | The worker cell prompt carries a short search rule. bee-planning's cell rules say a cell action names each function it changes as path plus symbol. | b4521fc9 |
| D5 | `packages/bee/LEADER.md` shrinks from 21685 bytes to at most 15000. Every rule marker id stays with its boundary sentence. Moved detail lands in full in a playbook or reference that LEADER.md names with a read-when line. No rule is deleted or weakened. | 4c6e84ea |
| D6 | The fixed text of a cell brief shrinks by at least 25 percent in bytes, measured on one rendered non-read-only cell brief for the same cell before and after. Every result field and status the parser accepts is still named. | acb2c2d8 |
| C1 | Test contract thin-worker-listings-off. | 1dd8b534 |
| C2 | Test contract handoff-after-merge-door. | 8cceb386 |
| C3 | Test contract herding-result-job-digest. | 7e81e0f2 |
| C4 | Test contract cell-brief-search-rule-and-size. | 5737bdba |

### Agent's Discretion

Where the merging session id is stored for D1, which text leaves LEADER.md
for D5, and which fixed brief sentences go for D6, inside the contracts above.

## Measurements

Baseline for D3, `claude -p "Reply with the single word ok." --model haiku`
with the thin worker env, in this repo, two runs each, spread 0:

| Launch | First-call context (tokens) |
|---|---|
| thin worker today | 20739 |
| plus `SLASH_COMMAND_TOOL_CHAR_BUDGET=1` | 14803 |
| plus `--disable-slash-commands` | 13041 |
| plus `--disable-slash-commands --disallowedTools Agent` | 8868 |
| all three | 8887 |

With the two flags, the SessionStart, PreToolUse and Stop hooks still fired.
Decision threshold: a switch is kept when it cuts at least 2000 tokens.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | home of the token rules; gains this feature's rules |
| docs/knowledge/areas/hook-runtime/hook-runtime-the-worker-outward-guard.md | suspect | read-only shell rules that D4 mirrors for cells |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/herding/run.rs` | `THIN_PEER_CLAUDE_ENV`, `thin_peer_env`, launch paths |
| `packages/bee-rs/crates/bee/src/herding/claude_session.rs` | `spawn_argv`, `settings_json` |
| `packages/bee-rs/crates/bee/src/herding/wave.rs` | built-in claude pane argv, `resolve_agent_command` |
| `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs` | `merge_text_lines`, merge session id |
| `packages/bee-rs/crates/bee/src/verbs/feature_group.rs` | `run_start` |
| `packages/bee-rs/crates/bee/src/herding.rs` | `envelope_result`, `result_reader` |
| `packages/bee-rs/crates/bee/src/herding/mailbox.rs` | `render_brief`, `parse_result_text`, `MailboxResult` |
| `packages/bee/prompts/worker-cell.md` | the cell prompt text |
| `packages/bee/LEADER.md` | source of `.bee/LEADER.md` |

## Out of scope

- An argument channel through paseo. `paseo run` passes only `--env` to the
  agent, so a paseo worker gets the env switch (14803) and keeps the agent
  listing.
- The leader's thinking share and its private memory files.
- A new measurement of `dispatch wave --rolling --run`; the next real feature
  run gives it.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
