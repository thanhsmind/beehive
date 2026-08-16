---
type: bee.delivery
title: okf-foundation — Bee OKF Profile foundation — delivery
description: "Delivery record proposed by bee knowledge promote for work item okf-foundation: 9 capped cell(s), 4 recorded deviation(s)."
tags: [okf, knowledge-bundle, migration, high-risk]
timestamp: 2026-07-22
bee:
  id: okf-foundation-delivery
  lifecycle: active
  required_context: [work/okf-foundation/work-item.md]
  decisions: [D17-D38 active set (docs/history/okf-foundation/CONTEXT.md), "D29 (F1 proof area: docs/specs/advisor-protocol.md)", D30 (workflow-state.md decomposition locked as F2 input), D34 (ships list + guard propagation — ledger/manifest/plugin trees/session-close hook), "D35 (coverage report law: every numbered source anchor lands in exactly one concept)", D37 (pointer-stub anchor map — citations rewired in the same cell as the stub)]
  sources: [docs/knowledge/work/okf-foundation/work-item.md, "removed 2026-08-16: .bee/cells/okf-1.json through .bee/cells/okf-9.json no longer exist on disk (cell traces, pruned after the okf-foundation feature closed, 2026-07-22) — every line in this delivery record's What Shipped/Verify/Deviations sections was copied verbatim from those traces at promote time and is the surviving record"]
  lane: high-risk
---

# okf-foundation — Bee OKF Profile foundation — Delivery

## What shipped

- **okf-1** — OKF S1 format core: knowledge.mjs (emitter-first parser + concept model + two-level check with round-trip guard) wired as bee knowledge check in both lib roots, mirrors/renders/ledger/manifest refreshed, RED-first suite green, full 61-suite chain green (14 file(s) changed)
- **okf-2** — Seeded docs/knowledge/ bundle (index.md hand-authored with okf_version-only root frontmatter, log.md's first ISO-dated entry) and wrote the Bee OKF Profile area spec (docs/specs/okf-profile.md) documenting the nine D18 types + polarity, D19/D32 field rules, D31 authority rules, D33 carry-over map, D35/D37/D36/D11, and knowledge.mjs's exact finding codes; added the okf-profile row to reading-map.md. (4 file(s) changed)
- **okf-3** — knowledge check joins verify chain as chain-failing suite, pinned in mandatory manifest; red-side fixture proof captured and removed (3 file(s) changed)
- **okf-4** — bee knowledge index (byte-identical per-level generation, --check in verify chain) + bee knowledge list shipped; root docs/knowledge/index.md now generated; full 63-suite chain green (18 file(s) changed)
- **okf-5** — advisor-protocol migrated end-to-end: 4 bee.area concepts + pointer stub with 26-anchor map, okf_migrate coverage gate in chain (64 suites), session-close nudge scans docs/knowledge/ (16 file(s) changed)
- **okf-6** — critical-patterns.md migrated into 47 bee.pattern concepts (D35 coverage 47/47, --check-patterns extension); okf-foundation's own work-item.md + plan.md authored as concepts; okf-profile.md Templates section added with three canonical worked examples (round-trip proven); indexes regenerated; full chain green (65 suites) (58 file(s) changed)
- **okf-7** — bee knowledge context --work --budget shipped (D27): transitive required_context walk with silent cycle dedupe, critical + area-decision tiers, bytes/4 budget cut, ordered manifest of paths/sizes/reasons, never content — dogfooded on the real bundle (49 entries, 19781/20000 est tokens) (14 file(s) changed)
- **okf-8** — Startup bridge live: the session preamble now TELLS a session with a matching bee.work-item to run the knowledge-context command (pointer only, 3 lines, never the manifest); silence when no feature is active; exactly one author-one offer line when an active feature has no work item. AGENTS.md startup step 4 + bee-hive Session Scout carry the same rule. AGENTS.block.md 17562 -> 17821 B, AGENTS.md 17680 -> 17939 B (hard budget 20480). 65 suites green incl. test_agents_budget. (12 file(s) changed)
- **okf-9** — bee knowledge promote shipped (D38): finished work PROPOSES a delivery draft, area spec-sync bullets and pitfall candidates from capped cell traces, and writes nothing — proven by a 498-file byte-identical snapshot of docs/knowledge/ + .bee/cells/ across a real run (15 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **okf-1** — `node scripts/run_verify.mjs` — { "red_failure_evidence": "RED-FIRST: skills/bee-hive/templates/tests/test_knowledge.mjs written and run BEFORE skills/bee-hive/templates/lib/knowledge.mjs existed. Verbatim: Error [ERR_MODULE_NOT_FO…
- **okf-2** — `node .bee/bin/bee.mjs knowledge check --json && node scripts/run_verify.mjs`
- **okf-3** — `node scripts/run_verify.mjs` — { "before_behavior": "git show HEAD:scripts/run_verify.mjs confirms no knowledge-check entry existed in EXTRA_SUITES prior to this cell — an OKF error under docs/knowledge/ never affected the verify …
- **okf-4** — `node scripts/run_verify.mjs` — PASS run_verify: 63 suite(s), concurrency=5, wall=57742ms (second run: 63 suite(s), wall=57911ms) — incl. 'PASS .bee/bin/bee.mjs knowledge check' and 'PASS .bee/bin/bee.mjs knowledge index --check'; …
- **okf-5** — `node scripts/run_verify.mjs` — PASS run_verify: 64 suite(s), concurrency=5, wall=57270ms — incl. 'PASS scripts/okf_migrate.mjs --check advisor-protocol', 'PASS .bee/bin/bee.mjs knowledge check', 'PASS .bee/bin/bee.mjs knowledge in…
- **okf-6** — `node .bee/bin/bee.mjs knowledge check --json && node scripts/run_verify.mjs` — PASS run_verify: 65 suite(s), concurrency=5, wall=57442ms — incl. 'PASS scripts/okf_migrate.mjs --check advisor-protocol', 'PASS scripts/okf_migrate.mjs --check-patterns' (new suite this cell), 'PASS…
- **okf-7** — `node scripts/run_verify.mjs` — { "tests_inspected": [ "skills/bee-hive/templates/tests/test_knowledge.mjs (36 pre-existing checks over check/index/list)", "skills/bee-hive/templates/tests/test_bee_cli.mjs (registry-example conform…
- **okf-8** — `node scripts/run_verify.mjs` — { "red_failure_evidence": "Before this cell, buildSessionPreamble() never mentioned docs/knowledge at all: `bee knowledge context` shipped in okf-7 but nothing told a session it existed, so a fresh s…
- **okf-9** — `node scripts/run_verify.mjs` — PASS run_verify: 65 suite(s), concurrency=5, wall=58084ms — incl. 'PASS skills/bee-hive/templates/tests/test_knowledge.mjs' (53 checks, 10 of them the new promote suite), 'PASS skills/bee-hive/templa…

## Deviations

- **okf-1** — Edited skills/bee-hive/templates/tests/test_bee_cli.mjs (not in the declared files list, reserved before writing): the cell requires the new knowledge group to pass test_bee_cli conformance, and that suite hardcodes group allowlists, the DA5 bijection GROUP_NAMES probe list, and an every-entry-example-executed coverage check — added 'knowledge' to the three lists and one knowledge.check example execution. The mirrored copies under .claude/skills/, .agents/skills/, .claude-plugin/, .codex-plugin/ followed via onboard --apply + render (those trees are in the declared files list).
- **okf-9** — Edited skills/bee-hive/templates/tests/test_bee_cli.mjs (declared in the cell's files list) to add the knowledge.promote registry-example execution: that suite's 'every registry entry had its example executed at least once' bijection fails the moment a new registry entry lands without one. The example runs against the isolated rootKnowledge fixture repo, seeded with one capped cell trace, and asserts promote left both the fixture bundle file and the fixture cell trace byte-identical.
- **okf-9** — The cell's zero-write must-have was written as 'a byte-identical bundle snapshot before/after a real run'. The first version of the suite check snapshotted the ENTIRE fixture repo across a CLI run and went red on .bee/cache/manifest-hash.json — which promote does not write: the dispatcher writes it on every bee invocation. Proven, not assumed: a read-only 'knowledge check' in a virgin fixture repo produces exactly the same one-file delta. The check was therefore split into an in-process proof (buildPromotion, whole tree, zero delta of any kind) plus a CLI proof scoped to the bundle and the runtime store, with the dispatcher-cache delta asserted equal to check's. The real-repo 498-file sha256 proof over docs/knowledge/ + .bee/cells/ is byte-clean.
- **okf-9** — The okf-foundation work item declares no bee.areas, so the dogfood's AREA UPDATES section is legitimately empty. Implemented exactly as specified (areas come from the work item, never guessed — D10) and recorded as an Open Gap in docs/specs/okf-profile.md; the populated path is covered by fixtures in both test_knowledge.mjs and test_bee_cli.mjs.

## Provenance

Proposed by `bee knowledge promote --work okf-foundation` from 9 capped cell trace(s) in `.bee/cells/` and the work item `docs/knowledge/work/okf-foundation/work-item.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
