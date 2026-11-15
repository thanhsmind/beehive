---
name: bee-principle-sequence-verifiable-units
description: "Apply to multi-step work — sweeps, migrations, runs of similar edits — and to how you order commits and cells. Break the work into small units that each end in a checkable state, check each before the next, and order delivery so the sequence proves itself."
---

# Sequence Work Into Verifiable Units

Order the work as small units that each end in a state you can check, and do
not advance until the current unit is green. The same discipline runs at two
altitudes: how you execute and how you deliver.

In a sweep, a migration or any run of similar edits, verify each change before
starting the next; never batch the edits and verify once at the end. Each unit
is a before and after bracket: a known-good state, one change, the check, then
proceed. Start from a clean base so every check measures against the real
baseline.

Deliver in the order that proves the work. The canonical shape is the failing
test first and the fix on top, so a reviewer sees the bug red and then
resolved. Other story orders: a subtraction before the reshape, a baseline
capture before the treatment, the scaffold before the feature. Each commit
stands on its own and the sequence reads as an argument.

**Check:** what is the smallest unit here that ends in a check, and did you
run that check before starting the next one?

**Not when:** a single small change already is the unit; there is no sequence
to order. A step that cannot be checked alone belongs inside the unit before
it (see split-on-a-real-dependency).

**Why:** a break caught at the unit that caused it is cheap to localize; a
break caught after a batch is buried under work built on a broken base.

**Depth:** `.bee/expertise/planning.md` § Sequence work into verifiable units.
