# Budget Fence Removal — Context

**Feature slug:** budget-fence-removal
**Date:** 2026-07-29
**Exploring session:** complete (revised once after fresh-eyes review — 2 P1, 10 P2, 4 P3 all folded in)
**Scope:** Standard
**Domain types:** RUN (verify pipeline), READ (knowledge, decisions, instruction text)

## Feature Boundary

Abolish every enforced size threshold on **bee's own instruction text** — the blocking
`skill_budget_fence.mjs` suite, its budget baseline, the `AGENTS.md` byte assertions, and the
doctrine that made "shrink the number" a standing law — while keeping every check that guards
*meaning* rather than size. Ends when verify is green, no size threshold on instruction text
survives anywhere in the repo, and no doctrine anywhere instructs a reader to trim for size.

Out of scope, explicitly: size limits that are not about instruction text and are not this law —
the `status --brief --json` payload cap (`packages/bee/tests/test_bee_cli.mjs:928`), the
`knowledge context --budget` parameter, and the worker evidence-report budget. They are unrelated
and stay.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

Each decision states an *outcome*. The file-level mechanics that satisfy them live in
**Scope Inventory** below, which planning may correct against the repo without a new D-ID —
an anchor is evidence, not law.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A size ceiling on instruction text is never a standing law in bee. A diet is a deliberate one-off optimization event that leaves no permanent gate behind. | The user's governing principle: optimize for information, not for smallness. Every decision below derives from this one. Logged repo-wide as decision `8f63adb4`. |
| D2 | The byte fence is deleted outright — script and baseline both. No report-only survivor, no advisory remnant, no rename. | The user stated twice that the tool must not stay attached to bee. `wc -c skills/*/SKILL.md` covers any future ad-hoc measurement, and git history holds the file if it is ever wanted back. |
| D3 | The fence stops running: it leaves the verify suite list and its cache declaration. | Removing the suite also lapses the "narrow supersession" it claimed over decision `6d9b9afc`; that older law resumes governing unmodified, and any comment still asserting the supersession is now false. |
| D4 | The D8 provenance-exile **grep** dies with the fence. The exile **convention** survives: provenance stays in `references/provenance.md` and nothing is merged back into any skill body. | Exile earned its keep independently — references are cheap and bodies read better. Scoping the supersession to the enforcement half keeps the 14 existing `provenance.md` headers accurate instead of orphaning their citation. |
| D5 | In the AGENTS budget suite, every size threshold, size assertion, and size figure is removed, along with the diet doctrine written in its comments. Every meaning guard survives, still blocking, at the same file path. | The kept guards assert content, not size. The suite's path is pinned verbatim in `MANDATORY_SUITES` and keyed in the impact registry, so renaming it reds the manifest guard — it keeps its name even though the name is now imperfect. |
| D6 | Every standing instruction about how long a skill body may be is replaced by one information-density rule: a body line must change agent behavior; a line that does not belongs in `references/`. No number, no threshold, nobody measures it. This covers the byte-budget citations *and* the "<200 lines preferred" line heuristic beside them. | Encodes the user's principle directly. Keeps the useful half of the old law (default to `references/`) and drops the size half. A line ceiling is the same instinct as a byte ceiling and falls with it. |
| D7 | The knowledge concept describing the fence is deleted, not archived in place. | The knowledge bundle is the state layer: it describes what IS. A concept describing a deleted gate is false, not historical. History lives in `docs/history/skill-token-diet/` and in the supersession records. |
| D8 | Every `skill-token-diet` decision whose subject is the fence is superseded through `bee decisions supersede`, citing `8f63adb4` as the replacing rule. Decisions that merely *touch* the fence are superseded only in their fence-specific clause; their surviving clauses are restated, not dropped. | The originals stay discoverable and are still cited from live files; silent deletion would leave pointers to a law that no longer exists. |
| D9 | Stale numbered-rule pointers are repaired repo-wide in this feature, at the source of truth, as their own cell. The known set is enumerated with evidence in `reports/stale-rule-pointers.md`; that inventory is a floor, not a total — the cell re-runs discovery and reports the final count. | Same root cause: renumbering done under size pressure. The inventory is explicitly unproven-exhaustive because one of its 13 rows was found only while verifying the others — trusting the count would repeat the original error. |

### Agent's Discretion

- Exact wording of the D6 replacement rule, within the stated shape.
- How `test_verify_cache.mjs` case (10) is preserved (see Deferred To Planning).
- Commit split across cells, provided one commit per cell carries its cell id.
- Correcting any anchor in Scope Inventory that has drifted, without a new D-ID.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Size ceiling | Any enforced numeric threshold on the length of instruction text, in bytes or lines. What D1 abolishes. |
| Information density | Every body line changes agent behavior. The replacement standard — a judgment applied per edit, never a measurement and never gated. |
| Meaning guard | A check asserting that content is present, correct, or consistent between two artifacts (rule roster, byte-identical render, marker pair). Survives untouched. |
| Diet | A deliberate one-off compression pass. Legal as an activity; illegal as a standing rule. |

## Scope Inventory

Evidence for the decisions above. Verified 2026-07-29 by direct read; counts measured, not estimated.
Planning corrects drift here freely.

### D2 — delete

- `scripts/skill_budget_fence.mjs` — **471 lines**, 9 `export` statements, **imported by nothing**
  (`rg "import.*skill_budget_fence"` → zero code hits). Only ever spawned as a subprocess.
- `scripts/skill-body-budget.json` — the budget baseline.

### D3 — stop running it

- `scripts/run_verify.mjs:358-372` explanatory comment plus entries `:373-374`; the array closes at
  `:375`, so nothing after shifts. Suite count falls 118 → 116 against `SUITE_FLOOR_COUNT = 65`
  (`scripts/tests/test_verify_manifest.mjs:219`) — floor is safe. The fence is in neither
  `MANDATORY_SUITES` nor `MANDATORY_SUITE_ARGS`.
- `scripts/verify-cache-inputs.json:4` — an **opt-in declaration table that is hand-maintained, not
  generated**: `rg 'verify-cache-inputs' scripts packages` finds readers only
  (`scripts/run_verify.mjs:1081`, `scripts/tests/test_verify_cache.mjs:31,85`). Edit it directly.
- `scripts/impact-registry.json:3433-3442` — this one **is** generated, by
  `scripts/impact_registry.mjs --write`. Regenerate, never hand-edit.

### D4 — provenance

- Remove `skills/bee-writing-skills/references/provenance.md:14` (the fence's own row).
- The other 13 `provenance.md` files are untouched; their headers cite skill-token-diet D8, which
  D4 keeps valid by scoping the supersession to the grep.

### D5 — the AGENTS budget suite

`scripts/tests/test_agents_budget.mjs` — **236 lines**.

Delete: `HARD_FAIL_BYTES` / `WARN_BYTES` (`:42-43`), both threshold check blocks (`:91-103`,
`:105-116`), the now-unused `utf8Bytes` / `templateBytes` / `rootBytes` (`:75-77`, `:86-87` — read
only by the deleted checks and the summary), and **only the size line of the summary print,
`:233-235`**.

Keep, blocking: `:232` (the `passed`/`failed` summary line) and `:236`
(`if (failed > 0) process.exit(1);` — the suite's only red path); the marker-pair check
(`:120-130`), the byte-identical render check (`:132-145` — a drift guard comparing two files to
each other, not a budget), the 17-rule roster (`:188-191`, `EXPECTED_RULE_COUNT` at `:59`), its
negative control (`:193-213`), and the terminal-home-rules check (`:215-230`).

Rewrite, because it is diet doctrine living in code: the file header (`:2-8`, "AGENTS.md stays
under a ratcheted byte budget"), the 19-line ratchet rationale (`:23-41`), and the comment at
`:147-151` ("The byte fence above rewards cutting… These two checks are what make the budget safe
to enforce") — which states the *reason* the surviving guards exist in terms of a fence that will
no longer be there. The pointer error at `:46-47` is row 7 of D9's inventory.

### D6 — the standing length instructions

- `skills/bee-writing-skills/SKILL.md:41` (regrowth law, byte budget) and `:40`
  ("Body <200 lines preferred") — both fall under D6.
- `skills/bee-evolving/SKILL.md:90-92` ("Learning placement… within its recorded budget").
- `skills/bee-writing-skills/references/provenance.md:12` — the regrowth-law row, which documents
  the rule D6 rewrites and cites the decision D8 supersedes. Rewrite it alongside `:14`'s removal.

### D7 — knowledge

- `docs/knowledge/areas/verify-pipeline/skill-body-budget-fence.md` — **70 lines**. Declares no
  anchors; the only inbound link is the generated index.
- `docs/knowledge/areas/verify-pipeline/index.md:14` — carries a GENERATED header (`:1-7`).
  Refresh through `bee knowledge index`, never by hand.
- `docs/knowledge/areas/doctrine-layer/placement-and-anchoring.md:209-214` — states the deleted
  thresholds as live fact ("Standing-sheet size fence (R5)… `WARN_BYTES` 14000 and
  `HARD_FAIL_BYTES` 15000"). False the moment D5 lands; correct it by D7's own rationale.
- Knowledge coverage gate stays green: `node scripts/okf_migrate.mjs --check verify-pipeline`
  passes with 14 anchors / 14 owned, and the deleted concept owns none.

### D3/D2 fallout — stale comments

- `scripts/skill_lint.mjs:6-10` and `:60` document the fence as the live home of the body budget and
  assert the supersession D3 lapses. Both point at a deleted file after D2. Rewrite.

### D8 — supersession list

Each is a separate entry in `.bee/decisions.jsonl`:

| skill-token-diet | id | Treatment |
|---|---|---|
| D1 (byte budget ≤8192) | `c4c17668` | supersede wholesale |
| D2 (grandfather at current `wc -c`, ratchet) | `6d6c6a98` | supersede wholesale |
| D5 (one in, one out) | `5a1b3228` | supersede wholesale |
| D6 (baseline JSON + promote to blocking fence) | `f1c259c3` | supersede wholesale |
| D8 (provenance exile + grep) | `cb78ad77` | supersede the **grep/enforcement** clause only; record that the placement convention survives (D4) |
| D3 (source-byte measurement) | `4a247bb6` | supersede the fence-measurement clause only ("the fence measures source bytes… render-time stripping does not relieve the budget"); restate the surviving half |
| validation D1/D2 (baseline re-seed is a direct edit; `migrated` is an explicit array) | `27fb6302` | supersede wholesale — pure fence mechanics, and cited in the frontmatter of the concept D7 deletes |

### D9 — pointers

Full evidence table: `docs/history/budget-fence-removal/reports/stale-rule-pointers.md`. 13 rows
confirmed across `packages/bee/` (4), `scripts/` (3), `docs/knowledge/patterns/` (2),
`docs/decisions/` (4). Source of truth is `packages/bee/`; `.bee/bin/` is a byte-identical synced
copy (`cmp` clean) and clears on re-sync.

### Confirmed clean — no work needed

- `packages/bee/AGENTS.block.md` and `AGENTS.md` contain no budget, ratchet, `8192`, or byte text.
- `docs/knowledge/index.md` `## Critical patterns` and `docs/history/learnings/critical-patterns.md`
  never mention the fence, the diet, or the ratchet.
- No test besides `test_agents_budget.mjs` and `skill_budget_fence.mjs` asserts on the size of
  instruction text.
- No live release manifest lists the fence.

### Frozen — leave alone

`.bee/cells/*.json` (historical `verify` fields), `.bee/reviews/session-laws-20260728.json`,
`docs/history/**` reports. Point-in-time work logs; they record what was true when written.

## Established Patterns

- **Meaning-over-size guard**, already in the repo: `test_agents_budget.mjs:188-191` asserts all 17
  critical rules survive, with the message *"a diet may compress a rule's body, never drop the
  rule"* — plus a negative control at `:193-213` proving the guard bites. This is the model D1
  generalizes.
- `bee decisions supersede` retires a decision without erasing it.

## Canonical References

- `docs/history/skill-token-diet/CONTEXT.md` — the decisions D8 supersedes.
- `docs/decisions/index.md:191` — decision `b8ec25aa` already declared that no byte/token-budget
  hook would be built; the skill fence grew up beside that line.
- `scripts/skill_lint.mjs:1-10` — the advisory-only precedent, and the record of the fence's
  promotion out of it.

## Outstanding Questions

### Resolve Before Planning

None. All gray areas are locked above.

### Deferred To Planning

- [ ] **`scripts/tests/test_verify_cache.mjs` case (10) (`:304-340`) must be preserved, and the fix
      is larger than it first looks.** The case proves that a declared extra input invalidates a
      cache entry, and it deliberately rides the fence's *real* wiring: the fixture copies the real
      runner (`:62`) and the real declaration table (`:82`). `skill_budget_fence.mjs` does not match
      the `test_*.mjs` discovery glob, so `EXTRA_SUITES` membership is its only route into `SUITES`.
      Once D3 removes both entries, `--only skill_budget_fence` (`:311`) matches no suite and the
      case dies at `:319`-`:320`, before the declaration assertion at `:321` is ever reached.
      A synthetic declaration alone is therefore insufficient — the fixture would need its own
      `EXTRA_SUITES` entry too. Alternative: re-point the case at another really-declared suite that
      carries extra inputs. The behavior under test is worth keeping either way.
- [ ] Which regeneration commands, in which order, refresh `impact-registry.json`, the knowledge
      index, and the plugin skill mirrors after the deletions — and where `.bee/bin/` re-sync fits.

## Deferred Ideas

- A guard against stale numbered-rule pointers. This feature repairs the known ones by hand (D9);
  preventing the next drift is a separate, larger question — the roster check proves rules exist,
  not that pointers to them are correct.
- Rewriting `docs/decisions/0006` and `0007` to cite rule *content* rather than rule *numbers*, so
  renumbering can never break them again.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked decisions, Scope
Inventory, canonical references, and deferred-to-planning questions. Planning's Gate 2 shape stage
and reviewing use locked decisions for coverage and UAT.
