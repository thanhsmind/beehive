# Swarming Reference

Load after Gate 3 approval, before spawning the first wave.

## Single execution worker in full

For `tiny` and `small`, the merged Gate 2+3 question and the frozen-judge
check stay with the orchestrator, but implementation itself runs through
**one dispatched execution worker** (AO14) — a lighter direct Agent dispatch
under the same execution contract as a swarm worker (same worker prompt
template, same status-token protocol, same reservation and cap discipline),
never a full bee-swarming wave: no wave analysis, no reviewers, no panels.
The orchestrator claims the cell itself (D1) before spawning — same as any
wave — then spawns it per the Operating Contract's Spawn step (param-carrying
dispatch — a `model` param or a pinned agent type, never a bare marker) and
the Delegation contract's execution-worker class
(`bee-hive/references/routing-and-contracts.md`): it registers in the swarm
registry (`state worker add`), validates the claim it was handed (`cells
show`, never `cells claim`) and takes reservations under its own nickname,
reads its `read_first`, implements within its `files`, runs its `verify`
command and quotes the fresh output, records `verification_evidence` (and
`red_failure_evidence` for `behavior_change` cells per the cap rules), caps
it, releases its reservations, and returns exactly one status token.

**Small-lane serial doctrine (hardening-7):** a `small` lane's cells (1-3)
never fan out to concurrent workers — process them SERIALLY, one live
execution worker at a time. Dispatch cell 1, wait for its status token and
author its done-report, THEN claim and dispatch cell 2 — never claim/dispatch
a second small-lane cell for the same feature while the first worker is still
live. Same one-worker contract as `tiny`, across more cells: `small`'s extra
cells scale the WORK, never the concurrency. Two or more live small-lane
workers for one feature is a wave shape wearing a `small` lane — the ceremony
mismatch lane scaling exists to catch. **Parallel criterion:** serial stays
the default; cells may run in parallel ONLY when every cell's file set —
including regen targets (release manifest, onboarding ledger, plugin
mirrors) — is provably disjoint; any shared generated artifact forces
serial; in doubt, serial.

After `[DONE]`, emit the cap tick, and when `ship_visibility` is active push
the cap (first cap of a feature opens the draft PR) —
`bee-hive/references/routing-and-contracts.md`, "Progress ticks" / "Ship
visibility". Then — never the worker — author the done-report, including the
slice's demo artifact when one is owed: its evidence is the worker's
verbatim diff plus the orchestrator's own independent verify re-run (AO14,
decision 0018's goal-check restated as authorship, not new mechanics). Then
hand off: both `tiny` and `small` present that done-report (diff + fresh
verify output + capture line) and invoke bee-scribing — no auto reviewer;
the 1-correctness-reviewer contract lives inside a user-invoked session
(implementation is verified; independent review runs only on user request,
R1).

The rest of this reference and the body's Operating Contract are the
multi-worker wave protocol for `standard`/`high-risk`; a tiny/small dispatch
borrows only its Spawn, tier-judgment, Record, and Goal-check steps for its
single worker — never wave analysis or multi-cell assignment.

## Operating Contract in full

1. **Wave analysis.** Run `node .bee/bin/bee.mjs cells schedule --json`: the
   computed waves are the **default** dispatch order — override only with a
   stated reason recorded in the swarm report. Refuse to dispatch when
   diagnostics report cycles. Two ready cells sharing a file means fix the
   reservations or split the cell scope — never "spawn both and be careful";
   the schedule already auto-serializes file overlap into a later wave
   rather than refusing it. The schedule computation and verify-output
   capture delegate as extraction-tier I/O workers per the Delegation
   contract (D2/D3, `bee-hive/references/routing-and-contracts.md`);
   judgment (assignment, tier choice, goal-check verdicts, override
   decisions) stays on the orchestrator.
2. **Assign and claim first (D1).** The orchestrator picks exactly **one
   cell per worker**, then claims it itself — `cells claim-next` or `cells
   claim --id <id> --worker <nickname>` — before spawning; `--session-id` is
   optional and self-derives from `CLAUDE_CODE_SESSION_ID` when omitted
   (D3). Workers never claim their own cell, never self-select, browse the
   ready list, or take a second cell — a spawned worker only validates the
   claim it was handed (`cells show`).
3. **Spawn with the isolation contract.** Each worker prompt contains: the
   cell id (already claimed under the worker's nickname per step 2), the
   path to `docs/history/<feature>/CONTEXT.md`, and — when the lane has one
   — `docs/history/<feature>/plan.md`; for `tiny`/`small` (no `plan.md`,
   D3/D4) cite the cell itself as the work spec instead. Also include the
   global constraints, its reservation identity (agent nickname), and the
   status-token protocol (`[DONE] [BLOCKED] [HANDOFF] [NOOP]`) — **nothing
   else, never session history, never a literal session id (D3)**. Use the
   template below.
<!-- bee:only claude -->
   **Spawn the tier-matched pinned type when its rendered agent exists**
   (W3, AO5/AO10/AO11): `subagent_type: "bee-gather"` for `generation`,
   `"bee-extract"` for `extraction`, `"bee-review"` for `review` — these are
   bee's own rendered agent definitions (`.claude/agents/bee-*.md`,
   config-sourced at onboarding), never another plugin's type. `ceiling` has
   no rendered agent (it IS the session model) — spawn it as the runtime's
   default/general subagent type; the same default applies when the tier's
   slot is cli-shaped or otherwise has no rendered file. NEVER pair a
   `[bee-tier: generation|extraction|review]` marker with `subagent_type:
   "general-purpose"` — `bee-model-guard` denies it (`generic-type-denied`,
   decision 0023/AO5) precisely so this rule cannot be skipped by habit.
<!-- bee:end -->
<!-- bee:only codex -->
   Codex has no per-agent `subagent_type` equivalent (AO11 asymmetry) — its
   tier stays enforced as a read budget + output cap only, exactly as
   before.
<!-- bee:end -->
   NEVER spawn any OTHER plugin's agent type, even when the name matches the
   role: a same-named agent carries a different contract and makes the run
   depend on what happens to be installed.
4. **Judge each cell's model tier at dispatch** — you (the orchestrator)
   assess the task in front of you and pick the fitting tier; it is NOT
   fixed by planning (a planning `tier` is at most a hint you may override;
   decision 0016). Rubric from the cell's lane + action + must_haves +
   files:
   - **extraction** — pure retrieval or mechanical edits: rename, reformat,
     move a file, a one-line change, no design judgment.
   - **generation** — normal implementation, wiring, writing tests: the
     default for most cells.
   - **ceiling** — integration across modules, architecture/design calls,
     security-sensitive or `high-risk`-lane work, ambiguous specs,
     cross-cutting change: where a wrong call is expensive.

   Record the choice so scarcity stays measurable: `node .bee/bin/bee.mjs
   cells tier --id <id> --tier <tier>`. Then resolve with `resolveTier(root,
   tier, runtime)` — full semantics, tier-marker anchoring, and dispatch
   economics: "Model Tiers — Config-Driven, Runtime-Keyed" below. Keep
   `ceiling` scarce — if `bee_status` flags ceiling scarcity, re-judge
   routine cells downward before spawning.

   **After the tier choice, resolve the advisor slot for this dispatch**
   (AO4/AO5): `resolveAdvisor(root, runtime)`. The configured advisor IS the
   advisor — no family test, no strength test, no self-judged skip (AO5);
   the orchestrator's only judgment is the one honest no-op below, never a
   hardcoded strength ladder. Add an `Advisor` line to the dispatch
   (template below) **only** when the advisor resolves AND passes that
   check:
   - No advisor configured → skip, no `Advisor` line.
   - The advisor resolves to **literally the same model name** as the
     worker's resolved model → skip (the one honest no-op; a `cli`-shaped
     advisor is never the same model, so it is always consulted).
   - Otherwise → **always** add the `Advisor` line, ceiling-tier workers
     included — config is the authority, the orchestrator does not
     second-guess it.
   - When it passes, the `Advisor` line names the advisor identity and
     states its proven transport verbatim (model-shaped vs cli-shaped, per
     the Worker Prompt Template below) — this must match what
     bee-executing's Advisor Consult section tells the worker to run.
5. **Record workers** before results arrive: `node .bee/bin/bee.mjs state
   worker add --nickname <n> --cell <id> --tier <tier> --status <status>`
   per worker.
6. **Tend** the swarm: collect status tokens, update cells and state, verify
   reservations were released. Silence is not failure — inspect cell status
   and `node .bee/bin/bee.mjs reservations list --active-only` before
   assuming a worker is stuck. Do not send routine mid-flight pings;
   interrupt only for explicit user aborts or confirmed deadlocks.
<!-- bee:only codex -->
   For native Codex agents, a completed `wait_agent` call with no completion
   is an **empty wait** and a timeout signal only. A `wait_agent`
   timeout/no-completion result is only an empty wait; silence is not
   failure. Never call `wait_agent` twice consecutively after an empty wait;
   authority, urgency, and no-chatter instructions create no exception.
   Before any later bounded wait, perform at least one material task-local
   action when work remains; that one action satisfies the interval, and
   exhausting all local work is not required. Only when no material work
   remains, take exactly one `list_agents` snapshot. Handle any completion
   that arrives during the interval exactly once, then recompute the
   relevant live-agent set. Send one concise commentary update naming both
   the live agent state and the next action. Only after this commentary may
   a later bounded wait run, and only while the relevant live-agent set is
   non-empty; zero live agents ends collection without another wait. No-op
   work, repeated state reads, hidden reasoning, generic commentary, or
   commentary alone do not qualify. Timeout never licenses interrupt,
   duplicate dispatch, claim release, or reservation release; every running
   agent, claim, and reservation stays owned. External process and artifact
   polling remains outside this native-agent rule and stays governed by the
   separate executor contract below.
<!-- bee:end -->
7. **Goal-check every `[DONE]` yourself (P12, decision 0018) — miss reruns,
   hit ships.** A worker's word is never the evidence; the orchestrator
   measures before the cell counts:
   - **Re-run the verify.** Run the cell's verify command yourself (fresh
     output, your own shell) — this is the cell's **targeted** suite
     (seconds), never the full configured chain (D4, decision `e54878b1`,
     superseded by ci-owned-verify D1/D6: the impacted run, `commands.test`,
     runs exactly once, at wave close, below — per-cell full-chain re-runs
     stay retired, and the full chain itself is CI-owned, never run locally
     at wave close). `tiny`/`small` lanes may spot-check one representative
     cell per wave; `standard`/`high-risk` re-run every behavior-change
     cell's targeted verify. Failure → the cell is NOT done: re-dispatch to
     the same tier with the failing output (a task miss is a rerun, never a
     silent tier escalation — provider errors, not task errors, are what
     the rescue ladder's tier rung is for).
   - **Frozen judge:** `node .bee/bin/bee.mjs cells judge --id <id>`. Hits
     (undeclared test/CI/lockfile/verify-config changes) → the cell never
     auto-counts toward a clean wave: record the hits in the cell trace and
     carry them into any review session that later covers this scope, and
     ask the worker's diff to justify each file or re-dispatch with
     corrected scope. A worker that rewrites the test is not passing the
     test.
   - **Semantic judge, `standard`/`high-risk` only (D4):** per capped
     `behavior_change` cell, dispatch the one checklist judge from the tier
     table in `bee-hive/references/routing-and-contracts.md` ("Goal-check
     judge tier") and record its verdict with `cells judge-record`. This is
     goal-check verification, distinct from the no-auto-reviewer stance
     above and from any user-invoked review session (565e68d0, Gate 4, and
     the candidates ledger stay untouched) — `NEEDS_REVISION`/`automatic`
     means the cell is NOT done yet.
   - A `[DONE]` report carrying a **Consults** section is goal-checked
     exactly like any other — advice never substitutes for fresh verify
     output; re-run the verify yourself regardless of what the advisor said.
8. **Wave clean → next wave.** A wave is clean only when every cell is
   capped, goal-checked, and judge-intact (or explicitly flagged and carried
   to review). Before declaring the wave clean, the orchestrator runs
   `commands.test` (the impacted run, `run_verify.mjs --impacted-from-git`)
   **exactly once** (fresh output, your own shell) — this single wave-close
   run is the independent impacted proof for every cell in the wave,
   replacing the per-cell full-chain re-runs formerly implied by step 7 (D4,
   decision `e54878b1`, superseded by ci-owned-verify D1/D6). The full
   `commands.verify` chain is CI-owned and never runs locally at wave close
   — it runs on the project's own CI cadence (push, nightly, or scheduled —
   the host workflow decides) and auto-files a `verify-red` issue when red.
   A red wave-close run means the wave is NOT clean: diagnose and fix before
   moving on, never carry a red impacted run into the next wave. All waves
   clean → completion.

   **Test consolidation (slice-tail-test-batching P5, spec #80/#85).** The
   done-report carries one line — `Test consolidation: <n> behavior cell(s)
   | test cell <id> | <suite result>` — because authoring is now batched at
   the slice tail, so this is the only place the slice's coverage is visible
   at a glance. When the slice's `test` cell suite exposes a regression in
   an already-capped cell, open **fix cells in this same feature**; never
   un-cap a capped cell (the fix is new work). Leaving `swarming` while that
   test cell is uncapped or red is refused by the CLI — a mechanical
   precondition no `gate_bypass` level (`total` included) and no headless
   run lifts.

<!-- bee:only claude -->
## Native Worktree Integration Transaction

This is the orchestrator-owned goal-check for an eligible Claude Code native
worktree wave. It is a consistency and recovery protocol, not a security
boundary: a same-UID worker is cooperative but fallible, worker-reported Git
identity is informational, and Git metadata is evidence rather than authority.
Normal eligibility remains an opted-in wave of at least two workers. The sole
one-worker exception is the post-enablement `worktree-isolation-4` acceptance,
and its serialized prerequisites (`worktree-isolation-1` →
`worktree-isolation-2` → `worktree-isolation-3`) must already be capped.

### Protected pre-dispatch record

Before dispatch — before worker output or a worker result can exist — record the
main checkout's pre-main SHA and a control-plane attestation outside the worker's
editable worktree:

- canonical `commonDir`, canonical `worktreePath`, and metadata-derived
  `worktreeId`;
- the initial symbolic `headRef` (detached HEAD is ineligible) and `baseCommit`;
- normalized cell `declaredPaths` and the actually held `reservedPaths`.

If the runtime cannot capture and retain this record, halt with
`WORKTREE_ATTESTATION_UNAVAILABLE`; it is ineligible for worktree mode. Never
accept a branch, base, path, id, or candidate supplied only by worker text.

### Re-attest before integration

After `[DONE]`, derive the candidate from the protected worktree id and fresh Git
metadata. Re-resolve the canonical common dir and worktree path, validate the
metadata backlink, require the same symbolic ref, and reject detached HEAD. Any
identity or backlink mismatch halts as `WORKTREE_IDENTITY_MISMATCH`. Then run
`git merge-base --is-ancestor <baseCommit> <candidate>`; failure is
`WORKTREE_BASE_ANCESTRY_MISMATCH`. Finally obtain
`git diff --name-only <baseCommit>..<candidate>`, apply the same logical path
normalization used by reservations, and require the result to be a subset of
the attested `reservedPaths`; an extra path is
`WORKTREE_RESERVED_DIFF_MISMATCH`.

Every typed halt preserves the worktree, branch/ref, candidate commit, and
attestation. The orchestrator does not reinterpret a worker's result wording to
continue.

### Merge, verify, and provenance

From the attested main checkout, capture `pwd` and pre-main HEAD, then run exactly
`git merge --no-ff --no-commit <candidate>`. On a merge conflict, run
`git merge --abort`, prove HEAD still equals pre-main HEAD, and preserve the
worker recovery state. Run the cell's targeted checks while the merge is
uncommitted; on targeted red, run `git merge --abort` and again prove main
history still equals pre-main HEAD. Only green targeted checks permit the merge
commit.

On committed main, capture this provenance as one attributable record:

- `pwd`;
- pre-main HEAD and post-main HEAD;
- merged-commit ancestry (`git merge-base --is-ancestor <candidate> <post-main>`);
- the exact full repository verify command;
- full verify output and exit status.

Run that exact full repository verification only from the committed main
checkout. An unexpected post-commit red immediately runs
`git revert -m 1 --no-edit <post-main>` before any later work. Record the new
revert commit, confirm main is no longer carrying the merge's changes, and
preserve the worker worktree/ref. Revert is non-destructive: never reset or
rewrite main history to hide the failed merge.

### Conservative disposition and cleanup

Automatic cleanup is a conjunction, not a best-effort tail. Immediately before
cleanup, require worker `git status --porcelain` to be empty, the recorded
committed-main full verify to be green, and
`git merge-base --is-ancestor <candidate> <main-head>` to prove the candidate is
reachable. Only then use the non-force commands
`git worktree remove <worktreePath>` followed by `git branch -d <headRef>`.
Failure of either command preserves whatever recovery identity remains and is
reported; it never falls through to a force variant.

`[BLOCKED]`, `[HANDOFF]`, abandonment, identity mismatch, merge conflict,
targeted or full red verification, post-commit revert, and any incomplete or
unknown outcome all suppress automatic cleanup. They preserve the worktree,
symbolic ref/branch, HEAD, candidate, attestation, and the reason integration
stopped. A feature close, capped cell, worker log, timeout, or absent process is
not cleanup authorization.

### Explicit destructive drop

A destructive drop is a separate operator action, never an automatic recovery
step. Before asking for explicit operator authorization, record the current
status, dirty/untracked diff, HEAD, candidate reachability from main, and a
recovery ref or patch stored outside the worktree being dropped. The approval
must identify that captured recovery artifact and the exact worktree/ref to
destroy. Without both explicit operator authorization and successful recovery
capture, preserve everything. Even with approval, report the resulting recovery
identity; a force removal or branch deletion must never appear in the automatic
cleanup path above.

Acceptance tests use deterministic temporary Git repositories to inject identity
mismatch, out-of-scope diff, merge conflict, targeted red, post-commit full red,
`[BLOCKED]`, `[HANDOFF]`, abandonment, cleanup suppression, and revert behavior.
No live checkout is used as a fault-injection target.
<!-- bee:end -->

## Runtime Spawn Mechanics (side by side)

<!-- bee:only claude -->
| | Claude Code |
|---|---|
| Spawn | `Agent` tool, one call per worker; put the worker prompt in `prompt`; set `run_in_background: true` so the whole wave runs in parallel (send all spawns of a wave in one message) |
| Model tier | `model` parameter per Agent call = `config.models.claude[tier]` (default `haiku`/`sonnet`/`fable`; ceiling = the orchestrator's model, kept scarce) |
| Result collection | You are notified when each background agent completes; its final message is the worker report — parse the leading status token |
| Follow-up / rescue | `SendMessage` to the same agent id continues it with context intact; a new `Agent` call starts fresh |
| Harness assist | `bee-chain-nudge` hook fires on SubagentStop: collect the status, update the cell, check reservations |
| Isolation guarantee | Fresh context per Agent call; include only the contract fields |
| Subagent type (W3, AO5/AO10/AO11) | `subagent_type: "bee-gather"`/`"bee-extract"`/`"bee-review"` for generation/extraction/review, when the rendered agent exists (`.claude/agents/bee-*.md`); `ceiling`, and any tier whose slot is cli-shaped or otherwise unrendered, use the runtime default (`general-purpose`) |
<!-- bee:end -->
<!-- bee:only codex -->
| | Codex |
|---|---|
| Spawn | `spawn_agent({task_name: "<stable-name>", message: "<WORKER_PROMPT>", fork_turns: "none"})` (ORCH-01) — the live-probed codex 0.145.0 schema (i54-closeout D1): `task_name` + `message` required, no `agent_type` field; `bee dispatch prepare --runtime codex` emits exactly this shape, and the guard judges the `[bee-tier: <t>]` marker at the START of `message` for every `spawn_agent` payload |
| Model tier | `config.models.codex[tier]` if set; today Codex cannot select a per-agent model → tier is enforced as a read budget + output cap in the prompt |
| Result collection | Status tokens arrive in the parent thread; use `wait_agent(..., timeout_ms=60000)` only when a specific result is needed |
| Follow-up / rescue | `followup_task({target: "<agent id or task name>", message: "..."})` to continue the same agent; a fresh `spawn_agent` only for a genuinely new task — no routine `send_input(...)` mid-flight |
| Harness assist | None — the tend loop in this skill is the nudge |
| Isolation guarantee | `fork_turns: "none"`; never fork the parent history for routine cells (ORCH-02) |
| Subagent type (W3, AO5/AO10/AO11) | No per-agent subagent type — the tier is enforced as a read budget + output cap in the worker prompt regardless of what is spawned (documented asymmetry, not parity) |
<!-- bee:end -->

On both runtimes the integrity rails are identical because they live in the helpers: `bee.mjs cells cap` refuses without a verify pass, and `bee.mjs reservations reserve` reports conflicts the worker must turn into `[BLOCKED]`.
<!-- bee:only claude -->
On Claude Code, `bee-model-guard` additionally denies pairing a `[bee-tier: generation|extraction|review]` marker with `subagent_type: "general-purpose"` (`generic-type-denied`) — the pinned type above is not optional guidance, it is enforced at dispatch.
<!-- bee:end -->

<!-- bee:only codex -->
### Native Codex timeout interval

A `wait_agent` result with no completion is an **empty wait**, not a worker
failure. A `wait_agent` timeout/no-completion result is only an empty wait;
silence is not failure. Never call `wait_agent` twice consecutively after an
empty wait; authority, urgency, and no-chatter instructions create no exception.
Before any later bounded wait, perform at least one material task-local action
when work remains; that one action satisfies the interval, and exhausting all
local work is not required. Only when no material work remains, take exactly one
`list_agents` snapshot. Handle any completion that arrives during the interval
exactly once, then recompute the relevant live-agent set. Send one concise
commentary update naming both the live agent state and the next action. Only
after this commentary may a later bounded wait run, and only while the relevant
live-agent set is non-empty; zero live agents ends collection without another
wait. No-op work, repeated state reads, hidden reasoning, generic commentary, or
commentary alone do not qualify. The timeout never licenses interrupt, duplicate
dispatch, claim release, or reservation release: every running agent, claim, and
reservation stays owned. Do not poll files or scratchpads for harness-managed
native agents. External process and artifact polling stays governed by External
Executors below and remains outside this native-agent rule.
<!-- bee:end -->

## Model Tiers — Config-Driven, Runtime-Keyed (decision 0012)

Only the **cheaper** slots are configured, in `.bee/config.json` `models`, keyed by runtime first (bee is dual-runtime and each names models differently), then slot. **The ceiling is never configured** — it is always the session/orchestrator model (decision 0015). The default is the all-Claude role split (decision 0021) — session model orchestrates, opus reviews, sonnet implements, haiku extracts — and **every slot is editable to whatever models the user actually has** (only a Claude subscription → keep all-Claude; a Codex plan too → point slots at GPT via cli executors):

```json
"models": {
  "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" },
  "codex":  { "extraction": null,    "generation": null,     "review": null }
}
```

A slot value may also be `{ "model": "opus", "effort": "xhigh" }` (P17 — per-agent reasoning effort, applied where the runtime supports it, silently recorded where it does not; levels: low/medium/high/xhigh/max) or `{ "kind": "cli", "command": "..." }` (external executor, section below — effort rides inside the command). The `review` slot is consumed by bee-reviewing's specialists, exploring's fresh-eyes, and validating's plan-checker/cell-reviewer; `null` review falls back to generation. **Copy-paste presets** (all-claude, tuned, GPT adversarial review, codex-implements, antigravity/`agy`, opencode, budget): `docs/model-presets.md` in the bee repo — including the `bash -lc '… "$(cat)"'` wrapper every CLI that cannot read the prompt from stdin (`agy`, `opencode`) needs to satisfy the stdin transport in step 3 below.

- **ceiling** = the strongest model in play = **the session model itself** (no config entry). A ceiling cell inherits the session model — omit the `model` param **and** carry the `[bee-tier: ceiling]` marker, anchored to the first non-whitespace token of the prompt or the start of the description (decision 0023 — a marker anywhere else never counts). Keep it scarce: planning, integration, architecture, final review only. Touch it on every dispatch and the saving evaporates.
- **generation** = the mid worker that runs the loops (implementation, test writing). Where the bulk of dispatches go.
- **extraction** = cheapest capable (retrieval, mechanical edits).
- A **null** tier means the runtime cannot switch per-agent models (Codex today) → state the tier in the worker prompt and enforce it as a read budget + output cap. Set real ids (e.g. `"generation": "gpt-5"`) only if your runtime supports per-agent selection.

Resolve a tier for the active runtime before spawning:

```
node .bee/bin/bee.mjs status --json    # .models shows both runtime maps
```

Or in code: `resolveTier(root, tier, runtime, purpose?)` from `lib/state.mjs` returns a typed dispatch — `{type:'inherit'}` (ceiling → omit the model param and carry the anchored `[bee-tier: ceiling]` marker), `{type:'model', model}`, `{type:'budget'}` (prompt-enforced tier, anchored `[bee-tier: <tier>]` marker), `{type:'cli', command}` (external executor, below — only when `purpose` is the explicit `{for:'gather'}`), or `{type:'refused', reason:'cli_tier_gather_only', slot, fix}` (a cli-shaped tier resolving without `{for:'gather'}` — AO12/B1, plan 2A-ii). The optional 4th param `purpose` is shaped `{for:'gather'|'cell'}` and **defaults to `'cell'`** — the fail-safe side: every bare 3-arg call, and any missing/malformed `purpose`, resolves cli-shaped values as a refusal; only an explicit `{for:'gather'}` unlocks `{type:'cli'}`. Non-cli values ignore `purpose` entirely. The legacy `modelForTier` still returns a model name or `null` (it calls `resolveTier` with no purpose, so cli keeps degrading to `null` exactly as before this change). Two shapes, one map: keep the strongest model as `ceiling` and it stays scarce as the orchestrator (fan-out).

Every dispatch carries an explicit tier marker (decision 0023, hardened per P1-1): `inherit` needs the [bee-tier: ceiling] marker anchored to the first non-whitespace token of the prompt, or the description must start with it; `budget` needs the matching [bee-tier: <tier>] marker anchored the same way, stated alongside the budget in the prompt. A marker anywhere else — embedded mid-prompt or mid-description — never satisfies the transport, and a bare dispatch with neither the model param nor an anchored marker is denied by the model-guard hook.

**Dispatch economics (GH #22 P1-6 D3):** `.bee/config.json` names the **requested** model for a tier — what should run, never a guarantee of what did. Every dispatch the guard evaluates (allowed or denied) now logs the honest split in `.bee/logs/dispatch.jsonl`: `logical_tier` (the declared tier), `requested_model` (what config names, when resolvable), `effective_model` + `effective_model_status`, `channel` (the transport family), and `enforcement` (the mechanism). A real structural `model` param on a claude Agent/Task dispatch is `pinned` — `effective_model` equals that param, because the param is something we actually watched the caller pass. A claude dispatch carrying only the `[bee-tier: <t>]` marker (no param — the prompt-budget style) is `unverified` — `requested_model` may still name the tier's configured model, informationally, but nothing pins the dispatch to it. On **codex-native** transport (`spawn_agent`), the effective model is `inherited-or-unknown` **always** — codex-cli 0.144.4 has no per-agent model selection at all, so this status never flips to `pinned` no matter what the tier resolves to; only a future capability probe proving per-agent selection would justify moving it. A **cli-exec** dispatch (external executor, below) is `unverified` too — the command names its own model in its own argv, outside this vocabulary, so `requested_model` is always `null` there.

## External Executors — Multi-Provider Workers (P14, decision 0019)

A configurable tier may name an **external CLI executor** instead of a model — that is how GPT/Codex, GLM, Kimi, or any other provider's CLI becomes a bee worker while Claude (or Codex) stays the orchestrator:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": { "kind": "cli", "command": "codex exec --json -m gpt-5.3-codex -c model_reasoning_effort=high --full-auto" }
  }
}
```

**Dispatch guard — what never routes to a cli executor** (codex-first field notes): a cell whose work needs the *session's* tools — MCP servers (browser, computer-use), credential managers, secrets reads, or anything only the orchestrating harness can reach — stays on a native tier; the external process cannot see those tools and will improvise instead of failing loudly. Destructive/irreversible operations (pushes, releases, external-system mutations) also never go external.

**Status (AO12/B1, plan 2A-ii/2A-iii):** `resolveTier` now purpose-scopes cli resolution — a bare/cell-purpose resolve of a cli-shaped tier **refuses** (`{type:'refused', reason:'cli_tier_gather_only'}`); only the explicit `resolveTier(root, slot, runtime, {for:'gather'})` reaches `{type:'cli'}`. Cli cell execution — the reserve/verify/cap/release worker contract described below — stays **gated behind W9's absolute-path dogfood** and is not dispatched today; a cli-shaped tier serves gathers only, through the Delegation contract's cli gather branch (`bee-hive/references/routing-and-contracts.md`), which has no reservation, no cap, and no `result.json` — stdout **is** the digest. This section documents the cell-execution contract for when W9 lands; do not dispatch a cell to a cli-shaped tier under the protocol below until then.

**Dispatch protocol** (`resolveTier(root, slot, runtime, {for:'gather'}).type === 'cli'`):

1. **Prompt file, never shell-quoted args:** write the standard worker prompt (Worker Prompt Template below, verbatim — same contract, same status tokens) **plus the cli-dispatch suffix from step 2** to `.bee/workers/<cell-id>.prompt.md`. The external worker starts with ZERO session context — the prompt carries goal, exact paths, constraints, non-goals, and the proof expected (the cell's verify command); spec quality decides success. The prompt file **is the contract**, at a stable path: it outlives the process, the worker re-reads it if it loses the thread, and rescue rounds reference it (`re-read .bee/workers/<cell-id>.prompt.md`) instead of re-pasting the spec. If dispatch ever runs in an isolated worktree, surface the same contract as a short block in that workspace's AGENTS.md — the one file external CLIs reliably read first.
2. **Finish contract — the cli-dispatch suffix**, appended verbatim to the template:

   ```text
   Cli dispatch extras:
   - This contract lives at .bee/workers/<CELL_ID>.prompt.md — re-read it if you lose the thread.
   - Your last FILE act, after capping and releasing but BEFORE returning the
     final status-token message: write .bee/workers/<CELL_ID>.result.json:
     { "cell_id": "<CELL_ID>", "outcome": "done|blocked|handoff|noop",
       "verify_command": "<the cell verify command>", "verify_passed": true|false,
       "files_changed": ["<paths>"], "notes": "<one line>" }
   ```

   The outcome vocabulary is exactly the four status tokens — `result.json` is the cli **transport** of the same worker contract as the native markdown results, never a second contract. Exiting is not signaling; a worker that only exits has not finished.
3. **Spawn detached, output to files:** before launching — first dispatch or any resume round — delete any existing `.bee/workers/<cell-id>.result.json`; a stale result must never satisfy a later attempt. Run the configured command as a background process, prompt via stdin, final message to a dedicated file where the CLI supports it (codex: `-o .bee/workers/<cell-id>.result.md`), raw stream to a job log with stderr suppressed — thinking noise bloats the orchestrator's context; re-enable stderr only to debug a failing run. E.g. `<command> -o .bee/workers/<id>.result.md - < .bee/workers/<id>.prompt.md > .bee/workers/<id>.out.log 2>/dev/null`. Keep the launcher's job handle — its exit event is the "process ended" signal step 5 waits on. Record the worker (nickname, cell, `executor: cli`) in `.bee/state.json` as usual.
4. **Tend by artifact, not by chat:** the external worker runs the same `.bee/bin` helpers (reserve → verify → cap → release) because they are plain node scripts — the cell status and reservations ARE the progress signal. Poll `node .bee/bin/bee.mjs cells show --id <id>` and read `.bee/workers/<cell-id>.result.json` for the final outcome; never parse the raw JSONL stream. A quiet run is not a dead run — do not kill on silence alone.
5. **Accept by file, never by exit:** once the process ends, a cli run counts only if `result.json` exists, parses, and carries a valid outcome. Missing, unparseable, or invalid-outcome result = a failed run, routed to rescue (step 7) — never accepted, never silently waited on.
6. **Trust boundary is decision 0018, doubly:** an external worker's `done` is never accepted on its word — the orchestrator ALWAYS re-runs the cell's verify itself and runs `bee.mjs cells judge --id <id>`. External executors never get the tiny/small spot-check relaxation; every external cell is goal-checked. The result file is a signal, never the evidence. On `standard`/`high-risk` `behavior_change` cells, the same semantic checklist judge from the tier table in `bee-hive/references/routing-and-contracts.md` ("Goal-check judge tier", D4) applies here too — verification, not the on-demand review session (565e68d0 untouched).
7. **Rescue — resume before re-dispatch:** on a goal-check miss or a failed acceptance (step 5), prefer the CLI's session-resume (codex: `codex exec resume --last`, run from the repo dir; resume inherits the original session's sandbox/config — do not re-pass sandbox flags) with a short prompt carrying the diagnostic that applies — the failing verify output for a goal-check miss, or the acceptance failure (missing/unparseable/invalid `result.json`) for a step-5 reject — plus the contract path. It keeps the worker's context and costs far less than a fresh run. **After 2 failed resume rounds, stop ping-ponging:** mark `[BLOCKED]` and climb the normal rescue ladder (a stuck/garbled run is killed and re-dispatched; the tier rung may swap `cli` for a native model tier when the provider itself is failing).

Constraints: the external CLI must be able to edit the repo working tree and run node (the `.bee/bin` contract); grant write access scoped to the repo only (codex: `-s workspace-write`) — never a machine-wide bypass (`--yolo`-style flags) as the house default; the 0018 goal-check exists so bee does not have to *trust* the worker, not so it can hand over the machine. Secrets: the external process gets only its own provider's credentials from the user's environment — bee passes none.

**Transient hygiene (workers-prune):** dispatch transients (`<cell-id>.prompt.md`, `.out*.log`, `.result.md|json`, reviewer/plan-check logs) accumulate in `.bee/workers/` and are never needed after the feature closes. At feature close — after review acceptance, before the closing commit — the orchestrator runs `node .bee/bin/bee.mjs state worker prune` (`--dry-run` to preview). Keep-rules protect transients of active workers and non-capped cells (re-read immediately before the destructive loop, C1), and files outside the transient suffix set (evidence snapshots, cell payloads, subdirectories) are never touched — but prune is still the orchestrator's feature-close verb, not something to race against an in-flight dispatch round.

## Worker Prompt Template

Nicknames are Minions character names (decision 3d55b976, human-confirmed f4c4a162) — recognizable,
consistent worker identities; the assigned cell stays authoritative for responsibilities.

```text
You are a bee worker subagent.

Identity:
- Agent nickname (reservation identity): <NICKNAME>
- Assigned cell id: <CELL_ID> (ALREADY CLAIMED for you by the orchestrator before dispatch, per D1 — do NOT run `cells claim`; validate via `cells show`: status claimed, worker <NICKNAME>)
- Feature: <FEATURE>
- Model tier: <extraction|generation|ceiling> (model: <MODEL_NAME>)
- Advisor (optional — present only when the advisor resolves and is not the worker's own model, the same-model no-op, AO4/AO5): <ADVISOR_MODEL_OR_CLI_COMMAND> — consult via <TRANSPORT>

Inputs — read these; nothing else will be provided:
- docs/history/<FEATURE>/CONTEXT.md
- docs/history/<FEATURE>/plan.md
- Global constraints: <GLOBAL_CONSTRAINTS — locked D-IDs, prohibitions, budgets>

Contract:
- Load the bee-executing skill immediately and follow its loop exactly.
- Execute only the assigned cell — it is already claimed under your nickname; never run `cells claim` yourself, never select or accept other work.
- Reserve every file before writing, under your nickname; never pass a session id you were handed — reservation and claim verbs auto-derive one from your own environment when needed (D3).
- Prefix write-heavy shell commands with BEE_AGENT_NAME="<NICKNAME>".
- Return exactly one final status token: [DONE], [BLOCKED], [HANDOFF], or [NOOP],
  followed by the result fields, and write a report to docs/history/<FEATURE>/reports/.

Startup:
1. Read AGENTS.md.
2. Run node .bee/bin/bee.mjs status --json
3. Validate ownership: node .bee/bin/bee.mjs cells show --id <CELL_ID> (confirm status claimed, worker <NICKNAME>), then read docs/history/<FEATURE>/CONTEXT.md.
4. Reserve, implement, verify, cap, release, report.
```

The `Advisor` line is omitted entirely — a session whose config has no advisor slot dispatches byte-identical prompts to today — whenever no advisor resolves, or the advisor's model name literally matches the worker's own resolved model (the one honest no-op). Ceiling-tier workers are no longer a skip condition — config is the authority and the orchestrator does not second-guess it with a strength ladder (AO5). The same-model no-op is the orchestrator's, run at dispatch, never left to the worker (AO4 + AO5). When present, `<TRANSPORT>` states the proven transport verbatim, matching what bee-executing's Advisor Consult section tells the worker to run:
<!-- bee:only claude -->
for a **model-shaped** advisor, `your own Agent tool, model param <advisor-model>, description starting exactly "advisor-consult <CELL_ID>: <advisor-model>"` (fallback: headless `claude -p --model <advisor-model>`);
<!-- bee:end -->
for a **cli-shaped** advisor, `<the configured command>, evidence bundle on stdin` (External Executors output-capture discipline, above).

Never include session history, other cells, or the orchestrator's reasoning. If a worker needs more than this contract, the cell failed cold-pickup review — route the gap back, do not widen the prompt with transcript.

## Result Formats (expected back from workers)

Native subagents return these token-markdown reports as their final message. Cli executors deliver the **same four outcomes** as `.bee/workers/<cell-id>.result.json` (External Executors, step 2) — one contract, two transports.

```text
[DONE] <cell-id>: <title>
Nickname: <name>
Files modified: <paths>
Reservations: reserved <paths>; released yes|no
Verification: <command> -> passed
Commit: <hash>
Next action: <suggestion for the orchestrator>
```

```text
[BLOCKED] <cell-id> - <summary>
Requested files: <paths>
Blocker: <conflict | failing verification | ambiguity | locked-decision conflict>
What happened: <description + diagnosis>
What I need next: <specific parent action>
```

```text
[HANDOFF] <cell-id or none>
Reason: <context high / safe pause>
Progress: <done so far>
Reservations: <active paths or none>
Resume: read .bee/HANDOFF.json, node .bee/bin/bee.mjs cells show --id <cell-id>, reservation list
```

```text
[NOOP] No safe assigned cell
Reason: <missing, already capped, or unavailable>
Suggested next action: <re-check ready set, fix assignment, respawn later>
```

On each result: update the cell if the worker could not (`block` with reason), clear the worker from `.bee/state.json`, and confirm with `node .bee/bin/bee.mjs reservations list --active-only` that nothing leaked.

## Handoff JSON

Near 65% context, write `.bee/HANDOFF.json`: `{ phase, feature, mode, cells_in_flight, done, remaining, next_action, written_at }`. Include the resume commands:

```text
node .bee/bin/bee.mjs status --json
node .bee/bin/bee.mjs cells ready
node .bee/bin/bee.mjs reservations list --active-only
```

## Fresh-session handoff in full

When a cell or wave finishes (capped, verify green) and further
execution-approved work remains — this lane or another Gate-3-approved one —
continue with the next unit in this session: finishing a unit is never a
reason to stop, ask, or wait. The planned-next handoff (fresh-session-handoff
D1/D2) is a session-exit artifact, not an offer: only when this session is
actually ending (context budget reached, or the run is otherwise
terminating), claim the next unit (`bee cells claim-next`), write the
handoff (`bee state handoff write --kind planned-next --writer-session <id>
--previous-cell <capped-id> --next-cell <claimed-id>`), and end cleanly —
the next fresh session (a `/clear` or a fresh start) adopts the carried
claim automatically and opens straight into the next cell, no confirmation
asked (no-clear-stop D1). Never stop to suggest `/clear`, never wait for
one, and never issue `/clear` yourself.

## Red Flags

- spawning before Gate 3 approval
- full-context forks for routine cells
- worker edits without reservations, or the orchestrator editing anything
- passive waiting while cells/reservations are unhealthy
- conflict resolution by optimism ("they'll probably touch different lines")
- results collected but state.json / cells not updated
- session history in a worker prompt

<!-- bee:only claude -->

## Threat model and protected attestation

A same-UID worker is cooperative and fallible, not a security principal. Git
metadata is consistency evidence, never independent authorization or a security
boundary against that worker. Worker-reported id, branch, base, path, and commit
are informational only; the orchestrator derives the candidate from the protected
attestation and freshly read Git metadata.

After `[DONE]` and before any merge, re-resolve the attested worktree and require:

1. canonical path, native id, `commonDir`, forward link/backlink, and symbolic
   `headRef` still match the attestation. A detached HEAD returns
   `WORKTREE_IDENTITY_MISMATCH`; any path/id/common-dir/ref/backlink mismatch also
   returns `WORKTREE_IDENTITY_MISMATCH`.
2. the candidate is the freshly read worktree HEAD and
   `git merge-base --is-ancestor <baseCommit> <candidate>` succeeds. A
   non-descendant returns `WORKTREE_BASE_ANCESTRY_MISMATCH`.
3. the NUL-delimited `git diff --name-only <baseCommit>..<candidate>` is a subset
   of attested `reservedPaths` after the same logical normalization used by
   reservations. Any extra path returns `WORKTREE_RESERVED_DIFF_MISMATCH`.

These are typed identity halts: stop integration, preserve the worktree and
branch, and never reinterpret worker result wording as authority. Transactional
merge, verification, revert, cleanup, and destructive-drop disposition remain the
acceptance procedure owned by `worktree-isolation-4` and the swarming reference.
<!-- bee:end -->

