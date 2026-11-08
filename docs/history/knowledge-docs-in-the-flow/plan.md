---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Knowledge docs in the flow

## Summary

A feature's context file now says what the agent read before deciding: which
knowledge concepts, how fresh each was, and which code paths decide what. The
plan approval refuses a context file that skips that section. The templates
agents write documents into ask one question per section and say when to
delete a line, a rule can say it is temporary and which test proves it, and
the knowledge check refuses an area entry that links to no decision. Nothing
is pushed into worker briefs.

Mode: `standard` — 2 risk flags: public-contracts (the CONTEXT.md grammar and
a new check finding), multi-domain (Rust verbs and skill references).
Why this is the least workflow that protects the work: two refusals land in
doors every feature passes, so each ships with its tests and a recorded
escape, while the template work is docs that the parity tests already pin.

## Requirements (from CONTEXT.md)

- D1: documentation is a working part of the flow — shaping and planning
  record a pulled read set, cap and close update what changed, templates ask
  for information, substance over volume.
- D2: the read set is a `## Read set` section of CONTEXT.md with a knowledge
  table (path, grade, why) and a code table (path, what it decides); shaping
  fills it, planning extends it, `read_first` cites it, shape approval checks
  the section exists and every knowledge path resolves; nothing pushed into
  briefs.
- D3: every document template's sections open with the one question they
  answer and name their sources, plus a deletion line; rule bullets carry
  status and a verification pointer only where they differ from the concept;
  the cut area template is map, boundaries, invariants, entry points,
  pointers.
- D4: `bee knowledge check` refuses an area concept with no decision link as
  a profile error naming `bee decisions log` and the `decisions` list.
- D5: pull only; `brief.rs` and the dispatch prompt renderer untouched.
- D6: portable — binary and skills only, no host configuration.

## Load-bearing claims

Labels are `read` (opened the file at that line this session), `ran` (ran the
command and hold its output) or `guessed`; evidence is a verbatim byte
substring of the anchored line, multi-line joined with " / ".

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | CONTEXT.md already has one heading-scoped table parser the read-set parser can sit beside | read | packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs:25 | `let heading_idx = lines.iter().position(\|l\| l.trim() == "## Locked Decisions")?;` |
| 2 | Shape and merged approval already run a zero-mutation refusal before writing, where the read-set refusal joins | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:949-950 | `if approved && (merge \|\| name == "shape") {` / `if let Some(msg) = gate_preview_refusal(root, target.record(), target.lane()) {` |
| 3 | The preview refusal is a function taking the root, the record and the lane, returning an optional message | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:563 | `pub(crate) fn gate_preview_refusal(` |
| 4 | The close driver resolves CONTEXT.md under docs/history/<feature>/ from a docs root, the same path the gate will read | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1126 | `let context_path = docs_root.join("docs").join("history").join(feature).join("CONTEXT.md");` |
| 5 | `check_bundle` is the one function that builds finding rows, and profile errors use the `finding(path, code, message)` shape | read | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:557 | `pub(crate) fn check_bundle(dir: &Path, strict: bool) -> Option<CheckReport> {` |
| 6 | An existing profile error, `owns_missing`, is the row shape and remedy wording to copy | read | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:896 | `"owns_missing",` |
| 7 | An existing test pins that profile error, so the new one has a test shape to copy | read | packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs:2045 | `fn owns_missing_errors_when_area_overview_carries_no_owns_keys() {` |
| 8 | Frontmatter round-trips through one emitter, so `bee.decisions` needs no new field | read | packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs:90 | `pub(crate) fn emit_frontmatter(data: &Map<String, Value>) -> Result<String, ()> {` |
| 9 | The CONTEXT.md template's Locked Decisions table header is the grammar the read-set tables mirror | read | skills/bee-shaping/references/context-template.md:26 | `\| ID \| Decision \| Rationale (only if it changes implementation) \|` |
| 10 | The area template already names per-section sources, the form D3 generalises | read | skills/bee-capturing/references/area-spec.md:33 | `\| Source \| May feed \| Never feeds \|` |
| 11 | Exactly five area concepts carry no decision link today, so D4 as an error is a bounded fix | ran | `python3 -I` over docs/knowledge/areas/*/*.md counting empty or absent `decisions:` | `140 concepts; 5 without decisions:` |
| 12 | Skill edits reach the rendered trees through one regen verb | ran | `.bee/bin/bee dev regen --help` | `Run the three-step regen chain in order — render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write` |

## Discovery

Inspected the gate approval path, the CONTEXT.md parser, the knowledge check
and the four skill references the templates live in (claims 1-10). A gather
worker read check.rs, frontmatter.rs and the skill references in full; the
Rust gate and render-chain anchors were read by the leader. Measured the
decision-link gap (claim 11). The read-set section in this feature's own
CONTEXT.md is the first instance of the grammar and the fixture the parser
tests copy.

## Approach

Recommended path: one slice, four cells.

- kdf-1 (per D2, D5): `parse_read_set` beside `parse_locked_decision_ids`
  returns the knowledge paths of the `## Read set` section's first table
  whose header starts with `| Concept |`; a new `read_set_refusal(root,
  feature)` beside `claims_refusal` runs at ONE call site, the pre-lock peek
  where `plan_claims_refusal` already runs for `shape` and `--merge`
  approvals (set_gate.rs:934 `if let Some(msg) = plan_claims_refusal(root,
  &peek, lane) {`), with the same `tiny|small|docs` lane exemption
  `gate_preview_refusal` uses (plan_packets.rs:584 `let is_exempt =
  matches!(mode.as_deref(), Some("tiny") | Some("small") | Some("docs"));`)
  and CONTEXT.md resolved the worktree-aware way `advisor_plan_path`
  resolves plan.md. No `## Read set` section, a CONTEXT.md that is missing,
  or a knowledge path that does not resolve under docs/knowledge/, refuses
  with the path and the remedy; a `<!-- bee:no-read-set: <reason> -->`
  marker anywhere in CONTEXT.md passes — the marker in the file is the
  record, no gate output field is added.
- kdf-2 (per D4): `missing_decision_link` profile error in `check_bundle`
  for every concept of type bee.area — an area's overview.md included, only
  the generated index.md (no frontmatter) excluded — whose `bee.decisions`
  is absent or empty, added as a branch of the existing bee.area loop in
  check.rs rather than a new per-concept loop. The error fires only when
  the bundle already carries at least one area concept with a non-empty
  `bee.decisions`; a bundle with none (a host that stood its bundle up with
  `bee knowledge bootstrap` or `bee knowledge new`, which write no
  decisions) gets one NOTE naming the rule, exactly as the no-anchors case
  does today — so a host is never red on records it was never asked to
  keep (D6; hat-user-impact gray area 5). The five bee concepts that fail
  today gain their decision links from `bee decisions search`.
- kdf-3 (per D2, D3): the templates — context-template.md gains `## Read
  set` with the two tables; bee-shaping Explore runs `bee knowledge context`
  and `bee knowledge search` and Lock renders the section; planning-reference
  `read_first` cites the read set and the plan's Discovery extends it;
  area-spec.md sections open with their question and sources and carry a
  deletion line, Business Rules bullets gain the optional `— status: <active|temporary|retired>; proves: <path>` tail, and a new "Cut area template"
  section holds map, boundaries, invariants, entry points, pointers;
  concept-authoring.md's "Write through the CLI" step points at the section
  questions, the deletion line and the rule-bullet tail as the body contract
  every concept type follows; then `bee dev regen`.
- kdf-4 (per D1-D4): the okf-profile concept that records the read set, the
  templates rule and the decision-link error, with anchors, and the
  bee-capturing SKILL.md paragraph that cites it.

Rejected alternatives:
- A separate read-set.md manifest — a second file to keep true (map D11).
- A warning instead of an error for the missing decision link — five
  concepts fail today, a warning would never clear them (D4 says refuse).
- Pushing the read set into worker briefs — rejected by D5.
- A `read_set_waived` field in the gate's `--json` output — no consumer
  exists; the marker in CONTEXT.md is the record (hat-alternatives S2).
- Merging kdf-3 and kdf-4 into one cell — loses kdf-3's parallel run beside
  the Rust cells and mixes the docs and writer roles; a second regen costs
  seconds (hat-alternatives S5).
- Two call sites for the read-set refusal — CONTEXT.md is a file, not a
  record field, so the pre-lock peek is the one place it can change the
  answer (hat-alternatives S1).

Risk map:

| Component | Level | Lands in | Proof needed |
|---|---|---|---|
| Read-set refusal blocks a legacy CONTEXT.md mid-flight | MEDIUM — the only in-flight lane other than this one is paseo-worktree-workspace at phase idle (`bee orient` 2026-11-08: "phase=idle feature=paseo-worktree-workspace") | kdf-1 | test: a CONTEXT.md with the `bee:no-read-set` marker approves; the refusal text names the marker |
| `missing_decision_link` reds bee's own bundle | MEDIUM — five concepts measured (claim 11) | kdf-2 | `bee knowledge check` exits 0 on the branch after the five are linked |
| A host's in-flight features and never-linked bundles are unmeasurable from here | MEDIUM — the marker is the whole mitigation for a legacy CONTEXT.md; the arm-on-first-link rule is the whole mitigation for a bootstrapped bundle | kdf-1, kdf-2 | the two host matrix rows: tiny exempt and zero-link bundle NOTE |
| Skill reference edits drift from the rendered trees | LOW | kdf-3 | parity tests green after `bee dev regen` |
| Template growth makes agents fill shape instead of information | LOW | kdf-3 | each new section line is one question plus sources, no sub-bullets |

Waves: kdf-1, kdf-2 and kdf-3 run in parallel (disjoint files; the read-set
grammar is fixed here, in Approach, so the template and the parser agree
without a serial edge); kdf-4 runs after all three.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage": "rust-doors", "classification": "required", "role": "code", "reason": "kdf-1 and kdf-2 write Rust verbs and their tests"},
    {"stage": "template-edits", "classification": "required", "role": "docs", "reason": "kdf-3 edits skill references and regenerates the trees"},
    {"stage": "knowledge-sync", "classification": "required", "role": "writer", "reason": "kdf-4 writes the okf-profile concept and the capturing paragraph"},
    {"stage": "code-scout", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of more than three files", "reason": "I/O offload"},
    {"stage": "test-repair", "classification": "conditional", "role": "test", "condition": "a cell's related tests red for a reason outside its files", "reason": "red-first repair"},
    {"stage": "release", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "plan-drafting", "classification": "not-applicable", "role": "plan", "reason": "the leader drafted the plan inline"},
    {"stage": "fact-lookup", "classification": "conditional", "role": "extraction", "condition": "a single fact from a known location", "reason": "cheap reader"},
    {"stage": "fallback-writer", "classification": "not-applicable", "role": "generation", "reason": "every cell names its role"},
    {"stage": "cell-review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "high-risk-consult", "classification": "not-applicable", "role": "advisor", "reason": "standard lane"},
    {"stage": "observer", "classification": "not-applicable", "role": "supervisor", "reason": "no unattended control loop"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no open design shape"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no open design shape"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no open design shape"},
    {"stage": "plan-check-facts", "classification": "required", "role": "hat-facts-gaps", "reason": "hat wave seat 1"},
    {"stage": "plan-check-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "standard lane opens three seats"},
    {"stage": "plan-check-value", "classification": "not-applicable", "role": "hat-value", "reason": "standard lane opens three seats"},
    {"stage": "plan-check-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "hat wave seat 2"},
    {"stage": "plan-check-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "hat wave seat 3"},
    {"stage": "ui", "classification": "not-applicable", "role": "design", "reason": "no rendered UI"}
  ]
}
```

## Shape

Phase plan (one slice):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Read-set parser and refusal, decision-link error, templates, knowledge sync | The owner agreed D9-D12 on 2026-11-08 and the content cut waits on the template | `bee gate --merge` refuses a CONTEXT.md with no read set; `bee knowledge check` names the unlinked concept; a new feature's Lock renders `## Read set` | PBI p-3a756890 (the content cut) |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| kdf-1 | Refuse a shape or merged approval whose CONTEXT.md has no read set | packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs, packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs, packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs, packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs, packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs | — | `bee gate --merge --approved true` on a standard feature whose CONTEXT.md lacks `## Read set` prints the refusal naming the section and the `bee:no-read-set` marker; with the section and resolving paths it approves; tiny, small and docs lanes are exempt | `cargo test … --bin bee -- read_set` green |
| kdf-2 | Refuse an area concept with no decision link in knowledge check | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs, packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs, docs/knowledge/areas/hook-runtime/argv-decidable-refusals-precede-the-stdin-read.md, docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md, docs/knowledge/areas/rust-runtime/parity-and-conformance-proof-discipline.md, docs/knowledge/areas/verify-pipeline/verify-pipeline-skill-trigger-cases.md, docs/knowledge/areas/worktree-parallelism/path-identity.md | — | `bee knowledge check` on a bundle with an unlinked area concept prints `missing_decision_link` with the remedy; bee's own bundle checks clean | `cargo test … --bin bee -- missing_decision_link` green and `bee knowledge check` OK |
| kdf-3 | Give every document template its section questions, deletion lines, the read set and the rule bullet fields | skills/bee-shaping/references/context-template.md, skills/bee-shaping/SKILL.md, skills/bee-planning/SKILL.md, skills/bee-planning/references/planning-reference.md, skills/bee-capturing/references/area-spec.md, skills/bee-capturing/references/concept-authoring.md, .agents/skills/*, .claude/skills/*, .codex/skills/*, .bee/onboarding.json, docs/history/codex-harness-hardening/release-manifest.json | — | a new feature's CONTEXT.md template shows `## Read set`; bee-shaping tells the agent to run `bee knowledge context` and `bee knowledge search` before Lock; area-spec sections open with their question; a rule bullet can end `— status: temporary; proves: <path>` | parity tests green after `bee dev regen`; `bee dev release-manifest --check` match |
| kdf-4 | Record the read set, the templates rule and the decision-link error in the OKF profile | docs/knowledge/areas/okf-profile/freshness-and-document-homes.md, docs/knowledge/areas/okf-profile/overview.md, docs/knowledge/areas/okf-profile/index.md, docs/knowledge/index.md, skills/bee-capturing/SKILL.md, .agents/skills/*, .claude/skills/*, .codex/skills/*, .bee/onboarding.json, docs/history/codex-harness-hardening/release-manifest.json | kdf-1, kdf-2, kdf-3 | `bee knowledge search --text "read set"` returns the okf-profile concept with its grade; bee-capturing names the templates rule | `bee knowledge check` OK and `bee knowledge index --check` 0 stale; parity tests green |

```json
[
  {
    "id": "kdf-1",
    "feature": "knowledge-docs-in-the-flow",
    "title": "Refuse a shape or merged approval whose CONTEXT.md has no read set",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D2", "D5"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-docs-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-docs-in-the-flow/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add `parse_read_set(text: &str) -> Option<Vec<String>>` to context_table.rs beside `parse_locked_decision_ids` (context_table.rs:23 `pub(crate) fn parse_locked_decision_ids(text: &str) -> Option<Vec<String>> {`), the same prefix-tolerant shape (reuse its loop, `is_separator_row` and `skip_table`): find the `## Read set` heading, then before the next `## ` heading the first pipe table whose header row starts with `| Concept |`, and return its first-column cells trimmed of backticks; `None` when the heading or the table is absent (per D2; grammar fixed in plan.md Approach). Add `read_set_refusal(root: &Path, feature: &str) -> Option<String>` in plan_packets.rs beside `claims_refusal`, copying its skeleton: exempt `tiny|small|docs` lanes exactly as plan_packets.rs:584 does (`let is_exempt = matches!(mode.as_deref(), Some(\"tiny\") | Some(\"small\") | Some(\"docs\"));`), resolve `docs/history/<feature>/CONTEXT.md` the worktree-aware way `advisor_plan_path` resolves plan.md; a CONTEXT.md that cannot be read refuses, naming the path it looked for; when the file carries a `<!-- bee:no-read-set: <reason> -->` marker anywhere return None (the marker in the file is the record; add no gate output field); when `parse_read_set` is None refuse with this message shape: `gate: approval refused — docs/history/<feature>/CONTEXT.md has no \"## Read set\" section. FIX: run \\`bee knowledge context --work <feature>\\` and \\`bee knowledge search --text \"<topic>\"\\`, write the section (skills/bee-shaping/references/context-template.md), or add \\`<!-- bee:no-read-set: <reason> -->\\` for a CONTEXT.md written before the read set existed.`; an empty knowledge table (section present, zero rows) passes; when any returned path does not start with `docs/knowledge/` or does not exist under the repo root, refuse naming that path and the remedy (fix the path or remove the row). Call `read_set_refusal` at exactly ONE site: the pre-lock peek where `plan_claims_refusal` already runs for `shape` and `--merge` approvals (set_gate.rs:934 `if let Some(msg) = plan_claims_refusal(root, &peek, lane) {`), right after it, never on a plain `--name execution` (per D2). Write the seven test-matrix rows for kdf-1 as tests whose fn names contain `read_set`: the parser cases in drivers/tests.rs beside the existing CONTEXT.md fixture helper (drivers/tests.rs:6332 `fn write_locked_decisions_table(root: &Path, feature: &str, ids: &[&str]) {`), the gate approval cases in state_group/tests.rs beside the existing gate approval tests. No code comments. Do not touch brief.rs or the prompt renderer (per D5).",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- read_set",
    "must_haves": {
      "truths": [
        "bee gate --merge --approved true on a standard feature whose CONTEXT.md lacks ## Read set refuses, names the section and the bee:no-read-set marker, and writes no state",
        "the same approval with the section and every knowledge path on disk approves",
        "a CONTEXT.md carrying <!-- bee:no-read-set: legacy --> approves",
        "a tiny lane with no section approves; a plain --name execution approval never runs the check"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs", "substantive": "parse_read_set with the heading-scoped, header-prefix table grammar"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "read_set_refusal with the lane exemption, the marker escape and the path check"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs", "substantive": "gate approval tests whose names contain read_set: happy, missing path, marker, missing section, absent file, tiny exempt, execution-only"}
      ],
      "key_links": [
        "set_gate.rs calls read_set_refusal once, at the pre-lock peek right after plan_claims_refusal"
      ],
      "prohibitions": [
        "No change to brief.rs or the dispatch prompt renderer",
        "No change to gate_preview_refusal's or claims_refusal's behavior",
        "No new field in the gate's --json output",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kdf-2",
    "feature": "knowledge-docs-in-the-flow",
    "title": "Refuse an area concept with no decision link in knowledge check",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D4", "D6"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "docs/knowledge/areas/hook-runtime/argv-decidable-refusals-precede-the-stdin-read.md",
      "docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md",
      "docs/knowledge/areas/rust-runtime/parity-and-conformance-proof-discipline.md",
      "docs/knowledge/areas/verify-pipeline/verify-pipeline-skill-trigger-cases.md",
      "docs/knowledge/areas/worktree-parallelism/path-identity.md"
    ],
    "read_first": [
      "docs/history/knowledge-docs-in-the-flow/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs",
      "docs/knowledge/areas/okf-profile/concept-model-and-authoring.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In `check_bundle` (check.rs:557 `pub(crate) fn check_bundle(dir: &Path, strict: bool) -> Option<CheckReport> {`) add the profile error `missing_decision_link` for every concept of type bee.area — an area's overview.md included, only the generated index.md (which has no frontmatter) excluded — whose `bee.decisions` is absent or an empty list, as a branch of the existing bee.area loop in check_bundle rather than a new per-concept loop. The error fires only when at least one area concept in the bundle carries a non-empty `bee.decisions`; when none does, emit one NOTE (the shape of the no-anchors NOTE) saying no area concept links a decision and that the rule arms on the first link, and no error rows (per D6; a bootstrapped host must not red). The row uses the same `finding(path, code, message)` shape as `owns_missing` (check.rs:896 `\"owns_missing\",`) and its message names the concept path, the remedy `bee decisions search --text \"<topic>\"` then `bee decisions log`, and the `decisions` list the id goes into (per D4). Patterns, work items and decision concepts are never checked. Add tests in knowledge/tests.rs with fn names containing `missing_decision_link`, modelled on tests.rs:2045 `fn owns_missing_errors_when_area_overview_carries_no_owns_keys() {`: an area concept with one decision gives no row; an area concept with `decisions: []` gives exactly one profile error row with that code; a pattern concept with no decisions gives no row. Then link the five area concepts named in files: for each, find the decision that settled it with `bee decisions search --text \"<topic>\"` (or the feature's docs/history CONTEXT.md D-number) and add it to `bee.decisions` in the canonical form the rest of the bundle uses (`<feature> D<n> (<short8>): <gist>` when a history feature exists, else the short8 id and a gist), keeping the frontmatter canonical (frontmatter.rs:90 `pub(crate) fn emit_frontmatter(data: &Map<String, Value>) -> Result<String, ()> {`). Portable per D6: no bee-repo path in the check. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- missing_decision_link && cargo run --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- knowledge check",
    "must_haves": {
      "truths": [
        "bee knowledge check on a bundle holding an area concept with decisions: [] prints one profile error row missing_decision_link whose message names bee decisions log and the concept path",
        "a pattern concept with no decisions produces no row",
        "a bundle in which no area concept carries a decision gets one NOTE and no missing_decision_link row",
        "bee knowledge check on this branch's bundle reports 0 OKF errors and 0 profile errors"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs", "substantive": "missing_decision_link profile error with its remedy message"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs", "substantive": "three tests whose names contain missing_decision_link"},
        {"path": "docs/knowledge/areas/worktree-parallelism/path-identity.md", "substantive": "bee.decisions carries the settling decision, frontmatter canonical"}
      ],
      "key_links": [
        "the new row is produced inside check_bundle and counted as a profile error, so strict mode exits non-zero on it"
      ],
      "prohibitions": [
        "No new frontmatter field",
        "No bee-repo-only path in the check",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kdf-3",
    "feature": "knowledge-docs-in-the-flow",
    "title": "Give every document template its section questions, deletion lines, the read set and the rule bullet fields",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["D2", "D3"],
    "files": [
      "skills/bee-shaping/references/context-template.md",
      "skills/bee-shaping/SKILL.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-planning/references/planning-reference.md",
      "skills/bee-capturing/references/area-spec.md",
      "skills/bee-capturing/references/concept-authoring.md",
      ".agents/skills/*",
      ".claude/skills/*",
      ".codex/skills/*",
      ".bee/onboarding.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/knowledge-docs-in-the-flow/CONTEXT.md",
      "docs/history/knowledge-docs-in-the-flow/plan.md",
      "skills/bee-shaping/references/context-template.md",
      "skills/bee-capturing/references/area-spec.md",
      "skills/bee-planning/references/planning-reference.md",
      "skills/bee-shaping/SKILL.md",
      "skills/bee-planning/SKILL.md"
    ],
    "affects_skills": [
      "skills/bee-shaping/references/context-template.md",
      "skills/bee-shaping/SKILL.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-planning/references/planning-reference.md",
      "skills/bee-capturing/references/area-spec.md",
      "skills/bee-capturing/references/concept-authoring.md"
    ],
    "affects_specs": [],
    "action": "Templates ask for information, not shape (per D3), and the read set lives in CONTEXT.md (per D2). (1) context-template.md: add a `## Read set` section after `## Terms` with the grammar fixed in plan.md Approach — `### Knowledge` table `| Concept | Grade at read | Why it matters |` and `### Code` table `| Path | What it decides |` — each table preceded by one line saying the question it answers and the source that feeds it (`bee knowledge context --work <feature>`, `bee knowledge search --text \"<topic>\"`, the scout), one deletion line (drop a row the plan never cites), and the one-row form a host with no docs/knowledge/ writes in the knowledge table (`| (no bundle) | — | this host has no docs/knowledge/ |`). (2) bee-shaping SKILL.md: in Explore, the scout that already starts at the area index runs `bee knowledge context` and `bee knowledge search --text \"<topic>\"` and keeps the hits with their printed grade — the read set is recorded from that scout, never a second read pass; in Lock, the `## Read set` section renders those hits and the scout's code paths, and the section is named as what the shape gate checks, with the `<!-- bee:no-read-set: <reason> -->` marker as the recorded escape for a context written before the section existed. (3) bee-planning SKILL.md Research and planning-reference.md: a cell's `read_first` cites paths from the read set first (planning-reference.md:282-283 `read_first` what it must read.`), and the plan's `## Discovery` extends the read set with what planning read. (4) area-spec.md: every template section (Entry Points & Triggers through Pointers) opens with one line naming the question the section answers for a rebuilder and pointing at its row of the Gather Sources table (never repeating the row), plus a deletion line specific to that section (Behaviors: a behavior a test already states; Business Rules: a bullet a test already states; Data Dictionary: an element the code names and nothing explains — one sentence each, no generic line repeated); the Business Rules section gains the optional rule-bullet tail `— status: active|temporary|retired; proves: <test or verify feature path>` used only where the rule differs from its concept, and this tail replaces the ad-hoc parenthetical status form the template shows today (`R2 (not yet implemented — backlog b-12)`); add a `## Cut area template` section holding exactly map, boundaries, invariants, entry points, pointers, each with its question line, opened by one line saying the content cut (PBI p-3a756890) uses it and hosts keep the nine-section template until told otherwise. (5) concept-authoring.md, step \"Write through the CLI\": one paragraph saying every concept type's body follows the section questions, the deletion line and the rule-bullet tail of area-spec.md, with a link to that section. Keep every addition to one question line plus sources per section; no sub-bullets. Then run `.bee/bin/bee dev regen` so .agents/skills, .claude/skills, .codex/skills, .bee/onboarding.json and the release manifest record follow, and commit them with the cell. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "context-template.md renders a ## Read set section with the Knowledge and Code tables and their question lines",
        "bee-shaping SKILL.md tells the agent to run bee knowledge context and bee knowledge search before Lock and names the bee:no-read-set marker",
        "area-spec.md sections each open with one question line and sources, Business Rules carries the deletion line and the optional status/proves tail, and a Cut area template section exists",
        "the rendered skill trees match skills/ after bee dev regen"
      ],
      "artifacts": [
        {"path": "skills/bee-shaping/references/context-template.md", "substantive": "## Read set with both tables and question lines"},
        {"path": "skills/bee-capturing/references/area-spec.md", "substantive": "question lines per section, deletion line, rule bullet tail, Cut area template"},
        {"path": "skills/bee-planning/references/planning-reference.md", "substantive": "read_first cites the read set"}
      ],
      "key_links": [
        "bee-shaping SKILL.md Lock names the ## Read set section the shape gate checks",
        "bee dev regen was run and the rendered trees are committed with the cell"
      ],
      "prohibitions": [
        "No new template file; existing references are extended",
        "No rule marker (<!-- rule: -->) added or moved",
        "No code comments"
      ],
      "open_choices": [
        "Whether the rule-bullet tail uses an inline em-dash form or a sub-bullet"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "kdf-4",
    "feature": "knowledge-docs-in-the-flow",
    "title": "Record the read set, the templates rule and the decision-link error in the OKF profile",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": ["kdf-1", "kdf-2", "kdf-3"],
    "decisions": ["D1", "D2", "D3", "D4"],
    "files": [
      "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md",
      "docs/knowledge/areas/okf-profile/overview.md",
      "docs/knowledge/areas/okf-profile/index.md",
      "docs/knowledge/index.md",
      "skills/bee-capturing/SKILL.md",
      ".agents/skills/*",
      ".claude/skills/*",
      ".codex/skills/*",
      ".bee/onboarding.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/knowledge-docs-in-the-flow/CONTEXT.md",
      "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md",
      "docs/knowledge/areas/okf-profile/overview.md",
      "skills/bee-capturing/references/concept-authoring.md",
      "skills/bee-capturing/SKILL.md"
    ],
    "affects_skills": ["skills/bee-capturing/SKILL.md"],
    "affects_specs": [],
    "action": "Extend the okf-profile concept freshness-and-document-homes.md (never a new concept) with three sections written through bee-technical-writing: the read set (what CONTEXT.md records, the grammar, the shape-gate refusal and its marker escape, per D2), the templates rule (sections ask one question and name their sources, the deletion line, the rule-bullet status and proves tail, the cut area template, per D3), and the decision-link error (`missing_decision_link`, its remedy, per D4), each citing knowledge-docs-in-the-flow D1-D4 in `bee.decisions` and adding the new code paths from kdf-1 and kdf-2 to `bee.anchors`; update `verified_at` and `verified_commit` with `bee knowledge confirm --id okf-profile-freshness-and-document-homes` after the edit. Update overview.md's split paragraph and index.md so the concept's description names the read set and the templates rule, and regenerate docs/knowledge/index.md with `bee knowledge index`. Add one paragraph to skills/bee-capturing/SKILL.md (Scribe section) saying the area template's sections open with their question and sources and a rule bullet may carry status and proves, citing the concept; then run `.bee/bin/bee dev regen` and commit the rendered trees with the cell. Rebuild test: a stranger could rebuild the read-set refusal and the check error from the concept alone.",
    "verify": ".bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "bee knowledge search --text \"read set\" returns okf-profile/freshness-and-document-homes.md",
        "the concept's bee.decisions cites knowledge-docs-in-the-flow D1, D2, D3 and D4 and its bee.anchors names context_table.rs, plan_packets.rs and check.rs",
        "bee knowledge check and bee knowledge index --check report 0 errors and 0 stale"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md", "substantive": "three new sections: the read set, the templates rule, the decision-link error"},
        {"path": "skills/bee-capturing/SKILL.md", "substantive": "one paragraph citing the templates rule and the concept"}
      ],
      "key_links": [
        "overview.md and index.md describe the extended concept",
        "bee dev regen was run and the rendered trees are committed with the cell"
      ],
      "prohibitions": [
        "No new concept file",
        "No restating of the grade rule or the homes rule already in the concept",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

Read-set grammar (fixed here so kdf-1 and kdf-3 agree): a `## Read set`
heading; under it, `### Knowledge` with a pipe table whose header row starts
with `| Concept |` and whose first column is a repo-relative path beginning
`docs/knowledge/`; then `### Code` with a pipe table whose header row starts
with `| Path |`. The parser reads only the knowledge table's first column; the
code table is for readers. The marker `<!-- bee:no-read-set: <reason> -->`
anywhere in the file waives the check with its reason recorded.

## Test matrix

| Case | Cell | Pass when |
|---|---|---|
| Happy: CONTEXT.md with `## Read set`, both tables, every knowledge path on disk | kdf-1 | `bee gate --merge --approved true` approves |
| Edge: section present, one knowledge path missing on disk | kdf-1 | refusal names the missing path and the remedy; no state write |
| Edge: no section but `<!-- bee:no-read-set: legacy -->` | kdf-1 | approves |
| Error: no section, no marker | kdf-1 | refusal names `## Read set` and the marker; no state write |
| Error: CONTEXT.md absent on a standard lane | kdf-1 | the same refusal, naming the path it looked for; no state write |
| Regression: a `tiny` lane with a short CONTEXT.md and no section | kdf-1 | approves (lane exemption) |
| Regression: a plain `--name execution` approval never runs the check | kdf-1 | approval unchanged on a CONTEXT.md with no section |
| Happy: area concept with one decision | kdf-2 | no `missing_decision_link` row |
| Error: area concept with `decisions: []` | kdf-2 | one profile error row `missing_decision_link` whose message names `bee decisions log` and the concept path |
| Edge: pattern concept with no decisions | kdf-2 | no row (patterns are not area concepts) |
| Host: bundle where no area concept links a decision | kdf-2 | check OK with one NOTE, no `missing_decision_link` row |
| Edge: `## Read set` present with an empty knowledge table (no-bundle host) | kdf-1 | approves |
| Main vs head: `bee knowledge check` on bee's bundle | kdf-2 | main: 0 errors; head: 0 errors after the five are linked |
| Parity: skill trees after regen | kdf-3, kdf-4 | `cargo test --test rule_index_parity --test pointer_integrity` green |

## Open Questions

(none)

## Out of scope

- The content cut of bee's own areas — PBI p-3a756890.
- Glossary and event catalogue — map tickets 003 and 004, waiting on the
  pull-rate trigger.
- Any change to `brief.rs` or the dispatch prompt renderer (D5).
- A product direction document per host — on the map as fog.
