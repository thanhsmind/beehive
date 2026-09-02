---
artifact_contract: bee-research/v1
topic: pi-workflows-xia
depth: deep
date: 2026-09-02
---

## Bottom Line

> **ANSWERED 2026-09-02 — the user chose to keep herding-only.** The research
> recommendation below (rung: built-in, wire Pi's own subagent surface) was put
> to the user and **declined**, with the evidence in hand. Decisions
> **9f5c6d17** (Pi dispatch stays herding-only; 7f9c8518 and 8650ca7b stand
> unchanged; bee takes design rules only, no engine and no second transport) and
> **5d87f14e** (`pi` means the `pi` binary 0.84.x only; omp is not a bee target)
> are the live answer. Everything below is kept as the evidence those decisions
> rest on — read it as findings, not as a live recommendation. The one section
> that is still forward-looking is **§ Five rules worth taking**.

- **Recommendation (ladder rung): built-in** — Pi 0.84.4's own extension API
  already carries the two things bee is missing on Pi. Do **not** port
  pi-workflows' engine. *(Declined — see the note above.)*
- **The one finding that changes bee's Pi story**: Pi has **three** working
  agent fan-out paths that bee's locked decision `7f9c8518-0d26-4a40-b51a-d18a433e42a2`
  ("Pi runtime has no native subagent surface … every worker dispatch routes
  through the herding transport") says do not exist. The decision's *premise* is
  true — Pi ships no built-in Task tool the model can call. Its *conclusion* —
  "the only execution fan-out available under Pi is herding panes" — is
  falsified by the extension API, and bee already ships a Pi extension
  (`.pi/extensions/bee-guard.ts`) to hang it on.
- **Why this beats adapt-upstream (rung 3)**: pi-workflows is ~10k lines of
  TypeScript, a SQLite host process, a socket protocol, controllers and a Rust
  viewer. bee already owns every job it does — lanes, cells, gates, decisions,
  proof, worktrees — in Rust, on JSON state. Porting the engine buys nothing and
  costs a second state layer. What is worth taking from it is **five design
  rules**, not code.
- **Why reuse (rung 1) lost**: the Pi subagent surface is not wired in bee today
  — `dispatch prepare --runtime pi` refuses every non-herding slot by
  construction (`prepare.rs:1525`).
- **Confidence: 92%** — the subagent surface is proven from the installed
  binary's own shipped example and its own docs; the 8% is whether bee wants to
  keep herding as the single Pi transport for reasons outside this brief.
- **Suggested next step: bee-shaping**, one feature — "pi native dispatch" —
  because it supersedes a locked decision, which is the user's move, not the
  agent's. *(The user's move was made: declined. The remaining candidate work is
  § Five rules worth taking, unshaped.)*

---

## Repo Snapshot

- **This repo (bee)**: Rust CLI (`packages/bee-rs`, edition 2024), plugin
  version 2.32.0, plus one hand-written TypeScript belt per harness. `Local`
- **Harness runtimes bee knows**: `RUNTIMES = ["claude","codex","opencode","pi"]`
  (`models.rs:41`); dispatch doors accept
  `DISPATCH_RUNTIMES = ["codex","claude","pi"]` (`prepare.rs:40`). `Local`
- **Target harness**: `pi` = `@earendil-works/pi-coding-agent` **0.84.4**
  (installed via mise; `github.com/earendil-works/pi`). `Local`
- **Source distilled**: `github.com/osolmaz/pi-workflows` @
  `7c1b460d7998cf625d024e087076731bb9a48254`, version 0.15.3, pinned against Pi
  SDK `>=0.84.2 <0.85.0`. `Upstream`
- **A second agent is also installed**: `omp` **18.1.2**
  (`refs/oh-my-pi`, `github:can1357/oh-my-pi`) — a fork with `~/.omp/` paths and
  a *native* task subsystem. It is not `pi`. Every Pi claim below is checked
  against `pi` 0.84.4 only; omp is treated separately in § Two smaller finds.
  `Local`

---

## Question & Assumptions

- **What was asked**: read pi-workflows, learn how they build workflows for Pi,
  and apply that to making the bee harness run well in Pi.
- **What success means**: bee on Pi loses no capability it has on Claude Code,
  and gains nothing bee does not already own.
- **Assumption, now resolved**: this brief assumed the user wanted Pi to reach
  *parity* with Claude Code. The user answered otherwise on 2026-09-02 —
  herding-only is a deliberate constraint they still want (**9f5c6d17**), and
  `pi` means the `pi` binary alone (**5d87f14e**). So the dispatch half of this
  brief is evidence, not a proposal, and only § "Five rules worth taking"
  remains live.

---

## Findings

### Local — what bee already has on Pi (the baseline; do not rebuild it)

bee's Pi story is already complete, tested and decision-backed.

- `.pi/extensions/bee-guard.ts` (996 lines) holds **zero rules of its own** —
  every verdict comes from `.bee/bin/bee hook <name>`, the same brain the
  Claude, Codex and OpenCode belts call. Five Pi events wired:
  `tool_call` → `write-guard` (BLOCKING, fail **closed**), `session_start` →
  `session-init`, `before_agent_start` → `prompt-context`, `tool_result` →
  `state-sync`, `agent_settled` → `session-close` (all four ADVISORY, fail
  **open**). Exactly two policies, never mixed. `bee-guard.ts:16-34, 851-991`
- **Passivity**: no `.bee` directory → the belt does nothing, re-checked every
  call. `bee-guard.ts:45-50`
- **pi-result-mailbox** already solves detached digest loss:
  `bee herding run --inbox-session <token>` drops a marker before the pane
  splits; the belt polls every 2 s, claims by atomic rename, injects a
  400-char header block. At-least-once, `job_id` is the dedupe key, needs a live
  session. `bee-guard.ts:562, 686-775`; `config-reference.md:184`
- **Skills already load**: bee ships `.agents/skills/` (44 skills), which Pi
  discovers as project skills. Pi invokes them as `/skill:<name>`, not
  `/<name>`. `Local` (`.agents/skills/`, `pi/docs/skills.md:26-40, 88-96`)
- **The gap list** bee itself records for Pi: no Agent/Task tool → no
  model-guard, no escalation dispatch; no "ask" permission primitive (Pi
  `tool_call` is two-valued, so bee's `ask` verdicts become blocks); no
  text-injection on `tool_call` (reservation warnings go to stderr, unseen by
  the model); no `activity` / PreCompact / SessionEnd / SubagentStop wiring;
  continuation nudges on `agent_settled` are logged, never enforced.
  `bee-guard.ts:52-64, 145-149, 218-241, 961-967`
- **Prior brief**: `docs/history/research/pi-harness-support.md` (2026-08-29)
  reached "write a Pi extension, not JSON hooks" — correct, shipped. This brief
  extends it; it does not retire it.

### Docs — Pi 0.84.4's real extension surface (version-matched, from the installed binary)

Read from `~/.local/share/mise/installs/pi/0.84.4/pi/docs/` and its shipped
`examples/`.

1. **`pi.registerTool(definition)` works at load time *and* after startup** —
   "inside `session_start`, command handlers, or other event handlers. New tools
   are refreshed immediately in the same session, so they appear in
   `pi.getAllTools()` and are callable by the LLM without `/reload`."
   `pi/docs/extensions.md:1365-1416`
   The definition takes `name`, `label`, `description`, `promptSnippet`,
   `promptGuidelines`, a TypeBox `parameters` schema, and
   `execute(toolCallId, params, signal, onUpdate, ctx)` returning
   `{ content: [{type:"text", text}], details }`. `onUpdate` streams progress.

2. **Pi ships a working subagent extension as an example.**
   `pi/examples/extensions/subagent/` — 1038-line `index.ts` doing
   `pi.registerTool({ name: "subagent", … })` (`index.ts:472-473`) and spawning
   real `pi` child processes (`node:child_process.spawn`, `index.ts:15, 346`)
   with `--model`, `--thinking`, `--tools`, `--append-system-prompt`
   (`index.ts:303-341`). Agent definitions are **markdown with YAML
   frontmatter** in `~/.pi/agent/agents/*.md` (user) and `.pi/agents/*.md`
   (project) — `name`, `description`, `tools`, `model`, body = system prompt.
   Three modes: single `{agent, task}`, parallel `{tasks:[…]}` (max 8, 4
   concurrent), chain `{chain:[…]}` with a `{previous}` placeholder. Streaming
   tool calls, per-agent usage and cost, Ctrl+C propagation, 50 KB per-task
   output cap. `pi/examples/extensions/subagent/README.md`, `index.ts:33-34, 605-645`
   *Note the shape*: it is bee's dispatch contract, already built.

3. **A second, in-process path: the Pi SDK.** pi-workflows creates child
   sessions with `createAgentSession` / `createAgentSessionFromServices` +
   `SessionManager.inMemory` — no subprocess at all, tools restricted per child,
   `session.prompt()` / `session.subscribe()` / `session.abort()`, caps at 8
   agents / 8 concurrent. `Upstream` `pi-agent-group.ts:4-19, 995-1140`;
   `pi-workflows/docs/workflows.md:590`

4. **A third path: `pi --mode rpc` headless children** with a small bridge
   extension. `Upstream` `pi-workflows/src/host/rpc-bridge.ts:10-44`

5. **Injecting work into a live session** — this is what bee's mailbox already
   half-uses: `pi.sendMessage({customType, content, display, details},
   {deliverAs: "steer"|"followUp"|"nextTurn", triggerTurn: true})`.
   `pi/docs/extensions.md:1416-1439`. Operational gotchas pi-workflows learned
   the hard way: `sendMessage` does **not** return the entry id (scan
   `ctx.sessionManager.getBranch()` for your own `details` id afterwards), and
   you must gate on `ctx.isIdle() && !ctx.hasPendingMessages()` before
   injecting. `Upstream` `session-delivery.ts:22-28, 43, 91-100`

6. **Real TUI surface** bee does not touch at all: `ctx.ui.setWidget(key, fn)`
   (component factory returning painted lines, capped at 10 lines),
   `ctx.ui.setStatus`, `ctx.ui.notify`, `ctx.ui.input(prompt, default, {signal})`,
   `ctx.ui.custom()` for a full interactive component, and
   `pi.registerMessageRenderer(customType, renderer)` for collapsible transcript
   cards. Plus `pi.registerCommand(name, {description, getArgumentCompletions,
   handler})` and `pi.registerShortcut`. `Upstream` `session-view.ts:50-94`,
   `decision-channels.ts:173-265`, `step-message.ts:39-43`, `index.ts:173-282`

7. **Distribution**: a repo can be a Pi package — `"keywords": ["pi-package"]`
   plus a `"pi": {"extensions": […], "skills": […]}` key in `package.json`;
   installed with `pi install git:github.com/user/repo`. Users disable one
   bundled skill with `{"source": "…", "skills": ["-skills/monitor"]}`.
   `Upstream/Docs` `pi-workflows/package.json:5-7, 94-101`; `pi/docs/packages.md:5-58`

### Upstream — how pi-workflows builds workflows (the design, not the code)

- **Five primitives, hard-capped.** `agent` (model judgment), `compute` (pure
  local calculation), `action` (external effect, `shell` is its command form),
  `notify` (durable user message), `checkpoint` (human/external input). A new
  primitive is admitted only if composition cannot express it.
  `DESIGN_PHILOSOPHY.md`, `WORKFLOW_UPDATES.md:58-65`
- **A workflow is a TypeScript graph** — `defineWorkflow({name, startAt,
  maxSteps, nodes, edges})`, discovered from `.pi/workflows/*.workflow.ts`,
  loaded with jiti so plain TS runs with no build step. One outgoing edge per
  node; `switch` edges route on a JSON path, including `$result.outcome`
  (`ok|failed|timed_out|cancelled`) so failure gets a route, not a crash.
  Missing case = routing error, not a guess. `workflows.md:35-52, 402-423`
- **The model is a replaceable executor, never the state owner.** An agent step
  ends only when the model calls the `workflow` tool with `{action:"submit",
  step, attempt, output}` and the output passes `validate`. Wrong step id, stale
  attempt id, or failed validation are rejected and retried in-step.
  `workflows.md:193-200, 662-681`
- **Human gates are protected at the store, not the UI.** "The model-facing
  `workflow` tool cannot satisfy a human decision"; "The host assigns the
  source. A workflow or model cannot claim that an answer came from a person."
  A checkpoint **ends the run** in `waiting` — no process, no claim — and the
  human answer starts a linked *continuation run*. `HUMAN_DECISIONS.md:113-128,
  164-171`
- **Durability = one transaction.** claim check → renew that exact live token →
  domain transition → immutable event → revision bump → outbox effects → commit.
  "An expired claim cannot renew itself, even when the owner ID and token hash
  still match." `SQLITE_STATE.md:169-185`; `WORKFLOW_HOST.md:100`
- **Effects are trichotomous**: `idempotentEffect` (safe to retry),
  read-back-provable, or `manualEffect` → `ambiguous`, which parks the run for a
  human. No exactly-once claim is made. `workflows.md:259-262, 317-319`
- **The out-of-process host exists for one measured reason**: an embedded engine
  kept Node's event loop busy, the lease-renewal timer never fired, a 30 s lease
  expired mid-work, and durable state ended self-contradictory.
  `2026-08-30-out-of-process-workflow-host-plan.md:11`
- **Explicit anti-features**: no hidden polling, no implicit retries, no
  model-generated commands ("a command owned by the workflow definition"), no
  invented ETAs, no model self-approval, no source drift mid-run.
  `workflows.md:266-267`; `WORKFLOW_UPDATES.md:444-448`

### Inference — mapping the source onto bee (dependency matrix)

| pi-workflows component | bee today | Verdict |
|---|---|---|
| Graph engine, node primitives, edges | lanes, cells, slices, `bee cells claim-next` | `EXISTS` — bee's is coarser but owns the same job |
| `checkpoint` / human decision gate | gates in `.bee/state.json`, `uat_stop`, `waiting-on set` | `EXISTS` |
| "model cannot answer a protected decision" | "Never approve a gate yourself" + `gate_bypass` | `EXISTS`, as doctrine rather than a store constraint |
| SQLite events + projections + leases | JSON files, reservations, holds, session heartbeat | `CONFLICT` — same intent, weaker atomicity |
| Out-of-process host, socket protocol | none (bee is a CLI invoked per call) | `NEW` — and **not needed**: bee has no long-lived event loop to starve |
| Controllers (reconciliation loops) | `bee herding` control loop | `EXISTS` |
| `notify` durable message | pi-result-mailbox inbox drain | `EXISTS` |
| Live viewer / TUI widget | none on Pi | `NEW` — cheap, high value |
| Subagent dispatch | herding panes only | `CONFLICT` — see Bottom Line |
| Package distribution (`pi install git:…`) | `bee onboard` copies the belt | `NEW` — optional |

### Cross-cutting sweep

Wiring outside the feature folder that any Pi dispatch change would touch —
each one **checked**, none assumed clean:

- `prepare.rs:92, 118-152, 1525-1531, 2881-2886` — the `pi_requires_herding`
  refusal, at both the dispatch door and the wave door.
- `prepare.rs:94-98, 1772-1777` — the pi herding payload's `detached_delivery`
  instruction naming `--inbox-session`.
- `hook_manifests.rs:44-69` — `Runtime` enum is `Claude, Codex` only; Pi and
  OpenCode are named exclusions because their belts are hand-written TS.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — parity test
  derives its rows from the TS source; a new `task` tool needs a row here.
- `bee-guard.ts:312-321` — `PI_BUILTIN_TOOLS` enumerates Pi's 8-tool registry;
  a bee-registered tool is currently an *unknown* tool taking the fail-safe
  write-capable route.
- `bee-guard.ts:52-59` — the model-guard named exclusion, asserted **by name**
  in the parity test. It would stop being vacuous.
- `docs/config-reference.md:180-232, 450` — the `pi_requires_herding` law and
  the `models.pi` copy-sample omission.

### Two smaller finds, checked against the installed binary

1. **The "continuation nudge is unenforceable" gap may be closable.**
   `bee-guard.ts:961-967` records that nothing on `agent_settled` can force the
   session to keep going. That is true of the *event's return value* — Pi 0.84.4
   has no `session_stop`-style `{continue, additionalContext}` result;
   `agent_settled` is notification-only. `Docs`
   `pi/docs/extensions.md:567-578`
   But `pi.sendMessage(…, {triggerTurn: true})` "trigger[s] an LLM response
   immediately" when the agent is idle — and `agent_settled` is exactly the idle
   moment. bee's own mailbox drain already uses this call. `Docs`
   `pi/docs/extensions.md:1416-1439`; `bee-guard.ts:528-530`
   `Inference` — a nudge on `agent_settled` is probably enforceable today. Needs
   a spike; not proven here.

2. **`omp` is a second live target on this machine, and bee has zero support for
   it.** `refs/oh-my-pi` is `@oh-my-pi/pi-coding-agent` v18.0.4 — and the binary
   `omp` **18.1.2** is installed here
   (`/home/thanhsmind/.local/bin/omp`, mise `github:can1357/oh-my-pi@18.1.2`),
   beside `pi` 0.84.4. There is no `.omp/` directory anywhere in bee and no
   `omp` runtime in `RUNTIMES`. `Local`
   It is a fork with `~/.omp/` paths. It has
   what Pi 0.84.4 lacks: a **built-in `task` tool** with markdown agent
   definitions, model-role aliases, spawn-depth guards and an agent `hub`; a
   `session_stop` hook returning `{continue, additionalContext,
   decision:"block"}`; managed `ctx.setInterval`; and a per-tool `approval`
   tier (`read|write|exec`). `Upstream`
   `oh-my-pi/docs/task-agent-discovery.md:26-158`,
   `oh-my-pi/packages/coding-agent/src/extensibility/shared-events.ts:394-403`,
   `…/extensions/types.ts:493-509, 614-668`
   Neither the `packages` settings entry nor the `skills: ["-skills/x"]` disable
   syntax exists there — those are `pi`'s. On omp the fan-out question answers
   itself: the `task` tool is built in.
   **Compatibility runs one way.** omp loads legacy pi extensions (it rewrites
   `@mariozechner/*` imports and shims `ctx.isProjectTrusted()`), so an
   extension written to `pi`'s narrower surface runs on **both**; one using
   omp-native surfaces runs on omp only. `Upstream`
   `oh-my-pi/docs/extension-loading.md:229`, `…/types.ts:526-539`
   `Inference`: `.pi/extensions/bee-guard.ts` as written is probably already
   omp-compatible, but bee never copies it into `.omp/` and never registers an
   `omp` runtime, so nothing loads it. Unverified — no omp session was run.

---

## Five rules worth taking from pi-workflows (this is the real "xia" answer)

None of these need code from the source. All five are cheap in bee.

1. **Declare crash semantics per effect, never assume them.** bee's cells have
   proof lines but no statement of what is safe to re-run after an interrupted
   worker. pi-workflows forces every action to say idempotent / read-back /
   ambiguous, and parks the ambiguous ones for a human instead of retrying.
2. **Waiting work holds no process and no claim.** This one invariant is what
   makes their whole system restart-safe: resume is always a *new generation*
   from the last committed boundary, never process resurrection. bee's holds and
   reservations are close; the invariant is not written down.
3. **Idempotent command receipts.** Same request id + same payload → return the
   stored receipt; same id + different payload → conflict. That is exactly what
   bee's at-least-once mailbox needs so a replayed `job_id` is provably safe.
4. **Split the machine-readable subject from the human presentation, and cap
   it.** Their human gate carries a canonical `subject` (JSON) and a separate
   `presentation` allowlist — 5 block kinds, hard byte caps, digest-bound so a
   changed question makes an old answer stale. bee's gate questions are free
   prose today.
5. **The model may not assert human provenance.** "The host assigns the source."
   bee says this in doctrine (`Never approve a gate yourself`); pi-workflows
   enforces it in the store. bee's `gate_bypass` is the recorded exception —
   which is the right shape, but the enforcement is a hook, not an invariant.

---

## Risks, Unknowns, Follow-Ups

- **A locked decision is contradicted.** `7f9c8518-0d26-4a40-b51a-d18a433e42a2`
  (confidence 90, source: user, 2026-08-29) rests on "Pi 0.84.3 exposes no
  Task/Agent-style subagent tool". That is still true of the *built-in* registry
  at 0.84.4, and three decisions touch it (`4a6e38be`, `4d7438ec`,
  `8650ca7b`). Superseding it was the user's move, and on 2026-09-02 they
  **declined**: `7f9c8518` stands, reaffirmed by **9f5c6d17** with this brief's
  evidence now on the record. **Closed.**
- **Registering a bee tool changes the guard surface.** A `bee_task` tool is a
  write-capable unknown to `mapToolCall` today — it must get an explicit row,
  and model-guard would stop being a named exclusion. That is a parity-test
  change, not a rewrite.
- **Security posture is real, not theoretical.** Pi's own subagent example
  loads only *user-level* agents by default and prompts before running
  project-local agents in untrusted repos. bee's agents live in the repo, so
  bee would be taking the `agentScope: "project"` path deliberately.
- **Version boundary**: everything here is Pi 0.84.x. pi-workflows pins
  `>=0.84.2 <0.85.0`. The SDK path (`createAgentSession`) is the least stable of
  the three — the subprocess path is the safest.
- **Untested claim**: no bee code was written or run against a Pi subagent tool
  in this research. Every API shape is quoted from Pi's own docs or a shipped
  example, but the integration itself is `Inference` until a spike proves it.
- **~~Open question for shaping~~ — ANSWERED**: do the three fan-out paths
  replace herding on Pi, or sit beside it? **Neither.** Herding stays the single
  Pi dispatch path (**9f5c6d17**). The three paths are recorded here as
  evidence, unused.
- **~~Second open question — which binary is "pi"?~~ — ANSWERED**: the `pi`
  binary only, 0.84.x (**5d87f14e**). `omp` 18.1.2 is installed on the
  maintainer machine but is not a bee target. An extension written to `pi`'s
  surface happens to load on omp too; that is free compatibility, never a design
  constraint and never a tested claim.
- **Still open — the one live follow-up**: § Five rules worth taking is
  unshaped. Each of the five is a real change to bee's own doctrine or store
  behavior, so none of them is a docs edit.

---

## Source Pack

**Local files read**
`.pi/extensions/bee-guard.ts`, `.agents/skills/`,
`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, `…/models.rs`,
`…/devtools/hook_manifests.rs`, `packages/bee/hooks/claude-hooks.json`,
`docs/config-reference.md`, `docs/model-presets.md`,
`docs/knowledge/work/pi-result-mailbox/{index,delivery}.md`,
`docs/knowledge/areas/bee-herding/{handing-a-foreign-agent-its-brief,the-run-verb-and-worker-outcomes}.md`,
`docs/history/pi-support/CONTEXT.md`, `docs/history/pi-result-mailbox/CONTEXT.md`,
`docs/history/research/pi-harness-support.md`, `.bee/decisions.jsonl`

**Pi 0.84.4 (installed binary)**
`docs/extensions.md`, `docs/skills.md`, `docs/packages.md`, `package.json`,
`CHANGELOG.md`, `examples/extensions/subagent/{README.md,index.ts,agents/scout.md}`

**Upstream — pi-workflows @ 7c1b460**
`README.md`, `AGENTS.md`, `package.json`, `herdr-plugin.toml`,
`docs/{DESIGN_PHILOSOPHY,workflows,WORKFLOW_COMPOSITION,WORKFLOW_UPDATES,DEFERRED_TURNS,WORKFLOW_STEP_MESSAGES,WORKFLOW_HOST,SQLITE_STATE,HUMAN_DECISIONS,HUMAN_DECISION_PRESENTATIONS,CONTROLLERS,MONITOR,session-event-journal,live-replay-protocol}.md`,
`docs/2026-08-30-out-of-process-workflow-host-plan.md`,
`src/extension/*.ts`, `src/herdr/*.ts`, `src/host/rpc-bridge.ts`,
`src/builtins/pi-agent-group.ts`, `src/workflows/index.ts`, `skills/*/SKILL.md`

**Second target — omp 18.x (`refs/oh-my-pi`, binary `omp` 18.1.2 installed)**
`docs/{extensions,extension-loading,hooks,custom-tools,agent-hub,collab,marketplace,magic-keywords,context-files,skills,approval-mode,task-agent-discovery,rpc}.md`,
`docs/skills/authoring-extensions.md`,
`packages/coding-agent/src/extensibility/extensions/types.ts`,
`…/extensibility/shared-events.ts`, `src/config/settings-schema.ts`
Kept separate on purpose: its native task subsystem is **not** evidence about
Pi 0.84.4.
