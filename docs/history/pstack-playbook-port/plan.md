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
| 12 | Every class playbook ends on the class proof floor line a merge must keep | read | `skills/bee-planning/playbooks/bugfix.md:9` | `Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).` |
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

Slices 1 and 2 are capped: orient lists situations, the plan check holds a situation citation, the parity fence guards situation files, and all 23 pstack playbooks are ported into the nine class indexes and the session index. This is slice 3: ppb-25 fixes the slice 2 checker findings (decision `b25486e3`), the eleven pstack skills the playbooks cite come in as bee skills (D5, `6ca3cd62`), and ppb-24 restores the playbook steps that waited for those skills and adds the `bee-figure-it-out` fallback line to every index.

Waves: every docs cell writes under `skills/`, so the scheduler runs them one after another, and each runs `bee dev regen` itself. Each checker follows its docs cell and runs beside the next one.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ppb-25 | Fix the slice 2 checker findings in the session and merged playbooks | `skills/bee-planning/playbooks/session/babysit.md`, `skills/bee-planning/playbooks/session/shipping.md`, `skills/bee-planning/playbooks/session/autopilot-full.md`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-25c | Check the slice 2 fixes | its report only | ppb-25 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-13 | Port the pstack architect skill as bee-architect | `skills/bee-architect/`, `.claude-plugin/skills/bee-architect/`, `.codex-plugin/skills/bee-architect/`, the rendered trees, the release manifest | ppb-25 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-13c | Check bee-architect against its pstack source | its report only | ppb-13 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-14 | Port the pstack arena skill as bee-arena | `skills/bee-arena/`, `.claude-plugin/skills/bee-arena/`, `.codex-plugin/skills/bee-arena/`, the rendered trees, the release manifest | ppb-13 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-14c | Check bee-arena against its pstack source | its report only | ppb-14 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-15 | Port the pstack interrogate skill as bee-interrogate | `skills/bee-interrogate/`, `.claude-plugin/skills/bee-interrogate/`, `.codex-plugin/skills/bee-interrogate/`, the rendered trees, the release manifest | ppb-14 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-15c | Check bee-interrogate against its pstack source | its report only | ppb-15 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-16 | Port the pstack show-me-your-work skill as bee-show-me-your-work | `skills/bee-show-me-your-work/`, `.claude-plugin/skills/bee-show-me-your-work/`, `.codex-plugin/skills/bee-show-me-your-work/`, the rendered trees, the release manifest | ppb-15 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-16c | Check bee-show-me-your-work against its pstack source | its report only | ppb-16 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-17 | Port the pstack swarm skill as bee-fanout | `skills/bee-fanout/`, `.claude-plugin/skills/bee-fanout/`, `.codex-plugin/skills/bee-fanout/`, the rendered trees, the release manifest | ppb-16 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-17c | Check bee-fanout against its pstack source | its report only | ppb-17 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-18 | Port the pstack tdd skill as bee-tdd | `skills/bee-tdd/`, `.claude-plugin/skills/bee-tdd/`, `.codex-plugin/skills/bee-tdd/`, the rendered trees, the release manifest | ppb-17 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-18c | Check bee-tdd against its pstack source | its report only | ppb-18 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-19 | Port the pstack reflect skill as bee-reflect | `skills/bee-reflect/`, `.claude-plugin/skills/bee-reflect/`, `.codex-plugin/skills/bee-reflect/`, the rendered trees, the release manifest | ppb-18 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-19c | Check bee-reflect against its pstack source | its report only | ppb-19 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-20 | Port the pstack recall skill as bee-recall | `skills/bee-recall/`, `.claude-plugin/skills/bee-recall/`, `.codex-plugin/skills/bee-recall/`, the rendered trees, the release manifest | ppb-19 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-20c | Check bee-recall against its pstack source | its report only | ppb-20 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-21 | Port the pstack blast-radius skill as bee-blast-radius | `skills/bee-blast-radius/`, `.claude-plugin/skills/bee-blast-radius/`, `.codex-plugin/skills/bee-blast-radius/`, the rendered trees, the release manifest | ppb-20 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-21c | Check bee-blast-radius against its pstack source | its report only | ppb-21 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-22 | Port the pstack no-comments skill as bee-comment-review | `skills/bee-comment-review/`, `.claude-plugin/skills/bee-comment-review/`, `.codex-plugin/skills/bee-comment-review/`, the rendered trees, the release manifest | ppb-21 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-22c | Check bee-comment-review against its pstack source | its report only | ppb-22 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-23 | Port the pstack figure-it-out skill as bee-figure-it-out | `skills/bee-figure-it-out/`, `.claude-plugin/skills/bee-figure-it-out/`, `.codex-plugin/skills/bee-figure-it-out/`, the rendered trees, the release manifest | ppb-22 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-23c | Check bee-figure-it-out against its pstack source | its report only | ppb-23 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-24 | Restore the playbook steps that waited for the new skills | `skills/bee-planning/playbooks/`, `.claude-plugin/skills/bee-planning/`, `.codex-plugin/skills/bee-planning/`, the rendered trees, the release manifest | ppb-23 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-24c | Check the restored playbook steps | its report only | ppb-24 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ppb-25",
  "feature": "pstack-playbook-port",
  "title": "Fix the slice 2 checker findings in the session and merged playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "b25486e3-747d-4827-bb77-f9d11ef2cb42",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/session/babysit.md",
   "skills/bee-planning/playbooks/session/shipping.md",
   "skills/bee-planning/playbooks/session/autopilot-full.md",
   "skills/bee-planning/playbooks/session/autopilot-stack.md",
   "skills/bee-planning/playbooks/session/orchestrate.md",
   "skills/bee-planning/playbooks/session/opening-a-pr.md",
   "skills/bee-planning/playbooks/session/session-pickup.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "skills/bee-planning/playbooks/perf/perf-issue.md",
   "skills/bee-planning/playbooks/research/investigation.md",
   "skills/bee-planning/playbooks/refactor/refactoring.md",
   "skills/bee-planning/playbooks/spike/prototype.md",
   "skills/bee-planning/playbooks/feature/feature.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-11.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-12.md",
   ".claude-plugin/skills/bee-planning/",
   ".codex-plugin/skills/bee-planning/",
   ".claude/skills/bee-planning/",
   ".agents/skills/bee-planning/",
   ".opencode/skills/bee-planning/",
   "docs/history/codex-harness-hardening/release-manifest.json"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/reports/ppb-9c.md",
   "docs/history/pstack-playbook-port/reports/ppb-11c.md",
   "docs/history/pstack-playbook-port/reports/ppb-12c.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/"
  ],
  "affects_specs": [],
  "action": "Apply decision b25486e3 and the findings in reports ppb-9c, ppb-11c and ppb-12c: babysit and shipping land through bee worktree merge on green checks and review, never on GitHub's CLEAN state; autopilot owners never rebase, and a conflict goes to the merger as a cell; a red CI run is retried as a fresh run on a fresh ref with the pstack reason; every merged situation file ends with its class Proof floor line copied byte for byte from its class index; add the missing worktree-cleanup section and the orchestrate figure-it-out and inbox notes to the session port notes, and either keep inbox/ in orchestrate's record list or reword its inbox references; restore or note each clause ppb-12c lists as dropped with no note; fix the cosmetic port-notes slip. Run bee dev regen and commit the rendered trees and the manifest.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "no session playbook waits on GitHub's CLEAN merge state",
    "no session playbook tells a worker or owner to rebase",
    "every merged situation file ends with its class Proof floor line"
   ],
   "prohibitions": [
    "No situation is added or removed"
   ]
  }
 },
 {
  "id": "ppb-25c",
  "feature": "pstack-playbook-port",
  "title": "Check the slice 2 fixes",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-25"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-25c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/reports/ppb-9c.md",
   "docs/history/pstack-playbook-port/reports/ppb-11c.md",
   "docs/history/pstack-playbook-port/reports/ppb-12c.md",
   "skills/bee-planning/playbooks/session/"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Check each finding in reports ppb-9c, ppb-11c and ppb-12c against the files ppb-25 changed: fixed, noted, or still open. Write docs/history/pstack-playbook-port/reports/ppb-25c.md; no other edits.",
  "verify": "rg -c '^Proof floor' skills/bee-planning/playbooks/bugfix/bug-fix.md skills/bee-planning/playbooks/perf/perf-issue.md skills/bee-planning/playbooks/research/investigation.md skills/bee-planning/playbooks/refactor/refactoring.md skills/bee-planning/playbooks/spike/prototype.md skills/bee-planning/playbooks/feature/feature.md",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-13",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack architect skill as bee-architect",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-25"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-architect/",
   ".claude-plugin/skills/bee-architect/",
   ".codex-plugin/skills/bee-architect/",
   ".claude/skills/bee-architect/",
   ".agents/skills/bee-architect/",
   ".opencode/skills/bee-architect/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-architect.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/design-red-flags.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/rationale-template.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/runner-prompt.md"
  ],
  "affects_skills": [
   "skills/bee-architect/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-architect/ from pstack architect: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Parallel design runners go out through bee dispatch prepare --kind advisor; where bee already has blind lanes (bee blind check, lane-1 to lane-3 roles), use them instead of inventing a second mechanism. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-architect.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-architect/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-architect",
    "no file under skills/bee-architect/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-13c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-architect against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-13"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-13c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-architect/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-architect.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/design-red-flags.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/rationale-template.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/architect/references/runner-prompt.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-architect/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-13c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-architect/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-14",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack arena skill as bee-arena",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-13"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-arena/",
   ".claude-plugin/skills/bee-arena/",
   ".codex-plugin/skills/bee-arena/",
   ".claude/skills/bee-arena/",
   ".agents/skills/bee-arena/",
   ".opencode/skills/bee-arena/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-arena.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/arena/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-arena/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-arena/ from pstack arena: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Candidates and the judge go out through bee dispatch prepare; the judge is a --kind reviewer dispatch on a different role than the runners. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-arena.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-arena/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-arena",
    "no file under skills/bee-arena/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-14c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-arena against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-14"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-14c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-arena/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-arena.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/arena/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-arena/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-14c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-arena/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-15",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack interrogate skill as bee-interrogate",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-14"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-interrogate/",
   ".claude-plugin/skills/bee-interrogate/",
   ".codex-plugin/skills/bee-interrogate/",
   ".claude/skills/bee-interrogate/",
   ".agents/skills/bee-interrogate/",
   ".opencode/skills/bee-interrogate/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-interrogate.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/code-quality-review.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/lead-judgment.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/reviewer-prompt.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/rubric.md"
  ],
  "affects_skills": [
   "skills/bee-interrogate/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-interrogate/ from pstack interrogate: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Reviewers go out as bee dispatch prepare --kind reviewer; findings use bee-reviewing's P1/P2/P3 vocabulary, and a P1 blocks as bee-reviewing says. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-interrogate.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-interrogate/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-interrogate",
    "no file under skills/bee-interrogate/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-15c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-interrogate against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-15"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-15c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-interrogate/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-interrogate.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/code-quality-review.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/lead-judgment.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/reviewer-prompt.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/interrogate/references/rubric.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-interrogate/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-15c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-interrogate/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-16",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack show-me-your-work skill as bee-show-me-your-work",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-15"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-show-me-your-work/",
   ".claude-plugin/skills/bee-show-me-your-work/",
   ".codex-plugin/skills/bee-show-me-your-work/",
   ".claude/skills/bee-show-me-your-work/",
   ".agents/skills/bee-show-me-your-work/",
   ".opencode/skills/bee-show-me-your-work/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-show-me-your-work.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/references/decision-log-template.tsv",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/scripts/log.sh"
  ],
  "affects_skills": [
   "skills/bee-show-me-your-work/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-show-me-your-work/ from pstack show-me-your-work: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. The trail lives under docs/history/<feature>/ or .bee/tmp/<feature>/; a settled rule still goes through bee decisions log, the trail never replaces it. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-show-me-your-work.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-show-me-your-work/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-show-me-your-work",
    "no file under skills/bee-show-me-your-work/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-16c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-show-me-your-work against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-16"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-16c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-show-me-your-work/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-show-me-your-work.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/references/decision-log-template.tsv",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/show-me-your-work/scripts/log.sh"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-show-me-your-work/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-16c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-show-me-your-work/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-17",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack swarm skill as bee-fanout",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-16"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-fanout/",
   ".claude-plugin/skills/bee-fanout/",
   ".codex-plugin/skills/bee-fanout/",
   ".claude/skills/bee-fanout/",
   ".agents/skills/bee-fanout/",
   ".opencode/skills/bee-fanout/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-fanout.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/swarm/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-fanout/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-fanout/ from pstack swarm: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Name it bee-fanout (decision 6ca3cd62); its description says it is not bee-swarming, which runs approved cells. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-fanout.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-fanout/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-fanout",
    "no file under skills/bee-fanout/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-17c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-fanout against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-17"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-17c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-fanout/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-fanout.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/swarm/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-fanout/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-17c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-fanout/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-18",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack tdd skill as bee-tdd",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-17"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-tdd/",
   ".claude-plugin/skills/bee-tdd/",
   ".codex-plugin/skills/bee-tdd/",
   ".claude/skills/bee-tdd/",
   ".agents/skills/bee-tdd/",
   ".opencode/skills/bee-tdd/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-tdd.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/tdd/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-tdd/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-tdd/ from pstack tdd: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Point at bee-principle-red-before-green and the bugfix tester contract instead of restating them. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-tdd.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-tdd/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-tdd",
    "no file under skills/bee-tdd/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-18c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-tdd against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-18"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-18c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-tdd/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-tdd.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/tdd/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-tdd/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-18c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-tdd/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-19",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack reflect skill as bee-reflect",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-18"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-reflect/",
   ".claude-plugin/skills/bee-reflect/",
   ".codex-plugin/skills/bee-reflect/",
   ".claude/skills/bee-reflect/",
   ".agents/skills/bee-reflect/",
   ".opencode/skills/bee-reflect/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-reflect.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/divergent-reviewer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/judgment-reviewer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/synthesizer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/tooling-reviewer.md"
  ],
  "affects_skills": [
   "skills/bee-reflect/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-reflect/ from pstack reflect: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Each learning routes through bee mailbox reflect or bee decisions log, and a framework change goes through the bee chain (rule agents-feedback-changes-framework). Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-reflect.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-reflect/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-reflect",
    "no file under skills/bee-reflect/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-19c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-reflect against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-19"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-19c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-reflect/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-reflect.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/divergent-reviewer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/judgment-reviewer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/synthesizer.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/reflect/references/tooling-reviewer.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-reflect/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-19c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-reflect/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-20",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack recall skill as bee-recall",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-19"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-recall/",
   ".claude-plugin/skills/bee-recall/",
   ".codex-plugin/skills/bee-recall/",
   ".claude/skills/bee-recall/",
   ".agents/skills/bee-recall/",
   ".opencode/skills/bee-recall/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-recall.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/recall/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-recall/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-recall/ from pstack recall: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Sources are bee's own records first: bee orient, bee status, the handoff record, bee decisions search, then transcripts. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-recall.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-recall/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-recall",
    "no file under skills/bee-recall/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-20c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-recall against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-20"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-20c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-recall/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-recall.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/recall/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-recall/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-20c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-recall/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-21",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack blast-radius skill as bee-blast-radius",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-20"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-blast-radius/",
   ".claude-plugin/skills/bee-blast-radius/",
   ".codex-plugin/skills/bee-blast-radius/",
   ".claude/skills/bee-blast-radius/",
   ".agents/skills/bee-blast-radius/",
   ".opencode/skills/bee-blast-radius/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-blast-radius.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/blast-radius/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-blast-radius/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-blast-radius/ from pstack blast-radius: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Read the knowledge store first (bee knowledge search) before a code hunt. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-blast-radius.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-blast-radius/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-blast-radius",
    "no file under skills/bee-blast-radius/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-21c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-blast-radius against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-21"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-21c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-blast-radius/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-blast-radius.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/blast-radius/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-blast-radius/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-21c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-blast-radius/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-22",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack no-comments skill as bee-comment-review",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-21"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-comment-review/",
   ".claude-plugin/skills/bee-comment-review/",
   ".codex-plugin/skills/bee-comment-review/",
   ".claude/skills/bee-comment-review/",
   ".agents/skills/bee-comment-review/",
   ".opencode/skills/bee-comment-review/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-comment-review.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/no-comments/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/agents/comment-sicko.md"
  ],
  "affects_skills": [
   "skills/bee-comment-review/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-comment-review/ from pstack no-comments: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. Put the comment-sicko agent text into skills/bee-comment-review/references/comment-reviewer.md as a reviewer prompt dispatched through bee dispatch prepare --kind reviewer; tie the skill to bee's no-code-comments rule and comment baseline instead of restating them. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-comment-review.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-comment-review/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-comment-review",
    "no file under skills/bee-comment-review/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-22c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-comment-review against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-22"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-22c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-comment-review/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-comment-review.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/no-comments/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/agents/comment-sicko.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-comment-review/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-22c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-comment-review/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-23",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack figure-it-out skill as bee-figure-it-out",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-22"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-figure-it-out/",
   ".claude-plugin/skills/bee-figure-it-out/",
   ".codex-plugin/skills/bee-figure-it-out/",
   ".claude/skills/bee-figure-it-out/",
   ".agents/skills/bee-figure-it-out/",
   ".opencode/skills/bee-figure-it-out/",
   "packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-figure-it-out.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-writing-skills/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/figure-it-out/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-figure-it-out/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Create skills/bee-figure-it-out/ from pstack figure-it-out: SKILL.md and every reference file listed in read_first under /home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack, in bee terms. It is the fallback a class index names when no situation fits; the phases map onto bee: the done predicate goes into CONTEXT.md, the designed phase list into plan.md, the decision trail through bee-show-me-your-work. Port rules: the pstack text is the base (D1, D5); every pstack tool is swapped for its bee counterpart per D2: subagent spawning becomes bee dispatch prepare (the payload it returns, never a hand-picked model or subagent_type), model-per-role settings become bee team roles, Cursor /loop and Graphite are dropped, bugbot becomes bee-reviewing, how and why become bee-how and bee-why, show-me-your-work becomes bee-show-me-your-work. A step with no bee counterpart is dropped and named in your port notes. Frontmatter follows skills/bee-how/SKILL.md: name, a description under 300 characters that says what it does, when to use it and what it is not for, and metadata with ecosystem bee. No line names Cursor, Graphite, bugbot or poteto-agent. No code comments in any script (a #! line and a license header are the only exceptions). Copy the pstack LICENSE verbatim to the skill's LICENSE.pstack. Add at least three expect briefs and two near briefs for the skill to packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json, one of them in Vietnamese, in the fixture's existing shape. Run bee dev regen so the rendered trees and the release manifest match, and commit them with the skill. Write docs/history/pstack-playbook-port/port-notes/skill-bee-figure-it-out.md listing each swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-figure-it-out/SKILL.md exists with a description under 300 characters",
    "the trigger fixture holds at least three expect and two near briefs for bee-figure-it-out",
    "no file under skills/bee-figure-it-out/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No other skill is edited",
    "No hand-picked model or subagent_type in the skill text"
   ]
  }
 },
 {
  "id": "ppb-23c",
  "feature": "pstack-playbook-port",
  "title": "Check bee-figure-it-out against its pstack source",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-23"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-23c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-figure-it-out/SKILL.md",
   "docs/history/pstack-playbook-port/port-notes/skill-bee-figure-it-out.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/figure-it-out/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack source files beside skills/bee-figure-it-out/. Report each source step or rule as present, dropped with a port-notes entry, or missing; any pstack-only tool name; any hand-picked model or subagent_type; and whether the description fits 300 characters and says when not to use the skill. Write the report to docs/history/pstack-playbook-port/reports/ppb-23c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-figure-it-out/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-24",
  "feature": "pstack-playbook-port",
  "title": "Restore the playbook steps that waited for the new skills",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-23"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/",
   ".claude-plugin/skills/bee-planning/",
   ".codex-plugin/skills/bee-planning/",
   ".claude/skills/bee-planning/",
   ".agents/skills/bee-planning/",
   ".opencode/skills/bee-planning/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/restored-steps.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "docs/history/pstack-playbook-port/port-notes/"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/"
  ],
  "affects_specs": [],
  "action": "Read every port-notes file. Each step dropped because its skill did not exist yet (architect, arena, interrogate, swarm, tdd, show-me-your-work, figure-it-out, reflect, recall, blast-radius, no-comments) goes back into its situation file, now naming the bee skill (bee-architect, bee-arena, bee-interrogate, bee-fanout, bee-tdd, bee-show-me-your-work, bee-figure-it-out, bee-reflect, bee-recall, bee-blast-radius, bee-comment-review), with its reason. Add one line after the Situations section of every class index and of session/index.md: 'No situation fits, or the work is large and cross-cutting: use bee-figure-it-out.' Mark each restored step in the port notes and write docs/history/pstack-playbook-port/port-notes/restored-steps.md listing what came back where. Run bee dev regen and commit the rendered trees and the manifest.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every class index and session/index.md names bee-figure-it-out as the fallback",
    "every step a port note marked as waiting for a skill is back, naming the bee skill"
   ],
   "prohibitions": [
    "No situation is added or removed",
    "No skill is edited"
   ]
  }
 },
 {
  "id": "ppb-24c",
  "feature": "pstack-playbook-port",
  "title": "Check the restored playbook steps",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-24"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-24c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/port-notes/restored-steps.md",
   "skills/bee-planning/playbooks/"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Check every step restored-steps.md lists is in its situation file and names a skill that exists in skills/, and that every index carries the fallback line. Write docs/history/pstack-playbook-port/reports/ppb-24c.md; no other edits.",
  "verify": "rg -c 'bee-figure-it-out' skills/bee-planning/playbooks/bugfix.md skills/bee-planning/playbooks/session/index.md",
  "must_haves": {
   "truths": [
    "the report lists every source step with present, dropped or missing"
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
| ppb-13 to ppb-23 | the new skill. Pass when `skill_triggers` is green and the description fits 300 characters | a Vietnamese brief in the fixture. Pass when present | a pstack-only tool or a hand-picked model in the skill. Pass when the checker finds none |
| ppb-24 | every class index and the session index. Pass when each names `bee-figure-it-out` and the fence is green | a step that waited for a skill. Pass when it is back and names the bee skill | a restored step naming a skill that does not exist. Pass when the checker finds none |
| ppb-25 | the session playbooks. Pass when no landing waits on GitHub's CLEAN state and no owner rebases | the merged situation files. Pass when each ends with its class Proof floor line | a finding still open. Pass when ppb-25c names none |

The run on main and on head for changed behavior: on main `skills/` holds no `bee-figure-it-out`; on head the skill exists and every class index names it as the fallback.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
