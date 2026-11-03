promote proposal for work item "playbook-file-split" (docs/history/playbook-file-split/CONTEXT.md + docs/history/playbook-file-split/plan.md) — 2 capped cell(s): pfs-1, pfs-2
anchor: history — docs/history/playbook-file-split/CONTEXT.md, docs/history/playbook-file-split/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/playbook-file-split/delivery.md

---
type: bee.delivery
title: playbook-file-split — delivery
description: "Delivery record proposed by bee knowledge promote for work item playbook-file-split: 2 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-25
bee:
  id: playbook-file-split-delivery
  lifecycle: active
  areas: [doctrine-layer, verify-pipeline]
  required_context: [docs/history/playbook-file-split/CONTEXT.md, docs/history/playbook-file-split/plan.md]
  sources: [docs/history/playbook-file-split/CONTEXT.md, docs/history/playbook-file-split/plan.md, .bee/cells/pfs-1.json, .bee/cells/pfs-2.json]
---

# playbook-file-split — Delivery

## What shipped

- **pfs-1** — Nine playbooks moved to skills/bee-planning/playbooks/<class>.md; planning-reference.md carries a short ## Playbooks link section; class_playbook_parity reads the directory; spike.md pointer fixed to name planning-reference.md (11 file(s) changed)
- **pfs-2** — Citers quote Playbooks; living docs name playbooks/; spike.md pointer fixed; regen clean (8 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **pfs-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test class_playbook_parity` — the one fence this cell rewrites (2 passed); verify narrowed by the leader post-block to exclude the full suite, which pfs-2 (dep) is responsible for greening
- **pfs-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test agents_block_render_parity --test route_class_parity && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the cell own verify in full: parity fences 13/13, manifest 403 files match, full suite 4279 passed 0 failed

## Deviations

- **pfs-1** — Changed spike.md step 3 from "named above" to name skills/bee-planning/references/planning-reference.md — the moved text now lives in its own file, so "above" pointed nowhere; leader-approved — the plan was wrong about a fact
- **pfs-1** — Kept the duplicate-name assertion in every_route_class_has_exactly_one_playbook — it still typechecks against Vec<String>, as the action allows — followed the plan
- **pfs-1** — Capped with --sync-ack — affects_skills predicted the directory skills/bee-planning and the sync door compares file paths — the plan was wrong about a fact
- **pfs-1** — sync-ack: affects_skills names the directory skills/bee-planning; every touched skills/ path sits inside it, so the prediction was right at directory level and only the matcher wants file paths
- **pfs-2** — Fixed skills/bee-planning/playbooks/spike.md:7 by dropping a doubled skills/ path segment — pfs-2's verify found that pointer_integrity was already red on base 433f6a7ce, a defect from pfs-1; the leader added the file to scope — something else had to be fixed first
- **pfs-2** — Left the "Class playbooks" hits in .bee/decisions.jsonl, .bee/backlog.jsonl and .bee/cells/archive/** unchanged — they are history records in the store, and only the CLI may change them; tracked docs and code outside the excluded trees have zero hits — the plan was wrong about a fact
- **pfs-2** — sync-ack: AGENTS.md change is only the Deep contracts pointer anchor text (Class playbooks -> Playbooks), regenerated from AGENTS.block.md; the capture-line rule text is untouched, so its applied_at files need no sync

## Provenance

Proposed by `bee knowledge promote --work playbook-file-split` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/playbook-file-split/CONTEXT.md`, `docs/history/playbook-file-split/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "playbook-file-split" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-09-25T00:09:35.021Z), the work item declares no bee.areas.

area doctrine-layer:
  - [pfs-1] Nine playbooks moved to skills/bee-planning/playbooks/<class>.md; planning-reference.md carries a short ## Playbooks link section; class_playbook_parity reads the directory; spike.md pointer fixed to name planning-reference.md — feature-wide sync per the scribing stamp, 11 file(s) changed (trace .bee/cells/pfs-1.json)

area verify-pipeline:
  - [pfs-1] Nine playbooks moved to skills/bee-planning/playbooks/<class>.md; planning-reference.md carries a short ## Playbooks link section; class_playbook_parity reads the directory; spike.md pointer fixed to name planning-reference.md — feature-wide sync per the scribing stamp, 11 file(s) changed (trace .bee/cells/pfs-1.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell pfs-1 — save as docs/knowledge/patterns/playbook-file-split-pfs-1-pitfall.md

---
type: bee.pattern
title: playbook-file-split cell pfs-1 — pitfall candidate
description: "Pitfall candidate mined from cell pfs-1's capped trace: Changed spike.md step 3 from \"named above\" to name skills/bee-planning/references/planning-reference.md — the moved text now lives in its own file, so \"above\" …"
timestamp: 2026-09-24
bee:
  id: playbook-file-split-pfs-1-pitfall
  lifecycle: draft
  areas: [doctrine-layer, verify-pipeline]
  sources: [.bee/cells/pfs-1.json]
  polarity: pitfall
---

# playbook-file-split cell pfs-1 — pitfall candidate

## What the cell did

Nine playbooks moved to skills/bee-planning/playbooks/<class>.md; planning-reference.md carries a short ## Playbooks link section; class_playbook_parity reads the directory; spike.md pointer fixed to name planning-reference.md

## Recorded evidence (verbatim from .bee/cells/pfs-1.json)

- **deviation** — Changed spike.md step 3 from "named above" to name skills/bee-planning/references/planning-reference.md — the moved text now lives in its own file, so "above" pointed nowhere; leader-approved — the plan was wrong about a fact
- **deviation** — Kept the duplicate-name assertion in every_route_class_has_exactly_one_playbook — it still typechecks against Vec<String>, as the action allows — followed the plan
- **deviation** — Capped with --sync-ack — affects_skills predicted the directory skills/bee-planning and the sync door compares file paths — the plan was wrong about a fact
- **deviation** — sync-ack: affects_skills names the directory skills/bee-planning; every touched skills/ path sits inside it, so the prediction was right at directory level and only the matcher wants file paths
- **failure_signature** — f7ef14177d0c

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell pfs-2 — save as docs/knowledge/patterns/playbook-file-split-pfs-2-pitfall.md

---
type: bee.pattern
title: playbook-file-split cell pfs-2 — pitfall candidate
description: "Pitfall candidate mined from cell pfs-2's capped trace: Fixed skills/bee-planning/playbooks/spike.md:7 by dropping a doubled skills/ path segment — pfs-2's verify found that pointer_integrity was already red on base…"
timestamp: 2026-09-25
bee:
  id: playbook-file-split-pfs-2-pitfall
  lifecycle: draft
  areas: [doctrine-layer, verify-pipeline]
  sources: [.bee/cells/pfs-2.json]
  polarity: pitfall
---

# playbook-file-split cell pfs-2 — pitfall candidate

## What the cell did

Citers quote Playbooks; living docs name playbooks/; spike.md pointer fixed; regen clean

## Recorded evidence (verbatim from .bee/cells/pfs-2.json)

- **deviation** — Fixed skills/bee-planning/playbooks/spike.md:7 by dropping a doubled skills/ path segment — pfs-2's verify found that pointer_integrity was already red on base 433f6a7ce, a defect from pfs-1; the leader added the file to scope — something else had to be fixed first
- **deviation** — Left the "Class playbooks" hits in .bee/decisions.jsonl, .bee/backlog.jsonl and .bee/cells/archive/** unchanged — they are history records in the store, and only the CLI may change them; tracked docs and code outside the excluded trees have zero hits — the plan was wrong about a fact
- **deviation** — sync-ack: AGENTS.md change is only the Deep contracts pointer anchor text (Class playbooks -> Playbooks), regenerated from AGENTS.block.md; the capture-line rule text is untouched, so its applied_at files need no sync
- **failure_signature** — 11b55439e30b

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 2 capped cell(s) mined, 1 delivery draft, 2 area bullet(s), 2 pattern candidate(s), 0 file(s) written.