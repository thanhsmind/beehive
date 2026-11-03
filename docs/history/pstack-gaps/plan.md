---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset>
---

# Plan: pstack Gaps

Mode: `standard` — 2 risk flags: public-contracts (AGENTS.md gains a law), multi-domain (skills + knowledge bundle + a Rust test).
Why this is the least workflow that protects the work: there is no product code
here — one new skill reference file, three pointer edits, ten one-line additions
to an existing registry, one AGENTS.md paragraph, and one text-reading test. The
real exposure is drift between the three places a rule id is written, and one
test pins exactly that.

## Requirements (from CONTEXT.md)

- **D1** — Two procedures, one new home: `skills/bee-researching/references/trace-and-provenance.md`. One reference-table row in `bee-researching/SKILL.md`; one pointer line in the `research` class playbook. No transcription.
- **D2** — "Trace" fans out to 2-4 read-only workers over disjoint entry points through the dispatch door; the leader folds one account with `path:line` anchors. One entry point runs inline and says so.
- **D3** — "Provenance sweep" names seven evidence categories; an empty category is reported by name, an unswept one is named as unswept.
- **D4** — The existing `## AGENTS.md rule homes` index gains a one-line spoken form per rule. No new file, no new verb.
- **D5** — `packages/bee/AGENTS.block.md` states the invocation law and names the index path; the reply must name what changed, or say plainly that nothing changed.
- **D6** — `packages/bee-rs/crates/bee/tests/rule_index_parity.rs` pins markers ↔ markers ↔ index rows, and the non-empty spoken line.

## Load-bearing claims

Labels are `read` or `ran`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The rule registry already exists and is a parsed structure, not prose | read | `packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs:18-23` | `pub(crate) struct RuleHome { / pub(crate) rule: String, / pub(crate) home: String,` |
| 2 | Its parser finds the section by an exact heading, so the heading must not move | read | `packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs` (`parse_agents_rule_homes`) | `let Some(start_pos) = body.find("## AGENTS.md rule homes") else {` |
| 3 | The parser treats a line starting with `- ` as a NEW rule row — a spoken line added as a sibling bullet would be misparsed as an eleventh rule | read | same function | `if line.starts_with("- ")` |
| 4 | The index today carries the AGENTS.md section per rule but NO spoken line | ran | `awk` over `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` | `- ``agents-proof-at-cap`` (AGENTS.md § Prove, then say so):` |
| 5 | There are exactly ten rule ids, and both AGENTS surfaces carry the same ten | ran | `rg -n "<!-- rule: " AGENTS.md packages/bee/AGENTS.block.md` | ten markers in each file, same ids |
| 6 | `bee knowledge check` already validates ref→home, so this feature must NOT add a second ownership validator | read | `bee knowledge check --help` | `duplicate_rule_home, unknown_rule_ref, applied_at_unlinked` |
| 7 | `bee-researching` reaches its references through a table, so a new file costs one row | read | `skills/bee-researching/SKILL.md:93-99` | `## References` + the three existing rows |
| 8 | The `research` class playbook already exists and already states the read-only and empty-source steps this feature deepens | read | `skills/bee-planning/references/planning-reference.md:216-223` | `2. Trace the runtime path, not just the file list. / 3. Name every source searched that came up EMPTY.` |
| 9 | The text-reading test pattern needs no crate import | read | `packages/bee-rs/crates/bee/tests/route_class_parity.rs` | reads `workflows.rs` as text |
| 10 | Skill files have generated copies under five plugin trees and regen rewrites one shared manifest — skill-touching cells are SERIAL | read | `docs/history/pstack-adoption/plan.md` rows 14-15 | `the regen chain rewrites ONE shared manifest on every run, which serializes skill-touching cells` |

## Discovery

The CONFLICT row shrank on inspection. bee does not lack a rule registry — it has
one, homed in the knowledge bundle, parsed by `ownership.rs`, and already
validated by `bee knowledge check`. Two things are missing, and only two: a
sentence per rule in the words a person would actually say, and the law that
makes saying it do something. That moves item 3 from rung 4 (build) to rung 1
(reuse), and removes the new CLI verb the backlog item's CoS imagined.

Claim 3 is the trap. `parse_agents_rule_homes` reads any `- ` line as a new rule
row. A spoken line MUST therefore be an indented continuation, not a sibling
bullet, or `bee knowledge check` starts reporting ten phantom rules.

## Approach

**Recommended path.** Three surfaces, three cells, run SERIALLY because every one
of them touches a skill or a doc whose regen rewrites the shared release manifest
(claim 10).

1. The two procedures land in one new reference file and are cited, never copied
   (D1) — the same single-home-plus-pointer shape `pstack-adoption` used for the
   class playbooks.
2. The spoken forms land as an indented `spoken:` line inside each existing rule
   row (D4, claim 3), so the parser sees the same ten rules it sees today.
3. The invocation law lands once in `packages/bee/AGENTS.block.md` and reaches
   `AGENTS.md` through `bee dev regen` (D5).
4. `tests/rule_index_parity.rs` pins all three against each other (D6).

**Rejected.** A `bee knowledge rules` verb (rung 4 for a rung 1 gap; catalog,
registry payload, front-door test, help text — all to print a list a reader can
already open). A second reference file per procedure (two router rows for one
subject). A guard that refuses a source edit under the `research` class — that is
`p-69bee217` and stays out.

## Cells — current slice

| id | title | files | proof |
|---|---|---|---|
| pg-1 | Write the Trace and Provenance sweep procedures and cite them | `skills/bee-researching/references/trace-and-provenance.md`, `skills/bee-researching/SKILL.md`, `skills/bee-planning/references/planning-reference.md` | `bee dev regen` clean + pointer check |
| pg-2 | Give each rule a spoken line and state the invocation law | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`, `packages/bee/AGENTS.block.md`, `AGENTS.md` | `bee knowledge check` still reports ten rules, not twenty |
| pg-3 | Pin markers, index rows and spoken lines with one text-reading test | `packages/bee-rs/crates/bee/tests/rule_index_parity.rs` | `cargo test --test rule_index_parity` green; RED first against the un-migrated index |

Serial order: pg-1 → pg-2 → pg-3. pg-3 is red-first: it is written against the
index BEFORE pg-2's spoken lines exist only if pg-3 runs first; because pg-2 runs
first here, pg-3 proves itself by deleting one spoken line, watching the test
fail, and restoring it.

## Proof

- `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the declared suite, over a change that touches one new test and no product source.
- `.bee/bin/bee knowledge check --json` — the ownership validator, which must still see exactly ten rules after D4's edit (claim 3).
- `.bee/bin/bee dev regen` — the skill copies under the five plugin trees and the shared release manifest.
