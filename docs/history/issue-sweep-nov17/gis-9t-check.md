# gis-9t check: dispatch prepare reports an undeclared herding agent as not ready (issue 164)

Checked: cell gis-9, commit aa8c29eb5, decision D10 (2bcf3437). The tester did not use the gis-9 worker's context.

Verdict: both truths PASS. An edge case FAILS: gis-9 introduced a regression for the built-in herding agents. The tester reports it here and did not fix it.

## Reproduction

The team slot `{"kind": "herding", "agent": "ghost-worker"}` is used with `herding.agents = {normal-worker, other-worker}` and `herding.transport = "herdr"`.

- Parent commit aa8c29eb5^: the payload is `transport_ready` false with the reason `HERDR_ENV is not set — this session is not inside a herdr pane`. The reason does not name the unknown agent. Inside a herdr pane the same call reported true.
- Fixed code: the payload is `transport_ready` false with the reason `unknown herding agent "ghost-worker" — herding.agents declares: normal-worker, other-worker`.

The fix is a match arm at `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3063`. It uses the helpers at `prepare.rs:1674-1691`.

## Truths

| Truth | Result | Anchor |
|---|---|---|
| an undeclared herding agent gives transport_ready false naming the declared agents | PASS | `prepare.rs:7150` (gis-9 test), `prepare.rs:7203` (tester edge test: no `herding.agents` map, so the reason names `none`) |
| a declared herding agent is unchanged | PASS | `prepare.rs:7178` (`non_paseo_agent_keeps_today_probe_byte_identical`: same ready and reason as the transport probe) |

## Edge case that fails: built-in agents

`herding run` resolves an agent name through `agent_registry` at `packages/bee-rs/crates/bee/src/herding/wave.rs:313`. That registry starts from `built_in_agents()` at `wave.rs:190`, which holds `claude-sonnet` and `agy-flash`, and then overlays `herding.agents`. The gis-9 check in `declared_herding_agents` (`prepare.rs:1674`) reads only the config keys. As a result, a team slot that names `claude-sonnet` and runs fine is now reported as not ready.

Test `built_in_herding_agent_keeps_today_probe` (`prepare.rs:7225`) checks this case:

- On the parent: ok.
- On the fixed code: FAILED. left `unknown herding agent "claude-sonnet" — herding.agents declares: normal-worker`, right `HERDR_ENV is not set — this session is not inside a herdr pane`.

The test is committed with `#[ignore]`, so the suite stays green and the repro stays in the tree. The fixer removes the ignore. A likely fix is to check the agent against the same registry that `herding run` uses. That fix is out of scope for this cell.

A second, smaller mismatch: `declared_herding_agents` falls back to a top-level `agents` key, but `agent_registry` does not read that key. An agent declared only there passes the new check, and `herding run` then rejects it. The tester did not write a test for this case.

## Red on the parent commit

The tester removed the arm at `prepare.rs:3063` from the working copy, ran `cargo test --release --bin bee -- verbs::drivers::prepare::paseo_probe_tests`, and then restored the file. Result: `6 passed; 2 failed`.

- `undeclared_herding_agent_not_ready_names_declared_agents` FAILED: the reason `HERDR_ENV is not set — ...` lacks `ghost-worker`.
- `undeclared_herding_agent_with_no_agents_map_names_none` FAILED: same reason, same cause.
- `built_in_herding_agent_keeps_today_probe` ok.

Outside a herdr pane the transport probe already gives false on the parent. So in this environment the `transport_ready` assertion alone does not tell the parent and the fix apart; the reason assertions do. Truth 2 checks that existing behavior is kept, so its test passes on both commits.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::prepare`

`test result: ok. 77 passed; 0 failed; 2 ignored; 0 measured; 5330 filtered out`

New tests in the run:

- `undeclared_herding_agent_with_no_agents_map_names_none ... ok`
- `built_in_herding_agent_keeps_today_probe ... ignored, gis-9t: gis-9 reports built-in agent claude-sonnet as undeclared`
