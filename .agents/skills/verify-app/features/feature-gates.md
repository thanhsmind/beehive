# Feature start and gates

A user starts a named feature, records how the work was triaged, and then answers
bee's gates. The gates are what stop an agent from editing source before the
human has approved the shape — approving them is the human's move, and bee
records who approved, when, and under what bypass level.

## Sub-features

- `feature-start` starts a feature and resets all four gates.
- `feature-start-guard` refuses a start when the workspace is not clean.
- `route-set` records the triage (class, lane, flags, product-file count).
- `gate-merge` approves shape and execution together in one call.
- `gate-named` approves or unapproves a single named gate.
- `gate-audit` an auto approval records its bypass level and reason.

## How to get to it (user POV)

- Run `bee state start-feature --feature <slug> --mode <mode> --json`.
- Run `bee route --set --class <c> --lane <l> --flags <f> --files <n> --json`.
- Run `bee gate --merge --approved true --json` to answer the merged gate.
- Run `bee gate --name <gate> --approved true --json` for one gate.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- `bee orient --json` reports `phase: "idle"` and `feature: null`.

- **Start a feature.** Run
  `control-bee cli -- state start-feature --feature demo-note --mode standard --json`.
  The payload reports `phase: "exploring"`, `feature: "demo-note"`,
  `mode: "standard"`, every entry of `approved_gates` `false`, an empty
  `workers` array, and a `next_action` naming the feature.
- **A second start is refused.** Run the same command again with
  `--feature other-thing`. The `.exit` file holds a non-zero code and the payload
  carries an `error` naming the unclean workspace. `bee orient --json` still
  reports `feature: "demo-note"` — the refusal mutated nothing.
- **Record the route.** Run
  `control-bee cli -- route --set --class feature --lane small --flags "" --files 1 --json`.
  The payload reports `class: "feature"`, `lane: "small"`, `flags: []`,
  `product_files: 1`, and a `worktree` object whose `required` is `true` with the
  exact `command` to create it.
- **Read the route back.** Run `control-bee cli -- route --show --json`. It
  returns the same four values.
- **Approve the merged gate.** Run
  `control-bee cli -- gate --merge --approved true --json`. The payload's
  `approved_gates` now has `shape: true` and `execution: true`, with `context`,
  `review` and `uat` still `false`.
- **Approve one named gate.** Run
  `control-bee cli -- gate --name uat --approved true --json`. Only
  `approved_gates.uat` flips.
- **Unapprove it again.** Run
  `control-bee cli -- gate --name uat --approved false --json`. It flips back to
  `false`.
- **Proof.** Run `control-bee snapshot gates`. The snapshot's `state.json`
  carries the same `feature`, `phase` and gate values, and its `gate_records`
  entries name the approving `actor` and the approval time.

## Gotchas

- `--files` on `bee route --set` is a **count**, not a file list. Passing a path
  is refused with `must be a non-negative integer`.
- `--flags ""` is how you say "zero flags". Omitting `--flags` entirely is a
  different call.
- `bee gate --merge` and `bee gate --name` are mutually exclusive. Passing both
  is refused.
- An `--actor auto` approval requires **both** `--bypass-level` and `--reason`.
  bee refuses a bypassed approval that is less traceable than a stopped one.
- The `uat` gate is never bypass-approved. A recipe that auto-approves every gate
  will not get `uat` for free.
- Starting a feature resets all four gates. A recipe that approves gates first
  and starts the feature second proves nothing.
- `bee gate` is the flow spelling of `bee state gate`, and `bee route` of
  `bee state route`. Identical behavior; either name is a valid entry point.
