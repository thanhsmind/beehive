# jwa-8: check of jwa-7's doctrine edits

Cell jwa-8 checks commit 5e6d5372f (cell jwa-7) against the locked decisions D1, D2 and D4 in `docs/history/job-worker-adopt/CONTEXT.md`. This check found no conflict with the locked text. It found two minor issues, listed below. Fixing them is out of scope for this cell.

## D4: parallel-leader paragraph

- `packages/bee/playbooks/multi-session.md:32-35` holds the paragraph. It matches D4 (`docs/history/job-worker-adopt/CONTEXT.md:15`) on all three points:
  - The paragraph applies to a live leader that opens a parallel leader for new work in the same project.
  - The leader first checks `waggledance state` for a run already on the same PBI and steers that run with `--follow-up-of`.
  - Otherwise, the leader dispatches with `--no-handoff`.
- The paragraph's reason ("so it is not folded into a handoff of the live run") matches F9, the reason D4 gives.
- The paragraph does not name `job-worker-adopt` D4. The cell asks only the concept to cite D1 and D2, so this is not a defect.

## D1: worker adoption on `finish --from-job`

- `docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md:880` matches D1 (`CONTEXT.md:12`) on every point:
  - The trigger is a `trace.worker` with no registered row for the cell.
  - The job record must name the cell.
  - The job result must be `done`.
  - The worker row's nickname is the job id.
  - `trace.worker` is set to that nickname.
  - Finish writes a `worker-adopt` audit row, then caps as before.
- The code matches the concept:
  - `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1876` refuses any job whose result is not done.
  - `handlers_close.rs:1934` adopts only when the job names the cell and that cell has no registered worker.
  - `handlers_close.rs:1940-1960` registers the row, sets `trace.worker` and appends the `worker-adopt` audit row.
- The concept's line "jobs whose result is not `done` keep the existing refusal" agrees with that refusal at `handlers_close.rs:1876`.
- The frontmatter `decisions` entry at `the-run-verb-and-worker-outcomes.md:11` cites "job-worker-adopt D1" with a matching summary. `handlers_close.rs` is added to the concept's anchors list.
- Minor issue: the code also skips adoption when the cell is already `capped` or `dropped` (`handlers_close.rs:1925-1934`). The concept does not mention this. It is consistent with D1, because D1 covers a cell that is not yet capped.

## D2: floor-aware settle line

- `the-run-verb-and-worker-outcomes.md:878` matches D2 (`CONTEXT.md:13`) on every point:
  - The settle line reads `cell_proof_floor` with the route class.
  - A worker proof below the floor prints `--proof-result <floor>`.
  - A second line says to pass that floor only with its evidence.
  - The example (a test-role bugfix cell, `green:unit` against the floor `green:red-first`) is F13 from CONTEXT.
- The frontmatter entry at `the-run-verb-and-worker-outcomes.md:11` cites "job-worker-adopt D2" with a matching summary.
- Minor issue (inaccurate pointer): the concept says "`settle_command` computes the cell's proof floor". In the code, `settle_command` (`packages/bee-rs/crates/bee/src/herding/run.rs:6157`) only formats the command. `cell_settle_floor` (`run.rs:6177-6190`) calls `cell_proof_floor` with `route_class` (`run.rs:6183`). `settle_hint` (`run.rs:6192-6201`) prints the floor command and the evidence line. The claim about behavior is right; only the function name is wrong. The fix is to name `settle_hint` / `cell_settle_floor` at line 878.

## Citation form

- The body paragraphs at lines 878 and 880 write the cite as `` (`job-worker-adopt` D2) `` and `` (`job-worker-adopt` D1) ``. A backtick sits between the feature name and the D-ID.
- The verify pattern `rg -n 'job-worker-adopt D[12]'` therefore matches only the frontmatter line 11, not the body paragraphs. The verify still passes because line 11 carries both cites.

## Verify

```
$ rg -n -- --no-handoff packages/bee/playbooks/multi-session.md && rg -n 'job-worker-adopt D[12]' docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && .bee/bin/bee dev release-manifest --check
35:leader with `--no-handoff` so it is not folded into a handoff of the live run.
11:  decisions: ["leader-call-thrift D2 (auto-settle passes ...
release_manifest --check: 994 file(s) match stored manifest, 1 unhashed artifact(s) present
exit 0
```
