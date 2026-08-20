# Herding Review Slots — Context

**Feature slug:** herding-review-slots
**Date:** 2026-08-20
**Shaping session:** complete (user screenshot + relayed ask: review/advisor by herd name)
**Scope:** Quick

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `{kind:"herding", agent:"<name>"}` on a review or advisor slot resolves to the herding-exec Bash payload (one read-only job through the herd pane, mailbox result); gather/extraction purposes on the same slot keep the runtime-default-model fallback. Widens herding-executor D7's cell-only scope (decision logged touches:herding-executor) | The user's target: review → agy-flash, advisor → named herd. A review is one task in, one result out — the run verb's exact shape. Bulk gathers stay off panes |
| D2 | Spelling stays `{kind:"herding", agent:"<name>"}` on every slot — never a separate kind, never promptVia (the brief travels by mailbox, not stdin) | One kind, one resolver, no duplicate grammar |
