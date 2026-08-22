---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

In bee-on-bee work, how does bee detect that a change touched an area's
code but not that area's skill or expertise — what maps code paths to
skill paths, and what happens on a miss (refuse cap, demand a reason,
or only warn)? User is unsure of the mechanism; needs options.

## Answer

Option C (user, 2026-08-22): plan prediction + cap check.
- Each area spec declares in frontmatter the code paths and skill
  paths it owns (prerequisite: the map does not exist yet — ticket 004).
- Plan records predicted affected skills/specs per cell, or none.
- At cap bee compares the real diff against the map; a miss or a
  disagreement with the prediction refuses the cap until a reason is
  recorded. Warning-only rejected.
Logged as decision 3ea7500a.

