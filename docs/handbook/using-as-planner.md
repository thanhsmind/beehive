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

## 0. Which layer is this?

Answer before anything else, because it decides *what kind of edit exists*:

| If the change is about… | It belongs to | And it lands in |
|---|---|---|
| flow, state, gates, proof, context assembly, a refusal's wording | the **machine** | `packages/bee-rs/crates/bee/src/` |
| how the work is done well — judgment, craft, method | **craft** | `skills/` or `expertise/` |
| why bee is the way it is | **memory** | `docs/` |

Two consequences that catch most half-done changes:

- **A rule the machine enforces is deleted from prose, not restated in it.** If
  your change adds a check to the CLI, the corresponding sentence leaves the skill.
- **A skill that must sequence three CLI calls to express one intent is a missing
  verb.** That is a machine change wearing a prose change's clothes.

## 1. Route with the handbook (read-only)
Given a change request, consult — in this order — **without reading source yet**:

1. [index.md](index.md) — which **stage** does this concern? Use the *Route by
   intent* table. A change often spans more than one stage; list them all.
2. The matching [stages/&lt;id&gt;.md](stages/) page(s) — its purpose, inputs,
   outputs, **gate**, **state touched**, and **key rules**. The key rules are where
   most missed edit-sites hide (an invariant the change must preserve or update).
3. [register.md](register.md) — for every `.bee/` file the stage *State touched*
   section names, note the fields your change reads or writes. A change to state
   shape usually touches the verb, the reader, and the projection together.

## 2. Read the real source
The handbook points; the source decides. Open the actual files the stage page's
**Source** line names, plus anything `register.md` pointed you to:

- Stage behavior → `skills/bee-<name>/SKILL.md` (and its `references/`).
- Craft content → `expertise/<guide>.md` (source), never the vendored
  `.bee/expertise/` render.
- **CLI verbs, state shape, guards, onboarding →
  `packages/bee-rs/crates/bee/src/`**: `router.rs` for the front door (flow-verb
  aliases, the `internal` namespace, the refusal taxonomy), `verbs/<group>/` for a
  command, `hooks/<name>.rs` for a guard, `onboard/` for the installer,
  `state.rs` / `lease_store.rs` / `lock.rs` for the store layer. Read the
  **provenance header** at the top of a ported module before its code — it names
  the contract the module preserves and *why* the rule exists.
- Vendored payload assets (`AGENTS.block.md`, `prompts/`, `agents/*.tmpl`, the hook
  catalogs, `statusline/`) → `packages/bee/`. What lands in a host repo is a render;
  edit the source, never the render.
- Cross-cutting law → `AGENTS.md` (auto-loaded, rendered from
  `packages/bee/AGENTS.block.md`) and `docs/knowledge/`.

Never treat a handbook line as the current truth for an edit — it is the map, the
source is the territory. If they disagree, the source wins **and the handbook is
stale** — note it (see [resync](#5-resync-after-the-change-lands)).

## 3. Emit the EDIT plan
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

Sites that are missed most often, in this tree:

- **The registry.** A new or changed command is not just a handler: it has a
  registry entry (name, `invoke`, description, JSON-Schema `parameters`, examples,
  and its `surface: porcelain | plumbing`), and the embedded payload carries a
  `--check` that fails loudly when it drifts.
- **The generated artifacts.** The knowledge index, the release manifest, and the
  registry payload each regenerate deterministically and fail when stale — there
  are no hand-maintained mirror lists to update, but there *are* regen commands to
  run.
- **Both hook catalogs.** `hooks/hooks.json` (Codex, 8 events) and
  `hooks/claude-hooks.json` (Claude Code, 7 events) are two projections of one
  intent; changing one alone ships half a guard.
- **The rendered skill trees.** A SKILL change that forgets `.claude/skills/`,
  `.claude-plugin/skills/`, or `.codex-plugin/skills/` is the classic half-done
  migration — regenerate with `bee dev render-skill-trees`, never by hand.
- **The prose that the machine change makes redundant.** See §0.

## 4. Hand the plan to bee's own chain
The EDIT plan is not a green light to write. Route it:

- **Classify the lane** from the plan (risk flags + product-file count — see
  [index.md](index.md#lanes-how-much-of-the-chain-runs)). A change to auth, data
  loss, security, an external provider, or that weakens existing proof is
  **high-risk**, one flag is enough.
- Enter through **[hive](stages/hive.md)** → the chain. The EDIT plan becomes the
  raw material for `CONTEXT.md` (if gray areas remain) or the cells' `action` /
  `files` (if scope is clear).
- **No source edit happens before Gate 2's execution approval.** The planner's
  completeness check is what makes the cells' `files` lists correct, so nothing is
  missed at execution.

## 5. Resync after the change lands
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
- **Not a bypass.** The plan still flows through hive, the mode gate, and Gates 1–2.
  Localizing an edit is never approving it.
