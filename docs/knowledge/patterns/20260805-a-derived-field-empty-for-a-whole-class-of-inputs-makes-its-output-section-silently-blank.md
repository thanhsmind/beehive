---
type: bee.pattern
title: A derived field that is empty for a whole class of inputs makes its whole output section silently blank
description: "A section whose content is keyed on a derived field renders silently empty for every input the field never gets set for, and two independent empty-derivations can stack behind the very same always-empty render."
tags: [verification, derived-fields, silent-empty, promote, knowledge-layer]
timestamp: 2026-08-05
bee:
  id: pattern-20260805-a-derived-field-empty-blanks-its-output-section
  lifecycle: active
  areas: [okf-profile]
  decisions: ["2fa69010 (open gap: a history-anchored `bee knowledge promote` proposes zero area bullets, proven on knowledge-loop's own close)", 86d96c9f (promote takes its areas from the scribing ledger when the work item names none)]
  sources: ["packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs (area-update section keyed on the work item's bee.areas)", "decision 2fa69010 (2026-08-05: proven on knowledge-loop's own close — 5 capped cells mined, 0 area bullets, 0 pattern candidates)", "promote-reach cell pr-1 (commit ab000d54: promote falls back to the scribing ledger for areas when the work item names none)", "promote-reach commit 30550b39 (attribute every capped cell to every scribing-ledger area instead of per-file matching, since touches_subject never matched)", "docs/knowledge/areas/okf-profile/context-and-promote.md (decision 86d96c9f: only 5 of 95 area concepts carry a code path in bee.sources, and only 10 of 95 mention packages/ at all)"]
  polarity: pitfall
  critical: true
---

# A derived field that is empty for a whole class of inputs makes its whole output section silently blank

A section whose content is keyed on a derived field renders as empty for every input where that
field never gets a value — no error, no warning, just a section that always says "none". Worse,
the empty section can have two independent causes stacked, and fixing one leaves the render
unchanged for a reason nobody has named yet.

The instance: `promote`'s area-update section keys on the work item's `bee.areas`. Every feature
reached through the history anchor has no work item and therefore no `bee.areas`, so the section
rendered "None" for all of them — proven on `knowledge-loop`'s own close: 5 capped cells mined,
0 area bullets, 0 pattern candidates. When the area list was fixed — falling back to the feature's
`.bee/logs/scribing-runs.jsonl` stamp — the bullets STILL stayed empty, for a second, independent
reason: subjects are built from `bee.sources`, which holds prose citations and
`docs/specs/*.md#B5`-shaped anchors, and only 5 of 95 area concepts carry a code path in
`bee.sources` at all, so the per-file `touches_subject` match never fired. Two unrelated
empty-derivations stacked, and the only symptom either produced was the same silent "none".

## The rule

- A section that renders "none" needs its OWN test asserting a non-empty case, not just a test that
  the empty case doesn't crash. An always-empty section and a correctly-empty section produce
  byte-identical output.
- When a fix to one empty-derivation does not change the visible output, check whether a second,
  independent empty-derivation is still active before concluding the fix didn't work — or that it
  did.
- Structural matching against free-text fields (`touches_subject` against `bee.sources` prose)
  fails silently rather than loudly; measure the real hit rate (5 of 95 here) before trusting the
  match as a signal at all.

Fixed in `promote-reach` (cell `pr-1`, commit `ab000d54`; and commit `30550b39`, attributing every
capped cell to every scribing-ledger area instead of the per-file match that structurally could
not fire).
