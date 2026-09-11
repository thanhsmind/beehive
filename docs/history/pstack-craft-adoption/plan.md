---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: pstack-craft-adoption

## Summary

Bring pstack's better craft into bee's own skills, plus two user shortcuts.
Users get `/bee-how` and `/bee-why`. Every bee user gets these improvements:
plans that show what each cell will produce, class playbooks that state a
method, safer handoffs, rubric-first design lanes, graded review evidence, and
principles that carry a check you can run. Cells, in order: pca-1 to pca-9 run
in parallel, then pca-10 regenerates and proves the result.

Mode: `standard`. Flags: public-contracts, multi-domain and cross-platform. No
flag is a hard-gate flag.
Why this is the least workflow that protects the work: every edit is doctrine
text in `skills/` or `.bee/expertise/`, and it ships to every host through the
normal sync. The Rust checks and the new principles wait for later slices.

## Requirements (from CONTEXT.md)

No CONTEXT.md exists. These decisions are locked:

- D1 (decision `1e77b136`): ship the thin shortcut skills `bee-how` and
  `bee-why`. Each routes to Trace or Provenance sweep and never copies their
  steps.
- D2 (decision `1e77b136`): compare craft for every pstack skill, and move the
  better practices into bee's own skills. A feature overlap is never a reason
  to skip a practice.
- D3: the source is `docs/history/research/pstack-craft-study.md`. Its
  "Adoption backlog, ranked" is the scope. This plan builds Slice 1 of that
  backlog. Slices 2 and 3 are listed at the end as headlines.
- D4: each rule keeps one home. An adoption edits the section that already
  owns its rule. New detail goes into `references/` or an expertise guide,
  never into a second copy.

## Load-bearing claims

Labels: `read` means I opened the file at the anchor. `ran` means I ran the command
and kept its output. No row is `guessed`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | bee-planning forbids cells before the gate, but its hat wave edits drafted cells before the gate | read | `skills/bee-planning/SKILL.md:163` and `skills/bee-planning/references/planning-reference.md:533-534` | "No cells, no prep artifacts, before the gate is approved." / "applied directly to the cells (legal — cells are mutable before the gate)" |
| 2 | The CLI refuses to add cells before the execution gate | ran | `bee cells add --file cells-ship.json --dry-run` (during pstack-skills-ship) | "D3: no cells before the gate" |
| 3 | A principle names its sibling without the load-bearing prefix | read | `skills/bee-principle-reproduce-first/SKILL.md:17` | "**Not the same as `principle-red-before-green`.**" |
| 4 | The worker commit trailer is spelled with a capital C where the contract wants lowercase | read | `skills/bee-swarming/references/worker-details.md:182` | `git commit -m "<Imperative summary matching the cap outcome>" -m "Cell: <cell-id>"` |
| 5 | `bee finish` is a real verb, so it is not a defect | ran | `bee --help --names` | "bee finish — Flow spelling of `bee cells finish`" |
| 6 | The claims-table parser exists and can be reused by a future plan check | ran | `rg -n 'Load-bearing claims' packages/bee-rs/crates/bee/src` | `verbs/state_group/plan_claims.rs:40: pub(crate) const CLAIMS_HEADING: &str = "## Load-bearing claims";` |
| 7 | A new `###` heading under Class playbooks must be a class value, so every playbook adoption adds steps inside an existing class | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:16-19` | "2. Every `### ` section under that heading names a real class value." |
| 8 | Only the `bee-` prefix ships a skill, so `bee-how` and `bee-why` need no Rust change | read | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:448` | `if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {` |
| 9 | Cells that do not run regen can go in parallel with a wave-barrier ack, and the orchestrator owes one regen at wave close | read | `packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs:219` | "the recognized value \"wave-barrier\" defers the regen to the orchestrator, which owes the full regen chain once at wave close" |

## Discovery

Seven parallel craft lanes read pstack and bee side by side, one file at a
time. The findings, the verified defects and the ranked backlog are in
`docs/history/research/pstack-craft-study.md`. The leader re-read each defect
at its line before this plan cites it (claims 1 to 5). Claim 5 shows that one
reported defect was false.

## Approach

**Cell rules.** Every cell edits the section that already owns the rule it
adopts (D4). It translates the pstack idea into bee's terms, and it never
pastes pstack text. It keeps bee's one-line step form, and it never renames or
re-levels a cited heading. Every cell sets `regen_obligation_ack:
"wave-barrier"`, and pca-10 runs the one regen.

**TDD for skills.** The two new skills get full RED and GREEN runs, with four
scenarios run before any text ships (scenarios H1 to H4 in the study). The
edits to existing skills are a named deviation from the Iron Law. They carry
practices that pstack already uses in the field, and the study verified each
one at file:line. Their proof is structural: `pointer_integrity`,
`class_playbook_parity`, `rule_index_parity` and `instruction_laws`, plus a
leader read of every changed section. A decision records this deviation before
the gate.

**Waves:** pca-1 to pca-9 form one parallel wave. They share no files. pca-10
runs after them, alone, because it regenerates the shared release manifest.

**Rejected alternatives:**
- Build all three slices at once. That is too big to prove honestly, and the
  Rust checks need their own reality touch.
- Drop the doctrine edits and ship only the shortcuts. That breaks D2.
- Pressure-test every edited skill. That means about fifteen RED and GREEN
  pairs for text that the study already verified line by line. The structural
  tests plus the leader read cover the risk.

**Risk map:**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| shortcut skills | MEDIUM: a wrong trigger fires a sweep | pca-1 | RED and GREEN H1 to H4 |
| plan template and class playbooks | MEDIUM: every plan and every class reads them | pca-3 | `class_playbook_parity` and `pointer_integrity` green; no heading renamed |
| swarming handoff and triage | MEDIUM: a wrong rule can lose work | pca-5 | leader read against the study rows; suite green |
| principles | LOW: short additive lines | pca-8 | `principle_index_parity` green; the prefix fix resolves |
| doctrine size | MEDIUM: more always-read text | every cell | new depth goes into `references/`; no rule restated |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pca-1 | Ship bee-how and bee-why, and route Trace and Provenance questions straight to the reply | new `skills/bee-how/`, new `skills/bee-why/`, `skills/bee-researching/SKILL.md`, `skills/bee-teach/SKILL.md` | — | `/bee-how` and `/bee-why` appear in the skill list, and bee-researching answers a Trace or Provenance question in the reply with no brief file | GREEN H1 to H4; frontmatter and link check |
| pca-2 | Anchor, return-shape and self-check upgrades to Trace and Provenance sweep | `skills/bee-researching/references/trace-and-provenance.md` | — | Provenance sweep opens with an Anchor step; Trace names a worker return shape; the answer shape ends with a self-check | `pointer_integrity` green |
| pca-3 | Plan skeleton (Summary, cells preview, Waves, Pass when, Lands in) and class-playbook methods; fix the cells-before-gate conflict | `skills/bee-planning/references/planning-reference.md`, `skills/bee-planning/SKILL.md` | — | the template shows `## Summary` and `## Cells — current slice (preview)`; `### bugfix` states hypothesis elimination; the hat wave reviews the preview table | `class_playbook_parity` and `pointer_integrity` green |
| pca-4 | Two-line bug-fix proof and labelled live proof | `.bee/expertise/tests.md`, `skills/bee-verifying/SKILL.md` | — | tests.md asks for red-before and green-after lines; bee-verifying's proof line names `<feature>@<entry-point>` and an evidence path | `pointer_integrity` green |
| pca-5 | Safe handoff, in-flight triage, stuck, taxonomy, stop line, pilot, CI-red triage; fix the trailer and duplicated forms | `skills/bee-swarming/SKILL.md`, `skills/bee-swarming/references/swarming-reference.md`, `skills/bee-swarming/references/worker-details.md`, `skills/bee-hive/references/go-mode.md` | — | the handoff schema lists `verified`, `tree` and `gotchas`; worker-details writes `cell:`; the Result form has one home | `pointer_integrity` green; `rg -n 'Cell: <cell-id>'` finds nothing |
| pca-6 | Rubric first and base-then-graft in blind lanes; Dismissed list in the hat-wave digest | `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-planning/references/design-sketch.md` | — | Blind lanes opens with a 5-line spine and a move 0 that writes `## Rubric`; Converge names Base, Grafts and "diverge → reframe" | `pointer_integrity` green; no heading renamed |
| pca-7 | Evidence ladder, lead filter, structural smells and bounded reviewer retry | `.bee/expertise/review.md`, `skills/bee-reviewing/SKILL.md`, `skills/bee-reviewing/references/reviewing-reference.md` | — | review.md grades a claim on rungs 1-5; the wave synthesis runs a lead filter before severity | `pointer_integrity` green |
| pca-8 | Check, Not-when and evidence lines for the principles; fix the prefix | `skills/bee-principle-*/SKILL.md` | — | six principles show a `**Check:**` line; reproduce-first names `bee-principle-red-before-green` | `principle_index_parity` and `pointer_integrity` green |
| pca-9 | Catch-me-up reply shape, capture triage and harvest lenses, verify-upkeep landing fixes | `skills/bee-hive/references/routing-and-contracts.md`, `skills/bee-capturing/SKILL.md`, `skills/bee-verify-upkeep/SKILL.md` | — | the Communication contract defines a catch-me-up reply; verify-upkeep lands through a worktree, not a PR | `pointer_integrity` green |
| pca-10 | Wave-close regen, GREEN records and the declared suite | `skills/bee-how/CREATION-LOG.md`, `skills/bee-why/CREATION-LOG.md`, the five skill homes, `AGENTS.md`, the release manifest | pca-1 to pca-9 | `bee dev regen` is green; `bee-how` and `bee-why` exist in all five homes | `bee dev regen`, `bee dev release-manifest --check`, and the declared suite, all green |

## Test matrix

- **Happy path.** The GREEN runs of H1 to H4 pass. `/bee-how` states its reading
  of the target and traces it with anchors. `/bee-why` checks the guess in the
  question and keeps its empty rows. Pass when each GREEN reply meets its
  scenario's rule, and the CREATION-LOG quotes the reply verbatim.
- **Edge case.** `bee dev regen` is green, and the declared suite passes. Pass
  when `pointer_integrity`, `class_playbook_parity`, `principle_index_parity`,
  `rule_index_parity` and `instruction_laws` all report ok.
- **Error path.** None of these leader checks prints anything:
  - `rg -n 'Cell: <cell-id>' skills/`
  - `rg -n 'principle-red-before-green' skills/bee-principle-reproduce-first` (the name without the `bee-` prefix)
  - a check that `bee-how` and `bee-why` contain none of Trace's step text. Pass when a diff of the Trace steps against both skill bodies shows no shared step line.

## Open Questions

(none)

## Out of scope

- **Slice 2, Rust checks (headlines only):**
  - the `bee plan check` verb;
  - the upstream trace relay in `dispatch prepare`;
  - patch-id proof currency;
  - a `bee close` warning for dead proof pointers;
  - rolling refill, if `--limit` cannot already do it.
- **Slice 3, new principles (headlines only):** attack-the-premise,
  encode-lessons-in-structure, build-the-lever, migrate-callers-then-delete,
  subtract-before-you-add, boundary-discipline,
  make-illegal-states-unrepresentable, separate-before-serializing-shared-state.
- The release. It runs after the uat stop, on the user's word.
