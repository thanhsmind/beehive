# Paseo leverage — Context

**Feature slug:** paseo-leverage
**Date:** 2026-10-12
**Shaping session:** complete (gate bypass full; the user asked for all eight items: "làm toàn bộ")
**Scope:** High-risk
**Domain types:** CALL | RUN

## Feature Boundary

The user switched the codex roles to Paseo + pi + Antigravity (decision
`b8b40d64`) and asked what more bee could take from Paseo. A read of the three
slp reference repos (`~/Projects/refs/slp/paseo-pi-team`,
`paseo-doctrine-downstream`, `pi-peer`) found eight items. This feature builds
them on stock Paseo 0.11.2 (the installed version). Features that exist only in
the patched Paseo of `paseo-doctrine-downstream` (`notifyOnFinish`,
`autoArchive`, `paseo agent signal`) are out of scope. Herdr and tmux transports
do not change.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A running herded worker can send its leader a typed mid-flight message with a new verb `bee herding notify --kind <question\|blocked\|dependency\|progress\|reopen> --text <text> [--correlation-id <id>]`. The verb works only inside a herded worker (`BEE_HERDING_JOB_ID` set) and writes only into that worker's own job mailbox, as one line appended to `.bee/mailbox/<job>/messages.jsonl` carrying job id, cell id, kind, text, correlation id and time. Delivery to the leader goes through the existing paths, never through `paseo send`: the Pi leader's result-inbox drain injects new messages (steer when busy, a new turn when idle), and `bee herding status` lists unread messages for every leader. A message never ends the worker's turn and never replaces the `question` outcome: an answer still reaches the worker as the next round. | `paseo send` to a busy agent interrupts and replaces its turn (the-paseo-channel, "Answers and steering"), so it is unsafe toward a working leader. Today a worker can only speak at turn end (gather E). |
| D2 | Revised after the hat wave (store `168b8037`, superseding `b0576702`). `bee herding run` takes `--read-only`; `bee dispatch prepare` passes it for kinds `gather` and `advisor` only (reviewers run tests and stay as today), and `job.json` records `read_only: true`. A Paseo `pi` worker gets `BEE_HERDING_READ_ONLY=1`, and `worker-guard` enforces it on every tool: writes are allowed only inside the worker's own job mailbox directory; shell commands follow a mode-B-shaped allowlist that fails closed on truncation, command substitution, a redirect outside the mailbox, and per-tool exec or output flags. Any other Paseo provider refuses a read-only run before spawn with a FIX line. Herdr and tmux runs record `read_only`, and status shows it as not enforced. No Paseo mode is set. | Claude plan mode and a total write ban both stop the worker from writing its ack and result files; codex and antigravity have no guard belt under Paseo. |
| D3 | A Paseo worker in a permission-asking state is reported, never changed. After spawn the run records the inspect `Mode` as `paseo_mode_observed` in `job.json`. `bee herding status` marks a Paseo worker `parked` when it is `Blocked` on a pending permission for longer than the shared 120 s freshness constant, and names the mode and the pending tool. bee never runs `paseo agent mode` on its own. | Gives a cause for a worker that waits forever; the observer decides nothing (supervisor doctrine). |
| D4 | No code change. A failed or unparseable `paseo inspect` is `Unknown`, resets the died debounce and never counts toward `died` — this is already true (`run.rs:1279-1291`, test near `run.rs:9941`). The feature records this as verified and the Paseo channel concept states it in one line. | Gather A found the rule already shipped. |
| D5 | When a Paseo round ends, the run writes `usage` into `job.json`: `{input, output, cache_read, cache_write, total_tokens, cost_usd, source}`. Source order: the inspect `LastUsage` when it is not null; else, for provider `pi`, the sum of `usage` entries in the session file named by the agent's on-disk `persistence.nativeHandle`. When neither exists, `usage` is `null` and `usage_unavailable` names why — never zero. `bee herding status` shows tokens and cost per job. | `LastUsage` is null for pi agents (observed 2026-10-12); the pi session file carries per-turn `usage.cost`. |
| D6 | A new verb `bee herding fork --job <id> \| --agent <paseo-id>` makes a successor Paseo agent that keeps the source agent's history. For provider `pi`: copy the session file named by `persistence.nativeHandle` to a new file whose first line is a new session header (new UUIDv7 id, new timestamp, `parentSession` = source path), written to a temp file and renamed, then `paseo import <new id> --provider pi --cwd <source cwd> --label bee_fork_of=<source id>`. Other providers refuse with a FIX line. The verb never archives or stops the source. Only a human or the leader runs it; the supervisor's `dead-lead` observation names the fork command as the resume line when the dead lead is a Paseo agent. | slp-lead-recovery D1 ("no machine spawn, ever") and D4 ("the old lead is never killed, closed, or archived by anything but the human") stand. Mechanism from `paseo-pi-team/scripts/team-fork.mjs`. |
| D7 | Paseo's on-disk agent state (`<paseo home>/agents/<project>/<id>.json`) is a read source for views that need many agents or data the CLI lacks: `bee herding status`, the untracked-agent report, D5 usage and D6 fork. The run loop keeps the CLI (`inspect`, `wait`) as the liveness authority. A missing or unreadable file falls back to the CLI. Startup speed is a separate check: the plan measures spawn-to-ack for `paseo-pi-agy-flash` with and without `isolated_config: true`. | Measured 2026-10-12: `paseo --version` 349 ms, `inspect` 493 ms, `ls` 528 ms; the run makes three CLI calls at startup (about 1.5 s). The 7–14 s slower start seen in the 8-worker eval is Pi boot, not bee. |
| D8 | A live contract test, `#[ignore]` by default, checks the installed Paseo against what bee parses: the `inspect --json` keys (`Id`, `Status`, `UpdatedAt`, `PendingPermissions`, `ParentAgentId`, `Model`, `Thinking`, `Mode`, `LastUsage`), the `ls --json` shape, and the on-disk agent file keys bee reads (`id`, `labels`, `lastStatus`, `runtimeInfo`, `persistence.nativeHandle`). It skips with a message when the daemon does not answer. It runs with `cargo test -- --ignored paseo_live`. | Paseo moved from 0.10.3 to 0.11.2; nothing today catches a schema drift. |

### Hat-wave refinements (store `6fc537f6`)

- D1: "unread" means lines past a per-job `messages.cursor` in the job mailbox. The Pi drain delivers only for jobs run with `--inbox-session`, fences worker text as data through `headerValue`, and advances the cursor only when the send reports steer or turn. The brief allows `bee herding notify` as the one bee command a worker may run.
- D6: fork refuses a `Working` source unless `--force`, refuses a session file outside the default pi sessions root, and prints the new id, the source, the undo command, and that the model is the provider default.
- D7: the Paseo home comes from `--home` in `herding.paseo.command`, else `PASEO_HOME`, else `~/.paseo`.

### Environment facts

- Paseo 0.11.2. `paseo provider ls --json`: claude modes include Plan Mode;
  codex lists Default Permissions, Auto-review, Full Access; pi lists none;
  antigravity lists Full access.
- `paseo import <id> --provider --cwd --label` exists; the imported agent's model
  cannot be set from the CLI (`team-fork.mjs` header).
- On-disk agent file fields observed: `id`, `provider`, `cwd`, `labels`,
  `lastStatus`, `config`, `runtimeInfo.{model,thinkingOptionId,sessionId}`,
  `persistence.nativeHandle`, `archivedAt`.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Mid-flight message | A `bee herding notify` line from a worker that is still working. |
| Parked | A Paseo worker `Blocked` on a pending permission past 120 s. |
| Fork | A new Paseo agent imported from a copy of another agent's session file. |

## Out of scope

- Daemon push on finish, auto-archive, coordination signal (patched Paseo only).
- Scope leases (bee reservations cover it), multi-host `--host`, the Paseo MCP patch.
- Killing orphaned child processes of a killed pi worker (a pi/Paseo defect).

## Sources

- `docs/history/research/paseo-pi-team-human-elevation.md`
- `docs/history/slp-lead-recovery/CONTEXT.md`
- `docs/knowledge/areas/bee-herding/the-paseo-channel.md`
- `~/Projects/refs/slp/paseo-pi-team/scripts/team-fork.mjs`, `scripts/watchdog.mjs`, `scripts/team-communication.mjs`
