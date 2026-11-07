promote proposal for work item "windows-ci-1107" (docs/history/windows-ci-1107/plan.md) — 4 capped cell(s): wc7-1, wc7-2, wc7-3, wc7-4
anchor: history — docs/history/windows-ci-1107/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/windows-ci-1107/delivery.md

---
type: bee.delivery
title: windows-ci-1107 — delivery
description: "Delivery record proposed by bee knowledge promote for work item windows-ci-1107: 4 capped cell(s), 2 recorded deviation(s)."
timestamp: 2026-11-07
bee:
  id: windows-ci-1107-delivery
  lifecycle: active
  required_context: [docs/history/windows-ci-1107/plan.md]
  sources: [docs/history/windows-ci-1107/plan.md, .bee/cells/wc7-1.json, .bee/cells/wc7-2.json, .bee/cells/wc7-3.json, .bee/cells/wc7-4.json]
---

# windows-ci-1107 — Delivery

## What shipped

- **wc7-1** — Escaped the cwd in the steer_to listing fixture and set core.autocrlf false in the three main-repo worktree fixtures; commit 2aabd89f7 (2 file(s) changed)
- **wc7-2** — Both onboard tests now expect the platform vendored binary name; plan.rs unchanged (item was copy_binary, not a hook re-wire). Commit 18db105f152d9bfa08694cb0efa6cfe2c714628e (1 file(s) changed)
- **wc7-3** — Fixture now single-quotes forward-slash paths for Git Bash, strips the \\?\ prefix and pins core.autocrlf false; Linux 10/10 pass, Windows unproven until CI; commit 07c75bf60 (1 file(s) changed)
- **wc7-4** — Split the spill test by platform: unix keeps the spill assertions, non-unix asserts pass-through with no marker and no spill file; commit b7fd1a08d5d5720b89e71a122bb384e299dd78d1 (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **wc7-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- claude_session_paseo_claude_run uncommitted_bee_store_changes_on_main` — cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- claude_session_paseo_claude_run uncommitted_bee_store_changes_on_main — green:unit — 6 passed 0 failed; the two named test…
- **wc7-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- onboard::tests::` — 107 passed on Linux; the two edited tests live in that module. The full-suite run had one comment_baseline failure that passed on isolated re-run (another worker was editing the checkout).
- **wc7-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test merge_full_suite` — PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test merge_full_suite — green:unit — Linux run of the one touched integration tes…
- **wc7-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test output_spill` — the cell changes one integration test file; 8 passed on linux

## Deviations

- **wc7-1** — sync-ack: test fixture change only; no skill behavior changes
- **wc7-3** — sync-ack: test fixture change only; no skill behavior changes

## Provenance

Proposed by `bee knowledge promote --work windows-ci-1107` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/windows-ci-1107/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell wc7-1 — save as docs/knowledge/patterns/windows-ci-1107-wc7-1-pitfall.md

---
type: bee.pattern
title: windows-ci-1107 cell wc7-1 — pitfall candidate
description: "Pitfall candidate mined from cell wc7-1's capped trace: sync-ack: test fixture change only; no skill behavior changes"
timestamp: 2026-11-07
bee:
  id: windows-ci-1107-wc7-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/wc7-1.json]
  polarity: pitfall
---

# windows-ci-1107 cell wc7-1 — pitfall candidate

## What the cell did

Escaped the cwd in the steer_to listing fixture and set core.autocrlf false in the three main-repo worktree fixtures; commit 2aabd89f7

## Recorded evidence (verbatim from .bee/cells/wc7-1.json)

- **deviation** — sync-ack: test fixture change only; no skill behavior changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wc7-3 — save as docs/knowledge/patterns/windows-ci-1107-wc7-3-pitfall.md

---
type: bee.pattern
title: windows-ci-1107 cell wc7-3 — pitfall candidate
description: "Pitfall candidate mined from cell wc7-3's capped trace: sync-ack: test fixture change only; no skill behavior changes"
timestamp: 2026-11-07
bee:
  id: windows-ci-1107-wc7-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/wc7-3.json]
  polarity: pitfall
---

# windows-ci-1107 cell wc7-3 — pitfall candidate

## What the cell did

Fixture now single-quotes forward-slash paths for Git Bash, strips the \\?\ prefix and pins core.autocrlf false; Linux 10/10 pass, Windows unproven until CI; commit 07c75bf60

## Recorded evidence (verbatim from .bee/cells/wc7-3.json)

- **deviation** — sync-ack: test fixture change only; no skill behavior changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell wc7-4 — save as docs/knowledge/patterns/windows-ci-1107-wc7-4-pitfall.md

---
type: bee.pattern
title: windows-ci-1107 cell wc7-4 — pitfall candidate
description: "Pitfall candidate mined from cell wc7-4's capped trace: 0223bfb925ff"
timestamp: 2026-11-07
bee:
  id: windows-ci-1107-wc7-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/wc7-4.json]
  polarity: pitfall
---

# windows-ci-1107 cell wc7-4 — pitfall candidate

## What the cell did

Split the spill test by platform: unix keeps the spill assertions, non-unix asserts pass-through with no marker and no spill file; commit b7fd1a08d5d5720b89e71a122bb384e299dd78d1

## Recorded evidence (verbatim from .bee/cells/wc7-4.json)

- **failure_signature** — 0223bfb925ff

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.