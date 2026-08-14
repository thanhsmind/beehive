---
type: bee.pattern
title: A cell's declared file list is a hypothesis — the worker verifies the real site and corrects the record
description: "Three cells in one day declared implementation files that did not exist or did not own the mechanism (handlers_add.rs vs validate.rs, apply.rs vs source.rs, handlers.rs vs registry.rs); the cell author guesses from names, the worker holds the code. Authoring instructs the worker to follow the real site and name it in the trace — a worker that treats the declared file as ground truth either blocks falsely or edits the wrong home."
tags: [swarming, cells, authoring, worker-contract]
timestamp: 2026-08-11
bee:
  id: pattern-20260811-declared-file-is-a-hypothesis
  lifecycle: active
  polarity: pitfall
  critical: false
  sources: ["close-bookkeeping-p3 cell cbp-2 (declared handlers_add.rs, real site validate.rs normalize_new_cell)", "onboard-root-resolution cell orr-1 (declared apply.rs, real site source.rs Engine::locate)", "worktree-store-hygiene cell wsh-1 (declared handlers.rs, real site registry.rs bootstrap_worktree_store)", "traceable-runs cell trun-2 (action text pointed at default_gates() in state_group/policy.rs; real seam was default_gate_entry() in workflow_store/record.rs, already landed by trun-1 — the worker re-read the prior cell's own note against live code before implementing, left the wrong file untouched, and proved the real seam with a test instead)", "traceable-runs cell trun-9 (action text said to enqueue a scribe record inside cells/handlers_close.rs::run_cap, outside the cell's declared files; the worker moved the trigger to drivers/close.rs::scribing_debt, in scope, and named the deviation rather than widening file scope silently)"]
---

# A cell's declared file list is a hypothesis

The cell author writes `files:` from module names and precedent, without
holding the code; the executing worker holds the code. In one day, three
cells declared files that were wrong in three different ways: a file that
does not exist in the codebase at all (`handlers_add.rs`), a sibling that
does exist but does not own the mechanism (`apply.rs` — the resolution
lived in `source.rs`), and a display-layer file when the write path lived
in the registry (`handlers.rs` vs `registry.rs`).

## The rule

- The author states the declared file as a starting hypothesis and says
  so in the action ("the declared file is a guess — follow the real
  site, name it in your report").
- The worker greps for the mechanism, follows the real site, and NAMES
  the correction in its report and trace — never silently edits
  elsewhere, never blocks solely because the declared path is absent.
- The capper records the corrected file list on the cell (`--files` at
  finish carries the real paths).

## Why it recurs

Promote-proposal mining reads only structured trace fields
(`trace.deviations`, failure signatures); a correction narrated in the
worker's prose report never reaches them, so the pipeline reports "0
pattern candidates" for exactly this class. Until deviations are recorded
structurally at finish (filed as friction), this pattern is invisible to
the automatic loop — which is why it is written here by hand.
