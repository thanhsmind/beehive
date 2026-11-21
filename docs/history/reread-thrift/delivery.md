# reread-thrift — delivery

## What shipped, per decision

- D1 handoff door. A successful `bee worktree merge` stores the merging session and feature. `bee feature start` refuses a second feature in that session with `handoff_due`. `--same-session "<reason>"` passes and writes a `handoff-waived` audit row. A caller with no session id is not refused. The tester found two faults in the first cut, so rrt-2 was reworked (commit a85c938ac): `handoff-waived` is now a registered audit kind, its row is written after the worktree and lane exist, and a failed merger record write warns on stderr.
- D2 result digest. `bee herding result --job <id>` prints at most 1500 characters of the job's result file, with `--json` and typed refusals. It never prints the report body. bee-swarming tells the leader to read the digest first.
- D3 thin worker switches. Thin claude launches get `SLASH_COMMAND_TOOL_CHAR_BUDGET=1`. Claude-session and pane launches also get `--disable-slash-commands` and `--disallowedTools Agent`.
- D4 search rule. The worker cell prompt carries a `Search rule:` line. bee-planning's cell rules say an action names each function as path plus symbol.
- D5 leader doctrine. `packages/bee/LEADER.md` is cut. The full text of each condensed block sits in `packages/bee/playbooks/leader-detail.md`. All 14 rule markers stay.
- D6 brief size. The fixed text of a cell brief is cut by 25 percent. Every result field and status the parser accepts is still named.
- D7 fix-first (cell rrt-9). The flag count pin in `catalog.rs` moved from 258 to 259, and the proof-line example with `cargo test -p bee` is back in the worker cell prompt.
- Tests (rrt-7). 15 contract tests in existing test modules, each red at its code cell's parent and green at HEAD. The judge report is `docs/history/reread-thrift/reports/rrt-7.md`.
- Doctrine and knowledge (rrt-6). The rules are in bee-swarming, the cell rules, the handoff playbook and the token-economy concept.

## Measured numbers

D3 first-call context, `claude -p "Reply with the single word ok." --model haiku` with the thin worker env, two runs each, spread 0:

| Launch | Tokens |
|---|---|
| Before | 20739 |
| Env switch only (paseo workers) | 14803 |
| Env switch and both flags (claude-session and pane workers) | 8887 |

The threshold was a cut of at least 2000 tokens per switch. All three switches pass it.

| Measure | Before | After |
|---|---|---|
| `LEADER.md` bytes (D5, limit 15000) | 21685 | 14915 |
| Fixed brief text bytes (D6) | 15227 | 11413 (25.05 percent cut) |
| Flag vocabulary pin (D7) | 258 | 259 |

The rrt-7 fixture measured the brief at 15164 and 11350 bytes. It leaves out path bytes, so it differs from the 15227 and 11413 figures. Its ratio matches.

## Out of scope

- An argument channel through paseo. A paseo worker keeps the agent listing and gets the env switch only.
- The leader's thinking share and its private memory files.
- A spend measure of the next real feature run.

## Proof

- Cell verifies passed for rrt-1 to rrt-9.
- `bee knowledge check` for this cell, see the cell trace.
