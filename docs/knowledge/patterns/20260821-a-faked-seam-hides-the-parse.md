---
type: bee.pattern
title: A faked seam hides the parse
description: "A trait seam that every test fakes proves the seam is exercised, never that the parse behind it is right; a fake returns whatever the test configured, so the one thing only a real process reply can falsify — the extraction of a live response — stays unchecked while the suite is green."
tags: [testing, trait-seam, parsing, external-tool, proof-discipline]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-a-faked-seam-hides-the-parse
  lifecycle: active
  areas: [herding, rust-runtime]
  decisions: ["9391e9e8 (herding-prompt-stall D1, 2026-08-21: the RealHerdr misread this pattern generalizes from; narrowed within the same feature by D4 — the receipt is the worker's ack file — and by D6 — a stalled submission is retried, not fatal. Neither narrowing touches this pattern, which is about the parse behind a faked seam, not about what counts as a receipt.)"]
  sources: ["packages/bee-rs/crates/bee/src/herding/run.rs (RealHerdr::agent_wait, extract_agent_wait_status)", "live 2026-08-21: 2261 unit tests green, every live bee herding run died at the readiness gate"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "packages/bee-rs/crates/bee/src/herding/run.rs (extract_agent_wait_status, pinned to a captured live reply in extract_agent_wait_status_reads_the_captured_live_reply)"
  signature: faked-trait-seam-hides-real-parse
---

# A faked seam hides the parse

A trait seam that every test fakes hides the one thing only production runs:
the parse of the real reply. A fake returns whatever the test configured, so
the seam is exercised on every run — but the extraction behind the real
implementation, the part that turns an external tool's actual response into
a value bee acts on, is never touched by any test that goes through the
fake.

The instance: two new calls to an external tool went in behind an existing
trait whose fake returns canned values. 2,261 unit tests stayed green while
the real extraction read the wrong path out of the tool's JSON reply and
returned nothing for every live reply, so every dispatch through that path
died at the readiness gate. Nothing in the suite could have caught it — the
seam was exercised, the parse never was.

This is the same family as [source that ships without reinstalling the
binary the hooks call is
inert](20260805-source-that-ships-without-reinstalling-the-binary-the-hooks-call-is-inert.md):
in both, the thing under test and the thing that runs are not the same
thing.

## The check

When a change adds a call to an external tool, capture one real reply and
pin the extraction to that captured text in a unit test — write the
extraction as a pure function over a string so the test needs no live
process. A green suite over a faked seam is not evidence that the path
crossing the real process boundary works; only a test against a captured
real reply is.
