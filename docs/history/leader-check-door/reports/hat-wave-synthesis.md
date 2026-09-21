# leader-check-door — plan-step hat wave synthesis

Three seats, one feature, opened at the plan step over
`docs/history/leader-check-door/plan.md` revision 1. Decision logged
with tag `plan-hat-wave`. Synthesis is the leader's; the seats advised.

This synthesis is the plan check AND the Gate 2 advisor consult.

## Verdict

The core design survives every pressure applied: a new verb, a new
trace key, one debt function, two doors, one named escape. Four cheaper
shapes were pressed and all four fail against the code. Two cheaper
SPELLINGS were adopted and shrank the plan by one product file.

Seven blockers were raised. All seven are resolved in revision 2; none
required widening the scope.

## Blockers and their resolutions

**B1 — `--verdict gap` was an escape hatch, not teeth.** (facts-gaps;
corroborated from the other side by user-impact's Probe E.) Every debt
reader filters `Some("capped")` (`close.rs:265,329,460`) and neither
exit has an open-cell gate (all 12 `CLOSE_*_PREFIX`, all 18
`WORKTREE_MERGE_*` codes swept). A `capped` → `open` reopen would drop
the cell out of the debt scan and clear both doors, making a recorded
gap the cheapest way past the door.
→ **D8 superseded.** `gap` appends and never mutates status; the door
counts a capped cell as debt unless its newest entry reads `ok`. One
decision also kills three further blockers: the five-required-effects
reopen (`release_trace`, `clear_merge_ready_for`, …), the archived-cell
undefined behaviour (`util.rs:283-294`), and the open-cell-stranded-on-
main collision.

**B2 — `must_haves.truths` is optional below `standard`.**
(`validate.rs:319` requires it only at `standard`/`high-risk`.) The
verb would either accept an empty mark — restoring the tick-without-
looking defect at exactly the lanes the user asked about — or refuse
forever, making the door a permanent wall.
→ **D3a.** The requirement list is derived in order: `must_haves.truths`
when non-empty; else one per `trace.files_changed` entry; else exactly
one free requirement. Never zero, never a wall.

**B3 — "ordered after `mistakes`" named two different orders.** Push
order and refusal-arm order differ in `close.rs`, and revision 1 said
which for neither.
→ **D6a** names both, each anchored: push after `dissent-debt`
(`:1671`), arm last among the cell-debt arms, after `mistakes`
(`:2655`) and before uat (`:2685`), so the specific debts surface first.

**B4 — the grandfather cutoff had no constant, no value, no home.**
`judge_debt` reads `JUDGE_DOOR_INTRODUCED_AT` (`close.rs:443,459`);
`capped_at` alone is not a cutoff.
→ **D5** names `LEADER_CHECK_DOOR_INTRODUCED_AT`, puts it beside the
debt function, and copies the fail-open posture at `close.rs:466-468`.
It also records that the `mistakes_debt` legacy `trace.report` skip is
deliberately NOT copied.

**B5 — claim "a new flag forces a pinned-count bump" was false.** The
pin counts distinct flag NAMES (`catalog.rs:802-806`), not flags.
→ Re-verified by counting the payload: 210 names, and `id`, `verdict`,
`file`, `session-id`, `force-ownership`, `json` are all already among
them. Adopting `--file` (the `judge-record` shape) means zero new
names. **`catalog.rs` drops out of scope; the bespoke `::` parser and
its tests drop with it.** Product files 8 → 7.

**B6 — slice 1 was "close door only", the shape the plan itself
rejected.** Under the default `uat_stop: "close"` (`src/uat.rs:44`),
merge runs before close.
→ The merge door moved into slice 1. Slice 1 now ships the verb and
both doors; `lcd-2` and `lcd-3` touch disjoint files and run
concurrently after `lcd-1`.

**B7 — the doctrine cell targeted the wrong files.** `AGENTS.md`
carries no copy of this rule (only an unrelated line at `:140`), while
`skills/bee-hive/SKILL.md:48,112` does and was missing.
→ `lcd-4` retargeted; `AGENTS.md` dropped on one-fact-one-home, with
the reason recorded. Its "regen chain re-run" also contradicted
decision 3358743e, so the cell now names the recorded fallback.

## Findings not acted on, named

- **Typing cost on a many-cell feature.** No batch form ships. The
  `--file` payload already removes most of the typing, and
  `judge-record` sets the one-call-per-cell precedent. Recorded as a
  known gap for backlog rather than silently absorbed.
- **Self-check on an inline `tiny` cell.** The leader is also the
  worker there, so the check is not independent. Kept anyway (D4): the
  value is the forced re-read of one's own diff against the
  requirements, and the door is what makes that re-read non-optional.

## The one thing that changed the feature's character

The user-impact seat asked whether a recorded line proves verification
or only that someone typed. It only proves typing. **D3 now verifies
the artifacts**: an artifact that looks like a repo path must exist on
disk, and at least one answer per cell must name a path present in that
cell's `trace.files_changed`. That is the difference between a
checkbox and a check.
