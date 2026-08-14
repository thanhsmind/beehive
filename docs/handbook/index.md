# bee harness — index

The routing backbone. Start here, pick the **stage** your change concerns, open
its page, then cross-reference [register.md](register.md) for the state it touches.
New to the system? Read [overview.md](overview.md) first, then
[architecture-map.md](architecture-map.md) — the whole harness in six Mermaid
diagrams: what contains what, the lifecycle and gates, one feature end to end,
lane routing, the memory loop, and the guard layer.

## Stages (the chain, in order)

A **stage** is a phase of the machine (what `.bee/state.json` records); a
**skill** is the instruction set that runs it. The two do not map one-to-one:
one skill can own several stages, so the skill column below is how you get from
a stage name to the file you edit.

| # | Stage | Skill (move) | One line | Gate | Page |
|---|-------|--------------|----------|------|------|
| 0 | **hive** | `bee-hive` | Bootstrap, route, and keep the gates. Loaded first every session. | presents all three | [stages/hive.md](stages/hive.md) |
| 1 | **exploring** | `bee-shaping` (Explore · Qualify · Lock · Brief) | Fuzzy request → locked decisions in `CONTEXT.md`. | Gate 1 | [stages/exploring.md](stages/exploring.md) |
| 2 | **planning** | `bee-planning` | Locked decisions → lane, smallest honest shape, cells — plus the reality check (SMALLER PATH) and the review wave. | Gate 2 (merged shape+execution) | [stages/planning.md](stages/planning.md) |
| 3 | **swarming** | `bee-swarming` | Orchestrate bounded workers over cells opened after Gate 2. | — | [stages/swarming.md](stages/swarming.md) |
| 4 | **executing** | `bee-swarming` ("Execute") | Implement, test, and finish exactly one cell. | — | [stages/executing.md](stages/executing.md) |
| 5 | **scribing** | `bee-capturing` ("Scribe") | Sync durable, tech-agnostic knowledge of every area. | — | [stages/scribing.md](stages/scribing.md) |
| 6 | **compounding** | `bee-capturing` ("Compound") | Capture learnings + decisions; close the feature. | — | [stages/compounding.md](stages/compounding.md) |
| R | **reviewing** | `bee-reviewing` | On-demand independent review gate over a chosen scope. | Gate 3 | [stages/reviewing.md](stages/reviewing.md) |

**Capture** is one skill over two stages: `bee-capturing` runs Scribe (stage 5)
then Compound (stage 6), and there is no separate `bee-scribing` or
`bee-compounding` skill. The stage names stay because the machine still keys on
them — `phase: scribing` / `compounding`, `state scribing-run` /
`compounding-run`, `last_scribing_run` / `last_compounding_run`.

Stages 5–6 are **deferred by design**: a green `bee close` records capture as
pending and names what remains; they run at the owner's pace, often batching
several closed features into one session.

On-demand side steps (not chain stages, so no dedicated page): the Brief move
of `bee-shaping` (render one implement plan), `bee-grooming` (hunt tech debt),
the Qualify move of `bee-shaping` (unattended triage of a backlog row),
`bee-researching` (research scout), `bee-herding` (the autonomous cockpit), and
`bee-hive`'s "Gates" section (toggle gate autopilot).

## Maintainer guides (developing bee itself, not shipped product)

- [writing-skills.md](writing-skills.md) — how bee skills are written and
  pressure-tested (TDD-for-skills; supporting material in
  [writing-skills-references/](writing-skills-references/)).
- [evolving.md](evolving.md) — how bee's gated self-improvement loop runs over
  its collected feedback digest (Gate A / Gate B, bee repo only).

## Route by intent

| Your change is about… | Go to |
|-----------------------|-------|
| How a request is classified into a lane/mode; onboarding; the gates | [hive](stages/hive.md) |
| How gray-area product decisions get resolved and locked | [exploring](stages/exploring.md) |
| How the lane is routed, the work shaped, cells created; the reality gate | [planning](stages/planning.md) |
| How workers are dispatched, reserved, and their results judged | [swarming](stages/swarming.md) |
| How one cell is implemented, tested, finished, committed | [executing](stages/executing.md) |
| How specs/knowledge stay current after behavior changes | [scribing](stages/scribing.md) — `bee-capturing` "Scribe" |
| How learnings/critical patterns/decisions are captured; feature close | [compounding](stages/compounding.md) — `bee-capturing` "Compound" |
| How an independent review is run, findings graded, merge approved | [reviewing](stages/reviewing.md) |
| A runtime file's schema (`state.json`, `cells`, `decisions.jsonl`, …) | [register.md](register.md) |
| The CLI's two surfaces, the flow verbs, the refusal taxonomy | [register.md](register.md#the-cli--how-registers-are-mutated) |
| Which door refuses an operation, and what its one escape hatch is | [register.md](register.md#the-doors-that-refuse) |
| Multisession state: workflow records, leases, handoff mailboxes, worktrees | [register.md](register.md#the-control-plane-beeruntime) |
| Craft content — how to plan, test, review, document, debug well | `expertise/` (routed from its own `INDEX.md`) — never a stage page |
| Using this handbook to localize an edit before touching code | [using-as-planner.md](using-as-planner.md) |

**Which layer?** Before editing anything, answer it: flow, state, gates, proof
and context assembly belong to the **machine** (the Rust CLI + hooks); judgment
about how to do the work belongs to **craft** (`skills/`, `expertise/`); why bee
is the way it is belongs to **memory** (`docs/`). A rule the machine already
enforces is deleted from prose, not restated in it.

## The gates (who owns each)

Gates are presented and enforced by **hive**, but each is *earned* at the end of a
particular stage:

- **Gate 1** — earned by **exploring**: "Decisions locked. Approve CONTEXT.md before planning?"
- **Gate 2** — earned by **planning**: the old shape and execution approvals merged into one question that approves both together (`bee gate --merge`, flipping `approved_gates.shape` and `approved_gates.execution` in one call) *(no source edits before this)*
- **Gate 3** — earned by **reviewing** only: P1>0 → "P1 findings block merge. Fix before proceeding?"; P1=0 → "Review complete. Approve merge?"

Every lane merges the old shape and execution approvals into one question:
tiny/small ask it inline before cells persist, standard/high-risk ask it once
shape and the brief are ready. The docs lane stops at Gate 1 only — a short brief
and a one-line approval — with no Gate 2 and no cells.

A gate is a record, not a boolean: approving one writes `state`
(`pending` / `approved` / `rejected`), `actor` (`user` / `auto`), a timestamp, a
reason, and the bypass level in force. A new feature starts with every gate at
`pending`. `gate_bypass` (in [config.json](register.md#beeconfigjson)) can
auto-approve gates by level (`normal` / `full` / `total`) — it decides whether the
run **stops**, never whether the brief and approval record **exist**.

## Lanes (how much of the chain runs)

| Lane | Trigger (from the request alone) | Stages that run |
|------|----------------------------------|-----------------|
| `docs` | every touched file is knowledge, not runtime | brief → Gate 1 → write → format-check → capture |
| `tiny` | 0–1 risk flags, ≤2 product files, one direct task | hive → merged gate → one cell (may run inline) |
| `small` | 0–1 flags, ≤3 product files, no gray areas | hive → merged gate → dispatched execution worker(s) |
| `standard` | 2–3 flags, or story-sized behavior | full chain, Gates 1–2 |
| `high-risk` | 4+ flags **or any hard-gate flag** | full chain + brief + persona panel, Gates 1–2 |
| `spike` | one yes/no proof decides whether the plan is real | disposable feasibility proof, then re-route |

Risk flags (counted mechanically): auth · authorization · data model ·
audit/security · external systems · public contracts · cross-platform · changing
behavior a test asserts · weakening/deleting existing proof · multi-domain.
Hard-gate flags (any one forces `high-risk`): auth · authorization · data loss ·
audit/security · external provider · validation removal.
**Product files** exclude `.bee/**`, `docs/**`, plans, and generated renders.

A code-touching route creates the feature's worktree in the same step
(`bee worktree new --feature <slug>`) and the work lives there; `docs` and a solo
`tiny` stay in the main checkout.
