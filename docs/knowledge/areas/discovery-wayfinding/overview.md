---
type: bee.area
title: Discovery Wayfinding — charting fog-state ideas before shaping
description: "How an idea with no nameable outcome becomes a discovery map of decision tickets, how a ticket's frontier state is derived, how open maps surface in status/preamble/orient so resume is un-missable, and how a finished map hands its decisions to shaping without re-asking."
timestamp: 2026-08-17
bee:
  id: discovery-wayfinding-overview
  lifecycle: active
  areas: [discovery-wayfinding]
  decisions: ["wayfinding-flow D1 (separate bee-wayfinding skill)", "wayfinding-flow D2 (map as markdown under docs/discovery/<effort>/)", "wayfinding-flow D3 (name bee-wayfinding)", "wayfinding-flow D4 (status + preamble show open maps from v1)", "wayfinding-flow D5 (orient recommends bee-wayfinding deterministically when idle with frontier)", "wayfinding-flow D6 (four activation mechanisms; park-for-vagueness creates a stub; shaping entry check stays semi-hard)", "wayfinding-flow D7 (four ticket types, destination-first, one HITL ticket per session, convention-only claim/block lines)", "wayfinding-flow D8 (resolved tickets log decisions; map gists; exit feeds bee-shaping Lock)"]
  sources: ["cells wayf-1..wayf-6 (capped, .bee/cells/, 2026-08-17)", docs/history/wayfinding-flow/CONTEXT.md, docs/history/wayfinding-flow/plan.md, "judge verdicts 6/6 PASS (trace.semantic_judge, model independence confirmed)", "wayfinding-craft cell wayfc-1 (interview craft, fact-lookup dispatch, spike rules; trace .bee/cells/archive/wayfinding-craft/wayfc-1.json, 2026-08-17)"]
  authoritative_for: "discovery-wayfinding: pre-shaping discovery maps, their tickets, and their activation surfaces"
  owns.code: [packages/bee-rs/crates/bee/src/verbs/discovery.rs]
  owns.skills: ["skills/bee-wayfinding/*"]
  owns.tests: [packages/bee-rs/crates/bee/tests/discovery_verbs.rs]
---

# Discovery Wayfinding (charting fog-state ideas before shaping)

Before this area existed, a request whose outcome the user could not yet
name had no home: shaping assumes a nameable ask, so fog either got a
premature interview or sank silently. This area gives fog a durable,
visible place — a discovery map — and a loop that resolves one decision
at a time until shaping can take over.

## The store

An effort lives at `docs/discovery/<effort>/` as plain markdown — no
runtime state store. `MAP.md` is an index with five sections:
Destination, Notes, Decisions so far, Not yet specified (the fog), Out
of scope. The map only gists and links; a decision's single source is
the decision log. Tickets live one file each under `tickets/` with
frontmatter keys `type`, `status`, `claimed-by`, `blocked-by` and a
body of `## Question` then `## Answer` on close.

## Ticket lifecycle and the frontier

A ticket is one of four types: grilling (conversation with the human,
the default), research (agent-alone), prototype (a cheap mock the human
reacts to), task (manual work that unblocks a decision). A ticket is
**frontier** when its status is open, no one claims it, and every
ticket named in `blocked-by` is closed — an unknown `blocked-by` id
fails closed (not frontier). Claim and block lines are convention-only:
no CLI guard enforces them. One with-user ticket resolves per session;
research tickets fan out. The destination is named before any ticket
exists; it fixes scope.

## The stub — fog never sinks

`bee discovery stub --effort <slug> --from <text>` creates a minimal
map: Destination reads "(unknown — charting session needed)", the
`--from` text lands under Notes, the remaining sections are empty. It
refuses, writing nothing, when the effort directory already exists.
Shaping's headless triage calls it when an item parks for vagueness
(not for risk), so a parked fog becomes a visible map instead of a
buried note.

## Surfacing — resume is un-missable

`bee discovery list` reports each effort's name, destination line, and
open/frontier counts; an unreadable map yields a visible
"unreadable <path> — remedy: fix or delete" line, never a crash. The
same scan feeds three surfaces: the status JSON (`open_maps` field) and
its text section, the session preamble (rendered in-process by the
session hook, not via the status command), and orient. When no
discovery directory exists, no section appears anywhere.

Orient's rule is deterministic: when at least one map has frontier
tickets AND the pipeline is idle (no open or claimed cells, no pending
handoff, terminal phase), the recommended next step becomes the
wayfinding skill with the map named; while work is active the map
degrades to a report-only blocker line and the recommendation is left
alone. A pending handoff always wins over the wayfinding override.

## Resolution craft

How a ticket is worked has its own rules (wayfinding-craft wayfc-1,
2026-08-17): the interview craft mirrors shaping's — one question per
turn, options carried with trade-offs, the user's words quoted back
before a decision is drawn. A pure fact question never burns an
interview turn: it dispatches as a fact lookup and the answer returns
to the ticket, keeping the human conversation for judgment calls only.
Spike-type tickets carry their own rules — time-boxed, one yes/no
proof, thrown away after the answer is recorded — and the craft text
cross-points to shaping's gray-area probes so the two interview layers
stay one discipline.

## Hand-off to shaping

Wayfinding decides; it never builds. Each resolved ticket's answer is
recorded in the ticket file and logged as a decision; the map links the
decision id. When no tickets and no fog remain, each buildable feature
falls out of the map into shaping's Lock, which consumes the map's
settled decisions into the feature's context record, citing them —
never re-asking. Shaping's own entry gained the mirror rule: a request
with no nameable outcome routes to wayfinding before any interview.
Wayfinding adds no gate of its own; the human checkpoints are the
destination-naming conversation and shaping's existing Gate 1.

## Flow

```mermaid
flowchart TD
    A[Fog-state idea] -->|explicit invocation| C[Chart: name destination, sweep wide, write MAP.md]
    A2[Vague backlog item parks] -->|discovery stub| M[Map stub visible in status]
    M --> C
    C --> T{Frontier tickets?}
    T -->|yes, session resumes| R[Resolve ONE ticket: grilling / research / prototype / task]
    R --> L[Answer in ticket + decision logged, map gists]
    L --> G[Graduate fog into new tickets, prune invalidated ones]
    G --> T
    T -->|no tickets, no fog| H[Hand decisions to shaping Lock — normal chain]
```

## Open Gaps

- No staleness nudge for an idle map yet (deferred: triggers-based
  reminder).
- Orient recommends only on resume; brand-new fog still enters via
  explicit invocation or shaping's entry check.
- Ticket claim/block conventions are unguarded; a malformed ticket file
  simply drops out of the frontier count.

## Pointers

- `packages/bee-rs/crates/bee/src/verbs/discovery.rs` — scan, frontier
  derivation, stub creation, list/stub verbs.
- `packages/bee-rs/crates/bee/src/verbs/status_full/{build,render}.rs`,
  `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` —
  the three surfacing sites.
- `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` —
  `pipeline_idle_for_wayfinding`, `first_frontier_effort`.
- `skills/bee-wayfinding/SKILL.md` ("Interview craft"),
  `skills/bee-wayfinding/references/wayfinding-reference.md`
  ("Interview craft", "Spike rules") — the resolution-craft text;
  cross-points to `skills/bee-shaping/references/gray-area-probes.md`.
- `skills/bee-wayfinding/` — the flow's skill; templates in
  `references/wayfinding-reference.md`.
- `skills/bee-shaping/SKILL.md` — entry check, park-to-stub, Lock
  consumption.
