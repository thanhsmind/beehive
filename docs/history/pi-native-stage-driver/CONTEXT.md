# Pi native stage driver — Context

**Feature slug:** pi-native-stage-driver
**Date:** 2026-09-18
**Shaping session:** complete
**Scope:** Deep
**Domain types:** CALL | RUN

## Feature Boundary

A Pi leader session runs every bee stage dispatch — gather, extraction, advisor,
hat seat, reviewer, cell — as a child `pi` process with no tmux pane required,
and gets each worker's full answer back inside the stage budget; herding stays
configured and serves every dispatch the native path cannot. bee's Pi extension
separately narrows the model's tool list per stage and warns on a close that
leaves a claimed cell uncapped. It ends at the transport and the stage surface:
bee's store, gates, cells, proof and worktrees are unchanged.

**Amended 2026-09-19, after the plan-step hat wave** (decision `31fb9e15`,
supersedes `0d11a415`): the child is spawned in Rust, inside `bee herding run`,
NOT by the Pi extension. The extension gains no dispatch tool and no spawner.
See D11. The rest of this boundary is unchanged.

## Why now

Five Pi features shipped (`pi-stage-dispatch`, `pi-run-friction-fixes`,
`pi-harness-workflow-parity`, `pi-full-workflow-parity`,
`pi-parity-review-fixes`) and the owner still reports that bee stages run badly
on Pi. The common cause is the transport: every fan-out pays a tmux pane, a
foreground budget and a hand-read report path. `docs/history/research/pi-workflows-xia.md`
(2026-09-02, confidence 92) already proved Pi's extension API carries the missing
surface and that bee ships the extension to hang it on; the owner declined that
recommendation then and accepted it on 2026-09-18 with both options costed.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

Decision log: `0d11a415` (architecture, supersedes `9f5c6d17`), `7c980c9c`
(product shape). Decision `a20cf301` D9 ("Pi worker dispatch stays herding-only")
is stale from `0d11a415` forward and must be read with it.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Native dispatch is the DEFAULT on Pi; herding is the fallback, not removed. Every kind resolves native first. Herding serves what native cannot: a write-capable cell in a worktree, a seat the user wants to watch live, and every role whose configured agent is not a `pi` binary. Both transports stay tested. | `7c980c9c`. Removing herding is not available: `team.pi` routes `code`, `read`, `test`, `docs`, `extraction`, `generation`, `supervisor` and `lane-3` to `agy-flash`, and a native child can only ever be a `pi` model. |
| D2 | The native transport is a child `pi` SUBPROCESS (`pi --mode json -p …`), never the in-process SDK (`createAgentSession`) and never a `--mode rpc` bridge. **Amended by D11: the subprocess is spawned from Rust, inside `bee herding run`.** | `pi-workflows-xia.md:347` — the SDK is the least stable of the three and ships no `dist/*.d.ts` on this host to pin against; the subprocess is CLI-flag surface bee already drives through herding panes, and Pi ships a working example at `pi/examples/extensions/subagent/`. D2 never said WHO spawns it, which is why D11 amends rather than supersedes it. |
| D3 | A role reaches the native path only when its configured agent is a `pi` binary. Any other agent falls back to herding, by config, with no leader choice and no warning treated as an error. | D1. Keeps the Delegation contract's rule that transport is config, never the leader's choice. |
| D4 | Per-stage tool gating is a HARD gate: the extension narrows the model's active tool list with `pi.setActiveTools` so an off-stage tool cannot be called at all. One slash command re-opens the full set. | `7c980c9c`. Verified reachable: pi 0.85.1 `docs/extensions.md:1694` switches to a read-only pair; the additive-only rule at `:2372` binds a loader tool's own execution, not an event handler. Matches the existing fail-closed `tool_call` write-guard and stops the model burning turns on calls the guard then denies. |
| D5 | The close guard is WARN ONLY. An uncapped claimed cell at settle time gets a visible warning naming the cell and the verb to run; the session still ends. `tool_call` stays the ONLY blocking surface in the Pi belt. | `7c980c9c`. pi 0.85.1 ships no `session_stop` event (zero hits in its `extensions.md`), so blocking the close is not reachable at all; `agent_settled` is the settle event and is already wired. Preserves the belt's documented two-policy rule. |
| D6 | bee's `.bee/*.json` stays the single store. The pi-workflows engine is NOT adopted: no second state layer, no SQLite host, no new third-party npm runtime dependency. | `0d11a415`. bee already owns lanes, cells, gates, decisions, proof and worktrees in Rust. |
| D7 | `.pi/extensions/bee-guard.ts` is hand-written and IS the source of truth. Edits land there, and the Rust binary is rebuilt so `doctor`'s compiled byte-compare agrees. `.opencode/plugins/bee-guard.ts` is a sibling belt and is not edited for this feature. | No generator exists: `hook_manifests.rs:46-64` names Pi a NAMED EXCLUSION, `devtools/mod.rs:529-547` returns `None` for `"pi"`, and `bee dev regen` writes nothing here. `doctor.rs:47` embeds the file with `include_str!` and byte-compares it at `:310-312`, so a stale binary reports drift. |
| D8 | The dispatch door stays ONE door. `bee dispatch prepare --runtime pi` gains a native arm beside the herding arm; the leader never picks the transport and never hand-picks a model or `subagent_type`. | `prepare.rs:2237-2243` currently refuses every non-herding resolution for Pi with `reason: "pi_requires_herding"`. That refusal becomes a per-slot arm. |
| D9 | Existing Claude, Codex and OpenCode behavior does not change. No shared payload, prompt or hook route changes shape for them. | Three belts share `bee hook <name>`; a regression there is a harness-wide outage. |
| D10 | The hat wave contract holds unchanged on the native path: 3 seats default and 5 on high-risk, one wave, a 10-minute wall-clock ceiling, each result named by its seat, and a seat that misses the ceiling DROPPED and named — never silently lost. | `skills/bee-hive/references/gates-and-delegation.md` ("Hat wave"); already proven on the herding path by `pi-hat-wave` in the verification map. Under D11 this is inherited, not rebuilt: `--seat`, the 600 s clamp and the drain already live on that path. |
| D11 | The child `pi` process is spawned in Rust, inside `bee herding run`, as a no-pane runner that keeps the same job id, `--seat`, ceiling, mailbox report and result drain. The dispatch door payload shape does not change. `.pi/extensions/bee-guard.ts` gains NO dispatch tool and NO spawner, so the belt keeps its recorded premise that Pi has no native subagent surface and model-guard stays a named exclusion on it. The belt is still edited for D4 and D5, which are session-surface behavior, not dispatch. | Decision `31fb9e15`, from the plan-step hat wave. The belt-hosted shape owed six blockers; this one removes five of them by inheriting machinery that already ships. `bee-guard.ts:52-59` records why: a `bee herding run` call is a bash CLI call, already covered by write-guard. |
<!-- bee:not-a-deferral: The flagged phrase in this row is "deferred loading", the name of a Pi runtime feature, quoted from the installed host's docs/extensions.md section "Fallback behavior". It is a technical term describing why the hard gate costs cache, not a deferral of work: D12's three obligations all shipped in cell pnsd-6. -->
| D12 | The hard tool gate carries three obligations: the re-open slash command is named outright in the plan, the narrowing is announced to the USER where it happens, and the MODEL is told a tool was removed by stage policy. | Decision `2131829f`. Verified in the host's `docs/extensions.md` § "Fallback behavior": a non-additive active-tool change drops deferred loading and may invalidate the provider's cached prompt prefix. The owner kept the hard gate with that cost stated, on the condition the failure becomes legible — today the model apologizes, hallucinates, or falls back to `bash` redirection that trips write-guard, and the user reads that as the model being broken. |
<!-- /bee:not-a-deferral -->
| D13 | The close-guard warning is written into the visible session transcript, never raised only as an ephemeral UI toast. | Decision `2131829f`. `ctx.ui.notify` is a no-op when a session has no UI (`-p` and JSON modes), and a toast is not where a user looks. |

### Agent's Discretion

Planning picks: the registered tool's name and parameter schema and its explicit
`mapToolCall` row; the child's flag set (`--tools`, `--model`, `--thinking`,
`--append-system-prompt`, `--cwd`); the concurrency limit and the abort mechanism
for the ceiling; the config spelling of a native slot; the stage→tool-set table
and the re-open command's name; and the warning text at settle. Existing Claude,
Codex and OpenCode behavior must not change except where D9 already allows.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| native dispatch | A bee worker run as a child `pi` subprocess started by `.pi/extensions/bee-guard.ts`, with no tmux pane. |
| herding fallback | The existing `bee herding run` tmux-pane transport, kept configured and serving every dispatch native cannot (D1, D3). |
| stage gate | The per-stage narrowing of the model's active tool list via `pi.setActiveTools` (D4). Distinct from a bee gate, which is a user approval. |
| close guard | The warn-only check at `agent_settled` for a claimed-but-uncapped cell (D5). |

## Specific Ideas And References

- Pi's shipped subagent example, `pi/examples/extensions/subagent/index.ts` — the working
  precedent for D2: `registerTool` at `:472`, child spawn at `:300-307, 346-350`,
  parallel max 8 with 4 concurrent at `:33-34`. Read it before designing the tool.
- `refs/ak-pi-workflow-roles` ADR 0010 — retired two orchestrators as "rigid and
  heavy". Read as the standing warning against encoding bee's lanes as a fixed graph.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `.pi/extensions/bee-guard.ts` — the belt. Factory at `:2069`. It registers **twelve**
  `pi.on` handlers, not five: `tool_call` `:2073`, `session_start` `:2098`,
  `before_agent_start` `:2136`, `tool_execution_start` `:2183`, `ui_prompt_start` `:2201`,
  `ui_prompt_end` `:2223`, `tool_result` `:2248`, `agent_settled` `:2305`, `turn_end` `:2403`,
  `session_tree` `:2407`, `session_before_compact` `:2414`, `session_shutdown` `:2434`.
  (An earlier revision of this file listed six and the plan said five; the hat wave caught
  the undercount, which would have let a worker rationalize editing the ones nobody named.)
  Commands `bee-worktree-*` at `:2482-2689`; `bee-worktree-exit` `:2566` is the house style
  for a `registerCommand`.
- `packages/bee-rs/crates/bee/src/herding/run.rs` — under D11 this is where the work lands.
  `:2499` sets `BEE_HERDING_WORKER=1` in the pane env; `:2482-2500` builds the per-agent
  environment explicitly; `--seat` rides the inbox marker and the result envelope.
- `packages/bee-rs/crates/bee/src/hooks/mod.rs` — `:91` reads that marker and `:145-146`
  make every hook invocation except `activity` exit 0 under it. This is the worker posture
  a belt-spawned child would NOT have had.
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — the door. `DISPATCH_RUNTIMES` `:101`,
  the Pi herding-only refusal `:2237-2243`, the herding payload builder `:2444-2530`,
  the pi-only `detached_delivery` note `:163`, hat ceiling 600 s `:2477-2482`.
- `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` — parses the belt source with
  `include_str!` `:107` and derives `PI_BUILTIN_TOOLS` `:117-145`, `mapToolCall` cases
  `:160-215`, `runAdvisoryHook` names `:265-283`, `pi.on` names `:285-313`,
  `registerCommand` names `:315+`.
- `.bee/verify/verify-app/features/pi-runtime.md` and `pi-hat-wave.md` — the two mapped
  features this feature CHANGES. Not a new door; both recipes must still pass.

### Established Patterns

- Exactly two failure policies, never a third: BLOCKING on `tool_call` only
  (`block()` at `:170`), ADVISORY everywhere else (`runAdvisoryHook` swallow at `:316-320`).
- `mapToolCall` never returns null (`:392-398`); its `default:` arm is the fail-safe
  write-capable route, and `pi_plugin_contracts.rs:1821-1845` asserts that arm has no
  `return null/undefined` and at least two `hook: "write-guard"` literals.
- Store and binary are re-located on EVERY call, never cached (`:102-164`), so an
  in-session `bee onboard` starts guarding without a `/reload`.
- `bee:only <runtime>` blocks carry runtime-specific skill text; rendered roots strip the others.

### Integration Points

- `pi.sendUserMessage` already has three callers — drain injection `:913-916`,
  relocation self-dispatch `:2329-2331`, close-verdict nudge `:2377`. A fourth caller
  must set `turnStartPending = true` first (as `:911`, `:2375`) or the drain at `:858`
  opens a second turn.
- The result-inbox drain `:580-981` runs on a raw module-level timer parked in
  `globalThis[Symbol.for("bee.pi.result-drain")]` `:669`. pi 0.85.1 has no
  `ctx.setInterval`, so a throw in that callback can tear the session down.
- `agent_settled` `:2305-2400` already does five things in order; anything added there
  needs its own `try`.
- `tool_result` `:2283-2295` returns a `{ content }` patch for shell markers only —
  a second returner must merge, never replace.

## Canonical References

- `skills/bee-hive/references/gates-and-delegation.md` — "Hat wave" (budget, seats,
  drop-and-name) and "Delegation contract" (worker contracts, transport is config).
- `docs/history/research/pi-workflows-xia.md` — the three Pi fan-out paths and their
  stability ranking; § "Five rules worth taking".
- `docs/history/pi-stage-dispatch/CONTEXT.md` — D1..D8 still hold; D9 is stale per `0d11a415`.
- `/home/thanhsmind/.local/share/mise/installs/pi/0.85.1/pi/docs/extensions.md` — the
  installed host's own API reference. The `refs/oh-my-pi` mirror is a FORK: it documents
  `session_stop` and `ctx.setInterval`, which pi 0.85.1 does not have.

<!-- bee:not-a-deferral: These sections are CONTEXT.md's own record of what was resolved and what was consciously left out of scope. Every open item is answered inline, carried by a cell, or filed as a backlog row; none is an unregistered promise to act later. -->
## Outstanding Questions

### Resolve Before Planning

None. Every product decision is locked above.

### Deferred To Planning

- [x] **Answered by D11.** Which ONE path returns a worker's result to the leader? — the
      existing result drain, inherited from the herding path. A tool return would have held
      the leader's turn for the whole ceiling; the drain is non-blocking by construction and
      already ships `green:live`.
- [x] **Answered by D11.** Does the child carry a reservation identity and a worker posture? —
      yes, by inheritance: `herding/run.rs:2482-2500` builds the per-agent env and sets
      `BEE_HERDING_WORKER=1`, which `hooks/mod.rs:91,145` read to mute leader-only hooks.
- [ ] **Still open, and owed either way.** How does the runner decide "this agent is a `pi`
      binary"? `.bee/config.json` `herding.agents` carries two shapes — a bare argv array
      (`pi-gpt-5.6-luna`) and an object with an `argv` key (`agy-flash`). A naive
      `agents[name][0]` is right only by luck today. The runner must also lift the model out
      of that argv. Planning must name the shape rule and the model extraction explicitly.
- [ ] Does the write guard actually ENFORCE inside a child, not merely load? — The probe run
      proved loading only; it used `--tools read` and an explicit "do not use any tool", so a
      denied write was never attempted. `write_guard/checks.rs:638-643` reaches `Allow` when no
      agent name is present. Needs its own deny-path run before the gate, as `green:live`.
- [ ] Does a `pi -p` child spawned by the runner need a recursion fence? — Under D11 the child
      has no dispatch tool, so the belt-hosted recursion hazard is gone; confirm that the child's
      `--tools` allowlist and the absent tool together close it, rather than assuming they do.

## Deferred Ideas

- The five design rules from `pi-workflows-xia.md` § "Five rules worth taking" —
  crash semantics per effect, waiting work holds no claim, idempotent command receipts,
  machine subject split from human presentation, host assigns provenance. Each is a
  change to bee's own doctrine or store, not to the Pi belt. Still unshaped.
- `bee-guard.ts:1955-1985` uses `os`, `fs` and `require` that the module never imports,
  so the Antigravity usage limits are silently dead. Filed as a P3 finding in
  `.bee/backlog.jsonl`; needs its own red-first test.

<!-- /bee:not-a-deferral -->

<!-- bee:not-a-deferral: The handoff note names the Deferred To Planning section as an input for the reader. It says where to look; it defers nothing itself. -->
## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
<!-- /bee:not-a-deferral -->

