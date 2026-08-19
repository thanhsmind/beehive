# herding-executor — Plan (scope A)

**Lane:** standard (flags: public-contracts, covered-contract-change, multi-domain; ~5 product files)
**Source:** docs/history/herding-executor/CONTEXT.md (D1–D9, cited below)
**Date:** 2026-08-20 (rev 2 — after independent plan review, 13 findings applied)

## Shape

One new native verb, `bee herding run`, built as a walking skeleton in slice 1:
dispatch one bee-ignorant external agent into a pane, wait on a file mailbox,
return one structured result. Everything the verb needs that is independently
testable is split out as pure modules first.

## Slice 1 — walking skeleton (cells now)

| Cell | What | Files (product) | Decisions |
|---|---|---|---|
| hx-1 | Mailbox contract module: path layout `.bee/mailbox/<job-id>/`, self-contained brief renderer (task, absolute paths, constraints, result schema, tmp-rename gesture), `result-N.json` parse + schema validation. Pure functions, unit-tested. | `packages/bee-rs/crates/bee/src/herding/mailbox.rs` (new), `herding.rs` (mod wiring) | D3, D4 |
| hx-2 | Drop the agent-kind allow-list: `resolve_agent_command` passes token 0 through — this reshapes `AgentCommandError` (its one variant + Display carry the `supported` list) while the empty-token fail-closed arm keeps a variant; update the wave.rs tests asserting `UnrecognizedKind`. Same decision's doc drift lands here too: `skills/bee-herding/references/operational-invariants.md:74-76` (typed-error sentence) and the caller-obligation comment `crates/fleet/src/backend/herdr.rs:150-161`. | `packages/bee-rs/crates/bee/src/herding/wave.rs`, `skills/bee-herding/references/operational-invariants.md`, `packages/bee-rs/crates/fleet/src/backend/herdr.rs` (comment only) | D2 |
| hx-3 | Write-guard carve: add `.bee/mailbox/` to the scratch-shape exemption list (`write_guard/guards.rs:118` `under_any([...])`) — the real deny is the scratch-shape guard firing on the tmp-rename staging file (`.tmp` suffix, guards.rs:150), which would block exactly D3's write gesture. Tests both ways: `.bee/mailbox/<job>/result-1.json.tmp` allowed, `.bee/other/x.tmp` still denied. | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs`, `write_guard/tests.rs` | D8 |
| hx-4 | The verb `bee herding run`: flags (`--task`/`--task-file`, `--cwd`, `--job-id`, `--idle-timeout`, `--ceiling`, `--close-always`, `--main-root`, `--json`, `--dry-run`); writes `job.json`; spawns via `herdr pane split` + `agent start` (reusing `herding.agent_command` + `HerdrBackend` argv builders); native poll loop — stat `result-N.json`, `log.txt` mtime, `herdr agent list` — idle-timeout kill + absolute ceiling; pane close on valid result, kept on failure, `--close-always` override; appends the `dispatch.jsonl` row and a wave-ledger `record-worker` row; emits the result JSON on stdout. Registry: hand-edit `generated/registry_payload.json` (regen does NOT write it; `herding.run` joins the 12 `herding.*` entries) with an `examples[0]` that executes green in the registry test's scratch tempdir without herdr — the `--dry-run` form (renders `job.json` + brief, spawns nothing) is that example. Gitignore: add `.bee/mailbox/` to `GITIGNORE_BLOCK_PATTERNS` (`onboard/templates.rs:24-54`, order load-bearing) and re-apply the block. Seam-tested without a real herdr (FakeBackend pattern). | `packages/bee-rs/crates/bee/src/herding/run.rs` (new), `herding.rs` (dispatch table), `generated/registry_payload.json`, `onboard/templates.rs`, `.gitignore` (rendered) | D1, D2, D5, D6, D9 + occupancy resolved: record-worker row = yes |
| hx-5 | Docs: `bee herding run` documented in `skills/bee-herding/references/operational-invariants.md` (the canonical home of `herding.agent_command` — NOT a net-new herding section in config-reference.md) + bee-herding knowledge area sync (fourth shape gains the run verb) + one doctrine paragraph beside the cli gather branch naming the herding execution branch. | `skills/bee-herding/references/operational-invariants.md`, `docs/knowledge/areas/bee-herding/overview.md`, `skills/bee-hive/references/gates-and-delegation.md` | D7 |

Dependencies: hx-1, hx-2, hx-3 disjoint → parallel (hx-2's operational-invariants.md edit is a different section from hx-5's — hx-5 runs after anyway). hx-4 after hx-1 (shared `herding.rs`). hx-5 after hx-4 and after hx-2 (shared operational-invariants.md).

Proof per cell: `cargo test` scoped to the touched module; **hx-4 additionally names `--test registry_dispatch`** (integration test outside module scope — asserts every served `try_native` arm is declared in the payload and executes `examples[0]`); hx-5 = knowledge index check + pointer parity.

## Slice 2 — headlines only (no cells yet)

- Follow-up rounds from swarming: drive `result-2..N` via `herdr agent prompt` for blocked results.
- bee-swarming skill wiring — including D4's second half (ALL bee bookkeeping done by the orchestrator after reading the result): when/how the orchestrator picks `bee herding run` over a bee-build dispatch (scope A = by hand on user request).
- Mid-run interactive stalls: measure what herdr status exposes per kind; decide if idle-timeout suffices (CONTEXT open question).
- Scope B: `{kind:"herding"}` tier kind (backlog proposal, gated on A green).

## Smaller-path check

Could hx-1 fold into hx-4? Yes, but the brief renderer and result validation are the protocol's contract — pure, parallel-buildable, and the pieces slice-2 reuses. Splitting costs one cell header and buys parallelism + focused tests. Could hx-5 drop? No — public verb without docs violates the knowledge sync rule. Shape stands: 5 cells, no cheaper honest shape found.

## Rollback

Every cell is additive (new verb, new module, one exemption token, one gitignore pattern, docs). Rollback = revert the worktree merge commit; no data, no migration, no config break (absent `herding.agent_command` keeps its default). Mailbox dirs are gitignored runtime data — deleting them loses nothing durable (results are consumed into cells/dispatch rows at read time).
