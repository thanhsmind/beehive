# real-bee-global-isolation — context

**Feature slug:** real-bee-global-isolation
**Date:** 2026-11-26
**Scope:** Small (test-harness bugfix)
**Domain types:** BUILD

## What was asked

The owner asked to push and release 2.123.0. The release suite went red on
one test, `injected_dispatch_guidance_extracts_and_executes_pi_runtime_herding`
in `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`: the default
Claude dispatch returned tool `Bash`, the test expects `Agent`.

## Cause

The `real_bee` helper in that test file starts the real `bee` binary but
does not isolate the user-global config. The non-test build of
`global_config_dir()` reads `BEEHIVE_CONFIG_DIR`, then `XDG_CONFIG_HOME`,
then `~/.config/beehive`. The host global config routes `team.claude`
roles to agy herding agents, so that layer merged into the fixture and
turned the Agent dispatch into a herding Bash dispatch. The opencode
contract tests and `control-bee` (verify-map-upkeep-3 D1) already set
`BEEHIVE_CONFIG_DIR` to an empty folder for the same reason.

## Locked Decisions

| ID | Decision | Store id |
|---|---|---|
| D1 | The `real_bee` helper sets `BEEHIVE_CONFIG_DIR` to an empty folder for every bee child it starts, so the host user-global config never merges into a test fixture. | 20732f48 |
| C1 | A `pi_plugin_contracts` test proves that a bee child started through `real_bee` ignores a user-global config that sets `team.claude` roles, while that config is present through `XDG_CONFIG_HOME` and `HOME`. | 77dff9a0 |

## Read set

<!-- bee:no-read-set: one test helper; the read set is the failing test and config_layers.rs global_config_dir -->
