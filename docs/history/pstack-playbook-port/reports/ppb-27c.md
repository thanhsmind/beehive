# ppb-27c: check of the principles ported in ppb-27

Checked against commit `58285028b` (ppb-27). The sources are the pstack
`principle-laziness-protocol`, `principle-minimize-reader-load` and
`principle-model-the-domain` `SKILL.md` files (each directory holds only that
one file). No file other than this report was edited.

## Verdict

Every source rule of the three principles is present in bee. One source link
is left out with its bee home named (the guard-the-context-window link, below).
None is missing. No pstack-only tool name appears in the new skills or in their
expertise sections, and no bee rule was removed. One example word is dropped:
"clean architecture" from the minimize-reader-load list of proxies (row 4
below); the rule it illustrates is kept.

"Present" means the rule is in the bee skill body, its depth section, or both.
Depth sections: `expertise/architecture.md` § Laziness Protocol (line 384),
§ Model the Domain (line 413); `expertise/review.md` § Reader load (line 388).
The vendored copies under `.bee/expertise/` match these files byte for byte,
and the three `SKILL.md` files are identical in all five rendered trees
(`.claude-plugin`, `.codex-plugin`, `.claude`, `.agents`, `.opencode`).

## laziness-protocol

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Writing code is cheap for you, so over-engineering is easy; borrow a human maintainer's fatigue | present | skill body, first paragraph; depth opening |
| 2 | Aim for the most result with the least code and complexity | present | skill body; depth opening |
| 3 | Prefer deletion: look for removals before additions | present | skill body; depth bullet, pointing at Subtract before you add |
| 4 | Flat call hierarchy; a rich interface hiding real work is not a deep chain | present | skill body; depth bullet |
| 5 | More than 3 files or layers to answer a question: flatten | present | skill body ("more than three files or layers"); depth bullet |
| 6 | Consolidate decisions behind one source of truth, pass the result as a simple flag | present | skill body; depth bullet, pointing at Single Source of Truth |
| 7 | Minimize the diff; fewer lines beat elegant boilerplate | present | skill body; depth bullet |
| 8 | Question the threading of a new signal through types, schemas, pipelines | present | skill body; depth bullet |
| 9 | Sweat the small leaks: pass-throughs, representation leaks, duplicated choices compound | present | skill body; depth bullet |
| 10 | Prime directive: code a human would find exhausting to maintain is bad | present | skill Check line; depth opening ("prime test") |

## minimize-reader-load

| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Maintainability is the work a reader must do | present | skill body, first sentence; depth opening |
| 2 | Axis one: layers to trace | present | skill body; depth ("Layers to trace") |
| 3 | Axis two: state to hold | present | skill body; depth ("State to hold") |
| 4 | Why: code is read more than written; LOC, cyclomatic complexity and "clean architecture" are proxies | present | skill Why line; depth opening. The "clean architecture" example is dropped in both; the proxy rule stays |
| 5 | The axes are independent; 50 globals vs 6-layer adapter stack; guard both | present | skill body; depth opening |
| 6 | Human analog of Guard the Context Window: working memory is finite for readers too | left out (link) | method kept as a plain sentence in the skill Why line and depth opening; the link waits for its bee home, the guard-the-context-window port listed in D5 (port notes, "Pointers kept as plain sentences") |
| 7 | Collapse layers that do not earn their keep: one-caller wrappers, single-implementation adapters, speculative indirection | present | skill body; depth bullet, pointing at The Deletion Test |
| 8 | Adjacent layers must change the abstraction; collapse pass-throughs | present | depth bullet only ("A layer that repeats its neighbour") |
| 9 | Demand interface compression; prefer boundaries that hide meaningful decisions | present | depth bullet ("A broad interface that hides little"); skill Not when line |
| 10 | Shrink state scope: pure functions, locals, fields, module state, globals; derive instead of sync | present | skill body; depth bullet |
| 11 | Name the invariant at the boundary, not in every consumer | present | skill body; depth bullet |
| 12 | Before adding a layer or state, ask if it reduces load elsewhere by as much | present | skill body; depth closing paragraph |
| 13 | The test: where does X come from, what can change X, under 30 seconds | present | skill Check line; depth closing paragraph |

## model-the-domain

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
| # | Source rule | Status | Where in bee |
|---|-------------|--------|--------------|
| 1 | Encode the domain in a data structure, not scattered conditionals | present | skill body; depth opening |
| 2 | Why: scattered booleans, shape assumptions, cross-file branching are accidental complexity | present | skill body; depth opening |
| 3 | A matching structure makes invalid states unrepresentable and deletes branches | present | skill body; depth opening |
| 4 | Cheap at write time; recovered later it reads as a refactor and gets deferred | present | skill body; depth opening |
| 5 | State machine over scattered booleans, phases, lifecycle checks | present | skill body; depth bullet |
| 6 | Typed object over loose parameters or repeated shape assumptions | present | skill body; depth bullet |
| 7 | Map, registry, lookup table, discriminated union over cross-file branching | present | skill body ("map or tagged union"); depth bullet (full list) |
| 8 | Reducer or command/event model over ad hoc mutation | present | skill body; depth bullet |
| 9 | Module around one body of domain knowledge, not load/validate/transform/save; execution order is not ownership | present | skill body; depth bullet (with "Execution order is not ownership") |
| 10 | Small module boundary gathering repeated behavior, ownership, invariants | present | depth bullet only |
| 11 | Queue, cache, index, graph/tree, normalized collection where access calls for it | present | depth bullet only |
| 12 | Any other structure; when none fits, find what must never be allowed and how data is read | present | skill body; depth bullet |
| 13 | Do not force an abstraction; prefer boring code when shape is clear, local, unlikely to grow | present | skill Not when line; depth closing paragraph |
| 14 | Distrust an abstraction that adds indirection without removing branches, rules, invalid states or lifecycle risk | present | skill Not when line; depth closing paragraph |
| 15 | Tell: a new feature grows an if/else chain by one branch | present | skill Check line; depth tells paragraph |
| 16 | Tell: a second boolean that must stay in sync with the first | present | skill Check line; depth tells paragraph |
| 17 | Tell: temporal decomposition, phase-named modules repeat domain rules | present | depth tells paragraph only |
<!-- /bee:not-a-deferral -->

## pstack-only tool names

None. The three sources name no pstack tool (D2's list: Cursor `/loop`,
Graphite, bugbot, `poteto-agent`, arena, architect, how, why), and a search of
the three new `SKILL.md` files and both guides for `pstack`, `graphite`,
`bugbot`, `cursor` and `guard-the-context` found nothing. The source's
`disable-model-invocation: true` frontmatter is dropped, as in every bee
principle skill.

## bee rules removed

None. The ppb-27 diff under `expertise/` is additions only (three routing rows
and three sections). The one changed line in
`docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
is its `anchors` list, extended by the three new skill paths with no entry
removed. The remaining changed lines are regenerated hashes in
`docs/history/codex-harness-hardening/release-manifest.json` and
`.bee/onboarding.json`.

## Added, not in the source

Recorded in `docs/history/pstack-playbook-port/port-notes/principles-ppb-27.md`
and confirmed in the files: the Not when line on each skill, a routing-table
row per section, the YAGNI-style limit paragraph in § Laziness Protocol, the
"finding only when you can point at the line" limit in § Reader load, and the
cross-references to Make Illegal States Unrepresentable and The Rule of Three in
§ Model the Domain.

## Proof

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity`:
5 passed, 0 failed (one row per principle skill, every row anchors an existing
guide heading, every skill ships in both plugin trees).
