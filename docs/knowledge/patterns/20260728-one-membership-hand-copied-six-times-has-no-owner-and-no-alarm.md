---
type: bee.pattern
title: One membership hand-copied six times has no owner and no alarm
description: "The same two-element phase set is written out by hand in six modules under three different names, none importing the enum it mirrors and none cross-checked — so the next phase rename silently splits behaviour across the six, and the copy that governs write-denial fails open."
tags: [architecture, enums, drift, fail-open, guards, parity-test, validation-diet]
timestamp: 2026-07-28
bee:
  id: pattern-20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm
  lifecycle: active
  sources: ["validation-diet cell vd-1 (GATED_PHASES + PHASE_GATE retarget, trace .bee/cells/vd-1.json, commit 4a03006e, 2026-07-28)", "validation-diet cell vd-2 (fall-through tail narrowed to !isKnownPhase, commit a7fab75d)", "validation-diet cell vd-4 (real-state-machine conformance test, commit 71729351)", "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs (rust-port equivalent of the retired packages/bee/lib/guards.mjs:142,151 TERMINAL_PHASES copy)", "packages/bee-rs/crates/bee/src/hooks/compaction.rs (rust-port equivalent of the retired packages/bee/lib/compaction.mjs:81; still keeps its own literal TERMINAL_PHASES copy at compaction.rs:74 — this pattern's hazard is not fully closed by the port)", "packages/bee-rs/crates/bee/src/verbs/tmp_group.rs (rust-port equivalent of the retired packages/bee/lib/scratch.mjs:62)", "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs (rust-port equivalent of the retired packages/bee/lib/inject.mjs:235)", "packages/bee-rs/crates/bee/src/verbs/intent_group.rs (rust-port equivalent of the retired packages/bee/lib/intent.mjs:49)", "packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs (rust-port equivalent of the retired packages/bee/lib/recovery.mjs:40)", docs/history/validation-diet/CONTEXT.md D3/D4/D13]
  polarity: pitfall
  critical: true
---

# One membership hand-copied six times has no owner and no alarm

Removing a workflow phase from the enum should have been a one-line change.
It was not, because two separate modules hardcoded that phase name
independently of the enum — `GATED_PHASES` in the write guard and
`PHASE_GATE` in the session-close hook — and **both fell through to allow**
when the name did not match. An incomplete removal would not have thrown; it
would have quietly stopped gating source writes while every existing test
stayed green, because every one of those tests hand-built its own phase
fixture and so never exercised the path the real state machine produces.

What made the cut safe was not the removal. It was forcing one test
(`test_conformance.mjs`) to drive the actual state machine to the
pre-approval state and assert the denial from there — a fixture cannot
disagree with the machine it never consults.

The sweep afterwards found the pattern is far wider than the two constants
that were fixed. The literal set `['idle','compounding-complete']` is written
out by hand in **six** modules under **three** names — `TERMINAL_PHASES` in
`guards.mjs:151`, `compaction.mjs:81`, `scratch.mjs:62`; `NO_WORK_PHASES` in
`inject.mjs:235`, `intent.mjs:49`; `TERMINAL_LANE_PHASES` in
`recovery.mjs:40` — none importing `KNOWN_PHASES`, and no test comparing any
of them to each other. Only the guard's copy is pinned at all. The next phase
change edits some subset, and the divergence has nothing to announce it:
`guards.mjs:1294` governs write-denial, `recovery.mjs:407` governs whether a
finished lane is nagged for resumption, and the rest govern whether
compaction and context injection fire — three different wrong behaviours from
one missed edit.

**Rule.** A membership that mirrors an enum has exactly two safe shapes:
derived from the enum at import, or hand-written **with a parity test that
names every copy**. Three names for one concept is the tell that no module
owns it. When derivation is too large a change to make now — and it often is,
because each copy has its own semantics layered on the shared list — the
cheap intermediate is a single parity suite asserting the copies agree and
naming the offending `file:line` on drift; that is strictly cheaper than the
refactor and catches the same class. And when a hand-copied membership guards
a *deny* decision, its fall-through must fail closed on an unrecognized value
or the copy is not a guard at all — narrow the denial to values genuinely
outside the enum, so legitimate-but-unhandled members keep their existing
behaviour.

See also [[pattern-20260713-a-guard-that-tests-one-state-is-a]]
and [[pattern-20260722-a-coverage-gate-derives-ground-truth-it-never-compares-two-hand-lists]].
