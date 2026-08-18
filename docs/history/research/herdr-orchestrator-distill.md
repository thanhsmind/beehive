---
artifact_contract: bee-research/v1
topic: herdr-orchestrator-distill
depth: standard
date: 2026-08-18
---

## Bottom Line

> **Blocking finding, found while researching: bee-herding's own spawn command is broken against the installed herdr.** The dispatch role's §8 spawn is
> `herdr agent start <slug> --cwd <worktree> --workspace <ws> --tab <runtime-tab> --split right --no-focus -- claude …`
> (`skills/bee-herding/references/role-dispatch.md:261-311`). On herdr 0.8.0 that returns `unknown option: --cwd`.
> In 0.8.0 `agent start` takes `<NAME> --kind <KIND> --pane <ID>` and *"never creates, splits, or moves layout."*
> **No dispatch iteration in this repo can spawn a working agent today.** Fix this before any orchestrator work — see Recommendation → Fix first.

- **Recommendation (ladder rung): built-in (rung 2) for the mechanics + adapt (rung 3) for the discipline.** Do NOT port the source's code.
- **Why this is the lightest credible path:** the installed `herdr 0.8.0` already ships, as native CLI verbs, the four things the source skill spends ~1,500 lines of Python building — readiness-gated agent start, atomic prompt-and-wait, lifecycle wait, and output-match wait. What the source still contributes is *discipline* (fail-closed status gating, a completion proof for `unknown`-status agents, concurrent fan-out with per-target failure buckets, orchestrator context succession) — prose, not code.
- **Why the next-best rung lost:** a full port (rung 3 as copy) is rejected on evidence. The source's central wait primitive is `herdr wait agent-status <pane> --status … --timeout …`; that command **does not exist in herdr 0.8.0** — `herdr wait` falls through to the top-level help. Its scripts are explicitly pinned to "herdr 0.7.4 semantics". Porting them imports a broken fast path and a maintenance burden bee does not need.
- **Confidence: 85%.** The herdr surface is proven against the running binary. The unproven part is behavioral: nobody has yet run a bee pane-fleet round trip end to end.
- **Suggested next step:** `bee-shaping` — the scope question below (which of bee's flows move onto panes) is a product decision, not a research one.

## Repo Snapshot

- Repo type: Rust CLI + markdown skill system. `packages/bee-rs/crates/bee` is the binary; `skills/*/SKILL.md` are the agent-facing contracts. `bee` 2.11.0. (`Local`)
- Terminal layer: `herdr 0.8.0`, protocol 19, server running at `~/.config/herdr/herdr.sock`. This session is itself a herdr-managed pane — `w4:p1`, workspace `w4` (`beehive`), `agent_status: working`. (`Local`)
- Relevant existing subsystems: `bee-herding` (the three-role cockpit), `bee-swarming` (Task-tool worker fan-out), `bee-reviewing` (multi-agent review panel), `bee-capturing` (compounding), `worktree-parallelism` (isolation + merge gate). (`Local`)
- Constraint that shapes the answer: bee's parallel-agent story already exists — but in a *different plane* (see Findings → Local).

## Question & Assumptions

- **What was asked:** distill `luongnv89-skill/skills/herdr-agent-comms`, and judge how to lift `bee-herding` from a spawn-and-merge cockpit into a real orchestrator that drives herdr — opening parallel agents for review, or for compounding.
- **What success appears to mean:** a bee session can open N live agents in herdr panes, hand each a task, wait on them concurrently, read their answers back, and fold those answers into bee state — with the same fail-closed safety bee already demands.
- **Assumptions still needing confirmation:**
  - That pane-hosted agents are wanted *in addition to*, not *instead of*, Task-tool subagents. (See the two-planes finding — this is the real scope decision.)
  - That the four-slot concurrency cap still applies when panes hold conversational agents rather than unattended workers.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/home/thanhsmind/projects/AI/luongnv89-skill` (`git@github.com:luongnv89/skills.git`) |
| Ref | `main` |
| Resolved commit SHA | `48730b30da90dfd2d2e3fa77a93c657cf75c4448` (2026-08-10, "feat(skills): add orchestrator context gate with HANDOFF succession (#85)") |
| Narrowed scope | `skills/herdr-agent-comms/` — SKILL.md, 3 references, 4 scripts, evals. 4,534 lines total. |

Mode: **`xia`** — distill and discuss. Steps 1–4 of the port protocol; no challenge pass, nothing built.

## Findings

### Local — bee has two agent planes, and only one of them talks

**Plane A — the Task-tool plane (mature).**
- `bee dispatch wave` is a real parallel fan-out: it resolves a feature, computes disjoint-file waves via `compute_schedule`, then per cell actually claims + reserves + registers + prepares, unwinding one cell's refusal into a typed `skipped` reason instead of aborting the batch (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1411-1642`). The orchestrator still spawns the model workers itself. (`Local`)
- Cap is 3–4 live workers over disjoint files; serial needs a named reason (`skills/bee-hive/references/gates-and-delegation.md:124-129`). (`Local`)
- Cross-session work-stealing exists: `bee cells claim-next` sweeps expired claims, then walks other live sessions' lanes for unheld ready cells (`packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:658-832`). (`Local`)
- Review already runs a parallel panel: "spawn the core four in parallel — `code-quality`, `architecture`, `security`, `test-coverage` — plus any conditional reviewer whose trigger the diff matches, capped at six" (`skills/bee-reviewing/SKILL.md:39-41`). The Rust side freezes the scope and records findings; it never launches the agents (`packages/bee-rs/crates/bee/src/verbs/reviews.rs:890-1058`). (`Local`)
- Compounding is a stamped close-out, not a fan-out: `bee state scribing-run` / `bee state compounding-run` write a lane-scoped ledger row (`packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:418-618`). (`Local`)

**Plane B — the pane plane (`bee-herding`) is write-only.**
- The three roles are bootstrap / dispatch / merge (`skills/bee-herding/SKILL.md`). Dispatch **only starts** work; merge **only retires** it. Neither converses with a running agent. (`Local`)
- Grepping every herdr call in `skills/bee-herding` and `scripts/`: `pane rename, list, split, close, send-text, current, layout, read, get, run`; `agent start`; `tab create, list`; `workspace list`. **Absent: `agent prompt`, `agent wait`, `agent read`, `agent list`, `pane wait-output`.** (`Local`)
- The merge role's enumerated surface is documented as `herdr pane current/rename/layout/list/send-text/close` (`skills/bee-herding/scripts/control-loop.sh:309`) — no read verb at all.
- Only `bootstrap-cockpit.sh` calls herdr from a script (`tab create`, `pane split`, `pane list`, `pane run` — all still valid on 0.8.0). Every other herdr call in the system is issued *live by a control-pane agent* following the role markdown; `control-loop.sh` itself only spawns `claude`. There is no herdr-driving Rust or Node module anywhere — `bee herding herdr-result` / `herdr-pane-id` only parse JSON piped in on stdin (`packages/bee-rs/crates/bee/src/herding.rs:565-656`). (`Local`)
- Communication back to the human is one-way too: refusals and announcements go out through `herdr pane send-text <chat_pane_id> "…"`, and the only read is a dedup scan of the chat pane's own scrollback (`role-dispatch.md:131`). No agent is ever read. (`Local`)
- Permission is *not* the blocker: both control roles are granted `Bash(herdr:*)`, the whole binary (`skills/bee-herding/scripts/control-loop.sh:306,318`). (`Local`)
- The knowledge area already records the consequence as an Open Gap: "The four-slot concurrency cap is not yet mechanical… a spawned agent that fails to self-name can lead the loop to over-spawn", and "The dependency on herdr's JSON shapes is unpinned — no capability probe, so an upstream shape change degrades to the silent-stall class" (`docs/knowledge/areas/bee-herding/overview.md`, Open Gaps). (`Local`)

**So the gap is exact:** bee can *open* pane agents and *close* them, but cannot ask one a question and receive the answer. Every "parallel review agents in panes" or "compound in panes" idea dies on that one missing round trip.

### Upstream — what the source skill actually is

`herdr-agent-comms` v1.23.0 is a seven-phase fleet protocol: resolve root → spawn an equal-width grid → resolve one exact target → send safely → wait/read/verify → broadcast/steer/tear down → hand off the orchestrator role at a context threshold.

**Its genuine inventions** (`Upstream`):
1. **Baseline + split completion marker.** Snapshot the transcript before sending; embed a `HERDR_DONE_<suffix>` marker split across two fragments in the task text, so only a fully generated reply reproduces the joined string. This defeats two races at once — prompt echo mistaken for activity, and an agent that finishes before the waiter even starts (`references/delivery-and-waiting.md:37-44,82`).
2. **Two-phase preflight.** Check safety before the first mutation *and again* immediately before the submitting Enter, because a pane can flip to `blocked` in between (`references/herdr-recipes.md:282-293`).
3. **Fail-closed status classification.** A distinct `LOOKUP_FAILED` sentinel, kept separate from a valid `unknown`, so a failed lookup can never fail open into "safe" (`scripts/wait_for_idle.py:104-134`).
4. **Concurrent fan-out with typed failure buckets.** `broadcast.sh` resolves + dedupes targets, preflight-rejects, baselines every target *before any send*, re-checks each target immediately before its own dispatch, then backgrounds one waiter per pane and aggregates five distinct buckets — `busy`, `blocked`, `unverifiable`, `send_failed`, `became_unsafe`. Any non-empty bucket fails the whole broadcast. Stated rationale: serial send→wait→read makes total time the *sum* of agents; concurrent waits make it the *max* (`scripts/broadcast.sh:123-292`; `references/delivery-and-waiting.md:356`).
5. **Orchestrator context succession (HANDOFF).** Self-check context only at three named checkpoints — before a spawn wave, before a broadcast, after a relayed reply — never mid-cycle between a dispatch and its wait. Threshold default 50%. When self-introspection returns UNKNOWN, fall back to deterministic counters (20 relayed reads or 4 spawn waves). Anti-thrash: must finish one full operation before handing off. The successor gets a compact brief (never transcripts or diffs), must reply an exact ack string `HANDOFF ACCEPTED gen=<N> fleet=<k>`, and only then does the outgoing pane go read-only — it is never closed (`references/context-succession.md:9-88`).
6. **Grid equalizer.** `next_grid_split.py --equalize` sweeps internal boundaries, resizing until all columns are within one cell, capped at 12 passes, and treats non-convergence as a hard error that aborts the spawn rather than proceeding onto a broken layout (`scripts/next_grid_split.py:408-486`).

**Its weaknesses** (`Upstream` + `Local`):
- **Version rot.** The scripts are pinned to herdr 0.7.4 (`scripts/next_grid_split.py:5-26`). Its primary wait path calls `herdr wait agent-status` — verified absent from 0.8.0. Its spawn recipe calls `herdr agent start <name> --tab <tab> --split right`; 0.8.0's `agent start` requires `--pane` and explicitly "never creates, splits, or moves layout".
- **Reinvention.** `wait_for_idle.py` is 466 lines of polling that `herdr agent prompt --wait` now does in one flag.
- **No state layer.** Results are relayed as transcript deltas into the orchestrator's chat. Nothing is recorded, claimed, reserved, or proven. There is no cell, no decision log, no proof line.
- **No isolation model.** Every worker shares the root pane's `project_dir`. Concurrent writers to one checkout is exactly what bee's worktree isolation exists to prevent.
- **Layout is load-bearing.** A failed resize aborts a spawn. bee should not let cosmetics gate work.

### Docs — herdr 0.8.0 is the authority, and it moved

`herdr --skill` prints an official 195-line agent skill from the installed binary. Verified against `--help` on the live server (`Local`, and `Docs` for the semantics):

| Need | herdr 0.8.0 native verb |
|---|---|
| Start an agent, gated on readiness | `agent start <NAME> --kind <KIND> --pane <ID> [--timeout MS=30000]` — returns only after the agent is detected and ready. 21 kinds incl. `claude`, `codex`, `opencode`. |
| Send a task and wait for it to settle | `agent prompt <TARGET> <TEXT> --wait [--until …] [--timeout MS]` — atomic submit + Enter honoring bracketed paste; a non-working start that shows no lifecycle change within 5000 ms returns `agent_prompt_stalled` rather than hanging. |
| Wait on a specific state | `agent wait <TARGET> --until idle\|working\|blocked\|done\|unknown [--timeout MS]` |
| Wait for specific output | `pane wait-output <PANE_ID> (--match TEXT\|--regex PATTERN) [--timeout MS]` — searches the existing snapshot first, then polls. |
| Read a transcript | `agent read <TARGET> --source recent-unwrapped --lines N` |
| Create an isolated worktree workspace | `worktree create [--branch NAME] [--base REF] [--path P] [--label TEXT] [--no-focus]` |

Status enum, stated precisely by the binary's own skill: `idle` = ready **and** its tab was seen in the focused UI; `done` = the same idle state after **unseen** background work finished; `blocked` = an approval/question UI was recognized; `unknown` = an agent is present but unclassifiable and **is not proof of completion**. CLI reads never mark a tab seen; focus commands do. Error contract: server errors are JSON on stderr with exit 1, syntax errors exit 2. (`Docs`)

One more thing the binary already tracks: `herdr agent list` returns `agent_session.value` — for this pane, the exact Claude Code session UUID. A pane and a bee session record can be correlated without inventing a protocol. (`Local`)

### Inference

- The source skill's Python is a **shim for a herdr that no longer exists**. Its value has migrated into the binary; what remains uniquely its own is the prose discipline and the succession protocol. (`Inference` from the version-rot evidence above.)
- Only invention #1 (baseline + split marker) still has a live purpose in 0.8.0, and only for `unknown`-status agents — where `agent_status` cannot classify, marker text in the transcript is the one available completion proof. For a recognized kind, `agent prompt --wait` is strictly better. (`Inference`)
- The four-slot cap being enforced "by the control model counting panes, not by code" (a recorded Open Gap) becomes mechanically fixable the moment bee reads `herdr agent list` and matches `agent_session.value` against its own session records. (`Inference`)

## Dependency Matrix

| Source component | Maps to in bee | Verdict | Evidence |
|---|---|---|---|
| Phase 1 root/context resolution | `bee herding herdr-pane-id`, `herdr-result` + `HERDR_*` env | `EXISTS` | `Local` |
| Phase 2 spawn + readiness gate | `bee-herding` dispatch §8 spawn | `CONFLICT` — **both** the source recipe and bee's own use the removed 0.7.4 `agent start` flags; both must move to split-then-`--kind`/`--pane` | `Local` |
| `next_grid_split.py` equalizer | nothing; `pane split --ratio` + `pane resize --amount` still exist in 0.8.0 | `NEW` (optional, cosmetic) | `Local` |
| Phase 3 fail-closed target resolution | nothing — bee resolves panes by label only | `NEW` | `Local` |
| `preflight_send.py` (status gate before write) | nothing | `NEW` | `Local` |
| Phase 4 send + delivery verification | superseded by `agent prompt --wait` | `CONFLICT` (source path calls a removed verb) | `Local` |
| `wait_for_idle.py` (466 lines) | superseded by `agent wait` / `prompt --wait`; marker path still useful for `unknown` | `CONFLICT` | `Local` |
| baseline + split completion marker | nothing | `NEW` (keep — the one script idea worth carrying) | `Upstream` |
| `broadcast.sh` fan-out + failure buckets | `bee dispatch wave` does the equivalent in the Task-tool plane; nothing in the pane plane | `NEW` for panes, `EXISTS` in concept | `Local` |
| Phase 6 teardown with confirmation | `bee-herding` merge role closes panes after merge | `EXISTS` | `Local` |
| Phase 7 HANDOFF succession | `bee state handoff write/adopt` — but session-level, not pane-level; no successor spawn, no ack protocol | `NEW` | `Local` |
| Context gate at named checkpoints | AGENTS.md "at roughly 65% context, write `.bee/HANDOFF.json` and pause" — a threshold, no checkpoint discipline, no UNKNOWN fallback | `CONFLICT` (bee pauses; source migrates the role) | `Local` |
| Report / acceptance-criteria block | bee's progress-tick + Communication contract | `EXISTS` | `Local` |
| Per-worker isolation | bee worktrees; source shares one `project_dir` | `CONFLICT` (bee's model is stronger — keep bee's) | `Local` |
| Result recording | bee cells, decisions, proof lines; source records nothing | `EXISTS` (bee wins outright) | `Local` |

## Cross-Cutting Sweep

Wiring outside the skill folder that any pane-orchestrator change would touch:

- **Permission surface** — `allowed_tools_for()` in `skills/bee-herding/scripts/control-loop.sh:293-320`. Already `Bash(herdr:*)`; new herdr verbs need no widening. The comment block naming the exact verbs does drift and would need updating. (`Local`)
- **Runtime adapter** — `.bee/config.json` `herding.agent_command` / `herding.control_command`, argv-token arrays substituted per token, never `eval` (`skills/bee-herding/SKILL.md`, "Runtime adapter"; `control-loop.sh:280-286`). A change to how agents are launched (`pane run claude …` → `agent start --kind claude`) lands squarely on this seam. (`Local`)
- **Knowledge area** — `docs/knowledge/areas/bee-herding/overview.md` is `authoritative_for` the cockpit; its R1–R8 business rules, Open Gaps, and Pointers all move. (`Local`)
- **Locked decisions** — herding-adopt D1/D7/D10/D11/D12, herding-dispatch-lock-toggle D1–D5, i54-closeout D4. **Nothing in this brief contradicts them**: merge stays a human gesture (D11/R2), dispatch stays interlocked (D10/R3), the posture split stays (D7/R4). A pane-orchestrator adds a *fourth* capability alongside the three roles; it does not relax any of them. (`Local`)
- **Current binary state** — `bee herding enable/disable/status` are **not built** into the Rust binary and refuse by name; only `classify-lane`, `interlock`, `command-template`, `herdr-result`, `herdr-pane-id` are live (`.bee/bin/bee herding --help`). Any design that assumes the CLI switch exists is already wrong. (`Local`)
- **Not swept, therefore unchecked:** the hook manifests (`packages/bee/hooks/`), the skill render/ledger pipeline (`bee dev regen`), and the plugin distribution manifests. A new skill or reference file passes through all three.

## Recommendation

### Fix first — the spawn line

Verified against the running binary: `herdr agent start testslug --cwd /tmp --workspace w4 --tab w4:t1 --split right --no-focus -- claude --model sonnet` → `unknown option: --cwd`. Nothing was mutated; the parse fails before any action. (`Local`)

On 0.8.0 the spawn becomes two steps, because `agent start` can no longer make its own pane:

```bash
# 1. make the pane, in the worktree, without stealing focus
herdr pane split <runtime-pane-id> --direction right --cwd <worktree_path> --no-focus
#    → parse .result.pane.pane_id

# 2. start the agent in that pane, gated on readiness
herdr agent start <slug> --kind claude --pane <new-pane-id> --timeout 60000 \
  -- --model sonnet --permission-mode bypassPermissions "<opening instruction>"
```

This inverts a recorded proof: `references/spawn-proof.md` warns that splitting first "leaks a stray pane" and tells the role never to do it. Under 0.8.0 splitting first is the only way. That proof needs re-recording, not just the command. The `--no-focus` / `--ratio` / `--cwd` flags on `pane split`, and every call in `bootstrap-cockpit.sh`, are still valid — the break is confined to `agent start`. (`Local`)

Two smaller drifts found in the same sweep, worth folding into the same pass:

- `docs/knowledge/areas/bee-herding/overview.md` "Pointers" still cites the pre-Rust Node files (`scripts/dispatch-interlock.mjs`, `scripts/classify-lane.mjs`, `packages/bee/lib/herding.mjs`, `packages/bee/tests/test_herding_cli.mjs`). None exist any more; the live implementation is `packages/bee-rs/crates/bee/src/herding.rs`. (`Local`)
- This repo's `.bee/config.json` has `"gate_bypass": "normal"` and no `herding` key. The bootstrap pre-flight requires `full` or `total`, so dispatch would refuse every cycle here even with a working spawn line. Not a defect — just the reason a live test needs the owner to raise it first. (`Local`)

### Then — three things to take, in descending value:

1. **Give bee-herding a read path — a fourth capability, not a fourth role.** One round trip, built on native verbs: `agent start --kind <kind> --pane <id> --timeout` → `agent prompt <name> "<task>" --wait --timeout` → `agent read <name> --source recent-unwrapped --lines N`. This is the whole unlock. Everything else in this brief is refinement.
2. **Take the discipline, not the scripts.** Fail-closed status gating (`unknown` is never proof; a lookup failure is never "safe"), the re-check immediately before dispatch, the baseline + split marker as the *fallback* completion proof for `unknown`-status agents, and the typed failure buckets on a fan-out. Roughly 40 lines of prose in a reference file. Zero lines of Python.
3. **Take the succession protocol, adapted.** bee's current answer at high context is "pause and write a handoff" — which stalls a live fleet. The source's answer is "migrate the role to a fresh successor and go read-only, never close." For an orchestrator holding N live panes, migration beats pausing. The checkpoint discipline (never mid-cycle), the UNKNOWN counter fallback, and the required ack string are all worth carrying.

Three things to leave:

- **The Python.** All four scripts. The equalizer is the only one that still runs against 0.8.0, and it gates work on cosmetics.
- **The shared `project_dir`.** bee's worktree isolation is the stronger model; `herdr worktree create --branch --base` now makes the isolated variant native.
- **The seven-phase shape.** bee already has lanes, gates, cells, and proof. Layering a parallel phase vocabulary on top would be a second workflow grammar.

**The scope decision this brief cannot make for you** — which flows move onto panes:

- *Review on panes:* the panel is 4–6 read-only agents that already run fine as Task-tool subagents (cheap, no server dependency, results land in-session). Panes buy visibility and steerability, and separate context windows; they cost a herdr dependency on a flow that currently has none, and findings must be relayed back by hand. Weak case.
- *Compounding on panes:* compounding is synthesis — "decide-altitude never delegates" (AGENTS.md). Fan-out here is the reading, which is already delegated. Weakest case.
- *Long-running feature agents on panes:* this is where panes genuinely win — an agent that runs for an hour in its own worktree, that a human can watch, steer, and unblock, and whose completion the orchestrator can *wait on* instead of counting panes. This is also exactly what `bee-herding` already spawns and currently cannot talk to. **Strongest case, and it is the one that closes a recorded Open Gap.**

## Risks, Unknowns, Follow-Ups

- **Herdr JSON shapes stay unpinned.** The recorded Open Gap ("no capability probe, degrades to the silent-stall class") gets worse as bee depends on more verbs. A version/capability probe should ship with any adoption. (`Local`)
- **`agent prompt --wait` does not track turns.** The binary states it plainly: "if the agent is already working, that active turn's completion may match." A naive `prompt --wait` on a busy agent can return on the *wrong* turn — this is precisely why the source's marker discipline still earns its place. (`Docs`)
- **The alternate-screen read failure.** `--lines` cannot recover rows that left the alternate screen; the binary's own fallback is "ask the agent to write its response to a file and reply with the path". Any relay design needs that fallback path. (`Docs`)
- **`idle` vs `done` depends on UI focus**, not on work. An orchestrator that focuses panes changes the state it is observing. (`Docs`)
- **The spawn break proves the unpinned-shape gap is already live, not hypothetical.** The recorded Open Gap said an upstream shape change "degrades to the silent-stall class"; it did, and nobody noticed because no dispatch cycle has run here since. A capability probe at bootstrap — assert the verbs and flags the roles depend on, refuse loudly otherwise — is worth more than any new feature in this brief. (`Local` + `Inference`)
- **Nothing here has been run end to end.** The herdr claims come from `--help`, one live `agent list`, and one deliberate parse-failure probe; no bee pane round trip has been executed. That is the first proof any shaping should demand. (`Inference`)

## Source Pack

- Local files read: `skills/bee-herding/{SKILL.md,README.md}`; `skills/bee-herding/references/{role-bootstrap,role-dispatch,role-merge,operational-invariants,spawn-proof,dispatch-dry-run,dispatch-prompt,merge-prompt}.md`; `skills/bee-herding/scripts/{control-loop.sh,bootstrap-cockpit.sh}`; `packages/bee-rs/crates/bee/src/{herding,router,catalog}.rs`; `docs/knowledge/areas/bee-herding/overview.md`; `skills/bee-hive/references/gates-and-delegation.md`; `skills/bee-swarming/SKILL.md`; `skills/bee-reviewing/SKILL.md`; `skills/bee-capturing/SKILL.md`; `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`; `.../verbs/state_group/{workers,sessions}.rs`; `.../verbs/cells/{handlers_select,schedule}.rs`; `.../verbs/reviews.rs`; `.bee/config.json`; `.claude-plugin/plugin.json`.
- Upstream read at `48730b30da90dfd2d2e3fa77a93c657cf75c4448`: `skills/herdr-agent-comms/{SKILL.md,evals/evals.json}`, `references/{herdr-recipes,delivery-and-waiting,context-succession}.md`, `scripts/{broadcast.sh,next_grid_split.py,wait_for_idle.py,preflight_send.py}`.
- Docs: `herdr --skill` (195 lines, herdr 0.8.0); `herdr --help` and `herdr {agent,pane,tab,workspace,worktree,notification} --help`; `herdr {agent start,agent prompt,agent wait,agent read,pane split,pane run,pane resize,pane layout,pane wait-output,worktree create} --help`; live `herdr status`, `herdr agent list`, `herdr workspace list`.
