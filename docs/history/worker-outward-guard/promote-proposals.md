promote proposal for work item "worker-outward-guard" (docs/history/worker-outward-guard/CONTEXT.md + docs/history/worker-outward-guard/plan.md) — 2 capped cell(s): wog-1, wog-2
anchor: history — docs/history/worker-outward-guard/CONTEXT.md, docs/history/worker-outward-guard/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/worker-outward-guard/delivery.md

---
type: bee.delivery
title: worker-outward-guard — delivery
description: "Delivery record proposed by bee knowledge promote for work item worker-outward-guard: 2 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: worker-outward-guard-delivery
  lifecycle: active
  required_context: [docs/history/worker-outward-guard/CONTEXT.md, docs/history/worker-outward-guard/plan.md]
  sources: [docs/history/worker-outward-guard/CONTEXT.md, docs/history/worker-outward-guard/plan.md, .bee/cells/wog-1.json, .bee/cells/wog-2.json]
---

# worker-outward-guard — Delivery

## What shipped

- **wog-1** — The write guard refuses git push, GitHub writes and nested agent launches from a linked-valid worktree in every phase (4 file(s) changed)
- **wog-2** — guards.worker_outward is named in the four docs, with the read-only list and the gotchas in config-reference.md and a drivable feature file in the verify map (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wog-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks::write_guard` — 270 write_guard tests incl. the 9 new worker_outward ones; also ran hooks:: (655 green) and devtools::comment_baseline (9 green); the rest of the declared suite was not run
- **wog-2** — `.bee/bin/bee dev release-manifest --check && rg -l worker_outward docs/config-reference.md docs/07-contracts.md docs/handbook/register.md docs/product-description/foundations/guards.md` — docs-only cell: the manifest parity check and the four-path pointer check are the cell verify command; no code changed, so no test module applies

## Deviations

- **wog-1** — The deny text reads "denied this shell command" instead of naming the payload tool (Bash/exec) — the plan locks main.rs to ONE new argument and the tool name is not otherwise reachable in checks.rs; threading a second cosmetic signal through main.rs is the layered signal the craft rule refuses — found a better route
- **wog-1** — The worktree id comes from derive_current_worktree(cwd), not (root) — root is the store root, which is the MAIN root for an ungranted linked worktree, so deriving from root would never resolve a real id there — the plan was wrong about a fact
- **wog-1** — The launch refusal names the linked worktree id in its first sentence — the plan text for the Launch FIX names no id, but must_have truth 1 requires every deny to name the worktree id — hit an unforeseen obstacle
- **wog-1** — The truncation scan splits each opaque token on whitespace before running find_git_invocations — at depth 5 the hidden payload survives as ONE token ("git push"), so a token-level basename scan alone would have allowed it — the plan was wrong about a fact
- **wog-1** — The cli-command exemption reads config key "team" first, then "models" — read_config folds models into team (fold_team_key), so a models-only lookup never matched — the plan was wrong about a fact
- **wog-1** — outward_arm takes no &fenced argument — nothing in it reads the fenced text; the deep tokens are enough — found a better route
- **wog-2** — committed the regen-written .bee/onboarding.json timestamp line instead of reverting it — the concurrent-worker guard refuses a tree-wide restore while a sibling worker is live, and hand-editing bee state is not allowed — hit an unforeseen obstacle

## Provenance

Proposed by `bee knowledge promote --work worker-outward-guard` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/worker-outward-guard/CONTEXT.md`, `docs/history/worker-outward-guard/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wog-1 — save as docs/knowledge/patterns/worker-outward-guard-wog-1-pitfall.md

---
type: bee.pattern
title: worker-outward-guard cell wog-1 — pitfall candidate
description: "Pitfall candidate mined from cell wog-1's capped trace: The deny text reads \"denied this shell command\" instead of naming the payload tool (Bash/exec) — the plan locks main.rs to ONE new argument and the tool name i…"
timestamp: 2026-09-22
bee:
  id: worker-outward-guard-wog-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wog-1.json]
  polarity: pitfall
---

# worker-outward-guard cell wog-1 — pitfall candidate

## What the cell did

The write guard refuses git push, GitHub writes and nested agent launches from a linked-valid worktree in every phase

## Recorded evidence (verbatim from .bee/cells/wog-1.json)

- **deviation** — The deny text reads "denied this shell command" instead of naming the payload tool (Bash/exec) — the plan locks main.rs to ONE new argument and the tool name is not otherwise reachable in checks.rs; threading a second cosmetic signal through main.rs is the layered signal the craft rule refuses — found a better route
- **deviation** — The worktree id comes from derive_current_worktree(cwd), not (root) — root is the store root, which is the MAIN root for an ungranted linked worktree, so deriving from root would never resolve a real id there — the plan was wrong about a fact
- **deviation** — The launch refusal names the linked worktree id in its first sentence — the plan text for the Launch FIX names no id, but must_have truth 1 requires every deny to name the worktree id — hit an unforeseen obstacle
- **deviation** — The truncation scan splits each opaque token on whitespace before running find_git_invocations — at depth 5 the hidden payload survives as ONE token ("git push"), so a token-level basename scan alone would have allowed it — the plan was wrong about a fact
- **deviation** — The cli-command exemption reads config key "team" first, then "models" — read_config folds models into team (fold_team_key), so a models-only lookup never matched — the plan was wrong about a fact
- **deviation** — outward_arm takes no &fenced argument — nothing in it reads the fenced text; the deep tokens are enough — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wog-2 — save as docs/knowledge/patterns/worker-outward-guard-wog-2-pitfall.md

---
type: bee.pattern
title: worker-outward-guard cell wog-2 — pitfall candidate
description: "Pitfall candidate mined from cell wog-2's capped trace: committed the regen-written .bee/onboarding.json timestamp line instead of reverting it — the concurrent-worker guard refuses a tree-wide restore while a sibli…"
timestamp: 2026-09-22
bee:
  id: worker-outward-guard-wog-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/wog-2.json]
  polarity: pitfall
---

# worker-outward-guard cell wog-2 — pitfall candidate

## What the cell did

guards.worker_outward is named in the four docs, with the read-only list and the gotchas in config-reference.md and a drivable feature file in the verify map

## Recorded evidence (verbatim from .bee/cells/wog-2.json)

- **deviation** — committed the regen-written .bee/onboarding.json timestamp line instead of reverting it — the concurrent-worker guard refuses a tree-wide restore while a sibling worker is live, and hand-editing bee state is not allowed — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.