---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: playbook file split

## Summary

Each route class (`perf`, `bugfix`, `refactor`, `research`, `feature`, `docs`,
`release`, `spike`, `content`) currently has its playbook — its numbered
steps and proof-line rule — as one `###` subsection inside a single shared
file, `skills/bee-planning/references/planning-reference.md`. This plan
moves each class's playbook body into its own file under a new
`skills/bee-planning/playbooks/` directory, shrinks the old section to a
short pointer-and-index paragraph renamed `## Playbooks`, and updates every
place that names the old section so nothing still points at a heading that
no longer exists.

Mode: `standard` — 2 risk flags: public-contracts, covered-contract-change
Why this is the least workflow that protects the work: no new route class,
no new enum value, and no Rust production code changes — the only code
touched is one existing test file being reshaped to check the new file
layout instead of the old heading layout; everything else is prose moved
between files under a fence that already exists.

## Requirements (from CONTEXT.md)

- **D1** — One playbook = one file, at
  `skills/bee-planning/playbooks/<class>.md`, named exactly the class
  value. Supersedes bee-playbooks' D1/D2/D4 (decisions `c2aa0417`,
  `fe8ff6a5`, `ac42cc07`).
- **D2** — Clean break: `## Class playbooks` is deleted from
  `planning-reference.md` in the same change that adds the directory; every
  citer is updated in the same change; no transitional stub.
- **D3** — The citation mechanism is unchanged (bee-playbooks D3,
  `82331837`): a plan cites the playbook, never copies its steps. Only the
  anchor shape changes, from file+heading to file path.
- **D4** — Vendoring changes from copying one file to copying a directory;
  partially supersedes bee-playbooks D6 (`8fd45bbb`) only on the "one file"
  phrasing. No manifest code change needed — `bee dev release-manifest`
  walks `skills/**` recursively.
- **D5** — `class_playbook_parity.rs` is real code, not docs; it is
  rewritten to check one file per `ROUTE_CLASS_VALUES` entry, in both
  directions, over the new directory shape.
- **D6** — `AGENTS.md` is generated from `packages/bee/AGENTS.block.md`;
  edits land in the source, never the generated copy.

## Load-bearing claims

Labels are `read`, `ran`, or `guessed`. Evidence is a verbatim byte
substring of the anchored line(s); multi-line evidence joins lines with
`" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The `## Class playbooks` section runs from its heading to the next `## ` heading, `## Cell quality rules` — everything in between is in scope to move or rewrite | read | `skills/bee-planning/references/planning-reference.md:220,413` | `## Class playbooks` / `## Cell quality rules` |
| 2 | The 9 class headings sit at these exact lines, in this order, inside that span | read | `skills/bee-planning/references/planning-reference.md:244,265,287,303,327,342,356,372,393` | `### perf` / `### bugfix` / `### refactor` / `### research` / `### feature` / `### docs` / `### release` / `### spike` / `### content` |
| 3 | The section opens with a shared preamble (route-class-vs-change_class, then "How a plan uses one" citing D1/`132551fb`, then "Which principles a class routes") that is not itself any one class's playbook — it must survive the move, in one home, not duplicated into 9 files | read | `skills/bee-planning/references/planning-reference.md:220-242` | `Each playbook binds one **route class** — the value `bee route --set --class` / ... / **How a plan uses one (per D1, decision `132551fb`).** The plan **cites** its / class's playbook by name and anchor — this file, `("Class playbooks")`, plus / the playbook name` |
| 4 | The parity fence's two single homes are named as constants — changing the shape means changing these two constants and the function that reads them, not re-declaring the class list | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:34-40` | `const PLAYBOOKS_MD: &str = "skills/bee-planning/references/planning-reference.md"; / const PLAYBOOKS_HEADING: &str = "## Class playbooks";` |
| 5 | `playbook_names()` is the one function whose body must change from "read `### ` subsections under one heading" to "read directory entries" — both call sites (`every_route_class_has_exactly_one_playbook`, `every_playbook_names_a_real_route_class`) call it and need no other change | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:96-120` | `fn playbook_names() -> Vec<String> {` |
| 6 | `const_str_array` reads `ROUTE_CLASS_VALUES` from `workflows.rs` as text and is unaffected by this split — it stays exactly as written | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:56-92` | `fn const_str_array(src: &str, name: &str) -> Vec<String> {` |
| 7 | The AGENTS.md pointer sentence (rendered from AGENTS.block.md) is the one that names "the class-bound playbook" and must change its quoted anchor from "Class playbooks" to "Playbooks" | read | `packages/bee/AGENTS.block.md:350-351` | `convergence; the class-bound playbook each route class cites / (`bee-planning/references/planning-reference.md`, "Class playbooks");` |
| 8 | `bee-planning/SKILL.md`'s own citation-convention sentence names the same anchor and must change the same way | read | `skills/bee-planning/SKILL.md:44-46` | `anchor — `references/planning-reference.md` ("Class playbooks") — never by / copying its steps.` |
| 9 | `scout-and-ticks.md` names the same anchor on lines 35-36, but line 34 is machine-parsed by `route_class_parity.rs` and must NOT change | read | `skills/bee-hive/references/scout-and-ticks.md:34-36` | `- `class` ∈ `feature`, `bugfix`, `docs`, `refactor`, `research`, `release`, `spike`, `perf`, `content` / — each class has a playbook the plan cites by name and anchor, never copies: / `bee-planning/references/planning-reference.md` ("Class playbooks")` |
| 10 | `routing-and-contracts.md` has no "Class playbooks" citation — nothing there needs updating | ran | `rg -n "Class playbooks" skills/bee-hive/references/routing-and-contracts.md` | (no output — zero matches) |
| 11 | `edge-dimensions.md` has no "Class playbooks" citation — nothing there needs updating | ran | `rg -n "Class playbooks" skills/bee-planning/references/edge-dimensions.md` | (no output — zero matches) |
| 12 | `docs/02-architecture.md`'s repository-layout line is a living tree listing that names `planning-reference.md` and `edge-dimensions.md` as bee-planning's references — it should also name the new `playbooks/` directory | read | `docs/02-architecture.md:34` | `bee-planning/    SKILL.md + references/{planning-reference.md, edge-dimensions.md}` |
| 13 | `docs/04-skills-spec.md`'s bee-planning skill spec lists the same two reference files and should also name `playbooks/` | read | `docs/04-skills-spec.md:63` | `**References:** `planning-reference.md` (artifact templates, cell quality rules, epic-map vs phase-plan guidance), `edge-dimensions.md`` |
| 14 | `pointer_integrity.rs` scans `AGENTS.md`, `CLAUDE.md`, `packages/bee/AGENTS.block.md`, and the whole `skills/` tree, and refuses if a cited heading no longer exists in its target file — this is the fence that catches a missed citer | read | `packages/bee-rs/crates/bee/tests/pointer_integrity.rs:35-36` | (per gather digest: scans `AGENTS.md`, `CLAUDE.md`, `packages/bee/AGENTS.block.md` and the whole `skills` tree) |
| 15 | `route_class_parity.rs` reads only the `` `class` ∈ `` line for the class vocabulary and never touches the playbook heading — this feature does not risk that fence | read | `packages/bee-rs/crates/bee/tests/route_class_parity.rs:53-58` | (per gather digest: anchors on the `` "`class` ∈" `` line only) |
| 16 | `agents_block_render_parity.rs` compares `AGENTS.md` against `AGENTS.block.md` byte for byte — editing the source without running regen fails this fence | read | `packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs:43` | `const AGENTS_BLOCK: &str = "packages/bee/AGENTS.block.md";` |
| 17 | The declared full test command, needed for the final proof | read | `.bee/config.json` (`commands.test`) | `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| 18 | The 4 parity tests this feature touches or must keep green are all green on the unchanged tree — the refactor playbook's own step 2 ("prove that record green on the UNCHANGED tree"), applied before any edit | ran | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test class_playbook_parity --test route_class_parity --test pointer_integrity --test agents_block_render_parity` | `test result: ok. 2 passed` (class_playbook_parity) / `test result: ok. 10 passed` (pointer_integrity) / `test result: ok. 2 passed` (route_class_parity) — agents_block_render_parity's 1 test also passed in the same run |
| 19 | `bee dev release-manifest` walks `skills/**` recursively rather than naming files one by one, so the new `playbooks/` directory needs no manifest code change | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:339` | `the skills            skills/**  +  the two committed render trees` |
| 20 | No route class gains, loses, or renames in this feature — `ROUTE_CLASS_VALUES` in `workflows.rs` already carries 9 values today (unlike `bee-playbooks`, which added `content` as the 9th), and this plan's cells never touch this file | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356-357` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 9] = / ["feature", "bugfix", "docs", "refactor", "research", "release", "spike", "perf", "content"];` |

## Discovery

The class-bound playbook system is complete and correct today for all 9
classes (rows 1-3) — this feature does not invent or change any step, it
relocates them. Exactly three real citers name the anchor `("Class
playbooks")` and must change (rows 7-9); two more files searched came back
with zero hits (rows 10-11), ruling out a wider citer sweep. Two living docs
list bee-planning's reference files by name and should gain a `playbooks/`
mention for accuracy (rows 12-13), though no parity fence enforces those two
— they are a documentation-accuracy nicety, not a gate requirement.

The mechanical risk is narrower than it first looks: only ONE Rust file
changes (`class_playbook_parity.rs`, rows 4-6), and the other three parity
tests this feature's verify command touches either don't inspect the
playbook heading at all (`route_class_parity.rs`, row 15) or only check that
citations resolve and that `AGENTS.md` matches its generated source (rows
14, 16) — both of which this plan satisfies by updating every real citer
and running regen once, at the end, after all `skills/` edits land (row 19
confirms one regen call covers both cells' `skills/` writes; no manifest
code path needs touching).

The one real sequencing risk found: `class_playbook_parity.rs`'s OLD test
body hardcodes the OLD heading and section shape. If the content move
(new files, deleted section) landed in one cell and the test rewrite
landed in a separate, later cell, the suite would sit RED in between —
"never build on a red base" and the refactor playbook's own step 4 ("the
record stays green at every step") both rule that out. The fix is
structural, not a waiver: the content move and the fence rewrite are ONE
cell, so the record — the two `class_playbook_parity` tests — never
observes a moment where it is checking a shape that no longer exists.

## Approach

**Recommended path.** Two cells, sequential (the second depends on the
first landing, since its citation-heading rename must exist before any
citer can quote it and stay green under `pointer_integrity.rs`).

1. **Content move + fence rewrite** (one atomic cell, per Discovery above) —
   the 9 playbook bodies move verbatim into
   `skills/bee-planning/playbooks/<class>.md`; `planning-reference.md`'s
   `## Class playbooks` section shrinks to a `## Playbooks` section (the
   preserved preamble, updated citation wording, plus a link to each new
   file); `class_playbook_parity.rs` is rewritten to check the directory
   shape instead of the heading shape.
2. **Citer updates + regen** — the three real citers (rows 7-9) get their
   quoted anchor changed from `"Class playbooks"` to `"Playbooks"`; the two
   living docs (rows 12-13) gain a `playbooks/` mention; `bee dev regen`
   re-renders `AGENTS.md` from the now-edited `AGENTS.block.md`; `bee dev
   release-manifest --check` confirms the manifest is clean (row 19: no
   code change needed there, just confirmation).

**Rejected alternatives.**
- One single cell doing everything — rejected: it would bundle a pure
  content-and-test change together with pure prose-pointer changes across
  unrelated files, for no shared reason other than both being "part of this
  feature"; splitting by real dependency (row 1 cell must land before row 2
  cell's citations can resolve) keeps each cell's diff reviewable on its
  own terms, per the SMALLER PATH check — this is the cheaper shape that
  still honors D2's clean break, not a cheaper shape that skips it.
- Keeping a stub `## Class playbooks` heading in `planning-reference.md`
  that just points at the new directory — rejected per the user's own
  locked choice (CONTEXT.md D2, "Cắt luôn"): a transitional stub is exactly
  the compatibility surface the user chose not to keep.
- Giving the new `skills/bee-planning/playbooks/` directory its own
  `README.md` index, separate from `planning-reference.md`'s `##
  Playbooks` section — rejected for this slice: `planning-reference.md` is
  already the established single home every citer already points at for
  "how planning works"; adding a second index file would mean the
  directory's identity lives in two places. The renamed `## Playbooks`
  section IS the index (a link to each file), so no second file is needed.
  Left as Agent's Discretion in CONTEXT.md; this is the call, with reason.
- Splitting the 9 playbook-file creations into 9 separate cells — rejected:
  they are one mechanical, disjoint-by-construction move with one shared
  verify (the rewritten fence checks all 9 at once); nine cells would only
  add nine review boundaries around a move that either fully succeeds or
  fully fails together.

**Risk map.**

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| Content fidelity of the 9 moved bodies | MEDIUM | a step silently dropped or reworded during the move would be invisible until someone reads the file | pfs-1 | manual byte-diff at cap: each new file's numbered steps and proof line match the old section's verbatim text (rows 2-3 anchor the source) |
| `class_playbook_parity.rs` rewrite loses its two-directions guarantee | MEDIUM | the fence exists specifically so a class with no playbook fails LOUDLY (row 4's own header comment) — a sloppy rewrite could silently pass on an empty directory | pfs-1 | the two rewritten tests keep asserting both directions; `cargo test --test class_playbook_parity` green |
| A citer missed | MEDIUM | `pointer_integrity.rs` catches a citation to a heading that no longer exists, but only inside `skills/`, `AGENTS.md`, `CLAUDE.md`, `AGENTS.block.md` (row 14) — a citer in `docs/` outside the two named living docs would not be caught by any fence | pfs-2 | `cargo test --test pointer_integrity` green; rows 10-11 already ruled out the two most likely doc-tree citers by search |
| Regen drift | LOW | only `AGENTS.block.md` among pfs-2's files needs regen to reach `AGENTS.md`; row 19 confirms no manifest code path needs touching | pfs-2 | `bee dev regen && bee dev release-manifest --check` |
| A pointer citing the wrong new anchor name | LOW | `pointer_integrity.rs` checks the quoted heading text matches a real heading — a typo in "Playbooks" fails loudly, not silently | pfs-2 | same `pointer_integrity` run |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"The leader drafts cells and the synthesis."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow implementation fact is needed.","reason":"This role owns known-location extraction."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"This session already dispatched one multi-file gather for the claims table above."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Standard lane below the hat-wave threshold this feature's size sits at; evidence gaps were closed inline during planning (the claims table)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Alternatives were already weighed and rejected inline (see Approach) — CONTEXT.md's D1-D6 already fixed the one viable shape; there is no competing whole design left to explore."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"No user-facing runtime surface changes; this is bee's own doc/CLI internal structure."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Standard lane opens three seats only on hat-wave-eligible work; risk map above covers it inline."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Same as hat-risks."},
    {"stage":"implementation","classification":"required","role":"code","reason":"class_playbook_parity.rs is rewritten in pfs-1."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"The two parity fences plus pointer_integrity/agents_block_render_parity prove the two cells."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"The 9 playbook files, the reshaped planning-reference.md section, the 3 citer updates, and the 2 living-doc mentions are all prose."},
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

Two phases, strictly sequential — cell 2's own greenness (`pointer_integrity`
under the new anchor name) depends on cell 1's heading rename already having
landed (Discovery, "the one real sequencing risk").

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | pfs-1: the 9 playbook files exist; `planning-reference.md`'s section is renamed and shrunk; `class_playbook_parity.rs` checks the new shape | Nothing else in this feature can be verified green until the new shape exists | `ls skills/bee-planning/playbooks/` lists 9 files; `cargo test --test class_playbook_parity` passes against the new shape | Citers can now be updated to point at a real, existing anchor |
| 2 | pfs-2: the 3 real citers, the 2 living docs, `bee dev regen`, `bee dev release-manifest --check` | Depends on phase 1's renamed heading existing | `rg -n "Playbooks" AGENTS.md skills/bee-planning/SKILL.md skills/bee-hive/references/scout-and-ticks.md` finds the updated anchor in all three; `cargo test --test pointer_integrity --test agents_block_render_parity` passes | The feature is capture-ready |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pfs-1 | Move class playbooks into one file per class; rewrite the parity fence | `skills/bee-planning/playbooks/{perf,bugfix,refactor,research,feature,docs,release,spike,content}.md` (new), `skills/bee-planning/references/planning-reference.md`, `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs` | — | `ls skills/bee-planning/playbooks/` lists 9 files; `planning-reference.md` has a `## Playbooks` heading, not `## Class playbooks` | `cargo test --test class_playbook_parity` scoped first, then the full declared suite |
| pfs-2 | Point every citer at the new anchor; regen | `packages/bee/AGENTS.block.md`, `skills/bee-planning/SKILL.md`, `skills/bee-hive/references/scout-and-ticks.md`, `docs/02-architecture.md`, `docs/04-skills-spec.md`, `AGENTS.md` (generated), `docs/history/codex-harness-hardening/release-manifest.json` (generated) | pfs-1 | `rg -n "Playbooks" AGENTS.md` finds the updated pointer; no file still names `"Class playbooks"` anywhere in the repo | `cargo test --test pointer_integrity --test agents_block_render_parity --test route_class_parity` scoped first, then `bee dev regen && bee dev release-manifest --check`, then the full declared suite |

```json
[
  {
    "id": "pfs-1",
    "feature": "playbook-file-split",
    "title": "Move class playbooks into one file per class; rewrite the parity fence",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D5"],
    "files": [
      "skills/bee-planning/playbooks/perf.md",
      "skills/bee-planning/playbooks/bugfix.md",
      "skills/bee-planning/playbooks/refactor.md",
      "skills/bee-planning/playbooks/research.md",
      "skills/bee-planning/playbooks/feature.md",
      "skills/bee-planning/playbooks/docs.md",
      "skills/bee-planning/playbooks/release.md",
      "skills/bee-planning/playbooks/spike.md",
      "skills/bee-planning/playbooks/content.md",
      "skills/bee-planning/references/planning-reference.md",
      "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs"
    ],
    "read_first": [
      "skills/bee-planning/references/planning-reference.md:220-412",
      "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:1-175"
    ],
    "affects_skills": ["skills/bee-planning"],
    "affects_specs": [],
    "regen_obligation_ack": "Regen deferred to pfs-2 (this cell's dependent): pfs-2 also touches skills/ (AGENTS.block.md, SKILL.md, scout-and-ticks.md) and runs the full bee dev regen + release-manifest --check chain once, after both cells' skills/ edits have landed, per this plan's Discovery section (one regen call covers both cells' writes).",
    "action": "STEP 1 — create 9 new files under skills/bee-planning/playbooks/, one per route class. Each file's content is: a level-1 heading `# <class>` (the bare class name, e.g. `# perf`), a blank line, then the CURRENT body of that class's `### <class>` subsection in planning-reference.md copied VERBATIM (every numbered step, every bolded sub-paragraph, every proof-line paragraph) with no rewording — only the heading level changes (### becomes #, at the top of the new file; the old ### line itself is not copied, since the new file's own H1 replaces it). The 9 source spans in planning-reference.md are: perf 244-263, bugfix 265-285, refactor 287-301, research 303-325, feature 327-340, docs 342-354, release 356-370, spike 372-391, content 393-411 (each span runs from the line after its own ### heading to the line before the next ### or ## heading; copy through the final Proof line paragraph inclusive). STEP 2 — in planning-reference.md, delete the ENTIRE existing section from line 220 (`## Class playbooks`) through line 412 (the blank line before `## Cell quality rules` at 413), and replace it with the new, shorter '## Playbooks' section: the same route-class-vs-change_class paragraph, an added short list linking each of the 9 new files by class name, the 'How a plan uses one' paragraph updated to cite by file path instead of file+anchor (citing decision 132551fb for the citation rule and this feature's own decision bed1853c for the file location), and the unchanged 'Which principles a class routes' paragraph — full text supplied in this cell's must_haves.artifacts entry for planning-reference.md. Leave everything at and after `## Cell quality rules` untouched. STEP 3 — rewrite packages/bee-rs/crates/bee/tests/class_playbook_parity.rs: replace the two constants `PLAYBOOKS_MD` (line 37) and `PLAYBOOKS_HEADING` (line 40) with one constant `PLAYBOOKS_DIR: &str = \"skills/bee-planning/playbooks\";`. Replace the `playbook_names()` function (lines 96-120) with a function of the same name and return type that: resolves `repo_root().join(PLAYBOOKS_DIR)`, calls `std::fs::read_dir` on it (panicking with a FIX-style message naming PLAYBOOKS_DIR if the directory is missing or unreadable, mirroring the existing panic style in this file), collects every entry whose file name ends in `.md`, strips the `.md` suffix to get the class name, and returns the Vec<String> of names. Update every remaining panic message in the file that mentions `{PLAYBOOKS_HEADING:?}` or the `### <class>`-section wording to instead reference `{PLAYBOOKS_DIR:?}` and a `<class>.md` file, keeping the same FIX-style guidance shape (do not soften or shorten the guidance text, only swap the noun). The duplicate-name check inside `every_route_class_has_exactly_one_playbook` becomes structurally unreachable on a real filesystem (two files cannot share one name) — keep it only if it still typechecks cheaply against the new Vec<String>, otherwise delete exactly that duplicate-check block and its assertion, nothing else. Do not change `WORKFLOWS_RS`, `const_str_array`, or either `#[test]` function's name or its two assertions' structure — only what each assertion iterates over changes (directory entries instead of `### ` lines within one heading's span).",
    "must_haves": {
      "truths": [
        "skills/bee-planning/playbooks/ contains exactly 9 files: perf.md, bugfix.md, refactor.md, research.md, feature.md, docs.md, release.md, spike.md, content.md",
        "each new file's numbered steps and proof-line paragraph are byte-identical in substance to the old ### <class> section it replaced (only the heading level changed)",
        "planning-reference.md contains a '## Playbooks' heading and no longer contains a '## Class playbooks' heading anywhere",
        "class_playbook_parity.rs's two tests (every_route_class_has_exactly_one_playbook, every_playbook_names_a_real_route_class) still exist by name and both pass against the new directory shape"
      ],
      "artifacts": [
        {"path": "skills/bee-planning/playbooks/perf.md", "substantive": "the perf class's full step list and proof-line rule, verbatim"},
        {"path": "skills/bee-planning/references/planning-reference.md", "substantive": "old '## Class playbooks' section (lines 220-412) replaced by a '## Playbooks' section reading: route-class-vs-change_class paragraph (unchanged) + 'Every route class has one playbook file, at `skills/bee-planning/playbooks/<class>.md`:' + a markdown link list of all 9 classes to their files + 'How a plan uses one (per D1, decision `132551fb`; file location per playbook-file-split, decision `bed1853c`).' paragraph (cites by path, never transcribes steps, names 'named deviation is the system working') + the unchanged 'Which principles a class routes' paragraph"},
        {"path": "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs", "substantive": "PLAYBOOKS_DIR constant and a directory-reading playbook_names(), the two existing #[test] functions unchanged in name and both green"}
      ],
      "key_links": ["class_playbook_parity.rs's playbook_names() reads skills/bee-planning/playbooks/, not planning-reference.md"],
      "prohibitions": ["No step, sub-paragraph, or proof-line text dropped or reworded during the move", "No change to ROUTE_CLASS_VALUES or workflows.rs", "No change to any file outside this cell's files list", "class_playbook_parity.rs's two #[test] function names are unchanged"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true},
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test class_playbook_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml"
  },
  {
    "id": "pfs-2",
    "feature": "playbook-file-split",
    "title": "Point every citer at the new anchor; regen",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": ["pfs-1"],
    "decisions": ["D2", "D4", "D6"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-hive/references/scout-and-ticks.md",
      "docs/02-architecture.md",
      "docs/04-skills-spec.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "packages/bee/AGENTS.block.md:348-353",
      "skills/bee-planning/SKILL.md:42-47",
      "skills/bee-hive/references/scout-and-ticks.md:33-37",
      "docs/02-architecture.md:34",
      "docs/04-skills-spec.md:63"
    ],
    "affects_skills": ["skills/bee-planning", "skills/bee-hive"],
    "affects_specs": [],
    "action": "In each of the 3 real citers, change ONLY the quoted anchor text from `\"Class playbooks\"` to `\"Playbooks\"` — do not touch surrounding wording. (1) packages/bee/AGENTS.block.md:351 — current text `(`bee-planning/references/planning-reference.md`, \"Class playbooks\");` becomes `(`bee-planning/references/planning-reference.md`, \"Playbooks\");`. (2) skills/bee-planning/SKILL.md:45 — current text `anchor — `references/planning-reference.md` (\"Class playbooks\") — never by` becomes `anchor — `references/planning-reference.md` (\"Playbooks\") — never by`. (3) skills/bee-hive/references/scout-and-ticks.md:36 — current text `` `bee-planning/references/planning-reference.md` (\"Class playbooks\") `` becomes `` `bee-planning/references/planning-reference.md` (\"Playbooks\") ``. Do NOT touch scout-and-ticks.md line 34 (the `` `class` ∈ `` line — route_class_parity.rs parses it) or line 35. Then, in the two living docs, add a `playbooks/` mention beside the existing reference-file list: (4) docs/02-architecture.md:34 — append `+ playbooks/{perf,bugfix,refactor,research,feature,docs,release,spike,content}.md` into the existing `references/{...}` clause on that line, before the ` — also owns` clause, changing nothing else on the line. (5) docs/04-skills-spec.md:63 — append `, `playbooks/<class>.md` (one file per route class, per the \"Playbooks\" pointer)` to the end of the existing **References:** sentence, before its final period, changing nothing else on the line. Then run, in order: `.bee/bin/bee dev regen` (renders AGENTS.md from AGENTS.block.md, re-renders any mirrored skill trees, and updates docs/history/codex-harness-hardening/release-manifest.json), then `.bee/bin/bee dev release-manifest --check` to confirm it is clean. Finally, run `rg -n \"Class playbooks\" .` across the whole repo (excluding docs/history/**, docs/knowledge/**, docs/decisions/**, and docs/discovery/** — those are historical/decision records, per CONTEXT.md's Integration Points, and are never edited by this feature) and confirm zero hits outside those excluded trees.",
    "must_haves": {
      "truths": [
        "AGENTS.md (post-regen) and packages/bee/AGENTS.block.md both quote \"Playbooks\", not \"Class playbooks\"",
        "skills/bee-planning/SKILL.md and skills/bee-hive/references/scout-and-ticks.md both quote \"Playbooks\"",
        "scout-and-ticks.md line 34's `class ∈` list is byte-for-byte unchanged",
        "rg -n \"Class playbooks\" across the repo, excluding docs/history/**, docs/knowledge/**, docs/decisions/**, docs/discovery/**, returns zero hits",
        "bee dev release-manifest --check reports clean"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "Deep contracts pointer sentence quotes \"Playbooks\""},
        {"path": "docs/02-architecture.md", "substantive": "bee-planning repository-layout line names playbooks/"},
        {"path": "docs/04-skills-spec.md", "substantive": "bee-planning References line names playbooks/<class>.md"}
      ],
      "key_links": ["AGENTS.md is regenerated from AGENTS.block.md after the edit, not hand-edited"],
      "prohibitions": ["No hand-edit of the generated AGENTS.md file directly", "No change to scout-and-ticks.md line 34", "No edit to any file under docs/history/**, docs/knowledge/**, docs/decisions/**, or docs/discovery/**"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false},
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test agents_block_render_parity --test route_class_parity && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml"
  }
]
```

## Test matrix

The triad, at its smallest demonstrating size.

- **Happy path** — `class_playbook_parity`'s two tests pass against the new
  directory: every `ROUTE_CLASS_VALUES` entry has a matching
  `skills/bee-planning/playbooks/<class>.md` file, and every file in that
  directory names a real class.
- **Edge** — `pointer_integrity` passes: every citation of the new
  `"Playbooks"` anchor resolves to a real heading in `planning-reference.md`;
  `route_class_parity` is unaffected (it never reads the playbook heading,
  row 15) and stays green as a regression check.
- **Error** — a citer accidentally left quoting `"Class playbooks"` after
  the heading is deleted is exactly what `pointer_integrity` is designed to
  catch (a cited heading that no longer exists); this cell's own repo-wide
  `rg` check in pfs-2's action is the same check run by hand before relying
  on the fence.

Cell pfs-1 changes one Rust test file's internals but not its behavior
contract (same two test names, same two guarantees); the "record stays
green at every step" proof (row 18 before, cell pfs-1's own verify after)
is the before/after pair for that non-change.

## Open Questions

(none)

## Out of scope

- Any change to `ROUTE_CLASS_VALUES`, `workflows.rs`, or the route class
  vocabulary itself — this feature only moves and reshapes existing
  playbook prose.
- A `README.md` index inside `skills/bee-planning/playbooks/` separate from
  `planning-reference.md`'s `## Playbooks` section — considered and
  rejected in Approach.
- Editing any file under `docs/history/**`, `docs/knowledge/**`,
  `docs/decisions/**`, or `docs/discovery/**` that historically cites
  `"Class playbooks"` — those are dated records of past decisions, not
  living documentation, per CONTEXT.md's Integration Points.
- Whether execution workers (cells) should cite a playbook directly instead
  of only the planner citing it at plan time — a real, separate design
  question the user raised in conversation; not part of this feature's
  locked scope, and would need its own CONTEXT.md if pursued.
