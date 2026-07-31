# Stage: executing (`bee-swarming` — "Execute")

**Purpose** — Implement, verify, and cap *exactly one* parent-assigned cell as a
worker, then return a structured status token. This is the only stage that edits
source.

**When it runs** — Inside a swarming worker that received an assigned cell id.
Every lane's implementation (tiny/small included, since AO14) runs through a
dispatched execution worker — never in-session.

## Inputs
- `AGENTS.md`, `status --json`, `CONTEXT.md`.
- The assigned [cell](../register.md#beecellsfeature-njson) (`cells show --id`) and
  its `read_first` files.

## Outputs
- File edits *within the reserved `files`* of the cell.
- A `cells verify` record with output; a capped cell (`cells cap`) carrying
  `verification_evidence` for behavior-change cells.
- A short per-cell report `docs/history/<feature>/reports/<cell-id>.md`.
- **One commit per cell**, with the cell id in the message.

## Gate
None — workers never approve gates.

## State touched
[`reservations reserve/release`](../register.md#beereservationsjson),
[`cells verify/cap`](../register.md#beecellsfeature-njson), the cell's `trace`, one
git commit.

## Key rules
- **Never choose your own cell** or browse the ready list — validate the assignment,
  don't claim (D1).
- **No stubs, TODO-only, or dead code.** Capping requires a *passing recorded
  verify* — an assertion is not evidence.
- **Proof scales by `change_class × lane`** (test-economy): `cells cap` derives
  the required evidence tier from the cell's change class and lane — red-first
  proof only for `security`/`migration` (all lanes) and `high-risk` (all classes);
  a covered bugfix at tiny/small needs a targeted green test. Cap computes
  `diff_stats` and a new test suite needs a `new_suite_reason`.
- **Impacted tests only** — the dev loop runs registry-scoped/impacted suites
  (transitive tail capped, delegated to CI); the full verify suite is CI-owned.
- An architectural-change need → **STOP and `[BLOCKED]`**; never redesign inside a
  cell. Package installs always checkpoint (`[BLOCKED]`).
- Return **exactly one** status token: `[DONE]` · `[BLOCKED]` · `[HANDOFF]` · `[NOOP]`.

## Source
`skills/bee-swarming/SKILL.md` ("Execute") + `skills/bee-swarming/references/worker-details.md`
