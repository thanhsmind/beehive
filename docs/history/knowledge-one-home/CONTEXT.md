# Knowledge one-home — Context

**Feature slug:** knowledge-one-home
**Date:** 2026-08-22
**Shaping session:** complete (locked from docs/discovery/knowledge-one-home/MAP.md)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

bee gives every rule exactly one home with an outbound `applied_at`
list, computes the update list itself when a rule changes, refuses a cap
that skipped a listed file or an area's owned skill without a recorded
reason, and refuses the merged gate while a plan-time conflict candidate
has no verdict. It ends at bee's own tooling and bundle schema — the
migration of the 12 inventoried duplicated rules is the first use, not
a separate feature.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Fix at write time, not read time: the `knowledge context` ranking is untouched. The obligation fires when a rule settles (decision log, capture) and when a plan is gated. (map D1, folded into D4) | Retrieval scatter is a consequence of multi-home rules; smarter reads cannot fix scattered data. |
| D2 | Conflict detection runs at plan time, before the merged gate — never first at cap. (map D2, folded into D5) | — |
| D3 | Code-without-skill detection = plan prediction + cap check. Each area spec declares in frontmatter the code paths and skill paths it owns. The plan records predicted affected skills/specs per cell (or `none`). At cap bee diffs the real touched files against the ownership map; code in area X touched with X's skill untouched, or prediction ≠ diff, **refuses the cap** until a reason is recorded. Warning-only is rejected. (decision 3ea7500a) | The existing cap door precedent "warn, never refuse" (derived-check-hardening E1) does NOT apply to this check — the user rejected warn-only because soft reminders are what fails today. |
| D4 | One home per rule. Discipline rules (gates, proof, commit) live in AGENTS.md; mechanism rules (what cap does, what a hook blocks) live in the area spec. Skills and help text carry at most one line + a pointer. The home's frontmatter carries `applied_at`: every restating or enforcing file — skill text, help strings in Rust source, generated payloads, tests. bee computes the update list from `applied_at` + the ownership map; the agent may add, never remove; cap refuses an untouched listed file without a recorded reason. Rules carry an id; a copy longer than one line cites it; `knowledge check` flags an id-less rule-shaped block, one id in two bodies, and dangling `applied_at` targets. A capture stub for an area that owns a skill must answer "skill changed" or "not, because …". (decision 27e55095) | Inventory: 12 rules in multiple homes, 7 drifting, zero outbound arrows; copies hide in help strings and generated JSON that text search never visits. |
| D5 | Plan-time conflict check. bee derives candidates from the plan (cell titles, touched paths, area tags) via `decisions active` (tag, area, terms) and `applied_at` (touched path → rules homed there). Each candidate gets a verdict recorded on the plan: `compatible` / `conflicts` / `retires-prior <id>`. "0 conflicts" is valid only when bee returned zero candidates. `gate --merge` refuses while any candidate lacks a verdict — same precondition shape as the high-risk `advisor_ref` guard; `plan-rev bump` resets the verdicts. (decision efd6cbaa) | — |

### Agent's Discretion

The map's remaining fog lines were delegated to the agent as defaults
(user, 2026-08-22, "go"). Planning picks concrete values within these
constraints and records them as plan decisions, not new D-IDs:

- Field names for the ownership map and the outbound list (suggested:
  `owns: {code: [...], skills: [...], tests: [...]}` and `applied_at:
  [...]`), globs allowed, repo-relative paths, validated by
  `knowledge check` like `required_context` targets.
- Rule id scheme: stable slug per rule, registry is the home file
  itself (the rule block carries the id; no separate index file).
- Rule-shaped block recognition: an explicit marker the author writes
  (a fenced or tagged block), never a prose heuristic — a heuristic
  would re-create the "agent guesses" failure.
- `docs/history/` and `docs/discovery/` are exempt from the copy check
  (history is a record, not a live rule). `docs/specs/` is the read-only
  compatibility surface and is also exempt.
- Migration of the 12 inventoried rules (ticket 004) is the last slice
  of this feature: assign ids, write `applied_at`, reduce copies to
  one-line pointers, fix the two contradictions (delegation threshold;
  write-guard allowlist) by citing the knowledge concept / the crate as
  truth. The stale `bee cells cap --help` text ("bee close runs
  commands.test") is in that set.
- Who writes the first ownership map: the agent, one cell per area
  group, from each area's existing "Pointers (implementation)" section.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| home | The single file where a rule's full text lives: AGENTS.md for discipline rules, an area spec for mechanism rules. |
| applied_at | Frontmatter list on the home naming every file that restates or enforces the rule. |
| ownership map | Frontmatter on an area spec naming the code, skill, and test paths the area governs. |
| copy | Any restatement of a rule outside its home; allowed only as one line plus a pointer to the rule id. |
| conflict candidate | An active decision or homed rule bee matched to a plan; owes a verdict before the gate. |

## Specific Ideas And References

- docs/discovery/knowledge-one-home/MAP.md — the map; tickets 001–004
  hold the answers and the inventory.
- docs/discovery/knowledge-one-home/tickets/004-rule-home-inventory.md —
  the 12 duplicated rules with file:line sites; the migration slice's
  input.

## Existing Code Context

From the quick scout only.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs` — the
  derived regen obligation at cap (lib/cells D1/D2): the shape for a
  derived "applied_at / ownership" obligation.
- `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` — the
  high-risk `advisor_ref` precondition on `gate --merge`; D5 reuses this
  precondition shape.
- `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` — frontmatter
  walk, dangling `required_context` detection; D4's new checks extend it.
- `packages/bee-rs/crates/bee/src/verbs/decisions/` — `search` scoring and
  `log`'s `conflict_candidates`; D5's candidate derivation reuses them.

### Established Patterns

- Cap-door refusals name the remedy and are audited (judge override,
  ownership force) — the new refusals follow the same output shape.
- Frontmatter keys on area concepts: `id, lifecycle, areas,
  required_context, decisions, sources, authoritative_for, tags`
  (verified over 109 concepts; none carries paths today).

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` — cap
  handler; D3/D4 refusal lands here.
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` +
  `registry.rs` — help text lives here; it is an `applied_at` target
  class, and `bee dev regen` must keep it in sync.
- `skills/bee-capturing/SKILL.md` — the stub's "skill changed?" answer
  (D4 item 5).
- `skills/bee-planning/` templates — the predicted-affected line per
  cell (D3) and the conflict-verdict table (D5).

## Canonical References

- docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md
  — the declared, unenforced duplication boundary D4 now enforces.
- docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md
  — the cap door's current teeth; D3 adds a door.

## Outstanding Questions

### Resolve Before Planning

None.

### Deferred To Planning

- [ ] Does the cap-time ownership check run on `git diff` of the cell's
  commit or on `--files`? — read handlers_close.rs; prefer the commit
  diff (the `--files` list is self-reported).
- [ ] Where does the plan store conflict verdicts so `plan-rev bump` can
  reset them — plan.md section or a state field? — read set_gate.rs and
  plan-rev handling.
- [ ] Lane order: schema + check first, then cap door, then gate
  precondition, then migration — confirm nothing needs the migration
  earlier.

## Deferred Ideas

- Smarter `knowledge context` ranking — explicitly out of scope (map).
- Release checklist gaps — already fixed as its own example (map).

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
reads locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and
reviewing use locked decisions for coverage and UAT.
