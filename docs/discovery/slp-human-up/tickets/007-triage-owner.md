---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

d2701784 fixes the triage rule (one project → existing lanes; many →
A-spec path) but not its owner or surface. Who answers the one-vs-many
question at intake, and where does the intake live: the waggledance-layer
supervisor as it relays the human's ask, the first lead that receives it, or
an extension of bee's route classifier? And who picks WHICH repo's lead
becomes A when the work spans several?

## Answer

Resolved 2026-08-29 by the user (decision 4dda03e0): every human ask names
its target project explicitly at drop time — no agent triages which repo an
ask belongs to. The named repo's lead is A by default; the one-vs-many
question collapses into a discovery A makes while planning (d2701784's
A-spec path opens only then), and the human may name a different A when
dropping. The supervisor's only role at intake is its usual one: a question
if something looks mis-aimed.
