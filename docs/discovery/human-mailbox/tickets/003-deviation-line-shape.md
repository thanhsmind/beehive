---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

`deviations[]` already exists on every cell's Result form, but it is
written agent-to-agent, terse, and requires no reason — while the reason
is the whole point of the human's ask. Does the deviation line become a
required three-part shape — *what was done differently — why — which
kind* — with the kind drawn from a closed set (hit an unforeseen
obstacle / found a better route / the plan was wrong about a fact /
something outside the plan had to be fixed first)? And does a cell with
an empty `deviations[]` have to say so explicitly ("did what the cell
said") rather than leaving the field silently empty?

This is the only part of the effort that changes *capture* rather than
rendering, so it is the one that touches the worker contract.

➡️ Recommendation: yes to both. A three-part line with a closed kind
set, and an explicit "no departure" rather than an empty field —
silence and "nothing happened" must not look identical to the reader.

## Answer

Yes to both — logged as **D5 (e9cb4c15)**. Three required parts (what was
done differently — why — which kind), kind from a closed set: hit an
unforeseen obstacle / found a better route / the plan was wrong about a
fact / something else had to be fixed first. A cell that followed its
plan says so explicitly rather than leaving the field empty.

D5 narrows the free-form one-line `--deviation` value that decision
dc6a2d26 established. Its "no flag = today's behaviour byte-identical"
clause and the new explicit no-departure statement collided; **D10
(1fb69f4b)** resolved it — the explicit statement is enforced only while
the mailbox is armed for the run, and an unarmed cap stays
byte-identical (`tickets/010-no-departure-vs-unchanged.md`).
