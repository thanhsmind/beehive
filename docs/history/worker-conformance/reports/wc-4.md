# wc-4 — Stop demanding authored evidence at cap, keep every build-emitted proof

> Over the 40-line budget: high-risk validation-removal cell, full trace + a
> Consults section required.

**[DONE]** — worker Otto.

## Outcome

`capCell` no longer refuses a cap for failing to *author* evidence. Exactly two
throws became non-blocking recorded warnings (D1):

1. the `behavior_change`-without-`verification_evidence` door, and
2. decision 0004's "a passing verify flag but no recorded proof" door (small/
   standard/high-risk).

Both now record instead of refusing: the absence is stamped by wc-1's existing
`trace.proof = "unrecorded"` marker (no second marker invented), a warning is
appended to `trace.warnings` and written to stderr so the absence is visible in
the cap's own output, and the feature-boundary close-door — the sole blocking
proof (D3) — arms exactly as before. Proof was relocated, not removed.

Nothing else in the refusal chain moved. Verified by the advisor: throw count
inside `capCell` went 18 → 16, and every door downstream of the removed
behavior_change throw gained *reachability* (inputs it used to swallow now reach
them), which is the correct direction.

## Verify

Worker did not run the cell's `verify` — `verify_owner` is main at feature close;
capped on the feature-verify-pending path. Suites were run as ordinary
implementation feedback, not as cap evidence:

```
node packages/bee/tests/test_cells.mjs    -> 141 passed, 0 failed   (was 138)
node packages/bee/tests/test_bee_cli.mjs  -> 419 passed, 0 failed   (was 416)
node scripts/release_manifest.mjs --check -> 448 file(s) match stored manifest
```

Red-first "before": five pre-existing rows that encoded the two doors went red
the moment the throws were removed (`capCell refuses behavior_change without
verification_evidence`, `capCell refuses a small cell whose verify has no output
and no evidence (decision 0004)`, plus the two behavior_change-resolution rows
and one fixture-sequencing casualty). That red is the characterization of the
prior behavior; each row was **rewritten to the new law, never deleted**.

## D7 negative control (the point of the cell)

- `test_cells.mjs` — widened-paths cases array (behavior_change with no evidence
  at small/tiny; small, standard and high-risk with a verify recorded but no
  output), each asserting the cap succeeds, the marker where D14 says it should
  be, and the visible warning.
- `test_cells.mjs` — still-refused cases array: `security` (tiny/standard/
  high-risk), `migration` (tiny/small), `behavior`/`api`/`bugfix` at high-risk,
  `refactor` + new test file, `formatting` + new test file, `new_suite_reason`,
  the ratio ceiling at standard and high-risk, and the `files_changed` door D1
  deliberately kept (small and high-risk) — every one driven in exactly the
  shape the loosening created.
- `test_cells.mjs` — a `gate_bypass: "total"` repo proving three surviving
  refusals still bite: bypass self-approves gates, never proof.
- `test_bee_cli.mjs` — the seam wc-1's worker flagged, now closable: real
  `bee cells cap` runs at lanes small/standard/high-risk and for a
  `behavior_change` cell (all four shapes were refused outright before this
  cell), each asserting capCell stamped the marker itself and then that the real
  close-door refuses; plus a mirror proving caps holding real output stay
  unmarked and the door opens, so the pair cannot pass vacuously.

## Files + commit

- `packages/bee/lib/cells.mjs` (+ mirrored `.bee/bin/lib/cells.mjs`, byte-identical)
- `packages/bee/tests/test_cells.mjs`, `packages/bee/tests/test_bee_cli.mjs`
- `.bee/onboarding.json`, `docs/history/codex-harness-hardening/release-manifest.json`
- Regen chain run in the mandated order: skill-tree render → onboarding apply →
  release-manifest write (no skill text changed, so the render left no diff).

Full trace and evidence: `.bee/cells/wc-4.json`.

## Consults

1 consult — advisor **fable** (`advisor-consult wc-4: fable`).
Ask: did I loosen exactly two throws and nothing else; is there a shape where
both boundaries go silent; do the negative controls bite or pass for the wrong
reason; did the five rewritten rows lose coverage.
Answer: all five questions PASS — two throws exactly (18→16), no proof hole (the
predicate gap between the behavior_change door and D14's marker is real but
*correct*: verify output is build-emitted proof), every control bites under a
deletion test, resolution coverage non-vacuous in both directions. One P3 defect
raised and **fixed in this cell**: both warning texts unconditionally claimed the
close-door was armed, which is false where D14 correctly leaves the cap unmarked
(and in a `commands.verify: "none"` repo); texts now state only what holds on
every path. A stale `:2136` line reference in a new comment was also corrected.

## Deviations

1. Plan-vs-cell numbering drift (recorded, not acted on): `plan.md`'s slice queue
   lists `wc-4` as the trailing `change_class: 'test'` cell and `wc-3` as the
   D1 loosening; the cell record and the dispatch both assign the loosening to
   `wc-4`. Cell record followed — it contradicts no locked decision.
2. Five pre-existing test rows rewritten (see Verify). Required: they asserted
   refusals D1 removes. Each kept its guarded property, re-anchored on the
   observable that survives.
3. Two warning texts softened after the advisor consult (see Consults).

## Outstanding Questions

None blocking. One note for compounding: `bc`'s warning predicate keys on
`verification_evidence` alone while D14's marker keys on output-or-evidence, so a
`behavior_change` cap holding real verify output warns but is correctly not
marked. Intended per D14; worth a line in the knowledge sync so a future reader
does not read the warning as a missing marker.
