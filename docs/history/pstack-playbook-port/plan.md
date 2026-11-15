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
| 2 | orient matches its argv by exact slice patterns, so a new flag needs new patterns | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:866` | `["orient", "--class", class] => return run_class(class, false, t0),` |
| 3 | The route record is built field by field; this plan leaves it unchanged | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:647` | `route.insert("class".into(), json!(class_value.unwrap()));` |
| 4 | The route verb has a closed flag list; this plan adds no flag | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:844` | `&["set", "show", "class", "lane", "lane-record", "flags", "files", "rationale", "no-lane"],` |
| 5 | Feature start reads route flags itself; this plan leaves it unchanged | read | `packages/bee-rs/crates/bee/src/verbs/feature_group.rs:68` | `if let Some(v) = flags.get("class") {` |
| 6 | The plan check matches one `<name>.md` directly under the playbooks directory | read | `packages/bee-rs/crates/bee/src/verbs/plan_group.rs:82` | `let cited = \|name: &str\| text.contains(&format!("{PLAYBOOKS}{name}.md"));` |
| 7 | The parity fence reads only `.md` entries of the playbooks directory, so subdirectories leave its two tests intact | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:102` | `entry.file_name().to_str().and_then(\|n\| n.strip_suffix(".md")).map(str::to_string)` |
| 8 | There are nine route classes | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:356` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 9] =` |
| 9 | The release manifest walks the whole skills tree, so new playbook and skill files vendor with no manifest code change | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:387` | `records.extend(enumerate_tree(root, &root.join("skills"), "plugin_skill", &[])?);` |
| 10 | Each new `bee-` skill needs a description under 300 characters | read | `packages/bee-rs/crates/bee/tests/skill_triggers.rs:15` | `const DESCRIPTION_BUDGET: usize = 300;` |
| 11 | Each new `bee-` skill needs at least three trigger briefs in the fixture | read | `packages/bee-rs/crates/bee/tests/skill_triggers.rs:11` | `const MIN_EXPECT: usize = 3;` |
| 12 | Every class playbook ends on the class proof floor line a merge must keep | read | `skills/bee-planning/playbooks/bugfix.md:31` | `Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).` |
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

Waves: slice 1 runs two chains side by side. The Rust chain pp-1, pp-1t,
pp-2, pp-2t is serial because every cell compiles the same crate in one
worktree. pp-3 and pp-3c touch Markdown and the rendered trees only and run
beside it. pp-4 waits on both chains because the fence reads the bugfix files
and the crate.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pp-1 and pp-2 write Rust."},
    {"stage":"testing","classification":"required","role":"test","reason":"Every tester and checker cell runs on the test role, apart from its implementer."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"pp-3 and the later port slices write playbooks, skills and principles."},
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
| orient output changes for scripts | MEDIUM | pp-1 | the playbook line stays byte-identical; `situations` is always present (pp-1t) |
| Host repo has no `skills/` tree | MEDIUM | pp-1 | pp-1t runs orient in a repo with no `skills/` and gets the compiled-in situations (claims 14, 15) |
| Plans in flight that cite only a class index | MEDIUM | pp-2 | the finding fires only for a class that lists situations; slice 1 lists one (bugfix); the plan check is advisory, it never refuses a gate |
| The merged bug-fix file drops an enforced bee line | MEDIUM | pp-3 | pp-3c lists every old line as present (claim 12) |
| A vendored file names a tool hosts lack | MEDIUM | pp-3 and the port slices | the checker's `rg` over `skills/bee-planning/` finds none; port notes live under `docs/history/` |
| A class index names a skill not yet shipped | LOW | pp-3, slice 3 | slice 1 indexes name no skill; slice 3 adds the bee-figure-it-out fallback line with the skill |
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

All `cargo test` proofs run with `--manifest-path packages/bee-rs/Cargo.toml -p bee`.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pp-1 | List a class's situations in orient | `situations.rs` (new), `main.rs`, `orient.rs`, `status_full/tests.rs` | — | `bee orient --class bugfix` adds one line per bugfix situation; the playbook line is unchanged | `cargo test status_full — green:unit — touched orient.rs` |
| pp-1t | Test the situation reader and orient listing | `status_full/tests.rs`, `situations.rs` | pp-1 | tests pin the parser, the fallback and the JSON key | `cargo test situations status_full — green:unit — tester for pp-1` |
| pp-2 | Hold a plan to a situation playbook | `plan_group.rs` | pp-1t | `bee plan check` names the situation files a bugfix plan forgot to cite | `cargo test plan_group — green:unit — touched plan_group.rs` |
| pp-2t | Test the situation citation rule | `plan_group.rs` | pp-2 | tests pin cited, index-only, no-situation class and no-class cases | `cargo test plan_group — green:unit — tester for pp-2` |
| pp-3 | Port the pstack bug-fix playbook as the first situation | `skills/bee-planning/playbooks/bugfix.md`, `bugfix/bug-fix.md` (new), `LICENSE.pstack` (new), `port-notes.md` (new), the rendered trees | — | the bugfix class lists its situation with trigger phrases; the situation holds the pstack method and bee's enforced lines | `diff -r of the rendered tree — green:static — docs port` |
| pp-3c | Check the bug-fix port against its sources | its report only | pp-3 | a report of every source step present or missing | `rg pstack-only tools — green:static — checker for pp-3` |
| pp-4 | Fence the situation files against their index | `class_playbook_parity.rs` | pp-2t, pp-3c | the suite goes red when a situation file and its index disagree | `cargo test --test class_playbook_parity — green:unit — fence` |
| pp-4t | Check the situation fence goes red for each break | its report only | pp-4 | each break shown red, the clean tree green | `cargo test --test class_playbook_parity — green:unit — checker for pp-4` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "pp-1",
  "feature": "pstack-playbook-port",
  "title": "List a class's situations in orient",
  "lane": "standard",
  "role": "code",
  "deps": [],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410",
   "c64c5827-e110-4add-988c-5b668e93ccfe"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/situations.rs",
   "packages/bee-rs/crates/bee/src/main.rs",
   "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/src/principles.rs",
   "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Add situations.rs, the one reader of a class index's situations, shaped like principles.rs: read skills/bee-planning/playbooks/<class>.md from the repo root when it exists, otherwise a copy compiled in with include_str! for each of the nine class indexes; parse the '## Situations' section into (name, triggers, deliverable) rows; an absent section or a malformed line yields no row for that line, never an error. The class index grammar: a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, where <name> matches [a-z0-9-]+ and names playbooks/<class>/<name>.md; the section ends at the next '## ' heading. Register the module in main.rs. In orient.rs class_packet add a situations array (always present, empty for a class with none) and render_class_text one line per situation after the playbook line: `situation: skills/bee-planning/playbooks/<class>/<name>.md — <triggers>`. The existing playbook line stays byte-identical. Existing tests your change flips are yours; new tests belong to pp-1t.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full",
  "must_haves": {
   "truths": [
    "orient --class bugfix prints one situation line per row of the bugfix index's Situations section",
    "orient --class perf --json carries situations: [] while perf has no Situations section",
    "a repo with no skills/ tree gets the situations of the compiled-in index"
   ],
   "prohibitions": [
    "No new orient flag",
    "No change to the route record",
    "orient --class stays write-free"
   ]
  }
 },
 {
  "id": "pp-1t",
  "feature": "pstack-playbook-port",
  "title": "Test the situation reader and orient listing",
  "lane": "standard",
  "role": "test",
  "deps": [
   "pp-1"
  ],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs",
   "packages/bee-rs/crates/bee/src/situations.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/src/situations.rs",
   "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Cite the existing orient --class tests by name, then add only the gap: the parser on a well-formed section, a malformed line skipped, an absent section, the repo copy winning over the compiled-in copy, the compiled-in fallback with no skills/ tree, and the JSON situations key present and empty for a class with none. Parser unit tests go in situations.rs's own test module.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee situations && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full",
  "must_haves": {
   "truths": [
    "each truth of pp-1 has a test that fails if the behavior is removed"
   ],
   "prohibitions": [
    "No edit to non-test code"
   ]
  }
 },
 {
  "id": "pp-2",
  "feature": "pstack-playbook-port",
  "title": "Hold a plan to a situation playbook",
  "lane": "standard",
  "role": "code",
  "deps": [
   "pp-1t"
  ],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/plan_group.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/src/verbs/plan_group.rs",
   "packages/bee-rs/crates/bee/src/situations.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "In plan_group.rs playbook_findings, read the routed class's situations through situations.rs. When the class lists one or more situations: a plan citing skills/bee-planning/playbooks/<class>/<name>.md for a listed name passes; a plan citing only the class index is a finding whose detail names each listed situation path. When the class lists none, today's class-index rule holds unchanged. With no class known, a citation of playbooks/<class>/<name>.md counts as a playbook citation as well as playbooks/<name>.md. The inline tests your change flips are yours; new tests belong to pp-2t.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_group",
  "must_haves": {
   "truths": [
    "a bugfix route whose plan cites only bugfix.md yields one playbook finding naming bugfix/bug-fix.md",
    "a bugfix route whose plan cites bugfix/bug-fix.md has no playbook finding",
    "a perf route citing perf.md still passes while perf lists no situation",
    "a plan with no route class citing only bugfix/bug-fix.md has no playbook finding"
   ],
   "prohibitions": [
    "No other plan-check finding changes"
   ]
  }
 },
 {
  "id": "pp-2t",
  "feature": "pstack-playbook-port",
  "title": "Test the situation citation rule",
  "lane": "standard",
  "role": "test",
  "deps": [
   "pp-2"
  ],
  "decisions": [
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/plan_group.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/src/verbs/plan_group.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Cite plan_group_missing_playbook_citation_is_a_playbook_finding and its neighbours, then add only the gap in the inline test module: situation cited passes, index-only cited fails naming the situation paths, a class with no situations keeps the index rule, a no-class plan citing only a situation path passes, a name not listed on the index does not count.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_group",
  "must_haves": {
   "truths": [
    "each truth of pp-2 has a test that fails if the behavior is removed"
   ],
   "prohibitions": [
    "No edit outside the test module"
   ]
  }
 },
 {
  "id": "pp-3",
  "feature": "pstack-playbook-port",
  "title": "Port the pstack bug-fix playbook as the first situation",
  "lane": "standard",
  "role": "docs",
  "deps": [],
  "decisions": [
   "d480d467-1a0d-4feb-bbe6-3f181aa86f4b",
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188",
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410"
  ],
  "files": [
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "skills/bee-planning/playbooks/LICENSE.pstack",
   "docs/history/pstack-playbook-port/port-notes.md",
   ".claude-plugin/skills/bee-planning/",
   ".codex-plugin/skills/bee-planning/"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/bug-fix.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/LICENSE"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Write playbooks/bugfix/bug-fix.md: the pstack bug-fix method is the body (scientific stance, reproduce on the real surface, binary-search hypotheses with runtime evidence, revert what a refuted hypothesis motivated, confirm the mechanism before the fix, verify on the same surface, the failing repro lands before the fix), with every pstack tool swapped for its bee counterpart per D2 (how and why become bee-how and bee-why, subagent delegation becomes bee dispatch prepare, the control skill becomes the project's verify-app or the real interface) and every enforced line of the current bugfix.md kept per D3 (the red-before-green home pointer, the crash-site principle, the event-order statechart split, the formal verify rerun, the always-on signal sentence, Proof line, Proof floor). Rewrite playbooks/bugfix.md as the class index: one intro line, then The class index grammar: a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, where <name> matches [a-z0-9-]+ and names playbooks/<class>/<name>.md; the section ends at the next '## ' heading. List bug-fix as the one situation, then keep the Proof floor line. Name no skill that does not exist yet. Write docs/history/pstack-playbook-port/port-notes.md with one section per ported playbook listing each tool swap and each dropped step; that file is the only place a pstack-only tool is named. Copy the pstack LICENSE text verbatim to playbooks/LICENSE.pstack. Run bee dev regen so the rendered plugin trees match.",
  "verify": "rg -n 'Proof floor' skills/bee-planning/playbooks/bugfix.md skills/bee-planning/playbooks/bugfix/bug-fix.md && rg -c 'Lauren Tan' skills/bee-planning/playbooks/LICENSE.pstack && diff -r skills/bee-planning/playbooks .claude-plugin/skills/bee-planning/playbooks",
  "must_haves": {
   "truths": [
    "bugfix/bug-fix.md carries every step of pstack bug-fix and every enforced line of the old bugfix.md",
    "no file under skills/ names Cursor, Graphite, bugbot or poteto-agent",
    "the bugfix index Situations section parses under the grammar"
   ],
   "prohibitions": [
    "No other class playbook is edited"
   ]
  }
 },
 {
  "id": "pp-3c",
  "feature": "pstack-playbook-port",
  "title": "Check the bug-fix port against its sources",
  "lane": "standard",
  "role": "test",
  "deps": [
   "pp-3"
  ],
  "decisions": [
   "063a21e4-bccc-46a4-8fe8-1f1f99874c72",
   "5a1fc656-5549-462a-be0c-6bce5d919188"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/pp-3c.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "skills/bee-planning/playbooks/bugfix/bug-fix.md",
   "skills/bee-planning/playbooks/bugfix.md",
   "/home/thanhsmind/Projects/refs/ai-skills/cursor-plugins/pstack/skills/poteto-mode/playbooks/bug-fix.md",
   "docs/history/pstack-playbook-port/port-notes.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Read pstack bug-fix.md and the old bugfix.md (git show of the commit before pp-3) beside the new files. Report each pstack step and each old enforced line as present or missing, any pstack-only tool name under skills/, whether port-notes.md names every swap and drop, and whether the index lines match the grammar in the plan. Write the report to docs/history/pstack-playbook-port/reports/pp-3c.md; no other edits.",
  "verify": "rg -n -i 'cursor|graphite|bugbot|poteto-agent' skills/bee-planning/ ; test $? -eq 1",
  "must_haves": {
   "truths": [
    "the report lists every source step with present or missing"
   ],
   "prohibitions": [
    "No edit outside the report file"
   ]
  }
 },
 {
  "id": "pp-4",
  "feature": "pstack-playbook-port",
  "title": "Fence the situation files against their index",
  "lane": "standard",
  "role": "test",
  "deps": [
   "pp-2t",
   "pp-3c"
  ],
  "decisions": [
   "c64c5827-e110-4add-988c-5b668e93ccfe",
   "41bbba68-9de8-48b3-a9c7-ba1abfdd6410"
  ],
  "files": [
   "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs",
   "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Extend class_playbook_parity.rs without touching its two existing tests: every subdirectory of playbooks/ is a route class or session; every .md in a class subdirectory is listed on its class index's Situations section, and every listed name has its file; every .md in session/ other than index.md is listed in session/index.md and the reverse, when session/ exists; the class list stays read as text from workflows.rs. The class index grammar: a '## Situations' heading, then one line per situation of the exact form - `<name>` — <trigger phrases, English and Vietnamese> — delivers: <deliverable>, where <name> matches [a-z0-9-]+ and names playbooks/<class>/<name>.md; the section ends at the next '## ' heading.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "an unlisted situation file fails the fence",
    "a listed name with no file fails the fence",
    "a subdirectory that is neither a class nor session fails the fence"
   ],
   "prohibitions": [
    "No private copy of the class list"
   ]
  }
 },
 {
  "id": "pp-4t",
  "feature": "pstack-playbook-port",
  "title": "Check the situation fence goes red for each break",
  "lane": "standard",
  "role": "test",
  "deps": [
   "pp-4"
  ],
  "decisions": [
   "c64c5827-e110-4add-988c-5b668e93ccfe"
  ],
  "files": [
   "docs/history/pstack-playbook-port/reports/pp-4t.md"
  ],
  "read_first": [
   "docs/history/pstack-playbook-port/CONTEXT.md",
   "docs/history/pstack-playbook-port/plan.md",
   "packages/bee-rs/crates/bee/tests/class_playbook_parity.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Break the tree three ways with temporary edits you revert before reporting (an unlisted situation file, a listed missing file, a stray subdirectory), record the fence output for each, then confirm the clean tree is green. Write the report to docs/history/pstack-playbook-port/reports/pp-4t.md; every other file ends byte-identical.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test class_playbook_parity",
  "must_haves": {
   "truths": [
    "each break is shown red with its message"
   ],
   "prohibitions": [
    "Every file but the report ends byte-identical"
   ]
  }
 }
]
```

## Test matrix

| Cell | Happy path | Edge case | Error path |
|---|---|---|---|
| pp-1 | `orient --class bugfix`. Pass when the output has one `situation:` line naming `bugfix/bug-fix.md` | `orient --class perf --json`. Pass when `situations` is `[]` | repo with no `skills/` tree. Pass when the compiled-in situations print |
| pp-1 | parser on a well-formed section. Pass when one row per line | malformed line. Pass when that line yields no row and the rest parse | absent section. Pass when the list is empty, no error |
| pp-2 | bugfix plan citing `bugfix/bug-fix.md`. Pass when no playbook finding | no-class plan citing only a situation path. Pass when no playbook finding | bugfix plan citing only `bugfix.md`. Pass when one finding names `bugfix/bug-fix.md` |
| pp-3 | bug-fix.md beside its sources. Pass when pp-3c lists every step present | English and Vietnamese triggers on the index line. Pass when both appear | a pstack-only tool under `skills/bee-planning/`. Pass when `rg` finds none |
| pp-4 | clean tree. Pass when the fence is green | `session/` absent. Pass when accepted | unlisted file, listed missing file, stray directory. Pass when each is red with its message |

The run on main and on head for changed behavior: `bee orient --class bugfix`
on main prints the playbook line and principles; on head the same lines plus
one `situation:` line.

## Open Questions

(none)

## Out of scope

- pstack skills the playbooks do not cite: setup-pstack, typescript-best-practices, make-bot-ui, automate-me, bro (CONTEXT Deferred Ideas).
- Per-situation principle filtering; principles stay per class (`41bbba68`).
- A situation field on the route record (`41bbba68`).
- Publishing the new skills to the skillstation hub; that rides the next release.
