---
type: bee.pattern
title: "Evidence is checkpointed to disk per step, never held in context until the end"
description: "Evidence is checkpointed to disk per step, never held in context until the end"
tags: [failure, iron-law, workers, context]
timestamp: 2026-07-10
bee:
  id: pattern-20260710-evidence-is-checkpointed-to-disk-per-step-never
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT12", "original feature: evolving-loop"]
  polarity: pitfall
  critical: false
---

# Evidence is checkpointed to disk per step, never held in context until the end

An Iron Law worker edited `SKILL.md` and died before writing its RED pressure-test report; the edit
was reverted, because an unrecorded RED phase is not a RED phase and reconstructing it from the
worker's summary would be fabricating evidence. Its successor checkpointed each scenario to disk as
it finished, was interrupted mid-run, and lost nothing. Write each scenario, each proof, each
observation as it lands. Note also that `grep '## RED'` passes on a `touch`, and one commit holding
RED+GREEN proves no ordering — commit RED separately.
