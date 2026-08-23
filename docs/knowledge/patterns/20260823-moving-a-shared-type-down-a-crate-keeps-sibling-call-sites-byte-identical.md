---
type: bee.pattern
title: "Moving a shared type down a crate picks the form that keeps an in-flight sibling's call sites byte-identical"
description: "In a parallel wave, one cell moved the screen classifier into a lower crate and re-exported it under an alias; a sibling cell editing call sites of the old path at the same moment broke at merge. The move is a cross-cutting change inside a wave: keep the old path valid (a re-export at the old name, same signatures) until every in-flight sibling has landed, then retire it in its own cell."
tags: [swarming, parallel, refactor, rust, crates]
timestamp: 2026-08-23
bee:
  id: pattern-20260823-moving-a-shared-type-down-a-crate-keeps-sibling-call-sites-byte-identical
  lifecycle: active
  sources: ["tmux-herding-cockpit cell thc-2 (the shared screen classifier moved from the bee crate into fleet; the sibling cell's call sites broke on an alias until the old path was kept as a re-export)"]
  polarity: pitfall
  critical: false
---

# Moving a shared type down a crate keeps sibling call sites byte-identical

A wave runs cells in parallel on disjoint files. A type move is not disjoint:
it changes what every file that names the type compiles against, including
files a sibling cell is editing right now. The merge of two green branches
then goes red, and neither worker saw it.

When a move has to happen mid-wave, pick the form that leaves every existing
call site byte-identical — a re-export at the old path with the same names
and signatures — and retire the old path in a later cell once no sibling is
in flight. A rename, an alias with a different name, or a changed signature
is a cross-cutting change and needs the wave to serialize around it, named
as such in the plan.
