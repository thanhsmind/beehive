# Team config in its own file, and a user-global config layer — Context

**Feature slug:** team-global-config
**Date:** 2026-11-16
**Shaping session:** complete (owner chose 1B, 2A, 3A on 2026-11-16)
**Scope:** High-risk (touches the write guard and the governed-key view)
**Domain types:** RUN | READ

## Feature Boundary

Today bee reads config from two files: tracked `.bee/config.json` and the
per-machine `.bee/config.local.json`, the overlay winning. The team table
(`team`, `herding.agents`, `herding.agent_command`) is more than half of
`config.json`, and the owner repeats the same table in every project. This
feature moves the team table into its own tracked file, `.bee/team.json`,
and adds one user-global config file read by every project, so a value
the owner wants everywhere is set once. Existing projects keep working
unchanged; onboarding offers the move. The feature ends there: no new
config keys, no change to what any key means.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `.bee/team.json` is a tracked file that holds `team`, `herding.agents` and `herding.agent_command`, at the same key paths they have in `config.json` (decision 465ea1e4). Any other key in `team.json` is a `bee config validate` problem and is not read. | Same paths means one deep merge, no translation. |
| D2 | Precedence, lowest to highest: built-in default, user-global config, `.bee/config.json`, `.bee/team.json`, `.bee/config.local.json`. The highest layer that sets a key wins (decision 21aebfc1). A key set in both `config.json` and `team.json` is a validate problem (duplicate) and `team.json` wins. | One ordered list in one place. |
| D3 | The user-global config file is `$BEEHIVE_CONFIG_DIR/config.json` when that env var is set and non-empty, else `$XDG_CONFIG_HOME/beehive/config.json`, else `~/.config/beehive/config.json`. An absent file reads as an empty layer. | Matches the existing `~/.config/beehive` home of `performance.jsonl` (hooks/session_close/perf.rs:38-46); the env var makes tests hermetic. |
| D4 | The global layer may set any key except `guards` (and everything under it), `gate_bypass` and `hooks` (decisions 16275c9a, 65ef1514). Those keys are dropped from the global layer at read time, and `bee config validate` reports each one present as a problem. | One global line would auto-approve gates, open the cross-project guard, or turn the write guard off in every project. |
| D5 | One reader. Every merged-config read goes through one function that builds the ordered layer list and merges it. The strict reader (`state.rs` `read_config_raw`) and the fail-open readers (`hooks/compaction.rs` `read_config_failopen`, `hooks/session_preamble/state.rs` `read_config_raw_open`) differ only in corrupt-file policy, never in layer list. Direct reads of `config.json` for config VALUES (`hooks/write_guard/cross_project.rs`, `hooks/session_preamble/budget.rs`, `verbs/detector.rs`) move to it. Reads of `config.json` as a FILE (onboard text merge, report) stay. A corrupt layer warns and reads as empty, as today. | A layer added in one reader and not the others is the defect this rules out. |
| D6 | The governed-key guard covers `.bee/team.json` exactly as it covers `.bee/config.json`: the path gate and the merged-view comparison include it. An agent write to the global config file is refused outright (D12). `bee config set/unset` keep refusing a governed key in every layer. | A new file must not be a side door around the guard. |
| D7 | `bee config` gains the global layer, read-only (D12): `get` takes `--global`; `set` and `unset` have no global form; `explain` names the source layer as one of `global`, `tracked`, `team`, `local`, `unset` and prints each layer's value; `validate` checks all four files (parse, team.json foreign keys, duplicates between config.json and team.json, forbidden global keys, team problems on the merged view) and lists the keys the global layer supplies. `--global` with `--local` is refused. | |
| D11 | When the global config sets `team`, a first onboard seeds no team table into the project. `bee config split-team --drop-global` also removes each project team-table leaf whose value equals the global value, so the project keeps only its differences (decision 8a66d22b). | Every project seeds a full table and the project wins, so without this a global team value never applies. |
| D12 | Only waggledance writes the global config file (decision d628826a). bee reads it and never writes it. The write guard refuses any agent write to it: Edit, Write, apply_patch, a Bash command that names its path, and a Bash command that assigns `BEEHIVE_CONFIG_DIR` or `XDG_CONFIG_HOME`. The waggledance side is a separate request to the waggledance leader. | One change to the global file changes every project. |
| D13 | Plan-check refinements. The `models`-to-`team` fold runs once after the merge, as today. The guard's merged comparison includes the on-disk global layer, so an agent cannot add a project slot that shadows a global one; an unparseable team.json reads as an absent layer, so it diffs as a removal. split-team writes team.json first, then config.json, and its output says to commit both files together. Worktree bootstrap copies team.json beside config.json. Every direct read of a config value in the herding, paseo, report and doctor code moves to the layered reader. The beehive split runs after the new binary is installed. | Hat wave findings, `reports/hat-wave.md`. |
| D8 | `bee config split-team` moves `team`, `herding.agents` and `herding.agent_command` from `.bee/config.json` into `.bee/team.json` verbatim. The merged view is byte-identical before and after, so the governed comparison passes. It is idempotent: a second run is a no-op with exit 0. If `team.json` already sets a key that `config.json` also sets with a different value, it refuses with zero change and names the key. | A move that keeps the merged view equal is not a governed change. |
| D9 | Onboarding: a first onboard seeds the team table into `.bee/team.json`, not `config.json`. An onboard of a project whose `config.json` still holds the team table adds one notice that offers `bee config split-team`; it never moves it on its own. | Existing projects keep working; the move is the owner's choice. |
| D10 | beehive runs on `.bee/team.json` at the end of this feature (split-team run on this repo), and `docs/config-reference.md` plus the owning knowledge concept describe the layers. | Dogfood proves the move on the largest real table. |
| D14 | A herded worker pane (`BEE_HERDING_WORKER=1`) gets the D12 global-config write refusal and the D6 team.json governed check: `write_guard::run_herded` applies both beside its concurrent-tree check (decision a0f6985d). Every other guard stays open in that pane. | tgc-6 drove a herded pane and wrote the global file with no refusal; that file changes every project. |
| C1 | Contract config-layers: a pure merge takes the ordered layer list and returns the merged object; the layer builder reads the four files from a repo root and a global dir. Proving tests carry `config_layers` in their names: precedence for each pair of adjacent layers, the forbidden global keys dropped, team.json foreign keys ignored, a corrupt layer read as empty, `models` and `team` across layers giving today's result. | |
| C2 | Contract team-guard: a governed change in `.bee/team.json` is refused like one in `config.json`; a split that keeps the merged view equal is allowed; any agent write to the global config file is refused. Proving tests carry `team_guard` in their names. | |
| C3 | Contract config-verbs: `explain` reports the right source layer for a key set in each layer; `validate` reports each problem kind of D7; `--global` reads and writes only the global file; `split-team` moves, is idempotent, and refuses a conflicting value. Proving tests carry `config_global` or `split_team` in their names. | |

### Agent's Discretion

- The Rust type that names a layer, provided the order lives in one place.
- The exact wording of the onboarding notice and the validate problems, provided each names the file and key.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| layer | One config file in the precedence list of D2. |
| team table | `team`, `herding.agents` and `herding.agent_command` together. |
| global config | The user-global config file of D3. |

## Read set

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/state.rs` | `read_config_raw`, `merge_config_overlay`, `hook_enabled`. |
| `packages/bee-rs/crates/bee/src/hooks/compaction.rs` | `read_config_failopen`, shared with session_init. |
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs` | `read_config_raw_open`. |
| `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` | `fold_team_key`, `normalize_models`. |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs` | Governed view (1086-1147), diff (1248+), path gates (93, 1437). |
| `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` | Merged-view comparison (650-720), hook-enabled check (226). |
| `packages/bee-rs/crates/bee/src/verbs/config.rs` | `get/set/unset/explain/validate`, `governed_refusal`. |
| `packages/bee-rs/crates/bee/src/onboard/templates.rs`, `onboard/plan.rs` | Team seed and notices. |

### Knowledge

| Concept | Why it matters |
|---------|----------------|
| `docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md` | Why the guard compares merged views. |
| `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md` | Config read and overlay for the write guard. |
| `docs/knowledge/areas/bee-herding/thin-peer-boot.md` | Herding reads the main checkout's merged view. |
| `docs/knowledge/areas/rust-runtime/command-surface.md` | Rule R11: config changes go through `bee config`. |
