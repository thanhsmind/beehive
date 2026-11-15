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

Slices 1 to 4 are capped: the situation router, all 23 playbooks, the eleven skills, the thirteen new principles, and the merges into the bee skills and principles that already covered a pstack one. This is slice 5, the last: ppb-38 teaches the leader doctrine to route to a situation and names the fallback and the new skills; ppb-39 is the writer consolidation cell that writes the knowledge and `delivery.md`. After it caps, `bee test` runs the full suite once in the worktree before the merge.

Waves: ppb-38, then its checker, then ppb-39 and its checker; each depends on the one before.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ppb-38 | Teach the leader to route to a situation | `packages/bee/LEADER.md`, `.bee/LEADER.md`, `skills/bee-planning/SKILL.md`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-38c | Check the situation routing doctrine | its report only | ppb-38 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-39 | Consolidate pstack-playbook-port knowledge | its report only | ppb-38c | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-39c | Check the consolidated knowledge | its report only | ppb-39 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ppb-38",
  "feature": "pstack-playbook-port",
  "title": "Teach the leader to route to a situation",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "b749c98d-65e2-4346-b07f-6dc52dbd78ca",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "packages/bee/LEADER.md",
   ".bee/LEADER.md",
   "skills/bee-planning/SKILL.md",
   "skills/bee-planning/references/planning-reference.md",
   "skills/bee-hive/references/routing-and-contracts.md",
   "skills/bee-hive/SKILL.md",
   ".claude-plugin/skills/bee-planning/",
   ".codex-plugin/skills/bee-planning/",
   ".claude/skills/bee-planning/",
   ".agents/skills/bee-planning/",
   ".opencode/skills/bee-planning/",
   ".claude-plugin/skills/bee-hive/",
   ".codex-plugin/skills/bee-hive/",
   ".claude/skills/bee-hive/",
   ".agents/skills/bee-hive/",
   ".opencode/skills/bee-hive/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   ".bee/onboarding.json"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee/LEADER.md",
   ".bee/LEADER.md",
   "skills/bee-planning/SKILL.md",
   "skills/bee-planning/references/planning-reference.md",
   "skills/bee-hive/references/routing-and-contracts.md",
   "skills/bee-hive/SKILL.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/session/index.md"
  ],
  "affects_skills": [
   "skills/bee-planning/SKILL.md",
   "skills/bee-planning/references/planning-reference.md",
   "skills/bee-hive/references/routing-and-contracts.md",
   "skills/bee-hive/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Update the doctrine to the situation router (decision 41bbba68). packages/bee/LEADER.md § Route each task: after `bee orient --class <c>`, pick the situation whose trigger phrases match the task from the situation lines it prints, open that situation file, and seed the todo list from its numbered steps; when no situation fits, or the work is large and cross-cutting, use bee-figure-it-out; a session situation (pick up, pause, run until done, orchestrate, babysit, ship) comes from skills/bee-planning/playbooks/session/index.md and binds no class. Add to § Trigger index one line per new skill a leader reaches by situation (bee-architect, bee-arena, bee-interrogate, bee-fanout, bee-tdd, bee-show-me-your-work, bee-reflect, bee-recall, bee-blast-radius, bee-comment-review, bee-figure-it-out), pointing, never restating. In skills/bee-planning/SKILL.md and planning-reference.md § Playbooks, the plan cites the situation file skills/bee-planning/playbooks/<class>/<situation>.md, and the class index only for a class with no situations. In skills/bee-hive/references/routing-and-contracts.md, the todo list seeds from the situation's steps. Name no principle in any of these surfaces (decision a6e3d0bf). Copy packages/bee/LEADER.md to .bee/LEADER.md byte for byte, run bee dev regen, and commit. Bee commands you may run in this cell: cargo test, `.bee/bin/bee dev regen`, `.bee/bin/bee dev release-manifest --check`, `.bee/bin/bee knowledge check`, `.bee/bin/bee knowledge consolidate`, `.bee/bin/bee onboard --repo-root . --json`, and `bee herding notify`. Never run `bee dispatch` or any of its `--help`.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test rule_index_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pointer_integrity && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json",
  "must_haves": {
   "truths": [
    "LEADER.md § Route each task tells the leader to pick a situation from the orient output and names bee-figure-it-out as the fallback",
    "the plan template cites a situation file",
    "the trigger index points at every new skill"
   ],
   "prohibitions": [
    "No surface enumerates the principle set",
    "No rule is restated in two places"
   ]
  }
 },
 {
  "id": "ppb-38c",
  "feature": "pstack-playbook-port",
  "title": "Check the situation routing doctrine",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-38"
  ],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-38c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee/LEADER.md",
   ".bee/LEADER.md",
   "skills/bee-planning/SKILL.md",
   "skills/bee-planning/references/planning-reference.md",
   "skills/bee-hive/references/routing-and-contracts.md",
   "skills/bee-hive/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Check that each surface ppb-38 changed says the same thing about situations, that no surface restates another, that every new skill the trigger index names exists in skills/, and that .bee/LEADER.md equals packages/bee/LEADER.md. Write docs/history/pstack-playbook-port/reports/ppb-38c.md; no other edits.",
  "verify": "cmp packages/bee/LEADER.md .bee/LEADER.md",
  "must_haves": {
   "truths": [
    "the report names each surface and its agreement"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-39",
  "feature": "pstack-playbook-port",
  "title": "Consolidate pstack-playbook-port knowledge",
  "lane": "standard",
  "role": "writer",
  "deps": [
   "ppb-38c"
  ],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/knowledge/",
   "docs/history/pstack-playbook-port/delivery.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/port-notes/",
   "docs/history/pstack-playbook-port/reports/",
   "docs/knowledge/areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Run `.bee/bin/bee knowledge consolidate --feature pstack-playbook-port` and write what it asks into docs/knowledge/: the situation layer (class index grammar, situations.rs and its compiled-in fallback, the orient situation lines, the plan check citation rule, the parity fence), the session group, the fallback skill, the eleven new skills and the thirteen new principles, each anchored to its files; re-grade every concept the slice caps acknowledged as suspect and anchored by files this feature changed (bee knowledge confirm or a fix). Write docs/history/pstack-playbook-port/delivery.md: what shipped, per slice, with the commits. Bee commands you may run in this cell: cargo test, `.bee/bin/bee dev regen`, `.bee/bin/bee dev release-manifest --check`, `.bee/bin/bee knowledge check`, `.bee/bin/bee knowledge consolidate`, `.bee/bin/bee onboard --repo-root . --json`, and `bee herding notify`. Never run `bee dispatch` or any of its `--help`.",
  "verify": ".bee/bin/bee knowledge check",
  "must_haves": {
   "truths": [
    "docs/knowledge names the situation layer with anchors to situations.rs, orient.rs, plan_group.rs and class_playbook_parity.rs",
    "docs/history/pstack-playbook-port/delivery.md exists"
   ],
   "prohibitions": [
    "No skill or playbook text is edited"
   ]
  }
 },
 {
  "id": "ppb-39c",
  "feature": "pstack-playbook-port",
  "title": "Check the consolidated knowledge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-39"
  ],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-39c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/delivery.md",
   "docs/knowledge/index.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Run bee knowledge check, confirm each concept ppb-39 wrote names real anchors and that delivery.md matches the capped cells. Write docs/history/pstack-playbook-port/reports/ppb-39c.md; no other edits.",
  "verify": ".bee/bin/bee knowledge check",
  "must_haves": {
   "truths": [
    "bee knowledge check is green"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 }
]
```

## Test matrix

| Cell | Happy path | Edge case | Error path |
|---|---|---|---|
| ppb-38 | LEADER.md § Route each task. Pass when it names the situation step and the bee-figure-it-out fallback, and `rule_index_parity` is green | a session situation. Pass when the doctrine says it binds no class | a surface that enumerates the principle set. Pass when the checker finds none |
| ppb-39 | the situation layer concept. Pass when `bee knowledge check` is green and its anchors name the changed files | a concept acknowledged as suspect in an earlier cap. Pass when it is confirmed or fixed | a concept anchor to a missing file. Pass when `bee knowledge check` names none |

The run on main and on head for changed behavior: on main `.bee/LEADER.md` § Route each task stops at the class playbook; on head it continues to a situation and names the fallback.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
