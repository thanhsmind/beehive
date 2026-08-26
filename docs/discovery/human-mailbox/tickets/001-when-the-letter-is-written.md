---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

At what moment does the letter come into existence? Three candidates:
(a) composed once, when the run ends; (b) two layers — every clean stop
(a cell capped, a feature closed, a blocker hit) appends a raw entry
immediately, and the letter is composed from those entries when the run
ends; (c) nothing is composed until the human opens the mailbox, and the
letter is rendered on read.

The load-bearing case is the run that dies or hangs at 3am. Under (a)
that night leaves nothing.

➡️ Recommendation: (b). The human's instinct — "write it the moment the
work is done" — is right about *capture* and wrong about *composition*:
an overnight run has many "done" moments and no single one. Append at
each stop, compose at the end, and a dead run still leaves a readable
trail of everything up to the moment it died.

## Answer

(b) Two layers — logged as **D4 (1d56c1d2)**. Every clean stop appends its
raw entry the moment it happens; the letter is composed from those
entries when the run ends. Capture immediate, composition at the end, so
a run that dies at 3am still leaves everything up to the moment it died.

Refined by **D9 (d970d6fc)** and **D12 (05b5f964)**: every session
appends entries, attended or not, but only an unattended run composes
and files a letter — and a run that dies before its own end gets its
letter from the next session that starts, marked as unfinished.
