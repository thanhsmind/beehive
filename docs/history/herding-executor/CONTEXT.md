# Herding Executor — Context

**Feature slug:** herding-executor
**Date:** 2026-08-19
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

A native verb `bee herding run` starts one external CLI agent (any herdr-supported kind) in a fresh pane, hands it a fully self-contained brief, waits on a file mailbox with health-check liveness at zero token cost, and returns one structured result — making a foreign agent usable as a cell-execution worker the way an in-family subagent is today. It ends before any `models.*` config wiring (scope B, backlogged).

## Locked Decisions

Decision-log ids are cited beside each row (`bee decisions search`).

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (de911edd) | Ship the native verb `bee herding run` first (scope A); the `{kind:"herding"}` tier kind in `models.*` is a separate backlog item (scope B) gated on A running real cells green | B is unbuildable until A exists: `wave` cannot create panes, returns no transcript, hardcodes sonnet |
| D2 (851d79fd) | Drop bee's own agent-kind allow-list (`SUPPORTED_AGENT_KINDS`, wave.rs:67); token 0 of the agent command passes straight to herdr, which validates `--kind` itself (21 kinds as of herdr 0.8.0) | A mirrored list is already drifting 2/21 and has no machine-readable source. Accepted cost: an unknown kind is refused after the pane split, not before. Revisits herding-orchestration D14 — the typed error survives, sourced from herdr's refusal |
| D3 (b1b1a708) | Completion travels through a file mailbox `.bee/mailbox/<job-id>/` — `job.json`, round-numbered `result-N.json`, `log.txt` — written tmp-then-rename; the result file's appearance IS the done signal | Screen scraping is a guess per agent kind; an atomically renamed JSON file is exact, schema-checkable, identical across all 21 kinds. Round numbering keeps the done signal valid across follow-up prompts. The existing wave/Baseline path stays untouched |
| D4 (25f20a1b) | The worker stays bee-ignorant: the dispatch prompt is fully self-contained (task, absolute paths, file constraints, result schema, the tmp-rename write gesture); ALL bee bookkeeping (cells finish, proof line, reservations, dispatch row) is done by the orchestrator after reading the result | Any of 21 kinds may have never seen bee; state authority stays in one place |
| D5 (3bc0dceb) | Liveness is health-check based, two layers, native in the Rust verb: heartbeat (`log.txt` mtime, worktree diff activity, `herdr agent list` status) with `--idle-timeout`, plus a high absolute `--ceiling` as the busy-loop backstop; no fixed short wall-clock timeout | Wall-clock cannot tell a long cell from a stuck agent; heartbeat alone misses the infinite fix-test-fix loop. Polling is syscalls in Rust: zero tokens; the orchestrator dispatches via background Bash and receives one completion notification |
| D6 (f6c0a81d) | Pane lifecycle: a valid result closes the pane (`herdr pane close`); failure or timeout keeps it open as forensics; `--close-always` closes in every outcome | A dead foreign agent's pane is the only remaining trace |
| D7 (5120eaae) | The herding executor is cell-execution-only — the mirror of the `cli` tier kind (gather/review/advisor-only); a gather never dispatches through a herding pane | When scope B lands, `kind=herding` with a gather purpose falls back to the default, inverting the existing `for_gather` branch (models.rs:278) |
| D8 (22dd6bc2) | The write-guard hook must admit worker writes to `.bee/mailbox/`; the exact carve (guard allowlist vs mailbox location) is an open planning question | Without it the mailbox works for 20 foreign kinds and fails exactly for bee-hooked claude workers |
| D9 (5db8f358) | `bee herding run` itself appends the `dispatch.jsonl` row for every run it starts | Closes the named zero-dispatch-rows gap for the Bash-launched executor path, mechanically |

### Agent's Discretion

- Exact `result-N.json` schema fields (beyond `status: done|blocked`, `summary`, `files_changed[]`, `proof`), flag names, and default values for `--idle-timeout`/`--ceiling`.
- Whether the verb reuses `HerdrBackend` internals or drives `herdr` argv directly — behavior above is what's locked, not the plumbing.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| mailbox | `.bee/mailbox/<job-id>/` — the file channel between orchestrator and worker; never the pane screen |
| result file | `result-N.json`, atomically renamed into place; its existence is the completion signal for round N |
| heartbeat | Any of: `log.txt` mtime advancing, worktree diff changing, herdr status `working` |
| bee-ignorant | The worker needs zero bee knowledge; its whole contract is "do the task, write one JSON file" |
| scope A / scope B | A = the `bee herding run` verb (this feature). B = `{kind:"herding"}` in `models.*` (backlogged proposal) |

## Specific Ideas And References

- User's target topology: claude runs as the main agent (brain), cheap external agents do the work — e.g. `agy` running gemini-flash. Gather-shaped cheap work is already served today by the `cli` tier kind (`.bee/config-sample-cli-executors.json`); this feature covers the write-work half.
- Three-transport model settled in session: **subagent** = in-family, harness-managed; **cli** = one-shot stdin→stdout, read-only; **herding** = long-lived foreign agent in a pane with a worktree, write work.

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/fleet/src/backend/herdr.rs` — argv-safe herdr driver (`run_herdr`, `agent start/prompt/read` builders, spill-file pattern)
- `packages/bee-rs/crates/bee/src/herding/wave.rs` — config reading (`agent_command_tokens`), main-root resolution, ledger append
- `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs` — occupancy/worker recording (`record-worker` is the post-spawn gesture)

### Established Patterns

- Config-driven argv templates, per-token substitution, never joined-and-resplit (i54-closeout D4) — reuse for the agent command
- Fail-open template reads (`read_command_template_tokens`, herding.rs:556) — malformed config falls back to default

### Integration Points

- `packages/bee-rs/crates/bee/src/herding.rs:79-97` — the `try_native` dispatch table gains `run`
- `.bee/config.json` `herding.agent_command` — the existing spawn-command seam the verb reads
- `skills/bee-swarming` — the Execute stage is where the verb gets called (by hand in scope A)
- Write-guard hook (D8) — needs the mailbox carve

## Canonical References

- `docs/knowledge/areas/bee-herding/overview.md` — cockpit boundaries this feature must not violate (merge stays a human gesture; dispatch interlock untouched)
- `skills/bee-hive/references/gates-and-delegation.md:132` — the cli gather branch this executor mirrors
- `docs/history/herding-orchestration/CONTEXT.md` — D14 (revisited by D2 here), D17, D18

## Outstanding Questions

### Resolve Before Planning

- (none)

### Questions Handed To Planning

- [x] D8 mechanism (resolved by hx-3): write-guard allowlist path for `.bee/mailbox/` vs relocating the mailbox — measure what the guard actually matches today
- [x] Mid-run interactive stalls (permission prompt, login) beyond start-timeout: what herdr status exposes per kind, and whether idle-timeout suffices as the only net — re-opens on [[trigger:first-real-bee-herding-run-against-a-non__3bc0dceb]]
- [x] Occupancy (resolved by hx-4 — it records): whether `bee herding run` records into the wave ledger (`record-worker`) so cockpit occupancy counts these workers too

## Ideas Out Of Scope (trigger-registered)

- Scope B — `{kind:"herding"}` tier kind in `models.*.generation` (backlogged as a proposal, feature-tagged herding-executor; picks up on [[trigger:scope-a-bee-herding-run-has-executed-rea__de911edd]])
- Herding as a review-tier transport (panel members in panes) — not needed while `cli` covers review
