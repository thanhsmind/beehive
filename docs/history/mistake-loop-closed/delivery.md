# mistake-loop-closed — delivery

Feature: close the mistake loop end to end — one fix-at ledger, per-cell mistakes door, unattributed and rejudge sweeps, four-word recurrence detection, visible debt surfaces, proven fix-at close/verify/next/promote verbs, `evidence_ref` conformance warnings, and updated doctrine. Decisions are cited as D-numbers from `docs/history/mistake-loop-closed/CONTEXT.md`.

## What shipped per decision

- **D1 (One fix-at ledger):** `packages/bee-rs/crates/bee/src/verbs/fix_at.rs` adds `GroupState` (`Open`, `Closed`, `Recurred`), `Group`, `Ledger::from_rows`, `Ledger::load`, and `Ledger::current_epoch_feature`. `verbs/feedback.rs` (`fix_at_load`) and `verbs/backlog.rs` (`current_epoch_finding_feature`) now read from the ledger instead of duplicating the open/closed rule.
- **D2 (Per-cell mistakes door):** `verbs/mailbox.rs` adds `cells: Vec<String>` to `Entry` (written only when non-empty) and `--cell` (comma-separated) on `bee mailbox reflect` (both reflections and `--no-mistakes`). `verbs/drivers/close.rs` clears only the debt cells named by the closing run's answers and prints both ready commands with `--cell` last on refusal. `generated/registry_payload.json` documents `--cell`.
- **D3 (Session takeup unattributed sweep):** `fix_at::sweep_unfiled`, called from `verbs/work.rs` after `compose_digests_for_finished_periods`, reads all runs via the control root and files unclosed `check`/`architecture` reflections newer than the group's latest close under feature `unattributed` (P3 finding rows, idempotent and fail-open).
- **D4 (Repeated soft labels re-judged):** `sweep_unfiled` groups `doctrine` and `none` reflections by their layer-free four-word key (`mailbox_digest::normalize_shape`, `MIN_SHAPE_WORDS`) and files one `fix-at:rejudge` P3 finding row when two or more distinct runs newer than the key's latest close share it with no current-epoch fix-at group of any layer.
- **D5 (Wording drift caught):** `fix_at::alias_title` compares the four-word key within one layer at both close (`file_fix_at_rows`) and sweep: matching an `Open` or `Recurred` group skips, and matching a `Closed` group files under its exact title (keeping original wording in `detail`) so the group turns `Recurred`.
- **D6 (Visible debt):** `fix_at::debt` summarizes open counts per layer, recurred count, oldest open age in days, and `stale_open_rows` (>30 days). Surfaced in `hooks/session_preamble/budget.rs` (`### Fix-at debt`, registered in `DROPPABLE` with fetch target `bee backlog fix-at next`), `verbs/status_full/orient.rs` (report-only blocker), `verbs/status_full/build.rs` (`fix_at_debt` in `bee status --json`), and `doctor.rs` (non-voting advisory when `stale_open_rows > 20`).
- **D7 (Proven fix-at close):** `bee backlog fix-at close --layer --title --guard --guard-cmd --red-proof [--commit]` runs `--guard-cmd` from the repo root (600 s timeout), refuses non-zero exits or empty proof fields, and appends a `backlog-closed` row with proof metadata (`--kept <reason>` allowed on `fix-at:rejudge`). `bee backlog add` refuses `backlog-closed` on a fix-at layer.
- **D8 (Closed guards re-run):** `bee backlog fix-at verify` re-runs `guard_cmd` for every `Closed` group, appends a finding row and exits non-zero when a guard fails (turning the group `Recurred`), skips guardless closes as unverifiable, and logs a `fix-at-upkeep` audit row.
- **D9 (Puller and upkeep):** `bee backlog fix-at next` orders `Recurred` first, then findings count, then oldest age; `bee backlog fix-at promote` files one `Guard: <title>` PBI (WIP limit 1); session takeup auto-promotes through the control root when open debt exists and no fix-at PBI is open. `bee upkeep due` adds kind `fix-at` (skill `null`, command pair `verify` + `promote`, due when a `Closed` group exists and no `fix-at-upkeep` audit row is younger than 7 days).
- **D10 (`evidence_ref` warnings):** `verbs/knowledge/check.rs` warns `evidence_without_ref` when `bee.evidence` is `wired` or `exercised` with no `bee.evidence_ref`, and `evidence_ref_missing_path` for path tokens (with `/` and file extension) missing under repo root, `packages/bee-rs/crates/bee/src`, and `packages/bee-rs/crates/bee`.
- **D11 (Doctrine):** `packages/bee/LEADER.md`, `packages/bee/AGENTS.block.md`, `AGENTS.md`, `expertise/static-rules.md`, `skills/bee-capturing/references/promotion.md`, `packages/bee-rs/crates/bee/tests/instruction_laws.rs`, and `docs/history/codex-harness-hardening/release-manifest.json` updated via `bee dev regen`.
- **D12 (Plan-check refinements):** Layer-matched vs layerless closes, newer-than-close counting for `--kept` rejudge holds, shared `normalize_shape` four-word key, row-counted >30d doctor advisory, unverifiable skip for guardless closes, and control-root sweep/auto-promote all shipped in `mlc-1`..`mlc-5`.

## Proof per cell

| Cell | Commit | Verify / proof |
|---|---|---|
| `mlc-1` | `e2eab8e9b` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at` (plus `feedback` and `backlog` filters) |
| `mlc-1t` | `ce6901506` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_ledger` (9 tests, each shown red once) |
| `mlc-2` | `18d34f4e5` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- mistakes` |
| `mlc-2t` | `7f81860de` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- mistakes_door_` (6 tests, each shown red once) |
| `mlc-3` | `5526639ed` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at` |
| `mlc-3t` | `103f84656` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_intake` (9 tests, each shown red once) |
| `mlc-4` | `5590deacc` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at` |
| `mlc-4t` | `e87a07b46` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_debt` (8 tests, each shown red once) |
| `mlc-5` | `efd70c327` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- backlog` |
| `mlc-5t` | `d425f30c4` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_verbs` (9 tests, each shown red once) |
| `mlc-6` | `ca5b0ae62`, `d2966e682` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- knowledge` |
| `mlc-6t` | `fd9b3811b` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- evidence_ref_check_` (7 tests, each shown red once) |
| `mlc-7` | `6910acaa7` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws` + `bee dev release-manifest --check` |
| `mlc-7c` | `87ebbfaa4` | `pointer_integrity` (10), `rule_index_parity` (5), `agents_block_render_parity` (1), `instruction_laws` (10), `release-manifest --check` (`docs/history/mistake-loop-closed/reports/mlc-7c.md`) |

## Concepts synced

- `docs/knowledge/areas/human-mailbox/overview.md` — rules 17 and 20 for `mistake-loop-closed D2, D3, D4, D5`.
- `docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md` — `Ledger::from_rows`, `fix-at close/verify/next/promote`, and `bee upkeep due` kind `fix-at` (`mistake-loop-closed D1, D7, D8, D9`).
- `docs/knowledge/areas/okf-profile/conformance-check.md` — `evidence_without_ref` and `evidence_ref_missing_path` warning rows (`mistake-loop-closed D10`).
- `docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md` — `### Fix-at debt` block and `DROPPABLE` registration (`mistake-loop-closed D6`).

## Known limits

- The installed `.bee/bin/bee` binary reflects these changes once rebuilt and reinstalled after merge, at which point trigger `more-than-20-backlog-rows-with-layer-fix__6e79d14e` is resolved with the `doctor` advisory named as its outcome.
