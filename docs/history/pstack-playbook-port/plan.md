---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: pstack-playbook-port

## Summary

bee gets the whole pstack playbook set and the skills and principles those
playbooks lean on. A leader no longer picks only one of nine classes. `bee orient`
lists the situations inside the class, such as "bug with an unknown cause"
or "push a metric to a target", and the leader opens that situation's own
steps. The plan check then refuses a plan that cites only the class menu
when the class has situations. Every project that
runs bee gets this on its next refresh.

Mode: `standard` — 3 risk flags: public contracts (`bee orient`
and `bee plan check` output, vendored playbooks), covered contract change
(`class_playbook_parity.rs` pins one file per class), multi-domain (Rust
verbs, playbooks, skills, principles, doctrine)
Why this is the least workflow that protects the work: three verbs and the
vendored playbook surface change, so it needs a plan and the hat check, but
no hard-gate territory is touched.

## Requirements (from CONTEXT.md)

- D1: all 23 pstack playbooks become situation playbooks under the nine
  classes, plus a figure-it-out fallback; the pstack method text is the base.
- D2: each pstack tool is swapped for its bee counterpart; a step with no
  counterpart is dropped and named in the port notes.
- D3: an overlapping pair merges into one file per situation, pstack body
  plus bee's enforced lines.
- D4: playbooks that land work merge on green with `bee worktree merge`; the
  uat acceptance at `bee close` stays the owner's.
- D5: every pstack skill and principle the playbooks or router cite, and bee
  lacks, comes in; one bee already covers stays bee's and takes the pstack
  improvements.

Planning choices are store decisions `c64c5827` (situation layout and class
mapping), `41bbba68` (router contract after the hat wave, superseding
`0f6dae94`) and `6ca3cd62` (principle mapping and the `bee-fanout` name).
The hat wave report is `docs/history/pstack-playbook-port/reports/hat-wave.md`.

## Load-bearing claims

Labels are `read` (opened this session) or `ran` (command run this session);
the evidence column is the bytes found at the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `bee orient --class` maps a class to one playbook file path | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:741` | `packet.insert("playbook".into(), json!(format!("skills/bee-planning/playbooks/{class}.md")));` |
| 2 | orient matches its argv by exact slice patterns, so a new flag needs new patterns | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:881` | `["orient", "--class", class] => return run_class(class, false, t0),` |
| 3 | The route record is built field by field; this plan leaves it unchanged | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:647` | `route.insert("class".into(), json!(class_value.unwrap()));` |
| 4 | The route verb has a closed flag list; this plan adds no flag | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:844` | `&["set", "show", "class", "lane", "lane-record", "flags", "files", "rationale", "no-lane"],` |
| 5 | Feature start reads route flags itself; this plan leaves it unchanged | read | `packages/bee-rs/crates/bee/src/verbs/feature_group.rs:68` | `if let Some(v) = flags.get("class") {` |
| 6 | The plan check matches one `<name>.md` directly under the playbooks directory | read | `packages/bee-rs/crates/bee/src/verbs/plan_group.rs:83` | `let cited = \|name: &str\| text.contains(&format!("{PLAYBOOKS}{name}.md"));` |
| 7 | The parity fence reads only `.md` entries of the playbooks directory, so subdirectories leave its two tests intact | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:102` | `entry.file_name().to_str().and_then(\|n\| n.strip_suffix(".md")).map(str::to_string)` |
| 8 | There are nine route classes | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 9] =` |
| 9 | The release manifest walks the whole skills tree, so new playbook and skill files vendor with no manifest code change | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:387` | `records.extend(enumerate_tree(root, &root.join("skills"), "plugin_skill", &[])?);` |
| 10 | Each new `bee-` skill needs a description under 300 characters | read | `packages/bee-rs/crates/bee/tests/skill_triggers.rs:15` | `const DESCRIPTION_BUDGET: usize = 300;` |
| 11 | Each new `bee-` skill needs at least three trigger briefs in the fixture | read | `packages/bee-rs/crates/bee/tests/skill_triggers.rs:11` | `const MIN_EXPECT: usize = 3;` |
| 12 | Every class playbook ends on the class proof floor line a merge must keep | read | `skills/bee-planning/playbooks/bugfix.md:11` | `Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).` |
| 13 | The pstack source is MIT, so the port carries its notice | ran | `head -3 /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/LICENSE` | `MIT License` |
| 14 | Skills reach host repos through rendered plugin trees, not a `skills/` tree at the host root, so a host verb cannot read playbooks from disk | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:79` | `".claude-plugin/skills",` |
| 15 | principles.rs already compiles its index into the binary as the host fallback, the shape situations.rs copies | read | `packages/bee-rs/crates/bee/src/principles.rs:26` | `const BUILT_IN_INDEX: &str = include_str!(` |
| 16 | The rendered playbooks are byte copies of the source, so `diff -r` proves the render | ran | `diff -r skills/bee-planning/playbooks .claude-plugin/skills/bee-planning/playbooks && echo identical` | `identical` |
| 17 | Seven of the twelve principles the pstack playbooks cite have no bee skill | ran | `rg -o -I "principle-[a-z-]+" <pstack>/skills/poteto-mode/playbooks <pstack>/skills/figure-it-out \| sort -u` | `principle-build-the-lever principle-encode-lessons-in-structure principle-foundational-thinking principle-guard-the-context-window principle-laziness-protocol principle-migrate-callers-then-delete-legacy-apis principle-minimize-reader-load principle-model-the-domain principle-prove-it-works principle-redesign-from-first-principles principle-separate-before-serializing-shared-state principle-subtract-before-you-add` |

## Discovery

Read this session: the pstack router `skills/poteto-mode/SKILL.md`, 15 of its
playbooks in full and the rest by router entry, `figure-it-out/SKILL.md`, and
the skill list with descriptions. bee side: the three class playbooks bugfix,
research and feature, `orient.rs`, `plan_group.rs`, `workflows.rs`,
`class_playbook_parity.rs`, `skill_triggers.rs` and `principle_index_parity.rs`.
The 2026-09-24 gap-check (`docs/history/research/pstack-playbooks-gap-check.md`)
holds the per-playbook bee anchors D3 merges into. Of 12 principles the pstack
playbooks cite, 7 have no bee skill.

## Approach

Class playbook: `skills/bee-planning/playbooks/feature.md` — step 1 (data shape first: the class index Situations grammar), step 2 (walking skeleton: slice 1 drives the bugfix class end to end), step 3 (current slice only), step 4 (prove the user-visible path: `bee orient` and `bee plan check` output).

Recommended path: build the selection path first and prove it end to end on
one class (`bugfix`), then port the rest in parallel waves that only add
Markdown, then the skills and principles, then the doctrine and knowledge.
The router change is small (claims 1, 2, 6, 15): orient lists the
situations a class index declares, read from the repo copy or a copy compiled
into the binary (claim 14 rules out a disk read in hosts), and the plan check
holds a plan to a situation file. The bulk is careful text work that
parallelizes by class and by skill.

SMALLER PATH: is there a cheaper shape that honors every locked decision?
Yes, and this plan takes it after the hat wave: no situation field on the
route record and no new flag, so two of the three verbs change and the route
contract does not (`41bbba68`). A docs-only shape is cheaper still and
fails the Feature Boundary: no verb could hold a plan to a situation.

Rejected: a situation field on the route record with a write-time file
check (refuses in every host, claim 14, and grows orient's argv patterns,
claim 2); docs-only selection (the plan check could hold only the class
menu, so the steps stop being enforced); one route class per pstack playbook (breaks the proof-floor table,
the principle `classes:` filter and two parity fences for 23 values);
situations as sections inside the class file (the router could not name a
file and the plan check could not hold it); verbatim copy (D2).

Waves: slice 1 runs two chains side by side. The Rust chain ppb-1, ppb-1t,
ppb-2, ppb-2t is serial because every cell compiles the same crate in one
worktree. ppb-3 and ppb-3c touch Markdown and the rendered trees only and run
beside it. ppb-4 waits on both chains because the fence reads the bugfix files
and the crate.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ppb-1 and ppb-2 write Rust."},
    {"stage":"testing","classification":"required","role":"test","reason":"Every tester and checker cell runs on the test role, apart from its implementer."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"ppb-3 and the later port slices write playbooks, skills and principles."},
    {"stage":"consolidation","classification":"required","role":"writer","reason":"The final slice consolidates knowledge."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a leader lookup needs more than three files","reason":"Cheap reads."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Gathers use read."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Risk map

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| orient output changes for scripts | MEDIUM | ppb-1 | the playbook line stays byte-identical; `situations` is always present (ppb-1t) |
| Host repo has no `skills/` tree | MEDIUM | ppb-1 | ppb-1t runs orient in a repo with no `skills/` and gets the compiled-in situations (claims 14, 15) |
| Plans in flight that cite only a class index | MEDIUM | ppb-2 | the finding fires only for a class that lists situations; slice 1 lists one (bugfix); the plan check is advisory, it never refuses a gate |
| The merged bug-fix file drops an enforced bee line | MEDIUM | ppb-3 | ppb-3c lists every old line as present (claim 12) |
| A vendored file names a tool hosts lack | MEDIUM | ppb-3 and the port slices | the checker's `rg` over `skills/bee-planning/` finds none; port notes live under `docs/history/` |
| A class index names a skill not yet shipped | LOW | ppb-3, slice 3 | slice 1 indexes name no skill; slice 3 adds the bee-figure-it-out fallback line with the skill |
| New skills fail the trigger fixture | LOW | slice 3 | `skill_triggers` green (claims 10, 11) |

## Shape

Epic map. Outcome: a leader asks `bee orient --class <c>` for a task, sees the
situations of that class, opens one, and the plan check holds the citation;
the pstack skills and principles those steps name exist in bee.

| Epic | Capability | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| Selection | orient lists situations; plan check holds a situation citation | Feature boundary | 1 | unit tests per verb, the parity fence |
| Playbooks | 23 situation files, 9 class indexes, the session index | D1, D2, D3 | 1 (bugfix), 2 (the rest) | checker report per class |
| Skills | 11 pstack skills bee lacks, `bee-fanout` for swarm | D5, `6ca3cd62` | 3 | `skill_triggers`, rendered trees |
| Principles | 13 new principles and merges into covered ones (claim 17) | D5, `6ca3cd62` | 4 | `principle_index_parity` |
| Doctrine and knowledge | LEADER route step names situations and the fallback; knowledge consolidated | Feature boundary | 5 | rule index parity, `bee knowledge check` |

Slice queue: 1 selection plus the bugfix skeleton (current); 2 the other 22
playbooks, 8 class indexes and `session/index.md`, one cell per class or the
session group, each with its checker (deps 1); 3 the 11 skills including
`bee-figure-it-out` and `bee-fanout`, the comment-sicko reviewer prompt, their
trigger briefs, and the fallback line on every class index (deps 1, 2); 4 the
principles and the improvements merged into the bee skills and principles that
already cover a pstack one (deps 3); 5 doctrine, knowledge consolidation and
the full regen (deps 2, 3, 4).

## Cells — current slice (preview)

Slices 1 to 5 are capped. The full suite (`bee test` at `fec8793da`) went red in three orient class tests: 5186 passed, 3 failed. Their temp repo has no `skills/` tree, so orient reads the compiled-in indexes, which list situations since slice 2. The fallback behaves as designed; the tests pinned shipped text. This fix-first slice (rule agents-never-build-on-red) is one test cell, ppb-40, which gives those tests their own fixture indexes. `bee test` runs again after it caps.

Waves: one cell.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ppb-40 | Pin the orient class tests to their own fixture indexes | its report only | — | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ppb-40",
  "feature": "pstack-playbook-port",
  "title": "Pin the orient class tests to their own fixture indexes",
  "lane": "standard",
  "role": "test",
  "deps": [],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs",
   "packages/bee-rs/crates/bee/src/situations.rs",
   ".bee/logs/test-failure-test-0.log"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Three tests in status_full/tests.rs went red in the full suite once slice 2 filled the class indexes: bee_orient_class_perf_json_carries_an_empty_situations_key, bee_orient_class_research_prints_the_research_playbook_and_principles_and_writes_nothing and orient_class_names_the_playbook_and_the_class_principles_and_writes_nothing. Their temp repo has no skills/ tree, so orient reads the compiled-in indexes, which now list situations. Make each test write its own skills/bee-planning/playbooks/<class>.md into the temp repo (the repo copy wins over the compiled-in one), with no Situations section where the test pins an empty list, so the tests check the contract instead of the shipped playbook text. Keep one test that pins the compiled-in fallback by asserting it lists at least one situation for a class whose shipped index has one. Bee commands you may run: cargo test and `bee herding notify`.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full",
  "must_haves": {
   "truths": [
    "the three tests pass and no longer depend on the shipped playbook text",
    "a test still pins the compiled-in fallback"
   ],
   "prohibitions": [
    "No edit outside status_full/tests.rs"
   ]
  }
 }
]
```

## Test matrix

| Cell | Happy path | Edge case | Error path |
|---|---|---|---|
| ppb-40 | the three tests with fixture indexes. Pass when `cargo test status_full` is green | a fixture index with no Situations section. Pass when orient prints no situation line | the compiled-in fallback. Pass when a test still sees at least one situation with no `skills/` tree |

The run on main and on head for changed behavior: none; this cell changes tests only.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
