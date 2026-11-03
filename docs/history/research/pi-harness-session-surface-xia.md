---
artifact_contract: bee-research/v1
topic: pi-harness-session-surface-xia
depth: deep
date: 2026-09-20
---

## Bottom Line

- **Recommendation (ladder rung): reuse, with three built-in adopts.** bee's Pi
  belt already drives a *larger* share of Pi's host surface than
  pi-dynamic-workflows does — 13 events and 19 API members against the source's
  5 and 10. The source's bulk is its **workflow engine**, which bee declined on
  purpose (`pi-native-stage-driver/CONTEXT.md` D6). At the harness layer there
  is no broad gap to close.
- **Why this is the lightest credible path:** every remaining delta but one is
  a documented Pi 0.85.1 built-in bee simply has not called yet (rung 2), not a
  pattern to port from the source (rung 3).
- **Why the next-best rung lost:** porting from the source would drag its
  monkey-patch (`AgentSession.prototype` theft, `task-panel.ts:542-595`) and its
  unverified `.pi/agents` convention into a belt that already has documented
  equivalents. One item — the guidance baseline — has no Pi built-in and is the
  single honest rung-3 adapt.
- **Confidence: 80%** (was 88% — lowered 2026-09-20 after the A2 correction
  below). Docs and both repos read directly; the two eval scripts were read
  through a gather digest, not line by line. One Local claim in this brief was
  filed without its positive search and had to be corrected during planning —
  the remaining Local claims were re-checked, but that miss is priced in here.
- **Suggested next step:** `bee-shaping` for A1–A3 as one small feature; A4/A5
  are backlog polish rows.

## Repo Snapshot

- **bee** — Rust CLI (`packages/bee-rs`), four runtime belts sharing one brain
  (`bee hook <name>`). Pi belt: `.pi/extensions/bee-guard.ts`, 2927 lines,
  hand-written and authoritative (D7); `doctor.rs` embeds it with `include_str!`
  and byte-compares at `:310-312`.
- **Pi** — 0.85.1 active via mise (`~/.config/mise/config.toml`, `latest`).
  `docs/extensions.md` is 3023 lines and documents 36 events.
- **Source** — `/home/thanhsmind/Projects/refs/pi-dynamic-workflows`, v3.12.0,
  SHA `e29dbcae9a4739161abdf2bd21b302c076256379`, 172 files.
- **Constraint that shapes the answer:** D6 (no second state layer, no
  third-party npm runtime) and D11 (the belt gains no dispatch tool and no
  spawner; the child `pi` process is spawned from Rust inside `bee herding run`).

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/home/thanhsmind/Projects/refs/pi-dynamic-workflows` |
| Ref | `main` (v3.12.0) |
| Resolved commit SHA | `e29dbcae9a4739161abdf2bd21b302c076256379` |
| Narrowed scope | Session-surface engineering only: reload handoff, command registry, task panel, structured output, guidance evals. The workflow engine is out of scope by D6. |

Mode: **`xia`** — the ask reads as understanding, not bringing in. Stops after
the cross-cutting sweep; builds nothing.

## Question & Assumptions

- **What was asked:** bee is itself a workflow system, and the work in hand is a
  bee harness extension for Pi so the whole bee workflow runs there. With that
  approach, what must be learned from pi-dynamic-workflows to be effective?
- **What settles it:** an API-surface diff between the two extensions against
  the version-matched Pi docs, plus a delta list where the source reaches a
  surface bee's belt cannot.
- **Assumption confirmed:** the impression that bee "only ships a guard" is an
  artifact of file *count*. Verified false — see Local.

## Findings

### Local

- **Belt event surface (`Local`).** `.pi/extensions/bee-guard.ts` handles 13
  events: `agent_settled`, `before_agent_start`, `session_before_compact`,
  `session_shutdown`, `session_start`, `session_tree`, `tool_call`,
  `tool_execution_start`, `tool_result`, `turn_end`, `turn_start`,
  `ui_prompt_end`, `ui_prompt_start`.
- **Belt API surface (`Local`).** 19 members: `getActiveTools`, `getAllTools`,
  `getBranch`, `getSessionId`, `hasUI`, `isIdle`, `notify`, `on`,
  `registerCommand`, `sendMessage`, `sendUserMessage`, `sessionManager`,
  `setActiveTools`, `setStatus`, `switchSession`, `ui`, `cwd`, `model`,
  `result`.
- **The belt is not "only a guard" (`Local`).** It registers six slash commands
  (`bee-worktree-new/enter/exit/merge/relocate` at `:2666-2833`,
  `bee-tools-reopen` at `:2874`), drives the D4 hard tool gate
  (`setActiveTools`, `:2553`), renders a model-usage status line
  (`refreshModelUsageStatus`, `:2028`, refreshed at `session_start`, `turn_end`
  and `session_tree`), drains a result inbox on a timer, and forks sessions via
  `SessionManager.forkFrom` (`:1633-1650`).
- **Worker verdicts are prose, never parsed (`Local`).** `[DONE]`, `[BLOCKED]`,
  `[HANDOFF]`, `[NOOP]` appear in the Rust crate only as text *instructed to the
  model* (`hooks/chain_nudge.rs:147`, `verbs/cells/handlers_close.rs:1270`,
  `hooks/write_guard/checks.rs:663`). `classify_outcome`
  (`herding/wave.rs:471`) classifies only **transport** outcomes — `succeeded`,
  `timed_out`, `send_failed`, `resolution_failed`,
  `unverifiable_after_send`, `flipped_before_send`, `unsafe_at_preflight`. bee
  has already recorded the harm this causes, at `verbs/cells/dissent.rs:8`:
  *"its `[BLOCKED]` report, which is prose the orchestrator may summarize
  away."*
> **CORRECTION — 2026-09-20, from planning's reality touch.** The bullet above
> is wrong as written, and A2 below rested on it. bee **already has** a
> schema-validated worker verdict on the herding path. `herding/mailbox.rs:490`
> defines `MailboxResult` with a required `status` (`"done"` | `"blocked"`),
> `summary`, `files_changed` and `proof`, parsed and validated at `:655-690`;
> `result-N.json` is written by the worker tmp-then-rename and *its appearance
> at the final name IS the done signal* (`mailbox.rs:20-26`). Every real job
> under `.bee/mailbox/` carries one — e.g.
> `job-1789890452645-4031915-1/result-1.json`, written by a gather worker on
> 2026-09-20.
>
> What the original bullet actually proves is narrower and still true: the
> **chat-reply tokens** `[DONE]`/`[BLOCKED]` are prose, and `dissent.rs:8`'s
> recorded harm is about the orchestrator's summary to the user — a different
> channel from the mailbox result.
>
> The error was grepping for the token I expected, reading its absence as a
> never-claim, and skipping the positive search. **What survives for A2:**
> `terminate: true` (one saved assistant turn per worker) and host-side schema
> validation so a worker cannot malform the file it already owes — not "add
> structure", which exists.

- **No comprehension eval exists (`Local`).** `verbs/cells/judge.rs` and
  decision `0018-orchestrator-goal-check-and-frozen-judge.md` judge whether a
  cell's *work* met its goal. Nothing measures whether a model *understands*
  bee's model-facing prose, and no baseline of that prose is checked in.
- **`.pi/` ships one file (`Local`).** `.pi/extensions/bee-guard.ts` only. No
  agents directory. Skills install to `.agents/skills` and `doctor` checks that
  path (`.bee/verify/verify-app/features/pi-runtime.md`).
- **One reload slot already exists (`Local`).** `Symbol.for("bee.pi.result-drain")`
  on `globalThis` (`:669`) exists to **clear** a previous module instance's
  interval (`stopDrainTimer`, `:671-683`) — it deliberately carries no state
  forward.

### Upstream

- **The source drives less of Pi, not more (`Upstream`).** 5 events
  (`session_start`, `session_shutdown`, `model_select`, `turn_end`, `input`) and
  10 API members (`events`, `get`, `getActiveTools`, `getCommands`, `on`,
  `registerCommand`, `registerTool`, `sendMessage`, `setActiveTools`, `skills`).
  Its 172 files are the workflow engine, not host integration.
- **Live panel as a widget, not a transcript line (`Upstream`).**
  `ui.setWidget("workflow-tasks", …, { placement: "belowEditor" })`
  (`task-panel.ts:1642-1676`), re-rendered with `tui.requestRender()` and
  width-fitted per row (`:101-102`). Its stated rule (`:1656-1657`): *"Purely
  informational: it lists running runs and re-renders on events."*
- **Result delivery that continues the turn (`Upstream`).**
  `{ triggerTurn: true, deliverAs: "followUp" }` (`task-panel.ts:773-776`), with
  routing by `run.sessionId`, a durable pending marker cleared only after a
  verified ACK, and suspend/drop/bind handlers for session replacement
  (`:999-1108`, `:1191-1193`).
- **Reload handoff (`Upstream`).** A single process-wide slot on `globalThis`
  keyed `Symbol.for("@quintinshaw/pi-dynamic-workflows:reload-handoff-slot")`,
  holding `{ cwd, extensionVersion, manager, effort }`, claimed only on an exact
  version match, cwd-checked *after* claim, and expiring after 30 s into
  `pauseStrandedWorkflowRuntime()` (`extension-reload.ts:15-52, 84-95, 135-151`).
  `SESSION_REPLACEMENT_REASONS = new Set(["reload", "new", "resume", "fork"])`
  (`:192`).
- **Command ownership (`Upstream`).** `pi.getCommands()` checked before
  registering, with a `Symbol.for` ownership registry and a `WeakMap` fallback
  for a frozen API object (`command-registry.ts:37`, `workflow-commands.ts:142`).
- **Guidance baseline (`Upstream`).** `docs/workflow-guidance-baseline.json`
  pins sha256 digests of two model-facing prose surfaces
  (`compactGuidance`, `detailedProse`); `check-workflow-release.ts` warns on
  drift (`NON_CONTRACTUAL_PROSE_DRIFT`) and exits 1 on diagnostic errors;
  `accept-workflow-guidance.ts` is the reviewed way to move the baseline.
- **Additive arming, not narrowing (`Upstream`).** `pi.on("input")` *adds* the
  workflow tool and saves the original set, never removing
  (`workflow-editor.ts:333-350`) — the opposite direction to bee's D4.

### Docs

Pi 0.85.1 `docs/extensions.md`, version-matched to the installed binary.

- **`terminate: true` is built in (`Docs`, `:2019`).** *"Return `terminate: true`
  from `execute()` to hint that the automatic follow-up LLM call should be
  skipped after the current tool batch. This only takes effect when every
  finalized tool result in that batch is terminating."* It ships an official
  example: `examples/extensions/structured-output.ts`.
- **Widgets are built in (`Docs`, `:2613-2619`).** `setWidget` with
  `{ placement: "belowEditor" }`, alongside `setStatus` (`:2589-2591`),
  `setWorkingMessage` / `setWorkingVisible` / `setWorkingIndicator`
  (`:2593-2612`) and `setFooter` (`:2620-2625`).
- **No state handoff exists (`Docs`, `:1318-1322`).** *"Code after
  `await ctx.reload()` must not assume old in-memory extension state is still
  valid."* Extensions are re-created on `reload`, `new`, `resume` and `fork`;
  persistence is the extension's own job.
- **Duplicate command names are kept (`Docs`, `:1529`).** *"If multiple
  extensions register the same command name, pi keeps them all and assigns
  numeric invocation suffixes in load order, for example `/review:1` and
  `/review:2`."*
- **Tool removal is legal but costs cache (`Docs`, `:2394-2396`).** *"Pi also
  uses this safe fallback when the active set is not purely additive… Tool
  removals therefore work, but they do not use deferred loading."* This is the
  cost D12 already accepted on the record.
- **`model_select` names this exact use (`Docs`, `:740-760`).** Fires on
  `/model`, `Ctrl+P` cycling and session restore, carrying
  `source: "set" | "cycle" | "restore"`. *"Use this to update UI elements
  (status bars, footers) or perform model-specific initialization."*
  `thinking_level_select` (`:761`) is its sibling.
- **No agent-directory convention (`Docs`, Q8 — ABSENT).** 0.85.1 documents
  skill discovery under `.agents/skills/` (`:355`, `skills.md:28-39`) and **no**
  `.pi/agents/*.md` convention.
- **No built-in subagent tool (`Docs`, Q9 — ABSENT).** `createAgentSession` is
  an `sdk.md:46` entrypoint for external embedding, not an extension API. The
  only documented in-extension spawn pattern is `registerTool` + `exec`
  (`:3005`) — which is what D11 already chose, one layer down in Rust.

### Inference

- bee's belt and the source solve **different problems on the same host**. The
  source needed an engine because Pi has none; bee already owns lanes, cells,
  gates and proof in Rust. Copying its shape would rebuild what D6 rejected.
- The source's `.pi/agents` comment — *"matching pi-coding-agent's own built-in
  agent discovery convention"* (`agent-registry.ts:8-13`) — **overclaims**
  against 0.85.1 docs. It is the source's own convention. bee's `.agents/skills`
  is the documented one. Do not chase this.
- The source **steals** `sendCustomMessage` off `AgentSession.prototype`
  (`task-panel.ts:542-595`) because it had no clean delivery API. bee uses the
  documented `pi.sendUserMessage` with `deliverAs: "steer"` when busy and a
  `turnStartPending` latch against double turns (`:905-925`). **bee's path is
  the cleaner of the two** — a place to not learn from the source.

## Dependency Matrix

| Component | Source | bee local | Verdict | Evidence |
|---|---|---|---|---|
| Write/enforcement belt | none | `bee-guard.ts` blocking `tool_call` | `EXISTS` (bee ahead) | `Local` |
| Slash commands | `registerCommand` + ownership registry | 6 commands, no ownership guard | `EXISTS`, guard missing | `Local` / `Upstream` |
| Per-stage tool gating | additive arming | subtractive D4 hard gate | `CONFLICT` (by locked decision) | `Docs :2394` |
| Reload state handoff | versioned `globalThis` slot, 30 s expiry | timer-clear slot only | `EXISTS`, narrower by design | `Local :669` |
| Status line | not used | `setStatus` model usage | `EXISTS` (bee ahead) | `Local :2028` |
| Live multi-run panel | `setWidget` belowEditor | none | `NEW` | `Docs :2613` |
| Result → conversation | stolen prototype send | `sendUserMessage` + steer latch | `EXISTS` (bee cleaner) | `Local :905` |
| Structured worker verdict | `terminate: true` tool | `MailboxResult` schema + `result-N.json`, already validated | `EXISTS` — only the terminating tool is `NEW` | `Local mailbox.rs:490,:655-690` (corrected) |
| Model-change UI refresh | `model_select` hooked | unhooked; `turn_end` refresh | `NEW` (polish) | `Docs :740` |
| Agent definitions dir | `.pi/agents/*.md` | none | `CONFLICT` — not a Pi convention | `Docs` Q8 ABSENT |
| Workflow engine | 172 files | Rust store | `CONFLICT` — declined by D6 | `Local` |
| Prose comprehension baseline | sha256 + release gate | none | `NEW` | `Local`, `Upstream` |

## Cross-Cutting Sweep

Wiring outside the belt file, checked:

- `onboard/plan.rs:830-841` vendors `.pi/extensions/` and its files; `:117-126`
  enumerates them. **Never rendered by the skill-tree pipeline** (`:121`) — so a
  new belt file is a hand-edit plus a rebuild, per D7.
- `doctor.rs:47` `include_str!` + `:310-312` byte-compare — any belt edit that
  ships without a rebuild reports drift.
- `hook_manifests.rs:46-64` and `devtools/mod.rs:529-547` name Pi a **named
  exclusion**; `bee dev regen` writes nothing here.
- `prepare.rs:2237-2243` holds the Pi dispatch arm.
- `pi_plugin_contracts.rs` is the belt's contract suite; the belt-parity test
  derives a fourth belt from the Pi source with model-guard excluded **by name**.
- `.bee/verify/verify-app/features/pi-runtime.md` maps the user-facing surface.

Not swept: `packages/bee-rs/crates/bee/src/herding/run.rs` no-pane spawn path was
read only through its outcome classifier. Unchecked, therefore not confirmed clean.

## Recommendations, Ranked

**A1 — A live worker panel (`setWidget`, belowEditor). Rung 2, built-in.**
D1 makes native dispatch the default, so a Pi leader will hold several child
workers at once. Today that state has one footer line (`setStatus`). Pi already
ships the surface, and bee's `▸ ✓ ⚡ ✗` progress-tick contract already defines
what to draw in it. Take the widget; do not take the source's 1677-line panel.

**A2 — A terminating tool over the verdict bee already has. Rung 2, built-in.**
*(Rewritten 2026-09-20 after the correction above — the original claim that bee
had no structured verdict was false.)* `MailboxResult` already defines the
schema and `result-N.json` already carries it. What is missing is the tool that
writes it: register one terminating tool mirroring that existing schema, so Pi
validates the fields before `execute()` runs and the worker ends on the call.
Two honest gains, both smaller than first claimed: the worker stops paying for
a follow-up assistant turn on every dispatch, and it can no longer malform the
file it already owes — the `malformed_result` + exit-1 failure, with every file
correctly written, is a case this repo has already hit. Pi ships the example at
`examples/extensions/structured-output.ts`.

**A3 — A guidance baseline over bee's model-facing prose. Rung 3, adapt.**
bee's product *is* prose aimed at a model. Hash the surfaces that matter, check
the digests in, fail the release check on unreviewed drift, and make moving the
baseline a reviewed act. The source's three-file shape (baseline JSON, check
script, accept script) transfers directly; its scenario harness does not need to.

**A4 — Hook `model_select` and `thinking_level_select`. Polish.**
Both refresh `refreshModelUsageStatus`. The doc names this use outright. Bounded
harm today: the status line self-corrects at the next `turn_end`, so this is
latency, not correctness.

**A5 — Guard `registerCommand` with `pi.getCommands()`. Polish.**
Pi keeps duplicates as `/cmd:1`, `/cmd:2`. Extensions are re-created on session
replacement, so this is defensive, not a live defect.

**Do not take:** the `.pi/agents` convention (not Pi's), the
`AgentSession.prototype` monkey-patch (bee's `sendUserMessage` is cleaner), the
workflow engine (D6), or additive-only arming (D4 is locked, and D12 already
priced the cache cost).

## Risks, Unknowns, Follow-Ups

- **A locked decision is touched, not contradicted.** A2 registers a bee tool on
  Pi. `bee-guard.ts:52-59` and D11 rest on the belt having *no dispatch tool and
  no spawner*; a **verdict** tool is neither — it starts nothing and spawns
  nothing. But it does add a name to `mapToolCall`, and the belt-parity test
  asserts model-guard's exclusion **by name**. Both need an explicit row before
  A2 ships. Superseding anything here is the user's move, not this brief's.
- **Suspicion raised and refuted, recorded rather than dropped.** I suspected the
  D4 tool gate loses its full set across `/reload`, leaving `bee-tools-reopen`
  unable to restore. **Refuted:** `fullToolSet` re-captures from
  `getAllTools()` at the next `turn_start` (`:2482-2490`), and `getAllTools()`
  returns all *registered* tools independent of the active set (`Docs :1677`).
  No defect.
- **Unverified.** The two eval scripts were read through a gather digest, not
  line by line — A3's effort estimate is `Inference`, not measured.
- **Unchecked.** `herding/run.rs`'s no-pane spawn path (see sweep). If A2 lands,
  that is where a structured verdict would be read back.
- **Open question for shaping.** A2 needs a decision on where the verdict schema
  lives — the belt, or the Rust side that already owns `classify_outcome`.

## Source Pack

**Local files read**

- `.pi/extensions/bee-guard.ts` (`:1-60`, `:537-700`, `:905-925`, `:2028-2060`,
  `:2074-2200`, `:2428-2600`, `:2666-2915`)
- `packages/bee-rs/crates/bee/src/herding/wave.rs:471-500`
- `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs:8`
- `packages/bee-rs/crates/bee/src/onboard/plan.rs:103-126`, `:830-841`
- `docs/history/pi-native-stage-driver/CONTEXT.md` (D1–D12)
- `.bee/verify/verify-app/features/pi-runtime.md`
- `docs/history/research/pi-workflows-xia.md`, `pi-dynamic-workflows-xia.md`

**Upstream read** — `pi-dynamic-workflows` @ `e29dbcae`: `src/pi-extension.ts`,
`command-registry.ts`, `task-panel.ts`, `shared-store.ts`, `run-persistence.ts`,
`fs-persistence.ts`, `model-routing.ts`, `model-tier-config.ts`,
`extension-reload.ts`, `workflow-control-tool.ts`, `structured-output.ts`,
`agent-registry.ts`, `usage-limit-scheduler.ts`, `workflow-editor.ts`,
`extensions/workflow.ts`, `docs/workflow-guidance-baseline.json`, `scripts/*`

**Docs read** — Pi 0.85.1 `docs/extensions.md` (36 events; `:740`, `:1265`,
`:1318`, `:1365`, `:1525`, `:1529`, `:1677`, `:2019`, `:2082`, `:2375`,
`:2394`, `:2506`, `:2589`, `:2613`, `:3005`), `skills.md:28-39`, `sdk.md:46`

**Prior briefs** — this one covers the harness axis those two did not; their
engine-semantics findings stand unchanged.
