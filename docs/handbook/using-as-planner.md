# Using the handbook as a planner

This is the handbook's payload: a **read-only navigation guide** for an agent asked
to change the bee harness. Its job is to help you find *every* place a change must
touch — before you edit anything. Modeled on the
[Harness Handbook](https://github.com/Ruhan-Wang/Harness_Handbook) planner: **route
with the handbook, read the real source, then emit a precise EDIT plan.** The
planner never edits — it plans.

> **bee's own twist.** In most codebases the plan is the end of planning and you
> just start editing. Not here. bee governs itself: an EDIT plan against the bee
> harness is an *input to bee's own chain*, not a licence to write. Localize freely
> in read-only mode; then route the actual change through `bee-hive` and its gates
> (see [the handoff](#4-hand-the-plan-to-bees-own-chain)).

## The loop

### 1. Route with the handbook (read-only)
Given a change request, consult — in this order — **without reading source yet**:

1. [index.md](index.md) — which **stage** does this concern? Use the *Route by
   intent* table. A change often spans more than one stage; list them all.
2. The matching [stages/&lt;id&gt;.md](stages/) page(s) — its purpose, inputs,
   outputs, **gate**, **state touched**, and **key rules**. The key rules are where
   most missed edit-sites hide (an invariant the change must preserve or update).
3. [register.md](register.md) — for every `.bee/` file the stage *State touched*
   section names, note the fields your change reads or writes. A change to state
   shape usually touches the CLI verb, the reader, and the schema together.

### 2. Read the real source
The handbook points; the source decides. Open the actual files the stage page's
**Source** line names, plus anything `register.md` pointed you to:

- Stage behavior → `skills/bee-<name>/SKILL.md` (and its `references/`).
- State shape / CLI verbs → the **canonical source** `packages/bee/scripts/bee.mjs`
  + `packages/bee/lib/` (what a host repo runs at `.bee/bin/` is a vendored
  render of it — edit the package, never the render).
- Guardrails → the hook catalog (`.codex/hooks.json`, shipped from
  `packages/bee/hooks/`) and `AGENTS.md` (rendered from
  `packages/bee/AGENTS.block.md` — 14 critical rules in judgement form since
  v1.18.1).
- Cross-cutting law → `AGENTS.md` (auto-loaded) and `docs/knowledge/`.

Never treat a handbook line as the current truth for an edit — it is the map, the
source is the territory. If they disagree, the source wins **and the handbook is
stale** — note it (see [resync](#5-resync-after-the-change-lands)).

### 3. Emit the EDIT plan
Return a plan only — no diff. For each site:

```
EDIT <path>
  where:  <function / section / anchor to locate the change>
  change: <what changes, precisely>
  why:    <the requirement or invariant driving it>
```

Then a **completeness check** — the whole point of a handbook:

```
TOUCHES
  stages:    <every stage page whose behavior/rules this changes>
  registers: <every .bee/ file whose shape/reader/writer this changes>
  law:       <AGENTS.md / hook / critical-pattern lines that must move in lockstep>
  docs:      <this handbook page(s) + docs/knowledge concept(s) that go stale>
```

A change to a SKILL's behavior that forgets its handbook page, its `docs/knowledge`
concept, or a mirrored copy under `.claude/skills/` / `.codex/` is the classic
half-done migration this section exists to catch. Same for the payload: a change
to CLI/hook/onboarding behavior that edits a vendored render instead of
`packages/bee/` (or forgets the release-manifest `package_payload` role) ships
nothing — the next onboard overwrites it.

### 4. Hand the plan to bee's own chain
The EDIT plan is not a green light to write. Route it:

- **Classify the lane** from the plan (risk flags + product-file count — see
  [index.md](index.md#lanes-how-much-of-the-chain-runs)). A change to auth, data
  loss, security, an external provider, or that weakens existing proof is
  **high-risk**, one flag is enough.
- Enter through **[hive](stages/hive.md)** → the chain. The EDIT plan becomes the
  raw material for `CONTEXT.md` (if gray areas remain) or the cells' `action` /
  `files` (if scope is clear).
- **No source edit happens before Gate 3.** The planner's completeness check is
  what makes the cells' `files` lists correct, so nothing is missed at execution.

### 5. Resync after the change lands
Once the real change merges, the handbook is a *derived* layer and must roll
forward — the same discipline bee applies to `docs/knowledge/`:

- Update every `stages/<id>.md` and `register.md` anchor the change altered.
- This is bee's [scribing](stages/scribing.md) job: a settled change to how a stage
  works is captured the moment it settles, never left in chat.
- Keep the handbook honest: a stage page that describes retired behavior teaches
  the next agent the wrong map.

## Guarantees & limits
- **Read-only.** The planner reads and routes; it emits a plan and empty diff.
- **Not authoritative for truth.** On any disagreement, the source wins and the
  handbook is flagged stale.
- **Not a bypass.** The plan still flows through hive, the mode gate, and Gates 1–3.
  Localizing an edit is never approving it.
