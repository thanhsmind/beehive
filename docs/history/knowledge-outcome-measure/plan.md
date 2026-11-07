# knowledge-outcome-measure — Plan

**Lane:** standard · **Class:** feature · **Date:** 2026-11-07

## Summary

Find out whether the knowledge bundle helps cells finish well. Dispatch puts
10% of cells, chosen by a fixed hash of the cell id, into a holdout group whose
brief has no "Learned context" block. The dispatch record names each cell's
group. `bee knowledge report` then compares the two groups by how often their
cells got blocked, reopened, went red, or recorded mistakes.

## Problem

Every cell brief carries up to 8 knowledge paths, always with at least 3
critical patterns, so no cell runs without knowledge and nothing compares the
two. The only outcome measure, the `bee.signature` recurrence count, reads 0
for every pattern because no signature string is ever written into the
decision or capture logs (CONTEXT.md D1).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | One function builds the Learned context lines for a cell brief | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1075 | `pub(crate) fn learned_context_lines(root: &Path, cell: &Value) -> D<Vec<String>>` |
| 2 | The cell prompt render calls it once and passes the result as `learned_context` | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1293 | `let learned = learned_context_lines(bundle_root, cell)?.join("\n");` |
| 3 | The worker prompt renders the block only when the value is non-empty | read | packages/bee/prompts/worker-cell.md:39-43 | `{{#if learned_context}} ... Learned context (machine-assembled ...` |
| 4 | Herding and Agent-tool cell dispatches use the same rendered prompt | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1312-1314 | `let herding = if is_herding { "1" } else { "" };` in the same render call |
| 5 | The prepare-time dispatch record is one map built and appended in one place | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3211-3248 | `let mut record = Map::new(); ... append_prepare_record(root, &record);` |
| 6 | A sha256 hex helper exists for reuse | read | packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs:89 | `pub(crate) fn sha256_hex(s: &str) -> String` |
| 7 | Config is read raw from `.bee/config.json` | read | packages/bee-rs/crates/bee/src/state.rs:161 | `pub fn read_config_raw(root: &Path) -> Map<String, Value>` |
| 8 | The report verb builds measured, unmeasured and the evidence ladder | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:253 | `pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Value, Vec<String>)` |
| 9 | Cell traces hold the outcome signals | ran | .bee/cells/**/*.json (1278 cells) | trace keys: `blocked_reason` 50, `reopened_for_rework` 59 (object with `at`), `attempts` 37 (entries carry `verdict`, e.g. `tests-red`), `mistakes` 299 (non-empty arrays) |
| 10 | Most cell dispatches since October go through herding | ran | .bee/logs/dispatch.jsonl | kind cell since 2026-10-01: herding-exec 290, claude-agent 112, session-model 5 |

## Decisions

- D1 — Outcome measure, not signature counts (CONTEXT.md D1, decision `eeb666c4`).
- D2 — Deterministic 10% holdout, recorded arm, config switch (CONTEXT.md D2, decision `dafd05a1`).
- D3 — The arm is a pure function of the cell id, so a re-dispatch of the same
  cell stays in the same arm and the groups do not mix.
- D4 — The holdout removes only the Learned context block. The static read
  list (AGENTS.md, CONTEXT.md, plan.md) and the dispatcher-picked Expertise
  block stay in both arms. A worker may still search knowledge on its own;
  the comparison is by assigned arm (intent to treat).
- D5 — Cells with no recorded arm (every cell dispatched before this ships)
  are counted as `unrecorded` and never assigned an arm after the fact.

## Contract — cell kom-1 (holdout arm in dispatch prepare)

- Arm function: `holdout` when the first 8 hex digits of `sha256_hex(cell_id)`,
  read as a u32, modulo 100, is below the holdout percent; else `knowledge`.
- Holdout percent: `.bee/config.json` key `knowledge_holdout_percent`, read
  from the same root prepare reads its config from. Absent → 10. An integer
  0..=100 → that value. Any other value → 10. 0 means no holdout.
- Kind `cell`, arm `holdout`: the rendered prompt carries no "Learned context"
  block. Arm `knowledge`: the prompt is byte-identical to today.
- Kind `cell` prepare record gains `knowledge_arm` (`"knowledge"` |
  `"holdout"`) and `learned_context` (array of the repo-relative paths that
  the rendered block lists; `[]` for holdout or an empty block).
- Records of kind gather, reviewer and advisor stay byte-identical.
- Tests first, seen red: holdout cell prompt has no Learned context block;
  knowledge cell prompt unchanged; percent 0 never holds out; absent percent
  uses 10; an invalid value uses 10; the same cell id gives the same arm twice;
  the cell record carries both keys; a gather record carries neither.

## Contract — cell kom-2 (outcomes section in bee knowledge report)

- `bee knowledge report` keeps `measured`, `unmeasured` and the evidence
  ladder unchanged and gains `outcomes`:
  `{arms: {knowledge: Arm, holdout: Arm}, unrecorded: n, min_per_arm: 30, comparable: bool}`.
- Arm: `{cells, blocked, reopened, red_rounds, with_mistakes}` as counts, plus
  `rates` with the same four keys as a fraction of `cells`, 3 decimals, 0 when
  `cells` is 0.
- Population: cells under `.bee/cells/` (archive included) whose status is
  `capped` or `dropped`.
- Arm of a cell: `knowledge_arm` on the earliest `.bee/logs/dispatch.jsonl`
  row with `source: "prepare"`, `kind: "cell"` and that cell id. No such row
  with the key → `unrecorded`.
- Signals from `trace`: blocked = `blocked_reason` is a non-empty string;
  reopened = `reopened_for_rework` or `reopened_at` present and not null;
  red_rounds = `attempts` is a non-empty array; with_mistakes = `mistakes` is
  a non-empty array.
- `comparable` is true only when both arms hold at least `min_per_arm` cells.
  The text output adds one line per arm and, when not comparable, one line
  that names how many more cells each arm needs.
- Reads from the same root the report reads `.bee/decisions.jsonl` from. A
  missing file or an unparseable row is skipped, never an error.
- Tests first, seen red: a fixture with cells in both arms, an unrecorded
  cell, an open cell (excluded) and each signal gives the exact counts and
  rates; fewer than 30 per arm gives `comparable: false`; no dispatch log
  gives all cells unrecorded.

## Contract — cell kom-3 (help text and knowledge docs)

- Help text for `bee dispatch prepare` names the holdout arm, the
  `knowledge_holdout_percent` key and the two record keys. Help text for
  `bee knowledge report` names the outcomes section. The help text's one
  home is `packages/bee-rs/crates/bee/src/generated/registry_payload.json`
  (no source file elsewhere carries it; earlier features edited it in place),
  so edit it there and keep it valid JSON.
- `docs/knowledge/areas/workflow-state/dispatch.md` gains the holdout
  behavior; `docs/knowledge/areas/okf-profile/critical-bar.md` or the area
  that owns `knowledge report` gains the outcomes measure and states that the
  signature count is not the usefulness measure. Cite this plan's D1-D5, no
  raw decision ids.

## Cells — current slice (preview)

| id | title | you see |
|---|---|---|
| kom-1 | Hold out a fixed tenth of cells from the learned-context block | One cell in ten gets a brief with no knowledge list, and the dispatch log says which group each cell is in |
| kom-2 | Compare holdout and knowledge cells in the knowledge report | `bee knowledge report` shows blocked, reopened, red and mistake rates for both groups |
| kom-3 | Document the holdout and the outcomes measure | Help text and knowledge docs name the new behavior |

Order: kom-1 and kom-2 run at the same time; their files do not overlap and
kom-2 reads only the record keys this plan locks. kom-3 runs after both,
because it owns the shared generated registry file.

```json
[{"id":"kom-1","feature":"knowledge-outcome-measure","lane":"standard","role":"code","title":"Hold out a fixed tenth of cells from the learned-context block","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],"read_first":["docs/history/knowledge-outcome-measure/plan.md","docs/knowledge/areas/workflow-state/dispatch.md"],"action":"Follow the Contract for kom-1 in docs/history/knowledge-outcome-measure/plan.md. Kind cell dispatches get a deterministic arm: holdout when the first 8 hex digits of sha256_hex(cell id) as u32 mod 100 is below .bee/config.json knowledge_holdout_percent (absent or invalid = 10, integer 0..=100 honored, 0 = off). Holdout renders no Learned context block; knowledge arm prompt stays byte-identical. The kind cell prepare record gains knowledge_arm and learned_context (array of listed repo-relative paths, [] when none). Other kinds' records stay byte-identical. Write the tests first and watch them fail.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers","must_haves":{"truths":["A holdout cell brief carries no Learned context block","A knowledge-arm cell brief is byte-identical to before","The same cell id always lands in the same arm","knowledge_holdout_percent 0 holds out no cell; absent or invalid uses 10","The kind cell dispatch record carries knowledge_arm and learned_context; other kinds carry neither"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/dispatch.md"],"decisions":["dafd05a1"],"deps":[]},
{"id":"kom-2","feature":"knowledge-outcome-measure","lane":"standard","role":"code","title":"Compare holdout and knowledge cells in the knowledge report","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs","packages/bee-rs/crates/bee/tests/knowledge_outcomes.rs"],"read_first":["docs/history/knowledge-outcome-measure/plan.md","docs/knowledge/areas/okf-profile/critical-bar.md"],"action":"Follow the Contract for kom-2 in docs/history/knowledge-outcome-measure/plan.md. bee knowledge report keeps measured, unmeasured and the evidence ladder and gains outcomes: per arm (knowledge, holdout) the counts cells, blocked, reopened, red_rounds, with_mistakes plus rates (3 decimals), unrecorded, min_per_arm 30 and comparable. Population: capped or dropped cells under .bee/cells (archive included). Arm: knowledge_arm on the earliest dispatch.jsonl row with source prepare, kind cell and that cell id; none means unrecorded. Signals from trace: blocked_reason non-empty string, reopened_for_rework or reopened_at non-null, attempts non-empty, mistakes non-empty. Missing files or bad rows are skipped. Text output adds one line per arm and a too-few line when not comparable. Write the tests first and watch them fail.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test knowledge_outcomes","must_haves":{"truths":["bee knowledge report JSON carries outcomes with per-arm counts and rates","A cell with no recorded arm counts as unrecorded and in neither arm","Open cells are excluded from the population","comparable is false until both arms reach 30 cells","measured, unmeasured and the evidence ladder are unchanged"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/okf-profile/critical-bar.md"],"decisions":["eeb666c4","dafd05a1"],"deps":[]},
{"id":"kom-3","feature":"knowledge-outcome-measure","lane":"standard","role":"docs","title":"Document the holdout and the outcomes measure","behavior_change":false,"files":["packages/bee-rs/crates/bee/src/generated/registry_payload.json","docs/knowledge/areas/workflow-state/dispatch.md","docs/knowledge/areas/okf-profile/critical-bar.md"],"read_first":["docs/history/knowledge-outcome-measure/plan.md"],"action":"Follow the Contract for kom-3 in docs/history/knowledge-outcome-measure/plan.md. Update the help text for bee dispatch prepare (holdout arm, knowledge_holdout_percent, the knowledge_arm and learned_context record keys) and bee knowledge report (the outcomes section) in packages/bee-rs/crates/bee/src/generated/registry_payload.json, the help text's one home, keeping it valid JSON. Add the holdout behavior to docs/knowledge/areas/workflow-state/dispatch.md and the outcomes measure to docs/knowledge/areas/okf-profile/critical-bar.md, stating the signature count is not the usefulness measure. Cite plan D1-D5, never raw decision ids.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts && .bee/bin/bee knowledge check","must_haves":{"truths":["bee dispatch prepare --help names the holdout and knowledge_holdout_percent","bee knowledge report --help names the outcomes section","The knowledge docs describe the holdout and the outcomes measure citing plan D-numbers"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/dispatch.md","docs/knowledge/areas/okf-profile/critical-bar.md"],"decisions":["eeb666c4","dafd05a1"],"deps":["kom-1","kom-2"]}]
```
