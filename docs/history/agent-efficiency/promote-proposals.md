promote proposal for work item "agent-efficiency" (docs/history/agent-efficiency/CONTEXT.md + docs/history/agent-efficiency/plan.md) — 7 capped cell(s): aef-1, aef-2, aef-3, aef-4, aef-5, aef-6, aef-7
anchor: history — docs/history/agent-efficiency/CONTEXT.md, docs/history/agent-efficiency/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/agent-efficiency/delivery.md

---
type: bee.delivery
title: agent-efficiency — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-efficiency: 7 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-11-04
bee:
  id: agent-efficiency-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding, onboarding]
  required_context: [docs/history/agent-efficiency/CONTEXT.md, docs/history/agent-efficiency/plan.md]
  sources: [docs/history/agent-efficiency/CONTEXT.md, docs/history/agent-efficiency/plan.md, .bee/cells/aef-1.json, .bee/cells/aef-2.json, .bee/cells/aef-3.json, .bee/cells/aef-4.json, .bee/cells/aef-5.json, .bee/cells/aef-6.json, .bee/cells/aef-7.json]
---

# agent-efficiency — Delivery

## What shipped

- **aef-1** — bee feature finish refuses outside the main checkout (168367e16) (1 file(s) changed)
- **aef-2** — bee herding run caps a finished cell with green proof and commit df46984b5a9a5b23c931d4ade1c02c9446a24a2b (1 file(s) changed)
- **aef-3** — allow dispatch prepare after prompt change without manual rebuild 9ccf792386308cf869b49d230ad28eb4012dad15 (1 file(s) changed)
- **aef-4** — Recorded goal-check miss as gap verdict reopening capped cell for rework 391afd12d498c3d995da1267c3f89efb5c413bc6 (2 file(s) changed)
- **aef-5** — Carry block and reopen reasons in full in prior rounds digest eae5baed4 (2 file(s) changed)
- **aef-6** — Trim worker cell prompt to what the skill does not already carry f31c7a24c (2 file(s) changed)
- **aef-7** — Capture queue drained and every pre-cell promote proposal settled (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **aef-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- feature_group route` — 8 feature_group tests incl. feature_finish_from_worktree_refuses_before_merging; red-first for the final test was not shown (the patch-out attempt broke syntax); bee feature finish also ran live from…
- **aef-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- herding` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- herding — green:unit — herding run post-flight capping and tests passed
- **aef-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prompt — green:unit — narrow prompt renderer and skew test suite in bee crate
- **aef-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- leader_check` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- leader_check — green:unit — 39 tests passed covering gap verdict reopen, optiona…
- **aef-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prior_rounds prepare` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- prior_rounds prepare — green:unit — prior rounds digest and dispatch prepare tes…
- **aef-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers prompts` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers prompts — green:unit — drivers and prompts suites pass with trimmed prom…
- **aef-7** — `.bee/bin/bee status --json` — capture_queue holds only stub 52289a42 from the agent-efficiency-skills close after this cell; deferred-queue complete events for the 22 features and slp-role-gaps; orient's remaining promote item is…

## Deviations

- **aef-1** — Leader wrote round 3 inline: the round-2 herding worker hit its 30-minute ceiling with only a test that asserted the buggy partial-success path and deadlocked on a nested CwdGuard; the fix refuses outside main instead of running stamp and close from a worktree, because bee merges run from main — the plan was wrong about a fact
- **aef-1** — sync-ack: skill text for feature finish landed in agent-efficiency-skills asf-1
- **aef-2** — sync-ack: bee-herding skill text for the auto-cap lands in cell aef-8 of this feature
- **aef-3** — sync-ack: internal render fallback; no skill tells an agent to act differently
- **aef-5** — sync-ack: internal digest length change; no skill text describes the cut
- **aef-6** — sync-ack: prompt trim moves no rule; every removed rule already lives in bee-swarming, which the prompt tells the worker to load
- **aef-7** — Leader ran the 22 promote completes and settled slp-role-gaps from main — the worktree host guard refuses the completing verb, and judge round 2 found slp-role-gaps scoped out by date — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work agent-efficiency` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-efficiency/CONTEXT.md`, `docs/history/agent-efficiency/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "agent-efficiency" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-11-04T06:30:20.907Z), the work item declares no bee.areas.

area workflow-state:
  - [aef-1] bee feature finish refuses outside the main checkout (168367e16) — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-1.json)
  - [aef-2] bee herding run caps a finished cell with green proof and commit df46984b5a9a5b23c931d4ade1c02c9446a24a2b — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-2.json)
  - [aef-3] allow dispatch prepare after prompt change without manual rebuild 9ccf792386308cf869b49d230ad28eb4012dad15 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-3.json)
  - [aef-4] Recorded goal-check miss as gap verdict reopening capped cell for rework 391afd12d498c3d995da1267c3f89efb5c413bc6 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-4.json)
  - [aef-5] Carry block and reopen reasons in full in prior rounds digest eae5baed4 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-5.json)
  - [aef-6] Trim worker cell prompt to what the skill does not already carry f31c7a24c — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-6.json)
  - [aef-7] Capture queue drained and every pre-cell promote proposal settled — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-7.json)

area bee-herding:
  - [aef-1] bee feature finish refuses outside the main checkout (168367e16) — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-1.json)
  - [aef-2] bee herding run caps a finished cell with green proof and commit df46984b5a9a5b23c931d4ade1c02c9446a24a2b — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-2.json)
  - [aef-3] allow dispatch prepare after prompt change without manual rebuild 9ccf792386308cf869b49d230ad28eb4012dad15 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-3.json)
  - [aef-4] Recorded goal-check miss as gap verdict reopening capped cell for rework 391afd12d498c3d995da1267c3f89efb5c413bc6 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-4.json)
  - [aef-5] Carry block and reopen reasons in full in prior rounds digest eae5baed4 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-5.json)
  - [aef-6] Trim worker cell prompt to what the skill does not already carry f31c7a24c — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-6.json)
  - [aef-7] Capture queue drained and every pre-cell promote proposal settled — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-7.json)

area onboarding:
  - [aef-1] bee feature finish refuses outside the main checkout (168367e16) — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-1.json)
  - [aef-2] bee herding run caps a finished cell with green proof and commit df46984b5a9a5b23c931d4ade1c02c9446a24a2b — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-2.json)
  - [aef-3] allow dispatch prepare after prompt change without manual rebuild 9ccf792386308cf869b49d230ad28eb4012dad15 — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-3.json)
  - [aef-4] Recorded goal-check miss as gap verdict reopening capped cell for rework 391afd12d498c3d995da1267c3f89efb5c413bc6 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-4.json)
  - [aef-5] Carry block and reopen reasons in full in prior rounds digest eae5baed4 — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-5.json)
  - [aef-6] Trim worker cell prompt to what the skill does not already carry f31c7a24c — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/aef-6.json)
  - [aef-7] Capture queue drained and every pre-cell promote proposal settled — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/aef-7.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell aef-1 — save as docs/knowledge/patterns/agent-efficiency-aef-1-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-1 — pitfall candidate
description: "Pitfall candidate mined from cell aef-1's capped trace: Leader wrote round 3 inline: the round-2 herding worker hit its 30-minute ceiling with only a test that asserted the buggy partial-success path and deadlocked …"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-1-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-1.json]
  polarity: pitfall
---

# agent-efficiency cell aef-1 — pitfall candidate

## What the cell did

bee feature finish refuses outside the main checkout (168367e16)

## Recorded evidence (verbatim from .bee/cells/aef-1.json)

- **deviation** — Leader wrote round 3 inline: the round-2 herding worker hit its 30-minute ceiling with only a test that asserted the buggy partial-success path and deadlocked on a nested CwdGuard; the fix refuses outside main instead of running stamp and close from a worktree, because bee merges run from main — the plan was wrong about a fact
- **deviation** — sync-ack: skill text for feature finish landed in agent-efficiency-skills asf-1
- **failure_signature** — feature finish run from a non-ordinary checkout stops after worktree merge and exits 0 without the scribing stamp or close (feature_group.rs:315-318)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aef-2 — save as docs/knowledge/patterns/agent-efficiency-aef-2-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-2 — pitfall candidate
description: "Pitfall candidate mined from cell aef-2's capped trace: sync-ack: bee-herding skill text for the auto-cap lands in cell aef-8 of this feature"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-2-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-2.json]
  polarity: pitfall
---

# agent-efficiency cell aef-2 — pitfall candidate

## What the cell did

bee herding run caps a finished cell with green proof and commit df46984b5a9a5b23c931d4ade1c02c9446a24a2b

## Recorded evidence (verbatim from .bee/cells/aef-2.json)

- **deviation** — sync-ack: bee-herding skill text for the auto-cap lands in cell aef-8 of this feature

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aef-3 — save as docs/knowledge/patterns/agent-efficiency-aef-3-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-3 — pitfall candidate
description: "Pitfall candidate mined from cell aef-3's capped trace: sync-ack: internal render fallback; no skill tells an agent to act differently"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-3-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-3.json]
  polarity: pitfall
---

# agent-efficiency cell aef-3 — pitfall candidate

## What the cell did

allow dispatch prepare after prompt change without manual rebuild 9ccf792386308cf869b49d230ad28eb4012dad15

## Recorded evidence (verbatim from .bee/cells/aef-3.json)

- **deviation** — sync-ack: internal render fallback; no skill tells an agent to act differently

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aef-5 — save as docs/knowledge/patterns/agent-efficiency-aef-5-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-5 — pitfall candidate
description: "Pitfall candidate mined from cell aef-5's capped trace: sync-ack: internal digest length change; no skill text describes the cut"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-5-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-5.json]
  polarity: pitfall
---

# agent-efficiency cell aef-5 — pitfall candidate

## What the cell did

Carry block and reopen reasons in full in prior rounds digest eae5baed4

## Recorded evidence (verbatim from .bee/cells/aef-5.json)

- **deviation** — sync-ack: internal digest length change; no skill text describes the cut

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aef-6 — save as docs/knowledge/patterns/agent-efficiency-aef-6-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-6 — pitfall candidate
description: "Pitfall candidate mined from cell aef-6's capped trace: sync-ack: prompt trim moves no rule; every removed rule already lives in bee-swarming, which the prompt tells the worker to load"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-6-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-6.json]
  polarity: pitfall
---

# agent-efficiency cell aef-6 — pitfall candidate

## What the cell did

Trim worker cell prompt to what the skill does not already carry f31c7a24c

## Recorded evidence (verbatim from .bee/cells/aef-6.json)

- **deviation** — sync-ack: prompt trim moves no rule; every removed rule already lives in bee-swarming, which the prompt tells the worker to load

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aef-7 — save as docs/knowledge/patterns/agent-efficiency-aef-7-pitfall.md

---
type: bee.pattern
title: agent-efficiency cell aef-7 — pitfall candidate
description: "Pitfall candidate mined from cell aef-7's capped trace: Leader ran the 22 promote completes and settled slp-role-gaps from main — the worktree host guard refuses the completing verb, and judge round 2 found slp-role…"
timestamp: 2026-11-04
bee:
  id: agent-efficiency-aef-7-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, onboarding]
  sources: [.bee/cells/aef-7.json]
  polarity: pitfall
---

# agent-efficiency cell aef-7 — pitfall candidate

## What the cell did

Capture queue drained and every pre-cell promote proposal settled

## Recorded evidence (verbatim from .bee/cells/aef-7.json)

- **deviation** — Leader ran the 22 promote completes and settled slp-role-gaps from main — the worktree host guard refuses the completing verb, and judge round 2 found slp-role-gaps scoped out by date — something else had to be fixed first
- **failure_signature** — slp-role-gaps promote proposal predates the cell and had no complete event

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 21 area bullet(s), 6 pattern candidate(s), 0 file(s) written.