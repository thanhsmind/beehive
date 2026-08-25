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
finishes it — `cells finish` requires the `--report` proof line, caps the
cell, and releases the reservations in the same verb, and the boundary
doors check that recorded proof (rule: agents-proof-at-cap). Then it
returns exactly one status token.

**Default — parallel:** a `small` lane's cells (1-3) fan out to
concurrent execution workers whenever every cell's *product* file set is
disjoint — reservations are the proof and the police (the guard denies an
overlap; the worker count is a default, ~3-4 live). Serial is the
exception and carries a named conflict in the dispatch note. `tiny` stays
single-cell by shape, so the concurrency question does not arise;
`small`'s extra cells scale the WORK and, when disjoint, the concurrency
too — never concurrency with an unrecorded conflict. Two or more live
small-lane workers with an undeclared overlap is a lane mismatch: a wave
shape run under a `small` lane.

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
worker's verbatim diff plus the commit (the finish carries the proof line
in its `--report`, step 7 below), including the slice's demo
artifact when one is owed. `tiny`/`small`'s one slice is also the feature's
FINAL slice: close it with `bee close`, or `bee worktree merge` when the
feature has one — both check every capped cell's recorded proof instead
of running tests themselves ("Proof at finish and close, in full",
below). Then hand
off: both `tiny` and `small` present that done-report (diff + commit +
test result + capture line) and invoke bee-capturing — no auto
reviewer; the 1-correctness-reviewer contract lives inside a user-invoked
session (implementation is verified; independent review runs only on user
request).

The rest of this reference and the body's Operating Contract are the
multi-worker wave protocol for `standard`/`high-risk`; a tiny/small dispatch
borrows only its Spawn, role-and-escalation judgment, Record, and Goal-check
steps for its single worker — never wave analysis or multi-cell assignment.

## Operating Contract in full

1. **Wave analysis.** Run `.bee/bin/bee cells schedule --json`: the
   computed waves are the **default** dispatch order — an override carries a
   stated reason in the swarm report. Refuse to dispatch when
   diagnostics report cycles. Two ready cells sharing a file means fix the
   reservations or split the cell scope — never "spawn both and be careful";
   the schedule already auto-serializes file overlap into a later wave
   rather than refusing it. The schedule computation and verify-output
   capture delegate as read-job I/O workers per the Delegation
   contract (`bee-hive/references/gates-and-delegation.md`);
   judgment (assignment, a role override, escalation, goal-check verdicts,
   override decisions) stays on the orchestrator.
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
<!-- bee:only claude -->
   **Resolve through the one door, then run exactly what it returns.**
   `.bee/bin/bee dispatch prepare --runtime claude --kind
   <cell|gather|reviewer|advisor> [--role <name>] --json` returns the
   Agent/Bash payload to run (per D1). `--role <name>` names the JOB the
   dispatch is and overrides the slot the kind would have resolved —
   `--kind gather --role extraction` is how a read-only gather reaches the
   cheap reader. Any name `models.<runtime>` carries is legal; a `--role`
   naming a name nothing configures is refused by name, with a FIX listing
   the roles this runtime resolves and ending "Any role name you configure
   is legal; bee holds no fixed list" — never silently resolved.
   `subagent_type: "bee-build"` (cell execution),
   `"bee-gather"` (a read-only gather),
   `"bee-extract"` (the cheap reader), `"bee-review"` (review) are bee's
   own rendered agent definitions (`.claude/agents/bee-*.md`,
   config-sourced at onboarding) and appear ONLY as what `prepare`
   returns — never type one by hand, and never another plugin's type.
   An escalated cell has no rendered agent (it runs on the session model);
   `prepare` returns the runtime's default/general subagent type for it. A
   slot that resolves to no model (a cli- or herding-shaped slot) is not a
   subagent at all — `prepare`'s payload carries that transport too.
   NEVER pair a `[bee-tier: <role>]` marker with
   `subagent_type: "general-purpose"` — `bee-model-guard` denies it
   (`generic-type-denied`) precisely so this rule cannot be skipped by
   habit.
<!-- bee:end -->
<!-- bee:only codex -->
   Codex has no per-agent `subagent_type` equivalent — its role is
   enforced as a read budget + output cap only.
<!-- bee:end -->
   Default: bee's own agent types only. A same-named type from another plugin
   carries a different contract and makes the run depend on what is installed.
4. **The cell already names the job — read it, override only with a reason,
   escalate only where the work earns it.** `role` is REQUIRED on every cell
   (`bee cells add` refuses without it) and it is the cell's sole model
   selector. The question it answers is *what job is this work*, never *how
   expensive is this work*: some models plan well, some test well, some code
   well, and the host's config is what says which. Planning writes the role;
   you may override it at dispatch with a stated reason, exactly as you may
   override the schedule.

   The recommended vocabulary is **authoring guidance, never an enum** —
   `code`, `read`, `test`, `docs`, `review`, `design`. **Any non-empty name
   is legal**: bee validates a role's presence and shape, never its
   membership, so a repo that configures `migrate` or `sql` gets that model
   with no bee change. Name the job from the cell's action + files:
   - **`code`** — the work writes product code, wiring or tests. The
     default for most cells.
   - **`read`** — the work only reads: retrieval, tracing a call path,
     mining a transcript, an evidence digest.
   - **`test`, `docs`, `design`, `review`, or a name this repo invented** —
     the work is that job, and `models.<runtime>` decides which model is
     good at it.

   The dispatch asks for an ORDERED LIST headed by the cell's own role and
   ending in a name every host has configured for years —
   `[<cell role>, code, generation]`, or `[read, extraction, generation]`
   for a read job. **An unconfigured role never fails**: it yields to the
   next name in the list and WARNS on stderr, naming what it fell through
   to — silent only for `code` or `read` on a runtime whose
   `models.<runtime>` configures neither of them, the pre-roles window.
   That tail is why this costs no host its current model.

   **Escalation is a separate lever, not a role name.** Integration across
   modules, architecture/design calls, security-sensitive or
   `high-risk`-lane work, ambiguous specs, cross-cutting change — where a
   wrong call is expensive — run on the SESSION model:
   `.bee/bin/bee cells escalate --id <id>` sets the flag the dispatch reads
   (no `model` param, the `[bee-tier: ceiling]` marker instead), and
   `--off` puts the cell back on its role's model. The door keeps the old
   ration teeth: escalating past 40% of the feature's cells refuses unless
   `--reason "<text>"` names why, and that reason persists on the cell trace
   as `escalation_reason`. Keep it scarce — `bee status --json` reports
   `role_mix` (the rename of `tier_mix`) with the escalated share beside it,
   and the preamble warns when the share runs high; re-judge routine cells
   off the session model before spawning.

   Full resolution semantics, marker anchoring, and dispatch economics:
   "Model Roles — Config-Driven, Runtime-Keyed" below.

   **After the role resolves, resolve the advisor slot for this dispatch**:
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
   - Otherwise → **always** add the `Advisor` line, escalated workers
     included — config is the authority, the orchestrator does not
     second-guess it.
   - When it passes, the `Advisor` line names the advisor identity and
     states its proven transport verbatim (model-shaped vs cli-shaped, per
     the Worker Prompt Template below) — this must match what
     the worker contract's Advisor Consult section (references/worker-details.md) tells the worker to run.
5. **Record workers** before results arrive. `bee dispatch prepare --kind
   cell` registers its worker automatically — with `--claim`, on the fresh
   claim it just took (cell dpr-1); without it, on a cell `--worker`
   already owns (cell dpr-2) — same record `state worker add` writes,
   and the payload's `worker_registered` says so either way. An ownership
   refusal registers nothing. Manual `.bee/bin/bee state worker add
   --nickname <n> --cell <id> --tier <role> --status <status>` remains
   only for inline runs and claims made without the preparation step —
   the FLAG keeps its historical `--tier` spelling, the VALUE is the role
   the dispatch resolved (any non-blank name; membership is never asked).
6. **Tend** the swarm: collect status tokens, update cells and state, verify
   reservations were released. Silence is not failure — inspect cell status
   and `.bee/bin/bee reservations list --active-only` before
   assuming a worker is stuck. Default: no routine mid-flight pings — interrupt
   for an explicit user abort or a confirmed deadlock.
<!-- bee:only codex -->
   Native Codex empty waits require a progress interval before the next
   wait: the full ordered rule lives in `bee-hive` → `references/gates-and-delegation.md` ("Native Codex subagent tending").
   External process and artifact polling stays outside it, under the
   separate executor contract below.
<!-- bee:end -->
7. **Goal-check every `[DONE]` yourself — miss reruns, hit ships.** A
   worker's word is never the evidence; the orchestrator
   measures before the cell counts:
   - **Read the recorded proof; re-run only on smell.** The worker's cap
     carries the proof line it chose and ran (`<command> — <result> —
     <scope reason>`); reading it satisfies the fresh-output rule. Re-run
     `bee test` yourself on a smell — a missing/garbled report, a
     `[DONE]` with no diff, a `high-risk`/hard-gate cell, or a proof scope
     that looks too narrow for the diff. Orchestrator judgment, not a
     routine step. Failure on a spot-check → the cell is NOT done:
     re-dispatch on the same role with the failing excerpt (a task miss is
     a rerun, never a silent escalation onto the session model — provider
     errors, not task errors, are what the rescue ladder's escalation rung
     is for).
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
   completion; the feature's final slice closes through `bee close`, or
   `bee worktree merge` when the feature has one — both check every
   capped cell's recorded proof line instead of running tests themselves
   ("Proof at finish and close, in full", below).

## Proof at finish and close, in full

The agent owns test scope end to end — it picks the proof each cap needs,
runs it, and records one proof line, CHECKED (never re-run) at the
boundary doors (decisions `58ec9664` D1/D2/D5/D6, `1f534837` D7/D8/D9;
supersedes the boundary-auto-run half of test-cadence-boundary, decision
`13ce1858`, and the proof-economy tier system, decision `412e9b3a`,
`docs/knowledge/areas/verify-pipeline/`, 2026-07-31).

> The agent owns test scope: pick proof, record on cap, doors check and run nothing (rule: agents-proof-at-cap).

- **Declaration:** `.bee/config.json` `commands.test` still names the
  project's one declared suite — what CI runs on every push, and the
  default a code-change proof reaches for. It no longer obliges any
  boundary door to run anything.
- **Runner:** `bee test` runs the declared commands in order and writes
  ONE normalized record, `.bee/logs/test-results.json` — `{ran_at, green,
  commands: [{command, exit, duration_ms, failure_excerpt, failure_log}]}`.
  The runner is a program; an agent's word is never the record, but the
  agent chooses when to reach for it.
- **The proof line:** required on every `cells finish --report`,
  `<command> — <result> — <scope reason>` — three non-empty segments
  (split on the first two ` — ` separators, so the reason may itself
  contain one). A malformed or empty proof refuses the cap; a `red`
  result refuses the cap outright — a red is fix-first, never a done. A
  repo declared no-test (`commands.test` set to the sentinel `"none"`)
  proves with the command segment `none` and the reason naming the
  parity/docs check actually used.
- **At close:** `bee close` reads every capped cell's recorded proof and
  checks it parses — it runs nothing itself. A cell with no report at all
  (a pre-contract legacy cap) is grandfathered through with a named note,
  never a silent pass dressed as proof. A cell whose proof is missing or
  malformed is the remaining door, named by id; the fix is a re-cap
  carrying a real proof line — never un-cap, the fix is new work. When
  the feature has a worktree (including one kept pending-cleanup after a
  merge), close defers — the same proof check runs at `bee worktree
  merge` instead.
- **Merge:** `bee worktree merge` runs the same proof check against the
  staged merge's capped cells — the last local net before CI, which runs
  the full declared command on every push.
- **Never build on red** (rule: agents-never-build-on-red).

<!-- bee:only claude -->
## Native worktree integration

An eligible native-worktree wave integrates through a transactional protocol —
protected pre-dispatch attestation, re-attest before integration, `--no-ff --no-commit`
merge with targeted checks, revert on post-commit red, and conservative cleanup. It is
unbuilt: cell `worktree-isolation-4` owns its acceptance, and no hook or test enforces
any of it today. The full procedure is
`docs/history/worktree-isolation/integration-transaction.md` — read it before running
that acceptance, not on an ordinary wave.
<!-- bee:end -->

## Runtime Spawn Mechanics (side by side)

<!-- bee:only claude -->
| | Claude Code |
|---|---|
| Spawn | `Agent` tool, one call per worker; put the worker prompt in `prompt`; set `run_in_background: true` so the whole wave runs in parallel (send all spawns of a wave in one message) |
| Model | `model` parameter per Agent call = the model `config.models.claude` maps the dispatch's resolved ROLE to (a fresh config seeds `code`/`read` beside the historical `extraction`/`generation`); an ESCALATED cell carries no `model` param at all — it inherits the session model, kept scarce |
| Result collection | You are notified when each background agent completes; its final message is the worker report — parse the leading status token |
| Follow-up / rescue | `SendMessage` to the same agent id continues it with context intact; a new `Agent` call starts fresh |
| Harness assist | `bee-chain-nudge` hook fires on SubagentStop: collect the status, update the cell, check reservations |
| Isolation guarantee | Fresh context per Agent call; include only the contract fields |
| Subagent type | Resolved by the one door, never hand-typed: `.bee/bin/bee dispatch prepare --runtime claude --kind <cell|gather|reviewer|advisor> [--role <name>] --json` returns `subagent_type: "bee-build"` (executes a cell), `"bee-gather"` (reads only), `"bee-extract"` (the cheap reader — `--kind gather --role extraction`), or `"bee-review"`, when the rendered agent exists (`.claude/agents/bee-*.md` — each file declares the ordered ROLE list it serves and pins whatever model that list resolves to); an escalated dispatch returns the runtime default (`general-purpose`); a cli- or herding-shaped slot returns that transport instead (per D1). A role with no rendered agent (`advisor`, or a name this repo invented) is not repaired onto one. The guard repairs a generic type where the role names exactly one agent; where two agents share it, it refuses, because the role alone does not say whether the work writes |
<!-- bee:end -->
<!-- bee:only codex -->
| | Codex |
|---|---|
| Spawn | `spawn_agent({task_name: "<stable-name>", message: "<WORKER_PROMPT>", fork_turns: "none"})` — the codex 0.145.0 schema: `task_name` + `message` required, no `agent_type` field; `bee dispatch prepare --runtime codex` emits exactly this shape, and the guard judges the `[bee-tier: <role>]` marker at the START of `message` for every `spawn_agent` payload — a marker naming a role nothing configures is refused BY NAME |
| Model | `config.models.codex[<role>]` if set; today Codex cannot select a per-agent model → the role is enforced as a read budget + output cap in the prompt |
| Result collection | Status tokens arrive in the parent thread; use `wait_agent(..., timeout_ms=60000)` only when a specific result is needed |
| Follow-up / rescue | `followup_task({target: "<agent id or task name>", message: "..."})` to continue the same agent; a fresh `spawn_agent` only for a genuinely new task — no routine `send_input(...)` mid-flight |
| Harness assist | None — the tend loop in this skill is the nudge |
| Isolation guarantee | `fork_turns: "none"`; never fork the parent history for routine cells |
| Subagent type | No per-agent subagent type — the role is enforced as a read budget + output cap in the worker prompt regardless of what is spawned (documented asymmetry, not parity) |
<!-- bee:end -->

On both runtimes the integrity rails are identical because they live in the helpers: `bee close`/`bee worktree merge` refuse while a capped cell's recorded proof is missing, malformed, or red — and `bee reservations reserve` reports conflicts the worker must turn into `[BLOCKED]`.

## Model Roles — Config-Driven, Runtime-Keyed

`.bee/config.json` `models` is a **role→model map**, keyed by runtime first (bee is dual-runtime and each names models differently), then by the ROLE a dispatch asks for. The key is the JOB the work is, never a cost class. **The session model is never configured** — it is what an escalated cell inherits. A fresh config seeds four job names, and **every one is editable to whatever models the user actually has** (only a Claude subscription → keep all-Claude; a Codex plan too → point roles at GPT via cli executors):

```json
"models": {
  "claude": { "code": "sonnet", "read": "haiku", "extraction": "haiku", "generation": "sonnet", "review": "opus" },
  "codex":  { "code": null,     "read": null,    "extraction": null,    "generation": null,     "review": null }
}
```

A role value may also be `{ "model": "opus", "effort": "xhigh" }` (per-agent reasoning effort, applied where the runtime supports it, silently recorded where it does not; levels: low/medium/high/xhigh/max) or `{ "kind": "cli", "command": "..." }` (external executor, section below — effort rides inside the command). The `review` role is consumed by bee-reviewing's specialists, exploring's fresh-eyes, and bee-planning's merged reviewer (the review wave — Structure + cold-pickup cell review); `null` review falls back to generation. bee ships a config default for `code`, `read`, `extraction` and `generation` only — `review` and `advisor` resolve without a key, and publishing a value for either would decide a product question for every host. **Copy-paste presets** (all-claude, tuned, GPT adversarial review, codex-implements, antigravity/`agy`, opencode, budget): `docs/model-presets.md` in the bee repo — including the `bash -lc '… "$(cat)"'` wrapper every CLI that cannot read the prompt from stdin (`agy`, `opencode`) needs to satisfy the stdin transport in step 3 below.

- **`code`** = the work writes: implementation, wiring, test authoring. Where the bulk of dispatches go.
- **`read`** = the work only reads: retrieval, call-path tracing, evidence digests.
- **`extraction` / `generation`** = the historical names every ordered role list ENDS with. They resolve on every host that ever onboarded, which is why the move to roles took nobody's model away. Leave them set.
- **Any other name is legal** — `test`, `docs`, `design`, `migrate`, whatever this repo's work actually is. bee asks "is this name configured", never "is it one of four words", so a new job role needs no bee code and no new dispatch kind.
- **Escalation is not a role.** Running on the session model is the cell's `escalate` FLAG (`bee cells escalate --id <id>`): that dispatch omits the `model` param and carries the `[bee-tier: ceiling]` marker, anchored to the first non-whitespace token of the prompt or the start of the description (a marker anywhere else never counts). Keep it scarce: planning, integration, architecture, final review only.
- A **null** role means the runtime cannot switch per-agent models (Codex today) → state the role in the worker prompt and enforce it as a read budget + output cap. Set real ids (e.g. `"code": "gpt-5"`) only if your runtime supports per-agent selection.

Read the configured roles for the active runtime before spawning:

```
.bee/bin/bee status --json    # .models shows both runtime maps
```

**Resolution is a walk down an ordered list, and the walk is ONE function.** `resolve_role(models, roles, runtime, kind)` (`verbs/drivers/models.rs` — the single parser the dispatcher, the model guard and onboarding's agent renderer all call) takes the names the consumer will accept, best first, and returns a typed dispatch for the first that carries a resolvable configuration: a model, a prompt budget (anchored `[bee-tier: <role>]` marker), a cli executor (external, below — gather purposes only), or a refusal for a cli-shaped role asked for cell execution (`cli_tier_gather_only`). An unset or unresolvable name YIELDS to the next; a name nothing has heard of also **warns on stderr**, naming what it fell through to. The last entry always resolves, so the walk cannot dead-end. No name resolves a model the config does not carry for it, and the ONE unknown name that resolves silently is `code` or `read` on a runtime whose `models.<runtime>` configures NEITHER of them — the pre-roles window, where falling through to the historical name is the intended no-op and a warning would fire on every dispatch. The window is per runtime, because the table is, and the first of the two keys an operator configures shuts it, so a half-migrated config is loud about the sibling it missed. Every other unrecognized slot quietly reading as `generation` is exactly what this feature deleted.

Two doors ask the same question and answer it differently, on purpose:
- a **cell's** declared role heads an ordered list → an unconfigured name falls through and warns (silent only inside the pre-roles window above), and the work still runs;
- an explicit **`--role <name>`** on `dispatch prepare`, and a `[bee-tier: <name>]` marker at the guard, name the slot OUTRIGHT → an unconfigured name is REFUSED by name, its FIX listing the roles that runtime resolves and ending "Any role name you configure is legal; bee holds no fixed list."

Every dispatch still carries an explicit marker: an escalated dispatch needs the [bee-tier: ceiling] marker anchored to the first non-whitespace token of the prompt, or the description must start with it; a prompt-budget dispatch needs the matching [bee-tier: <role>] marker anchored the same way, stated alongside the budget in the prompt. A marker anywhere else — embedded mid-prompt or mid-description — never satisfies the transport, and a bare dispatch with neither the model param nor an anchored marker is denied by the model-guard hook.

**Runtime fallback chains are PUBLISHED, never walked by bee.** `retry.fallbackChains` maps a role name, a concrete model selector, or a `provider/*` wildcard to an ordered list of models a FAILED dispatch may move along (most specific key wins: model, then wildcard, then role). It is explicit-only — no built-in chain for any role, and a `default` key is refused out loud, so with nothing configured every payload is byte-identical to a bee that never heard of chains. `bee dispatch prepare` resolves the chain that applies and publishes it on the payload as `fallback_chain: {key, chain, fallback_when, advance_on, never_advance_on}`. **bee does not execute dispatches, so bee never retries** (decision `51341f84`): advancing a step, and recording the step taken, belong to whoever runs the payload. The gate travels with the chain — a step may fire ONLY on quota or rate limit, provider auth or policy rejection, empty response, replay-safe malformed tool call, stream stall or connection reset, or a 5xx; it may NEVER fire on a tool error, a wrong or unwanted result, a failed proof, or a red test. A red test moving the work to another model would hide the defect. Distinct from the resolution fall-through above: fall-through answers an unconfigured NAME before anything runs; a chain answers a model that was reached and FAILED.

**Dispatch economics:** `.bee/config.json` names the **requested** model for a role — what should run, never a guarantee of what did. Every dispatch the guard evaluates (allowed or denied) logs the honest split in `.bee/logs/dispatch.jsonl`: `logical_tier` (the declared ROLE — the field keeps its historical key), `requested_model` (what config names, when resolvable), `effective_model` + `effective_model_status`, `channel` (the transport family), and `enforcement` (the mechanism). A real structural `model` param on a claude Agent/Task dispatch is `pinned` — `effective_model` equals that param, because the param is something we actually watched the caller pass. A claude dispatch carrying only the `[bee-tier: <role>]` marker (no param — the prompt-budget style) is `unverified` — `requested_model` may still name the role's configured model, informationally, but nothing pins the dispatch to it. On **codex-native** transport (`spawn_agent`), the effective model is `inherited-or-unknown` **always** — codex-cli 0.145.0 has no per-agent model selection at all, so this status never flips to `pinned` no matter what the role resolves to; only a future capability probe proving per-agent selection would justify moving it. A **cli-exec** dispatch (external executor, below) is `unverified` too — the command names its own model in its own argv, outside this vocabulary, so `requested_model` is always `null` there.

## External Executors — Multi-Provider Workers

A configured role may name an **external CLI executor** instead of a model — that is how GPT/Codex, GLM, Kimi, or any other provider's CLI becomes a bee worker while Claude (or Codex) stays the orchestrator:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": { "kind": "cli", "command": "codex exec --json -m gpt-5.3-codex -c model_reasoning_effort=high --full-auto" }
  }
}
```

**Dispatch guard — what never routes to a cli executor:** a cell whose work needs the *session's* tools — MCP servers (browser, computer-use), credential managers, secrets reads, or anything only the orchestrating harness can reach — stays on a native role; the external process cannot see those tools and will improvise instead of failing loudly. Destructive/irreversible operations (pushes, releases, external-system mutations) also never go external.

**Status:** `resolve_role` purpose-scopes cli resolution by its `kind` argument — a cell-purpose resolve of a cli-shaped role **refuses** (`cli_tier_gather_only`); only a gather purpose reaches the cli transport. Cli cell execution — the reserve/verify/cap/release worker contract described below — is not dispatched today; a cli-shaped role serves gathers only, through the Delegation contract's cli gather branch (`bee-hive/references/gates-and-delegation.md`), which has no reservation, no cap, and no `result.json` — stdout **is** the digest. This section documents the cell-execution contract for when that path is enabled; until then, do not dispatch a cell to a cli-shaped role under the protocol below.

The cell-execution protocol for that path — prompt file, finish contract,
detached spawn, artifact tending, acceptance, trust boundary, rescue — is
`docs/history/cli-executors-cell-path.md`. It stays out of this reference
while `resolve_role` refuses a cli-shaped role for anything but a gather.

**Transient hygiene:** dispatch transients (`<cell-id>.prompt.md`, `.out*.log`, `.result.md|json`, reviewer/plan-check logs) accumulate in `.bee/workers/` and are never needed after the feature closes. At feature close — after review acceptance, before the closing commit — the orchestrator runs `.bee/bin/bee state worker prune` (`--dry-run` to preview). Keep-rules protect transients of active workers and non-capped cells (re-read immediately before the destructive loop), and files outside the transient suffix set (evidence snapshots, cell payloads, subdirectories) are never touched — but prune is still the orchestrator's feature-close verb, not something to race against an in-flight dispatch round.

## Herding Execution — A Foreign Agent In A Pane

`bee herding run` (herding-executor D1, D4) is a THIRD transport, distinct
from both the native subagent dispatch above and the cli gather branch: a
long-lived foreign agent (any herdr-supported kind) started in its own
pane, doing write work against a worktree. It supports two scopes:

- **Scope A (user-requested per cell)**: running the verb by hand against ONE
  cell the user names, when an external agent is explicitly requested.
- **Scope B (automatic role dispatch)**: a `{kind:"herding"}` role slot in
  `models.*` (herding-tier D1-D6, widened by herding-review-slots D1), which is
  selected automatically by `.bee/bin/bee dispatch prepare` (per D1) whenever
  the configured role slot is herding-shaped.

For model-shaped slots, the default for `standard`/`high-risk` cell execution
stays the wave protocol above.

**The pane worker is bee-ignorant (D4): it never runs a `bee` verb.** Its
whole contract is the self-contained brief `bee herding run` renders —
task, absolute paths, file constraints, the `result-N.json` schema, the
tmp-then-rename write gesture — and one JSON file written back. Every
piece of bee bookkeeping — verify, `cells finish`, reservations, judge —
is the ORCHESTRATOR's job, done AFTER reading the result, exactly as D4
requires. This mirrors the cli gather branch's stdout-is-the-digest
posture, but for write work: the external process never touches
`.bee/*.json(l)`, never claims, never caps.

The orchestrator's loop for one herding cell:

1. **Claim first**, same as any dispatch — `cells claim` under a chosen
   nickname — before starting the pane.
2. **Dispatch through background Bash**, one call:
   `bee herding run --task "<self-contained brief>" --cwd <worktree> --job-id <id>`.
   Background Bash delivers exactly one completion notification when the
   verb returns — the native poll loop inside `herding run` (heartbeat +
   idle-timeout + ceiling, D5) already does the waiting at zero token
   cost; the orchestrator does not poll it itself.
3. **Read the result.** `bee herding run` prints the validated
   `result-N.json` JSON on completion (`status: done|blocked`, `summary`,
   `files_changed[]`, `proof`) — read that, never re-derive it by
   screen-scraping the pane.
4. **Do the bookkeeping the worker never could:** verify per the cell's
   own proof type (re-run the declared test/parity check against the
   `files_changed` diff), then `.bee/bin/bee cells finish --id <id>
   --report <proof line>` carrying the worker's evidence, confirm
   reservations released, and run goal-check/judge exactly as step 7 of
   the Operating Contract already requires for any `[DONE]`. A `status:
   blocked` result is never force-capped — treat it as a `[BLOCKED]`
   report and re-triage.
5. **Blocked → `--continue`, not a fresh spawn.** When the result (or the
   orchestrator's own verify) surfaces something the same agent should
   retry — a failing test, a narrower ask — send the next round through
   the SAME job mailbox: `bee herding run --continue <job-id> --task
   "<round N+1 brief>"`. This reuses the existing pane and agent (`herdr
   agent prompt`, never `agent start` again) and waits on
   `result-(N+1).json`; it refuses typed when the job dir, a prior
   result, or the pane itself is gone (D3). Loop rounds this way until a
   `status: done` result passes verify, or the orchestrator gives up and
   reports `[BLOCKED]` itself.

**On spawn_failed:** the result JSON's `remedy` field names the unwind (`bee cells unclaim` and `bee reservations release`); claims and reservations stay the orchestrator's to release; the pane stays open as forensics.

## Worker Prompt Template

Nicknames are Minions character names — recognizable,
consistent worker identities; the assigned cell stays authoritative for responsibilities.

```text
You are a bee worker subagent.

Identity:
- Agent nickname (reservation identity): <NICKNAME>
- Assigned cell id: <CELL_ID> (ALREADY CLAIMED for you by the orchestrator before dispatch — do NOT run `cells claim`; validate against the inlined cell JSON below)
- Feature: <FEATURE>
- Role: <the cell's own role, e.g. code|read|test> (model: <MODEL_NAME>; an escalated cell says "escalated — session model")
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
3. Reserve, implement, commit, finish (`--report` carrying your proof line; `bee close`/`bee worktree merge` check it, never re-run it), report.
```

The `Advisor` line is omitted entirely — a session whose config has no advisor slot dispatches byte-identical prompts to today — whenever no advisor resolves, or the advisor's model name literally matches the worker's own resolved model (the one honest no-op). Escalated workers are not a skip condition — config is the authority and the orchestrator does not second-guess it with a strength ladder. The same-model no-op is the orchestrator's, run at dispatch, never left to the worker. When present, `<TRANSPORT>` states the proven transport verbatim, matching what the worker contract's Advisor Consult section (references/worker-details.md) tells the worker to run:
<!-- bee:only claude -->
for a **model-shaped** advisor, `your own Agent tool, model param <advisor-model>, description starting exactly "advisor-consult <CELL_ID>: <advisor-model>"` (fallback: headless `claude -p --model <advisor-model>`);
<!-- bee:end -->
for a **cli-shaped** advisor, `<the configured command>, evidence bundle on stdin` (External Executors output-capture discipline, above).

The dispatcher may compose an Expertise section for the worker leader-style via `--expertise` (one entry per line, `<path> :: <purpose> :: <read-to>`), choosing from bee's own skill references and knowledge files; optional and judgment-driven, never auto-derived.

Default: no session history, no other cells, no orchestrator reasoning. A worker that needs more than this contract means the cell failed cold-pickup review — route the gap back rather than widen the prompt with transcript.

## Result Formats (expected back from workers)

Native subagents return these token-markdown reports as their final message. Cli executors deliver the **same four outcomes** as `.bee/workers/<cell-id>.result.json` (External Executors, step 2) — one contract, two transports.

```text
[DONE] <cell-id>: <title>
Nickname: <name>
Files modified: <paths>
Reservations: reserved <paths>; released yes|no
Tests: <command> — <result> — <scope reason>
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

Near 65% context, write `.bee/HANDOFF.json` (rule: agents-context-handoff-65) — this is the record's schema, the one site that keeps it: `{ phase, feature, mode, cells_in_flight, done, remaining, next_action, written_at }`. Include the resume commands:

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
