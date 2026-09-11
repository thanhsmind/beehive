---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: pstack-craft-adoption

## Summary

Users can type `/bee-how` and `/bee-why` to ask how bee code runs and why it is built that way.
Every bee project gets clearer plans: each step says what the user will see and what proves it.
Bug fixes, handoffs, design choices, reviews and principles each gain a checkable step.
It all ships through the normal skill sync, and nothing in the Rust binary changes.

Mode: `standard`. Flags: public-contracts, multi-domain, cross-platform. No hard-gate flag.
Why this is the least workflow that protects the work: every edit is doctrine text in
`skills/` or `expertise/`, and each edit is one practice the study checked at file:line.
The Rust checks (Slice 2) and the new principles (Slice 3) wait.

## Requirements (from CONTEXT.md)

There is no CONTEXT.md. These decisions are locked:

- D1 (decision `1e77b136`): ship thin shortcut skills `bee-how` and `bee-why`. Each one
  routes to Trace or Provenance sweep and never copies their steps.
- D2 (decision `1e77b136`): run a craft comparison for every pstack skill, and adopt the
  better practices into bee's own skills. Having the same feature is never a reason to skip.
- D3: `docs/history/research/pstack-craft-study.md` "Adoption backlog, ranked" is the
  scope. This plan builds Slice 1, rows 1 to 10. Slices 2 and 3 are headlines at the end.
- D4: one home per rule. Each adoption edits the section that already owns its rule, and
  new detail goes into `references/` or an expertise guide, never into a second copy.
- D5 (decision `dc11c7f1`): the two new skills get RED and GREEN pressure tests. Edits to
  existing skills are a named deviation from the Iron Law. Their proof is structural.

## Load-bearing claims

Labels: `read` means I opened the file at the anchor. `ran` means I ran the command and kept
its output. No row is `guessed`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | bee-planning forbids cells before the gate, but its hat-wave text edits drafted cells before the gate | read | `skills/bee-planning/SKILL.md:164` and `skills/bee-planning/references/planning-reference.md:533-534` | "No cells, no prep artifacts, before the gate is approved." / "applied directly to the cells (legal — cells are mutable before the gate)" |
| 2 | The CLI refuses to add cells before the execution gate | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:153` | "D3: no cells before the gate" |
| 3 | A principle names its sibling without the load-bearing prefix | read | `skills/bee-principle-reproduce-first/SKILL.md:17` | "**Not the same as `principle-red-before-green`.**" |
| 4 | The worker commit example uses a capital-C trailer | read | `skills/bee-swarming/references/worker-details.md:182` | `-m "Cell: <cell-id>"` |
| 5 | `bee finish` is a real verb, not a defect | ran | `bee --help --names` | "bee finish — Flow spelling of `bee cells finish`" |
| 6 | The expertise guides have a source at `expertise/`, and `.bee/expertise/` holds exact copies that onboarding writes | ran | `cmp expertise/tests.md .bee/expertise/tests.md` and `cmp expertise/review.md .bee/expertise/review.md` | "tests.md: identical copy" / "review.md: identical copy" |
| 7 | Every level-three heading under Class playbooks must be a class value | read | `packages/bee-rs/crates/bee/tests/class_playbook_parity.rs:19` | "2. Every `### ` section under that heading names a real class value." |
| 8 | Only the `bee-` prefix ships a skill, so the shortcuts need no Rust change | read | `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:448` | `if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {` |
| 9 | Regen writes five skill homes | ran | `ls -d .claude/skills .agents/skills .opencode/skills .claude-plugin/skills .codex-plugin/skills` | all five paths listed |
| 10 | With a wave-barrier ack, parallel cells defer regen to one run at wave close | read | `packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs:218` | "the recognized value \"wave-barrier\" defers the regen to the orchestrator, which owes the full regen chain once at wave close" |
| 11 | The handoff writer keeps every input field, so new handoff fields need no Rust | read | `packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:599` (hat-alternatives spot check) | `write_handoff` copies every input key except `kind` |

## Discovery

Seven parallel craft lanes read pstack and bee side by side. Their findings, the verified
defects and the ranked backlog are in `docs/history/research/pstack-craft-study.md`.

A three-seat plan-step hat wave then checked this plan.
- **hat-facts-gaps** found one wrong anchor (claim 1, now line 164). It found a pass check
  that could never pass. It found two backlog items with no owning cell, and a cell that
  reached beyond the backlog. That backlog now has row 10.
- **hat-alternatives** found the shape close to minimal. Two of its cuts are taken: keep
  pca-5's detail out of the short SKILL.md, and let a text diff prove H4 in place of a model
  run.
- **hat-user-impact** raised five majors, and every one lands in a cell:
  - the gate message shows only the Summary and the "you see" lines;
  - "you see" is the effect the user sees;
  - the new plan skeleton and the bug-fix method have a lane scope;
  - the three explain skills each say when to use them;
  - no `wip:` commit survives a cap.

## Approach

**Cell rules.** Each cell edits the section that owns its rule (D4). It translates pstack's
idea and never pastes pstack text. It keeps bee's one-line step form and never renames or
re-levels a cited heading. It writes a qualified pointer without the `skills/` prefix. It
prefers words over a new numeric limit, and edits `expertise/`, never `.bee/expertise/` (claim 6).
Cells pca-1 to pca-9 set `regen_obligation_ack: "wave-barrier"`.

**Pressure tests (D5).** The RED runs H1 to H4 are done. All four pass on behavior, so the shortcut
bodies stay minimal. They are doors with a fixed answer shape. After the wave, the leader runs GREEN
for H1 to H3. pca-10 records GREEN in the two CREATION-LOGs, and it is the single home for that
record. H4 (a shortcut that copies Trace) is proven by the text diff in the test matrix.

**Waves:** pca-1 to pca-9 make one parallel wave, and no two of them write the same file. The
leader then runs GREEN. pca-10 runs alone after that, because it regenerates the shared release manifest.

**Rejected alternatives:**
- Build all three slices at once. That is too big to prove honestly, and the Rust checks need
  their own reality touch.
- Merge pca-2 into pca-1 so that GREEN tests the final Trace text. That is not needed, because
  GREEN runs after the whole wave.
- Defer the principle extras to Slice 3. Their cost is small, because principle bodies load
  only when a principle fires.
- Pressure-test every edited skill. That would take about fifteen pairs for text the study
  already verified.

**Risk map:**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| shortcut skills | MEDIUM: a wrong trigger fires a sweep, or overlaps bee-teach | pca-1 | GREEN H1 to H3; the three descriptions separate how, why and teach |
| plan template, gate message and class playbooks | MEDIUM: every plan reads them | pca-3 | `class_playbook_parity` and `pointer_integrity` green; no heading renamed |
| swarming handoff, `wip:` commits and triage | MEDIUM: a wrong rule can lose work or leave `wip:` in history | pca-5 | leader read against study row 5; suite green |
| expertise sources | MEDIUM: a copy edited by hand gets overwritten | pca-4, pca-7 | `cmp` of each source and copy after regen |
| principles | LOW: short additive lines | pca-8 | `principle_index_parity` green; the prefix fix resolves |

## Cells — current slice (preview)

The persisted cells are the authority. They are added only after the execution gate.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pca-1 | Ship `/bee-how` and `/bee-why`; answer Trace and Provenance questions in the reply; add the settled/refuted/inconclusive close | new `skills/bee-how/`, new `skills/bee-why/`, `skills/bee-researching/SKILL.md`, `skills/bee-teach/SKILL.md`, `README.md` | — | The user types `/bee-how` or `/bee-why` and gets a cited answer in the reply. The skill list says when each explain skill fits. | GREEN H1 to H3; frontmatter and link check |
| pca-2 | Anchor step, worker return shape and self-check for Trace and Provenance sweep | `skills/bee-researching/references/trace-and-provenance.md` | — | A why-answer opens with the question and its code range, and each gap names who would know. | `pointer_integrity` green |
| pca-3 | User-facing plan Summary and cells preview; gate shows Summary and "you see"; class-playbook methods; fix the cells-before-gate conflict | `skills/bee-planning/references/planning-reference.md`, `skills/bee-planning/SKILL.md` | — | A gate message opens with a short summary and one "you will see" line per step. A bug fix states how the cause was found. | `class_playbook_parity` and `pointer_integrity` green |
| pca-4 | A bug-fix proof carries a red line and a green line; live proof names feature, entry point and evidence | `expertise/tests.md`, `skills/bee-verifying/SKILL.md` | — | A finished bug fix shows the failing run, then the passing run. | `pointer_integrity` green |
| pca-5 | Safe pause, exact resume, stuck and failure handling; fix the trailer and the duplicated forms | `skills/bee-swarming/SKILL.md`, `skills/bee-swarming/references/swarming-reference.md`, `skills/bee-swarming/references/worker-details.md`, `skills/bee-hive/references/go-mode.md` (it owns the slice loop) | — | A paused session says what was proven and what is still open. Git history shows no `wip:` commits. | `pointer_integrity` green; the defect checks below |
| pca-6 | Rubric-first blind lanes, base and graft; the hat-wave digest lists what it dismissed | `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-planning/references/design-sketch.md` | — | A design choice cites the criteria it was judged on, written before the candidates ran. | `pointer_integrity` green |
| pca-7 | Evidence ladder with plain-word meanings, a lead filter, structural smells | `expertise/review.md`, `skills/bee-reviewing/SKILL.md`, `skills/bee-reviewing/references/reviewing-reference.md` | — | A review finding says how far its claim was proven, in words. | `pointer_integrity` green |
| pca-8 | Check and Not-when lines, the delegation clause, the vacuity check, sweep, second question and the prefix fix, across twelve named principles | twelve `skills/bee-principle-*/SKILL.md` (named in the cell) | — | When a principle fires, it names the yes/no check it ran. | `principle_index_parity` and `pointer_integrity` green |
| pca-9 | The catch-me-up reply shape; capture triage and harvest lenses; verify-upkeep lands through a worktree | `skills/bee-hive/references/routing-and-contracts.md`, `skills/bee-capturing/SKILL.md`, `skills/bee-verify-upkeep/SKILL.md` | — | "Catch me up" gets a short capsule, one status per feature in plain words, and one next step. | `pointer_integrity` green |
| pca-10 | Wave-close regen, GREEN records and the declared suite | two CREATION-LOGs, the five skill homes, `.bee/expertise/`, `AGENTS.md`, the release manifest | pca-1 to pca-9 | `bee-how` and `bee-why` are present in every runtime's skill list. | regen, `release-manifest --check` and the declared suite green |

## Test matrix

- **Happy path.** GREEN H1 to H3 pass with the skills loaded. `/bee-how` states its reading of
  the target, then traces it with anchors. `/bee-why` checks the guess in the question and keeps
  its empty rows. Pass when each GREEN reply meets its scenario's rule and the CREATION-LOG
  quotes it verbatim.
- **Edge case.** `bee dev regen` is green and the declared suite passes. Pass when
  `pointer_integrity`, `class_playbook_parity`, `principle_index_parity`, `rule_index_parity`
  and `instruction_laws` all report ok, and `cmp` shows each `expertise/` source equals its
  `.bee/expertise/` copy.
- **Error path.** Each of these leader checks prints nothing:
  - `rg -n 'Cell: <cell-id>' skills/`
  - `` rg -n '`principle-red-before-green' skills/bee-principle-reproduce-first `` (the unprefixed name, opened by a backtick)
  - `rg -n 'cells are mutable before the gate' skills/bee-planning`
  - H4: a comparison of each Trace step line against the bodies of `bee-how` and `bee-why`.
    Pass when no step line appears in either skill.

## Open Questions

(none)

## Out of scope

- **Slice 2, Rust checks** (headlines):
  - the `bee plan check` verb;
  - the upstream trace relay in `dispatch prepare`;
  - patch-id proof currency;
  - a `bee close` warning for dead proof pointers;
  - rolling refill, if `--limit` cannot already do it.
- **Slice 3, new principles** (headlines): attack-the-premise, encode-lessons-in-structure,
  build-the-lever, migrate-callers-then-delete, subtract-before-you-add, boundary-discipline,
  make-illegal-states-unrepresentable, separate-before-serializing-shared-state.
- **The release.** It runs after the uat stop, on the user's word.
