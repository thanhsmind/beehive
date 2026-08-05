---
date: 2026-08-05
feature: doc-viewer-links
categories: [hook-runtime, workflow-state]
severity: medium
tags: [session-briefing, compaction-survival, opt-in-config, close-doors, capture-debt]
---

# doc-viewer-links — a fact the session must not lose has to survive compaction, and a door that counts nothing reports clear

## What Happened

An opt-in `doc_viewer` key (`base_url` + `project`) now turns every doc
reference the agent writes into a clickable viewer URL. Two cells shipped it:
`dvl-1` put one shared reader in `state.rs` and injected the prefix into both
agent-facing briefings; `dvl-2` wrote the obligation into the agent contract
and documented the key.

The feature then closed with `door scribing-debt: clear` while
`docs/knowledge/` contained no `doc_viewer` content at all. The capture was
written afterwards, during a stale-lane sweep, only because someone grepped
the bundle by hand.

## What Was Learned

**A fact the agent must honor for a whole session belongs in the compaction
capsule, not only in the startup briefing.** The prefix is injected twice on
purpose. The startup briefing teaches it; the capsule re-teaches it after
compaction. Without the second injection the agent silently reverts to bare
paths at exactly the moment it has forgotten why it stopped writing them —
and nothing about that reversion looks like a failure. Any opt-in behavior
carried as *instruction text* rather than as *enforcement* has this same
half-life, and the capsule is where it is renewed.

**bee joins, it never encodes.** The reader normalizes only the separators
that would collide at the join — one trailing separator off the base, all
leading and trailing ones off the project. Nothing else is rewritten and the
appended path is never percent-escaped. Guessing at the author's intent in a
URL is a worse failure than a link the author must fix once.

**Half-set is loud; unset is silent.** An absent key is a decision and says
nothing. A key with one usable half is someone's unfinished intent and earns
exactly one warning line naming both halves. The distinction is worth the
extra branch: silence on a half-set key would strand the author with no
symptom to search for.

**A door that enumerates a moving set reports on the set, not on the
question.** `bee close`'s scribing-debt door counts capped `behavior_change`
cells found in the hot `.bee/cells/` scan. Once a feature's cells are archived
that scan returns nothing, so the door reports `clear` — not because the debt
was paid but because the count had nothing to count. The failure is invisible
by construction: a clear door and a paid debt render identically. Recorded as
a P1 in `.bee/backlog.jsonl`. The general shape, worth carrying: **when a
guard's verdict is computed from an enumeration, an empty enumeration must be
distinguishable from a satisfied condition — otherwise retiring the records
silently retires the guard.**

## Evidence

- Cells `dvl-1`, `dvl-2` (traces `.bee/cells/archive/doc-viewer-links/`).
- Behavior captured in
  `docs/knowledge/areas/hook-runtime/doc-viewer-links-in-agent-briefings.md`
  (B24-B26, R24-R25).
- Door blind spot: `close.rs:347-361` with `guard.rs:169-183`;
  archival gate `handlers_meta.rs:596-613`.
