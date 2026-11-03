promote proposal for work item "paseo-leverage" (docs/history/paseo-leverage/CONTEXT.md + docs/history/paseo-leverage/plan.md) — 7 capped cell(s): pl-1, pl-2, pl-3, pl-4, pl-5, pl-6, pl-7
anchor: history — docs/history/paseo-leverage/CONTEXT.md, docs/history/paseo-leverage/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/paseo-leverage/delivery.md

---
type: bee.delivery
title: paseo-leverage — delivery
description: "Delivery record proposed by bee knowledge promote for work item paseo-leverage: 7 capped cell(s), 13 recorded deviation(s)."
timestamp: 2026-10-12
bee:
  id: paseo-leverage-delivery
  lifecycle: active
  required_context: [docs/history/paseo-leverage/CONTEXT.md, docs/history/paseo-leverage/plan.md]
  sources: [docs/history/paseo-leverage/CONTEXT.md, docs/history/paseo-leverage/plan.md, .bee/cells/pl-1.json, .bee/cells/pl-2.json, .bee/cells/pl-3.json, .bee/cells/pl-4.json, .bee/cells/pl-5.json, .bee/cells/pl-6.json, .bee/cells/pl-7.json]
---

# paseo-leverage — Delivery

## What shipped

- **pl-1** — paseo_state.rs reads Paseo agent files (home resolution, find-by-id, list-by-label, parsed record); status untracked report reads disk first, falls back to paseo ls (2 file(s) changed)
- **pl-2** — round_usage treats a LastUsage with no known numbers as absent and falls through to the pi-session sum, then usage null plus usage_unavailable (1 file(s) changed)
- **pl-3** — herding status shows read_only enforced on paseo, not-enforced on herdr or tmux, for read-only jobs (1 file(s) changed)
- **pl-4** — bee herding fork copies a pi session under a new header, imports it with paseo import, and refuses non-pi, missing, out-of-root and Working sources (10 file(s) changed)
- **pl-5** — Pinned flag count bumped to 224 for the notify --correlation-id flag (1 file(s) changed)
- **pl-6** — Added three ignored paseo_live_* contract tests for ls, inspect keys and the on-disk agent file keys (1 file(s) changed)
- **pl-7** — Paseo channel concept documents notify, read-only, parked, usage, fork, on-disk reader and live test (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pl-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee paseo_state` — 6 tests named paseo_state ran (5 in herding::paseo_state::tests, 1 status test in herding::tests); herding::tests::status also ran, 13 passed; whole suite not run
- **pl-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::` — cell verify; 723 passed 0 failed, includes the new round_usage test that was red before the fix; full suite not run
- **pl-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee prepare && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_worker_guard_contracts` — the cell verify (pi contracts 10 passed) plus cargo test -p bee --bin bee herding::tests::status (16 passed, the new read_only status test listed ok) for the reopen fix in herding.rs; full suite not …
- **pl-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee fork && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — cell verify; 8 fork unit tests + catalog fork test ran by name, catalog 16, registry_contracts 12, plus registry_dispatch 9 and control_loop 72; run in a scratch copy of HEAD plus this cell files bec…
- **pl-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee notify && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee render_brief && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — the cell verify chain (notify 8, render_brief 19, pi_plugin_contracts 122, heartbeat 26, registry 12) plus the reopen fix test cargo test -p bee distinct_flag_vocabulary_is_pinned (1 passed); the ful…
- **pl-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test paseo_live_contract -- --ignored` — 3 passed against installed Paseo 0.11.2; same binary with paseo off PATH printed the skip line and passed 3; default run (no --ignored) shows 3 ignored
- **pl-7** — `rg -n 'herding notify|herding fork|read-only|parked|usage_unavailable|paseo_live' docs/knowledge/areas/bee-herding/the-paseo-channel.md` — docs change; cell verify matched 13 lines, every new Pointer path exists (test -f), and each store id matches .bee/decisions.jsonl

## Deviations

- **pl-1** — Tests set a cfg(test) thread-local TEST_PASEO_HOME that stands in for the PASEO_HOME env var, not the real env var — env mutation races in the parallel in-process suite and a real ~/.paseo on a dev box would leak into the old status tests — found a better route
- **pl-1** — sync-ack: plan sets affects_skills to none for pl-1; the reader is internal and changes no bee-herding skill text
- **pl-2** — usage_unavailable text now says LastUsage has no usage numbers instead of is null — the reason now covers the empty-object case too — found a better route
- **pl-2** — sync-ack: skills/bee-herding is outside this cell's files; pl-7 owns the Paseo channel docs for usage, mode and parked
- **pl-3** — Edited packages/bee-rs/crates/bee/src/herding.rs, outside the cell file list, reserved first — the reopen reason named it as the fix location for the status read_only field — something else had to be fixed first
- **pl-3** — sync-ack: plan assigns affects_skills [] to pl-3; the bee-herding skill and concept sync for status read_only land in the feature docs cell pl-7 (regen_obligation_ack wave-barrier)
- **pl-4** — Ran the verify commands in a scratch copy of HEAD plus this cell files, not in the worktree — sibling uncommitted run.rs/tests.rs edits make the worktree test build fail with E0063 missing read_only — hit an unforeseen obstacle
- **pl-4** — Committed only the five generated supervisor-prompt copies; bee dev regen also rewrote five .bee-render.json files, .bee/onboarding.json and the codex-harness-hardening release-manifest.json, left uncommitted for the wave-barrier regen per regen_obligation_ack — something else had to be fixed first
- **pl-4** — A failed paseo inspect does not refuse the fork (only a confirmed Working state refuses), matching D4; parentSession records the canonicalized source path — found a better route
- **pl-5** — followed the plan
- **pl-6** — the first agent id comes from paseo ls -a -g --json, not ls -a --json — ls -a is scoped to the cwd and listed no agent here while 9 exist globally — the plan was wrong about a fact
- **pl-7** — Cited D3=a7067517, D4=33df9d0d, D5=89d24cf0, D6=1742a721, D7=12211a7d, D8=28b0b5c2, and 6fc537f6 as the hat-wave refinements — the cell listed the ids off by one from D3 on, and .bee/decisions.jsonl maps them this way — the plan was wrong about a fact
- **pl-7** — Wrote that herdr and tmux runs record read_only but nothing enforces it, instead of saying status shows not enforced — herding.rs has no read_only or not-enforced status field, so I did not claim one — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work paseo-leverage` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/paseo-leverage/CONTEXT.md`, `docs/history/paseo-leverage/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pl-1 — save as docs/knowledge/patterns/paseo-leverage-pl-1-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-1 — pitfall candidate
description: "Pitfall candidate mined from cell pl-1's capped trace: Tests set a cfg(test) thread-local TEST_PASEO_HOME that stands in for the PASEO_HOME env var, not the real env var — env mutation races in the parallel in-proc…"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-1.json]
  polarity: pitfall
---

# paseo-leverage cell pl-1 — pitfall candidate

## What the cell did

paseo_state.rs reads Paseo agent files (home resolution, find-by-id, list-by-label, parsed record); status untracked report reads disk first, falls back to paseo ls

## Recorded evidence (verbatim from .bee/cells/pl-1.json)

- **deviation** — Tests set a cfg(test) thread-local TEST_PASEO_HOME that stands in for the PASEO_HOME env var, not the real env var — env mutation races in the parallel in-process suite and a real ~/.paseo on a dev box would leak into the old status tests — found a better route
- **deviation** — sync-ack: plan sets affects_skills to none for pl-1; the reader is internal and changes no bee-herding skill text

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-2 — save as docs/knowledge/patterns/paseo-leverage-pl-2-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-2 — pitfall candidate
description: "Pitfall candidate mined from cell pl-2's capped trace: usage_unavailable text now says LastUsage has no usage numbers instead of is null — the reason now covers the empty-object case too — found a better route"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-2.json]
  polarity: pitfall
---

# paseo-leverage cell pl-2 — pitfall candidate

## What the cell did

round_usage treats a LastUsage with no known numbers as absent and falls through to the pi-session sum, then usage null plus usage_unavailable

## Recorded evidence (verbatim from .bee/cells/pl-2.json)

- **deviation** — usage_unavailable text now says LastUsage has no usage numbers instead of is null — the reason now covers the empty-object case too — found a better route
- **deviation** — sync-ack: skills/bee-herding is outside this cell's files; pl-7 owns the Paseo channel docs for usage, mode and parked

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-3 — save as docs/knowledge/patterns/paseo-leverage-pl-3-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-3 — pitfall candidate
description: "Pitfall candidate mined from cell pl-3's capped trace: Edited packages/bee-rs/crates/bee/src/herding.rs, outside the cell file list, reserved first — the reopen reason named it as the fix location for the status re…"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-3.json]
  polarity: pitfall
---

# paseo-leverage cell pl-3 — pitfall candidate

## What the cell did

herding status shows read_only enforced on paseo, not-enforced on herdr or tmux, for read-only jobs

## Recorded evidence (verbatim from .bee/cells/pl-3.json)

- **deviation** — Edited packages/bee-rs/crates/bee/src/herding.rs, outside the cell file list, reserved first — the reopen reason named it as the fix location for the status read_only field — something else had to be fixed first
- **deviation** — sync-ack: plan assigns affects_skills [] to pl-3; the bee-herding skill and concept sync for status read_only land in the feature docs cell pl-7 (regen_obligation_ack wave-barrier)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-4 — save as docs/knowledge/patterns/paseo-leverage-pl-4-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-4 — pitfall candidate
description: "Pitfall candidate mined from cell pl-4's capped trace: Ran the verify commands in a scratch copy of HEAD plus this cell files, not in the worktree — sibling uncommitted run.rs/tests.rs edits make the worktree test …"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-4.json]
  polarity: pitfall
---

# paseo-leverage cell pl-4 — pitfall candidate

## What the cell did

bee herding fork copies a pi session under a new header, imports it with paseo import, and refuses non-pi, missing, out-of-root and Working sources

## Recorded evidence (verbatim from .bee/cells/pl-4.json)

- **deviation** — Ran the verify commands in a scratch copy of HEAD plus this cell files, not in the worktree — sibling uncommitted run.rs/tests.rs edits make the worktree test build fail with E0063 missing read_only — hit an unforeseen obstacle
- **deviation** — Committed only the five generated supervisor-prompt copies; bee dev regen also rewrote five .bee-render.json files, .bee/onboarding.json and the codex-harness-hardening release-manifest.json, left uncommitted for the wave-barrier regen per regen_obligation_ack — something else had to be fixed first
- **deviation** — A failed paseo inspect does not refuse the fork (only a confirmed Working state refuses), matching D4; parentSession records the canonicalized source path — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-5 — save as docs/knowledge/patterns/paseo-leverage-pl-5-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-5 — pitfall candidate
description: "Pitfall candidate mined from cell pl-5's capped trace: followed the plan"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-5-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-5.json]
  polarity: pitfall
---

# paseo-leverage cell pl-5 — pitfall candidate

## What the cell did

Pinned flag count bumped to 224 for the notify --correlation-id flag

## Recorded evidence (verbatim from .bee/cells/pl-5.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-6 — save as docs/knowledge/patterns/paseo-leverage-pl-6-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-6 — pitfall candidate
description: "Pitfall candidate mined from cell pl-6's capped trace: the first agent id comes from paseo ls -a -g --json, not ls -a --json — ls -a is scoped to the cwd and listed no agent here while 9 exist globally — the plan w…"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-6-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-6.json]
  polarity: pitfall
---

# paseo-leverage cell pl-6 — pitfall candidate

## What the cell did

Added three ignored paseo_live_* contract tests for ls, inspect keys and the on-disk agent file keys

## Recorded evidence (verbatim from .bee/cells/pl-6.json)

- **deviation** — the first agent id comes from paseo ls -a -g --json, not ls -a --json — ls -a is scoped to the cwd and listed no agent here while 9 exist globally — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pl-7 — save as docs/knowledge/patterns/paseo-leverage-pl-7-pitfall.md

---
type: bee.pattern
title: paseo-leverage cell pl-7 — pitfall candidate
description: "Pitfall candidate mined from cell pl-7's capped trace: Cited D3=a7067517, D4=33df9d0d, D5=89d24cf0, D6=1742a721, D7=12211a7d, D8=28b0b5c2, and 6fc537f6 as the hat-wave refinements — the cell listed the ids off by o…"
timestamp: 2026-10-12
bee:
  id: paseo-leverage-pl-7-pitfall
  lifecycle: draft
  sources: [.bee/cells/pl-7.json]
  polarity: pitfall
---

# paseo-leverage cell pl-7 — pitfall candidate

## What the cell did

Paseo channel concept documents notify, read-only, parked, usage, fork, on-disk reader and live test

## Recorded evidence (verbatim from .bee/cells/pl-7.json)

- **deviation** — Cited D3=a7067517, D4=33df9d0d, D5=89d24cf0, D6=1742a721, D7=12211a7d, D8=28b0b5c2, and 6fc537f6 as the hat-wave refinements — the cell listed the ids off by one from D3 on, and .bee/decisions.jsonl maps them this way — the plan was wrong about a fact
- **deviation** — Wrote that herdr and tmux runs record read_only but nothing enforces it, instead of saying status shows not enforced — herding.rs has no read_only or not-enforced status field, so I did not claim one — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 7 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 7 pattern candidate(s), 0 file(s) written.