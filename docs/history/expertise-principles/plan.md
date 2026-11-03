---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset>
---

# Plan: Expertise Principles

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the change adds a new always-visible surface (the preamble's principle block) and a new instruction law, both of which every future session reads — but it removes nothing, weakens no proof, and touches no auth, data, or external system.

Class playbook: `feature` — no `feature` playbook exists in
`skills/bee-planning/references/planning-reference.md` ("Class playbooks");
only `perf`, `bugfix`, `refactor`, and `research` are written (claim 2).
**Named deviation:** the cite-a-playbook step cannot be satisfied for this
class. The gap is recorded as a backlog item, not fixed here — writing four
missing playbooks is its own feature.

## Requirements (from CONTEXT.md)

- **D1**: The 16 `expertise/` guides stay as the body. A thin principle skill layer sits on top — one skill per principle, short, pointing into its guide section. No guide is deleted or shattered.
- **D2**: `bee orient` selects the principles from the recorded route; the candidates appear in the session preamble. The principle index is NOT always-loaded.
- **D3**: On a multi-step task the reply names each applied principle and the decision it changed, or states plainly it changed nothing. Extends `f051746e`.
- **D4**: Craft half only — `thinking`, `planning`, `architecture`, `decisions`, `tests`, `review`, `documentation`, `knowledge`, `debugging`, `merges`. The six domain guides are untouched.

## Load-bearing claims

Labels: `read` — the file was opened at the named line this session. `ran` — the named command was executed this session and its output is quoted. `guessed` — not permitted past the gate. A row matches only if the anchor resolves to the quoted bytes.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | **FALSIFIED at ep-8 — see the note under this table.** Both skill pipelines discover skills by scanning the canonical `skills/` directory, so new `skills/principle-*/` dirs reach the plugin trees and host repos with NO Rust change. | read (wrongly) | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:17-19`, `:436`, `:459` | `//   .claude-plugin/skills/ = render(canonical skills/, "claude")` / `//   .codex-plugin/skills/  = render(canonical skills/, "codex")`; `let Ok(entries) = std::fs::read_dir(src) else {` |
| 2 | There are EIGHT route classes but only FOUR class playbooks. `feature`, `docs`, `release`, and `spike` have none. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:287-288`; `skills/bee-planning/references/planning-reference.md` headings `185,195,207,216` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 8] =` / `["feature", "bugfix", "docs", "refactor", "research", "release", "spike", "perf"];` — and the only `### ` playbook headings are `perf`, `bugfix`, `refactor`, `research` |
| 3 | The doctrine-layer concept already carries a parsed row grammar for named rules: an id line, a `spoken:` line, then an indented `applied_at:` list. It is parsed by `parse_agents_rule_homes` and fenced by `rule_index_parity.rs`. | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:181-197`; `packages/bee-rs/crates/bee/tests/rule_index_parity.rs:1-45` | `- `agents-never-build-on-red` (AGENTS.md § Prove, then say so):` / `  spoken: never build on a red base — the red is the work now: fix it first, then carry on` / `  - applied_at:` |
| 4 | The session preamble renders the Route line — the exact place a principle block belongs beside — in `budget.rs`. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:545` | `"- Route: class={} | lane={} | flags={flag_count} [{flag_list}] | files={}",` |
| 5 | `bee orient` renders its own flat line block, ending `skill:` / `next:`. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:623-660` | `lines.push(format!("skill: {}", tpl(vget(&next, "skill"))));` |
| 6 | Control-plane verbs (`state start-feature`, `state route`) REFUSE inside a granted feature worktree; they must run from the main checkout. | ran | `.bee/bin/bee state route --set …` run from `/home/thanhsmind/Projects/goglbe/beehive--wt--expertise-principles` | `bee state route: refused inside a granted feature worktree — this command reads the shared control plane (sessions, claims, workers, workflows, handoff), which lives in the main checkout. FIX: run it from /home/thanhsmind/Projects/goglbe/beehive.` |
| 7 | `AGENTS.md` and `packages/bee/AGENTS.block.md` each carry exactly TEN `<!-- rule: … -->` markers, and the parity fence pins the two sets equal to the index rows. | ran | `rg -c '<!-- rule: ' AGENTS.md packages/bee/AGENTS.block.md` | `packages/bee/AGENTS.block.md:10` / `AGENTS.md:10` |
| 8 | The ten craft guides hold roughly 160 headings between them — far more than a principle set can carry, which is why D1 keeps the guides as the body. | ran | `for f in expertise/*.md; do … rg -c '^#{2,3} ' $f; done` | `expertise/tests.md 4611 words 22 headings`; `expertise/thinking.md 2529 words 21 headings`; `expertise/architecture.md 2483 words 20 headings` |

**Claim 1 was false, and this is what it cost.** The label said `read`, but
what was read was the module docstring in `devtools/skill_trees.rs`, not the
discovery predicate. Both pipes filter directory entries on a literal `bee-`
name prefix — `skill_trees.rs:448` and `onboard/render.rs:379` — so all 14
principle skills reached ZERO host repos while a 3628-test suite stayed green.
The `ep-8` worker refused to cap on it and filed a blocker dissent; the verdict
was `accept`. The layer was renamed into the `bee-principle-` namespace and the
parity fence grew a check that reads the SHIPPED plugin trees, not just the
source. Rule learned: a claim about WHAT a pipeline discovers is settled by the
filter expression. Seeing `read_dir` proves a directory is walked, never which
entries survive.

## Discovery

Three findings changed the shape.

1. **Distribution is free** (claim 1). Both the plugin render and the onboarding skill sync read the `skills/` directory. Adding principle skills needs no code in either pipeline. This removed a whole slice from the first draft.
2. **Class playbooks cannot be the mapping home** (claim 2). Half the route classes have no playbook, so a mapping hung off playbook sections would leave `feature`, `docs`, `release` and `spike` with no principles — including the class THIS feature routed as.
3. **The registry grammar already exists** (claim 3). The doctrine-layer concept's `## AGENTS.md rule homes` section has a parsed, fenced row format. A second section in the same file, with the same grammar plus one `classes:` line, reuses the parser shape and the fence pattern instead of inventing a registry.

## Approach

**Recommended.** One new `## Principle homes` section in the existing
doctrine-layer concept is the single home of the principle index. Each row
names the principle skill, its `spoken:` line, the route classes that trigger
it, and the `expertise/` guide anchor holding its depth. `bee orient` and the
session preamble read that section through ONE shared renderer and print the
principles whose `classes:` contains the recorded route class (D2). Each
principle ships as `skills/principle-<slug>/SKILL.md` — one page, the rule,
and the guide pointer (D1). A new text-reading fence pins skill dirs, index
rows, and legal class values against each other. D3's obligation lands in
`packages/bee/AGENTS.block.md` as an eleventh named rule beside the
`f051746e` invocation law it extends.

**Rejected.**

- *Mapping keyed on class playbooks* — half the classes have no playbook (claim 2).
- *A new registry file plus a new CLI verb* — rung-4 work for a rung-1 gap; the same reasoning `pstack-gaps` D4 already recorded.
- *Skip the Rust change and let `bee-hive` tell the agent to read the index* — cheaper, but D2 locks selection into `bee orient` and the preamble. A skill instruction is not a router.
- *Render the principle block separately in orient and in the preamble* — two copies of one decision. One shared renderer, two callers.

**SMALLER PATH check.** Asked: is there a cheaper shape that still honors
D1–D4? The one candidate was dropping the Rust work (last rejected item
above). It fails D2, which is locked. Evidence: CONTEXT.md D2 names `bee
orient` and the session preamble by name. The shape stands, minus the
distribution slice that claim 1 deleted.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| Principle index section + grammar | LOW | The new fence reads it; `bee knowledge check` still green |
| `bee orient` / preamble renderer | MEDIUM | `session_preamble` tests assert today's block order; new tests for present, absent, and unknown-class cases |
| `AGENTS.block.md` eleventh rule | MEDIUM | `rule_index_parity.rs` must go from 10 markers to 11 on both surfaces after `bee dev regen`; a half-done regen turns it red |
| ~15 principle skill pages | LOW | The fence pins dir↔row both directions; content quality is review's job, not a test's |

## Shape

**Slice 1 — walking skeleton.** One real principle, end to end, no stubs: a
task routes, the preamble names the principle, the agent can open its skill,
and the reply is obliged to cite it.

- **ep-1** — Add the `## Principle homes` section to the doctrine-layer
  concept with its row grammar, seeded with ONE row. Add
  `skills/principle-red-before-green/SKILL.md` — the rule in one page, with
  its depth pointer into `expertise/tests.md`.
- **ep-2** — Rust: one shared renderer that reads the section and filters by
  the recorded route class; called by `bee orient` and by the session
  preamble's budget block. Tests for present, absent, and unknown-class.
- **ep-3** — `packages/bee-rs/crates/bee/tests/principle_index_parity.rs`:
  every `skills/principle-*/` dir has a row, every row names a real dir,
  every `classes:` value is in `ROUTE_CLASS_VALUES`, every row has a
  non-empty `spoken:` line and a guide anchor that resolves. Pure
  filesystem, std only, nothing imported from the crate — the
  `rule_index_parity.rs` model.
- **ep-4** — D3's law into `packages/bee/AGENTS.block.md` as rule
  `agents-name-applied-principles`, its index row, then `bee dev regen` so
  `AGENTS.md` carries the eleventh marker and `rule_index_parity.rs` stays
  green.

<!-- bee:not-a-deferral: slice-planning machinery, and now archaeology — slices 2 and 3 were both built, capped and merged; the domain half's own deferral carries trigger the-craft-principle-half-has-lived-throu__f1b48dac -->
**Later slices (headlines only — no cells yet).**

- *Slice 2* — The remaining craft principles, derived from the ten guides
  named in D4. Expected 12–18 rows and skill pages total; a heading earns a
  principle only if speaking its name can steer a task.
- *Slice 3* — Point each existing class playbook and each craft guide at its
  principles, so a reader arriving from either side finds the same set.
<!-- /bee:not-a-deferral -->

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges
existing coverage first and authors only the gap.

- **Happy path** — a route with `class=feature` recorded: the preamble and
  `bee orient` both print the principle block, listing exactly the rows whose
  `classes:` contains `feature`.
- **Edge cases** — no route recorded (block absent, no crash); a route whose
  class matches zero rows (block absent, not an empty header); a row whose
  `classes:` names several values; the section missing entirely from the
  concept file (renderer degrades quietly, fence goes red).
- **Error paths** — a `classes:` value outside `ROUTE_CLASS_VALUES` (fence
  red, named); a `principle-*` skill dir with no row and a row with no dir
  (fence red in both directions); a row whose `spoken:` line is empty.

## Open Questions

- Do the principle skills need `disable-model-invocation: true`? pstack sets
  it so the model never auto-fires a leaf. Decide from how ep-2's live output
  actually reads; it is a one-line frontmatter change either way, and does
  not block ep-1.
- Adding ~15 skills grows every host repo's always-loaded skill-description
  index. Measure the current total against the projected one during slice 2,
  before the description wording is fixed. Slice 1 adds one skill and cannot
  move the number meaningfully.

## Out of scope

- The six domain guides (`data`, `apis`, `security`, `operations`,
  `performance`, `frontend`) — CONTEXT.md D4, backlogged.
- The four missing class playbooks (`feature`, `docs`, `release`, `spike`) —
  a real gap found by claim 2, backlogged, not fixed here.
- Deleting, splitting, or rewriting any `expertise/` guide — CONTEXT.md D1.
- Any new CLI verb, route class, or lane.
