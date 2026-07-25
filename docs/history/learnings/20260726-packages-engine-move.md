---
date: 2026-07-26
feature: packages-engine-move
categories: [refactor, migration, validation, cli]
severity: mixed
tags: [engine-move, shared-contract, tautology-anchor, stderr-evidence, rg-exit-codes]
---

# packages-engine-move — learnings

## What Happened

The onboarding/distribution engine moved from `skills/bee-hive/scripts/` to `packages/bee/scripts/`, completing the packages vision: `packages/bee/` is the full standard code set (payload + engine), `skills/` is instruction-only. Host projections shrank to pure compliance mirrors (engine invoked from a projection-shaped root fails closed with `blocked_no_source`, test-pinned). Plus: every CLI verb now rejects unknown flags on stderr, naming the flag and the known list. Validation caught 8 BLOCKER + 10 CRITICAL before code; 3 cells landed clean, 105 suites green after each.

## Root Causes and Findings

1. **Plan ordered a shared contract changed instead of its callers.** `classifySource` has 5 callers across 2 geometries; only the engine's 2 call sites needed a different argument. The repair made `source-identity.mjs` a prohibited edit and the function survived untouched.
2. **The first proposed identity anchor was a tautology.** `realpath(PLUGIN_ROOT/packages/bee/scripts) === SCRIPTS_DIR` is always true because PLUGIN_ROOT is derived *from* SCRIPTS_DIR — a guard comparing a value against its own derivation can never fail. Re-anchored on independent evidence: skills tree exists under the package root and the payload is readable.
3. **One variable hid three semantics.** `HIVE_DIR` was engine geometry, classifySource input, AND the skills-root that sync copies. Renaming it wholesale would have made skill-sync walk `packages/`. The fix enumerated all 8 use lines by semantic before the move.
4. **A friction record was filed on truncated evidence.** "`capture add --text` silently exits 0" was wrong — it always exited 1; the orchestrator had piped stderr through `tail -1` and saw only the timing line. The real defect (error not naming the unknown flag) was fixed; the false premise was corrected in the decision log.
5. **rg exit-2 false-green recurred (second feature in a row).** `! rg ...` inverts both exit 1 (no match — intended pass) and exit 2 (rg itself failed) into success. Local fixes don't stop recurrence; this needs mechanizing (backlog row filed: cells add refuses `! rg`-shaped verify clauses).

## Recommendations

- **When X = a plan says "change a shared function's contract"**, do: count every caller and their geometries first; at ≥2 geometries, change the call sites and mark the shared function a prohibited edit.
- **When X = writing an integrity anchor/guard**, do: ensure the two sides of the comparison come from independent derivations — a check whose expected value is computed from the observed value is a no-op wearing a guard's name.
- **When X = renaming/moving something referenced through one variable**, do: classify every use site by semantic before touching the name; one name may be several concepts.
- **When X = filing a friction/bug record from CLI output**, do: capture full stderr and the real exit code first — never infer from piped/truncated output.
- **When X = an rg-based acceptance clause**, do: guard the exit code explicitly (`[ $? -eq 1 ]`); never `! rg`.

## Suite census (test-economy D4)

105 suites in registry (unchanged); cell 3 added 1 test block to `test_bee_cli.mjs` (296 asserts, +1); engine test suites moved homes without growth.
