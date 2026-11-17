---
name: bee-principle-lock-the-contract
description: "Apply when you write a cell's action or a plan unit. Lock the contract the work must keep — inputs, outputs, persisted state, error shape, the tests that prove it — and leave the implementation to the worker."
---

# Lock the Contract

A plan is written before anyone has read the code closely enough to know how
it should change. It is good at saying what must stay true and bad at saying
how to write it. An action that prescribes helper names, a call graph or
line-level edits turns those early guesses into rules the worker must follow.

So the action names the outcome and the contract the work must keep — inputs,
outputs, persisted state, error shape, and the tests that prove it — and stops
there. A structural choice enters the plan only when it crosses a shared
contract that another part depends on. Read anchors that say where to look
stay: they point, they do not bind.

**Check:** for each line of the action, if the worker did it differently and
every contract test still passed, would anything break? If not, the line is
implementation — cut it.

**Why:** a detail the plan never proved turns into a constraint the moment
the action prescribes it, and the worker who reads the code is then bound to
the guess.

**Depth:** `.bee/expertise/planning.md` § Lock the contract.
