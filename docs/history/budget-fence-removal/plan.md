# Budget Fence Removal — Plan

**Feature:** budget-fence-removal
**Lane:** standard (class `refactor`, flags `covered-contract-change` / `proof-weakening` / `multi-domain`, 6 product files)
**Gate 1:** approved 2026-07-29 (bypass `total`, audit `69e5dee3`)
**Truth:** `docs/history/budget-fence-removal/CONTEXT.md` — decisions D1-D9, cited never reinterpreted
**Evidence:** `reports/stale-rule-pointers.md` — 13 verified pointer rows
**Revision:** rewritten once after the pre-code review wave (3 BLOCKER, 8 WARNING, 4 CRITICAL — all folded in)

## Approach

### Chosen path

Two slices. Slice 1 takes the fence out of the verify chain and leaves the chain green — this is
the user's actual ask, and it is done when it lands. Slice 2 cleans the trail the fence leaves in
doctrine, knowledge, decisions, and the numbered-rule pointers it damaged.

The split is not cosmetic: slice 1 is code under an active test chain and must stay green at every
step; slice 2 is instruction and knowledge text, which owes no test but does owe regeneration.
Mixing them would put a doctrine edit in the same commit as a suite-list change and make a red
ambiguous.

### Rejected alternatives

- **One cell for everything.** Rejected: spans seven file domains plus a repo-wide discovery sweep;
  unreviewable as one diff, and one-commit-per-cell traceability is lost exactly where this feature
  is about restoring traceability.
- **Defer all regeneration to a single tail cell.** Rejected: `REGEN_GUARDS`
  (`packages/bee/lib/cells.mjs:239-258`) refuses to write a cell whose declared files touch a
  manifest- or ledger-covered path unless that cell's own verify contains the guard's literal check
  string and its own files list the manifest. The obligation is per-cell by construction, enforced
  at `cells.mjs:388-393`.
- **Keep the fence as a report-only script.** Rejected at Gate 1 — locked by D2.
- **Drop `test_verify_cache.mjs` case (10).** Rejected: it is the sole owner of the assertion that
  editing a declared extra-input file, or adding a new glob-matched one, invalidates that suite's
  cache entry. Case (16) proves a different property (a corrupt declaration table fails closed).

### SMALLER PATH check

*Is there a cheaper shape that still honors every locked decision?*

**Yes, and it is taken.** The first draft carried seven cells, with knowledge cleanup and decision
supersession separate. They are both docs-layer, both no-code, and neither owes a test — merged into
one cell (BFR-6), six cells total.

Evidence that nothing cheaper survives: `rg -l 'skill_budget_fence|skill-body-budget|HARD_FAIL_BYTES|WARN_BYTES'`
spans `scripts/`, `skills/`, `docs/knowledge/`, and `docs/decisions/` — four trees with different
verify commands and different regen obligations, so no further merge leaves a runnable single verify.
D9's pointer work touches `packages/bee/`, a fifth tree, and is explicitly a discovery sweep rather
than an enumerated edit — it cannot be folded into a cell whose scope is a fixed file list.

### The regen obligation, measured

Derived from source, not assumed. `REGEN_GUARDS` has two entries (`packages/bee/lib/cells.mjs:239-258`):

| Guard | Fires when a cell's `files` touch | `verify` must contain, literally | `files` must list |
|---|---|---|---|
| manifest | `skills/`, `packages/bee/`, `.bee/bin/lib`, the two plugin skill trees, plugin/marketplace JSON, `scripts/install.{sh,ps1}`, `scripts/tests/test_verify_manifest.mjs`, `scripts/tests/test_release_tuple.mjs` (`scripts/release_manifest.mjs:43-65,138-143`) | `release_manifest.mjs --check` | `docs/history/codex-harness-hardening/release-manifest.json` |
| ledger | `.bee/bin/<group>` dirs (`cells.mjs:309-321`) | `ledger_parity.mjs --check` | — (`requiredFiles` is empty) |

The check is a literal substring test (`cells.mjs:388`), so a verify that merely *selects* those
suites through `--only` does not satisfy it. Both literals are spelled out in every cell below that
needs them.

Regen order, verbatim from `packages/bee/lib/cells.mjs:246`:

```
node scripts/render_plugin_skill_trees.mjs
node packages/bee/scripts/onboard_bee.mjs --repo-root . --apply
node scripts/release_manifest.mjs --write
```

`node scripts/impact_registry.mjs --write` runs after the file set is final — it scans the disk
(`scripts/impact_registry.mjs:418` imports `run_verify.mjs`'s live `SUITES`), so it must follow
deletions, never precede them.

### Risk map

| Risk | Mitigation |
|---|---|
| Deleting the fence silently un-guards something else | The removal is verified by invariants, not by names deleted (critical pattern 20260711). BFR-3 asserts the invariants directly. |
| `impact-registry.json` goes stale the moment the fence is deleted, reddening `test_impact_registry.mjs` | BFR-1 regenerates it in the same cell and verifies it. This was the review's first blocker. |
| A regen-covered cell is refused at `cells add` | Every such cell carries both guard literals in its verify and the manifest path in its files. |
| `test_verify_cache.mjs` case (10) dies unnoticed | Retarget decided with evidence, in BFR-1, same cell as the deletion that would break it — all six edit sites enumerated. |
| The 13-row pointer inventory is incomplete | It is declared a floor, not a total (D9). BFR-5 re-runs discovery with a stated command and reports the final count — the enumerated-move trap (critical pattern 20260712) is the named failure mode. |
| BFR-6's interpreter is overwritten mid-run | Slice 2 runs strictly serial. `onboard_bee.mjs --apply` re-vendors `.bee/bin/bee.mjs` itself, which BFR-6 executes. |
| A cell's verify string is wrong and only fails at the worker | Every `--only` form below was dry-run against `filterSuitesByOnly` + `SUITES` before this plan was finalized. |

### Files and order

Slice 1 — `scripts/skill_budget_fence.mjs` (delete), `scripts/skill-body-budget.json` (delete),
`scripts/run_verify.mjs`, `scripts/verify-cache-inputs.json`, `scripts/skill_lint.mjs`,
`scripts/tests/test_verify_cache.mjs`, `scripts/tests/test_agents_budget.mjs`,
`scripts/impact-registry.json`, `scripts/tests/test_instruction_size_law.mjs` (new).

Slice 2 — `skills/bee-writing-skills/SKILL.md`, `skills/bee-writing-skills/references/provenance.md`,
`skills/bee-evolving/SKILL.md`, `packages/bee/bee.mjs`, `packages/bee/lib/recovery.mjs`,
`docs/knowledge/areas/verify-pipeline/`, `docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md`,
`docs/knowledge/index.md` and area indexes (generated), `docs/knowledge/patterns/` (2 files),
`docs/decisions/0006`, `docs/decisions/0007`, `docs/decisions/index.md` (generated),
`docs/decisions/taxonomy.json` (may gain tag candidates),
`docs/history/codex-harness-hardening/release-manifest.json` (generated),
`.bee/decisions.jsonl` (through the CLI only).

## Slice 1 — the fence leaves the chain

Exit state: `node scripts/run_verify.mjs` green, and `node scripts/impact_registry.mjs --check`
green. No size threshold on instruction text survives anywhere in `scripts/`.

### BFR-1 — take the fence out of the verify chain

`change_class: behavior_change`. Implements D2, D3, D4's enforcement clause, and the D3/D2 fallout
row of Scope Inventory.

**D4's grep clause lands here, not in BFR-4.** The provenance-exile grep is code inside the deleted
file (`scripts/skill_budget_fence.mjs:169-170`), so deleting the file satisfies it. BFR-4 handles
only D4's provenance *row*.

- Delete `scripts/skill_budget_fence.mjs` (471 lines) and `scripts/skill-body-budget.json`.
- `scripts/run_verify.mjs` — remove the comment block `:358-372` and both entries `:373-374`. The
  array closes at `:375`; nothing after shifts.
- `scripts/verify-cache-inputs.json` — remove the `"scripts/skill_budget_fence.mjs"` key. Leaving it
  makes an orphan declaration matching nothing in `SUITES`.
- `scripts/skill_lint.mjs:6-10` and `:60` — rewrite. Both point at the deleted file, and `:8` asserts
  a "narrow supersession" of decision `6d9b9afc` that lapses with the fence. Wording is the worker's,
  provided it stops claiming a supersession and stops naming a deleted file.
- `node scripts/impact_registry.mjs --write` — **required in this cell.** The registry is derived
  from `run_verify.mjs`'s live `SUITES` (`scripts/impact_registry.mjs:418`) and currently carries a
  `scripts/skill_budget_fence.mjs` entry at `scripts/impact-registry.json:3433-3442`.
  `scripts/tests/test_impact_registry.mjs:174` asserts the committed file is byte-identical to a
  fresh build, and that suite is in the chain — skip the regen and slice 1 caps green over a red
  chain.

**Retarget `test_verify_cache.mjs` case (10) (`:304-340`) to `packages/bee/tests/test_misc.mjs`.**

Why it survives the swap: case (10) never runs the target's real logic — it overwrites the path with
`process.exit(0);\n` (`:314`). It needs exactly two things: a non-empty entry for that path in the
real `verify-cache-inputs.json`, and reachability by `--only` inside the fixture's byte-copied
`run_verify.mjs`. `packages/bee/tests/test_misc.mjs` is glob-discovered under `DISCOVERY_ROOTS`
(`run_verify.mjs:93-98`), so it needs no `EXTRA_SUITES` membership at all — which retires the exact
fragility that broke this case. Its declared inputs are
`["AGENTS.md", "packages/bee/AGENTS.block.md", "skills/**/*"]`
(`scripts/verify-cache-inputs.json:12-16`), so the existing mutations of `skills/demo/SKILL.md` and
`skills/second/SKILL.md` keep working unchanged against `skills/**/*`. `packages/bee/tests/` is
untouched by `makeFixture`, so nothing entangles with case (3)'s `helper.mjs`, and `writeFixtureFile`
creates parent dirs (`:268-273`).

**All six edit sites — edit every one or the case stays red:**

| # | Site | Change |
|---|---|---|
| 1 | `:311` `const FENCE = ["--only", "skill_budget_fence"]` | token → `test_misc` |
| 2 | `:312` `const CACHED_FENCE` regex | path → `packages/bee/tests/test_misc\.mjs` |
| 3 | `:313` `const RAN_FENCE` regex | same path |
| 4 | `:314` `writeFixtureFile(fx8, "scripts/skill_budget_fence.mjs", …)` | stub path → `packages/bee/tests/test_misc.mjs` |
| 5 | `:315` `writeFixtureFile(fx8, "scripts/skill-body-budget.json", …)` | declared input → `AGENTS.md` |
| 6 | `:321` `cacheEntries(fx8)["scripts/skill_budget_fence.mjs"]` | cache key → `packages/bee/tests/test_misc.mjs` |

Site 6 is the one an enumeration by eye misses; it fails as `undefined !== "green"`.

Also update the comments at `:76-81` and `:304-307`, which name the fence and state the deliberate
design intent ("not a synthetic stand-in"). Keep that intent — the case still rides a real
declaration — and name the new target.

**Verify:**
`node scripts/run_verify.mjs --only test_verify_cache,test_verify_manifest,test_impact_registry && node scripts/impact_registry.mjs --check`

### BFR-2 — strip size from the AGENTS budget suite

`change_class: behavior_change`. Implements D5. Independent of BFR-1 — different files, no dep,
runs in parallel.

`scripts/tests/test_agents_budget.mjs` (236 lines).

Delete: `HARD_FAIL_BYTES` / `WARN_BYTES` (`:42-43`); both threshold blocks (`:91-103`, `:105-116`);
`utf8Bytes` / `templateBytes` / `rootBytes` (`:75-77`, `:86-87`), read only by those blocks and the
summary; and **only** the size line of the summary print, `:233-235`.

Keep, blocking and unchanged: `:232` (the `passed`/`failed` line) and `:236`
(`if (failed > 0) process.exit(1);` — the suite's only red path); marker pair `:120-130`;
byte-identical render `:132-145`; roster `:188-191` with `EXPECTED_RULE_COUNT` at `:59`; its negative
control `:193-213`; terminal-home rules `:215-230`.

Rewrite, because it is diet doctrine living in code: the header `:2-8` ("AGENTS.md stays under a
ratcheted byte budget"), the 19-line ratchet rationale `:23-41`, and `:147-151` ("The byte fence
above rewards cutting… These two checks are what make the budget safe to enforce") — which explains
the surviving guards in terms of a fence that will no longer exist. Restate them as what they now
are: guards that no rule may be dropped and that the rendered block matches its template.
**Wording is the worker's** — CONTEXT.md's discretion clause is extended to cover these three comment
rewrites, provided no restatement reintroduces a size rule.

**Pointer row 7 lands here, not in BFR-5.** `:46-47` quotes hive law as `"Rules 2-4, 13 appear in
full in AGENTS.md"` and cites "its rule 13"; the live text (`skills/bee-hive/SKILL.md:108`) reads
"Rules 2-4, 12", and hive law 12 is the Guardrails pointer. BFR-2 is rewriting this exact comment
region anyway, and its edits to `:2-8` and `:23-41` shift every line below — leaving row 7 for BFR-5
would hand that cell a stale line number.

The file keeps its path. It is pinned verbatim in `MANDATORY_SUITES`
(`scripts/tests/test_verify_manifest.mjs:69`) and keyed in `scripts/impact-registry.json:3443`; a
rename reds the manifest guard. The name is now imperfect and stays anyway.

**Verify:** `node scripts/run_verify.mjs --only test_agents_budget,test_verify_manifest`

### BFR-3 — prove the removal by its invariants

`change_class: test`. `deps: [BFR-1, BFR-2]`. One trailing test cell for the slice.

New file: **`scripts/tests/test_instruction_size_law.mjs`**. It is auto-discovered under
`scripts/tests` (`run_verify.mjs:93-98`), so it needs no `EXTRA_SUITES` entry — the same property
that made `test_misc.mjs` the right retarget host.

Not a name-check. A removal is verified by its invariants, not the names it deletes (critical
pattern 20260711). Assert the net behavior of slice 1:

1. **No size law on instruction text survives in `scripts/`.** No `HARD_FAIL_BYTES`, no `WARN_BYTES`,
   no read of a per-skill byte baseline, no comparison of a `skills/**/SKILL.md` size against a
   recorded ceiling. Scope the assertion to the defect class, not to the two filenames just deleted
   (critical pattern 20260711 again).
2. **The meaning guards still bite.** Seed a violation and assert `test_agents_budget.mjs` exits
   non-zero — for the roster guard *and* for the byte-identical render guard. A guard that tests one
   state is a law with a hole (critical pattern 20260713).

   **Mechanism, so no worker has to guess:** `test_agents_budget.mjs:83-84` reads the real
   `TEMPLATE_PATH` and `ROOT_AGENTS_PATH`, so never mutate the live `AGENTS.md` — that would trip the
   write guard and critical rule 16. Follow the pattern the file already uses at `:193-213`, where
   the negative control builds fixture *text* in memory and runs the assertion helper against it.
   Confirm that control still runs after BFR-2's edit, and extend the same in-memory approach to the
   render guard.

Explicitly **not** in this cell: an orphan-declaration guard over `scripts/verify-cache-inputs.json`.
No decision D1-D9 authorizes new standing coverage, and CONTEXT.md already parks the adjacent
guard idea in Deferred Ideas. Filed to the backlog instead.

Explicitly **not** asserted: an exact `SUITES.length`. It would contradict the discovery design
(`run_verify.mjs:86-88`, "adding a new suite under one of these roots requires ZERO edits to this
file") and duplicate the existing floor at `test_verify_manifest.mjs:219`.

**Verify:**
`node scripts/tests/test_instruction_size_law.mjs && node scripts/run_verify.mjs --only test_instruction_size_law,test_agents_budget,test_verify_cache,test_verify_manifest`

The direct invocation is not redundant — it is the guard against a vacuous pass. Dry-run against
`filterSuitesByOnly` confirms that while the file does not exist, the `--only` form matches **three**
suites, not four, and reports green without ever running the new one. Invoking the path directly
fails loudly if the cell forgot to create it.

## Slice 2 — clean the trail

Exit state: no doctrine anywhere instructs a reader to trim for size; every numbered-rule pointer
resolves to the rule it names; full chain green.

Instruction and knowledge text owes no test cell (slice-tail batching rule), so slice 2 has none.

**Slice 2 is SERIAL: BFR-4 → BFR-5 → BFR-6.** Not a path-overlap technicality. BFR-4 and BFR-5 both
run `onboard_bee.mjs --repo-root . --apply`, which re-vendors `.bee/bin/bee.mjs` and its lib from
`packages/bee/` (`packages/bee/scripts/onboard_bee.mjs:1390-1391`). BFR-6 *executes* `.bee/bin/bee.mjs`
for `knowledge index`, `decisions supersede`, and `decisions render`. Running them concurrently
overwrites BFR-6's interpreter mid-run.

**Known shipped-state window, stated deliberately:** BFR-1 deletes `scripts/skill-body-budget.json`
while `skills/bee-writing-skills/SKILL.md:41` and `skills/bee-evolving/SKILL.md:92` still name it.
BFR-4 repairs them. No suite reds on the dangling reference, so this is a cosmetic window between
two commits, not a break — but it is real and is not to be discovered as a surprise.

### BFR-4 — replace the standing length instructions

`change_class: behavior_change` (doctrine). Implements D6, plus D4's provenance row.

- `skills/bee-writing-skills/SKILL.md:41` — replace the regrowth law's byte clause with the
  information-density rule: a body line must change agent behavior; a line that does not belongs in
  `references/`. Keep the "default to the knowledge bundle or `references/`" half.
- `skills/bee-writing-skills/SKILL.md:40` — "Body <200 lines preferred" falls under D6 too. A line
  ceiling is the same instinct as a byte ceiling.
- `skills/bee-evolving/SKILL.md:90-92` — same replacement for "Learning placement".
- `skills/bee-writing-skills/references/provenance.md:12` — rewrite the regrowth-law row (it
  documents the rule D6 replaces and cites a decision D8 supersedes); `:14` — remove the fence's own
  row.

Then the regen chain in order, then `node scripts/impact_registry.mjs --write`.

`files` must include `docs/history/codex-harness-hardening/release-manifest.json` or `cells add`
refuses this cell (`cells.mjs:391-393`).

**Verify:**
`node scripts/release_manifest.mjs --check && node scripts/ledger_parity.mjs --check && node scripts/okf_instructions_fence.mjs && node scripts/run_verify.mjs --only test_lib_mirror,test_render_race`

`okf_instructions_fence.mjs` (chain suite, `run_verify.mjs:356-357`) is the only guard over exactly
the surface this cell rewrites — skill bodies, hooks and `AGENTS.md`, line by line. The first two
commands are the literal strings `REGEN_GUARDS` requires.

### BFR-5 — repair every stale numbered-rule pointer

`change_class: behavior_change` (doctrine). Implements D9. `deps: [BFR-4]` — serial per the slice
rule above.

Evidence table: `reports/stale-rule-pointers.md`, 13 confirmed rows; **row 7 is already done by
BFR-2**, leaving 12 here. **Treat the table as a floor.** One row was found only while verifying its
neighbours, which proves the pass that produced the others was incomplete — the enumerated-move trap
(critical pattern 20260712).

Re-run discovery, do not work the table alone:

```
rg -n -i '\b(critical |priority |hive )?rules? [0-9]+\b' packages scripts skills docs/knowledge docs/decisions
```

A hit is a defect when the rule number it names does not match the rule content it describes, in
whichever of the two lists it means — critical rules (`packages/bee/AGENTS.block.md:35-52`, 17 rules)
or hive law (`skills/bee-hive/SKILL.md:106-123`, 14 rules). Both rosters are tabulated in the report.
**Report the final count**; do not confirm 12.

Excluded, deliberately: `docs/history/**`, `.bee/cells/**`, `.bee/reviews/**`, `.bee/decisions.jsonl`,
`.bee/backlog.jsonl` — frozen work logs that record the numbering as it stood when written.

Source of truth is `packages/bee/`; `.bee/bin/` is a byte-identical synced copy and clears on the
onboard re-sync. Never hand-edit `.bee/bin/`.

Also fix, same pass: both decision records cite `skills/bee-hive/templates/AGENTS.block.md`, a path
that no longer exists (now `packages/bee/AGENTS.block.md`). `docs/decisions/0006:23` cites
`skills/bee-hive/templates/lib/inject.mjs`, which resolves today to `packages/bee/lib/inject.mjs`.

Then the regen chain, then `node scripts/impact_registry.mjs --write`.

`files` must include `docs/history/codex-harness-hardening/release-manifest.json`.

**Verify:**
`node scripts/release_manifest.mjs --check && node scripts/ledger_parity.mjs --check && node scripts/run_verify.mjs && node scripts/impact_registry.mjs --check`

### BFR-6 — retire the fence's decisions and knowledge

`change_class: behavior_change` (docs). Implements D7 and D8. `deps: [BFR-5]` — serial, because this
cell's interpreter is what the two cells before it re-vendor.

Knowledge:

- Delete `docs/knowledge/areas/verify-pipeline/skill-body-budget-fence.md` (70 lines). It declares no
  anchors; the only inbound link is the generated index.
- `docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md:209-214` states the deleted
  thresholds as live fact ("Standing-sheet size fence (R5)… `WARN_BYTES` 14000 and `HARD_FAIL_BYTES`
  15000"). False the moment BFR-2 lands. Correct it.
- `node .bee/bin/bee.mjs knowledge index` — regenerates every area index **and the root
  `docs/knowledge/index.md`**. Never hand-edit either.

Decisions. **`bee decisions supersede` has no clause mode** — its parameters are
`{id, decision, rationale, tags, scope, json}`, and the superseded decision drops out of the active
set whole. So "clause only" is executed as: supersede wholesale, and **restate the surviving clause
verbatim inside the new `--decision` text**. There is no `--replaces`/`--cites` flag either; the
citation of `8f63adb4` lives in the `--rationale` prose. `docs/decisions/taxonomy.json` exists, so
every event needs at least one tag — omit `--tags` to inherit the target's, and pass tags explicitly
if a target carries none.

| skill-token-diet | id | Treatment |
|---|---|---|
| D1 (byte budget ≤8192) | `c4c17668` | wholesale |
| D2 (grandfather + ratchet) | `6d6c6a98` | wholesale |
| D5 (one in, one out) | `5a1b3228` | wholesale |
| D6 (baseline JSON + blocking fence) | `f1c259c3` | wholesale |
| validation D1/D2 (baseline re-seed, explicit `migrated`) | `27fb6302` | wholesale |
| D8 (provenance exile + grep) | `cb78ad77` | wholesale; the replacing text **restates the placement convention as surviving** (D4), so the 14 `provenance.md` headers stay accurate |
| D3 (source-byte measurement) | `4a247bb6` | wholesale; the replacing text **restates the surviving half** and drops only "the fence measures source bytes… render-time stripping does not relieve the budget" |

Then `node .bee/bin/bee.mjs decisions render` — `docs/decisions/index.md:1-4` declares itself
GENERATED from the store. No chain suite catches its staleness, which is exactly why it must be in
the cell.

**`decisions render --check` is already red before this cell starts**, and not because of anything
in this slice: the two decisions logged during this feature's own exploring and Gate 1
(`8f63adb4`, `69e5dee3`) left the generated index stale. Measured just now:

```
decisions render --check: docs/decisions/index.md is out of date — run `bee decisions render`
```

BFR-6's render clears that too. It is called out so the worker does not read the pre-existing red as
its own regression.

**Verify:**
`node scripts/run_verify.mjs --only knowledge && node scripts/okf_migrate.mjs --check doctrine-layer && node scripts/okf_migrate.mjs --check verify-pipeline && node .bee/bin/bee.mjs decisions render --check`

`--only knowledge` already runs `bee knowledge index --check`; `okf_migrate --check doctrine-layer`
is a separate chain suite (`run_verify.mjs:162`) carrying an F11 fidelity floor, and it is the gate
this cell's `placement-and-anchoring.md` edit can break.

## Outstanding Questions

None blocking. Both deferred-to-planning questions from `CONTEXT.md` are resolved above with
evidence: case (10) retargets to `packages/bee/tests/test_misc.mjs` (BFR-1, six edit sites), and the
regeneration order is the repo's own constant (`packages/bee/lib/cells.mjs:246`).

## Backlog filed from this plan

- Orphan-declaration guard: assert every key in `scripts/verify-cache-inputs.json` resolves to a
  suite in `SUITES`. Unauthorized by any locked decision here; the hole it would close is real.

## Handoff

Slice 1: BFR-1, BFR-2 (parallel), BFR-3 (trailing test, deps both).
Slice 2: BFR-4 → BFR-5 → BFR-6, strictly serial.
Current slice only — slice 2 cells are created after slice 1 caps.
