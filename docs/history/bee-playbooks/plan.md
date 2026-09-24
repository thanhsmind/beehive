---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: bee playbooks

## Summary

bee already has a "playbook" system — a named step list plus a proof rule
for each kind of work (bugfix, feature, refactor, docs, research, release,
spike, perf) — built during `pstack-adoption`. It just isn't named where a
person would look first. This plan: (1) names it, in one sentence, in the
one file every session reads first; (2) writes down, once, which of
pstack's 22 original playbooks bee already covers (most of them, several
better) and which are real, small, worth-it gaps; (3) closes the one gap
cheap enough to fix on the spot; (4) adds a 9th playbook, `content`, so
bee's system can drive work that has no source code at all — proving the
extension the user actually asked for.

Mode: `standard` — 2 risk flags: public-contracts, covered-contract-change
Why this is the least workflow that protects the work: one new enum value
plus its pinned refusal message is the only code; everything else is prose
routed at four fenced single-homes an existing parity-test suite already
guards.

## Requirements (from CONTEXT.md)

- **D1** — No new `playbooks/` directory, no `.bee/playbooks/` vendoring.
  The single home stays `skills/bee-planning/references/planning-reference.md`
  § "Class playbooks".
- **D2** — A playbook is the existing shape (class name, steps, proof-line
  rule, citations); this feature names the CONCEPT, it does not move content.
- **D3** — No new `bee-hive` "Playbook match" section, no verbatim copying.
  The existing cite-by-anchor mechanism (`bee-planning/SKILL.md:44-46`)
  is unchanged.
- **D4** — A new class's playbook is one new `###` subsection under the
  existing "Class playbooks" heading, plus a reference-table row wherever
  the class is first surfaced — never a second file.
- **D5** — First slice: a gap-check research artifact against pstack's 22
  playbooks; close only genuine, cheap, code-repo-shaped gaps; add ONE new
  non-code class, `content`; everything else backlogged by name.
- **D6** — `content`'s proof-line rule is host-agnostic: never names a
  specific plugin skill, since `planning-reference.md` is vendored to
  every bee host repo.

## Load-bearing claims

Labels are `read`, `ran`, or `guessed`. Evidence is a verbatim byte
substring of the anchored line(s); multi-line evidence joins lines with
`" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `ROUTE_CLASS_VALUES` is an 8-element Rust constant whose arity is in its type, so a 9th value changes the type too | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356-357` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 8] = / ["feature", "bugfix", "docs", "refactor", "research", "release", "spike", "perf"];` |
| 2 | A test pins the FULL refusal message listing every class value, so this is a covered-contract change | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1903` | `"route --set: invalid flag(s): --class \"nope\" (must be one of feature, bugfix, docs, refactor, research, release, spike, perf)"` |
| 3 | A second test hardcodes the arity and the last index by literal value, not by reading the constant | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1958-1959` | `assert_eq!(ROUTE_CLASS_VALUES.len(), 8); / assert_eq!(ROUTE_CLASS_VALUES[7], "perf");` |
| 4 | That test's own worked pattern for adding a class end-to-end is directly reusable for `content` | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1956-1998` | `fn route_set_accepts_perf_as_the_eighth_class()` |
| 5 | The class vocabulary is independently fenced in FOUR source documents by an existing test, not by this feature | read | `packages/bee-rs/crates/bee/tests/route_class_parity.rs:53-58` | `("skills/bee-hive/references/scout-and-ticks.md", "`class` ∈"), / ("docs/product-description/goal.md", "Route vocabularies: class"), / ("docs/product-description/lifecycle/planning.md", "`class` from"), / ("docs/product-description/verification/lifecycle.md", "`class` is a closed enum:"),` |
| 6 | Site 1's current text, to be extended with `content` | read | `skills/bee-hive/references/scout-and-ticks.md:34` | `` - `class` ∈ `feature`, `bugfix`, `docs`, `refactor`, `research`, `release`, `spike`, `perf` `` |
| 7 | Site 2's current text | read | `docs/product-description/goal.md:48` | `` - Route vocabularies: class `feature\|bugfix\|docs\|refactor\|research\|release\|spike\|perf`; `` |
| 8 | Site 3's current text | read | `docs/product-description/lifecycle/planning.md:35` | `` `class` from `feature\|bugfix\|docs\|refactor\|research\|release\|spike\|perf`; `` |
| 9 | Site 4's current text — the one that also runs as an executable verification row | read | `docs/product-description/verification/lifecycle.md:115` | `` `class` is a closed enum: `feature bugfix docs refactor research release spike perf` `` |
| 10 | A second, independent fence requires exactly one `### <class>` section per `ROUTE_CLASS_VALUES` entry under "Class playbooks" — both directions | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:17-19` | `//   1. Every value in `ROUTE_CLASS_VALUES` has exactly one `### <class>` / //      section under `## Class playbooks`. / //   2. Every `### ` section under that heading names a real class value.` |
| 11 | A third fence only requires a `classes:` line's VALUES to be real classes — it does NOT require every class to appear in some principle's `classes:` line, so `content` needs no principle-routing row to pass CI | read | `packages/bee-rs/crates/bee/tests/principle_index_parity.rs:17-19` | `//   2. Every value on a row's `classes:` line is a real route class, read from / `      `ROUTE_CLASS_VALUES` in `workflows.rs`. A class value nobody routes is a / `      row `bee orient` can never select.` |
| 12 | All 8 existing class playbook bodies are complete, not stubs — the perf body ends with its own proof-line rule | read | `skills/bee-planning/references/planning-reference.md:244-262` | `Proof line: the baseline, the after, the delta, the command, and the / artifact path.` |
| 13 | The perf body's sustained-target subsection has a stop-rule discipline but never names the plateau case | read | `skills/bee-planning/references/planning-reference.md:253-257` | `Set the stop rule before the first attempt: the target reached, plus a / minimum number of attempts. Log one row per attempt. Revert an attempt that / does not clear the noise. Never loosen the stop rule.` |
| 14 | `AGENTS.md` names zero of bee's routing mechanics as "playbook" anywhere in its own body — the actual gap | ran | `rg -n "playbook" AGENTS.md` | (no output — zero matches) |
| 15 | The word already lives correctly in three other places this feature must not duplicate: the citing skill, its reference doc, and the doctrine-layer knowledge concept | read | `skills/bee-planning/SKILL.md:44-46` | `That same step cites the matched class playbook into plan.md by name and / anchor — `references/planning-reference.md` ("Class playbooks") — never by / copying its steps.` |
| 16 | `AGENTS.md`'s own pointer-index section is the one place built for exactly this kind of one-clause addition | read | `AGENTS.md:350-362` | `## Deep contracts / / The full mechanics live in `skills/bee-hive/SKILL.md` and its / references, loaded when routing work: lanes and gate wording;` |
| 17 | The prior deep pstack study explicitly rejects a new directory and verbatim-copy playbooks, with the mechanism this feature must not reopen | read | `docs/history/research/pstack-xia.md:201-203` | `**Verbatim playbook todo-lists and per-reply principle citations** — collides / with bee's Direction of Truth (todo lists are projections) and its judgment / contract.` |
| 18 | `bee worktree prune` already covers pstack's Worktree-cleanup playbook, and does it automatically rather than as a manual audit | ran | `bee worktree prune --help` | `Sweep DEAD worktrees the merge path never reclaimed: every worktree with a live grant ... is removed only when its branch is fully merged` |
| 19 | The declared full test command, needed to know the final proof and that it already runs the two fences above | read | `.bee/config.json` (`commands.test`) | `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| 20 | `planning-reference.md` is a `skills/` file with generated plugin-tree copies, so touching it obliges one regen — but no OTHER planned cell also touches a `skills/` file, so no two cells contend for the shared regen manifest | read | `skills/bee-planning/references/planning-reference.md` (path itself, under `skills/`) | `skills/bee-planning/references/planning-reference.md` |
| 21 | A research artifact of this shape has a precedent frontmatter contract to follow | read | `docs/history/research/pstack-xia.md:1-6` | `--- / artifact_contract: bee-research/v1 / topic: pstack-xia / depth: deep / date: 2026-09-01 / ---` |

## Discovery

The class-bound playbook system already exists and is complete for all 8
classes (row 12) — this feature does not need to invent step lists, only
name the concept and extend it. The vocabulary gap is real but narrow: the
word is already correct in the citing skill and one knowledge doc (row 15);
it is simply absent from `AGENTS.md`, the one file loaded before anything
else (row 14), which already has a purpose-built pointer-index section
sized for exactly this addition (row 16).

The class vocabulary is guarded by TWO independent parity fences bee
already runs on every `commands.test` call: one pins the four documents
that spell it out (row 5, rows 6-9 give their current text), the other pins
that every class has exactly one playbook section (row 10). Adding
`content` correctly means: the 9th value in the constant (row 1), the two
hand-pinned assertions updated deliberately (rows 2-3, following the
worked pattern at row 4), the four doc sites extended (rows 6-9), and one
new `### content` section — at which point both fences pass by
construction, with no new test to author. Row 11 confirms `content` does
NOT also need a principle-routing row to stay green.

Running the gap-check gather against all 22 pstack playbooks (full digest
in the research artifact this plan's cell 3 writes) found: 14 of 22 already
covered as well or better by bee's existing class playbooks plus its skill
catalog (`bee-herding`, `bee-writing-skills`, HANDOFF+adopt, `bee worktree
prune` — row 18, "Phase plan vs epic map"); 5 are deliberate non-matches
bee's own prior research already named as skipped on purpose (Graphite
stacking, iOS-simulator cleanup, moodboard/reference-gathering, patch-id
stack mechanics); 2 are real but not cheap enough for this slice
(trace-forensics' artifact-parsing depth, eval's blinding protocol) and go
to the backlog by name; 1 is real and cheap (row 13 — Hillclimb's
plateau-means-pivot rule, missing from perf's sustained-target
subsection) and is closed in this slice since it is one sentence in a file
this plan already touches.

## Approach

**Recommended path.** Three cells, all disjoint in the files they touch, so
they run as one parallel wave (row 20 rules out the one shared-resource
risk `pstack-adoption` hit).

1. **Vocabulary** — one sentence added to `AGENTS.md`'s "Deep contracts"
   pointer list (row 16), naming the class-bound playbook procedure and
   pointing at its single home. Nothing moves; nothing else is touched,
   because the word is already correct everywhere else that matters
   (row 15).
2. **The `content` class** — the 9th `ROUTE_CLASS_VALUES` entry, its two
   hand-pinned test updates (rows 2-3), the four doc sites (rows 6-9), a
   new `### content` section in "Class playbooks" (satisfies the row-10
   fence), the row-13 plateau-pivot sentence added to the existing `###
   perf` section, and a migration note following the `perf` precedent's
   exact shape.
3. **The gap-check research artifact** — a new `docs/history/research/`
   file (row 21's contract), recording the per-playbook verdict for all 22,
   plus `bee backlog add` entries for the two deferred genuine gaps.

**Rejected alternatives.**
- Touching `bee-evolving`'s Gate A/B to add pstack's blind-judge protocol —
  rejected for this slice: a real methodology change to a skill outside
  this feature's four named surfaces, not a one-sentence top-up; named and
  backlogged instead (D5, split-never-shrink).
- A deeper research/trace-forensics playbook body — rejected: bee's own
  repo is a Rust CLI with no browser-trace or heap-snapshot tooling today;
  building the step list before a real need would be scope invented, not
  asked for.
- Visual parity as a new class or playbook — rejected: no design-parity
  surface exists in this repo today (`site/` is docs, not a component
  library); named in the artifact in case that changes later.
- Folding the vocabulary sentence into `bee-hive/SKILL.md`'s own body
  instead of `AGENTS.md` — rejected: `bee-hive/SKILL.md`'s References
  table already reaches `scout-and-ticks.md`, which already carries the
  definition (row 6); the file with the real, measured gap is `AGENTS.md`
  (row 14), so that is where the one sentence lands.

**Risk map.**

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| `ROUTE_CLASS_VALUES` arity change | LOW | the arity is in the type; a miscount fails to compile | cell 2 | `cargo test`, scoped first |
| The two hand-pinned assertions | MEDIUM | a deleted or stale assertion is proof-weakening, not a fix — both must be updated, never removed | cell 2 | the two fence tests plus `tests.rs`'s own unit test |
| Four-site doc parity | MEDIUM | one fence already guards this; missing a site fails loudly, not silently | cell 2 | `route_class_parity` test |
| Class-playbook section parity | MEDIUM | the second fence catches a missing or misnamed `### content` | cell 2 | `class_playbook_parity` test |
| `content`'s proof-line rule naming a host-specific skill | MEDIUM | D6 — would silently break in every other bee host repo | cell 2 | manual read at cap: no plugin-namespaced skill name appears in the new section |
| Regen drift on the touched skill file | LOW | only one cell touches `skills/`, so no cross-cell race (row 20) | cell 2 | `bee dev regen` run once, `release-manifest --check` |
| Gap-check verdicts read as overclaiming coverage | MEDIUM | a wrong "covered" verdict hides a real gap from a future reader | cell 3 | every verdict in the artifact carries its bee-side anchor, matching row 15-18's own pattern |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"The leader drafts cells and the synthesis."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow implementation fact is needed.","reason":"This role owns known-location extraction."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"The pstack-playbook-catalog gap check is a multi-file gather already run this session."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Standard lane below the hat-wave threshold this feature's size sits at; evidence gaps were closed inline during planning (this table)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Same as above — alternatives were already weighed and rejected inline (see Approach)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"No user-facing runtime surface changes; this is bee's own doc/CLI vocabulary."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Standard lane opens three seats only on hat-wave-eligible work; risk map above covers it inline."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Same as hat-risks."},
    {"stage":"implementation","classification":"required","role":"code","reason":"The `content` enum value and its two pinned Rust assertions."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"The two parity fences plus the worked route-set-accepts test pattern (row 4) prove cell 2."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"AGENTS.md, the four class-vocabulary sites, the new playbook body, the migration note, and the research artifact are all prose."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"The user explicitly requests independent review.","reason":"Review remains a separate user-invoked pass."},
    {"stage":"deployment","classification":"conditional","role":"deploy","condition":"The user asks for a release after the feature closes.","reason":"The configured deploy role publishes from main, waits for CI, and verifies assets."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No hat wave at this lane/size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended supervisor loop is needed."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"No competing whole designs; the approach is settled in this plan."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"Same as blind-lane-1."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"Same as blind-lane-1."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every planned job has a specific role."}
  ]
}
```

## Shape

One phase — the three cells are mutually independent (no cell's files
overlap another's, row 20 rules out the one shared-resource risk) and run
as a single parallel wave rather than a sequence. Forcing them into
separate phases would invent milestones that do not exist (the "Phase plan
vs epic map" rule this plan's own vocabulary points at).

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | All three cells, parallel: (a) the `AGENTS.md` vocabulary sentence; (b) the `content` class end to end — enum, two pinned assertions, four doc sites, new playbook section, the row-13 plateau-pivot line, migration note; (c) the gap-check research artifact plus backlog entries | Nothing here depends on anything else in this feature | `rg playbook AGENTS.md` finds it; `bee route --set --class content ...` is accepted and `--class nope` now lists `content`; both parity fences pass; the research artifact anchors every verdict to bee-side evidence; `bee backlog list` shows the two deferred gaps | The feature is capture-ready |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| bpb-1 | Name "playbook" in AGENTS.md's pointer index | `AGENTS.md` | — | `rg playbook AGENTS.md` finds a line pointing at the class-playbook home | `rg -c playbook AGENTS.md` is non-zero; `bee dev regen --check` still clean (AGENTS.md is not a regen'd skill file) |
| bpb-2 | Add `content` as the 9th route class with its playbook | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs`, `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs`, `skills/bee-hive/references/scout-and-ticks.md`, `docs/product-description/goal.md`, `docs/product-description/lifecycle/planning.md`, `docs/product-description/verification/lifecycle.md`, `skills/bee-planning/references/planning-reference.md`, `docs/history/bee-playbooks/migration-note.md` (new) | — | `bee route --set --class content --lane docs --flags "" --files 1` is accepted; `bee route --set --class nope ...` refuses naming `content` last; the "Class playbooks" section has a `### content` entry with a host-agnostic proof-line rule; perf's subsection names the plateau case | `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test route_class_parity --test class_playbook_parity --test principle_index_parity` scoped first, then the full declared suite; `bee dev regen && bee dev release-manifest --check` |
| bpb-3 | Write the pstack-playbook gap-check research artifact and backlog the deferred gaps | `docs/history/research/pstack-playbooks-gap-check.md` (new) | — | The artifact names a verdict and a bee-side anchor for all 22 pstack playbooks; `bee backlog list` carries the trace-forensics and eval-blinding entries | parity/pointer check: every anchor in the artifact opens (`ran`, spot-checked); `bee backlog list --json \| grep -c pstack-playbooks-gap-check` ≥ 2 |

```json
[
  {
    "id": "bpb-1",
    "feature": "bee-playbooks",
    "title": "Name playbook in AGENTS.md's pointer index",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["D2"],
    "files": ["AGENTS.md"],
    "read_first": ["AGENTS.md:350-362"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In AGENTS.md's `## Deep contracts` section, insert one clause into the existing pointer-index sentence (per D2). Current text at AGENTS.md:355-357: `bypass mode; § Progress ticks; § Judgment contract; § Goal-check\njudge tier; § Concurrency law in full; § Delegation contract;\n§ Hat wave (the plan-step consult, single home); § Blind lanes and\nconvergence; worktrees; plus the worker contract`. Insert, after `convergence;` and before `worktrees;`, the clause: `the class-bound playbook each route class cites (`bee-planning/references/planning-reference.md`, \"Class playbooks\");` so the sentence reads `...convergence; the class-bound playbook each route class cites (`bee-planning/references/planning-reference.md`, \"Class playbooks\"); worktrees; plus the worker contract...`. Do not touch any other line, section, or file. Do not restate the playbook's steps or format here — this is a pointer only, per D3 (no verbatim copying).",
    "must_haves": {
      "truths": ["AGENTS.md names the class-bound playbook procedure and points at planning-reference.md's \"Class playbooks\" section, in one clause"],
      "artifacts": [{"path": "AGENTS.md", "substantive": "Deep contracts section's pointer sentence gains the one playbook clause; no other line changes"}],
      "key_links": ["AGENTS.md § Deep contracts references skills/bee-planning/references/planning-reference.md (\"Class playbooks\")"],
      "prohibitions": ["No other AGENTS.md section changes", "No new file created", "No restatement of the playbook steps themselves in AGENTS.md"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false},
    "verify": "rg -n \"class-bound playbook\" AGENTS.md && rg -n \"## Class playbooks\" skills/bee-planning/references/planning-reference.md"
  },
  {
    "id": "bpb-2",
    "feature": "bee-playbooks",
    "title": "Add content as the 9th route class with its playbook",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D4", "D5", "D6"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "skills/bee-hive/references/scout-and-ticks.md",
      "docs/product-description/goal.md",
      "docs/product-description/lifecycle/planning.md",
      "docs/product-description/verification/lifecycle.md",
      "skills/bee-planning/references/planning-reference.md",
      "docs/history/bee-playbooks/migration-note.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356-368",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1903",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1951-1998",
      "packages/bee-rs/crates/bee/tests/route_class_parity.rs:53-58",
      "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:17-19",
      "skills/bee-planning/references/planning-reference.md:220-262",
      "docs/history/pstack-adoption/migration-note.md"
    ],
    "affects_skills": ["skills/bee-planning/SKILL.md", "skills/bee-hive/references/scout-and-ticks.md"],
    "affects_specs": [],
    "action": "Add `content` as the 9th ROUTE_CLASS_VALUES entry, following the exact `perf`-addition precedent (pstack-adoption D2, decision 1593e365; docs/history/pstack-adoption/migration-note.md is the template to mirror). Steps: (1) workflows.rs:356-357 — change `[&str; 8]` to `[&str; 9]` and append `, \"content\"` after `\"perf\"`. Update the safety-comment block immediately below (lines ~360-368) to note `content` is also absent from ROUTE_LANE_VALUES, so it cannot collide (per D6/row-7-style reasoning already in that comment). (2) tests.rs:1903 — update the pinned refusal-message assertion to end `...release, spike, perf, content)`. (3) tests.rs:1951-1998 — add a new test `route_set_accepts_content_as_the_ninth_class` immediately after `route_set_accepts_perf_as_the_eighth_class`, following its exact structure: assert `ROUTE_CLASS_VALUES.len() == 9` and `ROUTE_CLASS_VALUES[8] == \"content\"`, then the same start_default/validate_route_set_flags/resolve_mutation_target happy-path drive with `class=content`, and the same class-vs-lane refusal check (`--lane content` must refuse, listing all 6 lane values). (4) Append `, `content`` to the class list at each of the four CLASS_SITES, in the exact format each site already uses (comma/pipe/space per site — read_first shows each site's current line): skills/bee-hive/references/scout-and-ticks.md:34, docs/product-description/goal.md:48, docs/product-description/lifecycle/planning.md:35, docs/product-description/verification/lifecycle.md:115 (this last one also says 'all eight values, ending in perf' in its expected-refusal column two rows below the class line — update that prose to 'all nine values, ending in content' too, same file). (5) In skills/bee-planning/references/planning-reference.md, add a new `### content` subsection under `## Class playbooks`, placed after the existing `### spike` subsection (end of that section, ~line 391). Steps to write, per D2/D6 (host-agnostic, no plugin-skill names, no source-code/worktree assumptions): name the audience and the one claim or fact at risk first; draft; check the draft against the project's OWN style, fact, or brand rule if the project has documented one (cite it), otherwise do a documented second read naming what was checked; ship. Proof line (per D6, verbatim shape): 'the artifact checked against the project's own style/fact/brand rule when the project has one (name it), otherwise a documented second read naming what was checked — <check> — <result> — <scope reason>.' (6) In the same file's existing `### perf` subsection (~planning-reference.md:253-257, the 'When the ask is a sustained metric target' paragraph), add one sentence after 'Never loosen the stop rule.': 'A plateau is not a stop — it means pivot the hypothesis, not give up.' (7) Write docs/history/bee-playbooks/migration-note.md mirroring docs/history/pstack-adoption/migration-note.md's exact structure and headings, substituting content for perf, 9 for 8, and this feature's own decision id (D5) for D2/1593e365. (8) Run `bee dev regen` (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write, in that order) so docs/history/codex-harness-hardening/release-manifest.json picks up the touched skills/ files, then `bee dev release-manifest --check` to confirm clean.",
    "must_haves": {
      "truths": [
        "`bee route --set --class content --lane docs --flags \"\" --files 1` is accepted and `bee route --show` reads back class=content",
        "`bee route --set --class nope --lane docs --flags \"\" --files 1` refuses, and the message lists `content` last",
        "`bee route --set --class content --lane content ...` refuses naming content as an illegal lane value"
      ],
      "artifacts": [
        {"path": "skills/bee-planning/references/planning-reference.md", "substantive": "new ### content section with a full, host-agnostic proof-line rule; ### perf section gains the plateau-pivot sentence"},
        {"path": "docs/history/bee-playbooks/migration-note.md", "substantive": "migration note mirroring the perf precedent's structure for content as the 9th class"}
      ],
      "key_links": [
        "all four CLASS_SITES documents list content as the 9th/last class value",
        "route_class_parity and class_playbook_parity tests pass with 9 values"
      ],
      "prohibitions": [
        "content's proof-line rule must not name a specific plugin skill (e.g. marketing:brand-review or any `namespace:skill` form)",
        "no change to gates, worktrees, or the cell lifecycle code",
        "no deletion or weakening of the existing perf/bugfix/refactor/research/feature/docs/release/spike sections beyond the one added sentence in perf"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true},
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test route_class_parity --test class_playbook_parity --test principle_index_parity route_set_accepts_content_as_the_ninth_class route_set_accepts_perf_as_the_eighth_class && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check"
  },
  {
    "id": "bpb-3",
    "feature": "bee-playbooks",
    "title": "Write the pstack-playbook gap-check research artifact and backlog the deferred gaps",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["D5"],
    "files": ["docs/history/research/pstack-playbooks-gap-check.md"],
    "read_first": ["docs/history/research/pstack-xia.md:1-30"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Write docs/history/research/pstack-playbooks-gap-check.md, frontmatter `artifact_contract: bee-research/v1`, `topic: pstack-playbooks-gap-check`, `depth: standard`, today's date, following pstack-xia.md's section shape (Bottom Line / Findings / Inference). Body: one row per pstack poteto-mode playbook (all 22, listed in skills/poteto-mode/SKILL.md's own \"Playbooks\" section at the pinned commit already used by pstack-xia.md), each with: the playbook name, a one-line purpose, a verdict (one of: covered-stronger, covered-adequately, deliberate-non-match, genuine-gap-deferred, genuine-gap-closed), and a bee-side anchor (file:line or skill name) backing the verdict — never a verdict with no anchor. Use exactly these verdicts, derived this session from a full read of all 22 files against bee's class playbooks (planning-reference.md) and skill catalog: investigation=covered-adequately (research class, planning-reference.md:302-313); perf-issue=covered-adequately (perf class, planning-reference.md:244-262); hillclimb=genuine-gap-closed (the plateau-pivot sentence bpb-2 adds to planning-reference.md's perf section); runtime-forensics=covered-adequately (research class's live-runtime subsection, planning-reference.md:315-318); trace-forensics=genuine-gap-deferred (no artifact-parsing/sqlite/symbol-resolution depth exists; bee's own repo is a Rust CLI with no captured-profile tooling today); feature=covered-stronger (feature class, planning-reference.md:326-339, plus AGENTS.md's own dispatch mandate); refactoring=covered-stronger (refactor class, planning-reference.md:286-300); prototype=covered-stronger (spike class, planning-reference.md:371-390); visual-parity=deliberate-non-match (no design-parity surface in this repo); authoring-a-skill=covered-stronger (skills/bee-writing-skills/SKILL.md, its Iron Law and RED/GREEN/REFACTOR/VALIDATE cycle); eval=genuine-gap-deferred (skills/bee-evolving/SKILL.md exists but has no blind-judge protocol; a real methodology change, out of this slice); babysit=covered-adequately (skills/bee-herding/SKILL.md's route role); shipping=covered-adequately (bee-herding's merge role plus bee worktree merge's proof check); autonomous-run=covered-adequately (AGENTS.md \"Act. Don't ask.\" plus herding's dispatch loop); orchestrate=covered-adequately (bee-herding dispatch plus bee-swarming's worker-brief contract); autopilot-full=deliberate-non-match (no Graphite/owner-per-PR concept in bee); autopilot-stack=deliberate-non-match (no Graphite stacked-PR concept in bee); session-pickup=covered-stronger (AGENTS.md's HANDOFF.json plus `bee state handoff adopt`); pause-safely=covered-stronger (same HANDOFF.json mechanism, AGENTS.md \"Care for the session\"); multi-phase-plan=covered-adequately (planning-reference.md's \"Phase plan vs epic map\" section); worktree-cleanup=covered-stronger (`bee worktree prune`, automatic rather than manual); opening-a-pr=covered-adequately, by deliberate design difference (scout-and-ticks.md's ship_visibility draft-pr default is the opposite of pstack's never-draft rule — name this as a considered divergence, not a gap). After the table, add two `bee backlog add` calls (run them, do not just describe them) recording the two genuine-gap-deferred items: `.bee/bin/bee backlog add --type debt --severity P3 --layer eval-blind-judge --title \"pstack-playbooks-gap-check: bee-evolving needs a blind-judge protocol like pstack's eval playbook\" --feature bee-playbooks --detail \"pstack's eval playbook blinds the judge to model identity and strips eval vocabulary from candidate-visible paths; bee-evolving's Gate A/B has neither. See docs/history/research/pstack-playbooks-gap-check.md.\"` and `.bee/bin/bee backlog add --type debt --severity P3 --layer trace-forensics-depth --title \"pstack-playbooks-gap-check: no captured-artifact (sqlite/symbol-resolution) research depth for trace forensics\" --feature bee-playbooks --detail \"pstack's trace-forensics playbook parses a pre-captured profiling artifact into sqlite and resolves symbols; bee's research class has no equivalent depth. No current need in this Rust-CLI repo. See docs/history/research/pstack-playbooks-gap-check.md.\"`.",
    "must_haves": {
      "truths": ["The artifact lists all 22 pstack playbooks, each with exactly one verdict and one bee-side anchor", "bee backlog list carries the two new entries with layer values eval-blind-judge and trace-forensics-depth"],
      "artifacts": [{"path": "docs/history/research/pstack-playbooks-gap-check.md", "substantive": "22-row verdict table/list with anchors, bee-research/v1 frontmatter"}],
      "key_links": ["Each genuine-gap-deferred verdict in the artifact is backed by a matching bee backlog add row"],
      "prohibitions": ["No verdict recorded without a bee-side anchor or an explicit not-applicable reason", "No new class, playbook, or code change proposed inside this artifact itself — findings only"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false},
    "verify": "test -f docs/history/research/pstack-playbooks-gap-check.md && .bee/bin/bee backlog list --json | rg -c 'pstack-playbooks-gap-check'"
  }
]
```

## Test matrix

The triad, at its smallest demonstrating size.

- **Happy path** — `bee route --set --class content --lane docs --flags "" --files 1` is accepted and reads back `class=content`. A new unit test beside `route_set_accepts_perf_as_the_eighth_class` (row 4's pattern), asserting arity 9 and index 8.
- **Edge** — the two existing fence tests (`route_class_parity`, `class_playbook_parity`) now assert against 9 values instead of 8; no new fence is authored (row 10-11 already cover the shape).
- **Error** — `--class nope` still refuses with a typed message, now listing `content` last; the pinned assertion at `tests.rs:1903` is updated, never deleted.

Cell 1 and cell 3 change prose only; `commands.test`'s full run still proves
nothing else moved.

## Open Questions

(none — row 11 closed the one open question CONTEXT.md deferred to
planning about lane behavior: nothing in `workflows.rs`'s lane-vs-class
handling needs a change, since `content`'s LANE is chosen per-task at
`bee route --set --lane <l>` the same way every other class's is; a
content task with zero code files touched naturally routes to lane `docs`
or `tiny` under the existing rules, with no new logic.)

## Out of scope

- Marketing and sales as their own route-classes — CONTEXT.md Deferred
  Ideas; `bee backlog add` at capture.
- Image/video generation as a route-class or playbook — CONTEXT.md
  Deferred Ideas; `bee backlog add` at capture.
- `bee-evolving`'s blind-judge protocol (pstack's Eval playbook) — real
  gap, not cheap enough for this slice; backlogged by cell 3.
- Trace-forensics' artifact-parsing depth (sqlite load, symbol
  resolution) — real gap, no current need in a Rust-CLI repo with no
  captured-profile tooling; backlogged by cell 3.
- Visual parity as a class or playbook — no design-parity surface exists
  in this repo today.
- Any change to gates, worktrees, or the cell lifecycle.
