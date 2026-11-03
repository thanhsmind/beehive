# leader-check-door — plan

Route: class `feature` · lane `standard` · flags `public-contracts`,
`covered-contract-change`, `multi-domain` · product files 7.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

Revision 2 — rewritten after the plan-step hat wave. What the wave
changed is listed under `## Hat wave` below.

## Summary

The leader completeness check becomes a recorded mark with two doors. A
new verb, `bee cells leader-check --id <cell> --verdict ok|gap --file
<answers.json>`, makes the leader answer every derived requirement of a
capped cell with an artifact, and VERIFIES those artifacts exist rather
than checking that a string was typed. `bee close` and `bee worktree
merge` both refuse a feature whose capped cells carry no `ok` mark. The
shape is `dissent-debt`'s, copied deliberately: one debt function in
`verbs/cells/`, reached by both exits, with one named deferral escape
that lifts both.

Today the check is prose only, and the nearest machine door,
`judge-debt`, exists only for `standard`/`high-risk` routes and only for
`behavior_change` cells.

## Discovery

One reality touch per novel surface, all taken before this plan:

- `build_close_report_doors` read at `close.rs:1495` — the door push
  sequence, and the lane gate at `close.rs:1565`.
- `mistakes_debt` read at `close.rs:263` — the debt-loop template.
- `judge_debt` predicate read at `close.rs:459-473` — the cutoff
  constant and the fail-open grandfather arm this feature copies.
- The close refusal arms read at `close.rs:2550` and `close.rs:2655` —
  `Out::Emit(result, lines.join("\n"), 1)`, three lines: headline,
  `remedy:`, `next:`.
- The merge refusal arm read at `phases.rs:252-271` —
  `refuse_merge("WORKTREE_MERGE_DISSENT_DEBT", …)` inside the
  zero-mutation zone, guarded by `if let Some(feature) = …`.
- `feature_dissent_debt` + `has_dissent_deferral_decision` read at
  `dissent.rs:507` and `:536`.
- A real cell's `must_haves` read from
  `.bee/cells/archive/dispatch-cell-id-flag/dcif-1.json:56`.
- `run_judge_record` read at `handlers_meta.rs:169-292` — the `--file`
  payload path and the reopen site D8 deliberately does NOT copy.
- The flag vocabulary counted directly out of
  `src/generated/registry_payload.json`: 210 distinct names, and every
  flag this verb needs is already among them.

## Load-bearing claims

Paths are repo-relative. `B` abbreviates `packages/bee-rs/crates/bee/src`
only in the prose below the table; every Anchor cell carries the full
path.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The leader completeness check has no code enforcement today | read | `docs/history/leader-completeness-check/CONTEXT.md:22-26` | `## Scope` / `Instructional completeness check only:` / `- No Rust or schema changes` (three lines, quoted as three) |
| 2 | `judge-debt` never exists below `standard` | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1565` | `if matches!(feature_route(root, feature)?.as_deref(), Some("standard") \| Some("high-risk"))` gates the door's existence |
| 3 | `bee worktree merge` does NOT evaluate close's door list | read | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:211` | merge runs only `feature_proof_check` (`:211`), `feature_dissent_debt` (`:252`), `feature_advisor_nudge_debt` (`:293`), `uat_merge_precheck` (`:488`); all 18 `refuse_merge` sites swept, no other cell-debt reader |
| 4 | One debt fn can serve both doors | read | `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs:507` | `pub(crate) fn feature_dissent_debt(` — called unchanged at `close.rs:1654` and `phases.rs:252` |
| 5 | `registry_payload.json` is hand-edited here and has no regen chain | read | `docs/decisions/index.md:2013` | "…is hand-edited in this repo, with the reason recorded, because its declared regen chain does not exist here. Every cell that adds or changes a CLI command edits the payload directly and re-runs tests/registry_contracts.rs plus tests/registry_dispatch.rs as the proof." (decision 3358743e) |
| 6 | `catalog.rs` does NOT need touching — the pin counts distinct flag NAMES, and this verb introduces none | ran | `packages/bee-rs/crates/bee/src/catalog.rs:800-806` | `const PINNED_FLAG_COUNT: usize = 210;` then `let names: BTreeSet<&str> = entries().iter().flat_map(\|e\| e.properties.keys())`. Counted the payload: 210 distinct names; `id`, `verdict`, `file`, `session-id`, `force-ownership`, `json` all already present; `checked`/`gap` would be new — so the `--file` spelling adds none. A bump would turn the test red |
| 7 | `must_haves.truths` is required only at `standard` AND `high-risk` | read | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:319-327` | `if lane == "standard" \|\| lane == "high-risk"` → `addCell: lane \"{lane}\" requires non-empty must_haves.truths (observable truths to verify).` |
| 8 | A cap writes `trace.capped_at` | read | `.bee/cells/archive/dispatch-cell-id-flag/dcif-1.json:56` | `"capped_at": "2026-09-20T23:17:15.480Z"` |
| 9 | A new `cells` verb needs a `util.rs` match arm | read | `packages/bee-rs/crates/bee/src/verbs/cells/util.rs:88` | `"judge-record" => run_judge_record(flags, use_json, t0),` |
| 10 | `trace.leader_check` collides with nothing | read | `packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs:77` | `DISSENT_TRACE_KEY` plus every `trace.insert` site and `close.rs:270`'s `mistakes`: 34 keys in use (`attempts`…`worker`); `leader_check` is unused |
| 11 | Under the default `uat_stop`, merge runs BEFORE close | read | `packages/bee-rs/crates/bee/src/uat.rs:44` | `uat_stop_config` returns `Some(UatStop::Close)` when both keys are absent; merge gates only `if uat_stop == UatStop::Merge` (`phases.rs:487`). This is why D6 ships both doors in one slice |
| 12 | Every debt reader filters `Some("capped")`, and no open-cell gate exists at either exit | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:460` | `for cell in list_cells_including_archive(root, feature, Some("capped"))?` — same at `:265` and `:329`; all 12 `CLOSE_*_PREFIX` (`:32-89`) and all 18 `WORKTREE_MERGE_*` codes swept, none covers unfinished cells. This is why D8 no longer reopens |
| 13 | The grandfather cutoff needs a named constant | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:443` | `pub(crate) const JUDGE_DOOR_INTRODUCED_AT: &str = "2026-08-11T00:00:00.000Z";` read at `:459` as `let cutoff = date_parse(Some(&Value::String(JUDGE_DOOR_INTRODUCED_AT.to_string())));` |
| 14 | `feature_route` is read by exactly one door, but the uat door lane-filters through a different function | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1761` | `crate::uat::uat_lane_mode(root, feature)` + `uat_gate_applies_to_lane`, while `feature_route` appears once in the file (`:1565`). The claim is "no other door reads `feature_route`", never "no other door assumes a lane" |
| 15 | An archived cell cannot be status-mutated | read | `packages/bee-rs/crates/bee/src/verbs/cells/util.rs:283-294` | `assert_not_archived` throws `cell_archived_error`; called at `handlers_meta.rs:214`; `bee cells unarchive` is the road (`util.rs:102`) |
| 16 | No generic door registry exists — every door is hand-written | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1503` | literal `doors.push(Door {` blocks at `:1503,1525,1537,1590,1666` and literal `return Err(refuse_merge(` arms at `phases.rs:216,252,293`; no trait, no table |
| 17 | `cells update` cannot carry the mark | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:374` | `"must_haves",` sits in the 15-field patch whitelist; `trace` is not in it, and an unknown key refuses the whole patch |

## Smaller path check

*Is there a cheaper shape that still honours every locked decision?*

Four cheaper shapes were pressed, three by the `hat-alternatives` seat:

1. **Reuse `bee cells judge-record`.** FAIL — it hard-validates
   `judge-verdict/1` (`handlers_meta.rs:195-200`) and stamps
   `model_independence` (`:202-207`). A leader recording its own
   comparison would have to fabricate a judge payload and would make
   `"same-model"` the normal reading, weakening a door that works.
2. **Ride an existing verb.** FAIL, each for its own reason — claim
   table rows and `CONTEXT.md` D2: `cells update` refuses an unknown
   `trace` patch; `cap`/`finish` are worker-run, so the checked party
   would write its own mark; `close` runs after merge.
3. **Close door only.** FAIL — claim 11. The branch lands on main first.
4. **Reuse a generic door mechanism.** FAIL — claim 16. There is none.

Two cheaper spellings WERE adopted and shrank this plan:

- `--file <payload.json>` instead of a bespoke inline `::` string.
  Deletes the separator parser, its tests, the `catalog.rs` edit and the
  pinned-count bump (claim 6). Product files 8 → 7.
- The handler moves into `leader_check.rs` instead of
  `handlers_meta.rs` (already 917 lines), which is what the cited
  `dissent.rs` precedent actually does.

## Hat wave

Three seats, plan-step, one feature. Decision logged (tag
`plan-hat-wave`). What each changed:

| Seat | Finding acted on | Change |
|---|---|---|
| facts-gaps | `--verdict gap` reopening a cell drops it out of every `Some("capped")` debt scan, clearing both doors — recording a gap becomes the cheapest way past the door | `CONTEXT.md` D8 superseded: `gap` records, never mutates status |
| facts-gaps | claim 6 overstated the pinned-count test | Claim rewritten and re-verified by counting; `catalog.rs` dropped from scope |
| facts-gaps | "ordered after `mistakes`" names two different orders, and the plan said which for neither | D6a names push order and refusal-arm order separately, each anchored |
| facts-gaps | the grandfather cutoff had no constant, no value, no home | D5 names `LEADER_CHECK_DOOR_INTRODUCED_AT`, its file, and the fail-open posture |
| facts-gaps | "nothing else assumes a lane" is false (the uat door lane-filters via `uat_lane_mode`) | Claim 14 narrowed to "no other door reads `feature_route`" |
| alternatives | slice 1 as drafted was "close door only" — the shape the plan itself rejected | Merge door moved into slice 1 |
| alternatives | `lcd-4` targeted `AGENTS.md` (which carries no such text) and missed `bee-hive/SKILL.md:48,112` | `lcd-4` retargeted; `AGENTS.md` dropped on one-fact-one-home |
| alternatives | "regen chain re-run" contradicts decision 3358743e | `lcd-4` names the real fallback instead |
| user-impact | a recorded line proves typing, not verification | D3 adds artifact existence + `files_changed` overlap checks |
| user-impact | `must_haves.truths` is optional below `standard`; the verb would either wall or rubber-stamp | D3a derives the requirement list, never empty, never a wall |
| user-impact | the reopen strands an open cell on already-merged main | subsumed by the D8 supersede |

Findings NOT acted on, named: the typing cost on a many-cell feature
(no batch form) is accepted as a known gap and goes to backlog — the
`--file` payload already removes most of it, and `judge-record` sets the
one-call-per-cell precedent.

## Slices

### Slice 1 — the verb and both doors

A walking skeleton with real teeth: a leader can record a verified
check, and BOTH exits refuse without one. Both doors ship together
because of claim 11.

- **lcd-1** — `verbs/cells/leader_check.rs` (new): `run_leader_check`,
  `feature_leader_check_debt`, `has_leader_check_deferral_decision`,
  `LEADER_CHECK_DOOR_INTRODUCED_AT`; module wiring in `mod.rs`;
  dispatch arm in `util.rs`; the `cells.leader-check` entry in
  `generated/registry_payload.json`.
  Tests: a payload missing an answer for a derived requirement refuses;
  an artifact path that does not exist refuses; a payload whose answers
  name no file from `trace.files_changed` refuses; `ok` appends to
  `trace.leader_check`; `gap` appends and leaves `status` at `capped`;
  an uncapped cell refuses; an archived cell refuses naming
  `bee cells unarchive`; D3a's three derivation branches each covered.
  Proof: the module tests plus `tests/registry_contracts.rs` and
  `tests/registry_dispatch.rs` (decision 3358743e names those two as
  the payload's proof).
- **lcd-2** — `verbs/drivers/close.rs`: the `leader-check` door in
  `build_close_report_doors` at D6a's push slot, its deferral escape,
  and the refusal arm at D6a's arm slot.
  Tests: blocks an unmarked capped cell at `tiny` AND at `standard`;
  clears once an `ok` mark is recorded; stays blocking on a `gap` mark;
  grandfathers a cap before the stamp; grandfathers a missing
  `capped_at`; clears on a logged `leader-check-deferral` decision;
  names `bee cells unarchive` first for an archived offender; does not
  mask the `mistakes` or `judge-debt` arms.
- **lcd-3** — `verbs/worktree/phases.rs`: the
  `WORKTREE_MERGE_LEADER_CHECK_DEBT` refusal, in the zero-mutation zone
  beside the dissent arm, reading the SAME debt function and the SAME
  escape. Tests beside the existing merge-door tests.
  Proof: `cargo test … -p bee worktree`.

`lcd-2` and `lcd-3` touch disjoint files and run concurrently once
`lcd-1` lands. `lcd-1` is their dependency — both call its functions.

### Slice 2 — the doctrine sync

- **lcd-4** — the rule's text stops describing an honour-system step
  and names the verb and the two doors:
  `skills/bee-hive/references/routing-and-contracts.md:172-179` (the
  rule's one home), `skills/bee-hive/SKILL.md:48` and `:112`,
  `skills/bee-swarming/SKILL.md:76-81`, and
  `skills/bee-swarming/references/swarming-reference.md:234-258` and
  `:713`.
  `AGENTS.md` is deliberately NOT edited: it carries no copy of this
  rule today (only an unrelated proof-reporting line at `:140`), and
  adding one would duplicate the home at
  `routing-and-contracts.md:172`.
  Environment-fact check per AGENTS.md § Judgment and deviation: the
  payload has no regen chain here (claim 5), so the fallback is the
  recorded one — hand-edit plus the two registry test files. The SKILL
  mirrors DO have a chain: `bee dev regen`.
  Proof: `bee dev release-manifest --check` plus the parity/pointer
  check over the regenerated skill copies.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| lcd-1 | Record a verified leader check on a capped cell | `verbs/cells/leader_check.rs` (new), `verbs/cells/mod.rs`, `verbs/cells/util.rs`, `generated/registry_payload.json` | — | `bee cells leader-check` accepts an answer that names a real changed file, and refuses one that names a file the cell never touched | module tests + `tests/registry_contracts.rs` + `tests/registry_dispatch.rs` green |
| lcd-2 | Refuse close on a capped cell with no leader check | `verbs/drivers/close.rs` | lcd-1 | `bee close` names the unchecked cell and stops, at `tiny` as well as `standard` | `cargo test … -p bee close` green, red-first on each new door test |
| lcd-3 | Refuse merge on a capped cell with no leader check | `verbs/worktree/phases.rs` | lcd-1 | `bee worktree merge` stops before the branch lands, naming the same cells and the same escape | `cargo test … -p bee worktree` green, red-first |

```json
[
  {
    "id": "lcd-4",
    "feature": "leader-check-door",
    "title": "Name the verb and both doors in the leader-completeness-check doctrine",
    "lane": "standard",
    "role": "docs",
    "deps": [
      "lcd-1",
      "lcd-2",
      "lcd-3"
    ],
    "decisions": [
      "a38dc4bd-a94e-4ae5-91e8-2bfb0bf6a0b1"
    ],
    "files": [
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-hive/SKILL.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "skills/bee-hive/references/routing-and-contracts.md",
      "docs/history/leader-check-door/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-hive/SKILL.md",
      "skills/bee-swarming/SKILL.md",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_specs": [],
    "action": "The leader completeness check now has teeth. Update its doctrine text so it stops reading as an honour-system step, at these four sites and nowhere else. (1) skills/bee-hive/references/routing-and-contracts.md:172-179, the rule's ONE home: keep the four existing bullets, and add that the check is RECORDED with `bee cells leader-check --id <cell> --verdict ok|gap --file <answers.json>`, that the payload answers every derived requirement with an artifact and the verb verifies those artifacts exist (an artifact that looks like a repo path must be on disk; at least one answer must name a path from the cell's trace.files_changed), that a `gap` verdict records without reopening the cell so the door stays red, and that BOTH `bee close` and `bee worktree merge` refuse a feature whose capped cells carry no `ok` mark, at every lane, grandfathered by LEADER_CHECK_DOOR_INTRODUCED_AT, with a logged decision tagged `leader-check-deferral` as the one named escape. (2) skills/bee-hive/SKILL.md:48 (the bee-swarming routing row) and :112 (the hard rule): each gains a short clause that the check is recorded and that both exits check it \u2014 pointers only, never a second copy of the rule. (3) skills/bee-swarming/SKILL.md:76-81, step 5: after 'Run the leader completeness check before accepting', name the recording verb and say the cap is not the end of the cell \u2014 the recorded mark is. (4) skills/bee-swarming/references/swarming-reference.md:234-258 (step 7's first bullet) and :713: same, one pointer line each. Then run the skill regen chain: `bee dev regen` (it renders the .claude/.agents/plugin skill mirrors). ONE-FACT-ONE-HOME: AGENTS.md is deliberately NOT edited \u2014 it carries no copy of this rule today (only an unrelated proof-reporting line at :140), and adding one would duplicate routing-and-contracts.md:172. ENVIRONMENT-FACT CHECK: the registry payload has no regen chain in this repo (decision 3358743e) \u2014 that is lcd-1's concern and nothing here touches it. Write every edited sentence through the bee-technical-writing standard. REGEN: this cell owns the regen chain for its own skill edits \u2014 run bee dev regen (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write, in that order) and commit the refreshed docs/history/codex-harness-hardening/release-manifest.json with the rest.",
    "must_haves": {
      "truths": [
        "routing-and-contracts.md names the verb, the artifact verification, the gap-does-not-reopen rule, both doors, the grandfather stamp and the leader-check-deferral escape",
        "bee-hive/SKILL.md:48 and :112 point at the recorded mark without restating the rule",
        "bee-swarming/SKILL.md step 5 names the recording verb",
        "swarming-reference.md step 7 and the handoff line at :713 name it too",
        "AGENTS.md is unchanged",
        "the rendered skill mirrors match their sources after bee dev regen",
        "docs/history/codex-harness-hardening/release-manifest.json is refreshed in the same commit and bee dev release-manifest --check is green"
      ],
      "artifacts": [
        {
          "path": "skills/bee-hive/references/routing-and-contracts.md",
          "substantive": "the Leader completeness check section, now naming the verb and both doors"
        },
        {
          "path": "skills/bee-swarming/SKILL.md",
          "substantive": "step 5's pointer to the recording verb"
        }
      ],
      "key_links": [
        "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs is the behaviour this text must match",
        "docs/history/leader-check-door/CONTEXT.md D3, D3a, D5, D6, D7, D8 are the locked decisions the text may not reinterpret"
      ],
      "prohibitions": [
        "No edit to AGENTS.md",
        "No second copy of the rule outside routing-and-contracts.md \u2014 the other three sites carry pointers only",
        "No change to any Rust source",
        "No new claim the code does not implement"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee skill && .bee/bin/bee dev release-manifest --check"
  }
]
```

`lcd-4` (the doctrine sync) is slice 2 and is not a cell yet.

## Test matrix

The triad, at its smallest demonstrating size. Each writer judges
existing coverage first and authors only the gap.

| Dimension | Case | Pass when |
|---|---|---|
| happy | `leader-check --verdict ok` with one answer per truth, artifact present in `trace.files_changed` | entry appended to `trace.leader_check`, exit 0 |
| happy | close on a feature whose every capped cell has an `ok` mark | door detail reads `clear`, close proceeds |
| edge | cell with empty `must_haves.truths` and two `trace.files_changed` entries | verb demands exactly two answers |
| edge | cell with empty `must_haves.truths` AND empty `trace.files_changed` | verb demands exactly one non-blank answer |
| edge | cell capped before `LEADER_CHECK_DOOR_INTRODUCED_AT` | not counted as debt |
| edge | cell with no `capped_at` at all | not counted as debt (fail-open, `close.rs:466-468`) |
| error | answer whose artifact path is not on disk | refused, exit non-zero, offending artifact named |
| error | answers that name no path from `trace.files_changed` | refused, exit non-zero |
| error | `--verdict gap` recorded, then close | cell status still `capped`, close still refuses |
| error | archived offender | refusal names `bee cells unarchive` before the record command |
| behavior-change | same close run on main vs head, feature with one unmarked capped cell | main: close proceeds. head: close stops at the leader-check door |
| behavior-change | same merge run on main vs head, same feature | main: merge proceeds. head: `WORKTREE_MERGE_LEADER_CHECK_DEBT` |
| regression | distinct flag-name count | still 210; `catalog.rs` untouched and its pin test green |

## Test scoping

`commands.test` is the declared suite and CI runs it on every push. Each
cell records a scoped proof line plus the reason. Red-first applies to
every door test: write the door test, watch it fail because the door
does not exist, then build it.

## Open questions

None. D1-D8 are locked in `CONTEXT.md`.
