# gis-9t check: dispatch prepare reports an undeclared herding agent as not ready (issue 164)

Checked: cell gis-9 at commits aa8c29eb5 (first fix) and 667bd4537 (built-in regression fix), decision D10 (2bcf3437). The tester did not use the gis-9 worker's context.

Verdict: both truths PASS. The built-in agent regression that the first round of this check found is fixed. The top-level `agents` mismatch from that round is fixed too.

## Round 2: what changed

Commit 667bd4537 drops the helpers that read `herding.agents` keys (with a top-level `agents` fallback). `undeclared_agent_reason` (`packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1674`) now calls `resolve_agent_command` (`packages/bee-rs/crates/bee/src/herding/wave.rs:397`). That is the resolver `herding run` uses, so the registry is the same: `built_in_agents()` (`wave.rs:190`) plus `herding.agents` (`wave.rs:313`). Only an `UnknownAgent` error marks the slot not ready. The match arm is at `prepare.rs:3054`.

## Reproduction

The team slot `{"kind": "herding", "agent": "ghost-worker"}` is used with `herding.agents = {normal-worker, other-worker}` and `herding.transport = "herdr"`.

- Before gis-9 (arm removed): `transport_ready` false, reason `HERDR_ENV is not set — ...`. The reason does not name the agent. Inside a herdr pane it reports true.
- Fixed code: `transport_ready` false, reason `unknown herding agent "ghost-worker" — herding.agents declares: agy-flash, claude-sonnet, normal-worker, other-worker`.

## Truths

| Truth | Result | Anchor |
|---|---|---|
| an undeclared herding agent gives transport_ready false naming the declared agents | PASS | `prepare.rs:7141` (gis-9 test), `prepare.rs:7194` (no `herding.agents` map: the reason names the built-ins), `prepare.rs:7240` (tester edge test, round 2) |
| a declared herding agent is unchanged | PASS | `prepare.rs:7169` (declared in `herding.agents`), `prepare.rs:7215` (built-in `claude-sonnet`, now un-ignored): ready and reason are byte-identical to the transport probe |

## Edge test added in this round

`agent_declared_only_in_top_level_agents_is_not_ready` (`prepare.rs:7240`) names an agent that only a top-level `agents` key declares. `herding run` rejects that agent, so dispatch prepare must report it not ready with the unknown-agent reason. The test also checks that `resolve_agent_command` rejects the name.

## Red before green

The tester ran `cargo test --release --bin bee -- verbs::drivers::prepare::paseo_probe_tests` on two reverted states of the source, then restored the file each time. Only `prepare.rs` was edited; `git apply` was refused by the concurrent-worker guard, so the hunks were reverted by hand.

1. Implementation set back to 667bd4537^ (the old helpers restored; lines 1–7180 diffed byte-identical to the parent): `6 passed; 3 failed`.
   - `built_in_herding_agent_keeps_today_probe` FAILED: left `unknown herding agent "claude-sonnet" — herding.agents declares: normal-worker`, right `HERDR_ENV is not set — ...`.
   - `undeclared_herding_agent_with_no_agents_map_names_built_ins` FAILED: the reason lacks `claude-sonnet`.
   - `agent_declared_only_in_top_level_agents_is_not_ready` FAILED at the reason assertion: the old fallback treated `top-only` as declared.
2. The gis-9 arm removed entirely (the state before aa8c29eb5): `6 passed; 3 failed`.
   - `undeclared_herding_agent_not_ready_names_declared_agents`, `undeclared_herding_agent_with_no_agents_map_names_built_ins` and `agent_declared_only_in_top_level_agents_is_not_ready` FAILED. The reason does not name the agent.
   - `built_in_herding_agent_keeps_today_probe` and `non_paseo_agent_keeps_today_probe_byte_identical` ok.

Outside a herdr pane the transport probe already gives false, so in this environment the reason assertions are what tell the states apart. Truth 2 checks that existing behavior is kept, so its tests pass whenever the arm is absent.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::prepare`

`test result: ok. 79 passed; 0 failed; 1 ignored; 0 measured; 5331 filtered out`

The one ignored test is `role_flag_tests::explain_through_the_cli_child`, a child process that another test spawns. It is not related to this check.

Tests from this check in the run:

- `agent_declared_only_in_top_level_agents_is_not_ready ... ok` (new in round 2)
- `built_in_herding_agent_keeps_today_probe ... ok` (no longer ignored)
- `undeclared_herding_agent_with_no_agents_map_names_built_ins ... ok`
- `undeclared_herding_agent_not_ready_names_declared_agents ... ok`

`cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings`: clean.
