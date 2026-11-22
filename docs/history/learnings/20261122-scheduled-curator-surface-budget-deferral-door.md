---
date: 2026-11-22
feature: scheduled-curator
categories: [pattern, failure]
severity: standard
tags: [registry, preamble, plan-amendment, doc-deferral, judge]
---

# Learning: A cell that adds registry verbs also owns the preamble budget raise

**Category:** pattern
**Severity:** standard
**Tags:** [registry, preamble, plan-amendment]
**Applicable-when:** a plan cell adds commands to `registry_payload.json`

## What Happened

scur-2 declared five work-queue verbs. Its verify ran `registry_contracts`
and `registry_dispatch` only, so it went green. The full suite at the end
of the feature failed one test: the preamble command surface was 2947
characters against a 2900 budget. The fix took a new cell (scur-5), and the
plan was amended twice. The first amendment added scur-5. The second made
the tester cell scur-4 depend on it, because `cells add` warned that scur-5
had no dependent tester.

## Root Cause

The budget test lives in the `session_preamble` tests, which no registry
cell verify names. The same red happened in earlier features: decisions
da8592de (cell-question-park), aa45854c (workspace-protocol), and the
budget raises in commits d909a2335 and efc1408fe.

## Recommendation for Future Work

A cell that adds registry commands adds
`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- session_preamble`
to its verify and raises `COMMAND_SURFACE_BUDGET_CHARS` in the same cell
when the test fails. When a plan amendment adds a code cell, add the tester
dependency in the same amendment.

Waste prevented: one cell, two gate cycles (unapprove, plan-rev bump,
conflict verdicts for 51 candidates, preview, re-approve) per feature.

<!-- bee:not-a-deferral: this learning documents the deferral detector itself -->

# Learning: The verb name deferred-queue trips the deferral guards

**Category:** failure
**Severity:** standard
**Tags:** [doc-deferral, decisions]
**Applicable-when:** prose or a decision names the work-queue verb

## What Happened

`bee decisions log` refused a decision because its text named the verb.
At close, the doc-deferral door blocked on five lines in CONTEXT.md,
test-report.md and knowledge.md. Every one named the command, and none
promised future work. Each line needed a `bee:not-a-deferral` block.

## Root Cause

The deferral detector matches the substring "defer" and does not skip
backticked command names.

## Recommendation for Future Work

While the detector matches command tokens, write "the work-queue verbs" in
prose and wrap any line that must spell the verb.

<!-- /bee:not-a-deferral -->

# Learning: A judge brief must omit failure_signature on PASS

**Category:** failure
**Severity:** standard
**Tags:** [judge]
**Applicable-when:** writing a slice-judge brief

## What Happened

The brief asked for `failure_signature` as an empty string on PASS.
`bee cells judge-record` refused the verdict. The leader removed the key
and recorded the verdict again.

## Recommendation for Future Work

Tell the judge to omit `failure_signature` when the verdict is PASS.
