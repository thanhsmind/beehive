---
date: 2026-08-04
feature: doctrine-audit
categories: [doctrine-layer, enforcement, tooling]
severity: high
tags: [enforcement-map, advisory-decay, green-washing, delegation, audit-method, platform-parity]
---

# doctrine-audit — a check that cannot fail, and the four ways bee had one

## What Happened

An audit counted ~400 normative lines across the operating block and the
skill references, judged ~85% of them unenforced, and proposed three batches.
Batches A and B had already shipped (`counter-teeth`, `hook-teeth`). This
session ran the rest, and the interesting result is not the prose diet — it
is what turned up underneath it.

Five defects landed, in five lanes:

- **`linux-verify-green`** — the Linux suite was red on the 2.1.8 release
  commit and CI reported success. `ci.yml` piped `cargo test` into `tee`
  without `pipefail`, so the step exited with tee's status. Three tests had
  been failing invisibly; the Windows workflow, which does not pipe, was the
  only honest report. Also: `release-manifest` recorded raw stat mode, which
  Windows synthesises as 0666 and never carries the executable bit, so the
  manifest agreed only with the platform that wrote it — all 205 records read
  as drifted on the other, on both the manifest surface and the
  installed-package proof. Mode now comes from the git index.
- **`doctrine-prose-diet`** — duplication removed, two unbuilt specs moved
  out of references an orchestrator loads on every wave, unenforced craft
  demoted to named defaults.
- **`review-p1-teeth`** — "P1 findings always block" was doctrine with
  nothing behind it. `reviews record` validated the status enum and nothing
  else, so `approved` landed cleanly beside an open P1.
- **`execution-agent`** — `PINNED_AGENT_TYPE` mapped the `generation` tier,
  the tier that implements cells, to `bee-gather`, whose own definition is
  `Read, Grep, Glob` and "never writes". The guard silently *repaired* every
  execution dispatch into an agent contractually forbidden to execute.
- **`worker-proof`** — "cells from `small` up run through dispatched workers,
  never zero execution workers" was stated in the operating block and read by
  nothing at cap.

## The Pattern

All five are the same shape at different depths:

1. **The counter with no refusal** — measured, reported, never blocking (what
   batch A fixed).
2. **The rule with no reader** — stated in doctrine, no code ever looks
   (P1 blocking, the worker registry).
3. **The check whose harness swallows it** — the test runs, goes red, and the
   exit code is discarded (`| tee` with no `pipefail`).
4. **The check whose comparand is not portable** — it can only agree with the
   machine that wrote the baseline (stat mode across platforms).
5. **The enforcer that contradicts the rule** — the worst one. The guard did
   not fail to enforce delegation; it enforced it into an agent that could
   not do the work, and reported the rewrite as a repair. A rule with no
   enforcement decays. A rule whose enforcement is wrong is *believed*.

## Method Failures Worth Keeping

- **A missing filename is not a missing feature.** Two audit items were
  reported open because a grep for `handlers_judge*.rs` and a wave-cap search
  found nothing. Both were built — `validate_judge_verdict` runs at the
  `judge-record` call site, and the four counter flips had shipped in batch A.
  Check the call site, never the file listing.
- **A proxy metric measures the proxy.** The diet's target was ≥40% fewer
  normative lines, measured by counting must/never/always/only. Rewriting
  "never ping mid-flight" as "Default: no routine pings" moves that number
  without changing what a reader does. Real reduction came from two
  relocations (132 lines of unbuilt worktree protocol, ~20 of an unreachable
  cli cell path) and from pointing rules at their enforcers. Final: 438 → 377,
  13.9%. The 40% target was set against an estimate that counted duplicates
  and unbuilt specs as rules, and chasing it would have meant deleting
  doctrine the audit itself marked keep.
- **The audit's own batch order was wrong** because it assumed its groups
  were unbuilt. Verify state before sequencing work against it.

## Rules That Settled

- A doctrine line that no hook, test, or gate reviewer can catch is stated as
  a named default — it reads `Default:` and bends with a recorded reason.
  Boundary rules stay imperative (decision logged 2026-08-04).
- The release manifest records the executable bit as git carries it, and a
  platform that cannot observe that bit does not get to call it changed.
- The `generation` tier carries two rendered agents — `bee-build` writes,
  `bee-gather` reads — so the guard refuses a generic dispatch there instead
  of guessing. Extraction and review still repair.
- A `small`+ cap requires a registered execution worker, or an
  `--inline-reason` recorded on the cap's own trace.

## Open Gaps

- ~~`state set --phase compounding-complete` returns the Node delegate
  marker~~ — **fixed the same day** (`terminal-phase-port`). The door is
  native now: the freshness half it already had, plus the scribing-debt half
  chain-integrity D2 always specified, with two loud escapes — the
  `--waive-scribing-debt` flag, or a logged `capture-deferral` decision, the
  same one `bee close` accepts. All six lanes from this session then closed.
  Worth keeping as the shape: a cutover left a door delegating to a runtime
  that no longer existed, and the symptom was a generic
  `unsupported_argument_shape` — the router could not tell "this verb refuses
  your flags" from "this path was never ported".
- ~~`bee close` refuses a lane feature outright~~ — **fixed the same day**
  (`lane-close`). The delegation guard is gone and the lane record's own
  `last_scribing_run` now joins the scribing-debt threshold. Proven by closing
  its own lane: tests fresh, scribing door clear, cell retired.
- Three delegation gaps closed in one session — the terminal phase, `finish`
  from a worktree, `close` for a lane — all three the same shape: a comment
  that named its own debt, a runtime that was later deleted, and a router
  message that blamed the caller's flags. When a port leaves a `Delegate`
  behind, the debt outlives the runtime silently.
- ~~A worker cannot run `cells finish` from inside its own worktree~~ —
  **fixed the same day** (`worktree-finish`). The refusal was policy, not
  physics: `worktree`, `status`/`orient` and `reservations` already span
  checkouts through the FULL door and `hold_topology()`. `finish` now takes
  the same road — cell and claim at the main store, declared tests with cwd
  in the worktree, holds released as `(main_root, worktree-id)` — while every
  other cells verb stays narrow. Proven by capping its own cell from inside
  the worktree. The un-ported piece the door was waiting on was one function:
  cells' holds ledger keyed at the resolved root, which from a worktree would
  have released nobody's holds while reporting success.
- The concurrency suite goes red only when run under `bee cells finish` or
  `bee worktree merge`, green standalone and on every retry. Four sightings
  in one session, once blocking a merge. Filed P2, cause unknown — and it is
  the worst kind of red, because the honest response to a flaky refusal is
  indistinguishable from working around a real one.
- **Teeth bite their own path.** `wp-1` made a `small`+ cap require a
  registered execution worker; `dispatch prepare --claim` claims and reserves
  but never registers one, so bee's own sanctioned dispatch route now fails
  the door bee added. A guard added at one verb must be walked through every
  path that reaches it. Filed P1.
- `dispatch prepare --claim` already did more than the orchestrator was using
  it for — claim, per-file reservations under the worker's nickname, and the
  cell JSON inlined into the rendered prompt. Three dispatches were hand-rolled
  before anyone read what the tool already emitted. What it genuinely lacked
  was location: neither envelope nor prompt named the worktree or the store
  root, so a worker in a granted worktree could not tell where it was
  (`dispatch-worktree`, fixed). It also handed a cell execution to
  `bee-gather`, the read-only agent — the same defect the model guard carried,
  surviving in the payload builder because the first fix changed only the
  guard's map.
