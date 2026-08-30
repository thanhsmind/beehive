# The waggledance cockpit-supervisor seat is live

Provenance: from waggledance@faa6945, correlation id p-bea191e4, 2026-08-30.
This closes the loop on docs/discovery/slp-human-up/wd-cockpit-request.md, which asked
waggledance to take the cockpit-supervisor seat (beehive decision b59e50c8, touches 8fea3561).

## What shipped on the waggledance side

waggledance now carries a `waggledance-supervisor` agent skill, installed by
`waggledance doctor` alongside its viewer skill. The seat's whole procedure: take a spec
from the human, mint a correlation id, check `waggledance_ask_state` for a live lead,
dispatch a lead into the target repo with a task whose first line is
`spec-drop <corr-id> from <repo>@<commit>`, then track the run in `waggledance_runs`.

The seat adds no new authority. It uses only the three dispatch-family MCP tools that
already existed, holds no merge power, and its skill forbids it from merging or asking
for one. waggledance's own locked decisions required this: waggledance never decides what
to dispatch, it only executes dispatches safely.

The per-project orchestrator-dispatch opt-in is now ON for beehive, on the owner's
explicit word (2026-08-30). This message is the first real drop through the door.

## The one thing beehive has to decide

While shaping this, waggledance found a gap it cannot close from its side.

The original request's acceptance criteria include "merge to main stays human-only".
That is true of the seat — it has no merge power. It is **not** currently true of this
repo. beehive's `.bee/config.json` records:

    gate_bypass: "full"
    uat_stop:    "close"

By bee's own contract, `uat_stop: "close"` means *the agent merges on green without
asking*. Combined with `gate_bypass: "full"`, a lead opened here — by the waggledance
seat or by anything else — may self-approve Gates 1 and 2 and then merge to `main`
unattended. Only Gate 3 / UAT still stops it.

That may well be exactly what the owner wanted when gate_bypass was raised to `full` on
2026-08-30 for autonomous operation. It is recorded here rather than quietly assumed,
because it is the difference between "merge stays human" as a written guarantee and as a
hope.

For this first drop, waggledance contained the risk from its own side by sending a
file-and-stop task — the lead files the spec and stops, with no routing and no merge.
That is containment by instruction, not by construction, and it only holds for drops the
seat sends.

### Options

1. **Keep `uat_stop: "close"`.** Autonomous runs stay fast. Record the accepted risk so
   nobody later reads criterion 4 as a guarantee it is not.
2. **Set `uat_stop: "merge"`.** The merge door blocks for a human by construction, and
   criterion 4 becomes literally true. Costs a human stop on every autonomous run here,
   not just foreign-origin ones.
3. **Distinguish foreign-origin work.** A narrower rule where a spec-drop-derived feature
   carries a stricter merge door than locally-originated work. No machinery for this
   exists today in either repo; it would need its own shaping.

waggledance has no opinion it is entitled to hold here — the config belongs to this repo
and the decision belongs to its owner.

## Pointers

- waggledance's lock for the seat: `docs/history/wd-supervisor-seat/CONTEXT.md` in the
  waggledance repo (decisions D1-D5).
- The seat's procedure: `docs/waggledance-supervisor-skill-template.md`, same repo.
- The original request: `docs/discovery/slp-human-up/wd-cockpit-request.md`, this repo.
