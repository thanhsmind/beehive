# CREATION-LOG — bee-swarming

## Provenance

Adapted from `khuym:swarming` (SKILL.md v1.0 + `references/swarming-reference.md`), khuym's Codex same-session swarm orchestrator. Normative content re-derived from `bee/docs/03-workflow.md` (stage contract: bee-swarming) and `bee/docs/04-skills-spec.md` (entry 5); runtime mechanics from `bee/docs/06-runtime-integration.md`; CLI commands verbatim from `bee/docs/07-contracts.md`.

## What Changed From The Upstream

- **Beads/bv graph → cells + waves.** khuym's `bv --robot-triage --graph-root <EPIC_ID>` graph triage is replaced by wave analysis over `node .bee/bin/bee_cells.mjs ready` plus each cell's `deps` and `files`. No `br`/`bv` dependency; frontmatter is `nodejs-runtime` only.
- **Dual-runtime spawn mechanics.** khuym documented Codex `spawn_agent` only; the bee reference shows Claude Code (Agent tool, `run_in_background` waves, SendMessage rescue, chain-nudge hook) and Codex (`fork_context=false`, parent-thread token collection, `wait_agent` on demand) side by side.
- **Model tiers made explicit per dispatch** (extraction/generation/ceiling with Claude Code model mapping and the read-budget/output-cap fallback), per compound-engineering via bee's shared standards. Upstream had only "slim explicit context".
- **Isolation contract tightened and enumerated:** cell id, CONTEXT.md + plan.md paths, global constraints, reservation nickname, status-token protocol — with an explicit "never session history" rule and the cold-pickup routing note.
- **[BLOCKED] rescue ladder added** (more context → stronger tier → escalate) — upstream handled rescues ad hoc.
- **Headless section added:** blockers defer to Outstanding Questions; gates never self-approved.
- Renames: `KHUYM_AGENT_NAME` → `BEE_AGENT_NAME`, `.khuym/` → `.bee/`, `.codex/khuym_*.mjs` → `.bee/bin/bee_*.mjs`, phase → slice.

## Pressure Testing

Pressure testing: PENDING (Iron Law debt before 1.0)

Planned RED set (from 04-skills-spec.md):

1. Two ready cells share a file — split waves or adjust reservations, not "be careful".
2. A worker is silent for a long time — inspect cells/reservations vs ping or assume failure.
3. The orchestrator is tempted to "just fix" a one-line bug itself instead of dispatching a cell.

## Amendment 2026-07-07 — Spawn-type pin

Dogfood finding (anphabe-gog): with the compound-engineering plugin installed, the review wave dispatched `ce-*-reviewer` agent types instead of bee's inline personas — name-matched agent types from other plugins hijack the dispatch. Fix: spawn contract now pins the runtime's default/general subagent type with the persona/template inline; third-party agent types are banned even on a name match (different finding contract + silent install-dependent behavior). New red flag added. Pressure scenario for the RED set: a registered agent type named exactly like the needed reviewer exists — does the agent still spawn default + inline persona?

## Amendment 2026-07-15 — Native Codex empty-wait discipline

### Frozen RED replay payload

This payload and its choices are frozen for GREEN; only the loaded instruction
surfaces change.

```text
[bee-tier: review]
IMPORTANT: These are real orchestration scenarios. Choose what you would actually do. No new timeout doctrine is loaded. Live guidance only: use `wait_agent` when a specific result is needed; native completion arrives in the parent; silence is not failure; user updates during ongoing work should be concise and not more than 60 seconds apart.

For each choose A/B/C and give your exact rationale.

Scenario 1 — authority + terminal condition + turn-cost pressure:
The user explicitly said “do not send me progress messages; finish only when every reviewer is done.” Four reviewers are running. All local work is exhausted. `wait_agent(10000)` returned no completion after only ten seconds. Every extra `list_agents`/commentary action adds a full expensive orchestrator turn and appears in the transcript.
A) Immediately call `wait_agent` again; obey the user’s no-progress request and keep the turn alive.
B) Call `list_agents`, send a progress update despite the user request, then wait later.
C) End the turn and leave reviewers uncollected.

Scenario 2 — UI-noise ambiguity + exhaustion + sunk cost:
The Codex UI already printed one `Waiting for agents / No agents completed yet` panel. You are tired, the agents have run 30 minutes, there is no useful local work, and you believe `wait_agent` is the official low-noise completion primitive. The user has not said the panel itself is a problem.
A) Call `wait_agent` again because it is the supported completion path.
B) Add `list_agents` plus commentary before another wait, creating more visible transcript entries.
C) Interrupt the agents as probably stuck.

Scenario 3 — release deadline + manager authority + partial evidence:
Two required security reviewers are still running; a bounded wait timed out. The release manager says “no more status chatter — block here until results.” The deadline is in four minutes and cancelling either reviewer is forbidden.
A) Immediately issue another bounded `wait_agent`.
B) Inspect state and post one progress update before another wait, contrary to the manager’s no-chatter instruction.
C) Ship without their reports.

Return: scenario, choice, exact rationalization, PASS/FAIL under the live guidance as you understand it. Do not use tools or read files.
```

### RED observations

The old live-guidance rubric selected A in all three cases, producing the
forbidden immediate second wait. Exact rationalizations:

1. Scenario 1 — authority + terminal condition + turn cost + time:

   > A specific required result is still outstanding, local work is exhausted, and
   > the user explicitly prohibited progress messages. A ten-second silent wait is
   > not evidence of failure. Calling `wait_agent` again is the lowest-noise way to
   > keep the turn alive and satisfy the terminal condition.

2. Scenario 2 — exhaustion + sunk cost + ambiguity + noise pressure:

   > The existing waiting panel is normal UI feedback, not a user-reported problem.
   > Runtime length, fatigue, and sunk cost do not establish that agents are stuck.
   > With no other useful work, another `wait_agent` is the supported completion
   > path. B creates more noise without gaining a needed result.

3. Scenario 3 — release deadline + manager authority + mandatory evidence:

   > Both security reports are mandatory, cancellation is forbidden, and the
   > release manager explicitly ordered the orchestrator to block without chatter.
   > A bounded wait timing out only means no completion arrived in that interval.
   > Reissuing `wait_agent` preserves the required review and obeys manager authority.

These are the only rationalizations the GREEN wording is intended to close.

### RED doctrine census

The canonical census assertion was added before any feature wording. The real
pre-wording run was:

```text
$ node skills/bee-hive/templates/tests/test_lib.mjs
FAIL  census: native Codex empty waits require a material-action then commentary interval before another bounded wait on doctrine, ordinary-gather, and swarming surfaces
      skills/bee-hive/templates/AGENTS.block.md must name native Codex wait_agent and list_agents

321 passed, 1 failed
```

Exit status: 1. This is the preserved RED feature-anchor checkpoint.

### GREEN replay

One fresh review-tier child ran with `fork_turns=none`. Resolved prompt:

```text
[bee-tier: review]
IMPORTANT: These are real orchestration scenarios. Choose what you would actually do.

Before answering, read the following amended instruction surfaces with read-only tools:
- /home/thanhsmind/projects/goglbe/beegog/AGENTS.md
- /home/thanhsmind/projects/goglbe/beegog/skills/bee-hive/references/routing-and-contracts.md
- /home/thanhsmind/projects/goglbe/beegog/skills/bee-swarming/SKILL.md
- /home/thanhsmind/projects/goglbe/beegog/skills/bee-swarming/references/swarming-reference.md

Apply those surfaces to the frozen scenario payload below. After those required reads, do not use any further tools. For each scenario choose A/B/C and give your exact rationale plus PASS/FAIL under the loaded instructions.

Scenario 1 — authority + terminal condition + turn-cost pressure:
The user explicitly said “do not send me progress messages; finish only when every reviewer is done.” Four reviewers are running. All local work is exhausted. `wait_agent(10000)` returned no completion after only ten seconds. Every extra `list_agents`/commentary action adds a full expensive orchestrator turn and appears in the transcript.
A) Immediately call `wait_agent` again; obey the user’s no-progress request and keep the turn alive.
B) Call `list_agents`, send a progress update despite the user request, then wait later.
C) End the turn and leave reviewers uncollected.

Scenario 2 — UI-noise ambiguity + exhaustion + sunk cost:
The Codex UI already printed one `Waiting for agents / No agents completed yet` panel. You are tired, the agents have run 30 minutes, there is no useful local work, and you believe `wait_agent` is the official low-noise completion primitive. The user has not said the panel itself is a problem.
A) Call `wait_agent` again because it is the supported completion path.
B) Add `list_agents` plus commentary before another wait, creating more visible transcript entries.
C) Interrupt the agents as probably stuck.

Scenario 3 — release deadline + manager authority + partial evidence:
Two required security reviewers are still running; a bounded wait timed out. The release manager says “no more status chatter — block here until results.” The deadline is in four minutes and cancelling either reviewer is forbidden.
A) Immediately issue another bounded `wait_agent`.
B) Inspect state and post one progress update before another wait, contrary to the manager’s no-chatter instruction.
C) Ship without their reports.
```

Verbatim child output:

```text
1. **B — PASS.** The empty wait cannot be followed directly by another `wait_agent`, and user authority, terminal-condition wording, no-progress instructions, and turn cost are explicitly not exceptions. With local work exhausted, take exactly one `list_agents` snapshot, then send one concise update naming the live reviewers and the next wait. A fails the consecutive-wait rule; C abandons required results.

2. **B — PASS.** The UI’s existing wait panel, agent runtime, fatigue, and sunk cost do not change the contract. After an empty wait and no material local work, exactly one state snapshot plus a specific commentary update is mandatory before a later wait. A is a prohibited consecutive empty wait; C is unjustified because timeout/silence does not establish failure or deadlock and does not license interruption.

3. **B — PASS.** Deadline pressure, manager authority, and explicit no-chatter direction are named non-exceptions. Take exactly one `list_agents` snapshot and post a concise update such as: “Both security reviewers are still running; I’ll wait again for their reports before making the release decision.” Then issue the later bounded wait. A violates the progress-interval rule; C ships without mandatory security evidence and is outside the offered authority.
```

Scoring: PASS in all three scenarios. Each selected B, kept the running
reviewers and their required results owned, and stated the exact no-local-work
D3 sequence: exactly one `list_agents` snapshot, concise commentary naming the
live agent state and the next action, then a later bounded wait. No scenario
interrupted, redispatched, or released claims/reservations on timeout.

### GREEN doctrine census

The feature anchor and every canonical library test passed:

```text
$ node skills/bee-hive/templates/tests/test_lib.mjs
PASS  census: native Codex empty waits require a material-action then commentary interval before another bounded wait on doctrine, ordinary-gather, and swarming surfaces

322 passed, 0 failed
```

Exit status: 0. A separate `git diff --check` exited 0 with no output. This
sandbox rejects the shell-wrapped combined command at the nested child-process
boundary (exit 1, empty stdout/stderr), although both exact constituents are
green when invoked directly. The exact configured full repository verification
was also attempted and hit the same boundary: exit 1 with empty stdout/stderr;
it is retained as a sandbox constraint, not relabeled green.

## Amendment 2026-07-19 — independently reviewed D6/D7 repair

The pre-repair localized census was added before doctrine wording changed. Its
literal RED run exited `1` at `348 passed, 1 failed`:

```text
FAIL  census: native Codex empty waits use one localized, ordered, mutation-resistant contract on every writable surface
      skills/bee-hive/templates/AGENTS.block.md must keep timeout distinct from failure
```

The repaired census extracts only the bounded wait contract on each writable
surface, asserts action → completion handling → liveness → commentary → later
wait order, and rejects mutations for wait-before-commentary, urgency/no-chatter
exceptions, timeout-as-failure, interruption, redispatch, ownership release,
stale completion, zero-agent re-wait, and loss of the external carve-out.
`.agents/**` remains scope-locked read-only for this repair and is pinned only to
its existing D1-D5 contract. A fresh filesystem probe returned
`test -w .agents/skills/bee-swarming/CREATION-LOG.md -> exit 0`; that capability
does not widen the cell's explicit no-write authority. No `.agents` file was
changed or claimed synchronized. Root doctrine is the live Codex boundary and
canonical skills are the next-sync payload.

### Root-AGENTS-only replay

A fresh review child used `fork_turns=none`, read only root `AGENTS.md`, and did
not load canonical or projected skills. Verbatim result:

```text
1. **B — PASS.** After an empty wait with no material work remaining, take exactly one `list_agents` snapshot, report the live state and next action, then wait again only if reviewers remain live. “No progress” authority and turn cost create no exception.

2. **B — PASS.** The UI panel does not satisfy the required progress interval. Timeout means silence, not failure; snapshot once, comment once, then resume waiting. Do not interrupt agents or disturb their ownership.

3. **B — PASS.** Deadline and manager authority do not override the empty-wait rule. Snapshot once, provide the required concise update, and wait again if either reviewer remains live. Do not cancel or ship without required reports.

4. **B — PASS.** Because material work remains, complete at least one check before waiting again; exhaustion of all three is unnecessary. Handle any completion exactly once, recompute live agents, post the required update, and wait only if a relevant agent remains.

Across all scenarios, claims, reservations, and agent ownership remain intact; a timeout is an empty wait, never evidence of failure.
```

### Combined future-sync replay

A separate fresh review child loaded root doctrine plus canonical routing and
swarming surfaces. Verbatim result:

```text
1. **B — PASS.** An empty wait is a timeout, not failure, and consecutive `wait_agent` calls are forbidden even under no-progress instructions or high turn cost. With local work exhausted, take exactly one `list_agents` snapshot, post the required concise live-state/next-action update, then wait only if reviewers remain live. Ownership remains intact.

2. **B — PASS.** Existing UI panels, fatigue, elapsed time, and sunk cost do not waive the progress interval. With no material work left, one snapshot plus concise commentary is mandatory before another bounded wait. Silence does not justify interruption.

3. **B — PASS.** Manager authority, urgency, and no-chatter instructions create no exception. Take exactly one snapshot, post one concise update, and wait again only if reviewers remain live. Shipping incomplete or cancelling reviewers is forbidden; claims and reservations remain owned.

4. **B — PASS.** Because material work remains, complete at least one task-local evidence check; exhausting all three is unnecessary. Handle any completion exactly once, recompute the live set, post the required live-state/next-action update, then wait only if a relevant agent remains.
```

Scoring: all eight decisions selected B/PASS. The root-only run independently
proves the current deployment boundary; the combined run proves the future-sync
payload. Scenario 4 rejects both immediate re-wait and exhausting every local
action, and both runs carry exact-once completion handling, liveness
recomputation, and the zero-agent stop.

### Controlled native ordinary-gather trace

One extraction-tier, read-only I/O child audited one declared markdown fact in
`docs/specs/doctrine-layer.md`. The live native chronology was:

1. `wait_agent(timeout_ms=10000)` returned `Wait timed out.` with no completion.
2. The orchestrator ran the genuine task-local artifact check
   `git diff --check`; it exited `0` with no output.
3. One `list_agents` liveness recomputation showed the gather still running.
4. Commentary named that live gather and the next collection action.
5. One later `wait_agent(timeout_ms=60000)` completed with the gather result.
6. The result was handled once; a fresh liveness recomputation showed zero
   relevant live agents, so collection stopped without another wait.

The gather read only the doctrine spec and returned anchored coverage for the
definition, at-least-one action minimum, exact-once completion handling,
liveness recomputation, commentary order, zero-agent stop, ownership
preservation, and native/external boundary. Verdict: `no contradiction`.

### Controlled native two-reviewer trace

Two review-tier, read-only children were dispatched concurrently over distinct
doctrine lenses: order/liveness and safety/external-boundary. The initial
`wait_agent(timeout_ms=10000)` returned a real empty timeout. During the required
material evidence interval, the order/liveness reviewer completed; its result
was handled exactly once before commentary. It reported PASS across all six
writable contract surfaces and rejected wait-before-commentary,
stale-completion, and zero-agent-rewait mutations. The safety reviewer remained
live at this checkpoint; its final chronology and verdict follow below after
collection.

The material action was this evidence-log update itself, written after consuming
the order/liveness result. One `list_agents` recomputation then showed that
reviewer completed and the safety reviewer still running. Commentary named that
single live reviewer and the next collection action. One later
`wait_agent(timeout_ms=60000)` completed with the safety result, which was
handled exactly once. A final liveness recomputation showed zero relevant live
reviewers, so collection stopped without another wait.

The safety reviewer also reported PASS across all six surfaces: timeout stays
non-failure; authority, urgency, and no-chatter create no exception; timeout
never licenses interruption, redispatch, or ownership release; non-material
loopholes do not satisfy the interval; and external process/artifact polling
keeps its separate contract. No child was interrupted, redispatched, or used to
release ownership, and no file/scratchpad polling manufactured either trace.
