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

Slice 1 (ppb-1 to ppb-4t) is capped: orient lists situations, the plan check holds a situation citation, the bugfix class is ported, and the parity fence guards situation files. This is slice 2: the other 22 playbooks, eight class indexes and the session index.

Waves: the eight docs cells write disjoint directories and run in parallel, except ppb-12, which appends to the session index ppb-11 creates. Each checker follows its docs cell. Every docs cell carries `regen_obligation_ack: wave-barrier`; the leader runs `bee dev regen` once at wave close and commits the rendered trees and the release manifest. Port notes go to one file per class under `docs/history/pstack-playbook-port/port-notes/`.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ppb-5 | Port the perf-issue and hillclimb playbooks | `perf.md`, `perf/perf-issue.md`, `perf/hillclimb.md`, `docs/history/pstack-playbook-port/port-notes/perf.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-6 | Port the investigation, forensics and eval playbooks | `research.md`, `research/investigation.md`, `research/runtime-forensics.md`, `research/trace-forensics.md`, `research/eval.md`, `docs/history/pstack-playbook-port/port-notes/research.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-7 | Port the refactoring and visual-parity playbooks | `refactor.md`, `refactor/refactoring.md`, `refactor/visual-parity.md`, `docs/history/pstack-playbook-port/port-notes/refactor.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-8 | Port the prototype playbook | `spike.md`, `spike/prototype.md`, `docs/history/pstack-playbook-port/port-notes/spike.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-9 | Port the feature and multi-phase-plan playbooks | `feature.md`, `feature/feature.md`, `feature/multi-phase-plan.md`, `docs/history/pstack-playbook-port/port-notes/feature.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-10 | Give docs, release and content their situations and port authoring-a-skill | `docs.md`, `docs/docs.md`, `docs/authoring-a-skill.md`, `release.md`, `release/release.md`, `content.md`, `content/content.md`, `docs/history/pstack-playbook-port/port-notes/docs.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-11 | Port the session pickup, pause, autonomous, orchestrate and cleanup playbooks | `session/index.md`, `session/session-pickup.md`, `session/pause-safely.md`, `session/autonomous-run.md`, `session/orchestrate.md`, `session/worktree-cleanup.md`, `docs/history/pstack-playbook-port/port-notes/session-ppb-11.md` | — | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-12 | Port the babysit, shipping, autopilot and PR playbooks | `session/index.md`, `session/babysit.md`, `session/shipping.md`, `session/autopilot-full.md`, `session/autopilot-stack.md`, `session/opening-a-pr.md`, `docs/history/pstack-playbook-port/port-notes/session-ppb-12.md` | ppb-11 | the class index lists its situations with trigger phrases; each situation holds the pstack method in bee's terms | `cargo test --test class_playbook_parity — green:unit — docs port` |
| ppb-5c | Check the perf port against its sources | its report only | ppb-5 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-6c | Check the research port against its sources | its report only | ppb-6 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-7c | Check the refactor port against its sources | its report only | ppb-7 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-8c | Check the spike port against its sources | its report only | ppb-8 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-9c | Check the feature port against its sources | its report only | ppb-9 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-10c | Check the docs port against its sources | its report only | ppb-10 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-11c | Check the session port against its sources | its report only | ppb-11 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-12c | Check the session port against its sources | its report only | ppb-12 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ppb-5",
  "feature": "pstack-playbook-port",
  "title": "Port the perf-issue and hillclimb playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/perf.md",
   "skills/bee-planning/playbooks/perf/perf-issue.md",
   "skills/bee-planning/playbooks/perf/hillclimb.md",
   "docs/history/pstack-playbook-port/port-notes/perf.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/perf.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/perf-issue.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/hillclimb.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/perf.md",
   "skills/bee-planning/playbooks/perf/perf-issue.md",
   "skills/bee-planning/playbooks/perf/hillclimb.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the perf class. Write one situation file per row: playbooks/perf/perf-issue.md from pstack perf-issue.md; playbooks/perf/hillclimb.md from pstack hillclimb.md. perf-issue merges with the current perf.md steps per D3; the current sustained-target paragraph and plateau-pivot sentence move into hillclimb.md. Rewrite playbooks/perf.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/perf.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/perf.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on perf.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-6",
  "feature": "pstack-playbook-port",
  "title": "Port the investigation, forensics and eval playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/research.md",
   "skills/bee-planning/playbooks/research/investigation.md",
   "skills/bee-planning/playbooks/research/runtime-forensics.md",
   "skills/bee-planning/playbooks/research/trace-forensics.md",
   "skills/bee-planning/playbooks/research/eval.md",
   "docs/history/pstack-playbook-port/port-notes/research.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/research.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/investigation.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/runtime-forensics.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/trace-forensics.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/eval.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/research.md",
   "skills/bee-planning/playbooks/research/investigation.md",
   "skills/bee-planning/playbooks/research/runtime-forensics.md",
   "skills/bee-planning/playbooks/research/trace-forensics.md",
   "skills/bee-planning/playbooks/research/eval.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the research class. Write one situation file per row: playbooks/research/investigation.md from pstack investigation.md; playbooks/research/runtime-forensics.md from pstack runtime-forensics.md; playbooks/research/trace-forensics.md from pstack trace-forensics.md; playbooks/research/eval.md from pstack eval.md. investigation merges with the current research.md steps per D3, keeping the trace-and-provenance home pointer; the current live-runtime paragraph moves into runtime-forensics.md. Rewrite playbooks/research.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/research.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/research.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on research.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-7",
  "feature": "pstack-playbook-port",
  "title": "Port the refactoring and visual-parity playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/refactor.md",
   "skills/bee-planning/playbooks/refactor/refactoring.md",
   "skills/bee-planning/playbooks/refactor/visual-parity.md",
   "docs/history/pstack-playbook-port/port-notes/refactor.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/refactor.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/refactoring.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/visual-parity.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/refactor.md",
   "skills/bee-planning/playbooks/refactor/refactoring.md",
   "skills/bee-planning/playbooks/refactor/visual-parity.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the refactor class. Write one situation file per row: playbooks/refactor/refactoring.md from pstack refactoring.md; playbooks/refactor/visual-parity.md from pstack visual-parity.md. refactoring merges with the current refactor.md steps per D3. Rewrite playbooks/refactor.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/refactor.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/refactor.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on refactor.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-8",
  "feature": "pstack-playbook-port",
  "title": "Port the prototype playbook",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/spike.md",
   "skills/bee-planning/playbooks/spike/prototype.md",
   "docs/history/pstack-playbook-port/port-notes/spike.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/spike.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/prototype.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/spike.md",
   "skills/bee-planning/playbooks/spike/prototype.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the spike class. Write one situation file per row: playbooks/spike/prototype.md from pstack prototype.md. prototype merges with the current spike.md steps per D3. Rewrite playbooks/spike.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/spike.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/spike.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on spike.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-9",
  "feature": "pstack-playbook-port",
  "title": "Port the feature and multi-phase-plan playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/feature.md",
   "skills/bee-planning/playbooks/feature/feature.md",
   "skills/bee-planning/playbooks/feature/multi-phase-plan.md",
   "docs/history/pstack-playbook-port/port-notes/feature.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/feature.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/feature.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/multi-phase-plan.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/feature.md",
   "skills/bee-planning/playbooks/feature/feature.md",
   "skills/bee-planning/playbooks/feature/multi-phase-plan.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the feature class. Write one situation file per row: playbooks/feature/feature.md from pstack feature.md; playbooks/feature/multi-phase-plan.md from pstack multi-phase-plan.md. feature merges with the current feature.md steps per D3, keeping the statechart and formal-verification steps; multi-phase-plan keeps bee's phase-plan-vs-epic-map rule from skills/bee-planning/references/planning-reference.md by pointer. Rewrite playbooks/feature.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/feature.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/feature.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on feature.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-10",
  "feature": "pstack-playbook-port",
  "title": "Give docs, release and content their situations and port authoring-a-skill",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/docs.md",
   "skills/bee-planning/playbooks/docs/docs.md",
   "skills/bee-planning/playbooks/docs/authoring-a-skill.md",
   "skills/bee-planning/playbooks/release.md",
   "skills/bee-planning/playbooks/release/release.md",
   "skills/bee-planning/playbooks/content.md",
   "skills/bee-planning/playbooks/content/content.md",
   "docs/history/pstack-playbook-port/port-notes/docs.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "skills/bee-planning/playbooks/docs.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/authoring-a-skill.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/docs.md",
   "skills/bee-planning/playbooks/docs/docs.md",
   "skills/bee-planning/playbooks/docs/authoring-a-skill.md",
   "skills/bee-planning/playbooks/release.md",
   "skills/bee-planning/playbooks/release/release.md",
   "skills/bee-planning/playbooks/content.md",
   "skills/bee-planning/playbooks/content/content.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into the docs class. Write one situation file per row: playbooks/docs/docs.md from the current bee text; playbooks/docs/authoring-a-skill.md from pstack authoring-a-skill.md. docs/docs.md is the current docs.md steps moved unchanged; authoring-a-skill points at bee-writing-skills as its skill. Also move the current release.md steps unchanged into playbooks/release/release.md and the current content.md steps into playbooks/content/content.md, and rewrite release.md and content.md as class indexes listing that one situation each, Proof floor kept. Rewrite playbooks/docs.md as the class index: title, one intro line, the Situations section listing every situation above, then the class Proof floor line kept byte-identical. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report.  Write docs/history/pstack-playbook-port/port-notes/docs.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/docs.md && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every situation file listed on docs.md exists and carries its pstack source's steps",
    "every enforced line of the old class playbook survives in a situation file",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No playbook of another class is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-11",
  "feature": "pstack-playbook-port",
  "title": "Port the session pickup, pause, autonomous, orchestrate and cleanup playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "b749c98d-65e2-4346-b07f-6dc52dbd78ca",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/session/index.md",
   "skills/bee-planning/playbooks/session/session-pickup.md",
   "skills/bee-planning/playbooks/session/pause-safely.md",
   "skills/bee-planning/playbooks/session/autonomous-run.md",
   "skills/bee-planning/playbooks/session/orchestrate.md",
   "skills/bee-planning/playbooks/session/worktree-cleanup.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-11.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/session-pickup.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/pause-safely.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autonomous-run.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/orchestrate.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/worktree-cleanup.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/session/index.md",
   "skills/bee-planning/playbooks/session/session-pickup.md",
   "skills/bee-planning/playbooks/session/pause-safely.md",
   "skills/bee-planning/playbooks/session/autonomous-run.md",
   "skills/bee-planning/playbooks/session/orchestrate.md",
   "skills/bee-planning/playbooks/session/worktree-cleanup.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into playbooks/session/: session/session-pickup.md from pstack session-pickup.md; session/pause-safely.md from pstack pause-safely.md; session/autonomous-run.md from pstack autonomous-run.md; session/orchestrate.md from pstack orchestrate.md; session/worktree-cleanup.md from pstack worktree-cleanup.md. Create playbooks/session/index.md: title, one intro line saying these situations belong to a session, not a feature route, and a Situations section in the contract grammar listing the five situations of this cell. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report. bee counterparts: session pickup and pause safely use .bee/HANDOFF.json, bee state handoff write and bee state handoff adopt; autonomous run and orchestrate use bee-herding and its control loop; worktree cleanup uses bee worktree prune; babysit, shipping, autopilot-full and autopilot-stack land work with bee worktree merge on green without asking (D4, uat_stop close) and stop at the uat door of bee close, which stays the owner's; opening a PR follows the ship_visibility draft-PR rule. Write docs/history/pstack-playbook-port/port-notes/session-ppb-11.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every session situation file is listed on session/index.md and the reverse",
    "a playbook that lands work merges on green and stops at the uat door",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No class playbook is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-12",
  "feature": "pstack-playbook-port",
  "title": "Port the babysit, shipping, autopilot and PR playbooks",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-11"
  ],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "b749c98d-65e2-4346-b07f-6dc52dbd78ca",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/session/index.md",
   "skills/bee-planning/playbooks/session/babysit.md",
   "skills/bee-planning/playbooks/session/shipping.md",
   "skills/bee-planning/playbooks/session/autopilot-full.md",
   "skills/bee-planning/playbooks/session/autopilot-stack.md",
   "skills/bee-planning/playbooks/session/opening-a-pr.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-12.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/babysit.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/shipping.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autopilot-full.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autopilot-stack.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/opening-a-pr.md"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/session/index.md",
   "skills/bee-planning/playbooks/session/babysit.md",
   "skills/bee-planning/playbooks/session/shipping.md",
   "skills/bee-planning/playbooks/session/autopilot-full.md",
   "skills/bee-planning/playbooks/session/autopilot-stack.md",
   "skills/bee-planning/playbooks/session/opening-a-pr.md"
  ],
  "affects_specs": [],
  "regen_obligation_ack": "wave-barrier",
  "action": "Port into playbooks/session/: session/babysit.md from pstack babysit.md; session/shipping.md from pstack shipping.md; session/autopilot-full.md from pstack autopilot-full.md; session/autopilot-stack.md from pstack autopilot-stack.md; session/opening-a-pr.md from pstack opening-a-pr.md. Append the five situations of this cell to the Situations section of playbooks/session/index.md, keeping the existing lines. Index grammar (contract 913115b3): a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, <name> matching [a-z0-9-]+; the section ends at the next '## ' heading. Port rules: the pstack method text is the body (D1); every pstack tool is swapped for its bee counterpart per D2 (how and why become bee-how and bee-why gathers, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface, Opening a PR becomes the bee close and worktree merge path, show-me-your-work becomes a decision trail under docs/history/<feature>/); a step whose tool bee does not have yet (architect, arena, interrogate, swarm, tdd, figure-it-out, show-me-your-work) is dropped from the playbook text and named in your port notes file, because slice 3 brings those skills and re-adds the steps. Name no skill that does not exist in skills/ today. No line under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent. Every step states why it holds, as decision 61b52e96 requires of class playbooks; keep the pstack reason where it gives one. Follow bugfix.md and bugfix/bug-fix.md as the worked example of shape and voice. Do not run bee dev regen: the leader runs it once at wave close (regen_obligation_ack wave-barrier). Sibling cells edit other class directories in this same worktree at the same time: if the parity fence is red only for a directory you did not touch, that is a sibling mid-edit; wait, rerun, and name it in your report. bee counterparts: session pickup and pause safely use .bee/HANDOFF.json, bee state handoff write and bee state handoff adopt; autonomous run and orchestrate use bee-herding and its control loop; worktree cleanup uses bee worktree prune; babysit, shipping, autopilot-full and autopilot-stack land work with bee worktree merge on green without asking (D4, uat_stop close) and stop at the uat door of bee close, which stays the owner's; opening a PR follows the ship_visibility draft-PR rule. Write docs/history/pstack-playbook-port/port-notes/session-ppb-12.md with one section per ported playbook listing each tool swap and each dropped step.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "every session situation file is listed on session/index.md and the reverse",
    "a playbook that lands work merges on green and stops at the uat door",
    "no file under skills/bee-planning/playbooks/ names Cursor, Graphite, bugbot or poteto-agent"
   ],
   "prohibitions": [
    "No class playbook is edited",
    "No skill named that does not exist in skills/"
   ]
  }
 },
 {
  "id": "ppb-5c",
  "feature": "pstack-playbook-port",
  "title": "Check the perf port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-5"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-5c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/perf.md",
   "docs/history/pstack-playbook-port/port-notes/perf.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/perf-issue.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/hillclimb.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old perf playbook text (git show of the commit before ppb-5) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-5c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-6c",
  "feature": "pstack-playbook-port",
  "title": "Check the research port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-6"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-6c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/research.md",
   "docs/history/pstack-playbook-port/port-notes/research.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/investigation.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/runtime-forensics.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/trace-forensics.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/eval.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old research playbook text (git show of the commit before ppb-6) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-6c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-7c",
  "feature": "pstack-playbook-port",
  "title": "Check the refactor port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-7"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-7c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/refactor.md",
   "docs/history/pstack-playbook-port/port-notes/refactor.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/refactoring.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/visual-parity.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old refactor playbook text (git show of the commit before ppb-7) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-7c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-8c",
  "feature": "pstack-playbook-port",
  "title": "Check the spike port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-8"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-8c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/spike.md",
   "docs/history/pstack-playbook-port/port-notes/spike.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/prototype.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old spike playbook text (git show of the commit before ppb-8) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-8c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-9c",
  "feature": "pstack-playbook-port",
  "title": "Check the feature port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-9"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-9c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/feature.md",
   "docs/history/pstack-playbook-port/port-notes/feature.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/feature.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/multi-phase-plan.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old feature playbook text (git show of the commit before ppb-9) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-9c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-10c",
  "feature": "pstack-playbook-port",
  "title": "Check the docs port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-10"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-10c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/docs.md",
   "docs/history/pstack-playbook-port/port-notes/docs.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/authoring-a-skill.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old docs playbook text (git show of the commit before ppb-10) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-10c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-11c",
  "feature": "pstack-playbook-port",
  "title": "Check the session port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-11"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-11c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/session/index.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-11.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/session-pickup.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/pause-safely.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autonomous-run.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/orchestrate.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/worktree-cleanup.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old session playbook text (git show of the commit before ppb-11) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-11c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
  "id": "ppb-12c",
  "feature": "pstack-playbook-port",
  "title": "Check the session port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-12"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-12c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/session/index.md",
   "docs/history/pstack-playbook-port/port-notes/session-ppb-12.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/babysit.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/shipping.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autopilot-full.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/autopilot-stack.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/opening-a-pr.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read each pstack source and the old session playbook text (git show of the commit before ppb-12) beside the new files. Report each source step and each old enforced line as present, or dropped with its port-notes entry, or missing; any pstack-only tool name under skills/bee-planning/playbooks/; any skill named that does not exist in skills/; and whether each index line matches the contract grammar. Write the report to docs/history/pstack-playbook-port/reports/ppb-12c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/playbooks/ ; test $? -eq 1",
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
| ppb-5 to ppb-10 | class index and its situation files. Pass when the parity fence is green | a merged situation keeps the old enforced lines. Pass when its checker lists every old line present | a pstack-only tool under `skills/bee-planning/playbooks/`. Pass when `rg` finds none |
| ppb-11, ppb-12 | `session/index.md` and its files. Pass when the fence is green | a landing playbook. Pass when it merges on green and stops at the uat door | an index line with no file. Pass when the fence is red naming it |
| ppb-5c to ppb-12c | each source step. Pass when present, or dropped with a port-notes entry | a skill named that does not exist. Pass when the report names none | a missing step. Pass when the report names it |

The run on main and on head for changed behavior: `bee orient --class perf` on main prints the playbook line only; on head it adds one `situation:` line per perf situation.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
