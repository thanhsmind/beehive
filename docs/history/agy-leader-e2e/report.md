# Spike agy-leader-e2e — agy as the bee leader runtime

<!-- bee:not-a-deferral: "deferred-queue" is the name of the feature the agy leader built, not a promise to act -->
Date: 2026-11-25. Item driven: PBI p-74ec14b3 (feature deferred-queue-req-flags).
<!-- /bee:not-a-deferral -->
Friction log of the turn-2 leader: `friction.md` beside this file.

## Runs

| Run | How | Result |
|---|---|---|
| Probe | `agy -p` with no tools, conversation 708da942 | session-init and prompt-context text reach agy |
| Turn 1 | `agy -p --dangerously-skip-permissions` on a leader brief, conversation f2e08a5f, 887 s | feature start, gate, plan, herding dispatch of dqrf-1 done; stopped by the owner's order to stop print mode |
| Turn 2 | interactive gemini-4-neon leader opened by `waggledance dispatch --leader --provider antigravity/gemini-4-neon` (run-4fb51d2042719085, paseo agent 966f5623) | capped dqrf-1, dispatched the tester dqrf-2, settled it, ran `bee test` (green on the second run), merged to main at 72bc1fa6d |

## What works

- W1. `SessionStart` injection reaches agy: the probe quoted `## bee v2.122.0` verbatim.
- W2. `PreInvocation` prompt-context reaches agy: the probe quoted `bee: phase=idle mode=standard`.
- W3. An agy leader walks the chain on its own: `bee feature start` made the worktree, it ran the plan-conflict check when the gate named it, it passed the gate with `--actor auto`, and it planned cells dqrf-1 and dqrf-2.
- W4. Dispatch from an agy leader through herding works. It ran `bee herding run` from the `dispatch prepare` payload. Herding job job-1795620567998-3027644-1 spawned agy-hydrogen through paseo in the feature worktree. The worker reported done at commit 93c3be6ad.
- W5. `invoke_subagent` stayed unused. The leader went through the herding door.
- W6. Cap with proof from an agy leader: dqrf-1 capped after the F11 and F1 workarounds.
- W7. A separate tester dispatched through herding: job job-1795622147472-3191999-1 proved red on the pre-dqrf-1 payload, then green; the leader settled it with `bee cells finish --from-job --proof-result green:red-first`.
- W8. `bee test` and `bee worktree merge` driven by an agy leader: merge commit 72bc1fa6d on main (F14 first, then green). It recorded leader checks when the merge named `WORKTREE_MERGE_LEADER_CHECK_DEBT`.

Not proven: the Stop hook `continue` path. The leader never ended a turn with an open step that the Stop check refused, so it did not fire.

## What breaks or rubs

| # | Finding | Evidence | Size |
|---|---|---|---|
| F1 | An agy leader's claim does not resolve to its own conversation. Started from a Claude Code shell, the claim went to the parent Claude session; started by paseo, the claim refused `SESSION_REQUIRED`. The command process carries `ANTIGRAVITY_CONVERSATION_ID=6cc848e6-…`, the id the leader then passed by hand and that holds dqrf-2. Cause: `session_identity::SESSION_ENV_VARS` listed BEE_SESSION_ID, CLAUDE_CODE_SESSION_ID, PI_SESSION_ID only, while `locate_caller_from` checks the agy name first. | turn 1: dqrf-1 held by session 090619f2; turn 2: friction.md line 7; `/proc/<pid>/environ` of the leader's command | belt defect |
| F2 | `dispatch prepare` from inside a feature worktree refuses; the leader first ran it there. | friction.md line 3 | friction |
| F3 | `dispatch prepare --kind cell` needs `--worker`; the leader did not know. | friction.md line 4 | friction |
| F4 | Claim refused `CONTRACT_UNCITED` until the cell cited a decision. | friction.md line 5 | expected gate |
| F5 | `pbi status --to in_progress` guessed a wrong enum value. | friction.md line 1 | model slip |
| F6 | `team.agy` (global) sends test, review, plan and read to Claude agents. An agent may not change it: `bee config set` refuses governed key team.agy.*. | config set refusal text | config |
| F7 | waggledance had no way to open a project leader on the antigravity runtime; only `agy -p` was left. | `waggledance dispatch --help`; `agent_presets = []` | missing feature |
| F8 | In print mode the agy turn waits on its own background tasks and cannot watch a herding job across turns. | turn 1 output: "root agent idle; waiting for 1 background task(s)" | print-mode limit |
| F9 | A live leader that opens a second, parallel leader with `waggledance dispatch --leader` gets it folded into a handoff of its own run family; one owner request then became two leaders on one PBI (p-3b770721), caught only because the second read recent commits. | dispatch reply for run-51b72285dbb15d71: "treated as a handoff of run run-b75f3a967082bd8d" | doctrine |
| F10 | The Stop hook refused a Claude leader's turn end that named an owner-only step (`cp .bee/team-only-agy.json .bee/team.json`): the guard refuses that write for the agent, yet the turn-end check reads it as a step the agent can run. | Stop hook text "the reply asks the owner to bạn chạy" | check defect |
| F11 | After the first agy leader was stopped, its herding job finished, and the cell was unclaimed, the next leader could not cap dqrf-1: "no registered execution worker: trace.worker \"unknown\"". | friction.md line 6 | belt defect |
| F12 | The tester job's auto-cap refused because the agy leader held the claim; the leader settled it with `bee cells finish --from-job`. | friction.md line 9 | friction |
| F13 | `bee herding run`'s settle hint prints `--proof-result green:unit`, but a test-role bugfix cell has the floor `green:red-first`, so the hinted command refuses. Hit again on agy-session-env ase-2. | friction.md line 10 | hint defect |
| F14 | The first `bee test` from the agy leader failed 1 of 6394 tests (`pi_lifecycle_end_to_end_onboarded_repo_parity`): the agy shell's ANTIGRAVITY_CONVERSATION_ID leaked into the test's child `bee`. Tests stripped BEE_SESSION_ID and CLAUDE_CODE_SESSION_ID from a child but not the agy name. | friction.md line 11 | test defect |
| F15 | Guard rubs on the agy side: `$SCRATCH` in a redirect target is refused, and agy's own scratch dir (~/.gemini/antigravity-cli/brain/…) is outside the worktree. | friction.md lines 13-14 | friction |

Side note on F7: waggledance (run-a78c018edd9e7167) added `waggledance dispatch --leader --provider antigravity/<model> --thinking <level>`, which turn 2 used. paseo model ids drop the effort suffix (`gemini-4-hydrogen`, thinking `medium`), and herding reports `model_mismatch` against the configured `gemini-4-hydrogen-medium`.

## Routed

| Finding | Route |
|---|---|
| F1, F14 | fixed in feature agy-session-env (PBI p-b454de67): session env order BEE_SESSION_ID, ANTIGRAVITY_CONVERSATION_ID, CLAUDE_CODE_SESSION_ID, PI_SESSION_ID; ten integration test files strip the agy name |
| F10 | PBI p-e8eeae96 |
| F11 | PBI p-3f1d7db8 |
| F9 | doctrine PBI p-a0f03c4a |
| F13 | PBI p-41a23d81 |
| F6 | PBI p-3b770721 (team profiles, only-agy, global apply), built by run-51b72285dbb15d71 |
| F7 | waggledance run-a78c018edd9e7167 |
| F2-F5, F8, F12, F15 | none: the refusal names its remedy, or the cause is a model slip or print mode |
