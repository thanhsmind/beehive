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

Nothing changes until a role carries `"effort"` in `.bee/config.json`
(`team.<runtime>.<role>: {"model": …, "effort": …}`). Today no role does, so
after this ships `bee team show` says the ladder is idle until one is set.

Mode: `standard` — 1 risk flag: public-contracts (the payload, marker and
economics shape the hook enforces)
Why this is the least workflow that protects the work: one new function, four
existing readers, every change pinned by a unit test seen red first.

## Requirements (from CONTEXT.md)

- D1: the door applies effort; the leader passes none.
- D2: ladder on `EFFORT_LEVELS` — cell/high-risk +1, cell/tiny −1, cell/small|standard 0, gather −2, advisor −1, reviewer 0, clamped; no base → no effort, reason `no-base`. (D10 adds cell/spike 0.)
- D3: sinks — claude `Agent` payload `effort` key; codex `reasoning_effort` and `-c model_reasoning_effort`; herding arm untouched.
- D4: the claude guard recomputes the ladder and REPAIRS a different or missing `effort` param. D4a: the codex guard leg compares to the ladder-moved value, deny posture unchanged.
- D5: economics gains `effort`, `effort_base`, `effort_step`, `effort_reason` after `enforcement`; envelope and dispatch log both carry them.
- D6: no fast mode — out of scope.
- D7: one ladder constant beside `EFFORT_LEVELS`; `bee team show` prints it; `dispatch prepare --explain` names step and reason.
- D8: one new B-rule in `model-roles-and-escalation.md`, naming the audit grep.
- D9: contract `effort-ladder` (`d6ec1958`) is what the tests check.
- D10: the door writes `[bee-kind:]` always and `[bee-lane:]` for cells; the guard reads kind and lane from the tokens; no `[bee-kind:]` → allowed unchanged.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = the command was run; no `guessed` row remains.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The effort scale is a closed five-value constant | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:27 | `pub(crate) const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];` |
| 2 | The resolve site has `kind` and the cell in hand | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2259 | `let (resolved, is_escalated) = if role == Some(ADVISOR_ROLE) \|\| (role.is_none() && kind == "advisor")` |
| 3 | The claude Agent payload carries model only, no effort | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2843-2844 | `if let Resolved::Model { model, .. } = &resolved {` / `payload.insert("model".into(), Value::String(model.clone()));` |
| 4 | Economics key order is frozen and ends at `enforcement` | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:416 | `out.insert("enforcement".into(), Value::String(enforcement.to_string()));` |
| 5 | The claude guard path exists | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1211 | `fn evaluate_claude_dispatch(root: Option<&Path>, tool_input: &Value, models: &Map<String, Value>) -> Verdict {` |
| 5b | The claude guard repairs a wrong model param rather than denying | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1346-1347 | `return repair(` / `"param-tier-repaired",` |
| 6 | The guard already parses `[bee-cell:` | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:254 | `} else if let Some(tail) = strip_prefix_ascii_ci(rest, "[bee-cell:") {` |
| 6b | The guard can read a whole cell from the store | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:820 | `let cell = crate::verbs::cells::read_cell(&main_root, cell_id)` |
| 7 | `slot_for_kind` maps kind to slot in one place | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:172 | `pub(crate) fn slot_for_kind(kind: &str) -> Option<&'static str> {` |
| 8 | A test already pins `reasoning_effort` on the codex payload | read | packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:3765 | `assert_eq!(payload.get("reasoning_effort"), Some(&json!("high")));` |
| 9 | A test pins the economics mapping | read | packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:749 | `fn derive_economics_matches_the_honest_mapping() {` |
| 10 | `bee team show` lives in models_group.rs | read | packages/bee-rs/crates/bee/src/verbs/models_group.rs:327 | `let ctx = match g_prelude("team show", json, pre_json, t0)? {` |
| 11 | The Agent tool accepts an `effort` param on the same five values | read | Agent tool schema (this session's tool list), field `effort` | `"enum": ["low", "medium", "high", "xhigh", "max"]` |
| 12 | The concept never names effort today | ran | `rg -n effort docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` | (no output) |
| 13 | The codex guard leg denies any effort different from the configured one | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1139 | `(Some(expected), Some(actual)) if actual != expected => {` |
| 14 | `[bee-cell:]` is emitted only under an approved v2 plan | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2522-2524 | `let extra_markers: Option<String> = v2_info.as_ref().map(\|v2\| {` / `if kind == "cell" {` |
| 15 | The guard has a role→kind helper (not to be used for kind, per D10) | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1201 | `fn dispatch_kind_for_role(role: &str) -> Option<&'static str> {` |
| 15b | The guard has a hand-rolled economics twin | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1615 | `fn derive_dispatch_economics(` |
| 16 | The twin's own order test exists | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:4058 | `let guard_econ = derive_dispatch_economics(&config, &models, false, &verdict).unwrap();` |
| 17 | `dispatch prepare` has no human text mode; `--explain` renders check rows | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3526 | `pub(crate) fn explain_checks(` |
| 18 | A stale doc line says the Agent interface has no effort parameter | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:479 | `/// * Claude's Agent interface has no effort parameter.` |
| 19 | Lanes are five values including spike | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:27 | `pub(crate) const LANES: [&str; 5] = ["tiny", "small", "standard", "high-risk", "spike"];` |
| 20 | No role sets effort in this repo today | ran | `rg -c '"effort"' .bee/config.json` | (exit 1, no match) |

## Discovery

One read-only gather (`.bee/mailbox/job-1792164578083-3091621-1/report-1.md`)
mapped the effort plumbing; the three-seat plan check found four input
disagreements between door and guard (codex deny, missing `[bee-cell:]`,
gather-vs-cell by role, `plan`/`lane-*` as advisor) and one wrong pointer (no
prepare text mode). D4a and D10 answer them; the pointer moved to `--explain`.
No role sets effort today, so the feature is idle until config sets one.

## Approach

Recommended path: one function `effort_for_dispatch(base, kind, lane)` beside
`EFFORT_LEVELS` (D2, D7), applied once at the resolve site so every sink reads
the moved value (D3), stamped into economics (D5) and into the marker (D10);
both guard legs call the same function (D4, D4a); `team show` prints the
constant and the idle state (D7); the concept gains its rule (D8). Class
playbook: `skills/bee-planning/playbooks/feature.md` — step 1 (data shape) is
`EffortChoice`; step 2 (walking skeleton) is door → marker → payload → guard
for one claude cell; step 4 (user-visible path) is `--explain`'s effort row and
`team show`'s ladder line; step 5 (sync knowledge) is ead-3.

Rejected alternatives:
- Guard trusts an effort value stamped in the prompt — the leader's text would steer the guard; config stays the authority, only kind and lane ride the marker.
- Guard derives kind from the tier alone — undecidable (gather vs cell, `plan` as advisor).
- Per-role `effort_fixed` key — not needed; `reviewer` 0 pins verification.
- Carry effort to herding now — needs a `bee herding run` flag; backlogged.
- Fold ead-3 into ead-1 — no wall-clock gain, a heavier brief.

Risk map:
- Clamp, `no-base`, spike / LOW / ead-1 / table test over every row.
- Guard repair on the Agent `effort` param composing with the model repair / MEDIUM / ead-2 / five guard tests plus `plan`/`lane-1` allow test.
- Codex leg deny after the ladder / MEDIUM / ead-2 / `codex_spawn_mismatched_effort_override_refused` kept green with the moved value.
- Economics order in two homes / LOW / ead-1 + ead-2 / `derive_economics_matches_the_honest_mapping` and the twin's test at model_guard.rs:4058.

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
    {"stage": "plan-check-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "what the explain row and team show line read like"},
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
| ead-1 | Move each dispatch's effort by kind and lane at the door and record it | `…/drivers/models.rs`, `…/drivers/prepare.rs`, `…/drivers/guard.rs`, `…/drivers/tests.rs` | — | once a role has an effort: `bee dispatch prepare --explain` shows `effort: pass — high → xhigh (+1, cell/high-risk)`, the Agent payload carries `effort`, the prompt carries `[bee-kind:]`/`[bee-lane:]`, economics shows the four keys | `cargo test -p bee verbs::drivers` green, red first |
| ead-2 | Make both guard legs read the ladder and repair a hand-picked Agent effort | `…/hooks/model_guard.rs` | ead-1 | an Agent call with a wrong or missing `effort` is repaired and the human sees the `bee-model-guard` note; a right one is allowed; a codex spawn with the ladder-moved effort is allowed | `cargo test -p bee hooks::model_guard` green, red first |
| ead-3 | Print the ladder and its idle state in team show and state the rule in the concept | `…/verbs/models_group.rs`, `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` | ead-1 | `bee team show` ends with `effort ladder: …` plus `— idle: no role sets effort` when none does, and shows `model/effort` per role; the concept has the B-rule and the audit grep | `cargo test -p bee verbs::models_group` green, red first; `rg` finds the B-rule |

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
    "decisions": ["d6ec1958-6c5f-464f-aeb6-0774d866fb91", "b61a2784-6cd9-40f3-8321-79e0637fd163", "826497a9-095e-4095-a7f0-f993485c381b", "507dbdf8-1984-4873-a88f-349e1ba1bdc3", "8233df26-b0ea-4988-8987-14a5859431bd"],
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
    "action": "Per effort-at-the-door D2, D3, D5, D7, D10 and the contract decision d6ec1958. (1) LADDER, one home. In packages/bee-rs/crates/bee/src/verbs/drivers/models.rs, directly under `pub(crate) const EFFORT_LEVELS: [&str; 5] = [\"low\", \"medium\", \"high\", \"xhigh\", \"max\"];`, add `pub(crate) const EFFORT_LADDER: [(&str, &str, i64); 8]` with rows (\"cell\",\"high-risk\",1), (\"cell\",\"tiny\",-1), (\"cell\",\"small\",0), (\"cell\",\"standard\",0), (\"cell\",\"spike\",0), (\"gather\",\"\",-2), (\"advisor\",\"\",-1), (\"reviewer\",\"\",0), a `pub(crate) struct EffortChoice { pub effort: Option<String>, pub base: Option<String>, pub step: i64, pub reason: String }`, and `pub(crate) fn effort_for_dispatch(base: Option<&str>, kind: &str, lane: Option<&str>) -> EffortChoice`. Rules: base None or not in EFFORT_LEVELS → effort None, base as given, step 0, reason \"no-base\"; else find the row by (kind, lane) for kind cell or by (kind, \"\") otherwise — no row → step 0, reason \"unmapped:<kind>/<lane or ->\"; add the step to the base index, clamp 0..=4, reason \"<kind>/<lane or ->\" (e.g. \"cell/high-risk\", \"gather/-\"). (2) APPLY ONCE in packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs right after `let (resolved, is_escalated) = …` (search that prefix near ADVISOR_ROLE): `lane` = for kind cell, `cell.as_ref().and_then(|c| vget(c, \"lane\")).and_then(Value::as_str)`, else None; when `resolved` is `Resolved::Model { effort, .. }` or `Resolved::Native { effort, .. }`, compute the choice and REPLACE the resolved effort with `choice.effort`, so `codex_spawn_payload` (`reasoning_effort`) and the `-c model_reasoning_effort` CLI arm read the moved value with no edit of their own. Keep `choice` for steps 3-5. The escalated (ceiling / session-model) path stays untouched. (3) MARKER AND CLAUDE SINK (D10, D3). Where the prompt's marker line is built (search `[bee-tier:` and `extra_markers`), append ` [bee-kind: <kind>]` on EVERY native payload, outside the v2-only `extra_markers` block, and ` [bee-lane: <lane>]` when kind is cell and the lane is known. In the claude Agent arm (search `payload.insert(\"model\".into(), Value::String(model.clone()));`) also insert `effort` = `choice.effort` when Some. (4) ECONOMICS (D5). Give `derive_economics` in packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs one more parameter `effort: Option<&EffortChoice>` and insert AFTER `enforcement`, in this order: `effort` (string or null), `effort_base` (string or null), `effort_step` (integer), `effort_reason` (string); when the parameter is None emit null/null/0/\"none\". Update every caller in guard.rs, prepare.rs and drivers/tests.rs (about 16 sites). hooks/model_guard.rs has its own hand-rolled twin `derive_dispatch_economics` and does NOT call this function — leave that file alone; ead-2 owns it. (5) EXPLAIN ROW (D7). `bee dispatch prepare` has no human text mode; add one row to `explain_checks` (search `pub(crate) fn explain_checks(`): name `effort`, result `pass`, detail `<base> → <effort> (<+1|-1|0>, <reason>)` or `none (no-base)`. (6) RED FIRST, required, in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs: (a) a table test over effort_for_dispatch covering all eight rows, both clamps (max +1 → max; low −2 → low), None base, an unknown lane; (b) a prepare test: claude `--kind cell` on a high-risk cell with role effort `high` → payload `effort` = `xhigh`, prompt contains `[bee-kind: cell]` and `[bee-lane: high-risk]`, economics effort/effort_base/effort_step/effort_reason = xhigh/high/1/\"cell/high-risk\"; (c) a `--kind gather` on codex with effort `high` → `-c model_reasoning_effort='low'` and prompt contains `[bee-kind: gather]` (extend `codex_prepared_non_cell_gather_uses_read_only_sandbox_cli` or add beside it); (d) extend `derive_economics_matches_the_honest_mapping` with the four keys in order and the None case. Run before implementing and record the red; implement; record the green. Scope: `cargo test -p bee verbs::drivers`. Build the binary once at the end (`cargo build --release --manifest-path packages/bee-rs/Cargo.toml`) and run `packages/bee-rs/target/release/bee dispatch prepare --runtime claude --kind advisor --role hat-value --stage plan-check-value --json` from the worktree to show the four economics keys and `[bee-kind: advisor]` in the prompt; never overwrite .bee/bin/bee. No code comments (comment ratchet `.bee/bin/bee dev comment-baseline --check`).",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers` is green, AND the recorded red run of the new tests failed before the implementation; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean. `packages/bee-rs/target/release/bee dispatch prepare --runtime claude --kind advisor --role hat-value --stage plan-check-value --json` prints economics with keys effort, effort_base, effort_step, effort_reason and a prompt containing `[bee-kind: advisor]`.",
    "must_haves": {
      "truths": [
        "A high-risk cell dispatch on claude carries effort one step above the role's base; a gather carries two steps below; both clamp at the scale ends",
        "A role with no configured effort yields no effort key and reason no-base",
        "Every native payload's prompt carries [bee-kind:], and a cell's carries [bee-lane:]",
        "Economics carries effort, effort_base, effort_step, effort_reason after enforcement, in that order, in the envelope and the dispatch log",
        "The codex sinks read the moved value with no change to their own code"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs", "substantive": "EFFORT_LADDER, EffortChoice and effort_for_dispatch beside EFFORT_LEVELS"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "one application at the resolve site; marker tokens; effort key on the claude Agent payload; one explain row"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs", "substantive": "derive_economics with the four keys"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "ladder table test, prepare tests, economics test, seen red then green"}
      ],
      "key_links": ["prepare.rs calls models::effort_for_dispatch exactly once per dispatch", "every derive_economics caller passes the choice"],
      "prohibitions": ["No change to which model a role resolves to", "No edit to hooks/model_guard.rs", "No new config key", "No write to .bee/bin/bee", "No code comments"]
    }
  },
  {
    "id": "ead-2",
    "feature": "effort-at-the-door",
    "title": "Make both guard legs read the ladder and repair a hand-picked Agent effort",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["ead-1"],
    "decisions": ["d6ec1958-6c5f-464f-aeb6-0774d866fb91", "85192add-1382-4497-830b-790597e19f7a", "5009d4f1-6578-48df-9f88-47b3d237b08c", "8233df26-b0ea-4988-8987-14a5859431bd"],
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
    "action": "Per effort-at-the-door D4, D4a, D10 and the contract decision d6ec1958, in packages/bee-rs/crates/bee/src/hooks/model_guard.rs only. INPUTS (D10): parse two new marker tokens beside the existing `[bee-tier:` / `[bee-cell:` parse (search `strip_prefix_ascii_ci(rest, \"[bee-cell:\")`): `[bee-kind: <kind>]` and `[bee-lane: <lane>]`. Kind comes from the token; when `[bee-cell: <id>]` names a cell the store can read (`crate::verbs::cells::read_cell(&main_root, cell_id)`, already used in this file — search it), the cell's `lane` wins over the token; otherwise the token's lane; a prompt with NO `[bee-kind:]` token takes the legacy path: no effort expectation, allowed unchanged. Never derive kind from the role name. CLAUDE LEG (D4): in `evaluate_claude_dispatch` (search `fn evaluate_claude_dispatch`) read the optional `effort` param from tool_input, compute `crate::verbs::drivers::models::effort_for_dispatch(configured effort of the resolved tier, kind, lane)` — the ONE ladder home — and compare: expected Some(e) and param == e → allow; expected Some(e) and param missing or different → REPAIR the `effort` param to e, code `effort-param-repaired`, note `[bee-tier: t] expects effort \"e\", dispatch carried \"p\" → \"e\" (D4: the door picks effort)`; expected None and param present → repair by removing the param (same code); expected None and absent → allow. The effort repair COMPOSES with the model repair: the model leg returns early today (`return repair(\"param-tier-repaired\", …)`), so restructure so one `Verdict` can carry both corrected params — give `Verdict` an optional `effort` field, extend `repair(...)` (three callers) accordingly, and make `repair_stdout` emit both in one `updatedInput`. CODEX LEG (D4a): in `evaluate_codex_spawn`, where `expected` is the configured effort (search `(Some(expected), Some(actual)) if actual != expected`), compute expected as `effort_for_dispatch(configured, kind, lane).effort` from the same tokens; keep the deny posture and codes exactly; keep `codex_spawn_mismatched_effort_override_refused` green by moving its fixture to a laddered value where needed. TWIN: `derive_dispatch_economics` (search `fn derive_dispatch_economics(`) is a hand-rolled copy that does not call `derive_economics`; add the same four keys after `enforcement`, in the same order, and extend the twin's own order test (search `derive_dispatch_economics(&config, &models, false, &verdict)`). Delete the stale doc line `/// * Claude's Agent interface has no effort parameter.` (the comment ratchet may fall, never rise). RED FIRST, required, in the in-file `mod tests`: (a) wrong effort repaired; (b) missing effort repaired; (c) right effort allowed; (d) expected-none with a param present → param removed; (e) model and effort both wrong → ONE repair carrying both; (f) `[bee-kind: advisor]` on tier `plan` and on `lane-1` with a base → the door's value allowed unchanged; (g) no `[bee-kind:]` token → allowed unchanged; (h) codex spawn with the ladder-moved `reasoning_effort` → allowed, with the unmoved base → denied. Run before the fix and record the red; implement; record green. Scope: `cargo test -p bee hooks::model_guard`. No code comments.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee hooks::model_guard` is green, AND the recorded red run of the new tests failed before the fix; both outputs in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean.",
    "must_haves": {
      "truths": [
        "An Agent call whose effort differs from the door's value is repaired to it, never denied, and the human sees the guard's note",
        "An Agent call with the right effort, including plan and lane-* tiers dispatched as advisor, is allowed unchanged",
        "A codex spawn carrying the ladder-moved reasoning_effort is allowed; one carrying a different value is still denied",
        "A prompt with no [bee-kind:] token is allowed unchanged",
        "The guard and the door compute effort from the same function on the same inputs"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/model_guard.rs", "substantive": "marker token parse, effort leg in evaluate_claude_dispatch composing with the model repair, laddered expected value in evaluate_codex_spawn, four keys in derive_dispatch_economics, eight new tests seen red then green"}
      ],
      "key_links": ["model_guard.rs calls models::effort_for_dispatch, no second ladder", "the twin's order test pins the four keys"],
      "prohibitions": ["No new deny code", "No change to the deny posture of the codex leg", "No edit outside hooks/model_guard.rs", "No code comments"]
    }
  },
  {
    "id": "ead-3",
    "feature": "effort-at-the-door",
    "title": "Print the ladder and its idle state in team show and state the rule in the concept",
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
    "action": "Per effort-at-the-door D7 and D8. (1) TEAM SHOW. In packages/bee-rs/crates/bee/src/verbs/models_group.rs, the `team show` handler (search `g_prelude(\"team show\"`) prints the role table. (a) In the existing model column print `<model>/<effort>` when the slot carries an effort (e.g. `opus/high`), else the model alone — no new column, so `team_show_columns_order_on_claude_and_pi` stays green. (b) After the table print ONE line rendered FROM `crate::verbs::drivers::models::EFFORT_LADDER` and `EFFORT_LEVELS`, never typed by hand: `effort ladder: cell/high-risk +1 · cell/tiny -1 · cell/small 0 · cell/standard 0 · cell/spike 0 · gather -2 · advisor -1 · reviewer 0 (clamped to low…max; no base → none)`; when no role on the runtime carries an effort, append ` — idle: no role sets effort; add \"effort\" to a team.<runtime>.<role> slot`. (c) In `--json` mode add top-level `effort_ladder` (array of {kind, lane, step} from the constant) and `effort_ladder_idle` (bool). RED FIRST in the in-file tests: one test for the text line with the idle suffix on a roster with no effort, one for the `model/effort` cell and no suffix on a roster with one, one for the JSON keys — run before the change and recorded red, then green. (2) CONCEPT (D8). In docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md add one B-rule after the last B-rule, in the file's voice: the door moves effort by kind and lane on the ladder (name every row), the leader passes no effort, the door writes `[bee-kind:]`/`[bee-lane:]` and the guard recomputes the same function from them with config as the base and repairs a different or missing Agent effort (codex leg denies as before), economics carries effort/effort_base/effort_step/effort_reason, the audit path is `rg '\"effort\":\"max\"' .bee/logs/dispatch.jsonl` (or any value), nothing moves until a role sets `effort`, and the herding arm carries no effort until `bee herding run` gains a per-run thinking flag — name that last one under Open Gaps too. Cite `effort-at-the-door D2, D4, D4a, D5, D7, D10` and add Pointers to models.rs, prepare.rs, guard.rs, model_guard.rs, models_group.rs. Run `.bee/bin/bee knowledge index --check`. Scope: `cargo test -p bee verbs::models_group`. No code comments.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::models_group` is green, AND the recorded red run of the three new tests failed before the change. `packages/bee-rs/target/release/bee team show --runtime claude` (built with `cargo build --release --manifest-path packages/bee-rs/Cargo.toml`) prints a line starting `effort ladder:` ending with the idle suffix. `rg -c 'effort-at-the-door D2' docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` prints at least 1. `.bee/bin/bee knowledge index --check` is clean. `.bee/bin/bee dev comment-baseline --check` is clean.",
    "must_haves": {
      "truths": [
        "bee team show prints the ladder from the one constant, text and JSON, and says when it is idle",
        "A role's configured effort is visible in team show beside its model",
        "The concept states the rule, the repair posture, the audit grep, the idle condition and the herding gap"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/models_group.rs", "substantive": "effort ladder line with idle suffix, model/effort cell, JSON keys, three tests seen red then green"},
        {"path": "docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md", "substantive": "one new B-rule citing effort-at-the-door D2, D4, D4a, D5, D7, D10 with pointers and an Open Gap"}
      ],
      "key_links": ["models_group.rs reads models::EFFORT_LADDER, no copy"],
      "prohibitions": ["No new team show column", "No write to .bee/bin/bee", "No code comments"]
    }
  }
]
```

## Test matrix

- ead-1 happy: role effort `high`, kind cell, lane high-risk — Pass when payload effort is `xhigh`, prompt has `[bee-kind: cell] [bee-lane: high-risk]`, economics reads xhigh/high/1/"cell/high-risk".
- ead-1 edges: base `max` +1 — Pass when `max`; base `low`, gather −2 — Pass when `low`; base None — Pass when no effort key and reason `no-base`; lane spike — Pass when step 0, reason `cell/spike`; unknown lane — Pass when step 0, reason `unmapped:cell/<lane>`.
- ead-1 codex: gather with base `high` — Pass when `-c model_reasoning_effort='low'`.
- ead-2: wrong/missing/right/expected-none/both-wrong/plan-as-advisor/no-token/codex-moved — Pass when repaired/repaired/allowed/removed/one repair with both/allowed/allowed/allowed-and-unmoved-denied.
- ead-3: `team show --runtime claude` on this repo — Pass when the last line starts `effort ladder:` and ends with the idle suffix; `--json` — Pass when `effort_ladder` has 8 rows and `effort_ladder_idle` is true.
- Same scenario main vs head: on main `dispatch prepare --kind advisor --json` economics has no `effort` key and the prompt no `[bee-kind:]`; on head it has both.

## Open Questions

- Hat-wave findings land here before the gate (bypass `full`: recorded, recommended option proceeds). All eight (three BLOCKER, five WARNING) are folded into D4a, D10 and the cell texts above; none remains open.

## Out of scope

- Fast mode by urgency (D6).
- A `bee herding run --thinking` flag (backlog at close).
- Setting an effort on any role in this repo's `.bee/config.json` — the user's call after merge.
- Any change to role → model resolution.
