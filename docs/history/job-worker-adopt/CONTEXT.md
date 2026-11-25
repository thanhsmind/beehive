# job-worker-adopt — context

Follow-ups from the agy-as-leader spike (`docs/history/agy-leader-e2e/report.md`):
F11 (PBI p-3f1d7db8), F13 (p-41a23d81), F10 (p-e8eeae96) and F9 (p-a0f03c4a).
The leader triaged all four as build and runs them as one feature, because
each is small and all come from the same spike.

## Locked Decisions

| # | Decision | Why |
|---|---|---|
| D1 | `bee cells finish --from-job` adopts the herding job's worker when the cell's `trace.worker` has no registered row for the cell. The job record must name the cell and its result outcome must be `done`. Finish registers a worker row (nickname = the job id) for the cell, sets the cell's `trace.worker` to that nickname, writes an audit row of kind `worker-adopt`, then caps as before. | F11: a leader died before cap and the next leader could not cap a cell that a finished herding job had done. The job record proves an execution worker did the work. |
| D2 | The settle line that `bee herding run` prints reads the cell's proof floor (`cell_proof_floor` with the route class). When the worker's proof result is below the floor, the line prints `--proof-result <floor>`, and a second line says to pass that floor only with its evidence. | F13: the hint printed `green:unit` for a test-role bugfix cell whose floor is `green:red-first`, so the printed command was refused. |
| D3 | A write-guard refusal for an owner-only target (governed config key, the team layer file, a write under the home dir outside the repo) carries the marker `[owner-only: <target>]`. The Stop owner-ask check passes a reply that matches an owner-ask phrase when the reply names a target that an `[owner-only: <target>]` refusal in the session transcript named. | F10: the Stop hook refused a turn end that asked the owner to run a write the guard refuses for the agent. |
| D4 | The multi-session playbook says: a live leader that opens a parallel leader for new work in the same project first runs `waggledance state` for a run already on the same PBI (and steers it with `--follow-up-of`), and otherwise dispatches with `--no-handoff`. | F9: a parallel leader dispatch was folded into a handoff of the live run, and two leaders worked one PBI. |

## Read set

### Knowledge

| Path | Why |
|---|---|
| docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md | the run verb, its settle line, and finish --from-job |

### Code

| Path | Why |
|---|---|
| packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:537-548 | the registered-worker check at cap |
| packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1780-1835 | finish --from-job reads the job record |
| packages/bee-rs/crates/bee/src/verbs/state_group/workers.rs:163-175 | register_worker_for_cell |
| packages/bee-rs/crates/bee/src/herding/run.rs:6157-6170 | settle_command |
| packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:111-135 | cell_proof_floor and class_proof_floor |
| packages/bee-rs/crates/bee/src/hooks/session_close/owner_ask_net.rs:39-84 | owner-ask phrase match and Stop evaluation |
| packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:1587 | governed config key deny |
