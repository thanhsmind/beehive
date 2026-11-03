---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Fix the open bee-report issues of 2026-10-23

## Summary

This fixes ten defects that users filed against bee, and closes three
that earlier releases already fixed. After it lands: `cells add` takes
the new cells of a revised plan in one batch; a refused claim leaves no
stuck lease; `leader-check` accepts a directory entry; the Windows suite
goes green; a deploy worker can run the release script; the release
script keeps its permit when a cheap check fails; a ceiling timeout
stops the Paseo agent; the test suite passes inside a herded worker;
host repos stop conflicting on appended `.bee` logs; a merge over
untracked history files stops early with a clear message naming them; and `state route --set` can name its
lane record.

Mode: `standard` — 3 risk flags: cross-platform (Windows), public
contracts (CLI flags and refusals), multi-domain (cells, dispatch,
herding, worktree, onboarding, release script).
Why this is the least workflow that protects the work: each fix is a
small change at one traced root cause, proven by a red-first test next
to the existing tests for that code.

Playbook: `skills/bee-planning/playbooks/bugfix.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1 (#127): `cells add` checks each batch cell against the approved packet; every packet cell is in the batch or already stored; no batch-length check.
- D2 (#125): a refused claim unwinds its own lease under the session the claim file records.
- D3 (#121): `leader-check` counts a directory entry as touched when the commit touched a file under it.
- D4 (#96): `gate_pid_dead` has a Windows arm; paseo path match and the mailbox check compare canonical paths.
- D5 (#126): no `--read-only` on a deployment-stage herding payload.
- D6 (#117): `release.sh` spends the permit after the zero-mutation checks; `release-dirt.sh` allowlists `.bee/supervisor/broker-heartbeat.json`.
- D7 (#123): a paseo ceiling timeout stops the agent.
- D8 (#118): real-binary tests in `pi_plugin_contracts.rs` and `session_release.rs` remove `BEE_HERDING_WORKER`.
- D9 (#120): onboarding writes a marker-spliced `.gitattributes` block with `merge=union` for six append-only `.bee` logs; this repo gets it from its own onboarding, no hand edit.
- D10 (#122): untracked `docs/history/<feature>` files on main that the branch writes make the merge refuse `WORKTREE_MERGE_MAIN_DIRTY` up front, zero mutations; they are never staged.
- D11 (#124): `state route --set --lane-record <feature>` picks the lane record.
- D12: #104, #106, #119 already fixed; #69 out of scope.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | cells add compares the batch length to the whole packet | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:790 | if cells.len() != approved_cells.len() { |
| 2 | the claim unwind is owner-matched against the caller's session | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:597 | if owner.as_deref() != session { |
| 3 | leader-check compares exact paths | read | packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:312 | .filter(\|f\| !paths.contains(f)) |
| 4 | gate_pid_dead has no Windows arm | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:205 | #[cfg(not(unix))] |
| 5 | the path compare falls back to raw strings | read | packages/bee-rs/crates/bee/src/path_identity.rs:212 | sa.to_lowercase() == sb.to_lowercase() |
| 6 | the mailbox check is a raw starts_with | read | packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:307 | target != zone && target.starts_with(&zone) |
| 7 | Windows CI fails on exactly three tests | ran | gh run view 39895071304 --log-failed | test result: FAILED. 4268 passed; 3 failed; 24 ignored |
| 8 | prepare adds --read-only to every gather | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2619 | if kind == "gather" \|\| kind == "advisor" { |
| 9 | release.sh authorizes before the dirt check | read | scripts/release.sh:116 | "$BEE_BIN" dispatch authorize --id "$BEE_DISPATCH_ID" --release-version "$AUTH_VERSION" \ |
| 10 | a paseo ceiling only prints, never stops | read | packages/bee-rs/crates/bee/src/herding/run.rs:3727 | paseo worker hit the ceiling; pane kept for inspection |
| 11 | a stop argv helper exists | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:509 | pub fn stop_argv(id: &str) -> Vec<String> { |
| 12 | only some pi tests fence the marker | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1172 | cmd.env_remove("BEE_HERDING_WORKER"); |
| 13 | this repo unions only three logs | read | .gitattributes:31-33 | .bee/decisions.jsonl merge=union / .bee/backlog.jsonl merge=union / .bee/review-candidates.jsonl merge=union |
| 14 | onboarding renders a managed gitignore block to reuse | read | packages/bee-rs/crates/bee/src/onboard/merge.rs:61 | pub fn render_gitignore_block() -> String { |
| 15 | merge bookkeeping sweeps the feature history dir | read | packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:302 | roots.push(format!("docs/history/{feature}")); |
| 16 | route passes no lane record to the resolver | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:910 | let mut target = resolve_mutation_target(&ctx.root, None, "route", no_lane)?; |
| 17 | a shared lane selector exists | read | packages/bee-rs/crates/bee/src/verbs/state_group/flags.rs:192 | pub(crate) fn mutation_lane_selector(flags: &Flags, verb: &str) -> Result<(Option<String>, bool), Err2> { |
| 18 | prune is already fixed (#104, #106) | ran | bee worktree prune --dry-run --older-than-days 0 | would remove, 100.1 MB reclaimable |
| 19 | #119's fix is in v2.56.0 | ran | git log --format='%h %s' -1 7983338d3 | Route decisions and backlog to the control-root store from a granted worktree |
| 20 | a claim with no session adopts one | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:768 | if let Some(candidate) = resolve_session_adopt(control)? { |
| 21 | the unwind result is discarded | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:626 | release_claim_typed(control, session, cell_id, None).map(\|_\| ()) |
| 22 | prepare knows the deployment stage | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2460 | let is_deployment = v2_info |
| 23 | git refuses a merge over an identical untracked file | ran | bash .bee/tmp/gitt.sh | error: The following untracked working tree files would be overwritten by merge: |
| 24 | a Windows pid-liveness helper exists | read | packages/bee-rs/crates/bee/src/lock.rs:156 | pub fn is_pid_alive(pid: Option<f64>) -> bool { |

## Discovery

Three read-only gathers traced every issue to its code (reports in
`.bee/mailbox/job-1792728416449-2271460-1`, `job-1792728417446-2271672-1`,
`job-1792728418436-2271896-1`); the leader re-opened each anchor above.
#104/#106 were fixed by `e40edc403` (v2.54.0) and #119 by `7983338d3`
(v2.56.0, decisions from a granted worktree land in main's store); both
reports predate the fix. The #125 gather found that a missing
test-results record only warns; the torn state comes from the
owner-matched unwind (claim 2), so iss-2 must reproduce it red first.

## Approach

Each cell changes the one root-cause site and adds a red-first test next
to the existing tests. Rejected: union-reading worktree decision stores
for #119 (already fixed at the write side; two homes for one fact);
renaming route's `--lane` (breaks every caller); a `.gitattributes`
glob `*.jsonl` (would union non-append files a host owns).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Windows fixes cannot run red on Linux | MEDIUM | iss-4, iss-5 | Windows CI green on the merge commit; Linux tests stay green |
| Parallel cells break each other's build | MEDIUM | all | workers wait and re-run on a compile error outside their files |
| New flag changes the pinned flag count | LOW | iss-12 | distinct_flag_vocabulary and registry tests green |

Waves: all cells run in parallel (disjoint files) except iss-7 after
iss-6 (both edit `verbs/drivers/tests.rs`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"iss-1 to iss-12 change Rust code, scripts and tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader syncs knowledge at capture."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check."}
  ]
}
```

## Shape

Epic map. Outcome: every open bee-report defect has a fix and a test.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Cells | packet add, claim unwind, leader-check | correct work refused or left stuck | iss-1, iss-2, iss-3 | cells and plan_packets tests |
| Windows | pid liveness, path compare | CI red on main | iss-4, iss-5 | Windows CI |
| Dispatch and release | deploy payload, permit order | releases cannot run from a worker | iss-6, iss-7 | prepare and release-dirt tests |
| Herding and tests | ceiling stop, marker fence | agents outlive jobs; suite red in workers | iss-8, iss-9 | herding tests; suite under the marker |
| Worktree and state | union merge, history staging, route record | merges abort; claims refuse | iss-10, iss-11, iss-12 | onboard, worktree, route tests |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| iss-1 | Accept a batch of new cells after a plan revision | verbs/state_group/plan_packets.rs | — | `cells add` with only the new cells of a revised packet succeeds | cargo test plan_packets |
| iss-2 | Unwind a refused claim's lease under the session that holds it | verbs/cells/handlers_write.rs, verbs/cells/tests.rs | — | a refused claim leaves no held lease and the cell stays claimable | cargo test cells:: |
| iss-3 | Count a directory entry as touched in leader-check | verbs/cells/leader_check.rs | — | leader-check passes when the commit changed files under a listed directory | cargo test leader_check |
| iss-4 | Tell a dead claim-gate pid on Windows | verbs/cells/claims.rs | — | on Windows an unclaim frees a claim whose gate pid is dead | cargo test claims; Windows CI |
| iss-5 | Compare canonical paths on Windows for paseo projects and the job mailbox | path_identity.rs, hooks/worker_guard.rs | — | Windows CI passes the paseo project and mailbox tests | cargo test path_identity, worker_guard; Windows CI |
| iss-6 | Keep the deploy worker writable | verbs/drivers/prepare.rs, verbs/drivers/tests.rs | — | a deployment-stage prepare payload has no `--read-only` | cargo test deploy |
| iss-7 | Spend the release permit only after the cheap checks pass | scripts/release.sh, scripts/release-dirt.sh, verbs/drivers/tests.rs | iss-6 | a dirty tree refuses the release and the permit stays unspent | cargo test release_dirt, release_sh |
| iss-8 | Stop the Paseo agent when the ceiling times out | herding/run.rs | — | after `timed_out_ceiling` the Paseo agent is stopped | cargo test herding::run |
| iss-9 | Keep the herded-worker marker out of real-binary tests | tests/pi_plugin_contracts.rs, tests/session_release.rs | — | the suite is green with BEE_HERDING_WORKER=1 | BEE_HERDING_WORKER=1 cargo test both targets |
| iss-10 | Union-merge the append-only bee logs | onboard/merge.rs, onboard/plan.rs, onboard/apply.rs, onboard/templates.rs, onboard/tests.rs | — | onboarding writes a `.gitattributes` block; two EOF appends merge cleanly | cargo test onboard |
| iss-11 | Refuse a merge up front when main holds untracked history files the branch writes | verbs/worktree/merge.rs, verbs/worktree/phases.rs, verbs/worktree/tests.rs | — | `worktree merge` refuses `WORKTREE_MERGE_MAIN_DIRTY` naming the files, nothing staged | cargo test worktree:: |
| iss-12 | Let route --set name its lane record | verbs/state_group/workflows.rs, verbs/state_group/flags.rs, tests/route_lane_targeting.rs, catalog.rs, generated/registry_payload.json | — | `state route --set --lane-record <f>` writes that lane's route from an unbound session | cargo test route_lane_targeting |

Paths are under `packages/bee-rs/crates/bee/src/` unless they start with
`scripts/`, `tests/` (crate tests) or `.gitattributes`. The cell packet:

```json
[
  {
    "id": "iss-1",
    "feature": "issues-sweep-1023",
    "title": "Accept a batch of new cells after a plan revision",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "01708244-9838-4cb3-8dd0-be6dd79bba87"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 (issue 127). In verbs/state_group/plan_packets.rs `check_batch_matches_approved_preview` refuses with `addCells: batch contains N cell(s), but approved gate preview packet declared M cell(s).` whenever the batch length differs from the whole approved packet (`if cells.len() != approved_cells.len() {`). After a plan revision the packet holds old cells already in the store plus new ones, so a batch of only the new cells is refused while the same cells one by one pass. Replace the length check: keep the per-cell `check_cell_matches_approved_preview` loop, and refuse only when an approved packet cell is neither in the batch nor already a stored cell of the feature (read stored cell ids with the same cells-store reader the module or handlers_write.rs already uses; never a second parser). Name the missing ids in the refusal. Red first in the plan_packets.rs test module (see `pihp_gate_packet_refuses_zero_cells_and_preserves_one_and_seven_cells`): a packet of 3 cells where 1 is already stored and a batch of the other 2 is accepted; a batch that omits an unstored packet cell is refused naming it; an extra undeclared cell is still refused. Keep `pihp_gate_packet_cells_add_refuses_packet_differing_from_approved_preview` in verbs/cells/tests.rs green. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_packets && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pihp_gate_packet",
    "must_haves": {
      "truths": [
        "a batch holding only the not-yet-stored cells of the approved packet is accepted",
        "a batch that leaves an unstored packet cell out is refused naming it",
        "an undeclared or mismatched cell is still refused"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "stored-or-batched coverage check and its tests"}],
      "key_links": ["check_batch_matches_approved_preview reads stored cell ids"],
      "prohibitions": ["No change to per-cell field matching", "No code comments"]
    }
  },
  {
    "id": "iss-2",
    "feature": "issues-sweep-1023",
    "title": "Unwind a refused claim's lease under the session that holds it",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D2", "b364bcbe-9ef8-4ec1-abf3-bab299694cc9"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Per D2 (issue 125). Reported: `bee cells claim --id ruv-1 --worker ruv-code` from main, no --session-id, left the cell status open with no trace.worker while `cells show` reports the claim verdict held, and `dispatch prepare` then refuses claim_ownership not_claimed. Traced cause: `claim_cell_cross_session_ex` (verbs/cells/handlers_write.rs) writes the lease with `claim_cell_file` (verbs/cells/claims.rs), which may adopt a session (`resolve_session_adopt`) when none is given, so the claim file records that adopted session; every later refusal before `write_cell` unwinds with `release_claim(control, session, &cell_id)` using the caller's own session (None), and `release_claim_typed` refuses NOT_OWNER (`if owner.as_deref() != session {`) and the result is discarded, so the lease outlives the refused claim. The reporter saw only the test-results warning and no refusal line, so also check the silent `Err(Fail::Delegate)` unwind near the end of claim_cell_cross_session_ex (it uses the same caller session), and check that the status write (`write_cell`) targets the same store `cells show` and `dispatch prepare` read when the claim runs from main on a cell added from a granted worktree. Fix (smallest shape): right after `ClaimFileOutcome::Ok { claim }` in claim_cell_cross_session_ex, shadow `session` with the session the claim recorded (an owned Option<String>), so every later unwind, `trace.claim_session` and the route-count read use it; do not edit verbs/cells/claims.rs (record in the report that trace.claim_session now carries the adopted session). Red first in verbs/cells/tests.rs: reproduce the torn state — a claim with no session id whose lease gets an adopted session and that then refuses or delegates after the lease (rg -n release_claim verbs/cells/handlers_write.rs) — assert the claim file is gone and a later claim succeeds; watch it fail before the fix. Add a test that a claim without --session-id that succeeds sets status claimed and trace.worker (name it with `claim_without_session`). If you cannot reproduce the reported state, report [BLOCKED] with what you tried instead of guessing. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells::",
    "must_haves": {
      "truths": [
        "a claim that refuses after writing its lease leaves no lease behind, also when the lease adopted a session",
        "a claim without --session-id that succeeds sets status claimed and trace.worker"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "substantive": "torn-claim unwind test that failed before the fix"}],
      "key_links": ["post-lease unwinds release under the claim file's recorded session"],
      "prohibitions": ["No change to the ownership guard for a different live session", "No code comments"]
    }
  },
  {
    "id": "iss-3",
    "feature": "issues-sweep-1023",
    "title": "Count a directory entry as touched in leader-check",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D3", "cf4781e7-0f06-457f-81cd-bb56876ad032"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3 (issue 121). In verbs/cells/leader_check.rs the commit-diff arm of `run_leader_check` builds `untouched` with `.filter(|f| !paths.contains(f))`, an exact match against the files `git show --name-only` lists, so a files_changed entry that is a directory (`public/chinh-sach`) is refused even when the commit changed files under it. Treat an entry as touched when a committed path equals it or starts with the entry plus `/` (after `normalize_cell_path`; strip one trailing `/` from the entry first). Red first in the leader_check.rs test module next to `claimed_file_in_trailer_commit_passes_with_commit_diff_checked`: a commit that touches `d/a.txt` and `d/sub/b.txt` with files_changed `[\"d\"]` passes; `[\"dx\"]` with a commit touching `d/a.txt` is still refused naming `dx`. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee leader_check",
    "must_haves": {
      "truths": [
        "a directory entry in files_changed passes when the commit touched a file under it",
        "a path that is only a string prefix of a touched path is still refused"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs", "substantive": "directory-prefix match and its tests"}],
      "key_links": ["the untouched filter uses a component prefix match"],
      "prohibitions": ["No change to the trailer-commit lookup", "No code comments"]
    }
  },
  {
    "id": "iss-4",
    "feature": "issues-sweep-1023",
    "title": "Tell a dead claim-gate pid on Windows",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D4", "d8e689c8-26aa-4f92-aa49-d22f7f4857fb"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/claims.rs"],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/Cargo.toml"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4 (issue 96, Windows CI run 39895071304). `fn gate_pid_dead` in verbs/cells/claims.rs returns false on every non-unix target (`#[cfg(not(unix))]` arm), so on Windows the test `verbs::cells::tests::unclaim_breaks_a_dead_pid_gate_that_is_younger_than_the_age_window` fails: a fresh gate with pid 999999999 reads live and the unclaim keeps the claim. `crate::lock::is_pid_alive` (lock.rs) already has a Windows arm (OpenProcess with ERROR_INVALID_PARAMETER); reuse it — no new unsafe code and no second copy of the liveness logic. Replace the `#[cfg(not(unix))]` arm body with a `#[cfg(windows)]` arm that returns the negation of that helper for the pid (adapt the argument type to its signature), and keep a `#[cfg(not(any(unix, windows)))]` arm returning false. The unix arm does not change. Proof: Linux `cargo test claims` stays green and `cargo check --target x86_64-pc-windows-gnu` (if the target is installed; else record that it was skipped) compiles; the Windows CI after merge is the live proof — say so in the report. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee claim",
    "must_haves": {
      "truths": [
        "on Windows gate_pid_dead reports a non-existent pid as dead",
        "on Windows a live pid is never reported dead",
        "unix behaviour is unchanged"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs", "substantive": "cfg(windows) liveness arm"}],
      "key_links": ["gate_pid_dead calls crate::lock::is_pid_alive on Windows"],
      "prohibitions": ["No change to the unix arm", "No new unsafe code", "No code comments"]
    }
  },
  {
    "id": "iss-5",
    "feature": "issues-sweep-1023",
    "title": "Compare canonical paths on Windows for paseo projects and the job mailbox",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D4", "d8e689c8-26aa-4f92-aa49-d22f7f4857fb"],
    "files": [
      "packages/bee-rs/crates/bee/src/path_identity.rs",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/path_identity.rs",
      "packages/bee-rs/crates/bee/src/hooks/worker_guard.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "docs/history/windows-ci-green/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4 (issue 96, Windows CI run 39895071304). Two Windows-only failures. (a) `herding::paseo::tests::ensure_workspace_creates_with_the_main_checkout_project` (herding/paseo.rs ~1748): `rows_with_path` calls `crate::path_identity::canonical_paths_equal`, whose string fallback (`sa.to_lowercase() == sb.to_lowercase()` in path_identity.rs) compares `std::path::absolute` outputs, so a `C:\\Users\\RUNNER~1\\...` 8.3 spelling, a `\\\\?\\` prefix or mixed `/` and `\\` never match and `--project prj_main` is dropped. On Windows only (`#[cfg(windows)]`), before the string fallback, resolve both sides with `dunce::canonicalize` (the crate already depends on dunce; see verbs/worktree/handlers.rs) when they exist, falling back to the current value when canonicalize fails, and normalize separators; the unix path of canonical_paths_equal must not change (no new symlink resolution on Linux). (b) `hooks::worker_guard::tests::read_only_table` (hooks/worker_guard.rs ~519-547): `in_mailbox` compares `settle(&absolute)` with `settle(mailbox)` by `starts_with`; `settle` uses `std::fs::canonicalize` (verbatim `\\\\?\\` paths on Windows) and the test target mixes `\\` with `/`, so `mv <mailbox>/result-1.json.tmp <mailbox>/result-1.json` is denied. Make `settle` use `dunce::canonicalize` (on unix it equals std canonicalize, so Linux results do not change) and push the lexical components so separators are uniform on both sides. Do not change herding/paseo.rs. Proof: Linux tests for both modules stay green (`cargo test path_identity`, `cargo test worker_guard`); the Windows CI after merge is the live proof — say so in the report. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee path_identity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worker_guard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee ensure_workspace",
    "must_haves": {
      "truths": [
        "canonical_paths_equal matches an 8.3 or verbatim-prefixed spelling of the same existing path",
        "in_mailbox accepts a target under the mailbox whatever the separator mix",
        "Linux behaviour of both checks is unchanged"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/path_identity.rs", "substantive": "dunce-canonical string fallback"}],
      "key_links": ["rows_with_path and in_mailbox go through the dunce-canonical helpers"],
      "prohibitions": ["No change to herding/paseo.rs", "No code comments"]
    }
  },
  {
    "id": "iss-6",
    "feature": "issues-sweep-1023",
    "title": "Keep the deploy worker writable",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D5", "e0da7c92-3c20-4c14-aeba-54fd5e097bd3"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Per D5 (issue 126). In verbs/drivers/prepare.rs the herding arm adds ` --read-only` when `kind == \"gather\" || kind == \"advisor\"`, also for a deployment-stage dispatch (`is_deployment` is computed earlier in the same function from the v2 stage), so the deploy worker's guard refuses every command and scripts/release.sh can never run. Skip `--read-only` when `is_deployment` is true. Red first in verbs/drivers/tests.rs: in `test_deploy_payload_stdin_and_cwd_with_and_without_granted_worktree` (or a new test beside it using its fixture) assert the deployment payload command does not contain `--read-only`, and that a plain gather payload still does; watch the first assert fail before the fix. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy",
    "must_haves": {
      "truths": [
        "a deployment-stage gather payload carries no --read-only",
        "a plain gather or advisor payload still carries --read-only"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "deploy payload read-only assertion"}],
      "key_links": ["the read-only flag is gated on is_deployment"],
      "prohibitions": ["No change to the deploy prompt or cwd", "No code comments"]
    }
  },
  {
    "id": "iss-7",
    "feature": "issues-sweep-1023",
    "title": "Spend the release permit only after the cheap checks pass",
    "lane": "standard",
    "role": "code",
    "deps": ["iss-6"],
    "decisions": ["D6", "3b60bbef-4e72-487f-a5a5-01224d93a5f4"],
    "files": [
      "scripts/release.sh",
      "scripts/release-dirt.sh",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "scripts/release.sh",
      "scripts/release-dirt.sh"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D6 (issue 117). In scripts/release.sh the line `\"$BEE_BIN\" dispatch authorize --id \"$BEE_DISPATCH_ID\" --release-version \"$AUTH_VERSION\"` (section 2, preconditions) spends a one-time permit before the branch check, the version checks, the tag checks and the release-dirt.sh dirt check of section 3, so any of those refusals burns the permit. Keep the BEE_DISPATCH_ID-set and BEE_BIN-executable checks where they are, and move the authorization so it runs after the zero-mutation refusals and before the first write, on ALL THREE paths the script has: (1) the fresh-release path (`elif [ -n \"$ARG_VERSION\" ]`): authorize after its first `release-dirt.sh` collection and the remaining zero-mutation checks, right before the version bump; (2) the idempotent re-run path (`ARG_VERSION == HEAD_VERSION`): authorize before its tail; (3) the no-version-argument path, which skips section 3 and goes straight to tag and push: authorize before any tag or push. A simple shape: one shell function `authorize_release` called once per path, with a guard so it never runs twice. No path may reach a write, a tag or a push without a successful authorize. In scripts/release-dirt.sh `is_allowlisted` add `.bee/supervisor/broker-heartbeat.json` (exact path, not a glob). Red first in verbs/drivers/tests.rs: extend `test_release_dirt_script_filters_only_bee_paths` so the heartbeat path is filtered; add a test named `test_release_sh_authorizes_after_cheap_checks_on_every_path` that reads scripts/release.sh and asserts: the first authorize call comes after the first `release-dirt.sh` collection (the one at the fresh-path dirt check, not the ones inside `restore_tree`) and before the bump; and every path that can reach `git tag` or `git push` passes an authorize call first (follow the style of existing release.sh text tests: rg -n release.sh verbs/drivers/tests.rs). Keep `bash -n scripts/release.sh` clean. No code comments added to Rust; keep the script's comment style. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "bash -n scripts/release.sh && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee release",
    "must_haves": {
      "truths": [
        "a dirty tree, a wrong branch or a taken tag refuses before the permit is spent",
        "the re-run path and the no-version path still authorize before any tag or push",
        "release-dirt.sh does not list .bee/supervisor/broker-heartbeat.json"
      ],
      "artifacts": [{"path": "scripts/release.sh", "substantive": "authorize after the zero-mutation checks"}],
      "key_links": ["the order test reads scripts/release.sh"],
      "prohibitions": ["No release run", "No change to what the release commit carries"]
    }
  },
  {
    "id": "iss-8",
    "feature": "issues-sweep-1023",
    "title": "Stop the Paseo agent when the ceiling times out",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D7", "438886cc-f4f3-4eb8-85c2-29817c2d2de4"],
    "files": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Per D7 (issue 123). In herding/run.rs both paseo `PollDecision::TimedOutCeiling` arms (first round in `execute_paseo`, and the `--continue` round loop) only print `paseo worker hit the ceiling; pane kept for inspection` and return RunOutcome::TimedOutCeiling, so the Paseo agent keeps running and writing after the job returns. In both arms call `cli.call(&paseo::stop_argv(&agent_id))` first, the way `stop_for_model_mismatch` does (log a failed stop to stderr, never fail the run on it), and change the message to say the agent was stopped and kept for inspection. Do not archive the agent. Red first in the run.rs test module with the existing fake PaseoCli (see `execute_paseo_blocked_until_idle_timeout_times_out_with_log_tail_in_message` and `execute_paseo_happy_path_archives_agent`): drive a ceiling timeout and assert the fake saw a `stop <agent_id>` call and no `archive` call; watch it fail first. No real paseo call in tests. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run",
    "must_haves": {
      "truths": [
        "a paseo ceiling timeout sends paseo stop for the agent",
        "the agent is not archived and the outcome stays timed_out_ceiling"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "stop on ceiling and its test"}],
      "key_links": ["both TimedOutCeiling paseo arms call stop_argv"],
      "prohibitions": ["No real paseo call in tests", "No code comments"]
    }
  },
  {
    "id": "iss-9",
    "feature": "issues-sweep-1023",
    "title": "Keep the herded-worker marker out of real-binary tests",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D8", "3656a936-830e-4988-929e-66286a4d0b2c"],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/session_release.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/session_release.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D8 (issue 118). `BEE_HERDING_WORKER=1 cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` fails targets pi_plugin_contracts and session_release because their real-binary spawns inherit the marker (the binary then short-circuits hooks and `state session release` returns released:false reason herding_worker). In tests/session_release.rs add one private builder that returns the `Command::cargo_bin(\"bee\")` with `env_remove(\"BEE_HERDING_WORKER\")` and `env_remove(\"BEE_HERDING_JOB_ID\")` and route all its spawns through it. In tests/pi_plugin_contracts.rs add `env_remove(\"BEE_HERDING_WORKER\")` and `env_remove(\"BEE_HERDING_JOB_ID\")` to every real bee spawn that lacks it (the dispatch-guidance tests near 2897-3060, `pi_lifecycle_end_to_end_onboarded_repo_parity` near 7859-8550, and any other), preferably through one small local helper; leave the test near line 4769 that sets the marker on purpose. Red first: run the verify with the marker set and watch both targets fail before the change. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "BEE_HERDING_WORKER=1 PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts --test session_release && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts --test session_release",
    "must_haves": {
      "truths": [
        "pi_plugin_contracts and session_release are green with BEE_HERDING_WORKER=1",
        "they stay green without it"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/tests/session_release.rs", "substantive": "env-fenced bee builder"}],
      "key_links": ["every real-binary spawn goes through the fence"],
      "prohibitions": ["No product code change", "No code comments"]
    }
  },
  {
    "id": "iss-10",
    "feature": "issues-sweep-1023",
    "title": "Union-merge the append-only bee logs",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D9", "5e66f40a-87fb-401a-975e-61055b8d871e"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/merge.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/merge.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "docs/knowledge/areas/onboarding/managed-ignore-section.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/onboarding/managed-ignore-section.md"],
    "action": "Per D9 (issue 120). Host repos get no merge driver for bee's append-only logs, so when main and a worktree both append to `.bee/decisions.jsonl` at EOF, `bee worktree merge` aborts on a textual conflict. No Rust code writes `.gitattributes` today; onboarding only splices a `.gitignore` block (`render_gitignore_block` / `merge_gitignore_content` / `find_gitignore_markers` in onboard/merge.rs, markers `# BEE:START` / `# BEE:END` and patterns in onboard/templates.rs). Those markers are valid .gitattributes comments, so REUSE `merge_gitignore_content` and `find_gitignore_markers` unchanged for the new file; the new code is a pattern constant in templates.rs with `merge=union` for `.bee/decisions.jsonl`, `.bee/decisions-archive.jsonl`, `.bee/backlog.jsonl`, `.bee/review-candidates.jsonl`, `.bee/deferred-queue.jsonl` and `.bee/wave-ledger.jsonl`, a render function, a plan item and an apply arm that follow the gitignore block's (rg -n gitignore onboard/plan.rs onboard/apply.rs). A managed-ledger or managed-hash key for the block (like the gitignore one in onboard/plan.rs) is in scope; update the tests that pin that key list. Do NOT hand-edit this repo's root `.gitattributes`: this repo gets the block from its own onboarding via `bee dev regen`; record that in the report. Red first in onboard/tests.rs next to the gitignore block tests: onboarding a fresh repo writes the block; re-running is a no-op; host lines outside the markers survive; and a real git test — two branches that each append one line to `.bee/decisions.jsonl` after onboarding merge without conflict (follow the git-fixture style already in onboard/tests.rs or verbs/worktree/tests.rs). If the release-manifest check or regen chain names a new obligation, run it and record it. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard",
    "must_haves": {
      "truths": [
        "onboarding writes a bee-marked .gitattributes block with merge=union for the six append-only logs",
        "a re-run changes nothing and host lines outside the block survive",
        "two branches that both append to .bee/decisions.jsonl merge cleanly after onboarding"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/onboard/tests.rs", "substantive": "gitattributes block and union merge tests"}],
      "key_links": ["the gitattributes block is planned and applied like the gitignore block"],
      "prohibitions": ["No glob over every .jsonl", "No code comments"]
    }
  },
  {
    "id": "iss-11",
    "feature": "issues-sweep-1023",
    "title": "Refuse a merge up front when main holds untracked history files the branch writes",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D10", "17294ba4-bffa-465d-9806-784f03e3fcce"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/worktree-parallelism/control-plane-topology.md"],
    "action": "Per D10 (issue 122). Before `git merge --no-ff --no-commit`, `commit_main_bookkeeping` (verbs/worktree/merge.rs) runs `git add -A --` over `main_bookkeeping_roots`, which include `docs/history/<feature>`, and commits them on main. When main holds UNTRACKED copies of history files the branch also adds (e.g. CONTEXT.md, leader-check.json), that commit turns them into main-side additions and the merge meets an add/add conflict, then fails WORKTREE_MERGE_ABORT_FAILED. Git itself refuses a merge over an untracked file the branch writes, for identical and for different bytes alike (leader ran it: `untracked working tree files would be overwritten by merge`), so the files cannot be merged over. In `merge_stage` (verbs/worktree/phases.rs) the untracked-collision check (`main_dirt_excluding(main_root, &roots)` and the `blocking_untracked` list that becomes WORKTREE_MERGE_MAIN_DIRTY) skips everything under the bookkeeping roots. Extend it: untracked paths under the bookkeeping roots that the merging branch also writes (reuse the existing branch-changed-files helper; rg -n branch_changed_files verbs/worktree/) join `blocking_untracked`, so the merge refuses up front, zero mutations, with the existing WORKTREE_MERGE_MAIN_DIRTY refusal naming them — before `commit_main_bookkeeping` runs. Also make `commit_main_bookkeeping` (merge.rs) never stage those paths (pass the exclusions in, or filter specs), as a second fence. If a code comment in phases.rs says no untracked path is ever swept, delete it because it is now false; add no comments. Red first in verbs/worktree/tests.rs next to `an_untracked_path_the_branch_does_write_still_refuses_and_is_named`: main holds an untracked `docs/history/<feature>/CONTEXT.md` that the branch also adds; assert the merge refuses WORKTREE_MERGE_MAIN_DIRTY naming that path, main's HEAD did not move, and `git status --porcelain` on main shows no staged entry; watch it fail first (today it ends WORKTREE_MERGE_ABORT_FAILED or a conflict). Keep `a_failing_bookkeeping_commit_only_warns_and_the_merge_still_completes_green` and the untracked-collision pair green. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree::",
    "must_haves": {
      "truths": [
        "an untracked docs/history/<feature> file on main that the branch writes makes the merge refuse WORKTREE_MERGE_MAIN_DIRTY naming it, with zero mutations",
        "the bookkeeping commit never stages such a file"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "substantive": "untracked history merge refusal test"}],
      "key_links": ["merge_stage adds branch-written untracked bookkeeping paths to blocking_untracked"],
      "prohibitions": ["No deletion of user files on main", "No code comments"]
    }
  },
  {
    "id": "iss-12",
    "feature": "issues-sweep-1023",
    "title": "Let route --set name its lane record",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D11", "32712b8e-aef3-411f-87e1-a32fa3feaa7d"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/flags.rs",
      "packages/bee-rs/crates/bee/tests/route_lane_targeting.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/issues-sweep-1023/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/flags.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/ledger.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/gates.md"],
    "action": "Per D11 (issue 124). `run_route` in verbs/state_group/workflows.rs passes `None` as the lane record to `resolve_mutation_lock_scope` and `resolve_mutation_target(&ctx.root, None, \"route\", no_lane)`, so an unbound session can only write the bound lane or the default record and is refused while lanes are live. Its `--lane` flag is the triage lane classification (ROUTE_LANE_VALUES), so it cannot be the selector. Add `--lane-record <feature>` through the SAME selector the other verbs use (D11): `mutation_lane_selector` in verbs/state_group/flags.rs hard-reads the `lane` key, so give it (or a thin sibling that both share) a flag-key parameter, keep every existing caller on `lane`, and have run_route call it with `lane-record`; pass the result as the lane to both resolvers (refuses a missing or corrupt lane; refuses together with --no-lane); add it to keys_known; mention it in the unbound refusal's FIX text. `bee route` and `bee state route` are two registry entries over the same run_route: declare the flag on BOTH in generated/registry_payload.json by hand (follow an existing flag entry there; catalog.rs reads REGISTRY_PAYLOAD), and bump `PINNED_FLAG_COUNT` in catalog.rs by one (231 at plan time — read the current value) with a reason line in the same style as earlier bumps (that pin reason is the one comment this cell may add). Red first in tests/route_lane_targeting.rs next to `an_unbound_set_against_the_default_record_is_refused_while_lanes_are_live`: an unbound `state route --set ... --lane-record <f>` writes the route into that lane record and leaves the default record untouched, and the same through `bee route --set ... --lane-record <f>`; `--lane-record` with `--no-lane` refuses; an unknown lane refuses. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test route_lane_targeting && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee distinct_flag_vocabulary && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch",
    "must_haves": {
      "truths": [
        "an unbound session's route --set --lane-record <f> writes that lane's route",
        "--lane-record with --no-lane refuses",
        "--lane still means the triage lane"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/tests/route_lane_targeting.rs", "substantive": "lane-record selector tests"}],
      "key_links": ["run_route reads --lane-record through mutation_lane_selector and passes it to resolve_mutation_lock_scope and resolve_mutation_target"],
      "prohibitions": ["No change to the meaning of route --lane", "No code comments"]
    }
  }
]
```

## Test matrix

- Happy: each cell's red-first test — Pass when the named test is green after the fix and was red before it.
- Edge: iss-1 an undeclared cell still refuses; iss-3 a string-prefix path still refuses; iss-6 a plain gather keeps --read-only; iss-12 --lane keeps its meaning — Pass when those tests are green.
- Error: iss-1 a missing unstored packet cell refuses by name; iss-12 --lane-record with --no-lane refuses — Pass when each exits with the named error.
- Windows: iss-4 and iss-5 — Pass when the Windows CI run on the merge commit reports 0 failed; #96 closes only then.
- Herded suite (#118): the leader runs the declared `commands.test` once with `BEE_HERDING_WORKER=1` after all cells cap — Pass when it reports 0 failed targets.
- Behavior change on main vs head: iss-1 `cells add` of 2 new cells against a 3-cell packet with 1 stored — main refuses "batch contains 2", head adds both.

## Open Questions

(none)

## Out of scope

- #69 dashboard (waggledance covers the cross-project view).
- #104, #106, #119 are already fixed; the leader closes them with the evidence rows 18 and 19 after the merge (D12).
