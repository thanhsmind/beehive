---
type: grilling
status: open
claimed-by: none
blocked-by: 002-isolation-class-enforcement.md
---

## Question

Two joined questions about the per-file contract.

1. What does the header literally look like? A comment block the
   convention defines (portable, but unenforced), a runner-native
   attribute (enforced, but different in every stack), or both — the
   comment as the human layer, the attribute as the machine layer.
2. Is there a size cap, and is it counted in lines, in tests, or in
   behaviors? Bee's own worst file is 6042 lines / 178 tests. Name the
   number at which a file must split, and whether the cap is a hard
   refusal or a soft nudge.

001 has reported, so the parallel-unit half is settled; this ticket
now waits only on 002 for the enforcement mechanisms the machine layer
would use.

## Answer

<open>
