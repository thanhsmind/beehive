---
type: bee.work-item
title: "knowledge-one-home — one home per rule, and the two doors that hold it"
description: "Give every rule exactly one home with an outbound applied_at list, an ownership map on every area, a cap that refuses a skipped skill or listed file, and a merged gate that refuses an unverdicted plan-time conflict candidate."
tags: [knowledge, rule-homes, ownership, cap-door, gate-door, standard]
timestamp: 2026-08-22
bee:
  id: knowledge-one-home
  lifecycle: active
  areas: [okf-profile, workflow-state, decision-memory, doctrine-layer]
  required_context: [areas/okf-profile/concept-model-and-authoring.md, areas/okf-profile/conformance-check.md, areas/workflow-state/cells-completion-judge-and-archive.md, areas/workflow-state/gates.md, areas/decision-memory/overview.md]
  decisions: [D1, D2, D3, D4, D5]
  sources: [docs/history/knowledge-one-home/CONTEXT.md, docs/history/knowledge-one-home/plan.md, docs/discovery/knowledge-one-home/MAP.md, docs/discovery/knowledge-one-home/tickets/004-rule-home-inventory.md]
  lane: standard
---

# knowledge-one-home — One Home Per Rule, and the Two Doors That Hold It

## Outcome

Twelve rules lived in several places at once, seven of them had already
drifted apart, and no copy pointed back at any original. Nothing in the
system could tell an agent which files a settled rule obliged it to
update, so the update list was guessed — and the guess was usually the
file the agent happened to be looking at.

The work gives every rule a single home with an outbound list of the
files that restate or enforce it, gives every knowledge area a map of
the code, skills, and tests it governs, and then puts two refusing doors
on that data: a cap door that refuses work which touched owned code and
left the owning skill or a listed file untouched, and a merged-gate door
that refuses a plan whose derived conflict candidates carry no verdict.
The twelve inventoried rules are migrated through those doors as their
first real use, so the doors are proven on live cases rather than on
fixtures.

## Scope

In: the frontmatter schema (`owns.code`, `owns.skills`, `owns.tests`,
`applied_at`), six new `knowledge check` codes, ownership maps for all
fifteen areas, `affects_skills`/`affects_specs` on every cell, the
`update_obligations` line on `decisions log`, the cap-time sync door with
its `--sync-ack` escape, `bee state plan-conflicts derive|verdict`, the
merged-gate conflict precondition, the `--skill-answer` requirement on a
skill-owning area's capture stub, and the migration of the twelve
inventoried rules.

Out: `knowledge context` ranking (untouched by decision), a generator for
the hand-edited help payload, and any rule beyond the twelve inventoried
ones — later drift is caught by the check, not by this work.

## Acceptance

- A concept carrying all four new keys parses, re-emits byte-identically,
  and grades clean.
- `bee knowledge check` reports zero profile errors over the live bundle,
  including the six new codes.
- A cap that touches owned code and skips the owned skill is refused by
  name; the same cap with a recorded reason is capped and the reason is
  mined into the deviation list.
- `bee gate --merge --approved true` on a lane is refused while any
  derived candidate lacks a verdict, and refused again after a
  `plan-rev bump`.
- Every copy of a migrated rule is one line plus a pointer to the rule id.

## Decisions

The locked set lives in `docs/history/knowledge-one-home/CONTEXT.md`.

- **knowledge-one-home D1** — fix at write time, never at read time. The
  obligation fires when a rule settles (a logged decision, a capture) and
  when a plan is gated; `knowledge context` ranking is untouched, because
  retrieval scatter is a consequence of multi-home rules and no smarter
  read can repair scattered data. Shipped by koh-7 as the
  `update_obligations` list on `decisions log`.
- **knowledge-one-home D2** — conflict detection runs at plan time,
  before the merged gate, never first at cap. Shipped by koh-9 as the
  merged gate's precondition over a review that koh-8 derives at plan
  time.
- **knowledge-one-home D3** — code-without-skill detection is a plan
  prediction plus a cap check. Each area declares the code and skill
  paths it owns; each cell records its predicted affected skills and
  specs; at cap bee diffs the real touched files against that map and
  **refuses**. Warning-only was rejected outright: soft reminders are
  exactly what fails today, so the older "warn, never refuse" cap-door
  precedent does not reach this check. Shipped by koh-3, koh-5, koh-6.
- **knowledge-one-home D4** — one home per rule. Discipline rules live in
  the operating block, mechanism rules in the area concept; every other
  site carries at most one line plus a pointer. The home's frontmatter
  carries `applied_at`, bee computes the update list, and a cap that left
  a listed file untouched is refused without a recorded reason. Rules
  carry ids; `knowledge check` flags an id-less rule block, one id in two
  bodies, and a dangling target; a capture stub for a skill-owning area
  must answer the skill question. Shipped by koh-1, koh-2, koh-4, koh-6,
  koh-10, koh-11, koh-12.
- **knowledge-one-home D5** — plan-time conflict check. bee derives
  candidates from the plan, each candidate takes one verdict of
  `compatible` / `conflicts` / `retires-prior`, "0 conflicts" is true only
  when bee returned zero candidates, and the merged gate refuses while
  any candidate is unverdicted. A `plan-rev bump` resets the verdicts.
  Shipped by koh-8 and koh-9.

### Decision provenance and the reconciliation of the citing map

The discovery map and its tickets were written against the earlier
decision ids, and two later decisions touch them. Both citing documents
sit in the exempt `docs/discovery/` tree — a record, never a live rule —
so the reconciliation is recorded here rather than edited into them:

| Citing document | Cites | Touched by | Reconciliation |
|---|---|---|---|
| `docs/discovery/knowledge-one-home/tickets/003-code-without-skill-detector.md:24` | `3ea7500a` (D3) | `27e55095` (D4) | No contradiction. D4 extends D3's ownership map with the outbound `applied_at` list; the cap door D3 asks for and the door D4 asks for are one door with three checks (koh-6). |
| `docs/discovery/knowledge-one-home/MAP.md:39` | `3ea7500a` (D3) | `27e55095` (D4) | Same reading — the map's ownership-map line survives verbatim under D4. |
| `docs/discovery/knowledge-one-home/tickets/001-outbound-obligations.md:19` | `27e55095` (D4) | `efd6cbaa` (D5) | No contradiction. D5 adds a plan-time conflict door **on top of** D4's rule homes; it consumes `applied_at` to find rule candidates and changes nothing D4 settled. |
| `docs/discovery/knowledge-one-home/MAP.md:40` | `27e55095` (D4) | `efd6cbaa` (D5) | Same reading — the map's outbound-obligation line stands unamended. |

The map's own D1 and D2 were folded into D4 and D5 during shaping, which
is why the locked table carries five ids where the map carried five
tickets.

## Chosen Approach

Four phases, each one door, each demoable on its own:

1. **Schema and map** (koh-1 … koh-4) — make the data exist and gradeable
   before anything computes over it.
2. **Cap door** (koh-5 … koh-7) — put the refusing door on that data, and
   push the obligation at the moment a decision is logged.
3. **Gate door** (koh-8, koh-9) — derive conflict candidates at plan time
   and refuse the merged gate without verdicts.
4. **Migration** (koh-10 … koh-12) — run the twelve inventoried rules
   through the finished doors, which is both the migration and the proof
   the doors bite.

Rejected on the way: a warn-only cap door (rejected by D3 in its own
words), a separate ownership file (two homes for ownership), and a prose
heuristic for recognising a rule block (it re-creates the guessing the
work exists to remove — an explicit marker is written by the author or
there is no rule).
