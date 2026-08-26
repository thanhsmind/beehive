# bee product description

A written description of the experience of using bee: what the agent sees, what it can do, and exactly what happens when it does it.

## Purpose

bee is, from its user's point of view, a large state chart. The user of bee is a coding agent: an LLM session that runs `bee` commands, gets intercepted by bee's hooks, and coordinates with a human who approves gates. The agent moves through the state chart with invocations — a command run, or a guarded action (a file write, a subagent dispatch) that a hook allows, repairs, or denies. Most of that behavior is defined implicitly, spread across a hand-maintained command registry, fourteen verb-group modules, a family of hook guards, and twenty black-box test files. There is no single place that says, in plain language, "when the agent does X, this is what happens, and this is what happens if a sibling session does Y halfway through."

This project is that place. It describes the full experience an agent has in a host repository freshly onboarded with bee, in the default configuration (`gate_bypass` off, all hooks on, models as seeded), running under Claude Code.

The documents are for people who need to understand or change bee: its developers, writers of its skills, testers, and anyone evaluating whether a behavior is intentional. They are written from the outside in. They describe the experience, not the implementation.

### What this is not

- Not API documentation. The command surface lives in `bee --help --all` and the registry payload; the knowledge bundle under `docs/knowledge/` documents internals.
- Not organized by crate or module. `crates/bee` and `crates/fleet` are not described separately. A single behavior is described once, wherever the agent encounters it.
- Not a technical design document. Where a technical detail is critical to understanding the experience, it appears in a block quote labeled `Technical note:` and nowhere else.

## Conventions

- Describe the experience, not the code. "The claim is refused and the refusal names the session that holds it" rather than "claims.rs returns a conflict error".
- Technical detail goes in block quotes, prefixed with `Technical note:`. Use it only when the mechanism changes what the agent would expect.
- Use sentence case for headings.
- Name the vocabulary consistently. The [glossary](glossary.md) is the source of truth for terms like *the agent*, *the human*, *the store*, *gate*, *cell*, *claim*, *hold*, *deny*, *fail-open*.
- Every document ends with the commit of the beehive repo it was verified against and a list of open questions.
- When a behavior is surprising, say so and say why it is that way if the reason is known. Do not smooth it over.

## The work to be done

Each document describes one feature. Features are large things (the cell lifecycle from claim to cap) or small things (`bee capture add`), but each is described in full, including its edge cases and its interactions with other features.

### Document template

Every feature document follows the same skeleton so that documents are comparable and nothing is skipped.

1. **Summary.** One paragraph describing the feature abstractly. For example: "`bee capture add` files a one-line stub of something that just settled, so the insight survives until a capture session merges it into the knowledge bundle."
2. **The simple case.** The common path in prose.
3. **The interaction, event by event.** The five phases of an invocation: *invoke* (argument parsing, root resolution, what is validated before anything runs); *ends at once* (help, a refusal, a no-op: exit code, stderr vs stdout); *first side effect* (the first state write, the lock taken, the point after which the store has changed); *while running* (what is written when, what a concurrent invocation sees); *finish* (final output, exit code, the timing line, what is on disk). Include a small state diagram (Mermaid `stateDiagram-v2`) of the states the agent passes through.
4. **Modifiers.** A table of the variant axis: `--json`; the gate-bypass level; the store phase (idle / gated / execution-approved / terminal); where it runs (main checkout / feature worktree / granted worktree); who runs it (orchestrator / dispatched worker / a hook). What each does when set at invocation, and whether it can differ mid-flow.
5. **Cancel and interrupt.** The same checklist in every document:
   - The process killed mid-command (Ctrl+C, kill, the terminal closing)
   - The session turning elsewhere mid-flow (compaction, a handoff written, the turn ending before the flow completes)
   - The events bee treats as a clean completion from outside (a gate approved, a question answered, a new user message)
   - The store unavailable (lock contention, corrupt JSON, the hook binary missing)
   - The session going away (heartbeat expiry, a claim's lease running out, `session release`)
   - A sibling changing the target (a claim taken, a hold or reservation appearing, a worktree merged underneath)
   - The channel changing (output piped or `--json`, a different runtime, the command run from inside a hook)
6. **Interactions with other systems.** In this order: gates and approval; the store and history; worktrees and containment; claims, holds, and reservations; sibling sessions; what the human sees; configuration; output modes and exit codes.
7. **Edge cases.** Anything the agent could notice that is not covered above.
8. **Open questions and verification.** The beehive commit the document was verified against, and any behavior that could not be confirmed.

Item 5 matters most. Asking the same interrupt questions of every feature is how gaps and inconsistencies are found.

### Method

For each document:

1. Read the command's registry entry (`packages/bee-rs/crates/bee/src/generated/registry_payload.json`) and its verb module under `packages/bee-rs/crates/bee/src/verbs/`.
2. Read the matching tests in `packages/bee-rs/crates/bee/tests/`. Files like `front_door.rs`, `registry_dispatch.rs`, `concurrency.rs`, and `hook_contracts.rs` are close to executable specifications of the edge cases.
3. Draft the document.
4. Try anything ambiguous against the built binary (`.bee/bin/bee` in a scratch host repo). Tests settle "what happens"; the running binary settles wording, exit codes, and what appears on which stream.
5. Record the commit verified against.

### Verification

Drafting reads the code; verification watches the product. The `verification/` directory holds one checklist per cluster of documents, each item a single observable claim with setup, steps, expected result, a priority, and what it needs. A tester runs them against the built binary in a scratch host repository, records `pass`, `fail`, or `blocked` in the Result column, and files every failure in `bug-triage.md` with the item's ID. A document moves from `drafted` to `verified` in the coverage table only when every P1 and P2 item for it has passed or been filed.

`bug-triage.md` is the other half: every behavior the documents flagged as a likely defect, deduplicated, with reproduction steps, the reason in the code, a severity, and the decision the bee team needs to make. Entries confirmed against the binary carry a Status line.

### Order of work

1. **Pilot: `memory/capture.md`.** Small and self-contained: a leaf subcommand with a real side effect and flags. Used to settle the template, tone, and depth.
2. **Foundations.** `foundations/invocation.md` first; everything refers to it. Then `store.md`, `session.md`, `gates.md`, `guards.md`, `worktrees.md`.
3. **The lifecycle.** The bulk of the experience and the hardest part: six documents that must agree on where one state ends and the next begins. Written third so the template is already proven.
4. **Everything else.** Once the template and exemplars exist, the remaining documents can be drafted in parallel, followed by a consistency pass and a verification pass across the whole set.

Progress is tracked in the [coverage table](#coverage) below.

### Scope decisions

- **Where this repo lives.** The plan was a sibling repository. bee's own write guard refuses every write outside the worktree and refuses `git init` at idle, so the set lives at `docs/product-description/` inside beehive, committed through beehive's git. It can be moved out later; nothing in it links outside itself except source citations.
- **The surface is a fresh host repo, not beehive itself.** beehive runs with `gate_bypass` at normal and its own config; the documents describe the seeded defaults (`gate_bypass: false`, all six hooks on). Where beehive's own setup differs, that is a variant, noted in the document that owns the setting.
- **Runtime is Claude Code.** Codex and OpenCode projections exist; a document notes a Codex difference only where the behavior differs (no SessionEnd event, `spawn_agent` matcher, advisory-only events). OpenCode's hand-written plugin is out of scope.
- **`bee dev *` and `bee rs-info` are out of scope.** They build and inspect bee itself; they are not part of the host-repo experience. `scripts/release.sh` likewise.
- **Skills are described as one layer, not one document per skill.** The skills are instructions to the agent; the observable product is the CLI and hooks. `cross-cutting/skills-layer.md` describes how skills route work and drive commands; the prose of each skill is not restated.
- **The fleet crate has no document.** It is a library inside the binary; its behavior surfaces only through the commands that use it.
- **Interaction shape.** The unit of interaction is an invocation and its phases are *invoke*, *ends at once*, *first side effect*, *while running*, *finish*. The interrupt list and the order of cross-cutting concerns are fixed as written in the document template above.
- **Numbered rules.** These are prose documents, not numbered specifications. Stable heading anchors are enough for cross-references.

## Structure

```
README.md                        this file
goal.md                          the standing instructions for whoever drafts
AGENTS.md, CLAUDE.md             entry points for agents: read README.md, then goal.md
glossary.md                      shared vocabulary
bug-triage.md                    suspected defects collected from every document

verification/
  README.md                      how to run a verification pass and record results
  foundations.md                 checklists for foundations/
  lifecycle.md                   checklists for lifecycle/
  areas.md                       checklists for the remaining areas
  cross-cutting.md               checklists for cross-cutting/

foundations/
  invocation.md                  how a command is parsed, answered, refused; exit codes; --json; the timing line
  store.md                       what bee remembers on disk, the lock, corrupt-read fallback, CLI-only writes
  session.md                     what the harness does around the agent: preamble, hook events, waiting marks, release
  gates.md                       the five gates, the phases, and the bypass levels
  guards.md                      the write-guard family: what is denied, what a deny says, fail-open vs fail-closed
  worktrees.md                   main checkout, feature worktrees, granted worktrees, staging

lifecycle/
  orient.md                      orient, status, route: how a session finds its footing
  shaping.md                     intent, decisions, and the shape gate
  planning.md                    lanes, the plan, cells prepared, the execution gate
  cells.md                       the cell store: add, claim, claim-next, block, escalate, ready, show
  execution.md                   the worker's arc: reserve, implement, prove, cap (finish)
  close.md                       close, scribing and compounding, UAT, worktree merge

delegation/
  dispatch.md                    dispatch prepare and wave: the one door to a subagent
  workers.md                     the rendered agents and the model guard
  herding.md                     the unattended cockpit: bootstrap, dispatch, merge

memory/
  capture.md                     capture add, list, count, flush          (pilot)
  decisions.md                   decisions log, active, search, supersede, redact, archive
  knowledge.md                   the knowledge bundle: search, context, promote, report
  backlog.md                     the backlog: add, propose, rank, pbi
  feedback.md                    feedback collect, digest, rank
  mailbox.md                     the human mailbox

discovery/
  wayfinding.md                  discovery maps, stubs, and triggers

coordination/
  reservations.md                reserve, release, sweep; holds
  sessions.md                    state session bind/list/release, work set/show, multi-session etiquette

observability/
  status.md                      status, doctor, orient's read-only surface
  perf.md                        perf and timings

maintenance/
  onboarding.md                  onboard: what is installed and kept current
  recovery.md                    recovery scan/window, tmp sweep, reservations sweep
  testing.md                     bee test and the verify pipeline

reviews/
  reviewing.md                   reviews create/record/status and the judge

cross-cutting/
  configuration.md               config.json and config.local.json, the merge law, per-hook toggles
  privacy.md                     the secret guard and the privacy marker
  failure.md                     fail-open hooks, corrupt stores, the crash contract
  skills-layer.md                how the skills route the agent onto these commands
```

## Coverage

Status is one of `not started`, `drafted`, or `verified`.

| Document | Status |
| --- | --- |
| glossary.md | drafted |
| bug-triage.md | not started |
| verification/ (4 checklists) | not started |
| foundations/invocation.md | drafted |
| foundations/store.md | drafted |
| foundations/session.md | drafted |
| foundations/gates.md | drafted |
| foundations/guards.md | drafted |
| foundations/worktrees.md | drafted |
| lifecycle/orient.md | drafted |
| lifecycle/shaping.md | drafted |
| lifecycle/planning.md | drafted |
| lifecycle/cells.md | drafted |
| lifecycle/execution.md | drafted |
| lifecycle/close.md | drafted |
| delegation/dispatch.md | not started |
| delegation/workers.md | not started |
| delegation/herding.md | not started |
| memory/capture.md | drafted |
| memory/decisions.md | not started |
| memory/knowledge.md | not started |
| memory/backlog.md | not started |
| memory/feedback.md | not started |
| memory/mailbox.md | not started |
| discovery/wayfinding.md | not started |
| coordination/reservations.md | not started |
| coordination/sessions.md | not started |
| observability/status.md | not started |
| observability/perf.md | not started |
| maintenance/onboarding.md | not started |
| maintenance/recovery.md | not started |
| maintenance/testing.md | not started |
| reviews/reviewing.md | not started |
| cross-cutting/configuration.md | not started |
| cross-cutting/privacy.md | not started |
| cross-cutting/failure.md | not started |
| cross-cutting/skills-layer.md | not started |

## Reference

The source of truth is the beehive repo at `/home/thanhsmind/Projects/goglbe/beehive`. The relevant locations are:

- `packages/bee-rs/crates/bee/src/generated/registry_payload.json`: the command tree — every command's name, parameters, examples, availability (hand-maintained).
- `packages/bee-rs/crates/bee/src/router.rs` and `src/verbs/`: dispatch and the fourteen verb groups; refusal wording at `router.rs:320-326`.
- `packages/bee-rs/crates/bee/src/hooks/`: the harness — session preamble, prompt context, write guard, model guard, state sync, session close.
- `packages/bee-rs/crates/bee/src/state.rs`: gates, config merge, bypass levels, defaults.
- `packages/bee-rs/crates/bee/tests/`: twenty black-box suites over the built binary; `front_door.rs`, `registry_dispatch.rs`, `concurrency.rs`, `hook_contracts.rs`, `proof_gate.rs` read as specs.
- `packages/bee-rs/crates/bee/src/onboard/templates.rs`: the config seeded into a fresh host repo.
- `docs/knowledge/areas/`: sixteen areas of recorded intent — why a behavior is the way it is.
- `.claude/settings.json` and `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs`: which hook fires on which event.
