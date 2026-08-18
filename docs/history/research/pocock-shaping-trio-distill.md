---
artifact_contract: bee-research/v1
topic: pocock-shaping-trio-distill
depth: standard
date: 2026-08-18
---

## Bottom Line

- Recommendation (ladder rung): reuse + one adapt — bee-wayfinding
  already carries the whole shaping-trio structure (it descends from
  pocock's wayfinder); the one real stability gap is ticket claiming:
  bee's `claimed-by:`/`blocked-by:` lines are convention-only with no
  guard, while pocock rides the tracker's atomic assignee. Fix by
  reusing bee's existing reservation store as the ticket-claim guard —
  prose change only, no new code.
- Why lightest: the reservation store already exists, is
  heartbeat-tied, sweepable, and hook-enforced; one rule in the
  wayfinding reference turns the convention into a guard.
- Why next-best lost: building tracker integration (pocock's answer)
  or new `bee discovery ticket` CLI verbs is real code for a gap one
  reservation rule closes; filed as a backlog idea instead.
- Confidence: 85
- Suggested next step: bee-planning (scope already clear if approved)

## Source Manifest (xia)

| Field | Value |
|---|---|
| Repo or path | /home/thanhsmind/projects/AI/mattpocock-skill (github.com/mattpocock/skills) |
| Ref | HEAD |
| Resolved commit SHA | 84fdeff |
| Narrowed scope | skills/engineering/{wayfinder,prototype,research} — the "Shaping" flow trio |

## Dependency Matrix

| Pocock mechanism | Bee equivalent | Verdict | Label |
|---|---|---|---|
| Map issue + child tickets on the tracker, native blocking edges | docs/discovery/<effort>/ MAP.md + ticket files, `blocked-by:` frontmatter | EXISTS (file-based by design) | Local |
| "The assignee IS the claim" — atomic, tracker-enforced | `claimed-by:` line, "convention-only in v1 — no CLI guard" (wayfinding-reference) | **GAP — the one stability hole** | Local |
| Map is an index, never restates decisions | Decisions-so-far gists + D-ID links; decision log is single source | EXISTS — bee stronger (D-IDs, supersession) | Local |
| Ticket types (Research/Prototype/Grilling/Task), HITL vs AFK | Same four types, with-user vs agent | EXISTS (direct ancestry) | Local |
| One ticket per session, research exempt + parallel /research subagents | Same rule; gather-tier bee-researching fan-out | EXISTS | Local |
| "Refer by name, never bare id" | AGENTS.md: ids and counts trail, never lead | EXISTS | Local |
| Destination naming via /grilling + /domain-modeling | Session-1 interview + interview craft + pinned terms | EXISTS | Local |
| Map-clears handoff: merge onto /to-spec, never straight to /implement | Exit → bee-shaping Lock consumes D-IDs (D8) | EXISTS — bee stronger | Local |
| Resume: open map surfaced | `bee orient`/status list open maps + frontier counts | EXISTS — bee stronger | Local |
| prototype: throwaway, one-command, no persistence, state shown, variants for shape questions, kept as history | Spike rules (wayfinding-reference) + spike craft (planning.md) | EXISTS ~1:1 (already ported) | Local |
| prototype UI: in-app `?variant=` switcher, real data/auth, NODE_ENV gate | Nothing (bee is a CLI repo; spikes generic) | NEW but low value here | Local |
| prototype LOGIC: guided-walkthrough tabs + free-play buttons + labelled state panel | Spike rules have "state shown after every action" only | PARTIAL — recipe is sharper | Upstream |
| research: primary sources only, "follow every claim back to the source that owns it" | Docs label + official-domain bias in research-protocol step 4 | EXISTS in spirit; the phrasing is sharper | Upstream |
| research: background agent so you keep working | Delegation contract, gather fan-out | EXISTS | Local |

Cross-cutting sweep: pocock's trio leans on `docs/agents/issue-tracker.md`
(tracker abstraction) — the machinery bee deliberately does not carry;
bee's equivalents are the store, reservations, and orient. No other
hidden wiring. [Local]

## The one adapt — ticket claims ride the reservation store

Today a second session can claim the same wayfinding ticket: the
`claimed-by:` line is read by convention, nothing refuses the write.
Pocock never has this problem because the tracker's assignee field is
atomic. Bee's cheapest equivalent guard already exists:

- Claiming a ticket = `bee reservations reserve --agent <name> --cell
  <effort>-wayfinding --path docs/discovery/<effort>/tickets/NNN-<slug>.md`
  plus the existing `claimed-by:` line for display.
- The reservation is heartbeat-tied and sweepable — a dead session's
  claim expires with it (today a stale `claimed-by:` line lies forever).
- The write-guard hooks already refuse a reserved path — the collision
  becomes a typed deny instead of a silent double-claim.
- Frontier rule unchanged: open, unblocked, unreserved tickets.

Change surface: `skills/bee-wayfinding/SKILL.md` (claim step) +
`references/wayfinding-reference.md` (resolution protocol + frontier
definition). Prose only; regen chain applies.

Optional riders (same cell): the two sharper phrasings — "follow every
claim back to the source that owns it" into research-protocol step 4,
and the walkthrough-tabs/free-play/state-panel recipe into Spike rules.

## Weaknesses (dở — do not import)

- Tracker coupling (GitHub/GitLab/local trio + setup skill) — heavy
  machinery bee replaces with files + store. [Local]
- Map restates nothing but decisions live only in issue bodies — no
  D-IDs, no supersession, no single decision log. [Upstream]
- No fog ledger equivalent of `(agent-suspected)` markers — bee's
  interview-round machinery is already ahead. [Local]

## Risks, Unknowns, Follow-Ups

- Reservation on a docs path: reservations are path-based and the
  guard checks paths, so a docs/ path should hold — verify at planning
  with one live reserve/deny probe. [Inference → proof obligation]
- Backlog idea (not this effort): `bee discovery` ticket verbs
  (frontier/claim/close) as the durable CLI home, superseding the
  reservation spelling later.

## Source Pack

- Local: skills/bee-wayfinding/SKILL.md + references/wayfinding-reference.md
  (read in full this session), AGENTS.md, planning.md digest.
- Upstream: mattpocock/skills @ 84fdeff — wayfinder/, prototype/
  (SKILL+UI+LOGIC), research/ via gather digests.
- Docs: none (not applicable).
