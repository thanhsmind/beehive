---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

D4 guarantees appended entries survive a run that dies at 3am, but
nothing composes a letter for a run that never reaches its own end. Who
files that letter, and what may it honestly say? Candidates: the next
session that starts notices the orphaned entries and files the letter;
a letter is filed on a schedule regardless of run state; or the entries
simply sit unfiled until the human opens the mailbox and something
renders them.

Whatever files it must be able to say "this run did not finish" without
guessing why.

➡️ Recommendation: the next session that starts files it, marked plainly
as an unfinished run, listing the entries up to the last one and naming
the moment the run went silent. It needs no new scheduler, and the next
session is the first moment anyone can observe that the run stopped.

## Answer

The next session that starts files it, marked plainly as an unfinished run, listing entries up to the last one and naming the moment the run went silent — logged as **D12 (05b5f964)**.
