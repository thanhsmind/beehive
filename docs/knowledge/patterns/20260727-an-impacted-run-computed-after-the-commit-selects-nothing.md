---
type: bee.pattern
title: "An impacted-test run computed after the commit selects nothing, and caps false-green"
description: "A worker committed its change, then ran run_verify.mjs --impacted-from-git, which diffed against the now-clean tree and saw only leftover uncommitted bookkeeping files — reporting 0 suites and capping the cell verify_passed:true while the change was in fact red."
tags: [verification, false-green, impacted-tests, git, ordering, state-phase-lock-race]
timestamp: 2026-07-27
bee:
  id: pattern-20260727-an-impacted-run-after-commit-selects-nothing
  lifecycle: active
  sources: ["state-phase-lock-race cell splr-1 (verify_passed: true capped on IMPACTED RUN: 0 suite(s) from 5 changed file(s); trace .bee/cells/splr-1.json, commit e787819a, 2026-07-27)", docs/history/state-phase-lock-race/CONTEXT.md, decision f7de0f50-c59a-4455-8699-e7c183fca2b6 (D12 correction), "orchestrator's own full run_verify (109 suites) catching test_cli_state.mjs red after the cap"]
  polarity: pitfall
  critical: false
---

# An impacted-test run computed after the commit selects nothing, and caps false-green

`splr-1`'s worker made its change, `git commit`ted it, and only then ran
`node scripts/run_verify.mjs --impacted-from-git` as its capping evidence.
`--impacted-from-git` derives its suite selection from the working tree's
current diff against a base — and once the commit lands, the diff that
selection is computed against is whatever is *still* uncommitted, not the
change just made. Here that was five leftover bookkeeping files unrelated to
the fix. The run reported `IMPACTED RUN: 0 suite(s) from 5 changed file(s)`
— a real, well-formed pass over an empty selection — and the cell capped with
`verify_passed: true`. The change was in fact red on `test_cli_state.mjs`
(the blanket-lock regression — see the companion pattern on scoping locks to
the wrong record). Nothing caught it until the orchestrator's own full
109-suite run.

**Rule.** `--impacted-from-git`-style selection is a snapshot of the working
tree at the moment it runs, not of "the change this cell made." Run it
**before** committing (against the real uncommitted diff), or pass an
explicit base/range that names the commit — never run it after the commit
and trust an empty selection. More generally: **a suite count of zero from an
impacted-selection run is not evidence of a passing change — it is evidence
of nothing, and a behavior_change cell has no proof at all until the
selection is confirmed non-empty** (or the run is the full suite). A verify
line that says "0 suites, PASS" reads identically to a genuine no-op cell and
to this failure mode; only inspecting the changed-file list against what the
cell actually touched tells them apart.

See also
[[pattern-20260724-canonical-source-tests-cannot-see-vendoring-drift]]: both
patterns are the same shape of failure — a green run that never actually
exercised the code path it was meant to prove, because the run's *selection*
(which files, which copy) silently diverged from the change under test. There,
the canonical source resolved an import the vendored copy could not; here,
the post-commit diff resolved to bookkeeping noise instead of the real
change. Also related:
[[pattern-20260723-a-scan-scope-set-from-assumption-passes-green-while-hiding-the-bug]]
— a scope computed from assumption rather than measurement is the general
case this defect instantiates for test selection specifically.
