---
artifact_contract: bee-plan/v1
mode: standard
feature: herding-cockpit-completeness
lane: standard
class: feature
playbook: references/planning-reference.md ("Class playbooks" › feature)
---

# Herding Cockpit Completeness — Plan

## Requirements

From `docs/history/herding-cockpit-completeness/CONTEXT.md` (locked D1–D8, cited by store id):

- D1 (c943feb9) `bee herding interrupt <job-id>`: Escape to the recorded pane, pane stays open, outcome word `interrupted`, a waiting `bee herding run` returns `interrupted`, job resumable with `--continue`.
- D2 (468c6cb8) `stalled` / `recovered` status words in `bee herding status`, one threshold constant shared with the supervisor, one run-stream progress line per transition.
- D3 (1ef811f7) `retryable` on every non-result envelope; `true` only for `spawn_failed`, `send_failed`, `flipped_before_send`; wave buckets carry it per worker.
- D4 (7172010b) `bee herding cancel <job-id>` fail-closed: pid captured, pane closed, exit confirmed ≤5 s, else exit non-zero with `cancel_termination_failed` and job `cancel_pending`. `FirstSuccessCancelRest` uses the same path.
- D5 (d5a1f7e1) status + occupancy sweep: recorded pane gone and no result file → outcome `interrupted`, reason `process_restarted`. Nothing relaunched.
- D6 (e0f6b8b5) after `done`/`blocked`, six git reads of the worker checkout → envelope key `git` (branch, head_sha, base_sha, ahead, dirty, changed_paths); only when the worker cwd is a git checkout.
- D7 (9615be76) no automatic behavior.
- D8 (afec9446) word list + envelope no-new-key law; readers (Pi drain, wave ledger, control loop) test against the list.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The one 120 s freshness constant lives in mailbox.rs, not `hooks/activity.rs` as CONTEXT.md says; D2 reuses it. | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:168` | `pub(crate) const ACTIVITY_FRESHNESS_SECS: i64 = 120;` |
| 2 | Outcome words are minted in one label function; `interrupted` / `cancelled` are added there. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2722` | `fn outcome_label(o: &RunOutcome) -> &'static str {` |
| 3 | The herdr transport already closes a pane; D4 reuses it. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:717` | `fn pane_close(&self, pane_id: &str) -> Result<(), String> {` |
| 4 | The herdr transport already reports the pane's foreground pid; D4 captures the pid from it. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:769` | `fn process_info(&self, pane_id: &str) -> Liveness {` |
| 5 | The tmux transport closes a pane with kill-pane; same D4 path on tmux. | read | `packages/bee-rs/crates/bee/src/herding/tmux.rs:659` | `self.call(&["kill-pane", "-t", pane_id]).map(|_| ())` |
| 6 | A cross-platform pid liveness probe already exists (windows :164, unix :179); D4's ≤5 s confirm polls it. | read | `packages/bee-rs/crates/bee/src/lock.rs:164` | `pub fn is_pid_alive(pid: Option<f64>) -> bool {` |
| 7 | herdr can send a named key to a pane; D1's Escape is `pane send-keys <id> esc`. | ran | `herdr pane send-keys --help` | `Use esc as the canonical Escape key name; escape is also accepted.` |
| 8 | Wave ledger rows have a per-worker outcome slot; `retryable` sits beside it (D3). | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:86` | `pub(crate) outcome: Option<String>,` |
| 9 | Merge-base against the main checkout's HEAD is the existing precedent; D6's base_sha copies it. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:214` | `let base = run_git(main_root, &["merge-base", "HEAD", branch]);` |
| 10 | A git read helper with cwd already exists; D6's six reads reuse it. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:85` | `pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {` |
| 11 | `bee herding status` today prints only enable + transport lines; D2/D5 add a jobs list there. | read | `packages/bee-rs/crates/bee/src/herding.rs:749` | `fn status(flags: &[&str]) -> ExitCode {` |
| 12 | Occupancy already computes the live pane set; D5's sweep reuses that set. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:966` | `pub(super) fn occupancy(flags: &[&str]) -> ExitCode {` |
| 13 | `--continue` re-reads the job dir; D1's mark is read on the same path. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2424` | `fn execute_continue(opts: &Options, herdr: &dyn PaneTransport) -> ExecResult {` |
| 14 | The run tick already projects a status word from the activity file; D2's stalled/recovered projection extends it. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:1578` | `fn status_with_activity(` |
| 15 | `FirstSuccessCancelRest` is declared in fleet but marked not implemented; D4 wires the cancel function, the wave choreography stays out of scope. | read | `packages/bee-rs/crates/fleet/src/wave.rs:74` | `FirstSuccessCancelRest,` |
| 16 | The cockpit transport has a send-text method but no send-key; D1 adds one trait method. | read | `packages/bee-rs/crates/bee/src/herding/pane_verbs.rs:115` | `fn pane_send_text(&self, pane: &str, text: &str) -> Result<(), String>;` |
| 17 | The CLI registry is hand-maintained (decisions d515c1d7, 3358743e); new verbs need a payload entry and pass `tests/registry_contracts.rs`. | read | `packages/bee-rs/crates/bee/src/generated/registry_payload.json:1` | `"herding` |

## Discovery

- Deferred question (a) pid on Windows/POSIX: `process_info` (claim 4) returns `Liveness::Alive{pid}` from herdr `pane process-info`; tmux reads `#{pane_pid}` (tmux.rs:815). `lock::is_pid_alive` (claim 6) confirms exit on both OSes.
- Deferred question (b) how the waiter sees the mark: the poll tick reads `activity.json` each round (claim 14) but not `job.json` today (hat wave, facts seat: run.rs:1140-1197 and :2097-2144 read result, heartbeat, pane text, liveness only). The mark is a field in `job.json`; slice 1 adds one `job.json` read to the tick. No signal file.
- Deferred question (c) merge-base ref: the main checkout's HEAD (claim 9), never a branch name.
- CONTEXT.md points D2 at `hooks/activity.rs`; the constant is in mailbox.rs (claim 1). Same number, one constant. Recorded as a correction, not a reinterpretation.
- Pi drain (`.pi/extensions/bee-guard.ts` ~:534-760) reads job_id, cell_id, status, summary, proof, report_path from the worker result file and ignores unknown keys. D8 tolerance is a pinned test, no code change.
- Verify map: no herding feature file under `.bee/verify/verify-app/features/` (README ~:89 says the cockpit needs live panes). Environment-fact gap; proceed.
- Existing test counts: run.rs 186, wave.rs 48, mailbox.rs 48, herding.rs 12. Every slice adds cases beside them.

## Approach

Recommended: marks as `job.json` fields, one new module, transport methods reused.

- **Mark storage (discretion):** `job.json` gains `mark` (`interrupted` | `cancelled` | `cancel_pending`), `mark_reason`, `mark_at`. `--continue` (claim 13), the run tick (claim 14) and the sweep (claim 12) read the same field.
- **Verbs (discretion → new module):** `herding/job_verbs.rs` with `interrupt` and `cancel`, dispatched from the herding.rs table. Typed refusals with a FIX line.
- **Mark reads (hat wave finding):** today neither the poll tick nor `execute_continue` reads a mark. Slice 1 adds one helper `read_mark(bee_dir, job_id) -> Option<Mark>` in mailbox.rs; the tick closure (run.rs ~:2100-2145) calls it once per tick beside `status_with_activity`; `execute_continue` (claim 13) calls it after the `paused_limit_at` check: mark `cancelled` or `cancel_pending` → new `ContinueRefusal::Cancelled` with a FIX line; mark `interrupted` → allowed, mark cleared on resume. Latency of the interrupt signal is one poll interval (Open Question 4).
- **D1 interrupt:** new trait method `pane_send_key(pane, key)` on `PaneTransport`; herdr = `["pane","send-keys",pane,"esc"]` (claim 7), tmux = `send-keys -t <pane> Escape` WITHOUT `-l` (the `-l` flag in `send_text_argv` at pane_verbs.rs:311-318 means literal bytes; a named key needs no `-l`). Fakes get the method. Writes mark `interrupted`, `mark_reason` `user`. The run tick sees the mark and returns new `RunOutcome::Interrupted` → label `interrupted` (claim 2), pane left open. Success prints `herding: interrupted job <id> (pane kept open)`; a terminal or unknown job gets a typed refusal with a FIX line.
- **D4 cancel:** `process_info` → pid (claim 4); mark `cancel_pending`; `pane_close` (claims 3, 5); poll `is_pid_alive` every 100 ms up to 5 s (claim 6; a 5-line loop, the control-loop child-kill helper owns a `Child` handle and does not fit a foreign pid). Exit confirmed → mark `cancelled`, `RunOutcome::Cancelled`, line `herding: cancelled job <id> (pane closed, pid <pid> exited)`. Not confirmed → mark stays `cancel_pending`, exit non-zero, error `cancel_termination_failed`, FIX line `kill pid <pid> by hand, then run bee herding cancel <id> again`. A second `cancel` on a `cancel_pending` job skips the pane step and re-confirms the pid, finalising `cancelled` when it is gone. `bee herding status` shows `mark: cancel_pending` so a reader learns it. Fleet `FirstSuccessCancelRest` stays declared-only (claim 15); the per-job function is the path it must call when implemented.
- **D3 retryable:** `result_envelope` adds `retryable` on every envelope except `done` and `dry_run`; `blocked` carries `false` (D3 lists it); `true` only for `spawn_failed`; every other label `false`. Wave bucket rows and `WorkerRow` (claim 8) carry `retryable` with `true` for `send_failed` / `flipped_before_send`.
- **D2 status words:** `bee herding status` (claim 11) gains a `jobs` array read from `.bee/mailbox/*/job.json`: `job_id`, `pane_id`, `round`, `mark`, `status`. `status` = `stalled` when activity older than `ACTIVITY_FRESHNESS_SECS` (claim 1) and `process_info` is Alive; `recovered` on the first fresh read after stalled (a `last_status` field in job.json makes it print once); otherwise the existing word. The run tick prints one progress line per transition.
- **D5 sweep:** one shared helper `mark_orphans(main_root, live_panes)` in mailbox.rs, called by status and occupancy before their output: recorded pane not in the live pane set and no `result-*.json` and no mark → mark `interrupted`, `mark_reason` `process_restarted`. Writes only on that transition and prints one progress line per marked job (a query verb that mutates is what D5 locks; the line makes it visible). Nothing relaunched (D7). `mark_reason` separates a user interrupt (D1) from a sweep mark (D5) in status output; the run envelope keeps the D8 word list only.
- **D6 git block:** after a Result, `run_git(opts.cwd, …)` from `verbs/worktree/git.rs` (claim 10; it returns stderr, the status_full variant does not): `rev-parse --is-inside-work-tree` gates the key; then `rev-parse --abbrev-ref HEAD`, `rev-parse HEAD`, `merge-base HEAD <main_root HEAD sha>` (claim 9), `rev-list --count <base>..HEAD`, `status --porcelain`, `diff --name-only <base>..HEAD`. `changed_paths` is the union of the diff list and the porcelain paths, so a dirty tree's edits are not missed. Key `git` only when the first read succeeds (no-new-key law). `files_changed` (worker-reported) stays beside it untouched.
- **Registry:** hand entries for `herding interrupt` and `herding cancel` in registry_payload.json plus the catalog flag ledger (claim 17).

Rejected:
- Signal file per job for the mark: a second thing to sweep; job.json already exists and `--continue` already reads it.
- Implementing `FirstSuccessCancelRest` choreography in fleet: not shaped; D4 only says the policy uses the same path.
- A second freshness number for stalled: D2 forbids it.

Risk map:
- Windows herdr `send-keys esc` reaching a Claude session's stdin unverified live → Open Question 2; test through the fake transport.
- `is_pid_alive` on Windows returns true for a zombie handle until the OS releases it → the 5 s window covers normal exit; failure is the fail-closed path, never a false `cancelled`.
- Occupancy is called every control-loop tick; the sweep must not write when nothing changed (write only on transition).

SMALLER PATH check: the cheapest shape that keeps D1–D8 is three slices with one new module and one new trait method. Dropping the module keeps the same code in herding.rs; dropping the trait method means shelling out around the transport, which the `tmux`/`herdr` config key forbids. PASS.

## Shape

Phase plan (standard feature, walking skeleton first):

| Slice | Delivers | Decisions | Files |
|-------|----------|-----------|-------|
| 1 | Marks in job.json; `interrupt` + `cancel` verbs; `pane_send_key` trait method; `Interrupted`/`Cancelled` run outcomes read from the mark; `retryable` on run envelopes; registry entries. User path: run a job, interrupt it, see `interrupted`, continue it, cancel it, see `cancelled`. | D1, D3 (run part), D4, D7, D8 | run.rs, pane_verbs.rs, tmux.rs, herding.rs, new herding/job_verbs.rs, mailbox.rs, registry_payload.json, catalog.rs |
| 2 | `bee herding status` jobs list with `stalled`/`recovered`; run-stream transition line; orphan sweep in status + occupancy. | D2, D5, D7 | herding.rs, run.rs, wave.rs, mailbox.rs |
| 3 | `git` envelope block; `retryable` on wave buckets + ledger rows; Pi drain tolerance test; knowledge + config-reference sync. | D3 (wave part), D6, D8 | run.rs, wave.rs, wave_ledger.rs, bee-guard.ts test, docs/knowledge/areas/bee-herding/*, docs/config-reference.md |

Slice 1 is the current slice; slices 2–3 keep headlines until slice 1 caps.

## Test matrix (triad)

| Case | Slice | Proof |
|------|-------|-------|
| happy: interrupt writes mark, tick returns `interrupted`, pane open, `--continue` accepted | 1 | run.rs unit tests with FakeTransport |
| happy: cancel closes pane, pid gone → `cancelled` | 1 | job_verbs.rs test with a fake whose `process_info` flips to Absent |
| edge: cancel pid still alive after 5 s → `cancel_termination_failed`, mark `cancel_pending`, non-zero exit | 1 | same fake, never flips |
| edge: interrupt/cancel on unknown job → typed refusal with FIX line | 1 | job_verbs.rs |
| contract: `retryable` present on every non-Result label, `true` only for spawn_failed; absent on Result | 1 | run.rs envelope test |
| contract: registry knows both verbs | 1 | tests/registry_contracts.rs, tests/registry_dispatch.rs |
| happy: stalled after 120 s with Alive, recovered once, then working | 2 | herding.rs status tests |
| edge: sweep marks pane-gone + no result as interrupted/process_restarted, skips a job with a result | 2 | wave.rs occupancy test |
| happy: `git` block on a checkout cwd; absent on a non-git cwd | 3 | run.rs test with a temp git repo |
| contract: wave bucket rows carry retryable; Pi drain ignores the new keys | 3 | wave.rs, wave_ledger.rs; a bee-guard fixture test |

Proof command per cell: `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee <filter>` with filter `herding::job_verbs` / `herding::run` / `herding::mailbox` (slice 1), `herding::` (slice 2), `herding::wave` (slice 3); slice 1 adds `--test registry_contracts --test registry_dispatch`.

## Open Questions

1. `FirstSuccessCancelRest` is declared-only in fleet (claim 15). D4 names it; wiring the wave choreography is not shaped. Slice 1 exposes the per-job cancel function and records the pointer; the choreography waits for its own feature.
2. herdr `pane send-keys esc` on Windows conhost: verified on `--help` only, not live against a Claude pane. Tested through the fake transport; a live check is a uat item.
3. `recovered` prints once: `last_status` lives in job.json and both the run tick and `status` write it on a transition only. The hat wave noted a query that writes state; D5 already locks that shape for the sweep, so the plan keeps it.
4. Interrupt latency: the waiter sees the mark on its next tick, up to one poll interval later. The plan accepts that; a file watch is not shaped.
5. The word `stalled` also names herdr's unsubmitted-prompt stall (knowledge doc :288). D2 locks the word; the status output prefixes it with the job id so the two never share a line. Noted for the scribe.
6. A worker that writes a UTF-8 BOM before its result JSON gets `malformed_result` (seen live on the hat-user-impact seat, job hat-user-impact-hcc). Out of this feature's scope; backlog item.

## Out of scope

- Agent fallback, auto-retry, orphan relaunch (D7).
- Workflow runner, APPROVE hash binding, run.jsonl journal (decision 9f5c6d17).
- Any change to herding-orchestration D1–D18 or herding-adopt R1–R8.
- A second pane transport or engine port.
