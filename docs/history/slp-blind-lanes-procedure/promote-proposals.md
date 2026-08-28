promote proposal for work item "slp-blind-lanes-procedure" (docs/history/slp-blind-lanes-procedure/CONTEXT.md + docs/history/slp-blind-lanes-procedure/plan.md) — 6 capped cell(s): blp-1, blp-2, blp-3, blp-4, blp-5, blp-6
anchor: history — docs/history/slp-blind-lanes-procedure/CONTEXT.md, docs/history/slp-blind-lanes-procedure/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-blind-lanes-procedure/delivery.md

---
type: bee.delivery
title: slp-blind-lanes-procedure — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-blind-lanes-procedure: 6 capped cell(s), 13 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-delivery
  lifecycle: active
  required_context: [docs/history/slp-blind-lanes-procedure/CONTEXT.md, docs/history/slp-blind-lanes-procedure/plan.md]
  sources: [docs/history/slp-blind-lanes-procedure/CONTEXT.md, docs/history/slp-blind-lanes-procedure/plan.md, .bee/cells/blp-1.json, .bee/cells/blp-2.json, .bee/cells/blp-3.json, .bee/cells/blp-4.json, .bee/cells/blp-5.json, .bee/cells/blp-6.json]
---

# slp-blind-lanes-procedure — Delivery

## What shipped

- **blp-1** — --expertise now renders into gather, reviewer and briefless advisor prompts, and is a typed refusal beside a brief (10 file(s) changed)
- **blp-2** — decisions log --rejected records the options not taken as a list, declared in the registry, with a new handler-versus-registry drift net proven red first (6 file(s) changed)
- **blp-3** — The leaning guard skips a tagged fence in all three scans, refuses an unclosed one, and shares one fence scanner with the dossier parser (3 file(s) changed)
- **blp-4** — Blocking a cell now files a blocker letter whose Needs-your-call item names the cell and the reason it recorded (2 file(s) changed)
- **blp-5** — Truth Table Test and CRUD Lifecycle check land as full craft sections; the 5-Layer rubric lands as a frame citing existing homes; both reachable from reviewer method steps 1-2 (3 file(s) changed)
- **blp-6** — The blind-lane procedure now has one written home, and the two surfaces that contradicted it are corrected (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **blp-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml devtools::prompts && .bee/bin/bee dev release-manifest --check`
- **blp-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::decisions && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml catalog`
- **blp-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind`
- **blp-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::mailbox && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::cells`
- **blp-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws && .bee/bin/bee dev release-manifest --check`
- **blp-6** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts && .bee/bin/bee dev release-manifest --check && .bee/bin/bee knowledge index --check`

## Deviations

- **blp-1** — Also committed .bee/onboarding.json, which the cell did not name — bee dev regen rewrites its three prompt hashes, and leaving it out would report prompt drift on the next onboard — something else had to be fixed first
- **blp-2** — Filled the new struct field in packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs, a file the cell did not list — three exhaustive LogParams literals live there and the crate does not compile without them; reserved the path under blp-2-builder before writing — the plan was wrong about a fact
- **blp-2** — Each --rejected entry now passes assert_safe_content like --alternatives does — free prose reaching the append-only log must not walk past the verb's own stated secret / instruction-like refusal — something else had to be fixed first
- **blp-2** — packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs was listed by the cell but needed no edit — decisions log's whole write path (do_log) lives in verbs_read.rs; verbs_write.rs holds decisions tag — the plan was wrong about a fact
- **blp-2** — sync-ack: CLI surface only: no skill instruction changes because the prose telling an agent when to record a rejected set is slice 4's own cell (D2(d) procedure text), and this cell adds the flag the CLI refused to accept before it
- **blp-3** — Ran the verify without its PATH= prefix — the worktree write guard refuses a command with that shell expansion, and cargo is already on PATH — hit an unforeseen obstacle
- **blp-3** — Left the pre-existing clippy never_loop error in verbs/blind/mod.rs:366 untouched — it is at HEAD, already recorded, and outside this cell — something else had to be fixed first
- **blp-4** — Extracted run_block's mutation into a callable block_cell before wiring the producer — the stop lived inside a dispatch closure, so the letter could not be proved end to end without it — something else had to be fixed first
- **blp-4** — Capped through --sync-ack — the workflow-state sync door fired, but this change touches no skill text and the cell declares affects_skills: [] — hit an unforeseen obstacle
- **blp-4** — sync-ack: No skill surface changes: this wires the first producer of an already-declared entry kind inside the mailbox module. No bee-planning/swarming/reviewing/capturing text describes what cells block appends, and the cell declares affects_skills: []. The blind-lane procedure prose that DOES describe this channel is phase 4's own cell.
- **blp-5** — followed the plan
- **blp-6** — Edited packages/bee/AGENTS.block.md as well as AGENTS.md — AGENTS.md holds the template body verbatim between BEE:START/BEE:END and bee dev regen splices it back from the template, so editing only AGENTS.md would be reverted by the regen this cell requires — hit an unforeseen obstacle
- **blp-6** — sync-ack: The AGENTS.md edit is one pointer clause added to the Deep contracts list; it does not touch the agents-capture-line-at-close rule block, so its applied_at files carry nothing to sync.

## Provenance

Proposed by `bee knowledge promote --work slp-blind-lanes-procedure` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-blind-lanes-procedure/CONTEXT.md`, `docs/history/slp-blind-lanes-procedure/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell blp-1 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-1-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-1 — pitfall candidate
description: "Pitfall candidate mined from cell blp-1's capped trace: Also committed .bee/onboarding.json, which the cell did not name — bee dev regen rewrites its three prompt hashes, and leaving it out would report prompt drift…"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-1.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-1 — pitfall candidate

## What the cell did

--expertise now renders into gather, reviewer and briefless advisor prompts, and is a typed refusal beside a brief

## Recorded evidence (verbatim from .bee/cells/blp-1.json)

- **deviation** — Also committed .bee/onboarding.json, which the cell did not name — bee dev regen rewrites its three prompt hashes, and leaving it out would report prompt drift on the next onboard — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell blp-2 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-2-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-2 — pitfall candidate
description: "Pitfall candidate mined from cell blp-2's capped trace: Filled the new struct field in packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs, a file the cell did not list — three exhaustive LogParams literals…"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-2.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-2 — pitfall candidate

## What the cell did

decisions log --rejected records the options not taken as a list, declared in the registry, with a new handler-versus-registry drift net proven red first

## Recorded evidence (verbatim from .bee/cells/blp-2.json)

- **deviation** — Filled the new struct field in packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs, a file the cell did not list — three exhaustive LogParams literals live there and the crate does not compile without them; reserved the path under blp-2-builder before writing — the plan was wrong about a fact
- **deviation** — Each --rejected entry now passes assert_safe_content like --alternatives does — free prose reaching the append-only log must not walk past the verb's own stated secret / instruction-like refusal — something else had to be fixed first
- **deviation** — packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs was listed by the cell but needed no edit — decisions log's whole write path (do_log) lives in verbs_read.rs; verbs_write.rs holds decisions tag — the plan was wrong about a fact
- **deviation** — sync-ack: CLI surface only: no skill instruction changes because the prose telling an agent when to record a rejected set is slice 4's own cell (D2(d) procedure text), and this cell adds the flag the CLI refused to accept before it

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell blp-3 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-3-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-3 — pitfall candidate
description: "Pitfall candidate mined from cell blp-3's capped trace: Ran the verify without its PATH= prefix — the worktree write guard refuses a command with that shell expansion, and cargo is already on PATH — hit an unforesee…"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-3.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-3 — pitfall candidate

## What the cell did

The leaning guard skips a tagged fence in all three scans, refuses an unclosed one, and shares one fence scanner with the dossier parser

## Recorded evidence (verbatim from .bee/cells/blp-3.json)

- **deviation** — Ran the verify without its PATH= prefix — the worktree write guard refuses a command with that shell expansion, and cargo is already on PATH — hit an unforeseen obstacle
- **deviation** — Left the pre-existing clippy never_loop error in verbs/blind/mod.rs:366 untouched — it is at HEAD, already recorded, and outside this cell — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell blp-4 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-4-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-4 — pitfall candidate
description: "Pitfall candidate mined from cell blp-4's capped trace: Extracted run_block's mutation into a callable block_cell before wiring the producer — the stop lived inside a dispatch closure, so the letter could not be pro…"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-4.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-4 — pitfall candidate

## What the cell did

Blocking a cell now files a blocker letter whose Needs-your-call item names the cell and the reason it recorded

## Recorded evidence (verbatim from .bee/cells/blp-4.json)

- **deviation** — Extracted run_block's mutation into a callable block_cell before wiring the producer — the stop lived inside a dispatch closure, so the letter could not be proved end to end without it — something else had to be fixed first
- **deviation** — Capped through --sync-ack — the workflow-state sync door fired, but this change touches no skill text and the cell declares affects_skills: [] — hit an unforeseen obstacle
- **deviation** — sync-ack: No skill surface changes: this wires the first producer of an already-declared entry kind inside the mailbox module. No bee-planning/swarming/reviewing/capturing text describes what cells block appends, and the cell declares affects_skills: []. The blind-lane procedure prose that DOES describe this channel is phase 4's own cell.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell blp-5 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-5-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-5 — pitfall candidate
description: "Pitfall candidate mined from cell blp-5's capped trace: followed the plan"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-5.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-5 — pitfall candidate

## What the cell did

Truth Table Test and CRUD Lifecycle check land as full craft sections; the 5-Layer rubric lands as a frame citing existing homes; both reachable from reviewer method steps 1-2

## Recorded evidence (verbatim from .bee/cells/blp-5.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell blp-6 — save as docs/knowledge/patterns/slp-blind-lanes-procedure-blp-6-pitfall.md

---
type: bee.pattern
title: slp-blind-lanes-procedure cell blp-6 — pitfall candidate
description: "Pitfall candidate mined from cell blp-6's capped trace: Edited packages/bee/AGENTS.block.md as well as AGENTS.md — AGENTS.md holds the template body verbatim between BEE:START/BEE:END and bee dev regen splices it ba…"
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-procedure-blp-6-pitfall
  lifecycle: draft
  sources: [.bee/cells/blp-6.json]
  polarity: pitfall
---

# slp-blind-lanes-procedure cell blp-6 — pitfall candidate

## What the cell did

The blind-lane procedure now has one written home, and the two surfaces that contradicted it are corrected

## Recorded evidence (verbatim from .bee/cells/blp-6.json)

- **deviation** — Edited packages/bee/AGENTS.block.md as well as AGENTS.md — AGENTS.md holds the template body verbatim between BEE:START/BEE:END and bee dev regen splices it back from the template, so editing only AGENTS.md would be reverted by the regen this cell requires — hit an unforeseen obstacle
- **deviation** — sync-ack: The AGENTS.md edit is one pointer clause added to the Deep contracts list; it does not touch the agents-capture-line-at-close rule block, so its applied_at files carry nothing to sync.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 6 pattern candidate(s), 0 file(s) written.