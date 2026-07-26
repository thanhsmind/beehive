---
type: bee.pattern
title: A red-first proof whose oracle can be fed by live-environment detection proves nothing about the code under test
description: "Reverting a version-pin constant stayed green locally because the machine's live CLI version equals the new pin — the assertion string was satisfied by live detection, not by the constant. A local red-first that can be fed by the environment is not a red floor; require an environment-independent proof."
tags: [red-first, verification, oracle, environment-coupling]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-red-first-oracle-fed-by-live-environment
  lifecycle: active
  sources: [i54-closeout-8 (PROBED_CODEX_VERSION bump; local reversion re-test green because live codex is 0.145.0; JUDGE_STANDARD_INSUFFICIENT flagged via deliberate_exceptions; external canary rerun substituted as proof), docs/history/i54-closeout/reports/i54-closeout-8.md]
  polarity: pitfall
  critical: true
---

# A red-first proof whose oracle can be fed by live-environment detection proves nothing about the code under test

Before accepting a reversion re-test as the red floor, ask: can anything OTHER
than the code path under test satisfy this assertion? If the oracle string can
be produced by live-environment detection (a probed version, a live API reply,
a machine-local default that happens to match the new value), the local red
run is environmentally inconclusive — a genuinely broken change would still
show green on this machine and only fail elsewhere (CI, a machine on another
version). The honest close is to say so loudly (deliberate_exceptions), and to
substitute an environment-independent proof — here, the external canary run
against an isolated fixture.

**The tell:** the re-test's expected string contains a value the environment
can also produce (a version number, a hostname, a live-probed capability).
