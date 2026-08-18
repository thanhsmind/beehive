---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

D06 says the scenario ends by collecting what each agent produced.
Collected to where?

## Answer

The core returns typed results and records nothing — D05 makes that
structural, not a choice. On the bee side, one append-only wave ledger:
one row per wave, in the shape `.bee/` already uses everywhere
(`backlog.jsonl`, `decisions.jsonl`, `capture-queue.jsonl`).

A row carries the wave id, when it started, and per worker: name, pane
id, worktree, the task it was given, its outcome, and a pointer to the
evidence. Nothing else. Results do not become cells, decisions, or
proof lines — a cell has an owner, a claim and a proof of its own, and a
wave is not a cell.

The argument that decided it is not tidiness. The cockpit's four-slot
cap is enforced today by the control model **counting panes**, and an
agent that fails to name itself leaves a slot looking free, so the next
iteration over-spawns — a recorded Open Gap in
`docs/knowledge/areas/bee-herding/overview.md`. Pane-counting is a bad
source of truth. A wave ledger makes occupancy readable instead of
counted, which closes that gap mechanically rather than by care.

It also means an owner reading a day later can see what happened, and a
wave that dies mid-flight still leaves a row saying so.

Cost, named: one more file to keep honest and to sweep. Accepted
because it **replaces** a worse source of truth rather than adding to
one.

Rule carried forward: the core stays ignorant; a bee-side adapter maps
a typed result to a row. If a later caller wants cell caps out of a
wave, that is a second adapter, never a change to the core.

Logged as D10.
