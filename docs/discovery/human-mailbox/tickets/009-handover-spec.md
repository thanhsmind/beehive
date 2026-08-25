---
type: task
status: closed
blocked-by: 005-record-format, 006-letter-sections, 007-read-state-and-retention
claimed-by: none
---

## Question

Write the handover spec D1 promises: the record shape, the field
contract, the naming scheme, the lifecycle, and the guarantees bee makes
about them (what is always present, what may be absent, what may change)
— in a form the waggledance side can build an inbox against without
reading bee's internals. Where does it live: bee's docs, waggledance's,
or both?

➡️ Recommendation: authored in bee under `docs/knowledge/`, copied into
waggledance as a versioned contract file at handover — bee is the
producer and owns the shape; waggledance needs a pinned copy it can
build against.

## Answer

bee is a development harness used to build waggledance — two separate
projects, not two halves of one system. bee authors the record
description in its own docs (this map is that description), and the
handover crosses by **messaging the waggledance session**, which writes
its own backlog row and owns the work from there. A bee session never
writes a row into another project's backlog on its behalf.

Logged as **D17 (1660158a)**, replacing D16 (e255fe3a), which rested on
the wrong premise — that bee and waggledance were one system — and on
the wrong route.

Delivered: handover message sent to session `waggledance-e1`, carrying
the full record contract and pointing here for the rest. Accepted and released to
that session — the handover is delivered. The PBI this
session had already appended directly to waggledance's backlog
(p-e9386ebb, same content) was flagged in that message for the
waggledance side to adopt, amend or decline — it is theirs to decide.
