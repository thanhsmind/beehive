# Hat wave — plan-step synthesis, team-config-rename

Date: 2026-09-09. Three seats, two models, one draft (`plan.md` revision 1, commit `6e3d7061`).
Seats: `hat-facts-gaps` (opus), `hat-alternatives` (opus), `hat-user-impact` (agy-flash pane).
No seat dropped. This wave is the feature's plan check (absorption, decision `b34fdea9`).

## Verdict

**Revision 2 is owed.** Revision 1's claims table carried one false claim, one telemetry read
mislabelled as a config read, one fabricated `ran` row, and a count that was wrong three ways;
its survey was blind to a config *writer*; and its SMALLER PATH check passed against three
strawmen while the repo already held a cheaper shape. Every finding below was re-verified by the
leader at the anchor before it changed the plan.

## Accepted blockers → what changed

| # | Seat | Finding (verified anchor) | Change in revision 2 |
|---|---|---|---|
| B1 | alternatives · facts-gaps | Claim 10 false: `status_full/store.rs:384` is `let mut out = crate::verbs::drivers::normalize_models(raw);` — a wrapper, not a duplicate. | Claim, rejected-alternative and out-of-scope line deleted. |
| B2 | alternatives · facts-gaps | `session_close/perf.rs:576` reads a **session usage record** (`read_session_records()`, `:544`), not config. Routing it would corrupt telemetry. | Removed from the reader list; named as an explicit exclusion of the regression gate. |
| B3 | alternatives · facts-gaps | A **writer** was missed: `onboard/templates.rs:175` `"models": {` inside `default_config()`, written by `onboard/apply.rs:374` for every fresh host; and `.bee/config-sample.json` is compiled in via `include_str!` (`templates.rs:264`). | Both move to S1, in the same cell as the loader fold, so a fresh host never sees the old key and never trips its own warning. Tests `templates.rs:491`, `:582` named. |
| B4 | alternatives · facts-gaps | `bee status --json` publishes a top-level `models` **output** key (`build.rs:339-344`); its tests read it back. The zero-raw-reads gate would force an undecided breaking rename. | Decision recorded in the plan: the status output key **stays `models`** — D1 renames a config key, and `build.rs:346`'s own comment marks output keys as breaking. The gate gets an exact command with exclusions. |
| B5 | facts-gaps | Claim 12's `ran` evidence does not exist in `bee reviews record --help` output. | Re-anchored as `read` at `reviews.rs:81` `const DECISION_GATE_FIELD_LEGACY: &str = "gate4";`. |
| B6 | facts-gaps · user-impact | ~35 user-facing Rust strings name `models.<rt>` — the guard refusal (`model_guard.rs:293-297`), **the dispatch-door line every session preamble prints** (`:542`), `models.rs:489`, `prepare.rs:143,147`, fifteen `bee status` problem messages in `store.rs`, `doctor.rs:337`, `validate.rs:362`, `templates.rs:290`, `router.rs:86` — plus tests asserting them verbatim. Routing reads while the words still say `models` keeps the misreading alive on the one surface that causes it. | A prose-sweep cell in S1 with its test updates. |
| B7 | facts-gaps | Count wrong: the `rg` gives 15 lines not 14; 3 are `#[cfg(test)]` fixtures; 1 is telemetry. Real config readers outside drivers = **11**, and two of them (`budget.rs:650`, `compaction.rs:1525`) feed the same `dispatch_door_lines`. | Every count re-derived. |
| B8 | user-impact | A deprecation line on stdout corrupts `--json` (`models_group.rs:235` emits through `ctx.emit`). | Deprecation goes to stderr; stdout and JSON byte-identical to `bee team show`. |
| B9 | user-impact | A preamble warning on every session start is endless noise agents will try to "fix". | `bee doctor` owns the persistent warning; the preamble prints one terse line naming `.bee/config.json`, inside `### Dispatch door`. |

## Accepted warnings → what changed

- **Loader-level fold (alternatives W5).** Three loaders reach the config — `state.rs:161 read_config_raw`, `session_preamble/state.rs:50 read_config_raw_open`, `compaction.rs:124 read_config_failopen` — and two already normalize a key at load (`merged.shift_remove("advisor")`, `state.rs:185`, `session_preamble/state.rs:61`). Revision 2 puts the `team`-else-`models` fold **in one helper the three loaders call**, so every downstream site reads one spelling with no fallback logic. `onboard/agents.rs:73` bypasses the loaders (`read_json_if_exists`) and gets its own edit. This is D2's "one accessor" satisfied more strongly, not less.
- **One reader, not three (W6).** Preamble and compaction both call `model_guard::dispatch_door_lines` (`model_guard.rs:520`); the read moves inside it.
- **Router alias (W7).** `router.rs:173` `FLOW_VERBS` already rewrites head tokens (`("gate", &["state","gate"])`); `("models", &["team"])` aliases the verb in one line. Caveats kept: `models` still needs its registry entry for the help probe (`router.rs:203-212`, `catalog.rs:816`), and the deprecation line needs its own emit point.
- **Docs sweep resized (W8).** 101 files, of which **65 are `docs/history/`** — frozen records that must not be rewritten ("cite them, never reinterpret"). Live surface is 36 files. The "reach zero" proof gets an exclusion set.
- **S2 is a docs-lane task (W9)**, not a cell slice: this repo's `.bee/config.json` flip plus ~36 docs plus the knowledge sync. Its dependency on S1 is kept.
- **Claim 9's sentence was wrong** — `wave.rs:357` maps `pi` to `"claude"`; the pi path is `prepare.rs:40` / `models.rs:41`. Quote kept, sentence fixed.
- **Doctor warning has no per-runtime home** — `doctor.rs:294` is per-runtime and covers only Claude/Codex (`doctor.rs:49-52`). The warning gets a top-level check.
- **`team` present but not an object** had three different existing behaviors. Revision 2 picks one: the fold only acts when `team` is absent; a present non-object `team` is left in place and every site's existing handling applies unchanged.
- **Both keys present**: the fold drops `models` from the loaded map, so no site can read it; the warning names the ignored key.

## Rejected on evidence

- *Rename the `bee status --json` output key too* — rejected: it is a published output contract, marked breaking by the code's own comment; D1 is about the config key.
- *Rewrite `docs/history/**` to say `team`* — rejected: those are locked records of decisions made when the key was `models`, including the record of why D1 exists.
