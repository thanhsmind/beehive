promote proposal for work item "proof-strength-and-expiry" (docs/history/proof-strength-and-expiry/CONTEXT.md + docs/history/proof-strength-and-expiry/plan.md) — 3 capped cell(s): pse-1, pse-2, pse-3
anchor: history — docs/history/proof-strength-and-expiry/CONTEXT.md, docs/history/proof-strength-and-expiry/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/proof-strength-and-expiry/delivery.md

---
type: bee.delivery
title: proof-strength-and-expiry — delivery
description: "Delivery record proposed by bee knowledge promote for work item proof-strength-and-expiry: 3 capped cell(s), 11 recorded deviation(s)."
timestamp: 2026-09-01
bee:
  id: proof-strength-and-expiry-delivery
  lifecycle: active
  required_context: [docs/history/proof-strength-and-expiry/CONTEXT.md, docs/history/proof-strength-and-expiry/plan.md]
  sources: [docs/history/proof-strength-and-expiry/CONTEXT.md, docs/history/proof-strength-and-expiry/plan.md, .bee/cells/pse-1.json, .bee/cells/pse-2.json, .bee/cells/pse-3.json]
---

# proof-strength-and-expiry — Delivery

## What shipped

- **pse-1** — Closed the cap proof result segment over green:live/green:unit/green:static on the write path; the read path stays tolerant of historical bare-green caps (7 file(s) changed)
- **pse-2** — bee worktree merge names and prints the capped cells whose proof predates the merge base; advisory only, fails open, merge still lands (4 file(s) changed)
- **pse-3** — Every teaching site now shows a qualified proof value and proof_gate.rs fences the set; packages/bee/prompts/worker-cell.md changed, so .bee/bin/bee must be rebuilt and reinstalled at merge or dispatch prepare --kind cell refuses every worker (18 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pse-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **pse-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
- **pse-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check`

## Deviations

- **pse-1** — Also qualified the two bare-green examples in the boundary/undeclared refusal at finish_support.rs:192 — the cell named only :204, but :192 sits in the same function and would teach the exact form that function now refuses — the plan was wrong about a fact
- **pse-1** — Labelled the existing proof.rs test a_capped_cell_with_a_valid_proof_line_is_proven_not_blocking as the D2 test with a doc comment instead of adding a second one — it already caps a bare-green cell and passes feature_proof_check, so a new test would assert the same thing twice — found a better route
- **pse-1** — Reserved and edited packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs, which the files list on this cell did not name — its cap fixture goes through cap_cell_from_flags with a bare green and the change turns it red — hit an unforeseen obstacle
- **pse-1** — Also qualified the fixture proof strings in hooks/cli_shape.rs and hooks/write_guard/tests.rs — neither went red, but both are in the file set for this cell and both show a bee cells cap command carrying the now-refused form — the plan was wrong about a fact
- **pse-1** — Left hooks/session_preamble/tests.rs unedited though the cell listed it — its bare-green example pins a literal source string in session_preamble/budget.rs, which slice 3 owns; editing the test alone would go red — the plan was wrong about a fact
- **pse-1** — sync-ack: D5 assigns every skill, prompt and doc teaching site to slice 3 (skills/bee-swarming/references/worker-details.md among them); this cell is the vocabulary and its write-path check only, and its files list names no skill
- **pse-2** — staleness is the merge base NOT being an ancestor of the cap commit, the inverse of the wording in the cell and in D4 — verified in this repo that the literal direction flags every ordinary cap (the pse-1 commit 9ad04c73 is not an ancestor of merge base e2072df5), so it would fire on the clean merge of this very feature, while the inverted test is silent there and fires exactly on the event D4 names, main moving in after the proof was taken — the plan was wrong about a fact
- **pse-2** — added a 6-line render arm in verbs/worktree/handlers.rs (reserved before writing) beside the existing verify and warning lines — phases.rs only writes the result map, and merge_text_lines is the only place a merge prints, so phases.rs alone left the advisory visible under --json and invisible to the acceptance word prints — the plan was wrong about a fact
- **pse-2** — the staleness read sits after the branch-mismatch check instead of beside the proof read at phases.rs:156 — the merge base needs the branch name, which is not resolved until ~200 lines below the proof door; still inside the zero-mutation zone and still read-only git — hit an unforeseen obstacle
- **pse-2** — sync-ack: the swarming skills are the concurrent cell pse-3 teaching-site sweep, reserved by w-pse-3 and off limits to this worker; this cell adds a merge-door advisory and changes no rule a skill states
- **pse-3** — Also rewrote two prose claims that the result segment is free text — the Edge cases bullet in docs/product-description/lifecycle/execution.md and the EXEC-10 probe row in docs/product-description/verification/lifecycle.md — because pse-1 made both statements false and both files were already in scope — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work proof-strength-and-expiry` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/proof-strength-and-expiry/CONTEXT.md`, `docs/history/proof-strength-and-expiry/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pse-1 — save as docs/knowledge/patterns/proof-strength-and-expiry-pse-1-pitfall.md

---
type: bee.pattern
title: proof-strength-and-expiry cell pse-1 — pitfall candidate
description: "Pitfall candidate mined from cell pse-1's capped trace: Also qualified the two bare-green examples in the boundary/undeclared refusal at finish_support.rs:192 — the cell named only :204, but :192 sits in the same fu…"
timestamp: 2026-09-01
bee:
  id: proof-strength-and-expiry-pse-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/pse-1.json]
  polarity: pitfall
---

# proof-strength-and-expiry cell pse-1 — pitfall candidate

## What the cell did

Closed the cap proof result segment over green:live/green:unit/green:static on the write path; the read path stays tolerant of historical bare-green caps

## Recorded evidence (verbatim from .bee/cells/pse-1.json)

- **deviation** — Also qualified the two bare-green examples in the boundary/undeclared refusal at finish_support.rs:192 — the cell named only :204, but :192 sits in the same function and would teach the exact form that function now refuses — the plan was wrong about a fact
- **deviation** — Labelled the existing proof.rs test a_capped_cell_with_a_valid_proof_line_is_proven_not_blocking as the D2 test with a doc comment instead of adding a second one — it already caps a bare-green cell and passes feature_proof_check, so a new test would assert the same thing twice — found a better route
- **deviation** — Reserved and edited packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs, which the files list on this cell did not name — its cap fixture goes through cap_cell_from_flags with a bare green and the change turns it red — hit an unforeseen obstacle
- **deviation** — Also qualified the fixture proof strings in hooks/cli_shape.rs and hooks/write_guard/tests.rs — neither went red, but both are in the file set for this cell and both show a bee cells cap command carrying the now-refused form — the plan was wrong about a fact
- **deviation** — Left hooks/session_preamble/tests.rs unedited though the cell listed it — its bare-green example pins a literal source string in session_preamble/budget.rs, which slice 3 owns; editing the test alone would go red — the plan was wrong about a fact
- **deviation** — sync-ack: D5 assigns every skill, prompt and doc teaching site to slice 3 (skills/bee-swarming/references/worker-details.md among them); this cell is the vocabulary and its write-path check only, and its files list names no skill

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pse-2 — save as docs/knowledge/patterns/proof-strength-and-expiry-pse-2-pitfall.md

---
type: bee.pattern
title: proof-strength-and-expiry cell pse-2 — pitfall candidate
description: "Pitfall candidate mined from cell pse-2's capped trace: staleness is the merge base NOT being an ancestor of the cap commit, the inverse of the wording in the cell and in D4 — verified in this repo that the literal …"
timestamp: 2026-09-01
bee:
  id: proof-strength-and-expiry-pse-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/pse-2.json]
  polarity: pitfall
---

# proof-strength-and-expiry cell pse-2 — pitfall candidate

## What the cell did

bee worktree merge names and prints the capped cells whose proof predates the merge base; advisory only, fails open, merge still lands

## Recorded evidence (verbatim from .bee/cells/pse-2.json)

- **deviation** — staleness is the merge base NOT being an ancestor of the cap commit, the inverse of the wording in the cell and in D4 — verified in this repo that the literal direction flags every ordinary cap (the pse-1 commit 9ad04c73 is not an ancestor of merge base e2072df5), so it would fire on the clean merge of this very feature, while the inverted test is silent there and fires exactly on the event D4 names, main moving in after the proof was taken — the plan was wrong about a fact
- **deviation** — added a 6-line render arm in verbs/worktree/handlers.rs (reserved before writing) beside the existing verify and warning lines — phases.rs only writes the result map, and merge_text_lines is the only place a merge prints, so phases.rs alone left the advisory visible under --json and invisible to the acceptance word prints — the plan was wrong about a fact
- **deviation** — the staleness read sits after the branch-mismatch check instead of beside the proof read at phases.rs:156 — the merge base needs the branch name, which is not resolved until ~200 lines below the proof door; still inside the zero-mutation zone and still read-only git — hit an unforeseen obstacle
- **deviation** — sync-ack: the swarming skills are the concurrent cell pse-3 teaching-site sweep, reserved by w-pse-3 and off limits to this worker; this cell adds a merge-door advisory and changes no rule a skill states

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pse-3 — save as docs/knowledge/patterns/proof-strength-and-expiry-pse-3-pitfall.md

---
type: bee.pattern
title: proof-strength-and-expiry cell pse-3 — pitfall candidate
description: "Pitfall candidate mined from cell pse-3's capped trace: Also rewrote two prose claims that the result segment is free text — the Edge cases bullet in docs/product-description/lifecycle/execution.md and the EXEC-10 p…"
timestamp: 2026-09-01
bee:
  id: proof-strength-and-expiry-pse-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/pse-3.json]
  polarity: pitfall
---

# proof-strength-and-expiry cell pse-3 — pitfall candidate

## What the cell did

Every teaching site now shows a qualified proof value and proof_gate.rs fences the set; packages/bee/prompts/worker-cell.md changed, so .bee/bin/bee must be rebuilt and reinstalled at merge or dispatch prepare --kind cell refuses every worker

## Recorded evidence (verbatim from .bee/cells/pse-3.json)

- **deviation** — Also rewrote two prose claims that the result segment is free text — the Edge cases bullet in docs/product-description/lifecycle/execution.md and the EXEC-10 probe row in docs/product-description/verification/lifecycle.md — because pse-1 made both statements false and both files were already in scope — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 3 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 3 pattern candidate(s), 0 file(s) written.