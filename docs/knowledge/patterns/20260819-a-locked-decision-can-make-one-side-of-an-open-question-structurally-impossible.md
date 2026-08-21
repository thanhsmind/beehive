---
type: bee.pattern
title: A locked decision can make one side of an open question structurally impossible
description: "When a discovery ticket presents two options, an already-locked decision may have made one of them structurally impossible, and answering the ticket as written then produces a decision that only restates the constraint — two framing corrections of this shape were the whole value of a ten-ticket discovery map: 'core or bee records results' was no choice at all under the genericness boundary, and 'Rust API or recipe file' hid the real fork of wave-as-sequence versus wave-as-value."
timestamp: 2026-08-19
bee:
  id: pattern-20260819-locked-decision-collapses-an-open-question
  lifecycle: active
  areas: [discovery-wayfinding]
  sources: ["capture stub 85d648da (herding-orchestration: two framing corrections carried a ten-ticket map)", docs/discovery/herding-orchestration/MAP.md]
---

Two framing corrections were the whole value of a ten-ticket discovery
map, and both were cases where the obvious binary was the wrong
question. First: "does the core record results, or does bee?" is not a
choice at all — the crate boundary drawn for genericness makes it
structural, so the only live question is WHAT bee writes. The answer
was one append-only wave ledger, chosen because it REPLACES
pane-counting as the occupancy source and closes a recorded over-spawn
gap, not because tracing is nice to have.

Second: "Rust API or declarative recipe file?" is not the fork either.
The real fork is whether a wave is a call SEQUENCE or a VALUE. As a
value, a file format later is serde over types that already exist and
costs nearly nothing; as a call sequence, extracting it later means
designing it twice. A corollary fell out of the same reframe: the
failure policy (wait-all / first-success-cancel-rest / best-effort) is
the axis that actually varies between scenarios, so it belongs in the
value as an enum from day one, even with a single variant implemented.

**The rule:** when a discovery ticket presents two options, check
FIRST whether an already-locked decision has made one of them
structurally impossible. If so, the real question is one level down —
answering the ticket as written produces a decision that only restates
the constraint. Read the locked record, name the collapsed side in one
line, and rewrite the ticket to the question that is still live.
