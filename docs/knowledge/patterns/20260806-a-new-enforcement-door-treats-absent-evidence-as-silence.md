---
type: bee.pattern
title: "A new enforcement door treats absent evidence as silence, not as violation"
description: "Every door hook-teeth added refuses only on evidence that something is wrong, and warns or exempts when the evidence it reads is missing — an unreadable test record, a feature no record names, a session with no recorded start manner, and a unit that changed no files all proceed, because absence is ignorance and refusing on it converts a new guard into a work stoppage."
tags: [enforcement, guards, refusals, fail-open, prose-to-mechanism]
timestamp: 2026-08-06
bee:
  id: pattern-20260806-a-new-enforcement-door-treats-absent-evidence-as-silence
  lifecycle: active
  areas: [workflow-state, hook-runtime]
  decisions: [e1e41ec8 (prose-rule-audit batch B — six prose rules gain mechanical enforcement), "hook-teeth D7 (a test proving the condition computes correctly lands before the refusal wires in, red-first, same cell; no flip ships with a known false positive)"]
  sources: ["hook-teeth cells bh-1..bh-6 (traces in .bee/cells/, docs/history/hook-teeth/CONTEXT.md, 2026-08-04 — full suite 1058 passed, 0 failed)", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:665-720 (RedBaseStatus::Unknown warns instead of refusing)", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:104-156 (a feature neither the lane nor the default record names is never refused)", "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs:347-366 (a session with no recorded start manner warns and proceeds)", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:219 (a unit with no recorded file changes never reaches the commit-trailer scan)", "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:174-206 (a default record naming another feature is no opinion, never approval)"]
  polarity: practice
  critical: false
---

# A new enforcement door treats absent evidence as silence, not as violation

A guard reads some record to decide whether to refuse. That record can be missing, empty, or in a
shape the guard does not recognise — and the decision of what to do then is the difference between
a guard that catches a real mistake and a guard that stops all work in a repository that simply has
no history yet.

The six doors added by `hook-teeth` all answer it the same way: refuse on evidence of a violation,
never on the absence of evidence.

- The red-base claim door reads the last recorded test run. A record that is missing, or not in a
  recognised shape, classifies as *unknown*: the claim proceeds and the unknown colour is warned
  about. Only a record that says **red** refuses.
- The gated-authoring door reads the feature's lane record, then the default record. A feature
  neither record names is not refused — first authoring in a fresh repository stays open.
- The plan-freeze guard reads the shape gate of the feature named in the path. A default record
  about some *other* feature is treated as no opinion, never as approval in either direction.
- The handoff-adoption door reads the session's recorded start manner. A record predating the
  field, or one that never captured it, warns and proceeds.
- The commit-trailer door scans for a commit naming the unit — but only for a unit that recorded
  changed files. A unit with no file changes is exempt, because there is nothing for a commit to
  carry.

## The rule

- Separate the three states the guard's own input can be in: **violation**, **compliance**, and
  **unknown**. Write the unknown branch deliberately; a two-state guard silently folds unknown into
  one of the other two, and which one it lands in is an accident of how the code was written.
- Refuse on violation. Warn on unknown, on the channel the operator already reads — a silent
  unknown branch is how a guard that never fires gets mistaken for a guard that passes.
- State the exemption in the same breath as the rule when the subject genuinely cannot carry the
  evidence (a unit that changed no files cannot have a commit that names it). An exemption that is
  derived from the subject's own shape needs no escape hatch.
- Give the violation branch exactly one escape, and make it a *declaration* stored on the record —
  a fix-first reason, a commit-pending reason. An escape that leaves no trace is a hole; an escape
  that writes down why is an audit trail.
- Land the test that proves the condition computes correctly before the refusal wires in, in the
  same unit of work (hook-teeth D7). The false positive you never see is the one that ships.
