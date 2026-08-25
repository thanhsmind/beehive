---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

D5 (e9cb4c15) requires a cell that followed its plan to say so
explicitly. Decision dc6a2d26 promises the opposite for the existing
flag: "no flag = today's behaviour byte-identical". Both cannot hold
unchanged. Which gives:

(a) `bee cells finish` starts refusing a cap that states neither a
    departure nor "no departure" — honest, and it breaks every existing
    caller until they are updated;
(b) the explicit statement is required only while the mailbox is armed
    for the run, leaving unarmed caps byte-identical;
(c) the absent field is read as "no departure" and D5's explicitness
    requirement is dropped — cheapest, and it restores exactly the
    silence D5 exists to remove.

➡️ Recommendation: (b). It keeps D5's guarantee everywhere the letter is
actually written, which is the only place the human reads it, and costs
nothing to callers that are not producing letters at all.

## Answer

(b) — logged as **D10 (1fb69f4b)**. The explicit no-departure statement
is enforced only while the mailbox is armed for the run; a cap in a run
that files no letter keeps the byte-identical behaviour dc6a2d26
promised for the flagless case. The collision between D5 and dc6a2d26 is
resolved.
