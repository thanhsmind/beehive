---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: effort-at-the-door

## Summary

The dispatch door now picks how hard each helper thinks. It starts from the
role's configured effort and moves it one step by the kind of job and the
cell's lane: harder for high-risk cells, lighter for tiny cells, lookups and
advice. The leader never passes an effort; the hook corrects any it passes.
Every choice is written into the dispatch record with its reason, and
`bee team show` prints the ladder.

Mode: `standard` — 1 risk flag: public-contracts (the payload and economics
shape the hook enforces)
Why this is the least workflow that protects the work: one new function, three
existing readers, every change pinned by a unit test seen red first.

## Requirements (from CONTEXT.md)

- D1: the door applies effort; the leader passes none.
- D2: ladder on `EFFORT_LEVELS` — cell/high-risk +1, cell/tiny −1, cell/small|standard 0, gather −2, advisor −1, reviewer 0, clamped; no base → no effort, reason `no-base`.
- D3: sinks — claude `Agent` payload `effort` key; codex `reasoning_effort` and `-c model_reasoning_effort`; herding arm untouched.
- D4: the claude guard recomputes the ladder from the marker and REPAIRS a different or missing `effort` param.
- D5: economics gains `effort`, `effort_base`, `effort_step`, `effort_reason` after `enforcement`; envelope and dispatch log both carry them.
- D6: no fast mode — out of scope.
- D7: one ladder constant beside `EFFORT_LEVELS`; `bee team show` prints it; `dispatch prepare` names step and reason.
- D8: one new B-rule in `model-roles-and-escalation.md`.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = the command was run; no `guessed` row remains.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The effort scale is a closed five-value constant | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:27 | `pub(crate) const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];` |
| 2 | The resolve site has `kind` and the cell in hand | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2259 | `let (resolved, is_escalated) = if role == Some(ADVISOR_ROLE) \|\| (role.is_none() && kind == "advisor")` |
| 3 | The claude Agent payload carries model only, no effort | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2843-2845 | `if let Resolved::Model { model, .. } = &resolved {` / `payload.insert("model".into(), Value::String(model.clone()));` |
| 4 | Economics key order is frozen and ends at `enforcement` | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:416 | `out.insert("enforcement".into(), Value::String(enforcement.to_string()));` |
| 5 | The claude guard path exists and repairs a wrong model param | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1211, :1346-1350 | `fn evaluate_claude_dispatch(root: Option<&Path>, tool_input: &Value, models: &Map<String, Value>) -> Verdict {` / `return repair(` / `"param-tier-repaired",` |
| 6 | The guard already parses `[bee-cell:` and reads the cell file by id | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:254, :1762 | `} else if let Some(tail) = strip_prefix_ascii_ci(rest, "[bee-cell:") {` / `if let Some(v) = read(&cells_dir.join(format!("{id}.json"))) {` |
| 7 | `slot_for_kind` maps kind to slot in one place | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:172 | `pub(crate) fn slot_for_kind(kind: &str) -> Option<&'static str> {` |
| 8 | A test already pins `reasoning_effort` on the codex payload | read | packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:3765 | `assert_eq!(payload.get("reasoning_effort"), Some(&json!("high")));` |
| 9 | A test pins the economics mapping | read | packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:749 | `fn derive_economics_matches_the_honest_mapping() {` |
| 10 | `bee team show` lives in models_group.rs | read | packages/bee-rs/crates/bee/src/verbs/models_group.rs:327 | `let ctx = match g_prelude("team show", json, pre_json, t0)? {` |
| 11 | The Agent tool accepts an `effort` param on the same five values | read | Agent tool schema (this session's tool list), field `effort` | `"enum": ["low", "medium", "high", "xhigh", "max"]` |
| 12 | The concept never names effort today | ran | `rg -n effort docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` | (no output) |

## Discovery

One read-only gather (`.bee/mailbox/job-1792164578083-3091621-1/report-1.md`)
mapped the effort plumbing; the leader opened every anchor above. Finding: effort
reaches two codex sinks and no claude sink; `lane` reaches prepare only as a
knowledge budget; the guard compares effort only on the codex path; economics
never records effort. The herding arm has no per-run thinking flag (D3 gap).

## Approach

Recommended path: one function `effort_for_dispatch(base, kind, lane)` beside
`EFFORT_LEVELS` (D2, D7), applied once at the resolve site so every sink reads
the moved value (D3), stamped into economics (D5); the guard calls the same
function (D4); `team show` prints the constant (D7); the concept gains its rule
(D8). Class playbook: `skills/bee-planning/playbooks/feature.md` — step 1 (data
shape) is the ladder function's signature; step 2 (walking skeleton) is
door → payload → guard for one claude cell dispatch; step 4 (user-visible
path) is `bee dispatch prepare` text naming the step and `bee team show`'s
ladder line; step 5 (sync knowledge) is ead-3.

Rejected alternatives:
- Stamp the effort into the prompt marker and have the guard trust it — the guard would trust the leader's text; config is the authority (AO5).
- A per-role `effort_fixed` config key — not needed: `reviewer` 0 already pins verification.
- Carry effort to herding now — needs a new `bee herding run` flag; backlogged.

Risk map:
- Clamp and `no-base` edge cases / LOW / ead-1 / unit tests on every kind×lane cell.
- Guard repair loop with the Agent `effort` param / MEDIUM / ead-2 / in-file guard tests: absent → repaired, wrong → repaired, right → allow.
- Economics key order breaks a consumer / LOW / ead-1 / `derive_economics_matches_the_honest_mapping` extended.

Waves: wave 1 = ead-1. Wave 2 = ead-2 ∥ ead-3 (disjoint files; both need the ladder function from ead-1).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "gather done; report recorded"},
    {"stage": "plan-check", "classification": "required", "role": "hat-facts-gaps", "reason": "standard with more than five product files opens the three-seat wave"},
    {"stage": "plan-check-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "SMALLER PATH at plan altitude"},
    {"stage": "plan-check-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "what the prepare text and team show line read like"},
    {"stage": "plan-check-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-check-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "code", "classification": "required", "role": "code", "reason": "ead-1, ead-2, ead-3 Rust changes and tests"},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "the one concept edit rides the code cell ead-3"},
    {"stage": "test", "classification": "conditional", "role": "test", "condition": "a code worker cannot show the red run", "reason": "red-first author"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "goal-check smells or the 1-in-3 sample lands", "reason": "standard judge tier"},
    {"stage": "gather", "classification": "not-applicable", "role": "extraction", "reason": "gather done at research"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "fall-through only"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "hat seats carry the consult"},
    {"stage": "lanes", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lanes-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "lanes-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "the leader plans inline"}
  ]
}
```

## Shape

Standard, one slice, three cells, two waves.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ead-1 | Move each dispatch's effort by kind and lane at the door and record it | `…/drivers/models.rs`, `…/drivers/prepare.rs`, `…/drivers/guard.rs`, `…/drivers/tests.rs` | — | `bee dispatch prepare` for a high-risk cell prints `effort: high → xhigh (+1, cell/high-risk)` and the Agent payload carries `effort`; economics shows the four keys | `cargo test -p bee verbs::drivers` green, red first |
| ead-2 | Repair a hand-picked Agent effort to the door's value | `…/hooks/model_guard.rs` | ead-1 | an Agent call with a wrong or missing `effort` is repaired, named `effort-param-repaired`; a right one is allowed | `cargo test -p bee hooks::model_guard` green, red first |
| ead-3 | Print the ladder in team show and state the rule in the concept | `…/verbs/models_group.rs`, `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` | ead-1 | `bee team show` ends with one `effort ladder:` line; the concept has a B-rule for it | `cargo test -p bee verbs::models_group` green, red first; `rg` finds the B-rule |

```json
[
  {
    "id": "ead-1",
    "feature": "effort-at-the-door",
    "title": "Move each dispatch's effort by kind and lane at the door and record it",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["d6ec1958-6c5f-464f-aeb6-0774d866fb91", "b61a2784-6cd9-40f3-8321-79e0637fd163", "826497a9-095e-4095-a7f0-f993485c381b", "507dbdf8-1984-4873-a88f-349e1ba1bdc3"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/effort-at-the-door/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "behavior_change": true,
    "action": "Per effort-at-the-door D2, D3, D5, D7 and the contract decision d6ec1958. (1) LADDER, one home. In packages/bee-rs/crates/bee/src/verbs/drivers/models.rs, directly under `pub(crate) const EFFORT_LEVELS: [&str; 5] = [\"low\", \"medium\", \"high\", \"xhigh\", \"max\"];`, add a `pub(crate) const EFFORT_LADDER` table of (kind, lane-or-empty, step) rows — (\"cell\",\"high-risk\",1), (\"cell\",\"tiny\",-1), (\"cell\",\"small\",0), (\"cell\",\"standard\",0), (\"gather\",\"\",-2), (\"advisor\",\"\",-1), (\"reviewer\",\"\",0) — and one `pub(crate) fn effort_for_dispatch(base: Option<&str>, kind: &str, lane: Option<&str>) -> EffortChoice` returning a small struct `EffortChoice { effort: Option<String>, base: Option<String>, step: i64, reason: String }`. Rules: a base outside EFFORT_LEVELS or None → effort None, base as given (or None), step 0, reason \"no-base\"; otherwise index the base in EFFORT_LEVELS, add the row's step (an unknown kind or a cell with an unknown lane uses step 0 and reason \"unmapped:<kind>/<lane>\"), clamp to 0..=4, and reason \"<kind>/<lane or ->\" (for example \"cell/high-risk\", \"gather/-\"). (2) APPLY ONCE at the resolve site in packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs: right after `let (resolved, is_escalated) = …` (search that exact prefix near `ADVISOR_ROLE`), when `resolved` is `Resolved::Model { effort, .. }` or `Resolved::Native { effort, .. }`, compute `effort_for_dispatch(effort.as_deref(), kind, lane)` where `lane` is `vget(cell, \"lane\")` for kind cell and None otherwise, and REPLACE the resolved effort with the choice's effort so every existing sink (`codex_spawn_payload` reasoning_effort, the `-c model_reasoning_effort` CLI arm) reads the moved value with no further edit. Keep the escalated (ceiling / session-model) path untouched: no effort, as today. (3) CLAUDE SINK (D3): in the claude Agent arm (search `payload.insert(\"model\".into(), Value::String(model.clone()));`), also insert `effort` = the choice's effort when Some. (4) ECONOMICS (D5): extend `derive_economics` in packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs with one new parameter carrying the EffortChoice (Option, None on paths with no resolved effort such as cli/herding/session-model) and insert, AFTER `enforcement` and in this order: `effort` (string or null), `effort_base` (string or null), `effort_step` (integer), `effort_reason` (string). Update every caller (prepare.rs and the guard twin `derive_dispatch_economics` in hooks/model_guard.rs may call it — if the twin needs the parameter, pass None there and leave hooks/model_guard.rs otherwise untouched; it belongs to ead-2). (5) TEXT (D7): where `bee dispatch prepare` prints its human text (search `economics` rendering in prepare.rs), add one line `effort: <base> → <effort> (<step:+1|-1|0>, <reason>)` when a base exists, or `effort: none (no-base)` otherwise. (6) RED FIRST, required, in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs: (a) a table test over effort_for_dispatch covering every ladder row, both clamps (max +1 stays max; low -2 stays low), None base, and an unknown lane; (b) a prepare test: a claude `--kind cell` dispatch of a high-risk cell with role effort `high` yields payload effort `xhigh` and economics effort/effort_base/effort_step/effort_reason = xhigh/high/1/\"cell/high-risk\"; (c) a `--kind gather` dispatch on a codex runtime with effort `high` yields `-c model_reasoning_effort='low'` (clamped -2) — extend `codex_prepared_non_cell_gather_uses_read_only_sandbox_cli` or add beside it; (d) extend `derive_economics_matches_the_honest_mapping` with the four keys and their order. Run the tests before the implementation and record the red output; then implement and record green. Scope the run: `cargo test -p bee verbs::drivers`. No code comments (comment ratchet `.bee/bin/bee dev comment-baseline --check`).",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers` is green, AND the recorded red run of the new tests failed before the implementation; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean. `.bee/bin/bee dispatch prepare --runtime claude --kind gather --json` on this repo prints economics with the four effort keys.",
    "must_haves": {
      "truths": [
        "A high-risk cell dispatch on claude carries effort one step above the role's base; a gather carries two steps below; both clamp at the scale ends",
        "A role with no configured effort yields no effort key and reason no-base",
        "Economics carries effort, effort_base, effort_step, effort_reason after enforcement, in that order, in the envelope and the dispatch log",
        "The codex sinks read the moved value with no change to their own code"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs", "substantive": "EFFORT_LADDER table and effort_for_dispatch beside EFFORT_LEVELS"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "one application at the resolve site; effort key on the claude Agent payload; one text line"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs", "substantive": "derive_economics with the four keys"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "ladder table test, prepare tests, economics test, seen red then green"}
      ],
      "key_links": ["prepare.rs calls models::effort_for_dispatch exactly once per dispatch", "derive_economics callers all compile and pass the choice"],
      "prohibitions": ["No change to which model a role resolves to", "No edit to hooks/model_guard.rs beyond a compile-only caller fix", "No new config key", "No code comments"]
    }
  },
  {
    "id": "ead-2",
    "feature": "effort-at-the-door",
    "title": "Repair a hand-picked Agent effort to the door's value",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["ead-1"],
    "decisions": ["d6ec1958-6c5f-464f-aeb6-0774d866fb91", "85192add-1382-4497-830b-790597e19f7a"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs"
    ],
    "read_first": [
      "docs/history/effort-at-the-door/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "behavior_change": true,
    "action": "Per effort-at-the-door D4 and the contract decision d6ec1958. In packages/bee-rs/crates/bee/src/hooks/model_guard.rs, `evaluate_claude_dispatch` (search `fn evaluate_claude_dispatch`) reads `model` and `subagent_type` from the Agent tool input and, on a tier/param mismatch, returns `repair(\"param-tier-repaired\", …)` (AO5: config is the authority). Add the effort leg beside it: (1) read the optional `effort` param from tool_input; (2) derive the dispatch kind from the tier: the advisor role or any `hat-*` seat → \"advisor\"; the read/extraction roles → \"gather\"; the review role → \"reviewer\"; everything else → \"cell\"; (3) for kind cell, read the lane from the cell named by `[bee-cell: <id>]` through the cell-file read the guard already has (search `cells_dir.join(format!(\"{id}.json\"))`), None when no marker or no file; (4) call `crate::verbs::drivers::models::effort_for_dispatch(resolved effort, kind, lane)` — the ONE ladder home from ead-1 — and compare: expected Some(e) and param == e → allow as today; expected Some(e) and param missing or different → REPAIR the `effort` param to e with code `effort-param-repaired` and a note in the same shape as the model note (`[bee-tier: t] expects effort \"e\", dispatch carried \"p\" → \"e\" (D4: the door picks effort)`); expected None and param present → repair by REMOVING the param (same code); expected None and param absent → allow. The repair must compose with the existing model repair: when both are wrong, one repair verdict carries both corrected params. Extend `derive_dispatch_economics` (the guard twin, search its name) to log the four effort keys from the same choice. (5) RED FIRST, required, in the in-file `mod tests`: wrong effort repaired; missing effort repaired; right effort allowed; expected-none with a param present → param removed; model and effort both wrong → one repair with both. Run before the fix and record the red; then implement and record green. Scope: `cargo test -p bee hooks::model_guard`. No code comments.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::model_guard` is green, AND the recorded red run of the new tests failed before the fix; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean.",
    "must_haves": {
      "truths": [
        "An Agent call whose effort differs from the door's value is repaired to it, never denied",
        "An Agent call with the right effort is allowed unchanged",
        "The guard and the door compute effort from the same function"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/model_guard.rs", "substantive": "effort leg in evaluate_claude_dispatch, effort keys in derive_dispatch_economics, five new tests seen red then green"}
      ],
      "key_links": ["model_guard.rs calls models::effort_for_dispatch, no second ladder"],
      "prohibitions": ["No new deny code", "No change to the codex spawn effort checks", "No code comments"]
    }
  },
  {
    "id": "ead-3",
    "feature": "effort-at-the-door",
    "title": "Print the ladder in team show and state the rule in the concept",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["ead-1"],
    "decisions": ["d6ec1958-6c5f-464f-aeb6-0774d866fb91", "869dbfc8-eee2-45e0-9bf3-4cdfe4e47165", "3e6435bc-df1a-425c-9e36-0a3550beb25f"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/models_group.rs",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"
    ],
    "read_first": [
      "docs/history/effort-at-the-door/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/models_group.rs",
      "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "behavior_change": true,
    "action": "Per effort-at-the-door D7 and D8. (1) TEAM SHOW. In packages/bee-rs/crates/bee/src/verbs/models_group.rs, the `team show` handler (search `g_prelude(\"team show\"`) prints the role table; after the table print ONE line `effort ladder: cell/high-risk +1 · cell/tiny -1 · cell/small 0 · cell/standard 0 · gather -2 · advisor -1 · reviewer 0 (clamped to low…max; no base → none)` rendered FROM `crate::verbs::drivers::models::EFFORT_LADDER` and `EFFORT_LEVELS`, never typed by hand; in `--json` mode add a top-level `effort_ladder` array of {kind, lane, step} from the same constant. Keep the existing column order (the test `team_show_columns_order_on_claude_and_pi` pins it). RED FIRST in the in-file tests: one test asserting the text line and one asserting the JSON key, run before the change and recorded red, then green. (2) CONCEPT (D8). In docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md add one B-rule after the last B-rule, in the file's voice: the door moves effort by kind and lane on the ladder (name every row), the leader passes no effort, the guard recomputes the same function and repairs a different or missing Agent effort param, economics carries effort/effort_base/effort_step/effort_reason, and the herding arm carries no effort until `bee herding run` gains a per-run thinking flag (name that as an Open Gap too). Cite `effort-at-the-door D2, D4, D5, D7` and the pointers to models.rs, prepare.rs, guard.rs, model_guard.rs, models_group.rs in the Pointers section. Run `.bee/bin/bee knowledge index --check`. No code comments.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::models_group` is green, AND the recorded red run of the two new tests failed before the change. `.bee/bin/bee team show --runtime claude` prints a line starting `effort ladder:`. `rg -c 'effort-at-the-door D2' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` prints at least 1. `.bee/bin/bee knowledge index --check` is clean. `.bee/bin/bee dev comment-baseline --check` is clean.",
    "must_haves": {
      "truths": [
        "bee team show prints the ladder from the one constant, text and JSON",
        "The concept states the rule, the repair posture, and the herding gap"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/models_group.rs", "substantive": "effort ladder line and JSON key rendered from EFFORT_LADDER; two tests seen red then green"},
        {"path": "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md", "substantive": "one new B-rule citing effort-at-the-door D2, D4, D5, D7 with pointers"}
      ],
      "key_links": ["models_group.rs reads models::EFFORT_LADDER, no copy"],
      "prohibitions": ["No change to the existing team show columns", "No code comments"]
    }
  }
]
```

## Test matrix

- ead-1 happy: role effort `high`, kind cell, lane high-risk — Pass when payload effort is `xhigh` and economics reads xhigh/high/1/"cell/high-risk".
- ead-1 edges: base `max` +1 — Pass when `max`; base `low`, gather −2 — Pass when `low`; base None — Pass when no effort key and reason `no-base`; unknown lane — Pass when step 0 and reason `unmapped:cell/<lane>`.
- ead-1 codex: gather with base `high` — Pass when `-c model_reasoning_effort='low'`.
- ead-2: wrong/missing/right/expected-none/both-wrong — Pass when repaired/repaired/allowed/removed/one repair carrying both.
- ead-3: `bee team show --runtime claude` — Pass when the last line starts `effort ladder:`; `--json` — Pass when `effort_ladder` has 7 rows.
- Same scenario main vs head: on main `bee dispatch prepare --kind gather --json` economics has no `effort` key; on head it has four.

## Open Questions

- Hat-wave findings land here before the gate (bypass `full`: recorded, recommended option proceeds).

## Out of scope

- Fast mode by urgency (D6).
- A `bee herding run --thinking` flag (backlog at close).
- Any change to role → model resolution.
