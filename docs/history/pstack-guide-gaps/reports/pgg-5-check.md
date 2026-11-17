# pgg-5 check: pgg-1 and pgg-2 against the decisions

Checker: pgg-test-5, with no context shared with the pgg-1 or pgg-2 authors.
Commits checked: `411425195` (pgg-1) and `3e5a90c66` (pgg-2), base `a45947354`.
Decisions: D1 to D4 of `docs/history/pstack-guide-gaps/CONTEXT.md`.

Result: 25 rows, 20 pass and 5 findings. Two of the passing rows carry a
note. That makes 7 findings in total (F1 to F7). Two are exact sentence
copies from pstack.

## pgg-1 truths

| # | Truth | Result | Evidence |
|---|---|---|---|
| 1 | Both skills exist with name equal to directory, folded description, `disable-model-invocation: true`, metadata shaped like `bee-verifying` | pass | `skills/bee-formal-verifying/SKILL.md:1-15`, `skills/bee-formal-upkeep/SKILL.md:1-15`; same keys as `skills/bee-verifying/SKILL.md:1-15` |
| 2 | `bee-formal-verifying` records covered files, functions and a sync commit under `.bee/verify/<name>/`, and shows the check failing on a broken variant before handoff | pass | `skills/bee-formal-verifying/SKILL.md:48`, `:60`, `:68-77` |
| 3 | `bee-formal-upkeep` gives each check one of in sync, updated, bug, blocked, and edits only the verification | pass | `skills/bee-formal-upkeep/SKILL.md:23-28`, `:32-34` |
| 4 | `references/statechart.md` draws a Mermaid diagram from the TLA+ model and TLC trace in the reader's words and replays each bug path on real code | pass | `skills/bee-formal-verifying/references/statechart.md:17-19`, `:26-33` |
| 5 | `feature.md` steps 3 and 8 and `bug-fix.md` name the bee skills and the statechart reference, not the pstack names | pass, see F6 | `skills/bee-planning/playbooks/feature/feature.md:25`, `:79`, `:83`; `skills/bee-planning/playbooks/bugfix/bug-fix.md:61`. No pstack name is left in `skills/`. `bug-fix.md:63` names the formal verify skill only through `feature/feature.md` step 8, not by skill name |
| 6 | Both skills cite `.bee/verify/formal-cell-lifecycle` as the worked example | pass | `skills/bee-formal-verifying/SKILL.md:64`, `skills/bee-formal-upkeep/SKILL.md:38` |
| 7 | `skill-triggers.json` carries trigger briefs for both new skills | pass | `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json:126-134`, 4 rows each plus one near-miss row; `skill_triggers_fixture_is_complete` green |
| 8 | The rendered plugin trees carry both skills | pass | `SKILL.md` for both skills in `.claude-plugin/skills`, `.codex-plugin/skills`, `.agents/skills`, `.opencode/skills`, `.claude/skills` |
| 9 | Key link: `feature.md` step 8 points at skills that exist under `skills/` | pass | `feature.md:79` and `:83` name `bee-formal-verifying` and `bee-formal-upkeep`; both directories exist |
| 10 | Prohibition: not in `HUB_SERVED_SKILLS` (D6) | pass | `packages/bee-rs/crates/bee/src/onboard/skills.rs:81` lists 14 names, neither formal skill among them |
| 11 | Prohibition: no pstack TypeScript statechart pipeline | pass | `references/statechart.md:8` draws by hand, no script |

## pgg-2 truths

| # | Truth | Result | Evidence |
|---|---|---|---|
| 12 | `bee-principle-explain-the-number` has an Apply when description and Check, Not when, Why, Depth lines | pass | `skills/bee-principle-explain-the-number/SKILL.md:3`, `:18-27`; 27 lines, under the 30-line budget |
| 13 | `expertise/performance.md` § Measuring without lying to yourself holds the moves, seven checklist questions and the four verdicts | pass | `expertise/performance.md:204-218` (moves), `:220-258` (`### The benchmark checklist`, questions 1 to 7), `:259-263` (verdicts); `.bee/expertise/performance.md` is byte-identical |
| 14 | Principle homes has a row for the new principle with classes perf, research, feature, bugfix, release | pass | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:635-638` |
| 15 | `perf-issue.md` and `hillclimb.md` cite the section | pass | `skills/bee-planning/playbooks/perf/perf-issue.md:11-13`, `skills/bee-planning/playbooks/perf/hillclimb.md:31-33` |
| 16 | Every Principle homes row has a `signals:` line after its `classes:` line | pass, see F4 | 38 rows, 38 `signals:` lines, each the line after `classes:` |
| 17 | Key link: the new row's guide heading resolves | pass | `expertise/performance.md:183`; `every_row_anchors_an_existing_guide_heading` green |
| 18 | Prohibition: no separate benchmark-checklist skill | pass | no `skills/*benchmark*` directory |

## Pointers and verbatim text

| # | Check | Result | Evidence |
|---|---|---|---|
| 19 | Every path in the three new skill files resolves | finding F3 | All resolve except `.codex/skills/` at `skills/bee-formal-verifying/SKILL.md:66` |
| 20 | Every path and § heading in the four playbooks resolves | pass | Every `playbooks/...` path resolves under `skills/bee-planning/`; `.bee/expertise/performance.md` § Measuring without lying to yourself resolves at `:183` |
| 21 | Every TLA+ name the statechart reference uses as an example exists | pass | `ClaimFile`, `ClaimCellUnderLock`, `CloseCellUnderLock` at `.bee/verify/formal-cell-lifecycle/CellLifecycle.tla:50`, `:62`, `:88` |
| 22 | No exact sentence copied from pstack | finding F1 | two exact copies |
| 23 | No near-verbatim sentence | finding F2 | 11 sentences at 0.88 or more similarity |
| 24 | Knowledge that names the formal steps is current | finding F5 | stale paragraph in the router-triage concept |
| 25 | The statechart reference is not a skill (D2) | finding F7 | it carries skill frontmatter |

## Findings

**F1. Two sentences are copied from pstack verbatim.**
- `expertise/performance.md:226` (and `.bee/expertise/performance.md`):
  "a choice between options is never a ballpark." Source:
  `benchmark-checklist_SKILL.md:11`, "A choice between options is never a ballpark."
- `skills/bee-formal-upkeep/SKILL.md:28`: "the check could not be brought back
  in line or could not run." Source: `maintain-formal-verification-skill_SKILL.md:18`,
  the same words.

**F2. Eleven sentences change one or two words of a pstack sentence.**
A word-level sequence match of 0.88 or more against the sources at
`.bee/tmp/pstack-guide/up/`:
- `skills/bee-formal-upkeep/SKILL.md:25` in sync (0.93), `:27` bug (0.93),
  `:40` Run as-is (0.91), `:42` Read every change (0.93), `:32` Edit only the
  verification (0.86).
- `skills/bee-formal-verifying/SKILL.md:21` "A reader must be able to trust the
  result..." (0.91), `:43` Able to fail (0.89), `:46` Readable without the tool
  (0.86), `:23` and `:29` section headings (0.87).
- `expertise/performance.md:230` "Check the load generator too..." (0.93).
These are not exact copies, but the pgg-1 and pgg-2 action asked for "every
sentence in bee's own plain words". A further 14 sentences match between 0.75
and 0.85. They share the structure of their source but differ in their wording.

**F3. `skills/bee-formal-verifying/SKILL.md:66` names `.codex/skills/` as a home
that `bee onboard --verify-only --apply` renders into.** No `.codex/skills/`
exists in this repo, and `skills/bee-verifying/SKILL.md:32-33` names only
`.claude/skills/`, `.agents/skills/` and `.opencode/skills/`. The worked example
`formal-cell-lifecycle` is rendered only into those three. The homes come from
`REPO_SKILL_TARGETS` (`packages/bee-rs/crates/bee/src/onboard/plan.rs:641`).
Fix: name the same three homes `bee-verifying` names, or point at
`bee-verifying` for the list.

**F4. Nine `signals:` tokens are neither a route lane nor a route flag nor a
glob, which D4 requires.** The lanes are `ROUTE_LANE_VALUES` and the flags are
`ROUTE_FLAG_VALUES` in `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:357`
and `:368`.
- Class names used as tokens: `bugfix` (router-triage concept `:356`),
  `release` (`:377`, `:412`, `:447`, `:585`), `research` (`:398`, `:610`),
  `perf` (`:638`).
- `external` (`:506`) is not a flag. The flag is `external-systems`.
Under D4 as written, these tokens never match, so they do not affect the
ranking. The `perf` token on the new row came from the pgg-2 action's own
example, so the plan shares the cause. pgg-3 must either match class names too
(a D4 change) or these tokens must be replaced.

**F5. `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:129-134`
is stale.** It says the feature playbook's steps 6 and 7 name `statechart`,
`create-formal-verification-skill` and `maintain-formal-verification-skill`,
"all three live in the skillstation hub" and "bee vendors none of them". After
pgg-1, steps 3 and 8 name `bee-formal-verifying`, `bee-formal-upkeep` and
`skills/bee-formal-verifying/references/statechart.md`, and bee ships them.
The pgg-1 knowledge sync updated `verify-pipeline/overview.md` but not this
paragraph.

**F6. The step 3 fallback in `skills/bee-planning/playbooks/feature/feature.md:31-33`
still says "With no such skill on the host".** Step 3 now names a reference
file, not a skill, so the sentence should name the reference. `feature.md:29`
already says "when the host has that reference". This is minor.

**F7. `skills/bee-formal-verifying/references/statechart.md:1-4` carries skill
frontmatter (`name: statechart`, `description`).** D2 says the statechart is a
reference, "not a skill". No other `skills/*/references/*.md` file has
frontmatter. A loader that scans for `name:` frontmatter could list it as a
skill named `statechart`, which is the pstack name the feature retires. This is
minor.

## Method

- Truths: each file opened on disk at `3e5a90c66`. Each `file:line` above is
  from that tree.
- Pointers: every backticked path and markdown link in the 3 new skill files
  and the 4 playbooks was resolved against the repo root, the file's own
  directory and `skills/bee-planning/`. Paths relative to the worked example
  were checked in `.bee/verify/formal-cell-lifecycle/`.
- Verbatim: every sentence of 6 words or more in the new skill files and in
  the lines pgg-1 and pgg-2 added to the playbooks, the performance guide and
  the verify-pipeline overview was normalized to lowercase letters and digits,
  then compared with every source sentence by exact match and by
  `difflib.SequenceMatcher` ratio.

## Verify

The cargo half is green: `principle_index_parity` 5 passed, `skill_triggers`
2 passed and 1 ignored (the real-agent eval). `bee knowledge check` exits
FAIL on one profile error, `missing_decision_link` in
`docs/knowledge/areas/decision-memory/decision-memory-the-leader-triages-proposed-pbis-the-owner-gets-only-owner-only-calls.md`.
That file was last changed in `df30cfc21`, which predates both pgg-1 and pgg-2,
so the error was not caused by pgg-1 or pgg-2.
