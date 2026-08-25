---
type: bee.pattern
title: A flake is a defect in the test's mechanism, not noise to retry past
description: A flake is a defect in the test's mechanism, not noise to retry past
tags: [failure, tests, concurrency, verification]
timestamp: 2026-08-25
bee:
  id: pattern-20260825-a-flake-is-a-defect-in-the-tests-mechanism
  lifecycle: active
  areas: [rust-runtime, hook-runtime]
  sources: ["doctor-probe-honesty cell dph-1 (the product bug the flake was truthfully reporting, 2026-08-25)", "test-exec-race cell ter-1 (the mechanism fix, commit acc479c9, merged 88c0b7f)", "docs/history/doctor-probe-honesty/CONTEXT.md (the reproduction and the five-row matrix)"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "before deciding a red test is noise, ask what it asserted and whether the product could actually produce that value — a hermetic test that fails is reporting something real until proven otherwise"
---

# A flake is a defect in the test's mechanism, not noise to retry past

A full-suite run went red on one doctor test. It passed on rerun, passed alone, passed on
main. Every signal said "flake — move on".

Reading it instead found two separate real bugs.

**The product bug it was reporting.** The test asserted that a version mismatch must not pass.
It got `Some(true)`. That value could only come from an empty `ProbedBeeVersion::Failed` arm
falling through to a row announcing *"installed binary matches source (version X)"* — a claim
about a version nothing had read. The one check whose job is to notice a stale installed binary
reported freshness precisely when it had learned nothing. Re-running until green would have
buried it.

**The mechanism bug that made it intermittent.** The test helper wrote its fake binary straight
to the path it was about to execute. `cargo test` runs on many threads; while one thread holds
that path open for writing, another thread's `Command::spawn` forks, the child inherits the
write fd, and Linux refuses the exec with `ETXTBSY`. `O_CLOEXEC` does not save you — it closes
the fd at exec, which is already after the fork.

Both fixes were small. Neither was reachable without taking the red run seriously.

**The rule:** a hermetic test that fails is reporting something real until proven otherwise.
Before calling a red run noise, ask what the test asserted and whether the product could
legitimately produce the value it saw. "It passed on rerun" answers neither question.

**Fix the mechanism, never the symptom.** The remedies that make a flake quiet — a retry loop, a
sleep, a loosened assertion, `#[ignore]` — all leave the race and remove the detector. Here the
fix was to install the executable by `rename` from a sibling temp path, so the executed path is
never open for writing by anyone and the `ETXTBSY` precondition cannot occur. That is a
structural argument; three consecutive green suites are supporting evidence, not proof, and an
intermittent bug is never proven gone by a run count.

**The cost of getting this wrong is asymmetric.** A retry would have cost one line and looked
responsible. It would also have deleted the only thing in the system that had noticed a real
defect in a health check — and a health check that lies is worse than no health check, because
it is trusted.
