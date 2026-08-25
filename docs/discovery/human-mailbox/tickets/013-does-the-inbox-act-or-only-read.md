---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

D7 gives the letter a "Needs your call" section. Is that section only
*text the human reads*, or is each item addressable — carrying an id the
inbox can answer against, so a reply from the inbox reaches the waiting
session? The UI belongs to waggledance under D1, but whether the item is
addressable at all is a bee-side data question and has to be settled
before the handover spec is written.

➡️ Recommendation: make each "Needs your call" item addressable in the
frontmatter (a stable id plus what it is blocking) but ship no answering
path in this effort. It costs one field now and is the difference
between an inbox that can grow into a reply surface and one that can
never be more than a reading list.

## Answer

Addressable, with no answering path in this effort — logged as **D13 (c3ece144)**. Each Needs-your-call item carries a stable id and names what it blocks.
