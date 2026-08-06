# debt-door-archive — a debt counter that cannot be silenced by archiving

## The defect

Every scribing-debt counter in the tree enumerates `.bee/cells/<feature>` and
skips directory entries, so `.bee/cells/archive/<feature>/*.json` is invisible
to all of them. `bee close` archives a feature's cells on a green close. From
that moment the count is structurally zero and a clear door is
indistinguishable from a paid debt.

Measured live: `doc-viewer-links` closed with `door scribing-debt: clear`
while both of its `behavior_change` cells were uncaptured and
`docs/knowledge/` held no `doc_viewer` content at all.

## Blast radius of the fix — measured, not estimated

Counting archived cells against every feature's best scribing stamp today
yields **0 features and 0 cells** of newly-surfaced debt. No amnesty stamp is
needed; the alarm starts silent.

## The four counters

| Site | Enumerator | Feeds |
|---|---|---|
| `verbs/drivers/close.rs:347` | `guard.rs:175 list_cells` | `bee close`'s door, the close refusal, and both `set_gate` doors (close + feature swap) |
| `verbs/status_full/cells.rs:341,370` | `cells.rs:156 list_cells` | `bee status` per-feature debt and the global orphan sweep |
| `hooks/chain_nudge.rs:803` | `:686 list_cells_filtered` | the chain-nudge hook line |
| `hooks/session_preamble/store.rs:111,137` | `:25 list_cells` | the session-start debt line |

All four are the same loop over the same store, and they must never disagree
about what counts as unpaid — the codebase already says so at
`set_gate.rs:140-142` ("rather than a second, independently-drifting copy").

Precedent for the archive-aware walk already exists and is pinned by a test:
`verbs/knowledge/promote.rs:353-376` reads `.bee/cells/archive/<feature>/`
with live-copy-wins dedup (`verbs/knowledge/tests.rs:682`).

## Shape

Two cells, `dda-2` after `dda-1`.

**dda-1 — the doors.** Make the debt enumeration behind
`drivers::scribing_debt` archive-aware, live copy winning on a duplicate id.
The generic `list_cells` used by `bee cells list` stays active-only: this is a
debt-counting change, not a listing change.

**dda-2 — the reporting surfaces, then parity.** The same archive-awareness in
the three remaining counters, then one parity test: over a single fixture
holding a hot cell, an archived cell, and a duplicate id in both places, all
four counters return the same count and the same ids. The parity test is the
real deliverable — it is what stops the fifth copy from drifting.

## Out of scope

`bee cells archive`'s own precondition stays "every cell terminal". With the
counter fixed, archiving can no longer hide debt, so a second door there would
be ceremony.
