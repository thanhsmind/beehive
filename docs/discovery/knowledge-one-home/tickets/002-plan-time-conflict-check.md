---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

What does the plan-time conflict check look like concretely: which
inputs (plan text, cell titles, touched paths), which store (decisions
active, area specs), and what the gate record must carry — a literal
"0 conflicts" line, or named decision ids with a verdict each? Leaning
(user, round 1): before the shape gate, result recorded on the plan.

## Answer

(user, 2026-08-22) Decision efd6cbaa:
- bee derives candidates from the plan (cell titles, touched paths,
  area tags) via `decisions active` and D4's `applied_at` lists.
- Each candidate gets a verdict on the plan: compatible / conflicts /
  retires-prior <id>. "0 conflicts" is valid only when bee returned
  zero candidates.
- `gate --merge` refuses while a candidate lacks a verdict (same shape
  as the high-risk advisor_ref precondition); `plan-rev bump` resets the
  verdicts.

