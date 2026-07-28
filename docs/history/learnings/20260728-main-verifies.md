---
date: 2026-07-28
feature: main-verifies
categories: [orchestration, workflow-state, tests]
severity: high
tags: [feature-verify, philosophy, cap-law, zombie-workflow]
---

# main-verifies — feature close learnings

## What Happened

Installed the user's verification philosophy end-to-end (R82): workers
implement + commit + report — no suites; MAIN produces all proof — bugfix
repro red pre-dispatch, and ONE feature-level verify at the shippable
boundary, recorded via the new `state feature-verify record` verb (command +
output sha + result). `cells cap --feature-verify-pending` caps evidence-free
with a trace marker; `guardFeatureVerifyDebt` refuses leaving swarming while
pending cells lack a fresh green record — typed, and immune to every
gate_bypass level including total (hand-verified and test-asserted). Classic
per-cell path survives for transition/spot use. Doctrine flipped across
executing/swarming/routing-and-contracts; both touched bodies net-shrank
(executing 10221→9227, swarming 8175→8172). The feature itself closed under
the old law; its close dogfooded the new verb with the first recorded green.

## Findings

1. **Drift #5 exposed the deeper lifecycle hole.** fx-1's close-on-start fix
   worked when a workflow record existed (explicit-triage closed correctly)
   but several features never got records at all, and status-diet's survived
   active — root partially found by mv-4: `createWorkflow` never spread
   `baseWorkflowDefaults`, so record shape (and by extension lifecycle
   consistency) depended on the caller. mv-4 fixed the symmetry at
   construction; the record-creation coverage gap (which startFeature paths
   create records) remains open as the P1 friction. *Rule: a store's create
   path spreads its own defaults — shape by construction, never by reader
   synthesis.*
2. **A mid-flight law change needs an explicit override for in-flight cells.**
   mv-3 ran after mv-2 flipped the executing skill; its dispatch carried a
   sanctioned classic-loop override. *Rule: when a feature changes the law its
   own workers run under, every subsequent dispatch states which law governs
   that cell, explicitly.*
3. **Serial-when-named-conflict worked as designed:** the scheduler serialized
   mv-1→mv-2 on a genuinely shared file (test_misc census duty) while mv-4
   ran parallel with mv-2 on disjoint sets — both shapes in one wave, zero
   coordination failures.

## The day's arc (for the record)

Four philosophy-level changes shipped today, each mechanically enforced:
parallel-by-default (wave-barrier regen), explicit triage (route record),
brief worker orientation (status --brief), and feature-level verification
(this feature). Combined shape: main routes with a recorded route, dispatches
parallel waves of implement-only workers with embedded state, pays one
barrier, runs one recorded feature verify, and ships.
