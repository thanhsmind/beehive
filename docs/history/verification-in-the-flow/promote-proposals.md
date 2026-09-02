promote proposal for work item "verification-in-the-flow" (docs/history/verification-in-the-flow/CONTEXT.md + docs/history/verification-in-the-flow/plan.md) — 5 capped cell(s): vif-1, vif-2, vif-3, vif-4, vif-5
anchor: history — docs/history/verification-in-the-flow/CONTEXT.md, docs/history/verification-in-the-flow/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/verification-in-the-flow/delivery.md

---
type: bee.delivery
title: verification-in-the-flow — delivery
description: "Delivery record proposed by bee knowledge promote for work item verification-in-the-flow: 5 capped cell(s), 12 recorded deviation(s)."
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-delivery
  lifecycle: active
  required_context: [docs/history/verification-in-the-flow/CONTEXT.md, docs/history/verification-in-the-flow/plan.md]
  sources: [docs/history/verification-in-the-flow/CONTEXT.md, docs/history/verification-in-the-flow/plan.md, .bee/cells/vif-1.json, .bee/cells/vif-2.json, .bee/cells/vif-3.json, .bee/cells/vif-4.json, .bee/cells/vif-5.json]
---

# verification-in-the-flow — Delivery

## What shipped

- **vif-1** — Onboard verification notice branches on the verify-app source skill: absent draws one of two bee-verifying offers, present draws a bee-verify-upkeep pointer, legacy commands.verify still wins (2 file(s) changed)
- **vif-2** — AGENTS.md's bee block is now pinned byte-for-byte to its rendered source; an unregenerated AGENTS.block.md edit turns the fence red and the message names bee dev regen (1 file(s) changed)
- **vif-3** — verify-app is a constant in both verification skills, the commands.test composition is gone, and all five rendered trees match source (2 file(s) changed)
- **vif-4** — Feature map named as a read-first state layer in AGENTS.md, fourth proof case added, and the three skill load points pointed at it (5 file(s) changed)
- **vif-5** — Moved bee's verification skill to .bee/verify/verify-app (name verify-app, control-bee 0755 kept), removed the old .claude/skills/verify-bee tree in the same commit, rendered into all three runtime homes via onboard --apply, and drove the onboard feature end to end; evidence kept at /tmp/bee-verify/evidence/20260902-092442-728577 (11 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **vif-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml onboard::notices`
- **vif-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test agents_block_render_parity`
- **vif-3** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test specs_fence --test agents_block_render_parity && .bee/bin/bee dev release-manifest --check`
- **vif-4** — `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test agents_block_render_parity --test rule_index_parity --test pointer_integrity --test specs_fence && .bee/bin/bee dev release-manifest --check`
- **vif-5** — `bash .bee/verify/verify-app/control-bee doctor`

## Deviations

- **vif-1** — Also rewrote stale_advisor_key_is_reported in the same file — it asserted an empty notice list for a tested repo, a premise this change deletes, and the cell named only a_declared_test_command_draws_no_notice_in_any_shape as flipping red — something else had to be fixed first
- **vif-1** — Renamed a_declared_test_command_draws_no_notice_in_any_shape to a_declared_test_command_now_draws_the_undriven_product_offer — the old name states the old truth as a fact and would be a lie in the file, and the cell asked for a rewrite to the new truth — found a better route
- **vif-1** — Built the probe path from the existing pub verify_source_root helper in plan.rs instead of joining the path segments again in notices.rs — those segments already have one home — found a better route
- **vif-1** — Added the anti-nag sentence to BOTH generate notices, not only the new one — the existing constant is a generate offer too, and plan.md decision 5 binds every offer onboard can re-print on a version mismatch — found a better route
- **vif-2** — followed the plan
- **vif-3** — followed the plan
- **vif-4** — followed the plan
- **vif-4** — sync-ack: The edit inside agents-capture-line-at-close adds the feature map as a second read-first state layer, not a change to the capture line; its single home is AGENTS.md plus the three load-point skills (bee-shaping, bee-planning, bee-swarming), which this cell touched. bee-capturing owns docs/knowledge sync, not .bee/verify/verify-app/features/, so mirroring the fact there would duplicate it into a skill that does not own the artifact. bee-hive's rows cite the capture-line rule, which is unchanged.
- **vif-5** — Added the bash prefix to the .../control-bee shorthand forms in SKILL.md's Drive fence and to the VERIFY_CWD example, beyond the 7 literal-path refs the cell counted — a rendered 0644 copy refuses a bare path, and the cell asked for the shown form to be the one that works from both copies — found a better route
- **vif-5** — Repointed the Keeping this honest pointer from /maintain-verification-skill to the bee-verify-upkeep skill — that slash command exists only under .claude, and the tree is now rendered into three runtimes where it does not — hit an unforeseen obstacle
- **vif-5** — Fixed two stale verify-bee strings inside control-bee (a header comment and the sandbox README text it writes) — the prohibition is on rewriting behavior and neither string is behavior — something else had to be fixed first
- **vif-5** — Reverted the timestamp-only churn onboard --apply left in .bee/onboarding.json instead of committing it — it is bookkeeping noise, not part of this cell — found a better route

## Provenance

Proposed by `bee knowledge promote --work verification-in-the-flow` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/verification-in-the-flow/CONTEXT.md`, `docs/history/verification-in-the-flow/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell vif-1 — save as docs/knowledge/patterns/verification-in-the-flow-vif-1-pitfall.md

---
type: bee.pattern
title: verification-in-the-flow cell vif-1 — pitfall candidate
description: "Pitfall candidate mined from cell vif-1's capped trace: Also rewrote stale_advisor_key_is_reported in the same file — it asserted an empty notice list for a tested repo, a premise this change deletes, and the cell n…"
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-vif-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/vif-1.json]
  polarity: pitfall
---

# verification-in-the-flow cell vif-1 — pitfall candidate

## What the cell did

Onboard verification notice branches on the verify-app source skill: absent draws one of two bee-verifying offers, present draws a bee-verify-upkeep pointer, legacy commands.verify still wins

## Recorded evidence (verbatim from .bee/cells/vif-1.json)

- **deviation** — Also rewrote stale_advisor_key_is_reported in the same file — it asserted an empty notice list for a tested repo, a premise this change deletes, and the cell named only a_declared_test_command_draws_no_notice_in_any_shape as flipping red — something else had to be fixed first
- **deviation** — Renamed a_declared_test_command_draws_no_notice_in_any_shape to a_declared_test_command_now_draws_the_undriven_product_offer — the old name states the old truth as a fact and would be a lie in the file, and the cell asked for a rewrite to the new truth — found a better route
- **deviation** — Built the probe path from the existing pub verify_source_root helper in plan.rs instead of joining the path segments again in notices.rs — those segments already have one home — found a better route
- **deviation** — Added the anti-nag sentence to BOTH generate notices, not only the new one — the existing constant is a generate offer too, and plan.md decision 5 binds every offer onboard can re-print on a version mismatch — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell vif-2 — save as docs/knowledge/patterns/verification-in-the-flow-vif-2-pitfall.md

---
type: bee.pattern
title: verification-in-the-flow cell vif-2 — pitfall candidate
description: "Pitfall candidate mined from cell vif-2's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-vif-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/vif-2.json]
  polarity: pitfall
---

# verification-in-the-flow cell vif-2 — pitfall candidate

## What the cell did

AGENTS.md's bee block is now pinned byte-for-byte to its rendered source; an unregenerated AGENTS.block.md edit turns the fence red and the message names bee dev regen

## Recorded evidence (verbatim from .bee/cells/vif-2.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell vif-3 — save as docs/knowledge/patterns/verification-in-the-flow-vif-3-pitfall.md

---
type: bee.pattern
title: verification-in-the-flow cell vif-3 — pitfall candidate
description: "Pitfall candidate mined from cell vif-3's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-vif-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/vif-3.json]
  polarity: pitfall
---

# verification-in-the-flow cell vif-3 — pitfall candidate

## What the cell did

verify-app is a constant in both verification skills, the commands.test composition is gone, and all five rendered trees match source

## Recorded evidence (verbatim from .bee/cells/vif-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell vif-4 — save as docs/knowledge/patterns/verification-in-the-flow-vif-4-pitfall.md

---
type: bee.pattern
title: verification-in-the-flow cell vif-4 — pitfall candidate
description: "Pitfall candidate mined from cell vif-4's capped trace: followed the plan"
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-vif-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/vif-4.json]
  polarity: pitfall
---

# verification-in-the-flow cell vif-4 — pitfall candidate

## What the cell did

Feature map named as a read-first state layer in AGENTS.md, fourth proof case added, and the three skill load points pointed at it

## Recorded evidence (verbatim from .bee/cells/vif-4.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: The edit inside agents-capture-line-at-close adds the feature map as a second read-first state layer, not a change to the capture line; its single home is AGENTS.md plus the three load-point skills (bee-shaping, bee-planning, bee-swarming), which this cell touched. bee-capturing owns docs/knowledge sync, not .bee/verify/verify-app/features/, so mirroring the fact there would duplicate it into a skill that does not own the artifact. bee-hive's rows cite the capture-line rule, which is unchanged.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell vif-5 — save as docs/knowledge/patterns/verification-in-the-flow-vif-5-pitfall.md

---
type: bee.pattern
title: verification-in-the-flow cell vif-5 — pitfall candidate
description: "Pitfall candidate mined from cell vif-5's capped trace: Added the bash prefix to the .../control-bee shorthand forms in SKILL.md's Drive fence and to the VERIFY_CWD example, beyond the 7 literal-path refs the cell c…"
timestamp: 2026-09-02
bee:
  id: verification-in-the-flow-vif-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/vif-5.json]
  polarity: pitfall
---

# verification-in-the-flow cell vif-5 — pitfall candidate

## What the cell did

Moved bee's verification skill to .bee/verify/verify-app (name verify-app, control-bee 0755 kept), removed the old .claude/skills/verify-bee tree in the same commit, rendered into all three runtime homes via onboard --apply, and drove the onboard feature end to end; evidence kept at /tmp/bee-verify/evidence/20260902-092442-728577

## Recorded evidence (verbatim from .bee/cells/vif-5.json)

- **deviation** — Added the bash prefix to the .../control-bee shorthand forms in SKILL.md's Drive fence and to the VERIFY_CWD example, beyond the 7 literal-path refs the cell counted — a rendered 0644 copy refuses a bare path, and the cell asked for the shown form to be the one that works from both copies — found a better route
- **deviation** — Repointed the Keeping this honest pointer from /maintain-verification-skill to the bee-verify-upkeep skill — that slash command exists only under .claude, and the tree is now rendered into three runtimes where it does not — hit an unforeseen obstacle
- **deviation** — Fixed two stale verify-bee strings inside control-bee (a header comment and the sandbox README text it writes) — the prohibition is on rewriting behavior and neither string is behavior — something else had to be fixed first
- **deviation** — Reverted the timestamp-only churn onboard --apply left in .bee/onboarding.json instead of committing it — it is bookkeeping noise, not part of this cell — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 5 pattern candidate(s), 0 file(s) written.