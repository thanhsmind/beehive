# backlog-anchor — locked context

Lane: small · Route: feature / covered-contract-change / 4 files
Decision ledger: a98e27c2 (scoping), 71ec4252 (sibling: shaping reads the bundle first)

## Problem

`bee knowledge context` and the session-preamble invitation resolve a
work anchor through three arms (work-item concept, `docs/history/<work>/`,
ledger traces). During exploring none of those exist yet — CONTEXT.md is
what exploring produces — so the curated manifest cannot fire at the
moment initial context is most needed.

## Locked decisions

- **D1 (a98e27c2)** — `resolve_anchor` gains a fourth, last arm
  `Anchor::Backlog`: a folded `.bee/backlog.jsonl` PBI row whose `id` or
  `feature` field whole-matches the requested work. `meta` = title,
  `body` = detail/cos, tags/areas empty (same degraded ranking as
  History/Ledger). Arm priority: WorkItem > History > Ledger > Backlog —
  the backlog row is the thinnest text, so it is the last resort.
- **D2** — `drivers/kctx.rs` (hand-kept byte-parity port) mirrors the
  arm in the same change.
- **D3** — the session-preamble invitation needs no change: it already
  fires whenever ANY resolver arm answers (knowledge-in-flow D1,
  38b4153a).

## Open questions

None.
