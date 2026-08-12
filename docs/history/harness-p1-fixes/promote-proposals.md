promote proposal for work item "harness-p1-fixes" (.bee/logs/scribing-runs.jsonl + .bee/lanes/harness-p1-fixes.json) — 3 capped cell(s): hpf-1, hpf-2, hpf-3
anchor: ledger — .bee/logs/scribing-runs.jsonl, .bee/lanes/harness-p1-fixes.json
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/harness-p1-fixes/delivery.md

---
type: bee.delivery
title: harness-p1-fixes — delivery
description: "Delivery record proposed by bee knowledge promote for work item harness-p1-fixes: 3 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-11
bee:
  id: harness-p1-fixes-delivery
  lifecycle: active
  areas: [workflow-state]
  required_context: [.bee/logs/scribing-runs.jsonl, .bee/lanes/harness-p1-fixes.json]
  sources: [.bee/logs/scribing-runs.jsonl, .bee/lanes/harness-p1-fixes.json, .bee/cells/hpf-1.json, .bee/cells/hpf-2.json, .bee/cells/hpf-3.json]
---

# harness-p1-fixes — Delivery

## What shipped

- **hpf-1** — Judge-debt door owns its route (never reads another feature's state route), grandfathers cells capped before 2026-08-11, offers a judge-deferral decision escape, and names unarchive before judge-record for an archived offender (3 file(s) changed)
- **hpf-2** — Scoped dispatch wave to one resolved feature (refusing when none resolves), added --limit to bound the claimed batch, and unwound a mid-reserve claim leak with its own unwind_failed skip reason. (4 file(s) changed)
- **hpf-3** — Fixed: dispatch wave never force-unclaims a claim it never took, and its unwind now clears the worker row too. (3 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpf-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **hpf-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`
- **hpf-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work harness-p1-fixes` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/logs/scribing-runs.jsonl`, `.bee/lanes/harness-p1-fixes.json`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "harness-p1-fixes" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-12T00:00:23.123Z), the work item declares no bee.areas.

area workflow-state:
  - [hpf-1] Judge-debt door owns its route (never reads another feature's state route), grandfathers cells capped before 2026-08-11, offers a judge-deferral decision escape, and names unarchive before judge-record for an archived offender — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hpf-1.json)
  - [hpf-2] Scoped dispatch wave to one resolved feature (refusing when none resolves), added --limit to bound the claimed batch, and unwound a mid-reserve claim leak with its own unwind_failed skip reason. — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/hpf-2.json)
  - [hpf-3] Fixed: dispatch wave never force-unclaims a claim it never took, and its unwind now clears the worker row too. — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/hpf-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

None: no capped cell trace carries a deviation or a failure signature.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 0 pattern candidate(s), 0 file(s) written.