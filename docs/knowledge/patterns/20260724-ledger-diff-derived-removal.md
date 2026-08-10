---
type: bee.pattern
title: "A managed-file ledger needs a removal path derived from its own diff, not a hand-maintained retired list"
description: "A managed-file ledger needs a removal path derived from its own diff, not a hand-maintained retired list"
tags: [onboarding, drift-detection, self-derived-removal, ledger]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-ledger-diff-derived-removal
  lifecycle: active
  sources: ["installer-verify-orphan-drift-1 (bee.mjs status correctly flagged .bee/bin/lib/herding.mjs as orphaned drift; onboard_bee.mjs had no removal path for it -- only helpers had one, via a hand-maintained RETIRED_HELPERS list)", "docs/knowledge/areas/onboarding/release-identity-and-version-parity.md#R27"]
  polarity: pitfall
  critical: false
---

# A managed-file ledger needs a removal path derived from its own diff, not a hand-maintained retired list

When a fingerprint ledger tracks a set of vendored files (helpers, library
modules, hooks — anything a status check compares "recorded" against "on
disk"), the set can only ever grow safely if every retirement gets its own
removal step. Helper scripts already had one (a hand-maintained retired-name
list, checked on every apply). Library modules did not: the plan builder only
ever iterated the CURRENT source directory to decide what to copy, so a name
that disappeared from source simply stopped appearing in that loop — nothing
ever produced a removal item for the copy already on disk. The result: an
already-onboarded host kept the orphaned file forever, the drift check
correctly flagged it as unrecognized on every run, and a top-level installer's
final "no drift" verification failed permanently, with no apply able to ever
clear it.

The fix generalizes past a second hand-maintained list (which just moves the
"remember to add an entry" burden one file over): removal is derived from the
**ledger diff** — the previous recorded key set minus the current desired set.
Any name that drops out gets removed, self-deriving forever; no future
retirement needs its own entry.

**The tell:** a status/drift check that already reports "an installed file the
ledger doesn't recognize" as a defect, with no corresponding write path that
would ever clear that exact defect. A report-only check paired with no fix is
a permanent false positive waiting to ship — check next to every such flag
whether the removal that would clear it actually exists.
