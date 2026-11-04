---
type: bee.delivery
title: bee-deploying-skill — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-deploying-skill: 3 capped cell(s), 6 recorded deviation(s)."
timestamp: 2026-10-30
bee:
  id: bee-deploying-skill-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/bee-deploying-skill/CONTEXT.md, docs/history/bee-deploying-skill/plan.md]
  sources: [docs/history/bee-deploying-skill/CONTEXT.md, docs/history/bee-deploying-skill/plan.md, .bee/cells/dep-1.json, .bee/cells/dep-2.json, .bee/cells/dep-3.json]
---

# bee-deploying-skill — Delivery

## What shipped

- **dep-1** — bee-deploying skill, its record and script templates, and 5 trigger cases added (4 file(s) changed)
- **dep-2** — bee onboard --skillstation added end to end with a loud skip when the CLI is absent (6 file(s) changed)
- **dep-3** — Named --skillstation in all six onboard doc homes and added the two-ways install paragraph (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dep-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers` — the cell verify, run with the cargo binary at /home/thanhsmind/.cargo/bin; fixture completeness test passed, the paid real-agent eval is ignored and was not run
- **dep-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee onboard::` — the cell verify; 239 passed, 1 ignored, after the wave-close regen cleared the opencode projection drift
- **dep-3** — `rg -c -e --skillstation INSTALL.md README.md skills/bee-hive/references/onboarding.md docs/product-description/maintenance/onboarding.md docs/knowledge/areas/onboarding/host-project-artifacts.md .bee/verify/verify-app/features/onboard.md` — docs-only cell, the cell verify parity count is the proof (INSTALL.md:2 README.md:2 skills/bee-hive/references/onboarding.md:1 product onboarding.md:2 host-project-artifacts.md:2 onboard.md:2); no co…

## Deviations

- **dep-1** — The draft templates carried hash comment lines in the script and in the record code blocks; the script now holds bare placeholder lines and the record shows Preconditions and Verify as Command | Must show tables — the cell forbids comments beyond the shebang line — the plan was wrong about a fact
- **dep-1** — Committed with a path-scoped commit after an intent-to-add stage, not a plain add — the concurrent-worker guard refused staging into the shared index — hit an unforeseen obstacle
- **dep-2** — The apply stages a copy of the rendered skill under .bee/tmp/skillstation-publish and points the CLI at the copy instead of at .claude/skills/<name> — skillstation 0.9.0 skill commit --yes closes and unlinks the folder it is given, which would delete the per-project copy — the plan was wrong about a fact
- **dep-2** — The paseo worker hit the 1800s ceiling before capping; the leader stripped the comments it had written, re-ran the proof, committed and capped on its behalf — hit an unforeseen obstacle
- **dep-3** — Put the INSTALL.md flag row in the bee onboard Flags table, not the install-script table with the PowerShell column, and the two-ways paragraph says the install scripts do not pass the flag yet — scripts/install.sh:165 refuses unknown options, so a bash-column row would be false — the plan was wrong about a fact
- **dep-3** — README Useful flags sentence names --skillstation as bee onboard only, for the same reason — the plan was wrong about a fact

## Provenance

Proposed by `bee knowledge promote --work bee-deploying-skill` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/bee-deploying-skill/CONTEXT.md`, `docs/history/bee-deploying-skill/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
