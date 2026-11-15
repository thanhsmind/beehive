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

Slices 1 to 3 are capped: situations route through orient and the plan check, all 23 pstack playbooks are ported, the slice 2 findings are fixed, and the eleven pstack skills the playbooks cite exist as bee skills with the `bee-figure-it-out` fallback on every index. This is slice 4 (D5, decision `6ca3cd62`): thirteen pstack principles come in as bee principles with index rows and expertise depth (ppb-26 to ppb-29); pstack improvements merge into the nine principles bee already has (ppb-30) and into bee-how, bee-why, bee-teach, bee-unslop, bee-technical-writing and the verification skills (ppb-31 to ppb-36); ppb-37 restores the principle pointers the ports dropped and closes the slice 3 leftovers in reports ppb-24c and ppb-25c.

Waves: every docs cell writes under `skills/`, so they run one after another, each with its own `bee dev regen`; each checker runs beside the next docs cell.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ppb-26 | Port the pstack principles foundational-thinking, redesign-from-first-principles, outcome-oriented-execution | `skills/bee-principle-foundational-thinking/`, `skills/bee-principle-redesign-from-first-principles/`, `skills/bee-principle-outcome-oriented-execution/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-26c | Check the principles foundational-thinking, redesign-from-first-principles, outcome-oriented-execution | its report only | ppb-26 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-27 | Port the pstack principles laziness-protocol, minimize-reader-load, model-the-domain | `skills/bee-principle-laziness-protocol/`, `skills/bee-principle-minimize-reader-load/`, `skills/bee-principle-model-the-domain/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-27c | Check the principles laziness-protocol, minimize-reader-load, model-the-domain | its report only | ppb-27 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-28 | Port the pstack principles prove-it-works, sequence-verifiable-units, make-operations-idempotent | `skills/bee-principle-prove-it-works/`, `skills/bee-principle-sequence-verifiable-units/`, `skills/bee-principle-make-operations-idempotent/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-28c | Check the principles prove-it-works, sequence-verifiable-units, make-operations-idempotent | its report only | ppb-28 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-29 | Port the pstack principles guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first | `skills/bee-principle-guard-the-context-window/`, `skills/bee-principle-never-block-on-the-human/`, `skills/bee-principle-exhaust-the-design-space/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-29c | Check the principles guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first | its report only | ppb-29 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-30 | Merge pstack improvements into the principles bee already has | `skills/bee-principle-crash-site-versus-fault-site/`, `skills/bee-principle-reproduce-first/`, `skills/bee-principle-make-illegal-states-unrepresentable/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-30c | Check the merged principles | its report only | ppb-30 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-31 | Merge pstack how improvements into bee-how | `skills/bee-how/SKILL.md`, `skills/bee-researching/references/trace-and-provenance.md`, `.claude-plugin/skills/bee-how/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-31c | Check the bee-how merge | its report only | ppb-31 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-32 | Merge pstack why improvements into bee-why | `skills/bee-why/SKILL.md`, `skills/bee-researching/references/trace-and-provenance.md`, `.claude-plugin/skills/bee-why/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-32c | Check the bee-why merge | its report only | ppb-32 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-33 | Merge pstack teach improvements into bee-teach | `skills/bee-teach/SKILL.md`, `.claude-plugin/skills/bee-teach/`, `.codex-plugin/skills/bee-teach/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-33c | Check the bee-teach merge | its report only | ppb-33 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-34 | Merge pstack unslop improvements into bee-unslop | `skills/bee-unslop/SKILL.md`, `.claude-plugin/skills/bee-unslop/`, `.codex-plugin/skills/bee-unslop/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-34c | Check the bee-unslop merge | its report only | ppb-34 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-35 | Merge pstack technical-writing improvements into bee-technical-writing | `skills/bee-technical-writing/SKILL.md`, `.claude-plugin/skills/bee-technical-writing/`, `.codex-plugin/skills/bee-technical-writing/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-35c | Check the bee-technical-writing merge | its report only | ppb-35 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-36 | Merge pstack create-verification-skill improvements into bee-verifying | `skills/bee-verifying/SKILL.md`, `skills/bee-verify-upkeep/SKILL.md`, `.claude-plugin/skills/bee-verifying/`, the rendered trees, the release manifest | — | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-36c | Check the bee-verifying merge | its report only | ppb-36 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |
| ppb-37 | Restore principle pointers and close the slice 3 leftovers | `skills/bee-planning/playbooks/`, `skills/bee-architect/`, `skills/bee-arena/`, the rendered trees, the release manifest | ppb-29, ppb-30 | the new or fixed skill or playbook text is in bee terms and renders to every plugin tree | `cargo test — green:unit — docs port` |
| ppb-37c | Check the restored principle pointers and leftovers | its report only | ppb-37 | a report of every source step present, dropped or missing | `rg pstack-only tools — green:static — checker` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ppb-26",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack principles foundational-thinking, redesign-from-first-principles, outcome-oriented-execution",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-principle-foundational-thinking/",
   "skills/bee-principle-redesign-from-first-principles/",
   "skills/bee-principle-outcome-oriented-execution/",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "expertise/planning.md",
   "expertise/architecture.md",
   "docs/history/codex-harness-hardening/release-manifest.json",
   ".claude-plugin/skills/bee-principle-foundational-thinking/",
   ".claude-plugin/skills/bee-principle-redesign-from-first-principles/",
   ".claude-plugin/skills/bee-principle-outcome-oriented-execution/",
   ".codex-plugin/skills/bee-principle-foundational-thinking/",
   ".codex-plugin/skills/bee-principle-redesign-from-first-principles/",
   ".codex-plugin/skills/bee-principle-outcome-oriented-execution/",
   ".claude/skills/bee-principle-foundational-thinking/",
   ".claude/skills/bee-principle-redesign-from-first-principles/",
   ".claude/skills/bee-principle-outcome-oriented-execution/",
   ".agents/skills/bee-principle-foundational-thinking/",
   ".agents/skills/bee-principle-redesign-from-first-principles/",
   ".agents/skills/bee-principle-outcome-oriented-execution/",
   ".opencode/skills/bee-principle-foundational-thinking/",
   ".opencode/skills/bee-principle-redesign-from-first-principles/",
   ".opencode/skills/bee-principle-outcome-oriented-execution/",
   "docs/history/pstack-playbook-port/port-notes/principles-ppb-26.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-foundational-thinking/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-redesign-from-first-principles/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-outcome-oriented-execution/SKILL.md",
   "expertise/planning.md",
   "expertise/architecture.md"
  ],
  "affects_skills": [
   "skills/bee-principle-foundational-thinking/SKILL.md",
   "skills/bee-principle-redesign-from-first-principles/SKILL.md",
   "skills/bee-principle-outcome-oriented-execution/SKILL.md"
  ],
  "affects_specs": [],
  "action": "For each slug, create skills/bee-principle-<slug>/SKILL.md from pstack principle-<slug>, in the exact shape of skills/bee-principle-subtract-before-you-add/SKILL.md (description 'Apply when ...', body, Check, Not when, Why, Depth). Put the depth into a new section of the expertise guide named in files whose craft it belongs to, and point Depth at that section. Add one row per slug to the '## Principle homes' section of docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, in the existing row grammar: the guide anchor, a spoken line, a classes line naming the route classes it fires for (from the nine), then applied_at. Pick the classes from the pstack 'when it applies' line. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/principles-ppb-26.md.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-principle-foundational-thinking/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-redesign-from-first-principles/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-outcome-oriented-execution/SKILL.md exists and has an index row with spoken and classes lines",
    "principle_index_parity is green"
   ],
   "prohibitions": [
    "No existing principle row is changed"
   ]
  }
 },
 {
  "id": "ppb-26c",
  "feature": "pstack-playbook-port",
  "title": "Check the principles foundational-thinking, redesign-from-first-principles, outcome-oriented-execution",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-26"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-26c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-foundational-thinking/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-redesign-from-first-principles/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-outcome-oriented-execution/SKILL.md",
   "skills/bee-principle-foundational-thinking/SKILL.md",
   "skills/bee-principle-redesign-from-first-principles/SKILL.md",
   "skills/bee-principle-outcome-oriented-execution/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-26 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-26c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-27",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack principles laziness-protocol, minimize-reader-load, model-the-domain",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-principle-laziness-protocol/",
   "skills/bee-principle-minimize-reader-load/",
   "skills/bee-principle-model-the-domain/",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "expertise/architecture.md",
   "expertise/review.md",
   "docs/history/codex-harness-hardening/release-manifest.json",
   ".claude-plugin/skills/bee-principle-laziness-protocol/",
   ".claude-plugin/skills/bee-principle-minimize-reader-load/",
   ".claude-plugin/skills/bee-principle-model-the-domain/",
   ".codex-plugin/skills/bee-principle-laziness-protocol/",
   ".codex-plugin/skills/bee-principle-minimize-reader-load/",
   ".codex-plugin/skills/bee-principle-model-the-domain/",
   ".claude/skills/bee-principle-laziness-protocol/",
   ".claude/skills/bee-principle-minimize-reader-load/",
   ".claude/skills/bee-principle-model-the-domain/",
   ".agents/skills/bee-principle-laziness-protocol/",
   ".agents/skills/bee-principle-minimize-reader-load/",
   ".agents/skills/bee-principle-model-the-domain/",
   ".opencode/skills/bee-principle-laziness-protocol/",
   ".opencode/skills/bee-principle-minimize-reader-load/",
   ".opencode/skills/bee-principle-model-the-domain/",
   "docs/history/pstack-playbook-port/port-notes/principles-ppb-27.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-laziness-protocol/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-minimize-reader-load/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-model-the-domain/SKILL.md",
   "expertise/architecture.md",
   "expertise/review.md"
  ],
  "affects_skills": [
   "skills/bee-principle-laziness-protocol/SKILL.md",
   "skills/bee-principle-minimize-reader-load/SKILL.md",
   "skills/bee-principle-model-the-domain/SKILL.md"
  ],
  "affects_specs": [],
  "action": "For each slug, create skills/bee-principle-<slug>/SKILL.md from pstack principle-<slug>, in the exact shape of skills/bee-principle-subtract-before-you-add/SKILL.md (description 'Apply when ...', body, Check, Not when, Why, Depth). Put the depth into a new section of the expertise guide named in files whose craft it belongs to, and point Depth at that section. Add one row per slug to the '## Principle homes' section of docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, in the existing row grammar: the guide anchor, a spoken line, a classes line naming the route classes it fires for (from the nine), then applied_at. Pick the classes from the pstack 'when it applies' line. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/principles-ppb-27.md.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-principle-laziness-protocol/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-minimize-reader-load/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-model-the-domain/SKILL.md exists and has an index row with spoken and classes lines",
    "principle_index_parity is green"
   ],
   "prohibitions": [
    "No existing principle row is changed"
   ]
  }
 },
 {
  "id": "ppb-27c",
  "feature": "pstack-playbook-port",
  "title": "Check the principles laziness-protocol, minimize-reader-load, model-the-domain",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-27"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-27c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-laziness-protocol/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-minimize-reader-load/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-model-the-domain/SKILL.md",
   "skills/bee-principle-laziness-protocol/SKILL.md",
   "skills/bee-principle-minimize-reader-load/SKILL.md",
   "skills/bee-principle-model-the-domain/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-27 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-27c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-28",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack principles prove-it-works, sequence-verifiable-units, make-operations-idempotent",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-principle-prove-it-works/",
   "skills/bee-principle-sequence-verifiable-units/",
   "skills/bee-principle-make-operations-idempotent/",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "expertise/tests.md",
   "expertise/operations.md",
   "expertise/planning.md",
   "docs/history/codex-harness-hardening/release-manifest.json",
   ".claude-plugin/skills/bee-principle-prove-it-works/",
   ".claude-plugin/skills/bee-principle-sequence-verifiable-units/",
   ".claude-plugin/skills/bee-principle-make-operations-idempotent/",
   ".codex-plugin/skills/bee-principle-prove-it-works/",
   ".codex-plugin/skills/bee-principle-sequence-verifiable-units/",
   ".codex-plugin/skills/bee-principle-make-operations-idempotent/",
   ".claude/skills/bee-principle-prove-it-works/",
   ".claude/skills/bee-principle-sequence-verifiable-units/",
   ".claude/skills/bee-principle-make-operations-idempotent/",
   ".agents/skills/bee-principle-prove-it-works/",
   ".agents/skills/bee-principle-sequence-verifiable-units/",
   ".agents/skills/bee-principle-make-operations-idempotent/",
   ".opencode/skills/bee-principle-prove-it-works/",
   ".opencode/skills/bee-principle-sequence-verifiable-units/",
   ".opencode/skills/bee-principle-make-operations-idempotent/",
   "docs/history/pstack-playbook-port/port-notes/principles-ppb-28.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-prove-it-works/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-sequence-verifiable-units/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-make-operations-idempotent/SKILL.md",
   "expertise/tests.md",
   "expertise/operations.md",
   "expertise/planning.md"
  ],
  "affects_skills": [
   "skills/bee-principle-prove-it-works/SKILL.md",
   "skills/bee-principle-sequence-verifiable-units/SKILL.md",
   "skills/bee-principle-make-operations-idempotent/SKILL.md"
  ],
  "affects_specs": [],
  "action": "For each slug, create skills/bee-principle-<slug>/SKILL.md from pstack principle-<slug>, in the exact shape of skills/bee-principle-subtract-before-you-add/SKILL.md (description 'Apply when ...', body, Check, Not when, Why, Depth). Put the depth into a new section of the expertise guide named in files whose craft it belongs to, and point Depth at that section. Add one row per slug to the '## Principle homes' section of docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, in the existing row grammar: the guide anchor, a spoken line, a classes line naming the route classes it fires for (from the nine), then applied_at. Pick the classes from the pstack 'when it applies' line. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/principles-ppb-28.md.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-principle-prove-it-works/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-sequence-verifiable-units/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-make-operations-idempotent/SKILL.md exists and has an index row with spoken and classes lines",
    "principle_index_parity is green"
   ],
   "prohibitions": [
    "No existing principle row is changed"
   ]
  }
 },
 {
  "id": "ppb-28c",
  "feature": "pstack-playbook-port",
  "title": "Check the principles prove-it-works, sequence-verifiable-units, make-operations-idempotent",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-28"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-28c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-prove-it-works/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-sequence-verifiable-units/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-make-operations-idempotent/SKILL.md",
   "skills/bee-principle-prove-it-works/SKILL.md",
   "skills/bee-principle-sequence-verifiable-units/SKILL.md",
   "skills/bee-principle-make-operations-idempotent/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-28 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-28c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-29",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack principles guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-principle-guard-the-context-window/",
   "skills/bee-principle-never-block-on-the-human/",
   "skills/bee-principle-exhaust-the-design-space/",
   "skills/bee-principle-experience-first/",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "expertise/thinking.md",
   "expertise/decisions.md",
   "docs/history/codex-harness-hardening/release-manifest.json",
   ".claude-plugin/skills/bee-principle-guard-the-context-window/",
   ".claude-plugin/skills/bee-principle-never-block-on-the-human/",
   ".claude-plugin/skills/bee-principle-exhaust-the-design-space/",
   ".claude-plugin/skills/bee-principle-experience-first/",
   ".codex-plugin/skills/bee-principle-guard-the-context-window/",
   ".codex-plugin/skills/bee-principle-never-block-on-the-human/",
   ".codex-plugin/skills/bee-principle-exhaust-the-design-space/",
   ".codex-plugin/skills/bee-principle-experience-first/",
   ".claude/skills/bee-principle-guard-the-context-window/",
   ".claude/skills/bee-principle-never-block-on-the-human/",
   ".claude/skills/bee-principle-exhaust-the-design-space/",
   ".claude/skills/bee-principle-experience-first/",
   ".agents/skills/bee-principle-guard-the-context-window/",
   ".agents/skills/bee-principle-never-block-on-the-human/",
   ".agents/skills/bee-principle-exhaust-the-design-space/",
   ".agents/skills/bee-principle-experience-first/",
   ".opencode/skills/bee-principle-guard-the-context-window/",
   ".opencode/skills/bee-principle-never-block-on-the-human/",
   ".opencode/skills/bee-principle-exhaust-the-design-space/",
   ".opencode/skills/bee-principle-experience-first/",
   "docs/history/pstack-playbook-port/port-notes/principles-ppb-29.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md",
   "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-guard-the-context-window/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-never-block-on-the-human/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-exhaust-the-design-space/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-experience-first/SKILL.md",
   "expertise/thinking.md",
   "expertise/decisions.md"
  ],
  "affects_skills": [
   "skills/bee-principle-guard-the-context-window/SKILL.md",
   "skills/bee-principle-never-block-on-the-human/SKILL.md",
   "skills/bee-principle-exhaust-the-design-space/SKILL.md",
   "skills/bee-principle-experience-first/SKILL.md"
  ],
  "affects_specs": [],
  "action": "For each slug, create skills/bee-principle-<slug>/SKILL.md from pstack principle-<slug>, in the exact shape of skills/bee-principle-subtract-before-you-add/SKILL.md (description 'Apply when ...', body, Check, Not when, Why, Depth). Put the depth into a new section of the expertise guide named in files whose craft it belongs to, and point Depth at that section. Add one row per slug to the '## Principle homes' section of docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, in the existing row grammar: the guide anchor, a spoken line, a classes line naming the route classes it fires for (from the nine), then applied_at. Pick the classes from the pstack 'when it applies' line. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/principles-ppb-29.md.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "skills/bee-principle-guard-the-context-window/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-never-block-on-the-human/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-exhaust-the-design-space/SKILL.md exists and has an index row with spoken and classes lines",
    "skills/bee-principle-experience-first/SKILL.md exists and has an index row with spoken and classes lines",
    "principle_index_parity is green"
   ],
   "prohibitions": [
    "No existing principle row is changed"
   ]
  }
 },
 {
  "id": "ppb-29c",
  "feature": "pstack-playbook-port",
  "title": "Check the principles guard-the-context-window, never-block-on-the-human, exhaust-the-design-space, experience-first",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-29"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-29c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-guard-the-context-window/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-never-block-on-the-human/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-exhaust-the-design-space/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-experience-first/SKILL.md",
   "skills/bee-principle-guard-the-context-window/SKILL.md",
   "skills/bee-principle-never-block-on-the-human/SKILL.md",
   "skills/bee-principle-exhaust-the-design-space/SKILL.md",
   "skills/bee-principle-experience-first/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-29 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-29c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-30",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack improvements into the principles bee already has",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-principle-crash-site-versus-fault-site/",
   "skills/bee-principle-reproduce-first/",
   "skills/bee-principle-make-illegal-states-unrepresentable/",
   "skills/bee-principle-boundary-discipline/",
   "skills/bee-principle-build-the-lever/",
   "skills/bee-principle-encode-lessons-in-structure/",
   "skills/bee-principle-migrate-callers-then-delete/",
   "skills/bee-principle-separate-before-serializing-shared-state/",
   "skills/bee-principle-subtract-before-you-add/",
   ".claude-plugin/skills/bee-principle-crash-site-versus-fault-site/",
   ".claude-plugin/skills/bee-principle-reproduce-first/",
   ".claude-plugin/skills/bee-principle-make-illegal-states-unrepresentable/",
   ".claude-plugin/skills/bee-principle-boundary-discipline/",
   ".claude-plugin/skills/bee-principle-build-the-lever/",
   ".claude-plugin/skills/bee-principle-encode-lessons-in-structure/",
   ".claude-plugin/skills/bee-principle-migrate-callers-then-delete/",
   ".claude-plugin/skills/bee-principle-separate-before-serializing-shared-state/",
   ".claude-plugin/skills/bee-principle-subtract-before-you-add/",
   ".codex-plugin/skills/bee-principle-crash-site-versus-fault-site/",
   ".codex-plugin/skills/bee-principle-reproduce-first/",
   ".codex-plugin/skills/bee-principle-make-illegal-states-unrepresentable/",
   ".codex-plugin/skills/bee-principle-boundary-discipline/",
   ".codex-plugin/skills/bee-principle-build-the-lever/",
   ".codex-plugin/skills/bee-principle-encode-lessons-in-structure/",
   ".codex-plugin/skills/bee-principle-migrate-callers-then-delete/",
   ".codex-plugin/skills/bee-principle-separate-before-serializing-shared-state/",
   ".codex-plugin/skills/bee-principle-subtract-before-you-add/",
   ".claude/skills/bee-principle-crash-site-versus-fault-site/",
   ".claude/skills/bee-principle-reproduce-first/",
   ".claude/skills/bee-principle-make-illegal-states-unrepresentable/",
   ".claude/skills/bee-principle-boundary-discipline/",
   ".claude/skills/bee-principle-build-the-lever/",
   ".claude/skills/bee-principle-encode-lessons-in-structure/",
   ".claude/skills/bee-principle-migrate-callers-then-delete/",
   ".claude/skills/bee-principle-separate-before-serializing-shared-state/",
   ".claude/skills/bee-principle-subtract-before-you-add/",
   ".agents/skills/bee-principle-crash-site-versus-fault-site/",
   ".agents/skills/bee-principle-reproduce-first/",
   ".agents/skills/bee-principle-make-illegal-states-unrepresentable/",
   ".agents/skills/bee-principle-boundary-discipline/",
   ".agents/skills/bee-principle-build-the-lever/",
   ".agents/skills/bee-principle-encode-lessons-in-structure/",
   ".agents/skills/bee-principle-migrate-callers-then-delete/",
   ".agents/skills/bee-principle-separate-before-serializing-shared-state/",
   ".agents/skills/bee-principle-subtract-before-you-add/",
   ".opencode/skills/bee-principle-crash-site-versus-fault-site/",
   ".opencode/skills/bee-principle-reproduce-first/",
   ".opencode/skills/bee-principle-make-illegal-states-unrepresentable/",
   ".opencode/skills/bee-principle-boundary-discipline/",
   ".opencode/skills/bee-principle-build-the-lever/",
   ".opencode/skills/bee-principle-encode-lessons-in-structure/",
   ".opencode/skills/bee-principle-migrate-callers-then-delete/",
   ".opencode/skills/bee-principle-separate-before-serializing-shared-state/",
   ".opencode/skills/bee-principle-subtract-before-you-add/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-30.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-crash-site-versus-fault-site/SKILL.md",
   "skills/bee-principle-reproduce-first/SKILL.md",
   "skills/bee-principle-make-illegal-states-unrepresentable/SKILL.md",
   "skills/bee-principle-boundary-discipline/SKILL.md",
   "skills/bee-principle-build-the-lever/SKILL.md",
   "skills/bee-principle-encode-lessons-in-structure/SKILL.md",
   "skills/bee-principle-migrate-callers-then-delete/SKILL.md",
   "skills/bee-principle-separate-before-serializing-shared-state/SKILL.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-fix-root-causes/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-type-system-discipline/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-boundary-discipline/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-build-the-lever/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-encode-lessons-in-structure/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-migrate-callers-then-delete-legacy-apis/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-separate-before-serializing-shared-state/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/principle-subtract-before-you-add/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-principle-crash-site-versus-fault-site/SKILL.md",
   "skills/bee-principle-reproduce-first/SKILL.md",
   "skills/bee-principle-make-illegal-states-unrepresentable/SKILL.md",
   "skills/bee-principle-boundary-discipline/SKILL.md",
   "skills/bee-principle-build-the-lever/SKILL.md",
   "skills/bee-principle-encode-lessons-in-structure/SKILL.md",
   "skills/bee-principle-migrate-callers-then-delete/SKILL.md",
   "skills/bee-principle-separate-before-serializing-shared-state/SKILL.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead). fix-root-causes merges into bee-principle-crash-site-versus-fault-site and bee-principle-reproduce-first; type-system-discipline into bee-principle-make-illegal-states-unrepresentable; each other pstack principle into its bee namesake (decision 6ca3cd62). No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-30.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-30c",
  "feature": "pstack-playbook-port",
  "title": "Check the merged principles",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-30"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-30c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-principle-crash-site-versus-fault-site/SKILL.md",
   "skills/bee-principle-reproduce-first/SKILL.md",
   "skills/bee-principle-make-illegal-states-unrepresentable/SKILL.md",
   "skills/bee-principle-boundary-discipline/SKILL.md",
   "skills/bee-principle-build-the-lever/SKILL.md",
   "skills/bee-principle-encode-lessons-in-structure/SKILL.md",
   "skills/bee-principle-migrate-callers-then-delete/SKILL.md",
   "skills/bee-principle-separate-before-serializing-shared-state/SKILL.md",
   "skills/bee-principle-subtract-before-you-add/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-30 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-30c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-31",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack how improvements into bee-how",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-how/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md",
   ".claude-plugin/skills/bee-how/",
   ".codex-plugin/skills/bee-how/",
   ".claude/skills/bee-how/",
   ".agents/skills/bee-how/",
   ".opencode/skills/bee-how/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-31.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/how/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-how/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead). Trace stays single-homed in bee-researching/references/trace-and-provenance.md; a pstack how rule bee-how lacks goes into Trace there, and bee-how stays a door. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-31.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-31c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-how merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-31"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-31c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/how/SKILL.md",
   "skills/bee-how/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-31 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-31c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-32",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack why improvements into bee-why",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-why/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md",
   ".claude-plugin/skills/bee-why/",
   ".codex-plugin/skills/bee-why/",
   ".claude/skills/bee-why/",
   ".agents/skills/bee-why/",
   ".opencode/skills/bee-why/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-32.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-why/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/why/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-why/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead). Provenance sweep stays single-homed in bee-researching/references/trace-and-provenance.md; a pstack why rule bee-why lacks goes there. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-32.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-32c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-why merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-32"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-32c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/why/SKILL.md",
   "skills/bee-why/SKILL.md",
   "skills/bee-researching/references/trace-and-provenance.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-32 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-32c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-33",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack teach improvements into bee-teach",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-teach/SKILL.md",
   ".claude-plugin/skills/bee-teach/",
   ".codex-plugin/skills/bee-teach/",
   ".claude/skills/bee-teach/",
   ".agents/skills/bee-teach/",
   ".opencode/skills/bee-teach/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-33.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-teach/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/teach/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-teach/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead).  No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-33.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-33c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-teach merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-33"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-33c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/teach/SKILL.md",
   "skills/bee-teach/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-33 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-33c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-34",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack unslop improvements into bee-unslop",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-unslop/SKILL.md",
   ".claude-plugin/skills/bee-unslop/",
   ".codex-plugin/skills/bee-unslop/",
   ".claude/skills/bee-unslop/",
   ".agents/skills/bee-unslop/",
   ".opencode/skills/bee-unslop/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-34.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-unslop/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/unslop/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-unslop/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead).  No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-34.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-34c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-unslop merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-34"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-34c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/unslop/SKILL.md",
   "skills/bee-unslop/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-34 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-34c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-35",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack technical-writing improvements into bee-technical-writing",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-technical-writing/SKILL.md",
   ".claude-plugin/skills/bee-technical-writing/",
   ".codex-plugin/skills/bee-technical-writing/",
   ".claude/skills/bee-technical-writing/",
   ".agents/skills/bee-technical-writing/",
   ".opencode/skills/bee-technical-writing/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-35.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-technical-writing/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/technical-writing/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-technical-writing/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead).  No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-35.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-35c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-technical-writing merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-35"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-35c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/technical-writing/SKILL.md",
   "skills/bee-technical-writing/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-35 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-35c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-36",
  "feature": "pstack-playbook-port",
  "title": "Merge pstack create-verification-skill improvements into bee-verifying",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-verifying/SKILL.md",
   "skills/bee-verify-upkeep/SKILL.md",
   ".claude-plugin/skills/bee-verifying/",
   ".codex-plugin/skills/bee-verifying/",
   ".claude/skills/bee-verifying/",
   ".agents/skills/bee-verifying/",
   ".opencode/skills/bee-verifying/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/merge-ppb-36.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-verifying/SKILL.md",
   "skills/bee-verify-upkeep/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/create-verification-skill/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/maintain-verification-skill/SKILL.md"
  ],
  "affects_skills": [
   "skills/bee-verifying/SKILL.md",
   "skills/bee-verify-upkeep/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Read the pstack source beside the bee skill that already covers it (D5: the bee skill stays bee's). Merge in each pstack rule, step or check the bee skill lacks, in the bee skill's voice and structure; keep every bee rule, and never add a second copy of a rule bee already homes elsewhere (cite it instead). create-verification-skill merges into bee-verifying and maintain-verification-skill into bee-verify-upkeep. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell. Write docs/history/pstack-playbook-port/port-notes/merge-ppb-36.md listing what came in and what was left out because bee already has it.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pstack rule the bee skill lacked is merged in or named in the port notes with the bee home that already holds it"
   ],
   "prohibitions": [
    "No bee rule is removed",
    "The skill description stays under 300 characters"
   ]
  }
 },
 {
  "id": "ppb-36c",
  "feature": "pstack-playbook-port",
  "title": "Check the bee-verifying merge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-36"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-36c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/create-verification-skill/SKILL.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/maintain-verification-skill/SKILL.md",
   "skills/bee-verifying/SKILL.md",
   "skills/bee-verify-upkeep/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-36 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-36c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "ppb-37",
  "feature": "pstack-playbook-port",
  "title": "Restore principle pointers and close the slice 3 leftovers",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ppb-29",
   "ppb-30"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "6ca3cd62-c65d-499b-9351-7128c63c80b9",
   "b25486e3-747d-4827-bb77-f9d11ef2cb42",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "skills/bee-planning/playbooks/",
   "skills/bee-architect/",
   "skills/bee-arena/",
   "skills/bee-interrogate/",
   "skills/bee-show-me-your-work/",
   "skills/bee-fanout/",
   "skills/bee-tdd/",
   "skills/bee-reflect/",
   "skills/bee-recall/",
   "skills/bee-blast-radius/",
   "skills/bee-comment-review/",
   "skills/bee-figure-it-out/",
   ".claude-plugin/skills/bee-planning/",
   ".codex-plugin/skills/bee-planning/",
   ".claude/skills/bee-planning/",
   ".agents/skills/bee-planning/",
   ".opencode/skills/bee-planning/",
   ".claude-plugin/skills/bee-architect/",
   ".claude-plugin/skills/bee-arena/",
   ".claude-plugin/skills/bee-interrogate/",
   ".claude-plugin/skills/bee-show-me-your-work/",
   ".claude-plugin/skills/bee-fanout/",
   ".claude-plugin/skills/bee-tdd/",
   ".claude-plugin/skills/bee-reflect/",
   ".claude-plugin/skills/bee-recall/",
   ".claude-plugin/skills/bee-blast-radius/",
   ".claude-plugin/skills/bee-comment-review/",
   ".claude-plugin/skills/bee-figure-it-out/",
   ".codex-plugin/skills/bee-architect/",
   ".codex-plugin/skills/bee-arena/",
   ".codex-plugin/skills/bee-interrogate/",
   ".codex-plugin/skills/bee-show-me-your-work/",
   ".codex-plugin/skills/bee-fanout/",
   ".codex-plugin/skills/bee-tdd/",
   ".codex-plugin/skills/bee-reflect/",
   ".codex-plugin/skills/bee-recall/",
   ".codex-plugin/skills/bee-blast-radius/",
   ".codex-plugin/skills/bee-comment-review/",
   ".codex-plugin/skills/bee-figure-it-out/",
   ".claude/skills/bee-architect/",
   ".claude/skills/bee-arena/",
   ".claude/skills/bee-interrogate/",
   ".claude/skills/bee-show-me-your-work/",
   ".claude/skills/bee-fanout/",
   ".claude/skills/bee-tdd/",
   ".claude/skills/bee-reflect/",
   ".claude/skills/bee-recall/",
   ".claude/skills/bee-blast-radius/",
   ".claude/skills/bee-comment-review/",
   ".claude/skills/bee-figure-it-out/",
   ".agents/skills/bee-architect/",
   ".agents/skills/bee-arena/",
   ".agents/skills/bee-interrogate/",
   ".agents/skills/bee-show-me-your-work/",
   ".agents/skills/bee-fanout/",
   ".agents/skills/bee-tdd/",
   ".agents/skills/bee-reflect/",
   ".agents/skills/bee-recall/",
   ".agents/skills/bee-blast-radius/",
   ".agents/skills/bee-comment-review/",
   ".agents/skills/bee-figure-it-out/",
   ".opencode/skills/bee-architect/",
   ".opencode/skills/bee-arena/",
   ".opencode/skills/bee-interrogate/",
   ".opencode/skills/bee-show-me-your-work/",
   ".opencode/skills/bee-fanout/",
   ".opencode/skills/bee-tdd/",
   ".opencode/skills/bee-reflect/",
   ".opencode/skills/bee-recall/",
   ".opencode/skills/bee-blast-radius/",
   ".opencode/skills/bee-comment-review/",
   ".opencode/skills/bee-figure-it-out/",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "docs/history/pstack-playbook-port/port-notes/",
   "docs/history/pstack-playbook-port/port-notes/principle-pointers.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/reports/ppb-24c.md",
   "docs/history/pstack-playbook-port/reports/ppb-25c.md",
   "docs/history/pstack-playbook-port/port-notes/"
  ],
  "affects_skills": [
   "skills/bee-planning/playbooks/",
   "skills/bee-architect/SKILL.md",
   "skills/bee-arena/SKILL.md",
   "skills/bee-interrogate/SKILL.md",
   "skills/bee-show-me-your-work/SKILL.md",
   "skills/bee-fanout/SKILL.md",
   "skills/bee-tdd/SKILL.md",
   "skills/bee-reflect/SKILL.md",
   "skills/bee-recall/SKILL.md",
   "skills/bee-blast-radius/SKILL.md",
   "skills/bee-comment-review/SKILL.md",
   "skills/bee-figure-it-out/SKILL.md"
  ],
  "affects_specs": [],
  "action": "Every port note that dropped a pstack principle pointer because its bee principle did not exist yet: now that slices 4's principles exist, put the pointer back as the bee slug (bee-principle-<slug>) in the playbook or skill it came from, and mark it restored in the note. Then close the leftovers in reports ppb-24c and ppb-25c: worktree-cleanup names bee-arena; multi-phase-plan owners never rebase (decision b25486e3) and it names the read role for gathers; note or restore the feature port's rebase wording and the 'the app is small' rebuttal; move the misplaced restored note in session-ppb-12.md; note the table-only rows ppb-11c marked not noted. Write docs/history/pstack-playbook-port/port-notes/principle-pointers.md listing each pointer restored. No line names Cursor, Graphite, bugbot or poteto-agent; no hand-picked model or subagent_type; tools become bee counterparts per D2. Copy the pstack LICENSE verbatim to each new skill directory as LICENSE.pstack. Run bee dev regen and commit the rendered trees and the manifest with the cell.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test skill_triggers && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "every pointer a port note dropped for a missing principle is restored as a bee-principle slug",
    "no playbook tells an owner or worker to rebase",
    "session/worktree-cleanup.md names bee-arena"
   ],
   "prohibitions": [
    "No situation or skill is added or removed"
   ]
  }
 },
 {
  "id": "ppb-37c",
  "feature": "pstack-playbook-port",
  "title": "Check the restored principle pointers and leftovers",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ppb-37"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "3ac421ec-168b-4007-9976-9b6bcbe6e18a",
   "913115b3-e1f1-4004-902c-bc574fabb2f1"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/ppb-37c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "docs/history/pstack-playbook-port/port-notes/principle-pointers.md",
   "docs/history/pstack-playbook-port/reports/ppb-24c.md",
   "docs/history/pstack-playbook-port/reports/ppb-25c.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read the pstack sources beside what ppb-37 wrote. Report each source rule as present, left out with a named bee home, or missing; any pstack-only tool name; any bee rule removed. Write docs/history/pstack-playbook-port/reports/ppb-37c.md; no other edits.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test principle_index_parity",
  "must_haves": {
   "truths": [
    "the report lists every source rule with present, left out or missing"
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
| ppb-26 to ppb-29 | each new principle. Pass when `principle_index_parity` is green with its skill, row and guide anchor | a classes line. Pass when every value is one of the nine route classes | a row with no spoken line. Pass when the fence is red naming it |
| ppb-30 to ppb-36 | each merged skill or principle. Pass when every pstack rule is merged or named with its bee home | a rule bee homes elsewhere. Pass when it is cited, not copied | a bee rule removed. Pass when the checker finds none |
| ppb-37 | each dropped principle pointer. Pass when restored as a bee slug | worktree-cleanup and multi-phase-plan. Pass when they name bee-arena and no owner rebases | a leftover from ppb-24c or ppb-25c still open. Pass when ppb-37c names none |

The run on main and on head for changed behavior: `bee orient --class refactor` on main prints the refactor principles bee had; on head it also names the new principles whose classes line includes refactor.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
