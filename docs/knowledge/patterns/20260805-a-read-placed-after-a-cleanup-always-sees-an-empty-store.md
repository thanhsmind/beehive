---
type: bee.pattern
title: A read placed after a cleanup always sees an empty store
description: "A read-only step sequenced after a cleanup step sees whatever the cleanup left behind — a soft promote door placed after cell archival on close always scanned an emptied store, and the happy-path test only passed because it disabled the archiving the real path always runs."
tags: [ordering, cleanup, read-after-write, verification, knowledge-layer]
timestamp: 2026-08-05
bee:
  id: pattern-20260805-a-read-placed-after-a-cleanup-sees-an-empty-store
  lifecycle: active
  areas: [workflow-state]
  decisions: ["29c40516 (knowledge-loop D9 — compute the promote proposal before auto_archive_on_close retires the feature's cells)"]
  sources: [packages/bee-rs/crates/bee/src/verbs/drivers/close.rs (soft promote door inserted after auto_archive_on_close at the line the plan pinned), "decision 29c40516 (2026-08-05: with cells_archive_on_close at its default true, a real close archives the just-capped cells into .bee/cells/archive/ before build_promotion scans .bee/cells/*.json, so the close's own proposal comes back empty)", "knowledge-loop cell kl-3 (commit 384587a1: give bee close a soft promote door)", "knowledge-loop cell kl-5 (commit c8d25dff: run build_promotion before close retires the feature's cells)", docs/knowledge/areas/workflow-state/gates.md]
  polarity: pitfall
  critical: false
---

# A read placed after a cleanup always sees an empty store

Ordering a read after a cleanup step, because that is the line the plan pinned, is a decision — not
a detail — the moment the cleanup empties the very store the read scans. A read-only operation
costs nothing to move earlier, so there is no reason it should ever run after the step that empties
its own input.

The instance: the soft promote door (`bee knowledge promote`, run in-process at `bee close`) was
inserted after `auto_archive_on_close`, at the line the plan pinned. With `cells_archive_on_close`
at its default `true`, close moves the just-capped cells into `.bee/cells/archive/` before
`build_promotion` scans `.bee/cells/*.json` — so every real close proposed nothing. The happy-path
test passed only because it had been written with archiving disabled, which hid the ordering defect
from the one test built to catch it.

## The rule

- When a plan pins a line number for a new step, treat it as a starting hypothesis, not a locked
  position — check what runs immediately before that line does to the step's own inputs.
- A read-only step has no reason to be sequenced after a mutation unless the mutation is what it is
  reading FOR. If it reads the same store the mutation empties, move it earlier; nothing is lost,
  because the read never writes.
- A test that disables the exact side effect the real path always exercises is testing a world that
  does not occur in production. Run the happy-path test under the DEFAULT configuration, not the
  one that happens to make the assertion pass.

Fixed by `knowledge-loop` D9 (cell `kl-5`, commit `c8d25dff`): the door now computes its proposal
before `auto_archive_on_close` runs, still printing at the same place in close's output.
