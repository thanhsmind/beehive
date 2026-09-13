promote proposal for work item "codex-reliability-closeout" (docs/history/codex-reliability-closeout/CONTEXT.md + docs/history/codex-reliability-closeout/plan.md) — 5 capped cell(s): crc-1, crc-2, crc-3, crc-4, crc-5
anchor: history — docs/history/codex-reliability-closeout/CONTEXT.md, docs/history/codex-reliability-closeout/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/codex-reliability-closeout/delivery.md

---
type: bee.delivery
title: codex-reliability-closeout — delivery
description: "Delivery record proposed by bee knowledge promote for work item codex-reliability-closeout: 5 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/codex-reliability-closeout/CONTEXT.md, docs/history/codex-reliability-closeout/plan.md]
  sources: [docs/history/codex-reliability-closeout/CONTEXT.md, docs/history/codex-reliability-closeout/plan.md, .bee/cells/crc-1.json, .bee/cells/crc-2.json, .bee/cells/crc-3.json, .bee/cells/crc-4.json, .bee/cells/crc-5.json]
---

# codex-reliability-closeout — Delivery

## What shipped

- **crc-1** — Isolate Codex canary environment and direct executable routing (5 file(s) changed)
- **crc-2** — Retain truthful transcript regression history and direct token failure (9 file(s) changed)
- **crc-3** — Correct generated Codex onboarding guidance (5 file(s) changed)
- **crc-4** — Repaired gate preview flag parsing and verified approved_gates preservation (3 file(s) changed)
- **crc-5** — Repair Windows activity fixture first-call ordering. (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **crc-1** — `bash scripts/codex-parity-canary-test.sh && bash -n scripts/codex-parity-canary.sh` — seven controls passed; installed live-canary.json records denied and allowed patch/shell writes
- **crc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::session_close::tests && git diff --check c4122d41 HEAD -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs` — 43 transcript tests passed; committed whitespace check passed; full-suite 3929 passed
- **crc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard` — onboard tests passed; onboarding-cli.log retains fresh and refresh CLI checks; dev regen refreshed repository metadata
- **crc-4** — `Run targeted gate preview CLI regression tests on base and head; corrected aliases return a packet and leave approval gates unchanged; bee dev release-manifest --check at integration` — targeted gate preview CLI regression tests and release-manifest check pass
- **crc-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::activity::tests` — 55 activity tests passed; full suite 3930 passed, zero failed, 20 ignored.

## Deviations

- **crc-1** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-1** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-2** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-2** — Retained three historical failures and one pass with separately labeled synthetic mutation — saved 0690c1eb lacks Claude failure; decision 255ec53b — the plan was wrong about a fact
- **crc-2** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-3** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **crc-3** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **crc-4** — followed the plan
- **crc-4** — sync-ack: Parser repair adds preview to FLAG_ALONE_BOOLEANS without changing skill procedures
- **crc-5** — Leader completed stalled worker cell per decision aafa65ca-6f11-4c0c-a771-e6be102db070; actual Windows CI pending.
- **crc-5** — sync-ack: Test-only change; no skill or production contract changes. Existing hook-runtime knowledge owner records fallback regression and platform limitation.

## Provenance

Proposed by `bee knowledge promote --work codex-reliability-closeout` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/codex-reliability-closeout/CONTEXT.md`, `docs/history/codex-reliability-closeout/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "codex-reliability-closeout" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-13T08:49:05.629Z), the work item declares no bee.areas.

area hook-runtime:
  - [crc-1] Isolate Codex canary environment and direct executable routing — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/crc-1.json)
  - [crc-2] Retain truthful transcript regression history and direct token failure — feature-wide sync per the scribing stamp, 9 file(s) changed (trace .bee/cells/crc-2.json)
  - [crc-3] Correct generated Codex onboarding guidance — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/crc-3.json)
  - [crc-4] Repaired gate preview flag parsing and verified approved_gates preservation — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/crc-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell crc-1 — save as docs/knowledge/patterns/codex-reliability-closeout-crc-1-pitfall.md

---
type: bee.pattern
title: codex-reliability-closeout cell crc-1 — pitfall candidate
description: "Pitfall candidate mined from cell crc-1's capped trace: Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong abou…"
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-crc-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/crc-1.json]
  polarity: pitfall
---

# codex-reliability-closeout cell crc-1 — pitfall candidate

## What the cell did

Isolate Codex canary environment and direct executable routing

## Recorded evidence (verbatim from .bee/cells/crc-1.json)

- **deviation** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **deviation** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **failure_signature** — Set BEE_CODEX_PROBE_BIN in the host environment. The canary passes it into nested bee and hook processes, which can execute the inherited probe instead of the controlled Codex path. Prevent the fake probe from running. test_hostile_subprocess_isolation can still report PASS because it has no probe-executed or isolated-write assertion.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell crc-2 — save as docs/knowledge/patterns/codex-reliability-closeout-crc-2-pitfall.md

---
type: bee.pattern
title: codex-reliability-closeout cell crc-2 — pitfall candidate
description: "Pitfall candidate mined from cell crc-2's capped trace: Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong abou…"
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-crc-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/crc-2.json]
  polarity: pitfall
---

# codex-reliability-closeout cell crc-2 — pitfall candidate

## What the cell did

Retain truthful transcript regression history and direct token failure

## Recorded evidence (verbatim from .bee/cells/crc-2.json)

- **deviation** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **deviation** — Retained three historical failures and one pass with separately labeled synthetic mutation — saved 0690c1eb lacks Claude failure; decision 255ec53b — the plan was wrong about a fact
- **deviation** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell crc-3 — save as docs/knowledge/patterns/codex-reliability-closeout-crc-3-pitfall.md

---
type: bee.pattern
title: codex-reliability-closeout cell crc-3 — pitfall candidate
description: "Pitfall candidate mined from cell crc-3's capped trace: Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong abou…"
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-crc-3-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/crc-3.json]
  polarity: pitfall
---

# codex-reliability-closeout cell crc-3 — pitfall candidate

## What the cell did

Correct generated Codex onboarding guidance

## Recorded evidence (verbatim from .bee/cells/crc-3.json)

- **deviation** — Corrected verify field to an executable command — original mixed prose with shell and could not match proof parser; decision a8464d1c — the plan was wrong about a fact
- **deviation** — sync-ack: Owner behavior checked; Codex map and hook-runtime knowledge synchronized; bee dev regen completed at integration.
- **failure_signature** — The required actual-CLI proof cannot be reproduced from retained artifacts; only a narrative claim remains. Current repository onboarding metadata still emits the exact obsolete guidance that crc-3 claims to remove.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell crc-4 — save as docs/knowledge/patterns/codex-reliability-closeout-crc-4-pitfall.md

---
type: bee.pattern
title: codex-reliability-closeout cell crc-4 — pitfall candidate
description: "Pitfall candidate mined from cell crc-4's capped trace: followed the plan"
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-crc-4-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/crc-4.json]
  polarity: pitfall
---

# codex-reliability-closeout cell crc-4 — pitfall candidate

## What the cell did

Repaired gate preview flag parsing and verified approved_gates preservation

## Recorded evidence (verbatim from .bee/cells/crc-4.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: Parser repair adds preview to FLAG_ALONE_BOOLEANS without changing skill procedures

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell crc-5 — save as docs/knowledge/patterns/codex-reliability-closeout-crc-5-pitfall.md

---
type: bee.pattern
title: codex-reliability-closeout cell crc-5 — pitfall candidate
description: "Pitfall candidate mined from cell crc-5's capped trace: Leader completed stalled worker cell per decision aafa65ca-6f11-4c0c-a771-e6be102db070; actual Windows CI pending."
timestamp: 2026-09-13
bee:
  id: codex-reliability-closeout-crc-5-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/crc-5.json]
  polarity: pitfall
---

# codex-reliability-closeout cell crc-5 — pitfall candidate

## What the cell did

Repair Windows activity fixture first-call ordering.

## Recorded evidence (verbatim from .bee/cells/crc-5.json)

- **deviation** — Leader completed stalled worker cell per decision aafa65ca-6f11-4c0c-a771-e6be102db070; actual Windows CI pending.
- **deviation** — sync-ack: Test-only change; no skill or production contract changes. Existing hook-runtime knowledge owner records fallback regression and platform limitation.
- **failure_signature** — The test-only fallback switch changes production behavior because its branch is compiled and honored outside tests.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 5 pattern candidate(s), 0 file(s) written.