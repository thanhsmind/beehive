---
artifact_contract: bee-plan/v1
mode: standard
plan_rev: 2
---

# Plan: Leader Sees Team

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Why this is the least workflow that protects the work: a new word in the published dispatch record and a changed preamble line are contracts every host reads, and the rule must land in three rendered doctrine homes at once — that earns a plan and a three-seat check; nothing here changes what runs.

**Revision 2.** The wave held the claims table (14/14) and broke the shape: the helper sat where the config is out of reach, eight truth-table cells were undefined, the model guard is a second writer of the status vocabulary, a locked display law stood in D1's way, and the drafted roster line measured 639 characters with twelve roles hidden. Synthesis: `reports/hat-wave-synthesis.md`.

## Requirements (from CONTEXT.md)

- **D1** — The leader chooses a team member by job; transport is config. The two read surfaces lead with job and description; transport trails.
- **D2** — For a herding or cli dispatch bee reads the model out of the argv it built and records it as `requested_model` with status `declared`; no token → `null`/`unverified`. Both surfaces print the declared model.
- **D3** — One sentence in each of three rendered homes: the host `AGENTS.md` block, the delegation contract, the worker prompt.
- **D4** — `bee team show` may resolve for display (role, description, model, transport) and still writes nothing; the preamble roster prints every role one per line with no descriptions and no six-role cap. Amends doctrine B12/B13 on the display axis.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed, output held. Evidence is a verbatim byte substring; multi-line joins with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The null model for herding/cli is a hard-coded branch. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:390-391` | `    let requested_model = if channel == "cli-exec" \|\| channel == "herding-exec" \|\| channel == "session-model" {` / `        Value::Null` |
| 2 | So is the `unverified` status for those channels. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:381-382` | `    } else if channel == "cli-exec" \|\| channel == "herding-exec" {` / `        "unverified"` |
| 3 | **`derive_economics` has no config in reach** — it takes a channel, a role, a param model, a `Resolved` and a flag. The declared model must be computed by its caller. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:342-350` | `pub(crate) fn derive_economics(` / `    channel: &str,` / `    // The DECLARED role (or \`ceiling\`, the escalation word). Named for what it` / `    // now carries; the emitted key stays \`logical_tier\` — see above.` / `    role: &str,` / `    param_model: Option<&str>,` / `    resolved: &Resolved,` / `    native_confirmed: bool,` / `) -> Map<String, Value> {` |
| 4 | Its one production caller already computes `param_model` from channel + resolved, with cfg and runtime in scope — the natural place to compute `declared` too. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1972-1975` | `    let param_model = match (&channel[..], &resolved) {` / `        ("claude-agent", Resolved::Model { model, .. }) => Some(model.clone()),` / `        _ => None,` / `    };` |
| 5 | `Resolved::Herding` carries a name, not an argv. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:349` | `    Herding { agent: Option<String>, fallback: Option<String> },` |
| 6 | `Resolved::Cli` carries a command **string**, not an argv — the helper must tokenize. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:324-326` | `    Cli {` / `        command: String,` / `    },` |
| 7 | The name→argv resolver exists and has a fixed precedence that also covers `agent: None`. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:403` | `pub(crate) fn resolve_agent_command_for_runtime(` |
| 8 | The registry argv carries the model as a plain `--model` token today. | ran | `python3 -c "import json;print(json.load(open('/home/thanhsmind/Projects/goglbe/beehive/.bee/config.json'))['herding']['agents']['agy-flash']['argv'])"` | `['agy', '--model', 'gemini-3.8-flash-high', '--dangerously-skip-permissions']` |
| 9 | **The model guard is a second writer** of the same six economics fields — it recomputes, never calls `derive_economics`. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1254` | `    out.insert("effective_model_status".into(), Value::String(effective_status.to_string()));` |
| 10 | The status vocabulary is pinned by a test, so `declared` is a covered-contract change. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:670` | `            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"herding-exec","enforcement":"herding-command"}"#` |
| 11 | The roster's transport-first rendering. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:417-422` | `fn render_role(resolved: &Resolved) -> Option<String> {` / `    Some(match resolved {` / `        Resolved::Model { model, .. } => model.clone(),` / `        Resolved::Herding { agent, fallback } => {` / `            let mut s = match agent {` / `                Some(a) => format!("herding ({a})"),` |
| 12 | The roster's real emitter — one line, `k=v ("desc")` per role — and its input sees only the `team` subtree, so it cannot reach `herding.agents` today. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:463-465` | `pub(crate) fn role_slot_display(` / `    models_raw: Option<&Value>,` / `    runtime: &str,` |
| 13 | The roster is capped at six roles; this repo configures eighteen, so `plan` and `docs` are invisible at session start. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:346` | `const DOOR_ROLES_SHOWN: usize = 6;` |
| 14 | `bee team show` puts the description last, after the raw slot JSON. | read | `packages/bee-rs/crates/bee/src/verbs/models_group.rs:203-206` | `            let mut line =` / `                format!("  {role:<width$}  [{source}]  {}", slot_display(slot), width = width);` / `            if let Some(description) = role_row.get("description").and_then(Value::as_str) {` / `                line.push_str(&format!(" — {description}"));` |
| 15 | The host `AGENTS.md` block is rendered from one source file and pinned byte for byte by a test. | read | `packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs:3-4` | `// \`AGENTS.md\` is the file agents actually load; the bee-owned part of it is` / `// RENDERED from \`packages/bee/AGENTS.block.md\` by \`bee dev regen\`. Nothing` |
| 16 | The dispatch-door bullet in that source is D3's first home. | read | `packages/bee/AGENTS.block.md:145` | `- The ONE door for any dispatch is \`.bee/bin/bee dispatch prepare` |
| 17 | The delegation contract has one home. | read | `skills/bee-hive/references/gates-and-delegation.md:124` | `### Delegation contract (fan-out: decide-altitude vs gather-altitude)` |
| 18 | The worker prompt that reaches every worker only points at the skill — a pane worker with no skill tree reads nothing more. D3's third sentence belongs here. | read | `packages/bee/prompts/worker-cell.md:43` | `- Load the bee-swarming skill (Execute section) for the full worker contract.` |
| 19 | A shipped reference states the very thing D2 reverses and must be swept. | read | `skills/bee-swarming/references/swarming-reference.md:398` | `A **cli-exec** dispatch (external executor, below) is \`unverified\` too — the command names its own model in its own argv, outside this vocabulary, so \`requested_model\` is always \`null\` there.` |
| 20 | The knowledge home of the status vocabulary. | read | `docs/knowledge/areas/advisor-protocol/slots-and-tiers.md:68-69` | `  model-param dispatch is \`pinned\`; a bare-marker budget dispatch is` / `  \`unverified\`. A second-runtime native spawn stays \`inherited-or-unknown\`` |
| 21 | The baseline is every herding slot on every runtime, not pi alone. | ran | `.bee/bin/bee dispatch prepare --runtime claude --kind gather --role code --json` (economics block) | `"requested_model": null` / `"effective_model_status": "unverified"` / `"channel": "herding-exec"` |
| 22 | Nine of this repo's eighteen `claude` slots are herding — blind today. | ran | `python3 -c "import json; c=json.load(open('/home/thanhsmind/Projects/goglbe/beehive/.bee/config.json'))['team']['claude']; print(sum(1 for v in c.values() if isinstance(v,dict) and v.get('kind')=='herding'), 'of', len(c))"` | `9 of 18` |

## Discovery

`derive_economics` (1–3) nulls the model for herding and cli before looking and cannot look — it has no config. Its caller (4) has everything in scope. A `Resolved::Herding` is a name (5) and a `Resolved::Cli` is a command string (6); one resolver already turns a name — or no name — into an argv (7) whose model is a plain token (8). The model guard writes the same six fields on its own (9). The two read surfaces are transport-first by construction and the roster cannot even reach the registry (11–14). The rule has three rendered homes (15–18), and two shipped texts will go false or stale when `declared` exists (19–20). The blindness is every herding slot on every runtime (21–22).

## Approach

**`declared` — one helper, two writers.** `declared_model_for(cfg: &Value, resolved: &Resolved, runtime: &str) -> Option<String>` lives in the drivers module beside `resolve_role`. `Herding { agent }` — both arms — resolves through `resolve_agent_command_for_runtime(cfg, agent.as_deref(), runtime)` and reads the argv; `Cli { command }` splits the string quote-aware. Tokens accepted: `--model X`, `--model=X`, `-m X`; when more than one is present the **first** wins (a named guess). `prepare.rs` computes it beside `param_model` for `herding-exec` and for **both** cli-exec sites, reading the command that will run (at the `Native`-with-fallback site that is the fallback command, never the `model` field). `derive_economics` gains a sixth argument `declared: Option<&str>`: found → `requested_model = Some`, status `"declared"`; absent → today's exact bytes. `effective_model` stays `null`. A herding `fallback` model rides `payload.fallback.model` as today; the record describes the primary argv only. The model guard's own economics writer calls the same helper for a herding slot, so its audit row agrees with prepare's.

**Job first (D1, D4).** `role_slot_display` gains the whole config. The roster becomes one line per role — `- code → gemini-3.8-flash-high (herding: agy-flash)` — every role, no descriptions, ≤ 60 characters per line, `DOOR_ROLES_SHOWN` retired. `bee team show` renders through `Resolved` + the helper into aligned columns: `role  [source]  description  model  transport`; `--json` keeps the raw slot beside the resolved fields. The status word `declared` appears only in the record; both surfaces print the model name.

**Three sentences (D3).** `AGENTS.block.md` beside the door bullet: *"Pick the team member by job — the role's description says what it is good at; whether it runs as a native model, a herding pane or a cli is config, never the leader's choice."* `gates-and-delegation.md` "Delegation contract": the same rule at gather altitude. `packages/bee/prompts/worker-cell.md`: one clause — the shape you were handed is config, not a signal about the work. Then `bee dev regen`. Sweep `swarming-reference.md:398` and add `declared` to `slots-and-tiers.md` and `config-reference.md`'s status table; amend doctrine B12/B13 citing D4.

**SMALLER PATH check (re-run).** Compute `declared` inside `derive_economics` → impossible without threading a config through a pure function (claim 3). Skip the guard writer → two contradicting rows per role (claim 9). Keep the single-line roster → a 500-character wall (claim 13). Skip the three sentences → the mechanism changes and no host is told. PASS.

**Rejected alternatives.**
- *Widen `pinned` to argv flags* — `pinned` means bee controls the param; overclaim.
- *A second helper per surface* — three parsers of one fact.
- *Keep B13 and reorder description-only* — the roster shows a job with no model on pi, the lie the owner named.
- *Put D3's worker sentence in `worker-details.md` only* — a pane worker never loads it (claim 18).

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `declared_model_for` + the sixth argument | MEDIUM — a published record gains a word; the null path must stay byte-identical | tests: `--model X` / `--model=X` / `-m X` / first-wins / `agent: None` via generation slot / a fixture registry entry with **no** token → `null`+`unverified` (claim 10's bytes) / `Native` fallback command read, never its `model` |
| The guard's second writer | MEDIUM — two rows disagreeing is worse than one blind row | a test that prepare's row and the guard's row carry the same `requested_model` for a herding slot |
| Roster + `team show` | LOW — display, but tests assert the old bytes | updated tests; every role appears; ≤ 60 chars per roster line; `team show` columns in D4's order on claude and pi |
| Three sentences + regen + sweeps | LOW | parity test green; `rg` finds each sentence exactly once; `swarming-reference.md:398` no longer says "always null" |

## Shape

**Feature outcome.** A leader on any host reads *what each team member is for* and *what it will run as*, in that order, on every runtime, with every role visible at session start — and picks by the first.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | `declared`, one helper, two writers | D2; the record stops lying on every herding slot | S1 | the seven economics tests; the guard-agrees test; claims 21–22 flip to `declared` |
| E2 | Job-first surfaces | D1, D4 | S1 | display tests; live roster and `team show` |
| E3 | The rule in every host | D3 | S1 | parity test; one sentence per home; the false sentence gone |

**Slice queue — one slice, three cells.** (1) `declared`: helper, sixth argument, both prepare sites, the guard's writer, tests. (2) surfaces: `role_slot_display` config, multi-line roster, `team show` through `Resolved`, `TEACH`, tests — **depends on (1)**. (3) doctrine: three sentences, the two sweeps, B12/B13 amendment citing D4, `config-reference.md` status row, regen — **parallel with (1)**, wave-barrier. **Current slice to prepare: S1.**

## Test matrix

| Path | Probe |
|---|---|
| Happy | herding argv `--model X` → `requested_model: X`, `declared`; claims 21–22 re-run show `declared` |
| Happy | cli command `-m X` → `declared`; `--model=X` → `declared` |
| Edge | registry fixture with no token → exactly claim 10's bytes |
| Edge | `agent: None` → the generation-slot / `agent_command` argv's model |
| Edge | two tokens → the first |
| Edge | `Native` slot on cli-exec → the fallback command's model, never the `model` field |
| Edge | native model slot → still `pinned`; session-model → still `inherited-or-unknown` |
| Agreement | guard audit row == prepare row for one herding slot |
| Display | roster: every role, one per line, ≤ 60 chars, job → model → transport; `team show`: role · source · description · model · transport, both runtimes |
| Docs | parity test green; each sentence once; `swarming-reference.md` no longer says "always null" |

## Open Questions

- (none)

## Out of scope

- Injecting the declared model into the worker brief (fix 4 of the research).
- Economics for a **refused** dispatch (none is emitted today; a separate gap).
- Any change to which agent a role resolves to, or to the `models` alias.
