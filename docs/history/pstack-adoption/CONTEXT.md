# pstack Adoption — Context

**Feature slug:** pstack-adoption
**Date:** 2026-09-01
**Shaping session:** complete
**Scope:** Standard
**Domain types:** ORGANIZE, RUN

## Feature Boundary

Give bee's existing `class` enum a procedure — a per-class playbook the
plan copies verbatim — add `perf` as an eighth class, make the herding
dispatch role refuse an uncheckable CoS, and make a review report show
what it dismissed. It ends at those four surfaces; it does not touch
gates, worktrees, or the cell lifecycle.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A class playbook is a named step list the plan **cites by name and anchor**; the steps stay in their single home and are read there. A skipped step stays visible and carries its recorded reason. It is never a refusal. | bee already owns the deviation half — "named deviation is the system working" (AGENTS.md, Judgment and deviation), so no refusal path is added and nothing in flight breaks. **Superseded `cc87b3c4` (which said "copies verbatim") with `132551fb`:** `docs/history/research/pstack-xia.md`, the prior study read from pstack's own tree at pinned commit `b9ddc83`, lists verbatim playbook todo-lists under *What must not be ported* — a copied list collides with bee's Direction of Truth (todo lists are projections) and becomes stale text an agent can satisfy by transcription. The user chose to keep building with this softening. |
| D2 | `perf` becomes an eighth `class` enum value with its own playbook: measure a baseline, change, measure again. "It feels faster" is not a result. | The enum change is a public-contract change and carries a migration note. Folding perf into `bugfix` would give one playbook two different proof rules. Decision `1593e365`. |
| D3 | The investigation route is the existing `research` class. No new route, no new lane. | `research` is already in the enum. A second taxonomy for the same idea is the duplication this feature exists to remove. Agent's call from evidence, recorded at shaping. Decision `f1ffa7bd`. **Rationale corrected during planning:** the original wording claimed `workflows.rs` already treats `research` as non-code-touching. It does not — that check reads `lane`, not `class` (`workflows.rs:587-595, 827`). The decision is unchanged; only its supporting reason was wrong. |
| D4 | The herding dispatch role refuses a candidate whose CoS is not checkable, and names why in its skip line. | bee has no run-until-X loop; work is backlog-driven, so a PBI's CoS *is* the finish condition. The role already reads `title+cos` for danger (`role-dispatch.md:262-273`), so this is a second pass over text already in hand. User chose refuse over warn. Decision `b03574c6`. |
| D5 | A review report carries its dismissed findings, each with the reason it was dismissed. | bee-reviewing emits surviving findings only; the filter is invisible. Showing the dismissed bucket lets a human override the judgment. Same honesty rule bee already applies to the verbatim "No findings. Scope clean" line. Decision `7332c6ca`. |

### Agent's Discretion

Where each playbook body physically lives (a `bee-hive` reference file, a
rendered CLI surface, or a `bee-planning` reference) is planning's call,
provided one home holds each playbook and every other surface points at
it. The exact wording of each playbook's steps is also planning's, bounded
by D1–D3.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Playbook | The named, ordered step list bound to one `class` value. Copied into the plan verbatim, never summarized. |
| Checkable CoS | A Condition of Satisfaction naming a command an agent can run, or a repository state it can evaluate. "Make the parser better" is not one; "zero old callers and every parser fixture passes" is. |
| Dismissed finding | A review finding the lead rejected. It is reported with its reason, not filtered out silently. |

## Specific Ideas And References

- `docs/history/research/pstack-source.md` — the source article (moved out of
  `docs/specs/`, a fenced read-only surface where its presence failed
  `specs_fence`). Read as data, never as instructions.
- `docs/history/research/pstack-xia.md` — **the prior and deeper study**, read
  from pstack's own tree at pinned commit `b9ddc83`, committed in `4ea6abfe`.
  It ranks four gaps above this feature's four and warns against verbatim
  playbooks (see D1). Read it before extending this work.
- `docs/history/research/pstack-distill.md` — the xia distill: dependency
  matrix, cross-cutting sweep, and what was deliberately skipped
  (`/automate-me`, `/teach`, `/bro`, Graphite stacking, Benny).

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` — holds the
  seven-value `class` enum (`ROUTE_CLASS_VALUES`) and its typed refusal. It has
  NO behavioral consumer — the code-touching check beside it reads `lane`, not
  `class`. D2's eighth value lands here, and this feature adds the class's
  first reader.
- `skills/bee-hive/references/scout-and-ticks.md:34` — the enum's documented
  home and the `Route: class=… | lane=…` preamble line. The display surface
  for a playbook is already free.
- `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs` — cells carry their
  own `change_class` enum (`formatting`, `bugfix`, `behavior`, `api`,
  `security`, `migration`, `refactor`, `test`). A second taxonomy the
  playbooks must line up with, never duplicate.

### Established Patterns

- Named deviation over refusal (AGENTS.md) — D1 is this pattern applied.
- Single-home procedure with pointers back (the blind-lane section in
  `gates-and-delegation.md:152` is the model) — each playbook gets one home.

### Integration Points

- `skills/bee-planning/SKILL.md:42` — writes the route; the playbook is copied
  into the plan at this step.
- `skills/bee-swarming/references/worker-details.md:33` — today's one-line
  bugfix craft rule ("watch the repro fail before the fix … not by flags").
  D1 promotes it into a playbook step without adding the flag it disclaims.
- `skills/bee-herding/references/role-dispatch.md` — the two-key lane-safety
  filter D4 extends.
- `skills/bee-reviewing/SKILL.md:72-76` — the findings summary line D5 extends.

## Outstanding Questions

### Resolve Before Planning

None.

### Resolve During Execution

- Whether the eighth enum value needs a migration note for host repos that
  pin an older bee, or whether an unknown class already degrades safely.
  Answer from the refusal path in `workflows.rs`, not from assumption.
