# SLP Blind Lanes — Context

**Feature slug:** slp-blind-lanes
**Date:** 2026-08-28
**Shaping session:** complete
**Scope:** Deep
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Blind lanes: a procedure over bee's existing dispatch door that opens 2–3
isolated advisor lanes on one high-stakes ambiguous decision, cross-critiques
their proposals, and converges them into a dossier plus one decisions-log
entry with a registered revisit trigger. It ends at that record — it never
becomes a standalone agent layer, and it never relaxes the merge, interlock,
or permission-posture rules.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an explicit
supersession note, never a silent edit.

Source: `docs/discovery/slp-supervisor-lead-peer/MAP.md` (cluster 3 of the
build order) and `docs/discovery/slp-supervisor-lead-peer/tickets/006-blind-lanes.md`.
D-IDs below are local; the store ids they render are cited in each row.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The agent opens 2–3 blind design lanes on ITS OWN judgment when a decision is both high-stakes AND ambiguous, and logs the reason at open time. The user may also order lanes directly. A deadlocked convergence always hands the user the dossier, never a coin flip. (store `9cffdfb5-5f26-48b9-806b-c460eec41b16`) | Lane cost is acceptable without pre-approval as long as the open reason is logged and deadlock still escalates. No approve-each-lane wait. |
| D2 | Blind lanes are a PROCEDURE over the existing dispatch door, not new machinery: (a) one neutrality-linted LaneBrief, the lint enforced at the dispatch door as a lexical prose-guard refusal; (b) 2–3 parallel `--kind advisor` dispatches carrying a byte-identical brief and an explicit read-only path diet; (c) cross-critique as a SECOND advisor round handing each lane the rival proposal verbatim; (d) convergence as a dossier doc plus one `bee decisions log` entry whose revisit condition is a registered `bee triggers` id; (e) deadlock hands the user the dossier through a `waiting-on --kind question` mark, or a human-mailbox letter when unattended. (store `5981246b-381f-4430-be27-bbf29c51754c`) | Advisor-kind dispatches are already isolated by construction — read-only, ephemeral, output only in the final message. The two genuinely missing pieces are the brief lint and a structured rejected-set on the decision record. |
| D3 | Blind lanes NEVER run as `--kind cell`. | The worker-cell template injects machine-assembled `learned_context` and prior-round trace, which is shared memory across workers and leaks the very thing blindness protects. (part of `5981246b`) |
| D4 | Convergence carries an anti-fabrication check: every citation in the dossier must resolve against the verbatim lane proposals it synthesizes. The check is mechanical (string containment against the source payloads). (store `5144314c-b7b0-4967-b628-f989dd16ea2a`, clause b) | A multi-perspective fan-out stays honest only when the synthesis is validated against its sources. |
| D5 | An objection or pushback is valid only when it names the specific missing context. (store `5144314c`, clause c) | — |
| D6 | The 5-Layer rubric (Data Contract / Happy Path / Failure-Edge / NFR / Definition-of-Done), the Truth Table Test (every IF needs its ELSE) and the CRUD Lifecycle check join the reviewer/judge checklist material. (store `5144314c`, clause a) | Imported as rubric and checks only — bee-reviewing already carries its specialist reviewers. |
| D7 | Hats and lanes stay distinct instruments: LANES generate designs from a byte-identical brief; HATS critique one request from fixed disjoint perspectives. Hats never replace lanes. (store `5144314c`) | Different purpose — critique versus generate. |

### Agent's Discretion

D1 delegates the lane-opening call to the agent, bounded by two constraints:
the decision must be both high-stakes and ambiguous, and the reason must be
logged at open time. The user may still order lanes directly at any point.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Blind lane | One isolated `--kind advisor` dispatch that designs an answer to the LaneBrief without seeing a sibling lane's work or the orchestrator's leaning |
| LaneBrief | The single question payload every lane receives byte-identical, after passing the neutrality lint |
| Neutrality lint | A bounded lexical prose-guard scan over the LaneBrief that refuses leaning language at the dispatch door |
| Read diet | The explicit list of paths a lane may read, carried in its brief |
| LaneProposal | One lane's returned design, kept verbatim for cross-critique and for the dossier's citation check |
| Cross-critique | Round two: fresh advisor dispatches, each handed the rival LaneProposal verbatim |
| Convergence dossier | The document holding the verbatim proposals, the critiques, the chosen answer, and the rejected set with reasons |
| Deadlock | Convergence that produces no chosen answer; the dossier goes to the user unchanged |
| Hat | A fixed critique perspective applied to ONE request — a review instrument, never a lane |

## Specific Ideas And References

- The user picked more autonomy than the ask-first option in the 2026-08-26
  grilling round: lanes open without pre-approval, the reason is logged.
- bee-reviewing's wave is the nearest existing precedent — parallel reviewers
  on a stated information diet, synthesis only after all return. It critiques
  one artifact and generates no alternatives, so it is a pattern to copy, not
  a mechanism to reuse.

## Existing Code Context

From the advisor-tier research digest only. Downstream agents read these
before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:29-51` — the dispatch
  door: two runtimes, four kinds (`cell`, `gather`, `reviewer`, `advisor`), each
  mapped to a model slot by `slot_for_kind` (:44-51)
- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:115-117` —
  `purpose_is_gather`: every non-cell kind is read-only by construction, and
  `--claim` is refused for them
- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:306-355` and
  `:359-432` — the two prose guards: the supersession-stem scan and the deferral
  scan that refuses a "revisit when" decision with no `--trigger`. This is the
  lint pattern the LaneBrief guard copies
- `packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:11,22-33` — the revisit
  registry `bee triggers add --decision <id> --condition <text>`
- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:202-203` — the
  decisions-log record fields: decision, rationale, alternatives, tags, relation,
  trigger

### Established Patterns

- Bounded lexical scan with a typed refusal that names its remedy — the decisions
  prose guards; the LaneBrief neutrality lint is the same shape
- Information-diet fan-out then synthesis — `skills/bee-reviewing/SKILL.md:38-46`
  and its independent-corroboration promotion at `:58-63`

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — where the lint
  refusal attaches so neutrality is enforced at the chokepoint, not by discipline
- `packages/bee-rs/crates/bee/src/verbs/state/set_gate.rs` — Gate 3 already
  requires a recorded advisor consult for high-risk work
  (`high_risk_advisor_refusal`, `bee state advisor-ref record`): one mandatory
  consult today, the natural hook point for a multi-lane rung
- `bee state waiting-on set --kind question` and the `.bee/human-mailbox` letter
  path — the two deadlock channels named by D2(e)

## Canonical References

- `docs/discovery/slp-supervisor-lead-peer/MAP.md` — the closed map; cluster 3
  of its build order is this feature
- `docs/discovery/slp-supervisor-lead-peer/tickets/006-blind-lanes.md` — the
  grilling ticket that produced D1 and D2
- `docs/history/research/slp-blind-lanes-surfaces.md` — the advisor-tier surface
  digest every code anchor above comes from
- `docs/history/research/agent-harness-slap-distill.md` — the harness distill
  behind D4, D5, D6, D7
- `docs/knowledge/areas/bee-herding/overview.md` — the standing constraints R2
  (merge stays a human gesture), R3 (owner interlock), R4 (permission split)

## Outstanding Questions

### Resolve Before Planning

None. The map records "Not yet specified: Nothing" for this cluster.

### Deferred To Planning

- [ ] Does the neutrality lint attach as a new `--kind lane` at the dispatch
      door, or as a refusal on the existing advisor kind? — D2(a) locks the
      chokepoint, not the spelling; reading `prepare.rs` answers which fits
- [ ] Does the rejected set become a structured `rejected[]` field on the
      decision record, or a dossier-source convention over today's flat
      `alternatives` string? — the research names both; the record schema
      decides
- [ ] Where does the dossier document live when the lanes do not belong to a
      `docs/discovery/<effort>/` map? — the research assumed a discovery slug
- [ ] What forbidden-phrase set does the neutrality lint carry, and how is it
      proven red-first? — a lexical guard needs its own failing case

## Deferred Ideas

- Heterogeneous lane models (spec §10.b) — breaks the one-name advisor slot
  (decision `4faf1de9`, `prepare.rs:96-97`); returns only as a fresh effort
- Building SLP as a standalone six-agent layer with the spec's message names —
  the user chose merge-into-bee (`787a9eb0`)
- Relaxing R2, R3 or R4 for lane autonomy — out of scope by the map

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, canonical references, and deferred-to-planning
questions. Planning's Gate 2 shape stage and reviewing use locked decisions for
coverage and UAT.
