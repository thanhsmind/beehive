---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: path-rigor-floor

## Summary

Today the agent alone decides how careful a change must be. With this
change, bee knows that some paths need more care. A database migration, a
SQL schema file or a release workflow sets a minimum lane. `bee cells add`
and `bee cells update` refuse a cell that touches such a path below that
lane, and say how to raise the route. For migrations, SQL and release
workflows the agent can no longer auto-approve the plan gate, at any
autopilot level: the human approves it. A repo can add its own rows in
`.bee/config.json`; a repo row can raise a floor but never lower it.

Mode: `standard` — 3 risk flags: public-contracts, multi-domain,
covered-contract-change
Why this is the least workflow that protects the work: a new config key, a
new refusal on cells add/update and a new gate refusal are public
contracts; no hard-gate flag applies.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`4bf3a432`): path globs map to a minimum lane and hard-gate flags;
  built-in rows plus `.bee/config.json` rows that raise, never lower.
- D2 (`f613e3c2`): `bee cells add` and cell update refuse a cell whose files
  exceed the feature route; the remedy is to raise the route.
- D3 (`f9ba8d88`): owner-first rows (migrations, SQL schema, release and
  publish workflows) make the merged gate human-only at every `gate_bypass`
  level.
- D4 (`8258ae2d`): the request size rule is out of scope; it is backlog item
  `p-8b417003`.

## Load-bearing claims

Labels are `read` (file opened at that line); evidence is the exact line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The lane ladder rank covers only tiny, small and standard, so the floor needs its own full rank | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:407 | `pub(crate) fn triage_ladder_rank(lane: &str) -> Option<u8> {` |
| 2 | The hard-gate route flags are one named list | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:389 | `pub(crate) const HARD_GATE_ROUTE_FLAGS: [&str; 6] = [` |
| 3 | cells add runs its path obligation check at one seam | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:446 | `match assert_regen_obligation(map, "addCell") {` |
| 4 | cells update runs the same check at one seam before it writes | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:654 | `assert_regen_obligation(&merged, "updateCell")?;` |
| 5 | The existing path obligation matches by prefix, not by glob | read | packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs:190 | `if let Some(matched) = guard.roots.iter().find(` |
| 6 | A glob matcher already exists in the crate, so no new dependency is needed | read | packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:87 | `pub(crate) fn glob_match(toks: &[GlobTok], input: &[char]) -> bool {` |
| 7 | Config is an untyped map read in one place, so a new key needs a small reader | read | packages/bee-rs/crates/bee/src/state.rs:161 | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 8 | The gate already refuses an auto actor for one gate before any write; the owner-first refusal copies it | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:833 | `if actor == "auto" && name == "uat" {` |
| 9 | Before execution approval no cell exists, so the gate must read the preview packet for files | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:980 | `if let Some(prev) = state.get("gate_preview").cloned() {` |
| 10 | A cells verb can read the feature's route lane through one reader | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:583 | `pub(crate) fn feature_route(root: &Path, feature: &str) -> D<Option<String>> {` |

## Discovery

A read-only gather walked the route vocabulary, the path obligation seam,
the glob helpers, the config reader and the gate bypass path (anchors
above). Finding: the seams exist; the gap is a floor table, a full lane
rank, and a gate check that reads files from the preview packet.

Known limit: the gate learns "agent" from `--actor auto`. An agent that
passes `--actor user` is not stopped by code; the doctrine (never approve a
gate yourself) still holds. This feature does not change who can pass
`--actor`.

## Approach

Recommended path, per contract decisions `a653682b` (D1, D2) and
`49388de3` (D3): one module holds the rows (built-in plus config), the full
lane rank, the glob match and the floor lookup for a list of files. The cells
add and update seams call it beside the regen obligation. The gate calls it
for owner-first hits over the preview packet and the persisted cells.

Rejected: checking at `bee route --set` — the route stores only a file
count, not paths (D2 rationale); a config row that lowers a floor (D1).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Existing cells in this repo meet a new refusal | MEDIUM | prf-1 | built-in rows touch only migrations, SQL and release/publish workflows; the full cells suite stays green |
| Gate refusal blocks a legitimate auto approval | LOW | prf-2 | unit tests: auto approve refused only on an owner-first hit; user approve unchanged |
| Glob semantics of `**` | LOW | prf-1 | unit tests per built-in row |

Waves: prf-1 and prf-3 run in parallel (disjoint files). prf-2 waits for
prf-1 because it calls the floor lookup prf-1 adds.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"test","reason":"Rerouted from code: herding agent paseo-pi-agy-flash is out of quota; decision ff5427df (role-reroute)."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"Each cell writes its tests red-first."},
    {"stage":"documentation-and-capture","classification":"required","role":"test","reason":"prf-3 changes skill text; rerouted from docs for the same quota reason."},
    {"stage":"read-only-gather","classification":"required","role":"review","reason":"Gathers run on --kind gather --role review while read is out of quota."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane with more than five product files: the plan-step hat wave runs."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Out of quota; gathers use review."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."},
    {"stage":"herding-code","classification":"not-applicable","role":"code","reason":"Out of quota; rerouted to test."},
    {"stage":"herding-read","classification":"not-applicable","role":"read","reason":"Out of quota; rerouted to review."},
    {"stage":"herding-docs","classification":"not-applicable","role":"docs","reason":"Out of quota; rerouted to test."}
  ]
}
```

## Shape

Phase plan (one slice):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | floor table, cells add/update refusal, owner-first gate refusal, skill text | the whole ask fits one slice | add a small-lane cell on a migration path and see the refusal; auto-approve a gate with an owner-first cell and see the refusal | backlog p-8b417003 |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| prf-1 | Refuse a cell whose paths need a higher route | cells/rigor_floor.rs (new), cells/mod.rs, cells/validate.rs, cells/handlers_write.rs, cells/tests.rs | — | `bee cells add` on a migration path under a small route refuses with RIGOR_FLOOR and names the route to set | cells tests green |
| prf-2 | Keep owner-first gates with the human under autopilot | state_group/set_gate.rs | prf-1 | `bee gate --merge --actor auto` on a feature that touches a migration refuses with GATE_OWNER_FIRST | set_gate tests green |
| prf-3 | Name the rigor floor in the planning and bypass rules | skills/bee-planning/SKILL.md, skills/bee-hive/references/gates-and-delegation.md, release manifest | — | the route rules say paths set a floor; the bypass table says owner-first gates stop for the human | release-manifest check green |

```json
[
  {
    "id": "prf-1",
    "feature": "path-rigor-floor",
    "title": "Refuse a cell whose paths need a higher route",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["4bf3a432-d4c3-4c6a-9b59-416d5627b2b8", "f613e3c2-d396-42c3-8edd-86357e914253", "a653682b-521f-4312-921b-68c39a5cabf9"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/rigor_floor.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/validate.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/path-rigor-floor/CONTEXT.md",
      "docs/history/path-rigor-floor/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/state.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision a653682b (D1 4bf3a432, D2 f613e3c2) in a new module verbs/cells/rigor_floor.rs. Rows are {glob, min_lane, flags, owner_first}. Built-in rows: `**/migrations/**` and `**/*.sql` at min_lane high-risk, flags [data-model], owner_first true; `.github/workflows/release*` and `.github/workflows/publish*` at min_lane standard, no flags, owner_first true. Repo rows come from the `.bee/config.json` key `rigor_floor` (an array of the same shape), read through read_config_raw (state.rs `pub fn read_config_raw(root: &Path) -> Map<String, Value> {`); a malformed row is skipped with a warning on stderr. For a file, the effective floor is the highest min_lane, the union of flags and the OR of owner_first over every matching row, built-in and repo alike, so a repo row can raise and never lower. Lane rank: docs < tiny < small < standard < high-risk, spike ranks as standard (workflows.rs triage_ladder_rank covers only three lanes; do not change it). Flag names must be in the route flag vocabulary in workflows.rs. Reuse the crate's glob matcher (hooks/write_guard/paths.rs glob_tokens / glob_match, or verbs/knowledge/ownership.rs matches_owned) — whichever supports `**`; no new crate dependency. Export a lookup that takes the root and a list of files and returns the hits (path, row glob, floor), and an owner-first query, for prf-2. Wire the check: in validate.rs beside `match assert_regen_obligation(map, \"addCell\") {` (problems are collected there, not thrown) and in handlers_write.rs beside `assert_regen_obligation(&merged, \"updateCell\")?;` (thrown before write_cell). Read the feature route (lane and flags) from the lane record (verbs/workflow_store read_lane_display then the route key; drivers/close.rs feature_route reads only the lane). No recorded route: no check. Refuse with code RIGOR_FLOOR when a hit's min_lane ranks above the route lane or a hit flag is missing from the route flags; the message names the path, the row glob, the floor and the remedy: `bee route --set --class <class> --lane <floor lane> --flags <route flags plus missing>`. Tests red-first: add refuses a small-lane cell on db/migrations/001.sql under a small route; add passes under a high-risk route with data-model; a repo config row raises a floor; a repo row with a lower lane does not lower a built-in floor; update refuses the same way; a feature with no route is not checked; unmatched paths pass.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells",
    "must_haves": {
      "truths": [
        "cells add refuses with RIGOR_FLOOR a cell on a migration path when the route lane is below high-risk or lacks data-model",
        "cells update refuses the same cell the same way",
        "the refusal names the path, the row glob, the floor and a bee route --set remedy",
        "a .bee/config.json rigor_floor row raises a floor and never lowers a built-in one",
        "a feature with no recorded route and a cell with no matching path pass unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/rigor_floor.rs", "substantive": "rows, lane rank, glob lookup, owner-first query"}
      ],
      "key_links": [
        "validate.rs add path calls the floor check",
        "handlers_write.rs update path calls the floor check before write_cell"
      ],
      "prohibitions": [
        "Do not change triage_ladder_rank or the route verbs",
        "Do not add a crate dependency",
        "No code comments"
      ],
      "open_choices": ["spike ranks as standard", "no recorded route means no check"]
    },
    "behavior_change": true
  },
  {
    "id": "prf-2",
    "feature": "path-rigor-floor",
    "title": "Keep owner-first gates with the human under autopilot",
    "lane": "standard",
    "role": "test",
    "deps": ["prf-1"],
    "decisions": ["f9ba8d88-f0f3-45a6-bb14-02bfb8abf005", "49388de3-9a4f-4cf3-97f6-1dcff092bf29"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"
    ],
    "read_first": [
      "docs/history/path-rigor-floor/CONTEXT.md",
      "docs/history/path-rigor-floor/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/rigor_floor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement contract decision 49388de3 (D3 f9ba8d88) in set_gate.rs. When actor is auto (`let actor = flag_value(flags, \"actor\").unwrap_or_else(|| \"user\".to_string());`) and the call approves shape, execution or --merge, collect the feature's files from the lane record's gate_preview cell packet (the packet that becomes approved_cell_packet: `if let Some(prev) = state.get(\"gate_preview\").cloned() {`) and from the feature's persisted cells, ask prf-1's owner-first query, and on any hit refuse with code GATE_OWNER_FIRST before any lock or write, at every bypass level. The message names the path and the row glob and says the human approves this gate. Copy the shape of the uat refusal (`if actor == \"auto\" && name == \"uat\" {`), which refuses before any write; the feature is resolved later in run_gate, so read the record the way the existing preview peek does (peek_target_record) without taking the lock. An approval with actor user, a refusal (approved false), and an auto approval with no owner-first hit are unchanged. Tests red-first next to gate_refuses_uat_from_actor_auto_and_writes_nothing: auto merge refused with an owner-first file in the preview packet and nothing written; the same with actor user approves; auto merge with no owner-first file approves as before.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- set_gate gate",
    "must_haves": {
      "truths": [
        "bee gate --merge --approved true --actor auto refuses with GATE_OWNER_FIRST when the preview packet or persisted cells touch an owner-first path, and writes nothing",
        "the same approval with actor user succeeds",
        "an auto approval with no owner-first hit succeeds as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs", "substantive": "GATE_OWNER_FIRST refusal before any write"}
      ],
      "key_links": ["set_gate.rs calls the owner-first query from rigor_floor.rs"],
      "prohibitions": [
        "Do not change the uat refusal or the bypass-level checks",
        "No code comments"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "prf-3",
    "feature": "path-rigor-floor",
    "title": "Name the rigor floor in the planning and bypass rules",
    "lane": "standard",
    "role": "test",
    "deps": [],
    "decisions": ["4bf3a432-d4c3-4c6a-9b59-416d5627b2b8", "f613e3c2-d396-42c3-8edd-86357e914253", "f9ba8d88-f0f3-45a6-bb14-02bfb8abf005"],
    "files": [
      "skills/bee-planning/SKILL.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/path-rigor-floor/CONTEXT.md",
      "docs/history/path-rigor-floor/plan.md",
      "skills/bee-planning/SKILL.md"
    ],
    "affects_skills": ["skills/bee-planning/SKILL.md", "skills/bee-hive/references/gates-and-delegation.md"],
    "affects_specs": [],
    "action": "Doctrine for D1, D2 and D3. In skills/bee-planning/SKILL.md, section Route, add a short rule: paths set a floor — bee ships rows for migrations, SQL schema and release/publish workflows, a repo adds rows under rigor_floor in .bee/config.json (raise only), and bee cells add / update refuse a cell below its floor with RIGOR_FLOOR; the remedy is to raise the route, never to drop the file. In skills/bee-hive/references/gates-and-delegation.md, section Gate bypass mode, add one line: a feature whose cells touch an owner-first path keeps the merged gate with the human at every bypass level (GATE_OWNER_FIRST). Keep each addition short. Then run the regen chain (`bee dev regen`) so rendered skill trees and the release manifest follow, and run the manifest check.",
    "verify": ".bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the planning Route section says paths set a floor that can be raised, never lowered",
        "the gate bypass section says owner-first paths keep the merged gate with the human at every level"
      ],
      "artifacts": [
        {"path": "skills/bee-planning/SKILL.md", "substantive": "rigor floor rule in Route"}
      ],
      "key_links": ["gates-and-delegation.md names GATE_OWNER_FIRST in Gate bypass mode"],
      "prohibitions": ["Do not restate the row table in a second home"]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Row | Case | Pass when |
|---|---|---|
| Happy | migration cell under a high-risk route with data-model | Pass when cells add succeeds |
| Happy | auto merge gate, no owner-first file | Pass when the gate approves as on main |
| Edge | repo row lower than a built-in row | Pass when the built-in floor still refuses |
| Edge | feature with no route; unmatched paths | Pass when cells add succeeds unchanged |
| Error | migration cell under a small route | Pass when cells add and update refuse with RIGOR_FLOOR naming the remedy |
| Error | auto merge gate with an owner-first file in the preview | Pass when the gate refuses GATE_OWNER_FIRST and state is unchanged |

## Open Questions

(none)

## Out of scope

- The request size rule (D4) — backlog item `p-8b417003`.
- Proving who ran `bee gate` beyond the `--actor` value.
