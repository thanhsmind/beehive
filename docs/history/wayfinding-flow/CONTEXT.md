# Wayfinding Flow — Context

**Feature slug:** wayfinding-flow
**Date:** 2026-08-17
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Add a pre-shaping discovery flow to bee: a `bee-wayfinding` skill that
turns a fog-state idea (no nameable outcome yet) into a map of decision
tickets under `docs/discovery/<effort>/`, resolved one per session until
the way is clear, then handed to bee-shaping — plus the hard activation
mechanisms (status/preamble scan, orient recommendation, route
park-to-map-stub, shaping entry check). It ends at the hand-off into the
existing shape→plan→swarm chain; it never builds product work itself.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Settled with the user 2026-08-17; the design discussion
lives in docs/history/wayfinding-flow/design-draft.md (draft IDs D-A..D-F
map to D1..D6 here).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Separate skill `bee-wayfinding`, not a new move inside bee-shaping. | Shaping stays one-session convergent; wayfinding is multi-session divergent. |
| D2 | Map lives in `docs/discovery/<effort>/` as plain markdown: `MAP.md` + one file per ticket (`tickets/NNN-<slug>.md`). No new bee state store for tickets. | — |
| D3 | Skill name is `bee-wayfinding` (user pick over bee-scouting). | Keeps the source pattern traceable. |
| D4 | Open maps are visible from v1: `bee status` scans `docs/discovery/*/MAP.md` and the session preamble prints "open map: <name>, N frontier tickets". | The one status/preamble code change; resume must be un-missable. |
| D5 | `bee orient` returns `skill=bee-wayfinding` deterministically when an open map has frontier tickets and no feature is active. | Hard resume routing, not model judgment. |
| D6 | Activation ships four mechanisms in v1: (1) status/orient resume per D4/D5; (2) explicit invocation; (3) `bee route` Qualify, on a parked-for-vagueness verdict, creates a map stub in `docs/discovery/` so fog never sinks silently; (4) a mandatory first check at bee-shaping's entry — "no nameable outcome → stop, switch to bee-wayfinding" — with Gate 1 as the human backstop. Semantic fog detection is NOT enforceable by hooks; (4) stays semi-hard by design. | — |
| D7 | Ticket model follows wayfinder: four types (grilling default/HITL, research/AFK, prototype/HITL via `.bee/spikes/`, task/either); destination named before any ticket; one with-user ticket per session (research fans out); agent never answers the user's side; `blocked-by:`/`claimed-by:` lines in ticket files, convention-only in v1 (no CLI guard). | — |
| D8 | Each resolved ticket's answer is recorded in the ticket file AND logged via `bee decisions log`; MAP.md's Decisions-so-far only gists and links D-IDs. Exit: when no tickets and no fog remain, each buildable feature hands off to bee-shaping Lock, which consumes the map's decisions into CONTEXT.md without re-asking. Wayfinding adds no new gate; it edits only docs. | Decision record stays the single source. |

### Agent's Discretion

- MAP.md exact section wording, ticket file frontmatter shape, and the
  status line's exact phrasing — within the sections named in
  design-draft.md ("The flow").
- Where the map-scan code lives inside packages/bee-rs and how the
  frontier count is computed (open + unblocked + unclaimed).
- The map stub's minimal content when route creates one (D6.3):
  destination unknown, the parked item's text and evidence linked.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| destination | What "arrived" looks like for an effort: a spec, a locked decision, or a change. Named first; fixes scope. |
| map | `docs/discovery/<effort>/MAP.md` — index only: Destination, Notes, Decisions so far, Not yet specified, Out of scope. Never restates a decision. |
| ticket | A question whose resolution is a decision, not a build task. One file under `tickets/`. |
| frontier | Open, unblocked, unclaimed tickets — what a session may take. |
| fog | In-scope questions too dim to phrase sharply yet; live in "Not yet specified". |
| graduate | Fog patch becomes ticket(s) once an answer sharpens it. |

## Specific Ideas And References

- Source pattern: /home/thanhsmind/projects/AI/mattpocock-skill/skills/engineering/wayfinder/SKILL.md — map/ticket/fog mechanics adapted; tracker replaced by local markdown per D2.
- docs/history/wayfinding-flow/design-draft.md — the full flow narrative (entry, session 1 charting, ticket loop, exit) and the reuse table.

## Existing Code Context

Scout dispatched at lock time; planning fills exact anchors from the
gather report before shaping cells.

### Reusable Assets

- `bee decisions log` — resolved-ticket recording (D8).
- bee-researching / gather-tier subagents — research tickets.
- `.bee/spikes/` — prototype tickets (already shaping's one exception).
- `bee backlog add` — real work discovered mid-map.

### Integration Points

- packages/bee-rs status/preamble section builder (D4).
- packages/bee-rs orient `next` recommendation logic (D5).
- packages/bee-rs route Qualify park verdict path (D6.3).
- skills/bee-shaping/SKILL.md entry (D6.4) and Lock (D8 hand-off).
- skill tree render (`bee dev render-skill-trees`) for the new skill.

## Outstanding Questions

### Deferred To Planning

- [ ] Exact insertion points in bee-rs for D4/D5/D6.3 — answered by the
  dispatched gather report.
- [ ] Whether the preamble map line rides the existing status JSON or a
  new section key — pick whichever the section builder already affords.

## Deferred Ideas

Revisit condition registered: [[trigger:wayfinding-v1-has-been-used-for-at-least__wayfindi]]

- `bee triggers` predicate-tier nudge ("map X still has tickets after N
  days") — not in v1, per the trigger above.
- `bee orient` full route-table entry for brand-new fog (beyond resume)
  — v1 relies on explicit invocation + shaping entry check (D6), per
  the trigger above.
- CLI guard for ticket claim/blocked lines — convention-only in v1
  (D7), per the trigger above.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
reads locked decisions, code context, canonical references, and
deferred-to-planning questions. Planning's Gate 2 shape stage and
reviewing use locked decisions for coverage and UAT.
