promote proposal for work item "store-reach-gaps" (.bee/lanes/store-reach-gaps.json + docs/history/store-reach-gaps/promote-proposals.md) — 2 capped cell(s): srg-1, srg-2
anchor: ledger — .bee/lanes/store-reach-gaps.json, docs/history/store-reach-gaps/promote-proposals.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/store-reach-gaps/delivery.md

---
type: bee.delivery
title: store-reach-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item store-reach-gaps: 2 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-08-21
bee:
  id: store-reach-gaps-delivery
  lifecycle: active
  required_context: [.bee/lanes/store-reach-gaps.json, docs/history/store-reach-gaps/promote-proposals.md]
  sources: [.bee/lanes/store-reach-gaps.json, docs/history/store-reach-gaps/promote-proposals.md, .bee/cells/archive/store-reach-gaps/srg-1.json, .bee/cells/archive/store-reach-gaps/srg-2.json]
---

# store-reach-gaps — Delivery

## What shipped

- **srg-1** — An unbound session can no longer silently overwrite another feature's triage in the default record: route --set refuses with both exits named, --no-lane makes the default write explicit, and the flag-declaration drift net now covers route as well as gate. (4 file(s) changed)
- **srg-2** — A freshly bootstrapped worktree now carries .bee/bin/bee, symlinked to main so it can never go stale, with a mode-preserving copy fallback where symlinks are unavailable. (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **srg-1** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml route registry_contracts`
- **srg-2** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree`

## Deviations

- **srg-1** — Judge read raised one issue after the first commit — with live lanes and a featureless default record the refusal printed a literally false sentence about overwriting a feature that does not exist; fixed in 303ac98e by branching the middle clause, with the has-feature wording proven byte-identical.
- **srg-2** — Judge read raised four gaps after the first commit (loose method assertion, no test for the symlink_metadata guard, a comment naming an unreachable scenario, a comment contradicted by the commit); all four were closed in 98675cb6 and re-verified.

## Provenance

Proposed by `bee knowledge promote --work store-reach-gaps` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `.bee/lanes/store-reach-gaps.json`, `docs/history/store-reach-gaps/promote-proposals.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell srg-1 — save as docs/knowledge/patterns/store-reach-gaps-srg-1-pitfall.md

---
type: bee.pattern
title: store-reach-gaps cell srg-1 — pitfall candidate
description: "Pitfall candidate mined from cell srg-1's capped trace: Judge read raised one issue after the first commit — with live lanes and a featureless default record the refusal printed a literally false sentence about over…"
timestamp: 2026-08-21
bee:
  id: store-reach-gaps-srg-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/store-reach-gaps/srg-1.json]
  polarity: pitfall
---

# store-reach-gaps cell srg-1 — pitfall candidate

## What the cell did

An unbound session can no longer silently overwrite another feature's triage in the default record: route --set refuses with both exits named, --no-lane makes the default write explicit, and the flag-declaration drift net now covers route as well as gate.

## Recorded evidence (verbatim from .bee/cells/archive/store-reach-gaps/srg-1.json)

- **deviation** — Judge read raised one issue after the first commit — with live lanes and a featureless default record the refusal printed a literally false sentence about overwriting a feature that does not exist; fixed in 303ac98e by branching the middle clause, with the has-feature wording proven byte-identical.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell srg-2 — save as docs/knowledge/patterns/store-reach-gaps-srg-2-pitfall.md

---
type: bee.pattern
title: store-reach-gaps cell srg-2 — pitfall candidate
description: "Pitfall candidate mined from cell srg-2's capped trace: Judge read raised four gaps after the first commit (loose method assertion, no test for the symlink_metadata guard, a comment naming an unreachable scenario, a…"
timestamp: 2026-08-21
bee:
  id: store-reach-gaps-srg-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/archive/store-reach-gaps/srg-2.json]
  polarity: pitfall
---

# store-reach-gaps cell srg-2 — pitfall candidate

## What the cell did

A freshly bootstrapped worktree now carries .bee/bin/bee, symlinked to main so it can never go stale, with a mode-preserving copy fallback where symlinks are unavailable.

## Recorded evidence (verbatim from .bee/cells/archive/store-reach-gaps/srg-2.json)

- **deviation** — Judge read raised four gaps after the first commit (loose method assertion, no test for the symlink_metadata guard, a comment naming an unreachable scenario, a comment contradicted by the commit); all four were closed in 98675cb6 and re-verified.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 2 pattern candidate(s), 0 file(s) written.