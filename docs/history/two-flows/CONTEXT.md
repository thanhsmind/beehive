# Two Flows — Context

**Feature slug:** two-flows
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Standard
**Domain types:** READ | ORGANIZE

## Feature Boundary

bee names and surfaces two flows — **Main flow** (idea to ship) and
**Discovery flow** (an open question to a locked decision) — and makes
the Discovery flow whole: a named entry, a claim that is guarded rather
than conventional, three ticket-resolution kinds, and one exit into the
Main flow. Prose across the router, the contracts, and the two flows'
skills; plus the reservation-backed claim rule. No CLI change, no new
skill, no gate change.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The two flows are named **Main flow** (shaping → planning → swarming → capturing, with reviewing and uat as the merge doors) and **Discovery flow** (wayfinding as the spine; research, spike, and grilling are how its tickets resolve). "Shaping" is never used as a flow name — `bee-shaping` is a Main-flow skill | Avoids collision with the existing skill name; `docs/discovery/` and `bee discovery` already carry the word |
| D2 | The names surface in prose only: bee-hive's Route section leads with a two-door table, the AGENTS bee block names both flows in one line, and README's role table gains `bee-wayfinding` plus a flow column naming each skill's flow. Only `bee-wayfinding` carries an in-skill flow-position line — the Main-flow skills already read as one chain, so four more one-line notes would be churn. A `bee orient`/`status` flow line is NOT built here — backlog | Prose ships now; CLI waits for the names to prove stable |
| D3 | The agent routes; the user never has to name a flow. A nameable outcome → Main flow. Fog, or too big to name in one sitting → Discovery flow | Communication contract: bee terms stay out of the user's mouth |
| D4 | An explicit user word — "wayfinding", "brainstorm", "discuss", "discovery", in any language — routes straight into the Discovery flow and skips the classification in D3 | A deliberate manual override the user asked for |
| D5 | A Discovery-flow ticket claim is guarded, not conventional: claiming reserves the ticket file (`bee reservations reserve --agent <name> --cell <effort>-wayfinding --path docs/discovery/<effort>/tickets/NNN-<slug>.md`) alongside the display-only `claimed-by:` line. The frontier is open, unblocked, **unreserved** tickets. A dead session's claim expires with its heartbeat instead of lying forever | The one real stability gap (pocock rides the tracker's atomic assignee); bee's reservation store already gives atomicity, heartbeat expiry, and a hook-level deny |
| D6 | Cell tf-2 proves D5 before writing it as law: run one live reserve → observe → release on a real `docs/discovery/**` ticket path and record the output. If reservations do not hold on that path, the cell records the refusal and writes the sweep-based fallback instead of the guard claim | The reservation-on-docs-path assumption is `Inference`, so it carries a proof obligation |
| D7 | Spike rules gain the LOGIC-page recipe — title plus the question, a labelled state panel re-rendered after every action (never raw JSON), one free-play button per action, and guided-walkthrough steps that each reset to a known initial state — and the cap "more than five variants stops being different and starts being noise" | Distilled from pocock prototype/LOGIC.md + UI.md |
| D8 | `research-protocol.md` step 4 states the primary-source rule: a secondary write-up is not evidence — follow every claim back to the source that owns it (official docs, the source code, the spec, the first-party API) | Distilled from pocock research/SKILL.md |

### Agent's Discretion

Exact wording and placement inside each file, matching each file's
voice; the shape of the two-door table.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Main flow | The idea-to-ship spine: shaping, planning, swarming, capturing, with reviewing and uat as merge doors |
| Discovery flow | Question-to-decision: wayfinding charts the map; research, spike, and grilling resolve its tickets; exit is bee-shaping's Lock |
| flow position | A one-line note in a skill saying which flow it belongs to and what precedes/follows it |

## Canonical References

- docs/history/research/pocock-shaping-trio-distill.md — the xia brief
  behind D5, D7, D8 (source: mattpocock/skills @ 84fdeff)
- skills/bee-hive/SKILL.md "Route" — the surface D2 changes first

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Two cells:
tf-1 names and surfaces the flows, tf-2 makes the Discovery flow whole.
