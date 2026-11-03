---
type: bee.delivery
title: playbook-file-split — delivery
description: "Delivery record proposed by bee knowledge promote for work item playbook-file-split: 2 capped cell(s), 7 recorded deviation(s)."
timestamp: 2026-09-25
bee:
  id: playbook-file-split-delivery
  lifecycle: active
  required_context: [docs/history/playbook-file-split/CONTEXT.md]
  sources: [docs/history/playbook-file-split/CONTEXT.md, .bee/cells/pfs-1.json, .bee/cells/pfs-2.json]
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

Proposed by `bee knowledge promote --work playbook-file-split` from 2 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/playbook-file-split/CONTEXT.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
