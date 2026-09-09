---
artifact_contract: bee-plan/v1
mode: standard
plan_rev: 2
---

# Plan: Team Config Rename

Mode: `standard` — 3 risk flags: public-contracts, multi-domain, covered-contract-change
Why this is the least workflow that protects the work: the key being renamed is read by the hook that decides every dispatch in every host repo, and **written** by the command that onboards every fresh host — so the shape must prove that the fold happens once, at load, and that no reader, writer or user-facing string is left on the old spelling. Nothing here touches auth, data or an external system.

**Revision 2.** The three-seat hat wave broke revision 1: one claim was false, one "reader" was telemetry, a config writer was missed, the regression gate could not reach zero, and the SMALLER PATH check had passed against strawmen while the repo already held a cheaper shape. Synthesis: `reports/hat-wave-synthesis.md`.

## Requirements (from CONTEXT.md)

- **D1** — `models.<runtime>.<role>` → `team.<runtime>.<role>`; values and shapes unchanged; `bee models show` → `bee team show`.
- **D2** — `models` stays readable as an alias resolved in **one place**; a config still carrying `models` gets one warning line from `bee doctor` and one from the session preamble; alias removal is its own release.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed, output held. No `guessed` row survives the gate. Evidence is a verbatim byte substring; multi-line joins with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Config reaches the crate through three loaders. This is the first. | read | `packages/bee-rs/crates/bee/src/state.rs:161` | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 2 | The second loader — the session preamble's. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:50` | `pub(crate) fn read_config_raw_open(root: &Path) -> JMap {` |
| 3 | The third loader — compaction's. | read | `packages/bee-rs/crates/bee/src/hooks/compaction.rs:124` | `pub(crate) fn read_config_failopen(root: &Path) -> Map<String, Value> {` |
| 4 | **Load-time key normalization is an existing pattern in these loaders** — the fold has a precedent, not a new mechanism. | read | `packages/bee-rs/crates/bee/src/state.rs:185` | `    merged.shift_remove("advisor");` |
| 5 | The same normalization in the second loader. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:61` | `    merged.shift_remove("advisor");` |
| 6 | One reader bypasses all three loaders and reads the file directly — it needs its own edit. | read | `packages/bee-rs/crates/bee/src/onboard/agents.rs:73` | `    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));` |
| 7 | The model guard — the hook that decides every dispatch — reads the loaded map by the old key. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:80` | `    let models = normalize_models(config.get("models"));` |
| 8 | The preamble and compaction do not read it themselves — both hand the value to one shared helper, so they are one reader. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:520` | `pub(crate) fn dispatch_door_lines(models_raw: Option<&Value>, runtime: &str) -> Vec<String> {` |
| 9 | The preamble's call into that helper. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:650` | `    lines.extend(crate::hooks::model_guard::dispatch_door_lines(config.get("models"), "claude"));` |
| 10 | Compaction's call into the same helper. | read | `packages/bee-rs/crates/bee/src/hooks/compaction.rs:1525` | `        .push(crate::hooks::model_guard::dispatch_door_lines(config_val.get("models"), "claude"));` |
| 11 | The remaining production readers, by anchor: doctor, the verb, status (three), and the herding resolver. | read | `packages/bee-rs/crates/bee/src/doctor.rs:297` · `verbs/models_group.rs:149` · `verbs/status_full/build.rs:343` · `store.rs:448` · `store.rs:535` · `store.rs:853` · `herding/wave.rs:358` | `        .get("models")` / `    build_table(config.get("models"), runtime)` / `            config_models.raw.get("models"),` / `    let models = normalize_models(raw.get("models"));` / `    let Some(models) = obj.get("models") else { return problems };` / `        Value::Object(m) => m.get("models"),` / `    let slot = cfg.get("models")?.get(rt)?.get("generation")?;` |
| 12 | **A writer exists.** Every fresh host's config is generated with the old key. | read | `packages/bee-rs/crates/bee/src/onboard/templates.rs:175` | `        "models": {` |
| 13 | The sample config is compiled into the binary, so it moves with the writer, not with the docs. | read | `packages/bee-rs/crates/bee/src/onboard/templates.rs:264` | `pub const CONFIG_SAMPLE_JSON: &str = include_str!("../../../../../../.bee/config-sample.json");` |
| 14 | `bee status --json` publishes a top-level **output** key named `models` — a public contract, distinct from the config key, and marked breaking by the code's own comment two paragraphs down. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/build.rs:339-340` | `    status.insert(` / `        "models".into(),` |
| 15 | The code itself names output-key renames as breaking. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/build.rs:348` | `    // \`bee status --json\` is published output, so this is a BREAKING key` |
| 16 | The status normalizer is a wrapper over the drivers' one, not a duplicate — one fold at load feeds both. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:384` | `    let mut out = crate::verbs::drivers::normalize_models(raw);` |
| 17 | A read-time alias for a legacy spelling is an existing bee pattern. | read | `packages/bee-rs/crates/bee/src/verbs/reviews.rs:81` | `const DECISION_GATE_FIELD_LEGACY: &str = "gate4";` |
| 18 | The router already aliases head tokens, so `bee models show` → `bee team show` is one table row. | read | `packages/bee-rs/crates/bee/src/router.rs:175` | `    ("gate", &["state", "gate"]),` |
| 19 | The dispatch-door line — printed into every session preamble — names the old key in prose. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:542` | `            "- Roles ({runtime}): {listed} — open set: any name models.{runtime} configures is legal; one nothing configures refuses by name."` |
| 20 | The guard's refusal teaches the old key. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:293` | `models.{runtime} in .bee/config.json carries no \"{name}\" entry, so the dispatch would \` |
| 21 | The verb's own teaching line names the old key. | read | `packages/bee-rs/crates/bee/src/verbs/models_group.rs:48` | `const TEACH: &str = "models — the role table bee dispatches from. A role's \`description\` is written in \` |
| 22 | The prose surface is large. | ran | `rg -n 'models\.(claude\|codex\|pi\|opencode\|<runtime>\|<rt>\|\{)' packages/bee-rs/crates/bee/src --glob '*.rs' \| grep -v "/tests\|tests.rs" \| wc -l` | `88` |
| 23 | The verb emits JSON through the shared emitter, so a stdout deprecation line would corrupt `--json`. | read | `packages/bee-rs/crates/bee/src/verbs/models_group.rs:235` | `            Some(ctx.emit(&result, &text, 0))` |
| 24 | `perf.rs:576` reads a session usage record, not config — it is an exclusion, not a site. | read | `packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs:544` | `    for r in read_session_records() {` |
| 25 | The herding resolver never sees `pi` — it maps it to claude — so the pi path is elsewhere and this site is claude/codex/opencode only. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:357` | `    let rt = if matches!(runtime, "claude" \| "codex" \| "opencode") { runtime } else { "claude" };` |
| 26 | The docs surface splits into frozen history and live files. | ran | `rg -l 'models\.(claude\|pi\|codex\|<runtime>\|<rt>)' docs skills AGENTS.md README.md .pi \| grep -c "^docs/history/"` and the same with `-vc` | `65` history · `36` live |

## Discovery

The wave's survey replaced revision 1's. Config enters through three loaders (claims 1–3), two of which already normalize a key at load (4–5); one reader bypasses them (6). Of the raw readers, the three hook-path ones are really one reader plus one shared helper (7–10). A **writer** generates the old key for every fresh host (12), and the sample it ships beside is compiled in (13). `bee status --json` has its own public `models` output key that is not the config key (14–15). And the words users read — the preamble line, the refusal, the teaching line, 88 strings in all (19–22) — still say `models`.

## Approach

**Recommended path — fold once, at load.** One helper in the drivers module, `fold_team_key(map: &mut Map) -> bool`: when `team` is absent and `models` present, move `models` to `team` and return `true`; when both are present, drop `models` and return `true`; otherwise return `false`. The three loaders (claims 1–3) call it beside their existing `shift_remove("advisor")` (claims 4–5), and `onboard/agents.rs` calls it on its direct read (claim 6). Every downstream site then reads `"team"` — a string change, no fallback logic, no accessor to forget. `dispatch_door_lines` (claim 8) reads the map itself, collapsing two hook sites into one. The writer (claim 12) and the sample (claim 13) move to `team` in the same cell, so a fresh host never trips its own warning. `bee team show` is the verb; `("models", &["team"])` in `FLOW_VERBS` (claim 18) aliases it, and the alias's registry entry stays for the help probe; its deprecation line goes to **stderr** (claim 23). The 88 prose strings (claims 19–22) are swept in their own cell with their test assertions. The `bee status --json` output key **stays `models`** (claims 14–15).

**The warnings (D2).** Both read `fold_team_key`'s boolean on their own raw read. `bee doctor` owns the persistent warning as a top-level check — not inside the per-runtime `hat_slots_missing_a_description` (`doctor.rs:294`), which would print twice and never see `pi`/`opencode`. The preamble prints one terse line inside `### Dispatch door`, naming `.bee/config.json` and the new key, and nothing more.

**Defined cells of the alias table.** `team` only → used. `models` only → folded, both warnings fire once. Both → `team` used, `models` dropped from the map, warning names the ignored key. Neither → as today. `team` present but **not an object** → the fold does nothing; the value stays in place and each site's existing handling applies unchanged (no new refusal invented).

**SMALLER PATH check, re-run.** Revision 1's PASS tested three strawmen. The wave found the cheaper shape in the repo: three loaders instead of eleven call sites, one helper instead of two hook reads, one router row instead of a hand-written alias verb. That is now the plan. Is there a cheaper shape still? Skip the prose sweep → the preamble keeps teaching the old key at every dispatch, the misreading D1 exists to end. Skip the writer → every fresh host trips its own warning. Skip the docs sweep → two names in bee's own docs. PASS.

**Rejected alternatives.**
- *An accessor imported at every call site* (revision 1) — eleven places to forget, where the loader fold is three and already has precedent.
- *Rename the status output key* — a published contract, breaking by the code's own word (claim 15); D1 is about the config key.
- *Rewrite `docs/history/**`* — locked records; "cite them, never reinterpret them".
- *Rewrite host configs on read* — bee never rewrites a host's config; the alias-on-read pattern (claim 17) is the precedent.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `fold_team_key` and the three loaders | **HIGH** — every read in every host passes through it | four cases (`team`/`models`/both/neither) plus the non-object case, each a test; the guard's existing role tests pass against a `team`-keyed fixture |
| The writer + sample | HIGH — a fresh host with the old key trips its own warning on day one | `default_config()` emits `team`; `templates.rs:491` key-order test and `:582` sample-parse test updated; an onboarded scratch host shows no warning |
| The prose sweep | MEDIUM — 88 strings, tests assert them verbatim | zero `models.<rt>` in non-test Rust strings; every asserting test updated, suite green |
| The regression gate | MEDIUM — the wrong gate drives a worker to break telemetry or a public key | exact command below, with its exclusions, green |
| The verb alias | LOW | `bee models show` stdout and `--json` byte-identical to `bee team show`; one stderr line |
| The warnings | LOW | doctor: one line on a `models` config, none on `team`; preamble: one line, names the file and the new key |

**The regression gate, exactly:**

```
rg -n 'get\("models"\)' packages/bee-rs/crates/bee/src --glob '*.rs' \
  | grep -v 'hooks/session_close/perf.rs' \
  | grep -v 'verbs/status_full/build.rs:3[34][0-9]' \
  | grep -v '/tests\|tests.rs\|#\[cfg(test)\]'
```

must print nothing. Excluded by name: `perf.rs` (telemetry, claim 24), the status output key (claims 14–15), and test modules. Anything else is a missed reader.

## Shape

**Feature outcome.** The leader's roster is called what it is in the config, in every message an agent reads, and in every fresh host bee creates; a host that has not renamed loses nothing and is told once, in two places, tersely.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | The fold, once, at load | D2's one home; the hook-path reader is the danger | S1 | fold tests; the exact regression gate; guard tests on a `team` fixture |
| E2 | The writer and the sample | A fresh host must be born on the new key | S1 | onboarded scratch host shows `team` and no warning |
| E3 | The words agents read | The preamble line is the surface that caused the misreading | S1 | zero old-key prose in non-test Rust; asserting tests updated |
| E4 | Warnings and the verb alias | D2's two lines; D1's `team show` | S1 | one-line tests; stdout/JSON identical, stderr carries the notice |
| E5 | This repo speaks `team` | Dogfood config, 36 live docs, knowledge area | S2 (docs lane) | config on `team`; docs `rg` with the exclusion set reaches zero; B13/B16 synced |

**Slice queue.**

- **S1 — everything in code (current slice).** Four cells, three of them parallel: (a) the fold helper + three loaders + `agents.rs` + `dispatch_door_lines` + the eleven readers to `"team"` + the exact gate; (b) the writer and the compiled sample; (c) the 88-string prose sweep with its tests; (d) `bee team show`, the router alias row, the stderr deprecation, the doctor and preamble warnings — depends on (a). Ends with the full suite green and the gate empty. This repo's own `.bee/config.json` is **untouched** — the alias carries it, which is itself the live proof of D2.
- **S2 — this repo speaks `team` (docs lane, no cells).** Depends on S1 merged. Flip `.bee/config.json`; sweep the 36 live docs/skills files, documenting the alias once in `docs/config-reference.md`; sync `model-roles-and-escalation.md` (B13 `models show`, B16 `models.pi`). `docs/history/**` is never touched. Proof: the docs `rg`, excluding `docs/history/`, `docs/decisions/`, and this feature's own record, reaches zero. Wave-barrier regen at close.

**Current slice to prepare: S1.**

## Test matrix

Standard: the triad, against existing coverage — the guard's tests already drive fixtures through `normalize_models`, and those fixtures become `team`-keyed.

| Path | Probe |
|---|---|
| Happy | `team` only: guard resolves, preamble roster prints, `bee team show` lists it, no warning anywhere |
| Happy (alias) | `models` only: byte-identical behavior, plus exactly one doctor line and one preamble line, both naming `.bee/config.json` and `team` |
| Edge | both present: `team` wins; `models` is absent from the loaded map; the warning names the ignored key |
| Edge | neither: as today, no block |
| Edge | `team` present, not an object: unchanged per-site handling; no new refusal |
| Writer | `default_config()` and `CONFIG_SAMPLE_JSON` both carry `team` and no `models`; onboarding a scratch host emits no warning |
| Prose | `rg` for `models.<rt>` over non-test Rust strings → 0; every test that asserted the old wording passes on the new |
| Alias verb | `bee models show` stdout == `bee team show` stdout; `--json` identical; exactly one stderr line |
| Regression | the exact gate above prints nothing |
| Public contract | `bee status --json` still carries its `models` output key; its tests untouched |

## Open Questions

- None that block S1. S2's docs exclusion set is stated above.

## Out of scope

- Removing the `models` alias — its own release (D2).
- Renaming the `bee status --json` output key — a separate, breaking decision if ever wanted.
- Anything about which model a slot runs — separate features.
