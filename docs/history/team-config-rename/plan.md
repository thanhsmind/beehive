---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Team Config Rename

Mode: `standard` — 3 risk flags: public-contracts, multi-domain, covered-contract-change
Why this is the least workflow that protects the work: the key being renamed is read by a hook that decides every dispatch in every host repo — a reader left on the old spelling silently ignores the new block — so the shape must prove that **every** reader goes through one accessor, and that earns a plan and a wave; nothing here touches auth, data or an external system, so it earns no more.

## Requirements (from CONTEXT.md)

- **D1** — `models.<runtime>.<role>` → `team.<runtime>.<role>`; values and shapes unchanged; `bee models show` → `bee team show`.
- **D2** — `models` stays readable as an alias through **one accessor** (`team` first, `models` fallback); one warning line from `bee doctor` and one from the session preamble when a config still carries `models`; alias removal is its own release.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed, output held; `guessed` = never at the gate. Evidence is a verbatim byte substring; multi-line joins with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The drivers module already has a read door that every honest reader could share — it is one line, and it names the old key. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:286-293` | `pub(crate) fn read_models(root: &Path) -> D<Map<String, Value>> {` / `    Ok(normalize_models(config.get("models")))` |
| 2 | The role resolver lives beside it, so the accessor sits where B1 says the one parser is. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:526` | `pub(crate) fn resolve_role(` |
| 3 | **Fourteen sites bypass that door and read the raw key.** The model guard — the hook that decides every dispatch — is one of them. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:80` | `    let models = normalize_models(config.get("models"));` |
| 4 | The session preamble's dispatch-door line reads the raw key. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:650` | `    lines.extend(crate::hooks::model_guard::dispatch_door_lines(config.get("models"), "claude"));` |
| 5 | `bee doctor` reads the raw key. | read | `packages/bee-rs/crates/bee/src/doctor.rs:296-297` | `    let Some(table) = config` / `        .get("models")` |
| 6 | The `models show` verb reads the raw key. | read | `packages/bee-rs/crates/bee/src/verbs/models_group.rs:149` | `    build_table(config.get("models"), runtime)` |
| 7 | `bee status` reads the raw key. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:448` | `    let models = normalize_models(raw.get("models"));` |
| 8 | Onboarding's agent renderer reads the raw key. | read | `packages/bee-rs/crates/bee/src/onboard/agents.rs:78` | `        .and_then(\|c\| c.get("models"))` |
| 9 | The herding agent resolver reads the raw key — the `pi` runtime's whole path runs through here. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:358` | `    let slot = cfg.get("models")?.get(rt)?.get("generation")?;` |
| 10 | **The normalizer itself is already duplicated** — two `normalize_models` functions exist, in the drivers module and in the status store. One accessor must feed both, or elect one. | read | `packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:383` | `pub(crate) fn normalize_models(raw: Option<&Value>) -> JMap {` |
| 11 | The compaction hook reads the raw key. | read | `packages/bee-rs/crates/bee/src/hooks/compaction.rs:1525` | `        .push(crate::hooks::model_guard::dispatch_door_lines(config_val.get("models"), "claude"));` |
| 12 | A read-time alias for a legacy spelling is an existing bee pattern — the review store already normalizes `gate4` → `review` on read, never rewriting the file. | ran | `.bee/bin/bee reviews record --help` | `normalizes legacy \`gate4\` to \`review\`` |
| 13 | The raw-read count is the whole surface, not a sample. | ran | `rg -n 'get\("models"\)' packages/bee-rs/crates/bee/src --glob '*.rs' \| grep -v /tests \| grep -v drivers/models.rs` | 14 lines: doctor.rs:297 · models_group.rs:149 · status_full/build.rs:343 · store.rs:448 · store.rs:535 · store.rs:853 · onboard/agents.rs:78 · model_guard.rs:80 · model_guard.rs:2015 · :2549 · :2582 · session_close/perf.rs:576 · compaction.rs:1525 · herding/wave.rs:358 · session_preamble/budget.rs:650 |
| 14 | The docs surface that names the old key is large enough to be its own slice. | ran | `rg -l 'models\.(claude\|pi\|codex\|<runtime>\|<rt>)' docs skills AGENTS.md README.md .pi \| wc -l` | `99` |

## Discovery

Read every raw `get("models")` in the crate (claim 13). No accessor exists; `read_models` (claim 1) is the closest thing and is itself one of the raw readers. The model guard, the preamble and compaction — the three hook-path readers that run inside every host session — all read the raw key (claims 3, 4, 11), so a config that adopts `team` before those three are routed would have its roster silently ignored at every dispatch. That ordering is the whole shape.

## Approach

**Recommended path.** One accessor in the drivers module, `team_block(cfg) -> Option<&Value>`, returning `cfg["team"]` when present else `cfg["models"]`; `read_models` becomes `read_team` and calls it; every one of the fourteen raw sites calls it. D2's two warnings read the same accessor's "which key won" answer. `bee team show` is the renamed verb group; `bee models show` stays as an alias that prints one deprecation line. This repo's own `.bee/config.json` and `.bee/config-sample.json` move to `team` last, after every reader is routed, so the dogfood config never runs ahead of the code that reads it.

**SMALLER PATH check.** Cheaper shape honoring D1 and D2? Skip the accessor and rename the string at fourteen sites → fails D2's "one accessor, never two spellings at a call site", and leaves the next reader free to type the old key. Skip the docs sweep → a rule with two names in its own docs is the misreading this feature exists to end. Skip the verb alias → every skill that says `bee models show` breaks. PASS: each piece maps to a decision.

**Rejected alternatives.**
- *Rewrite host configs on read (migrate `models` → `team` in place)* — bee never rewrites a host's config; the alias-on-read pattern (claim 12) is the precedent.
- *A second table beside `models`* — CONTEXT.md forbids it; two homes for one fact.
- *Electing one `normalize_models` now* (claim 10) — real debt, but it is a refactor of the normalizer, not of the key; deferred with its own line rather than smuggled in.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| The accessor | **HIGH** — every dispatch in every host reads through it | `team` wins when both present; `models` serves when `team` absent; neither → `None`; a test per case |
| The three hook-path readers (guard, preamble, compaction) | **HIGH** — a missed one ignores the roster silently | after routing, `rg 'get\("models"\)'` outside the accessor returns zero; the guard's existing role tests pass against a `team`-keyed fixture |
| The eleven non-hook readers | MEDIUM | the same `rg` gate; `bee status`, `bee doctor`, `bee team show` each run against a `team`-keyed config |
| The warnings | LOW | doctor and preamble each emit exactly one line on a `models`-only config and none on a `team` config |
| The verb alias | LOW | `bee models show` output equals `bee team show` output plus one trailing line |
| The docs sweep | LOW | `rg` count of the old spelling outside the alias paragraph in `config-reference.md` reaches zero |

## Shape

**Feature outcome.** The leader's roster is called what it is; a host that has not renamed loses nothing and is told once; no reader anywhere can be found still holding the old spelling.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | One accessor, every reader through it | D2's single home; the hook-path readers are the danger | S1 | accessor tests; the zero-raw-reads gate |
| E2 | Warnings and the verb alias | D2's two lines; D1's `team show` | S1 | one-line-each tests; alias-equals-plus-one test |
| E3 | Dogfood config and docs | The repo that ships bee must speak the new name everywhere | S2 | config on `team`; docs `rg` reaches zero; knowledge area synced |

**Slice queue.**

- **S1 — the accessor and every reader (current slice).** Accessor + `read_team` in the drivers module with tests; the fourteen sites routed, hook-path first; doctor and preamble warnings; `bee team show` with the `models show` alias. Ends with the zero-raw-reads gate green and the full suite green. Config files untouched — the alias carries this repo.
- **S2 — the repo speaks `team`.** Depends on S1. `.bee/config.json` and `.bee/config-sample.json` to `team`; the 99-file docs sweep with the alias documented once in `config-reference.md`; knowledge-area sync (B13 names `models show`; B16 names `models.pi`). Wave-barrier regen at close.

**Current slice to prepare: S1.**

## Test matrix

Standard: the triad, judged against existing coverage first — the model guard's own tests already drive `normalize_models` against fixtures, and those fixtures are the pattern.

| Path | Probe |
|---|---|
| Happy | a config with only `team`: guard resolves, preamble prints the roster, `bee team show` lists it |
| Happy (alias) | a config with only `models`: everything above still works byte-identically, plus exactly one warning line from doctor and one from preamble |
| Edge | both keys present: `team` wins, `models` ignored, warning names the ignored key |
| Edge | neither present: every reader behaves as today with no block |
| Error | `team` present but not an object: the same typed refusal the old key gave |
| Regression | `rg 'get\("models"\)'` outside the accessor → 0 |

## Open Questions

- Whether `bee onboard` should write `team` into a fresh host's config (deferred-to-planning in CONTEXT.md). S2 decides it with evidence from `onboard/templates.rs`; S1 does not depend on it.

## Out of scope

- Electing one `normalize_models` (claim 10) — its own refactor.
- Removing the `models` alias — its own release (D2).
- Anything about which model a slot runs — `bee-model-binding-stability-xia.md` fixes 3 and 4 are separate features.
