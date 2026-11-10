# Plan: Knowledge that agents can use

## Summary

Today a worker gets one line on how to write knowledge, the pack shows only a
title, nothing warns on a directory anchor or a 1000-line concept, and nobody
knows which concepts ever helped. After this, the worker brief prints five
writing rules from the one list `bee knowledge consolidate` prints, the pack
prints a concept's `Read when:` line, `bee knowledge check` warns on a missing
Read-when line, a directory anchor and a concept over 300 lines, each cap can
record which concepts changed a decision, and `bee knowledge report` lists use
counts and concepts loaded often but never used. Nothing new refuses.

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain.
Why this is the least workflow that protects the work: it changes the worker
brief every host renders, a cap flag, check output and report output, and one
existing test pins the rule count at three.

Class playbook: feature — `skills/bee-planning/references/planning-reference.md` ("Playbooks").

## Requirements (from CONTEXT.md)

- D1, D3, D6: one rule list, five rules, rendered into the worker brief → kur-1.
- D2: the pack prints the Read-when line → kur-1; check warns when it is missing → kur-2.
- D4: check warns on directory anchors and long concepts → kur-2.
- D5: cap records knowledge used → kur-3; report lists uses and never-used → kur-4.
- Skills and the owning concept teach the rules → kur-5. Consolidation → kur-6.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The rule list exists once, with three rules | read | packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs:9 | `pub(crate) const THIN_LAYER_RULES: [&str; 3] = [` |
| 2 | A test pins the rule count, so the change is a covered contract | read | packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs:165 | `assert_eq!(value["rules"].as_array().unwrap().len(), 3);` |
| 3 | The worker brief carries one knowledge line today | read | packages/bee/prompts/worker-cell.md:69 | `- Write your knowledge delta (owning area concept update, a pitfall pattern when you hit one, bee.anchors for files you changed) in the same commit as your code, thin and anchored, no review step.` |
| 4 | The worker prompt has a fixed variable list to extend | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:101 | `"worker-cell" => Some(&[` |
| 5 | A learned-context line is path plus title | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1027 | `"- {} — {}",` |
| 6 | The close freshness door ignores warning codes it does not name, so new warnings never block | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:814 | `"not_canonical" \| "invalid_evidence_state" => report_only_count += 1,` |
| 7 | Check already emits profile warnings through one finding shape | read | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:674 | `"missing_profile_field",` |
| 8 | The cap reads a mistakes answer from flags and the report; knowledge-used copies that shape | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1036 | `pub(crate) fn read_mistakes_answer(id: &str, f: &CapFlags, report: &Value) -> MR<MistakesAnswer> {` |
| 9 | A cap from a herding job builds its report in one place | read | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:1039 | `map.insert("deviations".to_string(), Value::Array(deviations));` |
| 10 | Cell prepare rows record the loaded paths, so never-used is derivable | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3442 | `record.insert("learned_context".into(), paths(&knowledge));` |
| 11 | The report already reads cell prepare rows | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:343 | `if row.get("source").and_then(Value::as_str) != Some("prepare") \|\| row.get("kind").and_then(Value::as_str) != Some("cell") {` |
| 12 | The herding result has a fixed field set to extend | read | packages/bee-rs/crates/bee/src/herding/mailbox.rs:546 | `pub proof: String,` |

## Discovery

Ran `bee knowledge pack --feature knowledge-useful-rules --stage planning`: on
the installed 2.88.0 binary it lists only critical patterns (the area step of
feature-pack-areas is not released yet). Measured on 2026-11-10: 2 of 183
patterns open with a read-when line; area concepts have a median of 151 lines
and 27 are over 300; about 40 anchors are directories.

## Approach

- One list (D6): rename `THIN_LAYER_RULES` to a five-rule list; consolidate
  prints it; the worker prompt gets a `knowledge_rules` variable rendered from it.
- The Read-when line rides the existing title slot of the one assembler (D2),
  so cells, the pack verb and advisors all get it with no new line shape.
- Check warnings are new codes on the existing finding shape (D4); the close
  door's default arm already ignores them.
- `knowledge_used` copies the mistakes answer shape but never refuses (D5,
  knowledge-in-the-flow D1).
- Hat wave skipped, named deviation: the procedure home
  (`bee-hive/references/gates-and-delegation.md` "Hat wave") opens it for big,
  vague or high-risk work; this ask is five locked rules with a code precedent
  for each.

Rejected: a mandatory knowledge-used answer (breaks 9dca3714); rewriting the
existing concepts here (content work with its own lifecycle, filed as backlog).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Worker prompt render | MEDIUM | kur-1 | a rendered worker prompt contains the five rule lines |
| Cap flag surface | MEDIUM | kur-3 | registry contract tests stay green |
| Check output volume | LOW | kur-2 | warnings only; exit status without --strict unchanged |

## Shape

One slice, four parallel code cells (disjoint files), then docs, then the
writer cell. kur-4 waits on kur-3 for the trace field name.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| kur-1 | Give workers the five knowledge rules and print Read-when lines in the pack | consolidate.rs, prepare.rs, prompt.rs, drivers/tests.rs, worker-cell.md (both copies) | — | a worker brief lists five knowledge rules; `bee knowledge pack` prints `Read when:` for a concept that has one | `cargo test … -p bee -- knowledge_rules read_when consolidat` |
| kur-2 | Warn on missing Read-when, directory anchors and long concepts | check.rs, knowledge/tests.rs | — | `bee knowledge check` names `missing_read_when`, `directory_anchor`, `long_concept` | `cargo test … -p bee -- writing_warnings` |
| kur-3 | Record on each cap which concepts changed a decision | handlers_close.rs, finish_support.rs, cells/tests.rs, herding/mailbox.rs, registry_payload.json, hooks/cli_shape.rs | — | `bee cells finish --knowledge-used "<path> — <what>"` lands on trace.knowledge_used | `cargo test … -p bee -- knowledge_used` |
| kur-4 | Report concept use and concepts loaded but never used | knowledge/report.rs | kur-3 | `bee knowledge report` prints a knowledge-used line, top uses, and never-used concepts | `cargo test … -p bee -- report_knowledge_used` |
| kur-5 | Teach the rules in the skills and the owning concept | bee-swarming SKILL.md, bee-capturing SKILL.md and concept-authoring.md, freshness-and-document-homes.md, rendered skill trees | kur-1, kur-3 | the skills point at the one rule list and name the knowledge-used answer | `bee dev regen --check` and `bee knowledge check` |
| kur-6 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md | all | the okf-profile and workflow-state concepts describe the rules, warnings and use record | `bee knowledge check` |

```json
[
  {
    "id": "kur-1",
    "feature": "knowledge-useful-rules",
    "title": "Give workers the five knowledge rules and print Read-when lines in the pack",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["71241c48", "90988b99", "f4b38727", "721d06f5"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/pack.rs",
      "packages/bee/prompts/worker-cell.md",
      ".bee/bin/prompts/worker-cell.md",
      "docs/knowledge/areas/okf-profile/context-and-promote.md"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee/prompts/worker-cell.md",
      "docs/knowledge/patterns/20260819-a-test-that-derives-its-fixture-from-the-constant-under-test-only-proves-the-code-agrees-with-itself.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contracts 90988b99 and f4b38727. (1) In knowledge/consolidate.rs replace THIN_LAYER_RULES with a five-rule list, in this order: 'Anchor every entry to the code files it describes in bee.anchors — files, never directories.'; 'Never restate what the code or a test shows: keep the why, the boundary, and the rejected option with how it failed; drop the how.'; 'Extend the concept that owns the subject; find it with bee knowledge search --text and bee knowledge list --area, and never file a near-duplicate beside it.'; 'Open the body with one line Read when: <the situation the reader is in>, and put the words a reader will search for — the error text, the symptom, the mechanism — in the title and description.'; 'Split a concept that passes 300 lines along its lifecycles.' Update the consolidate test that pins three rules to assert the five literal strings (never derive the expected value from the constant). (2) Add a knowledge_rules variable to the worker-cell supported vars in drivers/prompt.rs, fill it in prepare.rs from the same list (one line per rule, each prefixed '  - '), and render it in packages/bee/prompts/worker-cell.md directly under the knowledge-delta bullet. In the same prompt: name the knowledge_used key in the native Result form JSON as \"knowledge_used\": [\"<docs/knowledge path> — <the decision it changed>\"] with one sentence saying an empty array means no concept changed a decision; in the herding paragraph add that the result file may carry the same knowledge_used array. Copy the prompt byte-identical to .bee/bin/prompts/worker-cell.md. (3) In prepare.rs bundle_learned_lines, when a selected concept's body (knowledge::concept_body) has a line starting 'Read when:' among its first 10 non-blank lines that are not headings, render that line in place of the title through the same one_line 140 cap; otherwise keep the title exactly as today. Red first for (2) and (3) in drivers/tests.rs: a rendered worker prompt contains each of the five rule lines, and a learned-context line for a fixture concept with a Read-when line shows it while one without keeps its title. Knowledge delta in the same commit: extend B5 in docs/knowledge/areas/okf-profile/context-and-promote.md with the five rules' one home and the Read-when line in the pack, citing this feature's CONTEXT.md D2 and D6 (not raw decision ids). No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge_rules read_when consolidat",
    "must_haves": {
      "truths": [
        "bee knowledge consolidate --json returns a rules array of the five strings in the stated order",
        "A rendered worker-cell prompt contains the five rule lines under the knowledge-delta bullet",
        "A learned-context line for a concept with a Read when: line shows that line; one without shows its title as before",
        "The worker prompt's Result form names the knowledge_used key",
        "packages/bee/prompts/worker-cell.md and .bee/bin/prompts/worker-cell.md are byte-identical"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs", "substantive": "a five-rule list replacing THIN_LAYER_RULES and a test asserting the literal strings"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "a knowledge_rules block under the delta bullet and the knowledge_used result key"}
      ],
      "key_links": [
        "prepare.rs fills knowledge_rules from the consolidate list, so the brief and consolidate cannot disagree"
      ],
      "prohibitions": [
        "No new refusal",
        "No second copy of the rule text in Rust",
        "No code comments"
      ],
      "open_choices": [
        "Whether the Read-when lookup lives in prepare.rs or a small helper in knowledge/"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kur-2",
    "feature": "knowledge-useful-rules",
    "title": "Warn on missing Read-when, directory anchors and long concepts",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["71241c48", "e8899530"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract e8899530. In knowledge/check.rs check_bundle add three profile warnings through the existing finding shape, never errors: missing_read_when on a concept of type bee.area or bee.pattern whose first 10 non-blank lines that are not headings hold no line starting 'Read when:'; directory_anchor once per bee.anchors entry that ends with '/' or resolves to a directory under the repo root (repo_root is already derived in check_bundle); long_concept on a concept whose body exceeds 300 lines. Exit status without --strict stays unchanged. Red first in knowledge/tests.rs: a fixture bundle with one compliant concept and one breaking each rule; assert each code appears exactly for the breaking file, the compliant one gets none, and okf errors and profile errors stay empty. Confirm by reading build_knowledge_freshness_door in drivers/close.rs that the new codes fall to its default arm (no change there). Knowledge delta in the same commit: in docs/knowledge/areas/okf-profile/freshness-and-document-homes.md extend 'The templates rule' with the Read-when line and the three warnings, citing this feature's CONTEXT.md D2 and D4. No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- writing_warnings",
    "must_haves": {
      "truths": [
        "bee knowledge check reports missing_read_when, directory_anchor and long_concept as profile warnings on the breaking files only",
        "A compliant concept gets none of the three",
        "bee knowledge check without --strict exits as before",
        "The close knowledge-freshness door never blocks on the three codes"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs", "substantive": "three warning codes in check_bundle"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs", "substantive": "a writing_warnings test over a fixture bundle"}
      ],
      "key_links": ["drivers/close.rs build_knowledge_freshness_door default arm ignores the new codes"],
      "prohibitions": ["No new error or refusal", "No code comments"],
      "open_choices": ["How the directory test treats an anchor that does not exist (orphan_anchor already covers it)"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kur-3",
    "feature": "knowledge-useful-rules",
    "title": "Record on each cap which concepts changed a decision",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["71241c48", "721d06f5"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract 721d06f5. Add --knowledge-used (string, '<path> — <what it changed>') and --no-knowledge-used (bool) to every cap verb that takes --no-mistakes, in the registry and in CapFlags parsing. Read the answer the way read_mistakes_answer reads mistakes: the flag, then the report JSON's knowledge_used array; a present-but-empty array counts as the explicit none. Persist trace.knowledge_used as [{path, changed}] when any entry exists (split on the first ' — '; no separator keeps the path with an empty changed), else trace.no_knowledge_used true when the none was given, else neither key. Never refuse: not on silence, not on a contradiction (entries win), not on an odd path. Herding: add an optional knowledge_used string array to MailboxResult (absent when the key is absent) and to the result contract text in mailbox.rs, and carry it through report_from_job in finish_support.rs into the report's knowledge_used. --from-job must not require it. Red first in cells/tests.rs: cap with the flag, with the report array, with the explicit none, and silent; assert the four trace shapes; and a from-job cap whose result carries knowledge_used. Keep the registry contract tests green. Knowledge delta in the same commit: extend docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md beside the mistakes answer, citing this feature's CONTEXT.md D5 and knowledge-in-the-flow D1. No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge_used registry",
    "must_haves": {
      "truths": [
        "bee cells finish --knowledge-used \"docs/knowledge/x.md — chose y\" records trace.knowledge_used [{path, changed}]",
        "--no-knowledge-used records trace.no_knowledge_used true; silence records neither key",
        "No input shape refuses a cap",
        "A herding result's knowledge_used array reaches the trace through --from-job"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "a knowledge-used answer reader and the trace write"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "the two flags on each cap verb"}
      ],
      "key_links": ["finish_support.rs report_from_job passes knowledge_used from MailboxResult"],
      "prohibitions": ["No refusal on any knowledge-used input", "No change to the mistakes answer", "No code comments"],
      "open_choices": ["Whether --knowledge-used is repeatable"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kur-4",
    "feature": "knowledge-useful-rules",
    "title": "Report concept use and concepts loaded but never used",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["kur-3"],
    "decisions": ["71241c48", "e759fbaa"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs",
      "docs/knowledge/areas/okf-profile/critical-bar.md"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract e759fbaa. In knowledge/report.rs add a knowledge-used section built from closed_cells traces (knowledge_used, no_knowledge_used) and the learned_context lists of dispatch.jsonl prepare rows of kind cell. Text: 'knowledge used: <a> cell(s) answered (<u> used a concept, <n> none), <s> silent'; up to 10 lines '  <uses>  <path>' sorted by uses desc then path; one line 'loaded, never used: <path> — loaded <n> time(s)' per docs/knowledge path loaded in 5 or more cell prepare rows with zero uses, sorted by loads desc then path. JSON: knowledge_used {answered, used, none, silent, concepts:[{path, uses}], never_used:[{path, loads}]}. Place it after the feature outcomes and before the pull rate. Red first in report.rs mod tests with a temp root holding cells and a dispatch.jsonl. Knowledge delta in the same commit: extend B6 in docs/knowledge/areas/okf-profile/critical-bar.md with the use measure, citing this feature's CONTEXT.md D5. No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- report_knowledge_used",
    "must_haves": {
      "truths": [
        "bee knowledge report prints the knowledge-used line with answered, used, none and silent counts",
        "A concept loaded 5 or more times with no recorded use prints a 'loaded, never used' line",
        "The JSON output carries the knowledge_used object"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs", "substantive": "a build_knowledge_used function and its tests"}],
      "key_links": ["reads trace.knowledge_used written by kur-3"],
      "prohibitions": ["No change to the arm outcome lines", "No code comments"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kur-5",
    "feature": "knowledge-useful-rules",
    "title": "Teach the rules in the skills and point at the one rule list",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": ["kur-1", "kur-3"],
    "decisions": ["71241c48", "90988b99"],
    "files": [
      "skills/bee-swarming/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      "skills/bee-capturing/references/concept-authoring.md",
      ".claude/skills/**",
      ".agents/skills/**",
      ".opencode/skills/**"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-capturing/references/concept-authoring.md"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md", "skills/bee-capturing/SKILL.md"],
    "affects_specs": [],
    "action": "Point the skills at the one rule list, never copy its text. bee-swarming Execute step 4: say the worker brief prints the knowledge rules (the same list bee knowledge consolidate prints) and that the cap answers which concepts changed a decision with --knowledge-used '<path> — <what it changed>' or --no-knowledge-used, never refused. bee-capturing Scribe and references/concept-authoring.md step 3: add the Read-when opening line and the searched words in title and description, name the three check warnings (missing_read_when, directory_anchor, long_concept), and change step 5 to say check --strict also fails on existing bundle warnings, so the bar is no NEW finding on the files you wrote. Then regenerate the skill trees with .bee/bin/bee dev regen and prove with .bee/bin/bee dev regen --check. One commit.",
    "verify": ".bee/bin/bee dev regen --check",
    "must_haves": {
      "truths": [
        "bee-swarming step 4 names the brief's rule list and the knowledge-used answer",
        "concept-authoring.md names the Read-when line and the three warnings",
        "The rendered skill trees match the sources"
      ],
      "artifacts": [{"path": "skills/bee-capturing/references/concept-authoring.md", "substantive": "Read-when, searched words, the three warnings"}],
      "key_links": ["the skills cite the consolidate rule list instead of copying it"],
      "prohibitions": ["No copy of the five rule sentences in a skill"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "kur-6",
    "feature": "knowledge-useful-rules",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": ["kur-1", "kur-2", "kur-3", "kur-4", "kur-5"],
    "decisions": ["71241c48"],
    "files": [
      "docs/knowledge/**",
      "docs/history/knowledge-useful-rules/delivery.md"
    ],
    "read_first": [
      "docs/history/knowledge-useful-rules/CONTEXT.md",
      "docs/history/knowledge-useful-rules/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature knowledge-useful-rules`. Dedupe and trim the deltas the cells wrote, anchor each concept to the code it describes, never restate code, and write docs/history/knowledge-useful-rules/delivery.md listing the concepts changed and why, one line each. Apply this feature's own rules to what it wrote: a Read when: line on each concept it touched, no how that code or a test already shows. Cite this feature's CONTEXT.md D-numbers, never raw decision ids. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "area and pattern concepts describe the feature's changes, anchored to code",
        "delivery.md lists each concept changed with one line on why"
      ],
      "artifacts": [{"path": "docs/history/knowledge-useful-rules/delivery.md", "substantive": "one line per concept changed"}],
      "key_links": [],
      "prohibitions": ["No raw decision ids in docs outside docs/history/knowledge-useful-rules/"],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

| Behavior | Test |
|---|---|
| five rules, literal | consolidate.rs tests |
| brief renders rules, pack shows Read-when | drivers/tests.rs |
| three warnings | knowledge/tests.rs writing_warnings |
| four trace shapes, from-job | cells/tests.rs knowledge_used |
| report section | report.rs report_knowledge_used |

## Open Questions

- (none)

## Out of scope

- Rewriting existing concepts to the rules: a backlog item.
- A release: the owner asks for it.
