---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Leader Sees Team

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Why this is the least workflow that protects the work: a new word in the published dispatch record and a changed preamble line are contracts every host reads, and the rule must land in three rendered doctrine homes at once — that earns a plan and a three-seat check; nothing here changes what runs, so it earns no more.

## Requirements (from CONTEXT.md)

- **D1** — The leader chooses a team member **by job**; transport is config. The preamble roster line and `bee team show` lead with job and description; transport trails.
- **D2** — For a herding or cli dispatch bee **reads the model out of the argv it built** and records it as `requested_model` with status **`declared`**; no token → `null`/`unverified`. Both read surfaces print the declared model beside the job.
- **D3** — One sentence in each of three rendered homes: the host `AGENTS.md` block, the delegation contract, the worker brief.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed, output held. Evidence is a verbatim byte substring; multi-line joins with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The null model for herding/cli is a hard-coded branch, not a limitation. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:390-391` | `    let requested_model = if channel == "cli-exec" \|\| channel == "herding-exec" \|\| channel == "session-model" {` / `        Value::Null` |
| 2 | So is the `unverified` status for those channels. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:381-382` | `    } else if channel == "cli-exec" \|\| channel == "herding-exec" {` / `        "unverified"` |
| 3 | The payload builder holds the agent name at the moment it writes the command — the fact `declared` needs is already in hand. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1804-1806` | `                if let Some(agent) = agent {` / `                    command.push_str(" --agent \"");` / `                    command.push_str(agent);` |
| 4 | That name resolves to its full argv through an existing function with a fixed precedence. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:403` | `pub(crate) fn resolve_agent_command_for_runtime(` |
| 5 | The registry argv carries the model as a plain `--model` token today. | ran | `python3 -c "import json;print(json.load(open('/home/thanhsmind/Projects/goglbe/beehive/.bee/config.json'))['herding']['agents']['agy-flash']['argv'])"` | `['agy', '--model', 'gemini-3.8-flash-high', '--dangerously-skip-permissions']` |
| 6 | The status vocabulary is pinned by a test, so `declared` is a covered-contract change. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:670` | `            r#"{"logical_tier":"generation","requested_model":null,"effective_model":null,"effective_model_status":"unverified","channel":"herding-exec","enforcement":"herding-command"}"#` |
| 7 | **The roster line renders transport first** — the leader reads `herding (agy-flash)` where D1 wants a job and a model. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:417-422` | `fn render_role(resolved: &Resolved) -> Option<String> {` / `    Some(match resolved {` / `        Resolved::Model { model, .. } => model.clone(),` / `        Resolved::Herding { agent, fallback } => {` / `            let mut s = match agent {` / `                Some(a) => format!("herding ({a})"),` |
| 8 | **`bee team show` puts the description last** — after the slot's raw JSON. | read | `packages/bee-rs/crates/bee/src/verbs/models_group.rs:203-206` | `            let mut line =` / `                format!("  {role:<width$}  [{source}]  {}", slot_display(slot), width = width);` / `            if let Some(description) = role_row.get("description").and_then(Value::as_str) {` / `                line.push_str(&format!(" — {description}"));` |
| 9 | The host `AGENTS.md` block is rendered from one source file, and a test pins the render byte for byte — so D3's sentence reaches every host through that file. | read | `packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs:3-4` | `// \`AGENTS.md\` is the file agents actually load; the bee-owned part of it is` / `// RENDERED from \`packages/bee/AGENTS.block.md\` by \`bee dev regen\`. Nothing` |
| 10 | The dispatch-door bullet in that source is where D3's first sentence belongs. | read | `packages/bee/AGENTS.block.md:145` | `- The ONE door for any dispatch is \`.bee/bin/bee dispatch prepare` |
| 11 | The bullet already says what the leader must NOT pick — the positive rule goes beside it. | read | `packages/bee/AGENTS.block.md:151` | `  call). Never hand-pick \`subagent_type\`, a \`model\` param, or a` |
| 12 | The delegation contract has one home. | read | `skills/bee-hive/references/gates-and-delegation.md:124` | `### Delegation contract (fan-out: decide-altitude vs gather-altitude)` |
| 13 | The worker brief names transports by shape — the one place a worker itself reasons about transport. | read | `skills/bee-swarming/references/worker-details.md:282` | `- **cli-shaped advisor:** run the given command with the evidence bundle on stdin, reusing the External Executors output-capture discipline.` |
| 14 | On the pi runtime every role is blind today; the measured baseline this feature must move. | ran | `bash /tmp/…/scratchpad/econ.sh` | `pi      code     req=None       eff=None       status=unverified             channel=herding-exec   shape=Bash/herding` |

## Discovery

`derive_economics` (claims 1–2) nulls the model for herding and cli before looking, while the payload builder (3) already holds the agent name and `resolve_agent_command_for_runtime` (4) turns it into an argv whose `--model` token is plain text (5). The two read surfaces are transport-first by construction (7–8). The rule has three rendered homes (9–13); nothing else needs to change for a host to receive it.

## Approach

**Recommended path.**
- **`declared`.** One helper beside the resolver — `declared_model_of(argv: &[String]) -> Option<String>`, reading the value after `--model` or `-m` — used by `derive_economics` for `herding-exec` (argv from the resolved agent) and `cli-exec` (the command's tokens). Found → `requested_model = Some(model)`, status `"declared"`; not found → today's `null` / `"unverified"`, byte-identical. `effective_model` stays `null` for both (bee did not watch it run). The vocabulary test (6) gains the new word.
- **Job first.** `render_role` (7) returns `<model-or-declared> (native|herding: <agent>|cli)` and `dispatch_door_lines` prints `<role>: "<description>" → <that>`; `bee team show` (8) prints `<role>  [<source>]  <description> — <model-or-declared> (<transport>)`. Description missing → the row simply has no description; the order never changes.
- **Three sentences.** `AGENTS.block.md` (10–11): *"Pick the team member by job — the role's description says what it is good at; whether it runs as a native model, a herding pane or a cli is config, never the leader's choice."* The delegation contract (12): the same rule at gather-altitude. The worker brief (13): one clause telling a worker that the shape it was handed is config, not a signal about the work. Then `bee dev regen`, so the block lands in `AGENTS.md` and the parity test (9) stays green.

**SMALLER PATH check.** Skip `declared` and only reorder the text → the roster shows a job with no model on pi, which is the lie the owner named. Skip the three sentences → the mechanism changes and no host is told why. Skip the reorder → `declared` prints beside `herding (…)` and the leader still reads transport first. PASS: each piece is one decision.

**Rejected alternatives.**
- *Widen `pinned` to argv flags* — overclaims; `pinned` means bee controls the param.
- *Inject the model into the worker prompt now* — fix 4 of the research; a separate feature once `declared` exists to inject.
- *A per-transport icon or column* — puts transport back in the leader's eye line; D1 says trailing detail.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `declared` in `derive_economics` | MEDIUM — a published record gains a word; the null path must stay byte-identical | tests: argv with `--model` → declared; with `-m` → declared; without → today's exact JSON (claim 6's bytes) |
| Roster line / `team show` | LOW — display only, but tests assert the old bytes | the asserting tests updated; a `team show` row and a roster line each read job → description → model → transport |
| Three doctrine sentences + regen | LOW | parity test green; each home carries exactly one new sentence |

## Shape

**Feature outcome.** A leader on any host reads *what each team member is for* and *what it will run as*, in that order, on every runtime — and picks by the first.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | `declared` | D2 — the record stops lying on pi | S1 | three economics tests; the pi baseline (claim 14) flips to `declared` |
| E2 | Job-first surfaces | D1 | S1 | updated display tests; live `bee team show` and preamble line |
| E3 | The rule in every host | D3 | S1 | parity test; one sentence per home |

**Slice queue.** One slice, four cells: (1) `declared` + helper + tests; (2) the two surfaces — depends on (1); (3) the three doctrine sentences + regen (wave-barrier) — parallel with (1); (4) knowledge/config-reference sync (`declared` row, B12/B13) — parallel with (1). **Current slice to prepare: S1.**

## Test matrix

| Path | Probe |
|---|---|
| Happy | herding argv with `--model X` → `requested_model: X`, `declared`; the pi econ table shows `declared` on all four roles |
| Happy | cli command with `-m X` → `declared` |
| Edge | argv with no model token → exactly today's bytes (`null`, `unverified`) |
| Edge | native model slot → still `pinned`; session-model → still `inherited-or-unknown` |
| Display | roster line and `team show` row: role, description, model-or-declared, transport — in that order, on claude and pi |
| Docs | parity test green after regen; `rg` finds the new sentence once in each of the three homes |

## Open Questions

- (none)

## Out of scope

- Injecting the declared model into the worker brief (fix 4).
- Any change to which agent a role resolves to, or to the `models` alias.
