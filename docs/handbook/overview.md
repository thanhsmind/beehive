# bee harness — system overview

> This handbook turns the bee harness into a navigable reference. Read the
> [index](index.md) to route to a **stage**, read [register.md](register.md) for
> the shared **state** each stage reads and writes, and read
> [using-as-planner.md](using-as-planner.md) to use the handbook the way a code
> agent should: locate every place a change must touch *before* editing.
>
> Format follows the [Harness Handbook](https://github.com/Ruhan-Wang/Harness_Handbook)
> convention — `overview → index → register → stages/<id>` — mapped onto bee's
> own architecture: the **chain is the set of stages**, and the `.bee/` runtime
> files are the **state registers**.

## What bee is

bee is a **workflow harness** for AI coding agents. It is not an application with
users and features of its own — it is the operating discipline a coding agent runs
*inside* when it works on a host project. Its job is to make an agent's work on a
codebase safe, resumable, and reviewable: turn a fuzzy request into locked
decisions, scale ceremony to real risk, gate the irreversible steps behind human
approval, and keep a durable memory of what settled so the next session starts
smarter.

bee ships as five things working together:

1. **Skills** (`skills/<name>/SKILL.md`) — the phases of the workflow, instruction
   content only (SKILL.md + references + scripts). Each skill is a self-contained
   instruction set the agent loads when the workflow routes to it. `bee-hive` is
   the router; the rest are the chain stages.
2. **The payload package** (`packages/bee/`, since v1.18.0) — the single standard
   code set: the CLI source (`scripts/bee.mjs`), the onboarding/distribution
   engine (`scripts/onboard_bee.mjs`), `lib/`, `hooks/`, `agents/`, `statusline/`,
   `AGENTS.block.md`, and its own `tests/`. Install resolves everything from here;
   what lands in a host repo is a vendored render of this package.
3. **A single CLI** (`.bee/bin/bee.mjs` — the vendored render of
   `packages/bee/scripts/bee.mjs`) — every state read and mutation goes through
   this one dispatcher across nine command groups. State is *never* hand-edited.
4. **Runtime state** (`.bee/*.json`, `.bee/*.jsonl`, `.bee/cells/`,
   `.bee/runtime/`) — the [state registers](register.md): workflow records, phase,
   gates, feature, cells, decisions, leases, backlog, handoff mailboxes.
5. **Hooks** (`.codex/hooks.json` catalog, 8 lifecycle events, shipped from
   `packages/bee/hooks/`) — a fail-open safety net that catches forgotten rules.
   The hook is a net, *not* the authority: an unblocked write is not an approved
   write.

## The core model

**One orchestrator, many I/O workers (the Delegation contract).** The session model
is the orchestrator — it decides. Mechanical gather/render/mine steps are dispatched
*down-tier* to worker subagents that read many files and return a compact digest, so
the orchestrator's scarce context window is spent on synthesis, gates, and human
conversation — never on raw file dumps. Deciding never delegates; gathering almost
always does.

**Lanes scale ceremony, never memory.** The same request can be a two-minute `tiny`
fix or a full `high-risk` feature. bee classifies the lane mechanically (risk-flag
count + product-file count) and runs the *least* workflow that honestly protects the
work. What never scales down is memory: a rule, behavior, or value that just settled
is captured the moment it settles, in every lane.

**Gates are the human checkpoints.** Three approval gates fence the irreversible
transitions — Gate 2 now approves shape and execution together in one call
(`bee state gate --merge`), folding the old standalone Gate 3 into it. They are
never self-approved — except when the opt-in `gate_bypass` switch is
deliberately set by the human (levels: `normal` / `full` / `total`).

**Knowledge over history.** The state layer an agent reads *first* is the knowledge
bundle (`docs/knowledge/`) when the repo has one, or `docs/specs/` otherwise.
`docs/history/` is archaeology, read last.

**Workflow-first, multisession-native (v1.17.0).** The source of truth for a
running workflow is its own record (`.bee/runtime/workflows/<wf-id>/state.json`);
legacy `.bee/state.json` is a read-only projection. State splits into a **control
plane** shared across worktrees (workflow records, sharded leases, handoff
mailboxes, cross-worktree holds) and a **data plane** isolated per worktree.
Sessions coordinate through leases, claims, and holds — never around them; new
feature work in an occupied checkout goes through `bee worktree new` /
`bee worktree merge`. Active workers are *derived* (live-heartbeat sessions joined
with cell claims), never stored.

**Proof scales with the change (test-economy, v1.17.1).** The evidence `cells cap`
demands is derived from `change_class × lane` — red-first proof is mandatory only
for `security`/`migration` classes and the `high-risk` lane; a covered bugfix at
tiny/small needs a targeted green test, not ceremony. The dev loop runs impacted
tests only (capped, transitive tail delegated); the full verify suite is CI-owned,
and a red CI run files a `verify-red` issue — never build on red.

## Architecture at a glance

```
skills/                     the workflow, one SKILL.md per phase (instructions only)
  bee-hive/                 router + gate keeper + onboarding  → stages/hive.md
  bee-exploring/            fuzzy request → locked CONTEXT.md   → stages/exploring.md
  bee-planning/             mode + shape + reality check + cells → stages/planning.md
  bee-swarming/             orchestrate bounded workers         → stages/swarming.md
  bee-executing/            implement + verify + cap one cell   → stages/executing.md
  bee-scribing/             sync durable knowledge              → stages/scribing.md
  bee-compounding/          capture learnings + decisions       → stages/compounding.md
  bee-reviewing/            on-demand independent review gate    → stages/reviewing.md
  (plus on-demand: bee-briefing, bee-grooming, bee-qualifying,
   bee-xia, bee-bypass-gate; maintainer guides for developing bee
   itself live in docs/handbook/writing-skills.md and
   docs/handbook/evolving.md)

packages/bee/               the payload package (v1.18.0) — single standard code set
  scripts/bee.mjs           CLI source (vendored into host as .bee/bin/bee.mjs)
  scripts/onboard_bee.mjs   onboarding + distribution engine
  lib/ · hooks/ · agents/ · statusline/ · tests/ · AGENTS.block.md

.bee/
  bin/bee.mjs               the single CLI, vendored render (9 command groups) → register.md
  runtime/workflows/<wf-id>/state.json  workflow record — SOURCE OF TRUTH → register.md
  runtime/leases/           sharded cell/path leases (control plane)     → register.md
  runtime/handoffs/<wf-id>/ per-workflow handoff mailbox                 → register.md
  state.json               read-only projection: phase · gates · feature → register.md
  config.json              commands · hook toggles · gate_bypass · models → register.md
  cells/<feature>-<n>.json  one unit of executable work          → register.md
  decisions.jsonl          append-only decision log             → register.md
  reservations.json        compat mirror of the lease store      → register.md
  backlog.jsonl            friction events + PBI records         → register.md
  HANDOFF.json             legacy pause/resume projection        → register.md
  onboarding.json          onboarding state + managed versions   → register.md

docs/
  knowledge/               the state layer (read FIRST)
  specs/                   read-only compatibility surface
  history/<feature>/       CONTEXT.md · plan.md · reports/ (archaeology)
  handbook/                ← you are here
```

## The chain (stages)

```
bee-hive  ─ route ─▶  exploring  ─[Gate 1]─▶  planning  ─[Gate 2]─▶  swarming
                                                                          │
   compounding  ◀─  scribing  ◀─  executing  ◀───────────────────────────┘

   on user request only:  reviewing  ─[Gate 4]─▶  merge
```

- **Gate 1** — "Decisions locked. Approve CONTEXT.md before planning?"
- **Gate 2** — approves shape and execution together (`bee state gate --merge`,
  folding the old standalone Gate 3 into this one call) *(no source edits before
  this)*
- **Gate 4** — merge approval, and it lives **only** inside a review session the user
  explicitly asked for. It is never an automatic end-of-chain step.

Every lane merges the old Gate 2 and Gate 3 into one shape+execution question;
the docs lane has no gates at all. See each stage page for its lane behavior.

## How to read this handbook

1. Start at [index.md](index.md) — pick the stage your change concerns.
2. Read that `stages/<id>.md` — what the stage does, what it reads and writes, its
   gate, and its hard rules.
3. Cross-reference [register.md](register.md) for any `.bee/` file the stage touches.
4. Then read the **real source** (`skills/<name>/SKILL.md`, `.bee/bin/`), and only
   then emit an edit plan — see [using-as-planner.md](using-as-planner.md).
