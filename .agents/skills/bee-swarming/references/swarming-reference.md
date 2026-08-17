# Swarming Reference

Load after Gate 2 approval (merged shape+execution), before spawning the first wave.

## Single execution worker in full

For `small`, the merged Gate 2 shape+execution question and the frozen-judge
check stay with the orchestrator, but implementation itself runs through
**one dispatched execution worker** — a lighter direct Agent dispatch
under the same execution contract as a swarm worker (same worker prompt
template, same status-token protocol, same reservation and cap discipline),
never a full bee-swarming wave: no wave analysis, no reviewers, no panels.
`tiny` may instead execute inline in the orchestrator session — same merged
gate, cap discipline, and done-report; dispatch stays legal and, when
chosen, follows this same contract.
The orchestrator claims the cell itself before spawning — same as any
wave — then spawns it per the Operating Contract's Spawn step (param-carrying
dispatch — a `model` param or a pinned agent type, never a bare marker) and
the Delegation contract's execution-worker class
(`bee-hive/references/gates-and-delegation.md`): it registers in the swarm
registry (`state worker add`), validates the claim it was handed (against
the inlined cell JSON in its prompt — never `cells claim`)
and takes reservations under its own nickname,
reads its `read_first`, implements within its `files`, commits, and
finishes it (`cells finish` — commit-only proof, caps and releases the
reservations in the same verb; tests prove at the boundary: `bee close`
runs `commands.test` when the feature has no worktree, `bee worktree
merge` runs it when it does). Then it returns exactly one status token.

**Default — parallel:** a `small` lane's cells (1-3) fan
out to concurrent execution workers whenever every cell's *product* file set
is disjoint — reservations are the proof and the police (the guard denies an
overlap; the worker count is a default, ~3-4 live). Serial is the exception and
carries a named conflict in the dispatch note. `tiny` stays
single-cell by shape, so the concurrency question does not arise; `small`'s
extra cells scale the WORK and, when disjoint, the concurrency too — never
concurrency wearing an unrecorded conflict. Two or more live small-lane
workers with an undeclared overlap is a wave shape wearing a `small` lane —
the ceremony mismatch lane scaling exists to catch.

**Disjointness and the wave-barrier regen protocol:** a cell's *effective*
file set for the disjointness check excludes shared generated artifacts
(release manifest, onboarding ledger, plugin mirrors) whenever the cell
carries `regen_obligation_ack: "wave-barrier"` — the cell itself skips
in-cell regen, and the ORCHESTRATOR owes the full regen chain (mirror
render → `onboard --apply` → `manifest --write`/`--check`) exactly ONCE at
wave close, folded into the wave-close/close commit, before the wave is
declared clean. This is what lets the scheduler's overlap check see truly
disjoint product sets instead of serializing every cell on a shared
generated file. Any *product* file actually shared (not just a regen
target) still forces serial — in doubt, serial.

After `[DONE]`, emit the cap tick, and when `ship_visibility` is active push
the cap (first cap of a feature opens the draft PR) —
`bee-hive/references/scout-and-ticks.md`, "Progress ticks" / "Ship
visibility". Then — never the worker — author the done-report from the
worker's verbatim diff plus the commit (the finish is commit-only proof;
tests prove at the boundary, step 7 below), including the slice's demo
artifact when one is owed. `tiny`/`small`'s one slice is also the feature's
FINAL slice: tests prove at the boundary — close it with `bee close`,
which runs `commands.test` when the feature has no worktree, or
`bee worktree merge`, which runs it when the feature has one ("Tests at
finish and close, in full", below). Then hand
off: both `tiny` and `small` present that done-report (diff + commit +
test result + capture line) and invoke bee-capturing — no auto
reviewer; the 1-correctness-reviewer contract lives inside a user-invoked
session (implementation is verified; independent review runs only on user
request).

The rest of this reference and the body's Operating Contract are the
multi-worker wave protocol for `standard`/`high-risk`; a tiny/small dispatch
borrows only its Spawn, tier-judgment, Record, and Goal-check steps for its
single worker — never wave analysis or multi-cell assignment.

## Operating Contract in full

1. **Wave analysis.** Run `.bee/bin/bee cells schedule --json`: the
   computed waves are the **default** dispatch order — an override carries a
   stated reason in the swarm report. Refuse to dispatch when
   diagnostics report cycles. Two ready cells sharing a file means fix the
   reservations or split the cell scope — never "spawn both and be careful";
   the schedule already auto-serializes file overlap into a later wave
   rather than refusing it. The schedule computation and verify-output
   capture delegate as extraction-tier I/O workers per the Delegation
   contract (`bee-hive/references/gates-and-delegation.md`);
   judgment (assignment, tier choice, goal-check verdicts, override
   decisions) stays on the orchestrator.
2. **Assign and claim first.** The orchestrator picks exactly **one
   cell per worker**, then claims it itself — `cells claim-next` or `cells
   claim --id <id> --worker <nickname>` — before spawning; `--session-id` is
   optional and self-derives from `CLAUDE_CODE_SESSION_ID` when omitted.
   A worker validates the claim it was handed (`cells show`) and takes no
   second cell — the claim guard refuses a worker that tries.
3. **Spawn with the isolation contract.** Each worker prompt contains: the
   cell id (already claimed under the worker's nickname per step 2), the
   path to `docs/history/<feature>/CONTEXT.md`, and — when the lane has one
   — `docs/history/<feature>/plan.md`; for `tiny`/`small` (no `plan.md`)
   cite the cell itself as the work spec instead. Also include the
   global constraints, its reservation identity (agent nickname), and the
   status-token protocol (`[DONE] [BLOCKED] [HANDOFF] [NOOP]`) — **nothing
   else, never session history, never a literal session id**. Use the
   template below.
   Codex has no per-agent `subagent_type` equivalent — its tier is
   enforced as a read budget + output cap only.
   Default: bee's own agent types only. A same-named type from another plugin
   carries a different contract and makes the run depend on what is installed.
4. **Judge each cell's model tier at dispatch** — you (the orchestrator)
   assess the task in front of you and pick the fitting tier; it is NOT
   fixed by planning (a planning `tier` is at most a hint you may
   override). Rubric from the cell's lane + action + must_haves +
   files:
   - **extraction** — pure retrieval or mechanical edits: rename, reformat,
     move a file, a one-line change, no design judgment.
   - **generation** — normal implementation, wiring, writing tests: the
     default for most cells.
   - **ceiling** — integration across modules, architecture/design calls,
     security-sensitive or `high-risk`-lane work, ambiguous specs,
     cross-cutting change: where a wrong call is expensive.

   Record the choice so scarcity stays measurable (`cells tier` refuses a
   ceiling choice over budget): `.bee/bin/bee cells tier --id <id> --tier <tier>`. Then resolve with `resolveTier(root,
   tier, runtime)` — full semantics, tier-marker anchoring, and dispatch
   economics: "Model Tiers — Config-Driven, Runtime-Keyed" below. Keep
   `ceiling` scarce — if `bee_status` flags ceiling scarcity, re-judge
   routine cells downward before spawning.

   **After the tier choice, resolve the advisor slot for this dispatch**:
   `resolveAdvisor(root, runtime)`. The configured advisor IS the
   advisor — no family test, no strength test, no self-judged skip;
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
     the worker contract's Advisor Consult section (references/worker-details.md) tells the worker to run.
5. **Record workers** before results arrive. A claim made through
   `bee dispatch prepare --claim` registers its worker automatically
   (same record `state worker add` writes; the payload's
   `worker_registered` says so — cell dpr-1). Manual `.bee/bin/bee state
   worker add --nickname <n> --cell <id> --tier <tier> --status <status>`
   remains only for inline runs and claims made without the
   preparation step.
6. **Tend** the swarm: collect status tokens, update cells and state, verify
   reservations were released. Silence is not failure — inspect cell status
   and `.bee/bin/bee reservations list --active-only` before
   assuming a worker is stuck. Default: no routine mid-flight pings — interrupt
   for an explicit user abort or a confirmed deadlock.
   Native Codex empty waits require a progress interval before the next
   wait: the full ordered rule lives in `bee-hive` → `references/gates-and-delegation.md` ("Native Codex subagent tending").
   External process and artifact polling stays outside it, under the
   separate executor contract below.
7. **Goal-check every `[DONE]` yourself — miss reruns, hit ships.** A
   worker's word is never the evidence; the orchestrator
   measures before the cell counts:
   - **Read the record; re-run only on smell.** Tests prove at the
     boundary: `bee close` runs `commands.test` when the feature has no
     worktree; `bee worktree merge` runs it when it does. A cap is
     commit-only proof and records `tests: boundary`; once the boundary has
     run, `.bee/logs/test-results.json` is the evidence, and quoting it
     satisfies the fresh-output rule. Re-run `bee test` yourself
     on a smell — a missing/garbled report, a `[DONE]` with no diff, a
     `high-risk`/hard-gate cell. Orchestrator judgment, not a routine step. Failure on a spot-check → the cell is NOT done: re-dispatch to
     the same tier with the failing excerpt (a task miss is a rerun, never a
     silent tier escalation — provider errors, not task errors, are what the
     rescue ladder's tier rung is for).
   - **Frozen judge:** `.bee/bin/bee cells judge --id <id>`. Hits
     (undeclared test/CI/lockfile/verify-config changes) → the cell never
     auto-counts toward a clean wave: record the hits in the cell trace and
     carry them into any review session that later covers this scope, and
     ask the worker's diff to justify each file or re-dispatch with
     corrected scope. A worker that rewrites the test is not passing the
     test.
   - **Semantic judge, `standard`/`high-risk` only:** ONE checklist-judge dispatch per SLICE at
     slice close, covering every capped `behavior_change` cell of that
     slice. `high-risk`: every slice. `standard`: selective — dispatch on
     smell, on a worker's/model's first slice of the feature, or on the
     ~1-in-3 sample (choice stated in the slice-close tick, never a silent
     skip); any `NEEDS_REVISION` escalates that worker to judge-every-slice
     for the feature's remainder (tier table in
     `bee-hive/references/gates-and-delegation.md`, "Goal-check judge
     tier"). The judge returns one verdict per cell and each verdict is
     recorded with `cells judge-record` — per-cell records and cap teeth
     unchanged; a single-cell slice is identical to the old per-cell shape.
     This is goal-check verification, distinct from the no-auto-reviewer
     stance above and from any user-invoked review session (Gate 3 and the
     candidates ledger are separate) —
     `NEEDS_REVISION`/`automatic` means the cell is NOT done yet.
   - A `[DONE]` report carrying a **Consults** section is goal-checked
     exactly like any other — advice never substitutes for fresh verify
     output; re-run the verify yourself regardless of what the advisor said.
8. **Wave clean → next wave.** A wave is clean once every
   cell is capped, goal-checked, and judge-intact (or
   explicitly flagged and carried to review). Smell-triggered spot checks
   (step 7) stay the orchestrator's judgment call. All waves clean →
   completion; tests prove at the boundary: the feature's final slice
   closes through `bee close`, which runs `commands.test` when the feature
   has no worktree, or `bee worktree merge`, which runs it when the
   feature has one ("Tests at finish and close, in full", below).

## Tests at finish and close, in full

One declared test path, one result record — tests prove at the boundary
only (decision `13ce1858`, test-cadence-boundary D1; supersedes the
proof-economy tier system, decision 412e9b3a,
`docs/knowledge/areas/verify-pipeline/`, 2026-07-31).

> Tests prove at the boundary: `bee close` runs `commands.test` when the
> feature has no worktree; `bee worktree merge` runs it when it does. A
> cap is commit-only proof and records `tests: boundary`. CI runs the
> same command on every push.

- **Declaration:** `.bee/config.json` `commands.test` is the single place
  a project declares how it is tested (string or array of commands) — the
  ONE command every boundary door runs. Nothing else
  declares test obligations.
- **Runner:** `bee test` runs the declared commands in order and writes
  ONE normalized record, `.bee/logs/test-results.json` — `{ran_at, green,
  commands: [{command, exit, duration_ms, failure_excerpt, failure_log}]}`. The runner
  is a program; an agent's word is never the record.
- **At finish:** `bee finish` is commit-only proof — it does not run
  `commands.test`. A cap in a declared-test repo records `tests: boundary`;
  a repo with no declared `commands.test` caps with `tests: undeclared`.
- **At close:** when the feature has no worktree, `bee close` runs the
  full declared suite fresh — green caps the close doors' test side; red
  is surfaced with the failing excerpt and becomes fix cells in the SAME
  feature (never un-cap a capped cell — the fix is new work). Per-cell
  commits + `git bisect` localize a regression across the feature's cells.
  When the feature has a worktree (including one kept pending-cleanup
  after a merge), close defers — tests prove at `bee worktree merge`
  instead.
- **Merge:** `bee worktree merge` runs `commands.test` against the
  staged merge — the last local net. The estate beyond that is CI-owned,
  running the same command on every push.
- **Never build on red:** a red result at close or merge is the next work
  item, never a base. Re-dispatch prompts (Prior rounds) cite the
  `failure_excerpt` directly.


## Runtime Spawn Mechanics (side by side)

| | Codex |
|---|---|
| Spawn | `spawn_agent({task_name: "<stable-name>", message: "<WORKER_PROMPT>", fork_turns: "none"})` — the codex 0.145.0 schema: `task_name` + `message` required, no `agent_type` field; `bee dispatch prepare --runtime codex` emits exactly this shape, and the guard judges the `[bee-tier: <t>]` marker at the START of `message` for every `spawn_agent` payload |
| Model tier | `config.models.codex[tier]` if set; today Codex cannot select a per-agent model → tier is enforced as a read budget + output cap in the prompt |
| Result collection | Status tokens arrive in the parent thread; use `wait_agent(..., timeout_ms=60000)` only when a specific result is needed |
| Follow-up / rescue | `followup_task({target: "<agent id or task name>", message: "..."})` to continue the same agent; a fresh `spawn_agent` only for a genuinely new task — no routine `send_input(...)` mid-flight |
| Harness assist | None — the tend loop in this skill is the nudge |
| Isolation guarantee | `fork_turns: "none"`; never fork the parent history for routine cells |
| Subagent type | No per-agent subagent type — the tier is enforced as a read budget + output cap in the worker prompt regardless of what is spawned (documented asymmetry, not parity) |

On both runtimes the integrity rails are identical because they live in the helpers: tests prove at the boundary — `bee close`/`bee worktree merge` refuse while the declared tests are red — and `bee reservations reserve` reports conflicts the worker must turn into `[BLOCKED]`.

## Model Tiers — Config-Driven, Runtime-Keyed

Only the **cheaper** slots are configured, in `.bee/config.json` `models`, keyed by runtime first (bee is dual-runtime and each names models differently), then slot. **The ceiling is never configured** — it is always the session/orchestrator model. The default is the all-Claude role split — session model orchestrates, opus reviews, sonnet implements, haiku extracts — and **every slot is editable to whatever models the user actually has** (only a Claude subscription → keep all-Claude; a Codex plan too → point slots at GPT via cli executors):

```json
"models": {
  "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" },
  "codex":  { "extraction": null,    "generation": null,     "review": null }
}
```

A slot value may also be `{ "model": "opus", "effort": "xhigh" }` (per-agent reasoning effort, applied where the runtime supports it, silently recorded where it does not; levels: low/medium/high/xhigh/max) or `{ "kind": "cli", "command": "..." }` (external executor, section below — effort rides inside the command). The `review` slot is consumed by bee-reviewing's specialists, exploring's fresh-eyes, and bee-planning's merged reviewer (the review wave — Structure + cold-pickup cell review); `null` review falls back to generation. **Copy-paste presets** (all-claude, tuned, GPT adversarial review, codex-implements, antigravity/`agy`, opencode, budget): `docs/model-presets.md` in the bee repo — including the `bash -lc '… "$(cat)"'` wrapper every CLI that cannot read the prompt from stdin (`agy`, `opencode`) needs to satisfy the stdin transport in step 3 below.

- **ceiling** = the strongest model in play = **the session model itself** (no config entry). A ceiling cell inherits the session model — omit the `model` param **and** carry the `[bee-tier: ceiling]` marker, anchored to the first non-whitespace token of the prompt or the start of the description (a marker anywhere else never counts). Keep it scarce: planning, integration, architecture, final review only. Touch it on every dispatch and the saving evaporates.
- **generation** = the mid worker that runs the loops (implementation, test writing). Where the bulk of dispatches go.
- **extraction** = cheapest capable (retrieval, mechanical edits).
- A **null** tier means the runtime cannot switch per-agent models (Codex today) → state the tier in the worker prompt and enforce it as a read budget + output cap. Set real ids (e.g. `"generation": "gpt-5"`) only if your runtime supports per-agent selection.

Resolve a tier for the active runtime before spawning:

```
.bee/bin/bee status --json    # .models shows both runtime maps
```

Or in code: `resolveTier(root, tier, runtime, purpose?)` returns a typed dispatch — `{type:'inherit'}` (ceiling → omit the model param and carry the anchored `[bee-tier: ceiling]` marker), `{type:'model', model}`, `{type:'budget'}` (prompt-enforced tier, anchored `[bee-tier: <tier>]` marker), `{type:'cli', command}` (external executor, below — only when `purpose` is the explicit `{for:'gather'}`), or `{type:'refused', reason:'cli_tier_gather_only', slot, fix}` (a cli-shaped tier resolving without `{for:'gather'}`). The optional 4th param `purpose` is shaped `{for:'gather'|'cell'}` and **defaults to `'cell'`** — the fail-safe side: every bare 3-arg call, and any missing/malformed `purpose`, resolves cli-shaped values as a refusal; only an explicit `{for:'gather'}` unlocks `{type:'cli'}`. Non-cli values ignore `purpose` entirely. `modelForTier` returns a model name or `null` (it calls `resolveTier` with no purpose, so cli degrades to `null`). Two shapes, one map: keep the strongest model as `ceiling` and it stays scarce as the orchestrator (fan-out).

Every dispatch carries an explicit tier marker: `inherit` needs the [bee-tier: ceiling] marker anchored to the first non-whitespace token of the prompt, or the description must start with it; `budget` needs the matching [bee-tier: <tier>] marker anchored the same way, stated alongside the budget in the prompt. A marker anywhere else — embedded mid-prompt or mid-description — never satisfies the transport, and a bare dispatch with neither the model param nor an anchored marker is denied by the model-guard hook.

**Dispatch economics:** `.bee/config.json` names the **requested** model for a tier — what should run, never a guarantee of what did. Every dispatch the guard evaluates (allowed or denied) logs the honest split in `.bee/logs/dispatch.jsonl`: `logical_tier` (the declared tier), `requested_model` (what config names, when resolvable), `effective_model` + `effective_model_status`, `channel` (the transport family), and `enforcement` (the mechanism). A real structural `model` param on a claude Agent/Task dispatch is `pinned` — `effective_model` equals that param, because the param is something we actually watched the caller pass. A claude dispatch carrying only the `[bee-tier: <t>]` marker (no param — the prompt-budget style) is `unverified` — `requested_model` may still name the tier's configured model, informationally, but nothing pins the dispatch to it. On **codex-native** transport (`spawn_agent`), the effective model is `inherited-or-unknown` **always** — codex-cli 0.145.0 has no per-agent model selection at all, so this status never flips to `pinned` no matter what the tier resolves to; only a future capability probe proving per-agent selection would justify moving it. A **cli-exec** dispatch (external executor, below) is `unverified` too — the command names its own model in its own argv, outside this vocabulary, so `requested_model` is always `null` there.

## External Executors — Multi-Provider Workers

A configurable tier may name an **external CLI executor** instead of a model — that is how GPT/Codex, GLM, Kimi, or any other provider's CLI becomes a bee worker while Claude (or Codex) stays the orchestrator:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": { "kind": "cli", "command": "codex exec --json -m gpt-5.3-codex -c model_reasoning_effort=high --full-auto" }
  }
}
```

**Dispatch guard — what never routes to a cli executor:** a cell whose work needs the *session's* tools — MCP servers (browser, computer-use), credential managers, secrets reads, or anything only the orchestrating harness can reach — stays on a native tier; the external process cannot see those tools and will improvise instead of failing loudly. Destructive/irreversible operations (pushes, releases, external-system mutations) also never go external.

**Status:** `resolveTier` purpose-scopes cli resolution — a bare/cell-purpose resolve of a cli-shaped tier **refuses** (`{type:'refused', reason:'cli_tier_gather_only'}`); only the explicit `resolveTier(root, slot, runtime, {for:'gather'})` reaches `{type:'cli'}`. Cli cell execution — the reserve/verify/cap/release worker contract described below — is not dispatched today; a cli-shaped tier serves gathers only, through the Delegation contract's cli gather branch (`bee-hive/references/gates-and-delegation.md`), which has no reservation, no cap, and no `result.json` — stdout **is** the digest. This section documents the cell-execution contract for when that path is enabled; until then, do not dispatch a cell to a cli-shaped tier under the protocol below.

The cell-execution protocol for that path — prompt file, finish contract,
detached spawn, artifact tending, acceptance, trust boundary, rescue — is
`docs/history/cli-executors-cell-path.md`. It stays out of this reference
while `resolveTier` refuses a cli-shaped tier for anything but a gather.

**Transient hygiene:** dispatch transients (`<cell-id>.prompt.md`, `.out*.log`, `.result.md|json`, reviewer/plan-check logs) accumulate in `.bee/workers/` and are never needed after the feature closes. At feature close — after review acceptance, before the closing commit — the orchestrator runs `.bee/bin/bee state worker prune` (`--dry-run` to preview). Keep-rules protect transients of active workers and non-capped cells (re-read immediately before the destructive loop), and files outside the transient suffix set (evidence snapshots, cell payloads, subdirectories) are never touched — but prune is still the orchestrator's feature-close verb, not something to race against an in-flight dispatch round.

## Worker Prompt Template

Nicknames are Minions character names — recognizable,
consistent worker identities; the assigned cell stays authoritative for responsibilities.

```text
You are a bee worker subagent.

Identity:
- Agent nickname (reservation identity): <NICKNAME>
- Assigned cell id: <CELL_ID> (ALREADY CLAIMED for you by the orchestrator before dispatch — do NOT run `cells claim`; validate against the inlined cell JSON below)
- Feature: <FEATURE>
- Model tier: <extraction|generation|ceiling> (model: <MODEL_NAME>)
- State at dispatch: phase=<PHASE> feature=<FEATURE> gates.execution=<BOOL> (copied from the orchestrator's own fresh read; the worker never re-fetches the full payload for this)
- Advisor (optional — present only when the advisor resolves and is not the worker's own model, the same-model no-op): <ADVISOR_MODEL_OR_CLI_COMMAND> — consult via <TRANSPORT>

Inputs — read these; nothing else will be provided:
- docs/history/<FEATURE>/CONTEXT.md
- docs/history/<FEATURE>/plan.md
- Global constraints: <GLOBAL_CONSTRAINTS — locked D-IDs, prohibitions, budgets>
- Your cell (inlined verbatim from the orchestrator's claim-time read — authoritative):
  <CELL_JSON — the full .bee/cells/<CELL_ID>.json content>

Contract:
- Load the bee-swarming skill (Execute section) and follow its loop exactly.
- Execute only the assigned cell — it is already claimed under your nickname; never run `cells claim` yourself, never select or accept other work.
- Reserve every file before writing, under your nickname; never pass a session id you were handed — reservation and claim verbs auto-derive one from your own environment when needed.
- Prefix write-heavy shell commands with BEE_AGENT_NAME="<NICKNAME>".
- Return exactly one final status token: [DONE], [BLOCKED], [HANDOFF], or [NOOP],
  followed by the result fields. Report file only for [BLOCKED]/[HANDOFF]/consult-carrying
  cells — a routine [DONE] relies on the cap trace + this message.

Startup (two reads, zero CLI round-trips):
1. Read AGENTS.md.
2. Read docs/history/<FEATURE>/CONTEXT.md. Validate ownership against the INLINED cell JSON above (status claimed, worker <NICKNAME>) — never re-run `status --brief` or `cells show` at startup; the dispatch state line and inlined cell are authoritative, and ownership is re-enforced at cap by the claim guard. A prompt missing the inlined cell JSON is malformed → [BLOCKED].
3. Reserve, implement, commit, finish (commit-only proof; tests prove at the boundary — close or merge), report.
```

The `Advisor` line is omitted entirely — a session whose config has no advisor slot dispatches byte-identical prompts to today — whenever no advisor resolves, or the advisor's model name literally matches the worker's own resolved model (the one honest no-op). Ceiling-tier workers are not a skip condition — config is the authority and the orchestrator does not second-guess it with a strength ladder. The same-model no-op is the orchestrator's, run at dispatch, never left to the worker. When present, `<TRANSPORT>` states the proven transport verbatim, matching what the worker contract's Advisor Consult section (references/worker-details.md) tells the worker to run:
for a **cli-shaped** advisor, `<the configured command>, evidence bundle on stdin` (External Executors output-capture discipline, above).

Default: no session history, no other cells, no orchestrator reasoning. A worker that needs more than this contract means the cell failed cold-pickup review — route the gap back rather than widen the prompt with transcript.

## Result Formats (expected back from workers)

Native subagents return these token-markdown reports as their final message. Cli executors deliver the **same four outcomes** as `.bee/workers/<cell-id>.result.json` (External Executors, step 2) — one contract, two transports.

```text
[DONE] <cell-id>: <title>
Nickname: <name>
Files modified: <paths>
Reservations: reserved <paths>; released yes|no
Tests: green (finish run — .bee/logs/test-results.json) | undeclared
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
Resume: read .bee/HANDOFF.json, .bee/bin/bee cells show --id <cell-id>, reservation list
```

```text
[NOOP] No safe assigned cell
Reason: <missing, already capped, or unavailable>
Suggested next action: <re-check ready set, fix assignment, respawn later>
```

On each result: update the cell if the worker could not (`block` with reason), clear the worker from `.bee/state.json`, and confirm with `.bee/bin/bee reservations list --active-only` that nothing leaked.

## Handoff JSON

Near 65% context, write `.bee/HANDOFF.json`: `{ phase, feature, mode, cells_in_flight, done, remaining, next_action, written_at }`. Include the resume commands:

```text
.bee/bin/bee status --json
.bee/bin/bee cells ready
.bee/bin/bee reservations list --active-only
```

## Fresh-session handoff in full

When a cell or wave finishes (capped, tests green) and further
execution-approved work remains — this lane or another Gate-3-approved one —
continue with the next unit in this session: finishing a unit is never a
reason to stop, ask, or wait. The planned-next handoff is a session-exit
artifact, not an offer: only when this session is
actually ending (context budget reached, or the run is otherwise
terminating), claim the next unit (`bee cells claim-next`), write the
handoff (`bee state handoff write --kind planned-next --writer-session <id>
--previous-cell <capped-id> --next-cell <claimed-id>`), and end cleanly —
the next fresh session (a `/clear` or a fresh start) adopts the carried
claim automatically and opens straight into the next cell, no confirmation
asked. Never stop to suggest `/clear`, never wait for
one, and never issue `/clear` yourself.

## Red Flags

- spawning before Gate 2 approval
- full-context forks for routine cells
- worker edits without reservations, or the orchestrator editing anything
- passive waiting while cells/reservations are unhealthy
- conflict resolution by optimism ("they'll probably touch different lines")
- results collected but state.json / cells not updated
- session history in a worker prompt
