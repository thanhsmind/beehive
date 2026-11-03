---
type: bee.learnings
title: "Verification in the flow — learnings"
description: "Harvest from verification-in-the-flow, verification-contract-parity, and the first bee-verify-upkeep run, 2026-09-02."
timestamp: 2026-09-02
bee:
  id: learnings-20260902-verification-in-the-flow
  lifecycle: active
  areas: [verify-pipeline, doctrine-layer, onboarding]
  sources: [docs/history/verification-in-the-flow/CONTEXT.md, docs/history/verification-in-the-flow/plan.md, docs/history/verification-contract-parity/CONTEXT.md, ".bee/mailbox job reports from the first bee-verify-upkeep run"]
---

# Verification in the flow — learnings

Three pieces of work in one session: a feature that put the project-local
verification skill into bee's own flow, a follow-up that built the parity
tests the first one declared and never wrote, and the first ever run of
`bee-verify-upkeep`.

## What the guards caught that the humans and the plan did not

Three refusals landed during this work, and every one of them was right
for a better reason than the rule it enforced.

**The prune rule caught a name that would have deleted itself.** The
proposed skill name was `bee-verify-app`. `onboard/skills.rs` proves the
`bee-*` sync emits `remove_skill` for any bee-named directory absent from
bee's own source, so a host's generated skill under that name would have
been wiped at the next `onboard --apply`. The prefix, not the fixed name,
was the defect: `verify-app` keeps every benefit and survives.

**The claim door caught a test that would have fought its own decision.**
A cell proposed asserting a doctrine line is present. That line's
decision carried a registered falsifier — revert it if two features do
not use it. `CONTRACT_UNSETTLED` refused the citation. Promoted as
[[pattern-20260902-pin-a-settled-contract-with-a-test-never-a-provisional-one]].

**The worktree-first guard caught planning written into the wrong
store.** `bee worktree new` bootstraps a worktree with its OWN state
record; every route, gate and cell written from main afterwards went to
main's store, and the worktree saw none of it. The control plane lives in
main by design — the mistake was doing feature planning before entering
the worktree, not doing it in main.

## What the plan got wrong, and how it was caught

The plan's own claims table carried a citation to the wrong file, three
drifted line numbers, and three load-bearing claims that lived only in
prose. A five-seat review pass found all of them before the gate.

More usefully, that pass killed a locked decision. The decision to flatten
the source path changed no observable behavior — the fixed name already
gave one constant path — while rewriting a deliberately commented design,
removing the ability to hold a second host skill, and stranding a phantom
skill directory under any older binary. Five seats reached that
independently. Retiring it deleted the plan's only HIGH-risk component.

**The lesson is about where the review sat, not that it happened.** It ran
at the plan step, on a draft, before any cell existed. The same finding
after execution would have cost five cells of rework.

## What no door caught

The feature closed with two rows of its own test matrix unbuilt. Every
close door passed: they check that capped cells carry proof lines, never
that a plan's declared matrix was exhausted. The gap surfaced only because
someone asked whether the work was finished.

A plan's test matrix is a promise with no enforcement behind it. Treat it
as a checklist to walk by hand at close, or accept that it is decoration.

## What the first upkeep run found

Four of five feature files carried drift. The sharpest: the map instructed
a driver to cap with a bare `green` result — a form the CLI now refuses.
Nobody had noticed because nobody had run the map. One of its stale line
citations had been moved by the very feature shipped that same week.

Recorded as a recurrence on
[[pattern-20260821-instruction-text-is-an-untested-code-path]], with the
escalation it asks for: where a claim is about a constant or a rendered
surface a text test owns it; where the claim is a RECIPE only running it
can, so the durable owner is a cadence, not a check.

The run also surfaced a product gap kept out of the doc fix: `bee worktree
merge --help` says cleanup runs by default; the code says the opposite and
its own comment reads "KEEP by default". The help text is wrong, and it
had already misled this session into passing `--no-cleanup` twice.

## Carried forward

- Two open triggers, both deliberate: a host needing a second verification
  skill, and the falsifier on the read-first rule.
- The feature map is a state layer now, but its freshness depends on a
  human-invoked skill. Nothing schedules it; the onboard notice tells an
  agent it exists, and the agent cannot run it.
