---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Outcome scorecard

## Summary

`bee report outcomes --by-feature` shows, for each feature, how often work
was redone, how often a judge sent it back, how long it took, how often a
plan gate was reopened, how many fix-first cells it needed afterwards, and
what the workers cost. Workers run through herding keep their stance on the
cell, so their features stop reading "not recorded". Every gate change now
leaves an audit row.

Mode: `standard` — 2 risk flags: public contracts (report output, a new
audit kind, a new optional cell field), changes behavior an existing test
asserts (audit kinds list, from-job cap).
Why this is the least workflow that protects the work: three disjoint code
changes plus a doc, each with its own tester; nothing deletes or migrates data.

Situation followed: `skills/bee-planning/playbooks/feature/feature.md`
("feature"). Architect skipped: each metric is a pure function over records
the store already holds; no competing structure. Statechart: n/a.

## Requirements (from CONTEXT.md)

- D1: `--from-job` carries reaction, skipped, unknowns, found, stopped.
- D2: every gate set appends an audit row of kind `gate`.
- D3: `bee report outcomes --by-feature [--since]` per-feature scorecard,
  read-only, `not recorded` never 0, default output unchanged.
- D4: optional cell field `fixes_feature`, documented for fix-first cells.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring of the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The herded cap builds its report in one function | read | packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:1089 | `map.insert("outcome".to_string(), Value::String(outcome.to_string()));` |
| 2 | The native cap already writes reaction to the trace | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:810 | `if let Some(reaction) = report_value.get("reaction") {` |
| 3 | Audit kinds are a fixed list that D2 extends | read | packages/bee-rs/crates/bee/src/audit.rs:6 | `pub(crate) const KINDS: [&str; 18] = [` |
| 4 | Only auto gate sets write an audit row today | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:1071 | `crate::audit::append(` |
| 5 | Rework is read from the trace flags the scorecard reuses | read | packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs:482 | `let reworked = ["reopened_for_rework", "reopened_at"]` |
| 6 | Each herding job records its usage, the cost source | read | packages/bee-rs/crates/bee/src/herding/run.rs:3227 | `fn record_round_usage(` |
| 7 | A lane records when it started, the cycle-time start | read | packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs:1055 | `record.insert("created_at".into(), Value::String(now_iso()));` |
| 8 | Today's herded features show no reaction | ran | `.bee/bin/bee report outcomes --feature brief-intent-and-wip` | `worker reaction: not recorded` |

## Discovery

Gather job-1794888871067-620864-1 mapped report_outcomes.rs (layers and
fixture), the from-job cap (drops reaction, skipped, unknowns, found,
stopped), timestamps (lane created_at, usage closed_at, capped_at; no merge
record), gate history (only auto sets audited), job usage (job.json `usage`
with feature), and fix-first detection (no link field; hence D4).

## Approach

Recommended path: D1 copies five keys in report_from_job; D2 adds one audit
kind and one append in set_gate; D3 adds two flags and pure metric functions
in report_outcomes.rs, default output untouched; D4 documents an optional
field.

Rejected: a merge timestamp from git commit dates (parses git, fragile across
squash); a stored scorecard at close (option B, later only if A's numbers
hold); inferring fix-first from titles (guesswork).

Waves: wave 1 runs osc-1, osc-2, osc-3 and osc-4 in parallel (disjoint
files; osc-3 reads the `gate` kind as a string, no compile dependency).
Wave 2 runs the three testers and osc-4c. osc-5 then osc-5c.

Plan check (inline, standard with five product files and no hard-gate
flag): structure — every D lands in a cell, deps form a DAG, each code cell
has a tester; cold pickup — each action names its anchors and its verify.
SMALLER PATH: dropping D2 would leave reopens unmeasurable; dropping D4
leaves fix-first unmeasurable; no cheaper shape keeps D3.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"osc-1, osc-2, osc-3 change Rust."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"Tester and checker cells, no shared context with implementers."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"osc-4 edits cells.md."},
    {"stage":"writing","classification":"required","role":"writer","reason":"osc-5 consolidates knowledge."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"The subsystem map."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"The gather covered it."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"User-invoked only."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"No competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"No competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"No competing designs."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Five product files and no hard-gate flag: the session runs the plan check inline."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline SMALLER PATH check."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"High-risk only."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"High-risk only."}
  ]
}
```

## Risk map

| Component | Risk | Why | Lands in | Proof |
|---|---|---|---|---|
| Default `report outcomes` output | MEDIUM | the one fact: existing tests pin the text, so any drift reds them | osc-3 | the existing report_outcomes tests stay green |
| Audit kinds list | LOW | a fixed-size array and any doc list must grow together | osc-2 | `cargo test … -- gate` green |
| Herded caps | LOW | new keys are copied only when present | osc-1 | from_job tests green |

## Shape

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Measure | scorecard, gate rows, reaction carry, fixes_feature | see whether agent work improves | 1 | unit tests per cell + full suite |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| osc-1 | Keep a herded worker's reaction on its cap | finish_support.rs | — | herded features show worker reaction counts | `cargo test … -- from_job` |
| osc-1t | Test the herded reaction carry | finish_support.rs tests | osc-1 | — | `-- from_job_carries` |
| osc-2 | Leave an audit row for every gate change | set_gate.rs, audit.rs | — | `bee audit list` shows `gate` rows | `-- gate` |
| osc-2t | Test the gate audit row | set_gate.rs tests | osc-2 | — | `-- gate_audit_row` |
| osc-3 | Report a per-feature scorecard | report_outcomes.rs | — | `bee report outcomes --by-feature` prints one scored line per feature | `-- report_outcomes` |
| osc-3t | Test the per-feature scorecard | report_outcomes.rs tests | osc-3 | — | `-- scorecard` |
| osc-4 | Teach plans the fixes_feature field | cells.md, regen output | — | cells.md documents `fixes_feature` | instruction_laws + manifest check |
| osc-4c | Check the planning doc edit | — | osc-4 | — | parity and pointer targets |
| osc-5 | Consolidate knowledge and write delivery | docs/knowledge/**, delivery.md (new) | osc-1t, osc-2t, osc-3t, osc-4c | concepts describe the scorecard | `.bee/bin/bee knowledge check` |
| osc-5c | Check the knowledge consolidation | — | osc-5 | — | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "osc-1",
    "feature": "outcome-scorecard",
    "title": "Keep a herded worker's reaction on its cap",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/outcome-scorecard/CONTEXT.md D1. In finish_support.rs report_from_job (anchor :1089 `map.insert(\"outcome\".to_string(), Value::String(outcome.to_string()));`) copy the herding result's `reaction`, `skipped`, `unknowns`, `found` and `stopped` into the report when present, in the shapes the native report contract validates (finish_support.rs:79-85, :309-354), so the cap writes them to the trace the way handlers_close.rs:810-820 does for the native path (`if let Some(reaction) = report_value.get(\"reaction\") {`). Read the values from the parsed handback (herding/mailbox.rs MailboxHandback :608-615). Absent stays absent. Edit only the existing tests the change forces; the tester cell adds the new ones. Run the verify and quote the count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- from_job",
    "must_haves": {
      "why": "bee report outcomes reads trace.reaction, so a herded cap that drops it makes every herded feature look unmeasured; brief-intent-and-wip showed not recorded on every layer.",
      "out_of_scope": [
        "counting dissent claims as departures",
        "report_outcomes.rs: osc-3 owns it"
      ],
      "truths": [
        "a --from-job cap of a result with reaction confirm writes trace.reaction confirm",
        "skipped, unknowns, found and stopped land on the trace when the result carries them",
        "a result without them leaves the trace keys absent"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "Never hand-edit .bee/*.json(l)"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "osc-1t",
    "feature": "outcome-scorecard",
    "title": "Test the herded reaction carry",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "osc-1"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Pin docs/history/outcome-scorecard/CONTEXT.md D1 with tests whose names contain `from_job_carries`, inside the test module of finish_support.rs (or the cells tests module the existing from_job tests use). Judge existing coverage first. Cover: reaction confirm carried; each of skipped, unknowns, found, stopped carried; absent keys stay absent; an invalid reaction value is handled as the native path handles it. Run the filter and quote its count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- from_job_carries",
    "must_haves": {
      "why": "A code cell never certifies itself.",
      "out_of_scope": [
        "production code"
      ],
      "truths": [
        "every case in the action has a test whose name contains from_job_carries",
        "the filter matches at least six tests and all pass"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "No edit outside test modules"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "osc-2",
    "feature": "outcome-scorecard",
    "title": "Leave an audit row for every gate change",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/audit.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/audit.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/outcome-scorecard/CONTEXT.md D2. Add kind `gate` to audit.rs KINDS (`pub(crate) const KINDS: [&str; 18] = [`) and to any test or doc list that pins the kinds. In set_gate.rs, beside the existing auto-only append (anchor :1071 `crate::audit::append(`), append one `gate` row for every successful gate set, manual or auto, with note `gate <name> set to <true|false> by <actor>` and the lane's feature; a merged shape+execution set writes one row per gate. Keep the `gate-bypass` row unchanged. A failed or refused set writes nothing. Edit only the existing tests the change forces; the tester cell adds the new ones. Run the verify and quote the count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- gate",
    "must_haves": {
      "why": "Without a row for manual gate changes nobody can count how often a plan was reopened, which is the cost a weak plan puts on the owner.",
      "out_of_scope": [
        "report_outcomes.rs: osc-3 owns it",
        "changing gate semantics or refusals"
      ],
      "truths": [
        "a manual bee gate set appends one audit row of kind gate naming the gate, the value and the actor",
        "an auto set appends the gate row and keeps its gate-bypass row",
        "a refused set appends nothing",
        "bee audit list shows the gate kind"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "Never hand-edit .bee/*.json(l)"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "osc-2t",
    "feature": "outcome-scorecard",
    "title": "Test the gate audit row",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "osc-2"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Pin docs/history/outcome-scorecard/CONTEXT.md D2 with tests whose names contain `gate_audit_row`, inside the test module of set_gate.rs. Cover: manual approve, manual unapprove, auto set (both rows), merged set (one row per gate), refused set (no row). Run the filter and quote its count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- gate_audit_row",
    "must_haves": {
      "why": "A code cell never certifies itself.",
      "out_of_scope": [
        "production code"
      ],
      "truths": [
        "every case in the action has a test whose name contains gate_audit_row",
        "the filter matches at least five tests and all pass"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "No edit outside test modules"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "osc-3",
    "feature": "outcome-scorecard",
    "title": "Report a per-feature scorecard",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/outcome-scorecard/CONTEXT.md D3 in report_outcomes.rs. Add flags `--by-feature` and `--since <duration like 7d or 48h, or an ISO timestamp>` to try_native (:29-67). Keep today's output byte-identical when neither flag is given. With --by-feature, compute per feature from the cells already scanned plus: lane `created_at` from `.bee/lanes/<feature>.json`; `closed_at` from `.bee/usage/<feature>.json` (close.rs write_usage_record :2567-2578); the newest `trace.capped_at`; audit rows from `.bee/audit.jsonl` of kind `gate` or `gate-bypass` whose note contains ` set to false`; cells whose top-level `fixes_feature` equals the feature; `usage.cost_usd` and `usage.total_tokens` summed from `.bee/mailbox/*/job.json` whose `feature` matches (herding/run.rs record_round_usage :3227). Print a header line `features: <n> (closed <c>) · rework rate <r>% overall`, then one line per feature, oldest first, each metric as a value or `not recorded`, never 0 for a missing source; --json returns {header, features:[...]} with null for not recorded. Keep each metric in a pure function over parsed records; read the store from the main root as today. Edit only the existing tests the change forces; the tester cell adds the new ones. Run the verify and quote the count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- report_outcomes",
    "must_haves": {
      "why": "The owner wants to see whether agent work is getting better or worse; counts without rates, time or cost cannot show that, and the article's countermeasure rule needs throughput beside rework.",
      "out_of_scope": [
        "writing any record",
        "the gate audit row: osc-2 owns it",
        "the from-job carry: osc-1 owns it",
        "a stored scorecard at bee close (option B)"
      ],
      "truths": [
        "bee report outcomes with no new flag prints exactly what it printed before",
        "--by-feature prints one line per feature with cells, rework rate, judge revision rate, reaction coverage, cycle time, gate reopens, fix-first cells and cost",
        "a metric with no source prints not recorded",
        "--since 7d drops features whose lane started earlier",
        "--json carries null for not recorded"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "Never hand-edit .bee/*.json(l)"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "osc-3t",
    "feature": "outcome-scorecard",
    "title": "Test the per-feature scorecard",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "osc-3"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Pin docs/history/outcome-scorecard/CONTEXT.md D3 with tests whose names contain `scorecard`, inside the test module of report_outcomes.rs, extending its fixture store (setup_fixture_store :763). Cover each metric with a source and without (not recorded), cycle time from usage closed_at and from capped_at fallback, gate reopen counting from both kinds, fix-first counting, cost summing across two jobs, --since with a duration and an ISO bound, json nulls, and the no-flag output unchanged against the current golden. Run the filter and quote its count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- scorecard",
    "must_haves": {
      "why": "A code cell never certifies itself.",
      "out_of_scope": [
        "production code"
      ],
      "truths": [
        "every case in the action has a test whose name contains scorecard",
        "the filter matches at least ten tests and all pass"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No code comments",
        "No edit outside test modules"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "osc-4",
    "feature": "outcome-scorecard",
    "title": "Teach plans the fixes_feature field",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "skills/bee-planning/references/cells.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md",
      "skills/bee-planning/references/cells.md"
    ],
    "affects_skills": [
      "skills/bee-planning/references/cells.md"
    ],
    "affects_specs": [],
    "action": "Implement docs/history/outcome-scorecard/CONTEXT.md D4 in skills/bee-planning/references/cells.md: in the cell field docs, add the optional top-level `fixes_feature: <slug>` and say a fix-first cell (the known-red-base rule) sets it to the feature whose defect it fixes, so `bee report outcomes --by-feature` can count it. Cite, never copy. Inside this cell run the full regen chain `bee dev regen` (render-skill-trees, onboard --repo-root . --apply, release-manifest --write) and commit its output with the change. Plain words.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "why": "Fix-first cells are invisible to the scorecard unless plan writers know to tag them.",
      "out_of_scope": [
        "Rust code",
        "LEADER.md and AGENTS.md"
      ],
      "truths": [
        "cells.md documents fixes_feature and when to set it",
        "regen output is committed with the edit"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No copy of a fact that already has a home"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "osc-4c",
    "feature": "outcome-scorecard",
    "title": "Check the planning doc edit",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "osc-4"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "skills/bee-planning/references/cells.md"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [
      "skills/bee-planning/references/cells.md"
    ],
    "affects_specs": [],
    "action": "Run `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws --test pointer_integrity --test rule_index_parity` over osc-4's commit and read its diff against CONTEXT.md D4. Edit nothing; report results.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws",
    "must_haves": {
      "why": "A docs cell needs a second worker to check parity and pointers.",
      "out_of_scope": [
        "editing any file"
      ],
      "truths": [
        "the three targets are green or each red is reported with output"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No edits"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    },
    "regen_obligation_ack": "read-only check cell: it writes nothing, so it owes no regen"
  },
  {
    "id": "osc-5",
    "feature": "outcome-scorecard",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "osc-1t",
      "osc-2t",
      "osc-3t",
      "osc-4c"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "docs/knowledge/**",
      "docs/history/outcome-scorecard/delivery.md"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run `bee knowledge consolidate --feature outcome-scorecard`. Dedupe and trim the deltas the cells wrote, anchor each concept to the code it describes, never restate code, and write docs/history/outcome-scorecard/delivery.md listing the concepts changed and why, one line each. Concepts must cover the from-job carry, the gate audit kind, the scorecard metrics with their sources, and fixes_feature; cite D-numbers, never raw decision ids. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "why": "docs/knowledge is the state layer; a concept that lags the code misleads the next reader.",
      "out_of_scope": [
        "code",
        "skills/"
      ],
      "truths": [
        "concepts describe D1-D4 anchored to code",
        "delivery.md lists every concept the feature changed"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No restating of code in area concepts",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "osc-5c",
    "feature": "outcome-scorecard",
    "title": "Check the knowledge consolidation",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "osc-5"
    ],
    "decisions": [
      "459dfff7-8215-42c3-8a3f-e3c6856b3137"
    ],
    "files": [
      "docs/history/outcome-scorecard/delivery.md"
    ],
    "read_first": [
      "docs/history/outcome-scorecard/CONTEXT.md",
      "docs/history/outcome-scorecard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Check osc-5's commit read-only: run `.bee/bin/bee knowledge check`, confirm delivery.md lists every concept the feature changed and that concepts cite D-numbers, never raw decision ids. Edit nothing; report each mismatch.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "why": "The writer cell needs a separate test-role check before the feature can close.",
      "out_of_scope": [
        "editing any file"
      ],
      "truths": [
        "bee knowledge check is green or its red is reported",
        "every concept named in delivery.md exists"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "No edits"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| report outcomes, no new flag | edge (same on main and head) | output identical |
| --by-feature with full fixture | happy | every metric has a value |
| feature with no jobs, no close record | edge | cost and closed-based cycle time print `not recorded`; cycle time falls back to capped_at |
| --since 7d | happy | older features dropped |
| herded cap with reaction confirm | happy | trace.reaction is confirm |
| manual gate unapprove | happy | one `gate` audit row, note ends `set to false by user` |
| refused gate set | error | no audit row |

## Open Questions

(none)

## Out of scope

A scorecard written at `bee close` (option B). Back-filling reaction for
cells already capped. Detecting reverts from git history.
