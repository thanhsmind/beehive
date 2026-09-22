promote proposal for work item "mistake-fix-at" (docs/history/mistake-fix-at/CONTEXT.md + docs/history/mistake-fix-at/plan.md) — 4 capped cell(s): mfa-1, mfa-2, mfa-3, mfa-4
anchor: history — docs/history/mistake-fix-at/CONTEXT.md, docs/history/mistake-fix-at/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/mistake-fix-at/delivery.md

---
type: bee.delivery
title: mistake-fix-at — delivery
description: "Delivery record proposed by bee knowledge promote for work item mistake-fix-at: 4 capped cell(s), 10 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: mistake-fix-at-delivery
  lifecycle: active
  areas: [human-mailbox]
  required_context: [docs/history/mistake-fix-at/CONTEXT.md, docs/history/mistake-fix-at/plan.md]
  sources: [docs/history/mistake-fix-at/CONTEXT.md, docs/history/mistake-fix-at/plan.md, .bee/cells/mfa-1.json, .bee/cells/mfa-2.json, .bee/cells/mfa-3.json, .bee/cells/mfa-4.json]
---

# mistake-fix-at — Delivery

## What shipped

- **mfa-1** — bee mailbox reflect and the cap now take a required --fix-at layer, read through one door and stored on the entry row, the letter and trace.mistakes (11 file(s) changed)
- **mfa-2** — bee close files one P3 fix-at backlog row per mechanizable mistake, deduped and fail-open (2 file(s) changed)
- **mfa-3** — The weekly miner keys a reflection on its fix-at layer plus the first four words, and the lesson row names the key and quotes every run (1 file(s) changed)
- **mfa-4** — The fix-at layer is spelled in every home an agent reads: the doctrine bullet, the worker prompt's Result form, the promotion tree, the mailbox area spec and the close recipe (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mfa-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee handlers_close && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee finish_support && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch` — the cell's own verify, all five stages exit 0: mailbox 218, handlers_close 19, catalog 11, registry_contracts 12, registry_dispatch 9. NARROWED, and the gaps are named: the finish_support stage match…
- **mfa-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers::close && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee backlog` — 73 close tests and 32 backlog tests, the two modules this cell touched; the full declared suite was NOT run here and is left to CI
- **mfa-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox_digest` — 28 tests, the whole mailbox_digest module, the only file this cell touched; cargo was already on PATH so the prefix was a no-op
- **mfa-4** — `.bee/bin/bee dev regen && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers::tests && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test agents_block_render_parity && .bee/bin/bee dev release-manifest --check && diff -q skills/bee-capturing/references/promotion.md .claude-plugin/skills/bee-capturing/references/promotion.md && diff -q skills/bee-capturing/references/promotion.md .claude/skills/bee-capturing/references/promotion.md && diff -q skills/bee-capturing/references/promotion.md .agents/skills/bee-capturing/references/promotion.md && rg -q fix-at packages/bee/AGENTS.block.md AGENTS.md packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md skills/bee-capturing/references/promotion.md docs/knowledge/areas/human-mailbox/overview.md .bee/verify/verify-app/features/worktree-and-close.md` — the cell's own verify, run step by step: regen 3/3 green, verbs::drivers::tests 305 passed 0 failed (every_template_carries_the_original_request_block_and_matches_disk ok), agents_block_render_parity…

## Deviations

- **mfa-1** — Added fix_at: None to the CapFlags, Entry and LetterItem literals in verbs/cells/tests.rs, verbs/knowledge/tests.rs and verbs/work.rs, and to the three non-test mailbox::Entry literals in handlers_close.rs and drivers/close.rs — compile-only, no behavior; the cell named only two compile-fix callers, but a new struct field breaks every exhaustive literal, and the three extra files were reserved before writing — hit an unforeseen obstacle
- **mfa-1** — Also refreshed the `mistake` and `report` flag descriptions on cells.cap and cells.finish in registry_payload.json; the cell named only the new fix-at property, but leaving "in two parts on one line" there would have walked a caller straight into the new refusal — something else had to be fixed first
- **mfa-1** — sync-ack: cell mfa-1 declares affects_skills [] on purpose: D4 of docs/history/mistake-fix-at/CONTEXT.md puts the fix-at instruction in AGENTS.md, the host onboarding template and the rendered worker/cap prompts, which the plan assigns to a later cell of this feature; this cell is the data shape and the door only
- **mfa-2** — The Filed line is suppressed when filed and skipped are both 0 — a close with no mechanizable mistake would otherwise print a 0 fix-at row(s), 0 already there line on nearly every close, and the Retired line already takes that rule at moved == 0 — found a better route
- **mfa-2** — BACKLOG_MAX_TITLE was made pub(crate) beside the three items the cell named, so close truncates the title to the same cap bee backlog add enforces instead of re-declaring the number — hit an unforeseen obstacle
- **mfa-2** — sync-ack: bee backlog add behaviour is byte-identical (the seven-key row builder was only lifted, no verb, flag or row shape changed), so no bee-shaping or bee-grooming page goes stale; the one doctrine edit this feature owes is mfa-4, which owns skills/bee-capturing/references/promotion.md and every other prose home, and this cell may not write outside its two declared files
- **mfa-3** — trouble_lines returns Vec<TroubleLine> (a 3-field struct) instead of the cell's Vec<(String,String)> — the lesson row must print `fix-at <layer>`, and a (key, verbatim) pair cannot tell a reflection key from a whole-line key without parsing the key back and mistaking a broken bullet that starts with the word "check" for a reflection — found a better route
- **mfa-4** — Committed regen outputs the cell's files list did not name (the .codex-plugin and .opencode promotion.md mirrors, the three verify-app feature copies that also carry mfa-1's cells-and-proof edit, five .bee-render.json files and .bee/onboarding.json) — bee dev regen's onboard step renders every tree, not only the three the cell named, and leaving them out would commit a half-rendered tree; each was reserved under mfa-w4 before the write — hit an unforeseen obstacle
- **mfa-4** — Ran the verify chain as separate commands with PATH="$HOME/.cargo/bin:$PATH" instead of the one &&-joined line — the worktree isolation guard refuses a compound command carrying ${CARGO_HOME:-...} — every step ran, in order, all green — hit an unforeseen obstacle
- **mfa-4** — sync-ack: No rule text changed: the edited reflect bullet sits outside every <!-- rule --> block in packages/bee/AGENTS.block.md, and AGENTS.md changed only as the regenerated render of that template, so agents-capture-line-at-close and its applied_at files are untouched.

## Provenance

Proposed by `bee knowledge promote --work mistake-fix-at` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/mistake-fix-at/CONTEXT.md`, `docs/history/mistake-fix-at/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "mistake-fix-at" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-22T06:32:30.869Z), the work item declares no bee.areas.

area human-mailbox:
  - [mfa-1] bee mailbox reflect and the cap now take a required --fix-at layer, read through one door and stored on the entry row, the letter and trace.mistakes — feature-wide sync per the scribing stamp, 11 file(s) changed (trace .bee/cells/mfa-1.json)
  - [mfa-2] bee close files one P3 fix-at backlog row per mechanizable mistake, deduped and fail-open — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/mfa-2.json)
  - [mfa-3] The weekly miner keys a reflection on its fix-at layer plus the first four words, and the lesson row names the key and quotes every run — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/mfa-3.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell mfa-1 — save as docs/knowledge/patterns/mistake-fix-at-mfa-1-pitfall.md

---
type: bee.pattern
title: mistake-fix-at cell mfa-1 — pitfall candidate
description: "Pitfall candidate mined from cell mfa-1's capped trace: Added fix_at: None to the CapFlags, Entry and LetterItem literals in verbs/cells/tests.rs, verbs/knowledge/tests.rs and verbs/work.rs, and to the three non-tes…"
timestamp: 2026-09-22
bee:
  id: mistake-fix-at-mfa-1-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/mfa-1.json]
  polarity: pitfall
---

# mistake-fix-at cell mfa-1 — pitfall candidate

## What the cell did

bee mailbox reflect and the cap now take a required --fix-at layer, read through one door and stored on the entry row, the letter and trace.mistakes

## Recorded evidence (verbatim from .bee/cells/mfa-1.json)

- **deviation** — Added fix_at: None to the CapFlags, Entry and LetterItem literals in verbs/cells/tests.rs, verbs/knowledge/tests.rs and verbs/work.rs, and to the three non-test mailbox::Entry literals in handlers_close.rs and drivers/close.rs — compile-only, no behavior; the cell named only two compile-fix callers, but a new struct field breaks every exhaustive literal, and the three extra files were reserved before writing — hit an unforeseen obstacle
- **deviation** — Also refreshed the `mistake` and `report` flag descriptions on cells.cap and cells.finish in registry_payload.json; the cell named only the new fix-at property, but leaving "in two parts on one line" there would have walked a caller straight into the new refusal — something else had to be fixed first
- **deviation** — sync-ack: cell mfa-1 declares affects_skills [] on purpose: D4 of docs/history/mistake-fix-at/CONTEXT.md puts the fix-at instruction in AGENTS.md, the host onboarding template and the rendered worker/cap prompts, which the plan assigns to a later cell of this feature; this cell is the data shape and the door only

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mfa-2 — save as docs/knowledge/patterns/mistake-fix-at-mfa-2-pitfall.md

---
type: bee.pattern
title: mistake-fix-at cell mfa-2 — pitfall candidate
description: "Pitfall candidate mined from cell mfa-2's capped trace: The Filed line is suppressed when filed and skipped are both 0 — a close with no mechanizable mistake would otherwise print a 0 fix-at row(s), 0 already there …"
timestamp: 2026-09-22
bee:
  id: mistake-fix-at-mfa-2-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/mfa-2.json]
  polarity: pitfall
---

# mistake-fix-at cell mfa-2 — pitfall candidate

## What the cell did

bee close files one P3 fix-at backlog row per mechanizable mistake, deduped and fail-open

## Recorded evidence (verbatim from .bee/cells/mfa-2.json)

- **deviation** — The Filed line is suppressed when filed and skipped are both 0 — a close with no mechanizable mistake would otherwise print a 0 fix-at row(s), 0 already there line on nearly every close, and the Retired line already takes that rule at moved == 0 — found a better route
- **deviation** — BACKLOG_MAX_TITLE was made pub(crate) beside the three items the cell named, so close truncates the title to the same cap bee backlog add enforces instead of re-declaring the number — hit an unforeseen obstacle
- **deviation** — sync-ack: bee backlog add behaviour is byte-identical (the seven-key row builder was only lifted, no verb, flag or row shape changed), so no bee-shaping or bee-grooming page goes stale; the one doctrine edit this feature owes is mfa-4, which owns skills/bee-capturing/references/promotion.md and every other prose home, and this cell may not write outside its two declared files

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mfa-3 — save as docs/knowledge/patterns/mistake-fix-at-mfa-3-pitfall.md

---
type: bee.pattern
title: mistake-fix-at cell mfa-3 — pitfall candidate
description: "Pitfall candidate mined from cell mfa-3's capped trace: trouble_lines returns Vec<TroubleLine> (a 3-field struct) instead of the cell's Vec<(String,String)> — the lesson row must print `fix-at <layer>`, and a (key, …"
timestamp: 2026-09-22
bee:
  id: mistake-fix-at-mfa-3-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/mfa-3.json]
  polarity: pitfall
---

# mistake-fix-at cell mfa-3 — pitfall candidate

## What the cell did

The weekly miner keys a reflection on its fix-at layer plus the first four words, and the lesson row names the key and quotes every run

## Recorded evidence (verbatim from .bee/cells/mfa-3.json)

- **deviation** — trouble_lines returns Vec<TroubleLine> (a 3-field struct) instead of the cell's Vec<(String,String)> — the lesson row must print `fix-at <layer>`, and a (key, verbatim) pair cannot tell a reflection key from a whole-line key without parsing the key back and mistaking a broken bullet that starts with the word "check" for a reflection — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mfa-4 — save as docs/knowledge/patterns/mistake-fix-at-mfa-4-pitfall.md

---
type: bee.pattern
title: mistake-fix-at cell mfa-4 — pitfall candidate
description: "Pitfall candidate mined from cell mfa-4's capped trace: Committed regen outputs the cell's files list did not name (the .codex-plugin and .opencode promotion.md mirrors, the three verify-app feature copies that also…"
timestamp: 2026-09-22
bee:
  id: mistake-fix-at-mfa-4-pitfall
  lifecycle: draft
  areas: [human-mailbox]
  sources: [.bee/cells/mfa-4.json]
  polarity: pitfall
---

# mistake-fix-at cell mfa-4 — pitfall candidate

## What the cell did

The fix-at layer is spelled in every home an agent reads: the doctrine bullet, the worker prompt's Result form, the promotion tree, the mailbox area spec and the close recipe

## Recorded evidence (verbatim from .bee/cells/mfa-4.json)

- **deviation** — Committed regen outputs the cell's files list did not name (the .codex-plugin and .opencode promotion.md mirrors, the three verify-app feature copies that also carry mfa-1's cells-and-proof edit, five .bee-render.json files and .bee/onboarding.json) — bee dev regen's onboard step renders every tree, not only the three the cell named, and leaving them out would commit a half-rendered tree; each was reserved under mfa-w4 before the write — hit an unforeseen obstacle
- **deviation** — Ran the verify chain as separate commands with PATH="$HOME/.cargo/bin:$PATH" instead of the one &&-joined line — the worktree isolation guard refuses a compound command carrying ${CARGO_HOME:-...} — every step ran, in order, all green — hit an unforeseen obstacle
- **deviation** — sync-ack: No rule text changed: the edited reflect bullet sits outside every <!-- rule --> block in packages/bee/AGENTS.block.md, and AGENTS.md changed only as the regenerated render of that template, so agents-capture-line-at-close and its applied_at files are untouched.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 3 area bullet(s), 4 pattern candidate(s), 0 file(s) written.