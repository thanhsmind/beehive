# wc-3 — Judge whether the proofless-cap story is already covered, then close only the gap

**Status:** [DONE]
**Outcome:** The coverage judgment ran first (CONTEXT D4) and found four of the five parts of
the net-behavior story already pinned — no rows authored against them. One gap was real and
smaller than it first looked: of the four registered `DEBT_DOORS`, only `scribing-run` had never
been crossed under any `gate_bypass` value. Closed with a single generated block (4 rows).

**Files touched:** `packages/bee/tests/test_bee_cli.mjs`,
`docs/history/codex-harness-hardening/release-manifest.json`, `.bee/onboarding.json`
**Commit:** `ae2d1528`
**Full trace / evidence:** `.bee/cells/wc-3.json`

## Step 1 — the coverage judgment (D4's first mandated step)

Every anchor below was opened and read directly, not taken from a digest.

| Part of the net-behavior story | Verdict | Anchor |
|---|---|---|
| A cap that recorded no proof is marked `trace.proof = "unrecorded"` | **covered** | `packages/bee/tests/test_cells.mjs:3200` — table-driven over every shape of absent output: `:3202` absent, `:3203` null, `:3204` empty string, `:3205` whitespace-only, `:3210` lane `spike` |
| …and the same through the real CLI, not a hand-written fixture | **covered** | `packages/bee/tests/test_bee_cli.mjs:3047` — a real cap stamps the marker and that cap holds the door shut; marker-independence asserted at `:3057` |
| A cap that recorded real proof is **not** marked | **covered** | `test_cells.mjs:3229` — one-line real output, D14's `verification_evidence`-with-empty-output case, and the explicit `--feature-verify-pending` path; mirror seam at `test_bee_cli.mjs:3079` |
| A `commands.verify: "none"` repo is never marked | **covered** | `test_cells.mjs:3278` |
| The marker buys no door bypass (red-first / `new_suite_reason` still refuse) | **covered** | `test_cells.mjs:3316` (refusals asserted at `:3322`, `:3333`, `:3342`) |
| Close-door reader 1 — `featureVerifyDebt` arms on the marker | **covered** | `test_bee_cli.mjs:3047` seam; generated matrix `:2830-2857` (4 doors × 2 kinds) |
| Close-door reader 2 — `testCellDebt` arms on the marker | **covered** | `test_bee_cli.mjs:3117` seam (feature-verify door satisfied on purpose so the refusal can only be the test-cell one), mirror `:3146`; branch pinned at `:2921` |
| Freshness runs over the union of `pending` and `unrecorded` caps | **covered** | `test_bee_cli.mjs:2859`, with the non-vacuous mirror at `:2886` |
| A new debt kind cannot ship without a fixture at every door | **covered** | `test_bee_cli.mjs:2739` (meta-check), structural check `:2767` |
| A dropped test cell owes nothing yet hides nothing | **covered** | `test_bee_cli.mjs:3214-3301` — 20 generated rows: `:3215` dropped falls through to the "missing" kind and is never named as an offender (`:3233`), `:3244` inert beside a green capped cell, `:3262` `open`/`claimed`/`blocked` still refuse as "not capped" |
| No bypass level passes any of it | **partly covered → the gap** | Covered for 3 of 4 doors, all hand-written: phase-departure `:2209` (pending) and `:2900` (unrecorded); feature-swap `:2274` (pending); start-feature `:2608` (pending) and `:2900` (unrecorded). **`scribing-run` was crossed by no bypass row at all.** |

**Rows authored against a "covered" line: zero.** That restraint is the point of D4, and of this cell.

## Step 2 — the gap, and only the gap

`packages/bee/tests/test_bee_cli.mjs` gained one generated block over `DEBT_DOORS` — four rows,
one per door, each carrying both halves:

- **refuses** an `unrecorded` cap under `gate_bypass: "total"`, with the refusal discriminated to
  the feature-verify door itself, naming the marker, the cell, and a runnable `FIX:`, and writing
  nothing (`door.untouched`);
- **still opens** under the same bypass level when nothing is owed, in its own repo — so a bug
  that welded every door shut under bypass cannot pass the first half.

Two scoping choices, both deliberate:

- **Generated, not hand-written for the one naked door.** The hole's shape is the one `gc-1`'s
  comment at `:2629` already records — a hand-written pair is how a P1 survived two rounds,
  because whoever writes the pairs writes them for the doors they have in hand. Generating means
  a door added later inherits the bypass question with nothing edited.
- **One marker, one kind.** The edit this defends against is a handler-level
  `if (bypassLevel(root) === 'total') return;` sitting *above* `guardFeatureDebt`
  (`packages/bee/lib/state.mjs:2782`), and above that seam a marker and a kind are
  indistinguishable. The wc-2 matrix already crosses the kinds below it; a doors × markers × kinds
  cube would buy rows, not coverage. The structural check at `:2767` cannot see this edit — it
  bans a door composing its own debt list, not a door returning early.

Each row asserts `bypassLevel(dir) === 'total'` before it knocks. Every hand-written bypass row
above shares one weakness: a silently-failed config seeding leaves the row exercising the
*no-bypass* path and passing anyway. These are the first rows that cannot rot that way.

## Verification

Real run, exit 0 — `.bee/logs/wc-3-verify.txt`, recorded on the cell (72,653 chars of output):

- `node packages/bee/tests/test_bee_cli.mjs` — **416 passed, 0 failed** (was 412; the four new rows)
- `node packages/bee/tests/test_cells.mjs` — **138 passed, 0 failed** (unchanged, untouched)
- `node scripts/release_manifest.mjs --check` — 448 file(s) match stored manifest

All four new rows named in the output, including the door that had never been asked:
`wc-3: door "scribing-run (state scribing-run)" REFUSES the unrecorded marker under gate_bypass
"total", and still OPENS under it when nothing is owed`.

Regen chain run in the cell's mandated order: `scripts/render_plugin_skill_trees.mjs` (100 files
per tree, byte-identical — no skill file was touched), `packages/bee/scripts/onboard_bee.mjs
--repo-root . --apply`, then `scripts/release_manifest.mjs --write`.

`packages/bee/lib` was not touched.

**A note the record should carry:** this cell capped with real recorded output, so capCell left
`trace.proof` unset. The cell that judged the marker is itself a live instance of the marker
behaving correctly.

## Consults

1 consult — advisor **fable** (`docs/history/worker-conformance/reports/wc-3-consult.md`;
recorded via `state advisor-ref record`).

- **Ask:** is part 5 a genuine hole or would per-door bypass rows duplicate a property already
  pinned, and are parts 1-4 graded too generously?
- **Answer:** genuine, but half the claimed size — the consult caught that `:2274` already crosses
  the **feature-swap** door under bypass total, which my inventory had missed, leaving only
  `scribing-run` naked. Endorsed the generated form on condition the block comment states the true
  provenance (three doors already covered piecemeal) and stays scoped to one marker. Flagged that
  my stated non-vacuity rationale was wrong and that a live-bypass assert is the check that
  actually bites. Parts 1-4 grades confirmed unchanged. All four points adopted.
