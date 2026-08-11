---
type: bee.pattern
title: "Fix the fan-out, not the one cell of it that was reported"
description: "A rule stated over a grid gets patched one cell at a time: three fixes to one dispatch-label rule each touched a single runtime-kind-transport combination, and the untouched combinations never failed — they resurfaced weeks later as a screenshot."
tags: [fan-out, scope, recurrence, enforcement, dispatch]
timestamp: 2026-08-07
bee:
  id: pattern-20260807-fix-the-fan-out-not-the-reported-cell
  lifecycle: active
  areas: [advisor-protocol, hook-runtime]
  decisions: ["4439bd7e (work-visibility D2): every dispatch description is one work-language intent sentence, 2026-07-24", "dispatch-label-chokepoint Gate 2: close the grid and add the device that makes an uncovered combination unshippable, 2026-08-07"]
  sources: ["bee 2.2.6: kind==cell on the claude Agent transport was fixed and a written rationale shipped for leaving the rest", "audit 2026-08-07: four gaps — codex task_name was the bare cell id (prepare.rs:687), claude gather/reviewer/advisor rendered kind-plus-model, cli-exec carried no label field, and the guard read the label and only logged it (model_guard.rs:732)", "cells dlc-1 and dlc-2: full suite 1350 passed, 3 ignored"]
  polarity: pitfall
  critical: true
  evidence: exercised
  evidence_ref: "packages/bee-rs/crates/bee/src/hooks/model_guard.rs (repair_dispatch_label runs once above the transport fan-out; tests bare_id_label_on_a_cell_dispatch_is_rewritten_to_carry_the_title, codex_spawn_label_repair_targets_task_name_from_message)"
  signature: fix-scoped-narrower-than-the-rule-it-implements
---

# Fix the fan-out, not the one cell of it that was reported

A rule is stated once — *a dispatch label says what the work is*. The surface it
governs is a grid: runtimes × kinds × transports. A report arrives from one
square of that grid. The fix lands in that square. Everything else in the grid
stays wrong, and stays quiet, because an unlabelled dispatch is not a failure —
it is a row someone has to look at before anyone notices.

Three rounds of this on one rule. The doctrine landed first and produced
nothing, because no code implemented it. Then code implemented it for one kind
on one transport, with a written rationale for leaving the others — a rationale
that read plausibly and was wrong, because it assumed a caller could supply what
the neighbouring squares needed when no mechanism existed for the caller to
supply anything. The screenshot that reopened it came from the square nobody had
touched.

The tell, in hindsight: the fix was written **inside** a branch of the very
match that fans out. Anything computed inside one arm is, by construction,
absent from the others.

## The rule

- Before fixing, name the grid. What are the dimensions the rule ranges over —
  runtimes, transports, entry points, callers, platforms? A fix scoped narrower
  than the rule is a scheduled recurrence, not a fix.
- Compute the shared thing **once, above the fan-out**, and let every branch
  read it. Then a new branch inherits correctness instead of having to remember
  it.
- Put the enforcement at the chokepoint every path crosses, not at the source
  each path happens to use. Sources can be bypassed — a hand-written call
  reaches the runtime without ever touching the builder — while the chokepoint
  cannot.
- Make an uncovered square **unshippable**: enumerate the grid from the
  constants that define it, never from a hand-written list, and assert the
  property for every pair. A list copied by hand is itself a square someone must
  remember to update.
- A square that genuinely cannot satisfy the rule is a **recorded limit** in the
  code and in the spec, never a silent omission. The next reader must be able to
  tell "impossible here" from "nobody got to it".
- Treat a written rationale for leaving part of a grid unfixed as the smell it
  is. Read it again in a month; that is when it stops being convincing.
