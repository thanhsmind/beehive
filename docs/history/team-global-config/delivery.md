# Delivery — `team-global-config`

**Feature:** `team-global-config`  
**Context:** [CONTEXT.md](CONTEXT.md)  
**Plan:** [plan.md](plan.md)

## What shipped

`bee` now reads configuration from four files on top of the built-in default, in precedence order (lowest to highest): built-in default, user-global config (`$BEEHIVE_CONFIG_DIR/config.json`, else `$XDG_CONFIG_HOME/beehive/config.json`, else `~/.config/beehive/config.json`), `.bee/config.json`, `.bee/team.json`, and `.bee/config.local.json` (D1, D2, D3, D5).

- **`.bee/team.json` (D1):** Holds only `team`, `herding.agents`, and `herding.agent_command` at the same key paths as `.bee/config.json`. Other keys are ignored at read time and flagged by `bee config validate`.
- **User-global config (D3, D4, D12):** Read-only in `bee`; only `waggledance` writes it. `guards`, `gate_bypass`, and `hooks` are dropped from the global layer at read time and flagged by `bee config validate`.
- **Write guard (D6, D12, D13, D14):** Covers `.bee/team.json` in the path gate and the layered merged-view comparison on top of the on-disk global layer, refuses any agent write to the global config file or shell assignment of `BEEHIVE_CONFIG_DIR` or `XDG_CONFIG_HOME`, and enforces both D12 and D6 in herded worker panes (`BEE_HERDING_WORKER=1`) via `write_guard::run_herded` (`hooks/write_guard/worker_tree.rs`) while keeping every other guard open there.
- **CLI and onboarding (D7, D8, D9, D11):** `bee config get --global`, layer-aware `explain` and `validate`, `bee config split-team [--drop-global]`, and first-onboard seeding of `.bee/team.json` (omitting keys already set globally) plus the `split-team` onboarding notice.

## Concepts changed

- [`docs/config-reference.md`](../../config-reference.md): Added the config layers and precedence section, updated `bee config` verb descriptions (`get --global`, `explain`, `validate`, `split-team [--drop-global]`), and updated `dispatch.thin_peer_boot` and `team` default notes.
- [`docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md`](../../knowledge/areas/hook-runtime/write-guard-request-shapes.md): Documented the ordered layer list in `config_layers.rs`, `GLOBAL_FORBIDDEN_KEYS`, `TEAM_FILE_KEYS`, the single post-merge `models`-to-`team` fold, and `no_code_comments` reading the layered view.
- [`docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md`](../../knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md): Updated `R42` for `.bee/team.json`, the on-disk global layer, the D12 global-config write refusal, and D14 herded worker-pane enforcement in `write_guard::run_herded`.
- [`docs/knowledge/areas/rust-runtime/command-surface.md`](../../knowledge/areas/rust-runtime/command-surface.md): Documented `R11a` (`get --global`, `explain`, `validate`, `split-team [--drop-global]`, `team.json`-first atomic writes) and the `catalog.rs` distinct-flag count bump (`250` to `252`).
- [`docs/knowledge/areas/onboarding/host-project-artifacts.md`](../../knowledge/areas/onboarding/host-project-artifacts.md): Documented `seed_files` splitting `.bee/team.json`, global-omission rules, `split_team_notices`, and `roster_source`.
- [`docs/knowledge/areas/bee-herding/thin-peer-boot.md`](../../knowledge/areas/bee-herding/thin-peer-boot.md): Updated `dispatch.thin_peer_boot` and herding config reads to use the main checkout's merged layered view and worktree `team.json` bootstrap.

## Known limits

- **Legacy `models` warnings:** `doctor.rs` and `hooks/session_preamble/budget.rs` still read `.bee/config.json` directly because they warn specifically about the legacy `models` key in that file.
- **`waggledance` writer:** `bee` only reads the user-global config file (D12). The writer side in `waggledance` is a separate request to the `waggledance` leader.
- **`cross_project.rs`:** Still reads `guards.*` from `.bee/config.json` directly; D4 forbids `guards` in the global layer and D1 excludes it from `.bee/team.json`.

## Rollout and undo

`beehive`'s own `.bee/config.json` is not split in this feature branch. Run the rollout in this exact order (D13):

1. Merge `wt/team-global-config` into `main`.
2. Build and reinstall `.bee/bin/bee` on `main` **first**, and confirm the installed binary recognizes `bee config split-team --help`. Running `split-team` before reinstalling `.bee/bin/bee` would leave an older two-file binary reading a split config.
3. Run `bee config split-team` in `beehive`, verify `bee team show` is unchanged, and commit `.bee/config.json` and `.bee/team.json` together.

**Undo:** Run `git revert` on the split commit to restore `.bee/config.json` and remove `.bee/team.json`.
