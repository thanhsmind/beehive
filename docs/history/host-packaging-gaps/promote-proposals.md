promote proposal for work item "host-packaging-gaps" (docs/history/host-packaging-gaps/CONTEXT.md + docs/history/host-packaging-gaps/plan.md) — 4 capped cell(s): hpg-1, hpg-2, hpg-3, hpg-4
anchor: history — docs/history/host-packaging-gaps/CONTEXT.md, docs/history/host-packaging-gaps/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/host-packaging-gaps/delivery.md

---
type: bee.delivery
title: host-packaging-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item host-packaging-gaps: 4 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-09-22
bee:
  id: host-packaging-gaps-delivery
  lifecycle: active
  required_context: [docs/history/host-packaging-gaps/CONTEXT.md, docs/history/host-packaging-gaps/plan.md]
  sources: [docs/history/host-packaging-gaps/CONTEXT.md, docs/history/host-packaging-gaps/plan.md, .bee/cells/hpg-1.json, .bee/cells/hpg-2.json, .bee/cells/hpg-3.json, .bee/cells/hpg-4.json]
---

# host-packaging-gaps — Delivery

## What shipped

- **hpg-1** — Pi doctor freshness reads .bee/onboarding.json in hosts with an installer remedy; Pi transport is ok with no multiplexer when every team.pi slot is a no-pane Pi agent (3 file(s) changed)
- **hpg-2** — Onboarding writes team.pi + herding.agents.pi, adds them once to existing configs, and accepts --runtime pi (7 file(s) changed)
- **hpg-3** — Installers accept runtime pi, and install.sh gains the macOS and ARM Linux asset map, a shasum fallback and a smoke-run fallback (5 file(s) changed)
- **hpg-4** — Release matrix builds five native targets with a sed version read; release.sh expects >= 5 binaries (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hpg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor` — 38 doctor tests pass incl. the 5 new ones by name and the unchanged pi_doctor_ready_case, missing_pane, absent_cases; run without the PATH prefix because cargo already resolves to ~/.cargo/bin/cargo;…
- **hpg-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` — cell verify: 195 onboard tests incl. 4 new pi-table tests plus the runtime-pi test and 3 extended template pins, 87 pi_plugin_contracts; also full --bin bee 3787 passed; other integration test target…
- **hpg-3** — `bash -n scripts/install.sh && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test installer_contracts && .bee/bin/bee dev release-manifest --check` — cell verify: 15/15 installer_contracts passed, manifest 376 files match; plus live refusal smoke (install.sh --runtime pi --distribution plugin-first exits 1 before any write) and a sha256_check fall…
- **hpg-4** — `bash -n scripts/release.sh && python3 -c "import yaml;d=yaml.safe_load(open('.github/workflows/release-binaries.yml'));print(len(d['jobs']['build']['strategy']['matrix']['include']))"` — cell verify printed 5 matrix rows; sed version read run locally returned 2.43.0; actionlint not installed so not run; the new runners have not run in CI yet

## Deviations

- **hpg-1** — host unknown-row detail now names both .bee/onboarding.json and the plugin manifest instead of only the manifest — a host reads both files now, so the old text would mislead — found a better route
- **hpg-2** — team.pi is built in default_config by mapping team.claude instead of an 18-entry literal — one source for names, order and descriptions, so the tables cannot drift — found a better route
- **hpg-2** — docs/product-description/verification/areas.md rows ONBD-05/ONBD-06 still quote the old runtime message; that file is outside this cell, left for the orchestrator — something else had to be fixed first
- **hpg-2** — notices closure in mod.rs now takes the plan or applied list instead of the extra-notices slice (the extra list is captured) — the pi notice reads the item list, and the old parameter carried the same value at both call sites — found a better route
- **hpg-3** — Fixed a literal backslash-n on the assert-recheck line of scripts/install.sh (it passed a stray 'n' argument instead of continuing the line) — found while editing the same file; harmless today because assert-recheck ignores args — something else had to be fixed first
- **hpg-3** — install.ps1 needed no probe/plugin-loop edits: its existing -in @('codex','both') / -notin @($rt,'both') checks already skip pi; only ValidateSet, refusal, the plugin-distribution call and the banner changed — the plan was wrong about a fact
- **hpg-4** — followed the plan
- **hpg-4** — header comment changed from 'Builds the two binaries' to 'Builds the binaries' — the count was no longer true — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work host-packaging-gaps` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/host-packaging-gaps/CONTEXT.md`, `docs/history/host-packaging-gaps/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

None: the work item declares no bee.areas, so there is no area to sync (D19).

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell hpg-1 — save as docs/knowledge/patterns/host-packaging-gaps-hpg-1-pitfall.md

---
type: bee.pattern
title: host-packaging-gaps cell hpg-1 — pitfall candidate
description: "Pitfall candidate mined from cell hpg-1's capped trace: host unknown-row detail now names both .bee/onboarding.json and the plugin manifest instead of only the manifest — a host reads both files now, so the old text…"
timestamp: 2026-09-22
bee:
  id: host-packaging-gaps-hpg-1-pitfall
  lifecycle: draft
  sources: [.bee/cells/hpg-1.json]
  polarity: pitfall
---

# host-packaging-gaps cell hpg-1 — pitfall candidate

## What the cell did

Pi doctor freshness reads .bee/onboarding.json in hosts with an installer remedy; Pi transport is ok with no multiplexer when every team.pi slot is a no-pane Pi agent

## Recorded evidence (verbatim from .bee/cells/hpg-1.json)

- **deviation** — host unknown-row detail now names both .bee/onboarding.json and the plugin manifest instead of only the manifest — a host reads both files now, so the old text would mislead — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hpg-2 — save as docs/knowledge/patterns/host-packaging-gaps-hpg-2-pitfall.md

---
type: bee.pattern
title: host-packaging-gaps cell hpg-2 — pitfall candidate
description: "Pitfall candidate mined from cell hpg-2's capped trace: team.pi is built in default_config by mapping team.claude instead of an 18-entry literal — one source for names, order and descriptions, so the tables cannot d…"
timestamp: 2026-09-22
bee:
  id: host-packaging-gaps-hpg-2-pitfall
  lifecycle: draft
  sources: [.bee/cells/hpg-2.json]
  polarity: pitfall
---

# host-packaging-gaps cell hpg-2 — pitfall candidate

## What the cell did

Onboarding writes team.pi + herding.agents.pi, adds them once to existing configs, and accepts --runtime pi

## Recorded evidence (verbatim from .bee/cells/hpg-2.json)

- **deviation** — team.pi is built in default_config by mapping team.claude instead of an 18-entry literal — one source for names, order and descriptions, so the tables cannot drift — found a better route
- **deviation** — docs/product-description/verification/areas.md rows ONBD-05/ONBD-06 still quote the old runtime message; that file is outside this cell, left for the orchestrator — something else had to be fixed first
- **deviation** — notices closure in mod.rs now takes the plan or applied list instead of the extra-notices slice (the extra list is captured) — the pi notice reads the item list, and the old parameter carried the same value at both call sites — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hpg-3 — save as docs/knowledge/patterns/host-packaging-gaps-hpg-3-pitfall.md

---
type: bee.pattern
title: host-packaging-gaps cell hpg-3 — pitfall candidate
description: "Pitfall candidate mined from cell hpg-3's capped trace: Fixed a literal backslash-n on the assert-recheck line of scripts/install.sh (it passed a stray 'n' argument instead of continuing the line) — found while edit…"
timestamp: 2026-09-22
bee:
  id: host-packaging-gaps-hpg-3-pitfall
  lifecycle: draft
  sources: [.bee/cells/hpg-3.json]
  polarity: pitfall
---

# host-packaging-gaps cell hpg-3 — pitfall candidate

## What the cell did

Installers accept runtime pi, and install.sh gains the macOS and ARM Linux asset map, a shasum fallback and a smoke-run fallback

## Recorded evidence (verbatim from .bee/cells/hpg-3.json)

- **deviation** — Fixed a literal backslash-n on the assert-recheck line of scripts/install.sh (it passed a stray 'n' argument instead of continuing the line) — found while editing the same file; harmless today because assert-recheck ignores args — something else had to be fixed first
- **deviation** — install.ps1 needed no probe/plugin-loop edits: its existing -in @('codex','both') / -notin @($rt,'both') checks already skip pi; only ValidateSet, refusal, the plugin-distribution call and the banner changed — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell hpg-4 — save as docs/knowledge/patterns/host-packaging-gaps-hpg-4-pitfall.md

---
type: bee.pattern
title: host-packaging-gaps cell hpg-4 — pitfall candidate
description: "Pitfall candidate mined from cell hpg-4's capped trace: followed the plan"
timestamp: 2026-09-22
bee:
  id: host-packaging-gaps-hpg-4-pitfall
  lifecycle: draft
  sources: [.bee/cells/hpg-4.json]
  polarity: pitfall
---

# host-packaging-gaps cell hpg-4 — pitfall candidate

## What the cell did

Release matrix builds five native targets with a sed version read; release.sh expects >= 5 binaries

## Recorded evidence (verbatim from .bee/cells/hpg-4.json)

- **deviation** — followed the plan
- **deviation** — header comment changed from 'Builds the two binaries' to 'Builds the binaries' — the count was no longer true — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 0 area bullet(s), 4 pattern candidate(s), 0 file(s) written.