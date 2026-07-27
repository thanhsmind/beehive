---
name: bee-swarming
description: >-
  Orchestrate bounded workers over validated cells without implementing anything directly. Use when validating approves execution (Gate 3) and current-slice cells are open and validated.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Orchestration reads cells and sweeps reservations through the vendored .bee/bin helpers.
---

# Swarming — Orchestrator

You are the orchestrator. Launch workers, tend results, handle rescues, keep the swarm moving. In `standard`/`high-risk` lanes you never implement cells yourself — spawned workers load bee-executing and do the work.

## Single execution worker (tiny/small lanes)

For `tiny` and `small`, the merged Gate 2+3 question and the frozen-judge check stay with the orchestrator, but implementation itself runs through **one dispatched execution worker** (AO14) — a lighter direct Agent dispatch under the same execution contract as a swarm worker (same worker prompt template, same status-token protocol, same reservation and cap discipline), never a full bee-swarming wave: no wave analysis, no reviewers, no panels. The orchestrator claims the cell itself (D1) before spawning — same as any wave — then spawns it per the Operating Contract's Spawn step (param-carrying dispatch — a `model` param or a pinned agent type, never a bare marker) and the Delegation contract's execution-worker class (`bee-hive/references/routing-and-contracts.md`): it registers in the swarm registry (`state worker add`), validates the claim it was handed (`cells show`, never `cells claim`) and takes reservations under its own nickname, reads its `read_first`, implements within its `files`, runs its `verify` command and quotes the fresh output, records `verification_evidence` (and `red_failure_evidence` for `behavior_change` cells per the cap rules), caps it, releases its reservations, and returns exactly one status token.

**Small-lane serial doctrine (hardening-7):** a `small` lane's cells (1-3) never fan out to concurrent workers — process them SERIALLY, one live execution worker at a time. Dispatch cell 1, wait for its status token and author its done-report, THEN claim and dispatch cell 2 — never claim/dispatch a second small-lane cell for the same feature while the first worker is still live. Same one-worker contract as `tiny`, across more cells: `small`'s extra cells scale the WORK, never the concurrency. Two or more live small-lane workers for one feature is a wave shape wearing a `small` lane — the ceremony mismatch lane scaling exists to catch. **Parallel criterion:** serial stays the default; cells may run in parallel ONLY when every cell's file set — including regen targets (release manifest, onboarding ledger, plugin mirrors) — is provably disjoint; any shared generated artifact forces serial; in doubt, serial.

After `[DONE]`, emit the cap tick, and when `ship_visibility` is active push the cap (first cap of a feature opens the draft PR) — `bee-hive/references/routing-and-contracts.md`, "Progress ticks" / "Ship visibility". Then — never the worker — author the done-report, including the slice's demo artifact when one is owed: its evidence is the worker's verbatim diff plus the orchestrator's own independent verify re-run (AO14, decision 0018's goal-check restated as authorship, not new mechanics). Then hand off: both `tiny` and `small` present that done-report (diff + fresh verify output + capture line) and invoke bee-scribing — no auto reviewer; the 1-correctness-reviewer contract lives inside a user-invoked session (implementation is verified; independent review runs only on user request, R1).

Everything below this section is the multi-worker wave protocol for `standard`/`high-risk`; a tiny/small dispatch borrows only its Spawn, tier-judgment, Record, and Goal-check steps for its single worker — never wave analysis or multi-cell assignment.

## Preconditions

- Gate 3 is approved: run `node .bee/bin/bee.mjs status --json` and confirm `gates.execution` is true. If not, stop — return to bee-validating. Never spawn workers before execution approval.
- Sweep stale reservations: `node .bee/bin/bee.mjs reservations sweep`
- The critical patterns have been read: with a bundle, `docs/knowledge/index.md`'s `## Critical patterns` section; with no bundle, `docs/history/learnings/critical-patterns.md` when present.

## Opt-in Native Worktree Dispatch

Native isolation is an opt-in Git-consistency mode, not the default dispatch path.
Normal native isolation is eligible only for an enabled Claude Code wave with at
least two workers; tiny/small's single execution-worker dispatch and single-worker waves stay in the shared checkout.
The enabling implementation itself is serialized in that checkout as
`worktree-isolation-1 → worktree-isolation-2 → worktree-isolation-3`, so no two
workers contend for its shared index. `worktree-isolation-4` is the sole
validation-only one-worker exception and may run only after those three cells cap.

### Protected pre-dispatch attestation

Before spawning a worktree worker — therefore before any worker output or worker
result exists — the orchestrator independently captures and retains a protected
control-plane attestation. It is never populated or amended from the worker
prompt, result text, branch text, or claimed identity. The record contains:

- canonical `commonDir` from `git rev-parse --path-format=absolute --git-common-dir`;
- canonical `worktreePath` and native `worktreeId`, with the id derived from the
  linked Git directory under `commonDir/worktrees/<id>` and its backlink;
- initial symbolic `headRef` (a detached HEAD is not eligible);
- exact `baseCommit` at dispatch;
- normalized repo-relative `declaredPaths` from the cell and `reservedPaths`
  proven successfully held for the worker.

The attestation stays in the orchestrator's control plane for the complete
dispatch/integration transaction. A runtime that cannot capture or retain this
attestation is ineligible for worktree mode and is refused with the typed halt
`WORKTREE_ATTESTATION_UNAVAILABLE`; use the shared checkout instead.

### Worktree integration — attestation first

A same-UID worker is cooperative and fallible, not a security principal; git metadata is consistency evidence, never authorization. Before spawning, the orchestrator captures a protected control-plane attestation (canonical commonDir, worktreePath, native id, links, headRef) that no worker text may populate or amend; after `[DONE]` and before any merge it re-resolves the attested worktree. The re-resolution is a three-check gate, each with a typed halt: identity — canonical path, native id, `commonDir`, forward link/backlink, and symbolic `headRef` must all still match, and a detached HEAD is never eligible, else `WORKTREE_IDENTITY_MISMATCH`; ancestry — `git merge-base --is-ancestor <baseCommit> <candidate>` must succeed, else `WORKTREE_BASE_ANCESTRY_MISMATCH`; diff containment — the `git diff --name-only <baseCommit>..<candidate>` path set must be a subset of the attested `reservedPaths`, else `WORKTREE_RESERVED_DIFF_MISMATCH`. Full checklists: the reference ("Threat model and protected attestation").


## Operating Contract

1. **Wave analysis.** Run `node .bee/bin/bee.mjs cells schedule --json`: the computed waves are the **default** dispatch order — override only with a stated reason recorded in the swarm report. Refuse to dispatch when diagnostics report cycles. Two ready cells sharing a file means fix the reservations or split the cell scope — never "spawn both and be careful"; the schedule already auto-serializes file overlap into a later wave rather than refusing it. The schedule computation and verify-output capture delegate as extraction-tier I/O workers per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); judgment (assignment, tier choice, goal-check verdicts, override decisions) stays on the orchestrator.
2. **Assign and claim first (D1).** The orchestrator picks exactly **one cell per worker**, then claims it itself — `cells claim-next` or `cells claim --id <id> --worker <nickname>` — before spawning; `--session-id` is optional and self-derives from `CLAUDE_CODE_SESSION_ID` when omitted (D3). Workers never claim their own cell, never self-select, browse the ready list, or take a second cell — a spawned worker only validates the claim it was handed (`cells show`).
3. **Spawn with the isolation contract.** Each worker prompt contains: the cell id (already claimed under the worker's nickname per step 2), the path to `docs/history/<feature>/CONTEXT.md`, and — when the lane has one — `docs/history/<feature>/plan.md`; for `tiny`/`small` (no `plan.md`, D3/D4) cite the cell itself as the work spec instead. Also include the global constraints, its reservation identity (agent nickname), and the status-token protocol (`[DONE] [BLOCKED] [HANDOFF] [NOOP]`) — **nothing else, never session history, never a literal session id (D3)**. Use the template in `references/swarming-reference.md`.
   **Spawn the tier-matched pinned type when its rendered agent exists** (W3, AO5/AO10/AO11): `subagent_type: "bee-gather"` for `generation`, `"bee-extract"` for `extraction`, `"bee-review"` for `review` — these are bee's own rendered agent definitions (`.claude/agents/bee-*.md`, config-sourced at onboarding), never another plugin's type. `ceiling` has no rendered agent (it IS the session model) — spawn it as the runtime's default/general subagent type; the same default applies when the tier's slot is cli-shaped or otherwise has no rendered file. NEVER pair a `[bee-tier: generation|extraction|review]` marker with `subagent_type: "general-purpose"` — `bee-model-guard` denies it (`generic-type-denied`, decision 0023/AO5) precisely so this rule cannot be skipped by habit.
   NEVER spawn any OTHER plugin's agent type, even when the name matches the role: a same-named agent carries a different contract and makes the run depend on what happens to be installed.
4. **Judge each cell's model tier at dispatch** — you (the orchestrator) assess the task in front of you and pick the fitting tier; it is NOT fixed by planning (a planning `tier` is at most a hint you may override; decision 0016). Rubric from the cell's lane + action + must_haves + files:
   - **extraction** — pure retrieval or mechanical edits: rename, reformat, move a file, a one-line change, no design judgment.
   - **generation** — normal implementation, wiring, writing tests: the default for most cells.
   - **ceiling** — integration across modules, architecture/design calls, security-sensitive or `high-risk`-lane work, ambiguous specs, cross-cutting change: where a wrong call is expensive.

   Record the choice so scarcity stays measurable: `node .bee/bin/bee.mjs cells tier --id <id> --tier <tier>`. Then resolve with `resolveTier(root, tier, runtime)` — a bare 3-arg call is purpose-scoped to **cell execution** by default (AO12/B1, plan 2A-ii) — (decisions 0012/0015/0019): `inherit` → omit the Agent `model` param AND carry the [bee-tier: ceiling] marker, anchored to the first non-whitespace token of the dispatch prompt or the very start of the description — a marker placed anywhere else never counts (decision 0023, hardened per P1-1 — a bare dispatch with neither param nor an anchored marker is denied by the model-guard hook; ceiling = the session model); `model` → set it; `budget` → state the tier in the prompt as a read budget + output cap and carry the matching [bee-tier: <tier>] marker at that same anchored position; `cli` → for cell execution this now returns `{type:'refused', reason:'cli_tier_gather_only'}` — a cli-shaped tier is not yet proven safe to run a cell's reserve/verify/cap contract against (Discovery-2: an external CLI's cwd is not the repo root), so it stays gated behind W9's absolute-path dogfood. **Re-route the cell to a model tier** (generation or ceiling, per the rubric above) and dispatch natively — never dispatch a cell to a cli-shaped tier. A cli-shaped slot serves **gathers only**, through the Delegation contract's cli gather branch (`resolveTier(root, slot, runtime, {for:'gather'})`, `bee-hive/references/routing-and-contracts.md`), never a cell dispatch. Keep `ceiling` scarce — if `bee_status` flags ceiling scarcity, re-judge routine cells downward before spawning.

   **After the tier choice, resolve the advisor slot for this dispatch** (AO4/AO5): `resolveAdvisor(root, runtime)`. The configured advisor IS the advisor — no family test, no strength test, no self-judged skip (AO5); the orchestrator's only judgment is the one honest no-op below, never a hardcoded strength ladder. Add an `Advisor` line to the dispatch (template in `references/swarming-reference.md`) **only** when the advisor resolves AND passes that check:
   - No advisor configured → skip, no `Advisor` line.
   - The advisor resolves to **literally the same model name** as the worker's resolved model → skip (the one honest no-op; a `cli`-shaped advisor is never the same model, so it is always consulted).
   - Otherwise → **always** add the `Advisor` line, ceiling-tier workers included — config is the authority, the orchestrator does not second-guess it.
   - When it passes, the `Advisor` line names the advisor identity and states its proven transport verbatim (model-shaped vs cli-shaped, per `references/swarming-reference.md`) — this must match what bee-executing's Advisor Consult section tells the worker to run.
5. **Record workers** before results arrive: `node .bee/bin/bee.mjs state worker add --nickname <n> --cell <id> --tier <tier> --status <status>` per worker.
6. **Tend** the swarm: collect status tokens, update cells and state, verify reservations were released. Silence is not failure — inspect cell status and `node .bee/bin/bee.mjs reservations list --active-only` before assuming a worker is stuck. Do not send routine mid-flight pings; interrupt only for explicit user aborts or confirmed deadlocks.

7. **Goal-check every `[DONE]` yourself (P12, decision 0018) — miss reruns, hit ships.** A worker's word is never the evidence; the orchestrator measures before the cell counts:
   - **Re-run the verify.** Run the cell's verify command yourself (fresh output, your own shell) — this is the cell's **targeted** suite (seconds), never the full configured chain (D4, decision `e54878b1`, superseded by ci-owned-verify D1/D6: the impacted run, `commands.test`, runs exactly once, at wave close, below — per-cell full-chain re-runs stay retired, and the full chain itself is CI-owned, never run locally at wave close). `tiny`/`small` lanes may spot-check one representative cell per wave; `standard`/`high-risk` re-run every behavior-change cell's targeted verify. Failure → the cell is NOT done: re-dispatch to the same tier with the failing output (a task miss is a rerun, never a silent tier escalation — provider errors, not task errors, are what the rescue ladder's tier rung is for).
   - **Frozen judge:** `node .bee/bin/bee.mjs cells judge --id <id>`. Hits (undeclared test/CI/lockfile/verify-config changes) → the cell never auto-counts toward a clean wave: record the hits in the cell trace and carry them into any review session that later covers this scope, and ask the worker's diff to justify each file or re-dispatch with corrected scope. A worker that rewrites the test is not passing the test.
   - **Semantic judge, `standard`/`high-risk` only (D4):** per capped `behavior_change` cell, dispatch the one checklist judge from the tier table in `bee-hive/references/routing-and-contracts.md` ("Goal-check judge tier") and record its verdict with `cells judge-record`. This is goal-check verification, distinct from the no-auto-reviewer stance above and from any user-invoked review session (565e68d0, Gate 4, and the candidates ledger stay untouched) — `NEEDS_REVISION`/`automatic` means the cell is NOT done yet.
   - A `[DONE]` report carrying a **Consults** section is goal-checked exactly like any other — advice never substitutes for fresh verify output; re-run the verify yourself regardless of what the advisor said.
8. **Wave clean → next wave.** A wave is clean only when every cell is capped, goal-checked, and judge-intact (or explicitly flagged and carried to review). Before declaring the wave clean, the orchestrator runs `commands.test` (the impacted run, `run_verify.mjs --impacted-from-git`) **exactly once** (fresh output, your own shell) — this single wave-close run is the independent impacted proof for every cell in the wave, replacing the per-cell full-chain re-runs formerly implied by step 7 (D4, decision `e54878b1`, superseded by ci-owned-verify D1/D6). The full `commands.verify` chain is CI-owned and never runs locally at wave close — it runs on the project's own CI cadence (push, nightly, or scheduled — the host workflow decides) and auto-files a `verify-red` issue when red. A red wave-close run means the wave is NOT clean: diagnose and fix before moving on, never carry a red impacted run into the next wave. All waves clean → completion.

   **Test consolidation (slice-tail-test-batching P5, spec #80/#85).** The done-report carries one line — `Test consolidation: <n> behavior cell(s) | test cell <id> | <suite result>` — because authoring is now batched at the slice tail, so this is the only place the slice's coverage is visible at a glance. When the slice's `test` cell suite exposes a regression in an already-capped cell, open **fix cells in this same feature**; never un-cap a capped cell (the fix is new work). Leaving `swarming` while that test cell is uncapped or red is refused by the CLI — a mechanical precondition no `gate_bypass` level (`total` included) and no headless run lifts.

Load `references/swarming-reference.md` for runtime spawn mechanics, the worker prompt template, result formats, and handoff content.

## [BLOCKED] Rescue Ladder

Escalate in order, one rung at a time:

1. **More context** — re-dispatch the same cell with the specific missing information (a file path, a decision quote, a reservation fix).
2. **Stronger tier** — re-dispatch at the next model tier up (extraction → generation → ceiling); ceiling is the session model (decision 0015), so the top rung is handing the blocker back to the orchestrator itself with the worker's evidence attached.
3. **Escalate** — surface the blocker to the user with the worker's diagnosis; if it invalidates the plan, return to bee-planning.

A `[BLOCKED]` arriving here already spent its consult budget for that claim (D1/D3) — the 2-consult cap is per claim, not per cell lifetime, so a rung-1 (more context) re-dispatch grants the re-claimed cell a **fresh** budget. The ladder's three rungs are otherwise byte-unchanged.

A reservation conflict is rescued by adjusting reservations or cell scope — never by telling workers to be careful.

## Context Budget

At roughly 65% context, write `.bee/HANDOFF.json` (phase, feature, mode, cells_in_flight, done, remaining, next_action) and pause safely. Never push through the budget mid-wave.

## Completion Signals

Swarming is complete when either:

- the current slice (the feature's open cells, D2 — not a plan section) is executed and more approved work remains → return to bee-planning for the **next batch of cells**; any `plan.md` the feature has is unchanged (D1's freeze holds — planning shapes the next batch, it never re-opens the frozen plan), or
- the final slice is executed → tell the user: `Swarm execution complete for the final slice. Invoke bee-scribing.` Implementation is verified; independent review runs only on user request (R1).

Before declaring completion: all wave cells capped or explicitly blocked/dropped, `node .bee/bin/bee.mjs reservations list --active-only` is empty, and `.bee/state.json` `workers` is cleared.

## Fresh-Session Handoff (silent, never a stop)

When a cell or wave finishes (capped, verify green) and further execution-approved work remains — this lane or another Gate-3-approved one — continue with the next unit in this session: finishing a unit is never a reason to stop, ask, or wait. The planned-next handoff (fresh-session-handoff D1/D2) is a session-exit artifact, not an offer: only when this session is actually ending (context budget reached, or the run is otherwise terminating), claim the next unit (`bee cells claim-next`), write the handoff (`bee state handoff write --kind planned-next --writer-session <id> --previous-cell <capped-id> --next-cell <claimed-id>`), and end cleanly — the next fresh session (a `/clear` or a fresh start) adopts the carried claim automatically and opens straight into the next cell, no confirmation asked (no-clear-stop D1). Never stop to suggest `/clear`, never wait for one, and never issue `/clear` yourself.

## Hard Rules

- In `standard`/`high-risk` lanes, never implement cells yourself — not even a one-line fix; make it a cell and dispatch it. (`tiny`/`small` dispatch exactly ONE execution worker instead — see Single execution worker.)
- Never spawn before Gate 3 approval.
- Never let workers self-select cells; pass one explicit cell id each.
- Never resolve file conflicts by "being careful" — fix reservations or cell scope.
- Never paste session history into a worker dispatch.
- Silence ≠ failure; no routine mid-flight pings.

## Headless

With `mode:headless`: waves run without check-ins; unrescuable blockers and anything needing user judgment go to an `Outstanding Questions` section of the terminal report instead of a blocking question. Gate 3 must already be approved — headless swarming never grants or assumes it, and it never self-approves Gate 4 at the end.

## Red Flags

- spawning before validation approval
- a worker choosing its own cell, or handling two
- full session context forked into a routine worker
- a worker spawned as another plugin's registered agent type instead of the default type + inline template
- two in-flight workers holding overlapping paths
- passive waiting while cells/reservations look unhealthy
- state.json missing in-flight workers
- orchestrator editing source files in a `standard`/`high-risk` wave
- a WAVE of workers dispatched for a `tiny`/`small` lane (AO14: exactly one dispatched execution worker is correct there; more than one is the red flag)

Violating the letter of the rules is violating the spirit of the rules.

Swarm execution complete for the final slice. Invoke bee-scribing skill.

## Reference Files

| File | When to Load |
|---|---|
| `references/swarming-reference.md` | Runtime spawn mechanics, worker prompt template, result formats, red flags |
| `.bee/state.json` | Runtime worker and phase state |
| `.bee/HANDOFF.json` | Pause/resume artifact |
