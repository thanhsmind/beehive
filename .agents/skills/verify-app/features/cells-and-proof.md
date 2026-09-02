# Cells and the proof line

A cell is one unit of approved work. A user adds cells, a worker claims one, does
it, and caps it with a **proof line** — `<command> — <result> — <scope reason>`.
The proof line is bee's core promise: a cap whose result segment reads `red` is
refused outright, so "done" can never mean "I said so".

## Sub-features

- `cell-add` adds one cell, or a whole slice, from stdin or a file.
- `cell-add-validate` `--dry-run` reports every schema problem and writes nothing.
- `cell-claim` claims an open cell for a named worker.
- `cell-claim-gated` refuses a claim while the execution gate is unapproved.
- `cell-cap-red` refuses a cap whose proof line result is `red`.
- `cell-cap-green` caps the cell and stores the proof line on its trace. The
  result segment is a CLOSED set — `green:live`, `green:unit`, `green:static`
  (`cells/finish_support.rs:81-89`). A bare `green` is refused on write; the
  read path stays tolerant of caps recorded before that rule.
- `cell-claim-contract` refuses a claim on contract grounds
  (`cells/handlers_write.rs:1248-1436`): `CONTRACT_UNCITED` when a test-writing
  cell cites no `contract:<name>` decision, `CONTRACT_UNSETTLED` when a cited
  decision carries an open revisit trigger, and `CONTRACT_RETIRED` when one has
  left the active set.

## How to get to it (user POV)

- Pipe a cell object to `bee cells add --stdin --json`.
- Point at a file with `bee cells add --file <path> --json`.
- Run `bee cells claim --id <id> --worker <name> --json`.
- Run `bee cells cap --id <id> --files <list> --report '<json>' --json`, or its
  porcelain spelling `bee cells finish` / `bee finish`.
- Run `bee cells list --json` and `bee cells show --id <id> --json` to read back.

## Driving it with control-bee

Preconditions:

- A launched sandbox, `control-bee doctor` fully `ok`.
- Feature `demo-note` started and the merged gate approved — see
  [feature-gates](./feature-gates.md).

- **Validation refuses an incomplete cell.** Pipe a cell missing `action` and
  `verify`. Run
  `printf '{"id":"demo-note-1","feature":"demo-note","title":"Add a note file","lane":"small","role":"code","affects_skills":[],"affects_specs":[]}' | control-bee cli -- cells add --stdin --dry-run --json`.
  The payload reports `dry_run: true`, `ok: false`, and a `problems[]` array
  naming `action` and `verify` by name. `control-bee cli -- cells list --json`
  shows no cell was written.
- **Add a complete cell.** Run
  `printf '{"id":"demo-note-1","feature":"demo-note","title":"Add a note file","action":"Write NOTE.md","verify":"test -f NOTE.md","lane":"small","role":"code","affects_skills":[],"affects_specs":[]}' | control-bee cli -- cells add --stdin --json`.
  The payload echoes the cell with `status: "open"` and an empty `trace`.
- **Claim it.** Run
  `control-bee cli -- cells claim --id demo-note-1 --worker verify-w1 --json`.
  The payload reports `status: "claimed"` and `trace.worker: "verify-w1"`.
- **Register the worker.** A `small`-lane cap requires the claiming worker to
  appear in the state record. Run
  `control-bee cli -- state worker add --nickname verify-w1 --cell demo-note-1 --tier generation --status working --json`.
- **Do the work the cell describes.** Run
  `printf 'a note\n' | control-bee put NOTE.md`, then
  `control-bee sh -- git add -A` and
  `control-bee sh -- git commit -m "add NOTE.md"`. Read the commit with
  `control-bee sh -- git rev-parse --short HEAD`.
- **A red proof line is refused.** Cap with a `red` result segment. Run
  `control-bee cli -- cells cap --id demo-note-1 --files NOTE.md --report '{"outcome":"note added","commit":"<sha>","files":["NOTE.md"],"tests":"test -f NOTE.md — red — touched NOTE.md","deviations":[]}' --json`.
  The `.exit` file holds `1` and the payload's `error` reads
  `result segment is "red" — a red is fix-first, never a cap`.
  `control-bee cli -- cells show --id demo-note-1 --json` still reports
  `status: "claimed"`.
- **A green proof line caps it.** Re-run the same command with the result segment
  changed to `green:unit`. The payload reports `status: "capped"` and
  `trace.report.tests` holding the proof line verbatim.
- **A bare `green` is NOT accepted.** Run it once with the result segment as
  plain `green`. The `.exit` file holds `1` and the payload's `error` reads
  `result segment is "green" — a cap records HOW the change was shown to work`,
  naming the three legal values. Drive this: the older form is still written
  from memory, and this is the check that catches it.
- **Proof.** Run `control-bee snapshot capped`. The snapshot's
  `cells/demo-note-1.json` shows `status: "capped"` with the green proof line on
  `trace.report.tests`, `git-log.txt` shows the commit the report names, and the
  earlier refusal's `.exit` file is still `1` in the evidence dir.

## Gotchas

- The `--report` value is a JSON **string** with exactly five keys: `outcome`,
  `commit`, `files`, `tests`, `deviations`. An unknown or missing key is refused
  by name. `mistakes` is an optional sixth.
- The proof line has three segments split on the first two ` — ` separators, and
  the separator is an em dash surrounded by spaces, not a hyphen.
- **`bee cells cap --no-mistakes` cannot be used at all.** Bare, it is refused as
  `unsupported_argument_shape` (`no-mistakes` is absent from `FLAG_ALONE_BOOLEANS`
  in `verbs/reservations/flags.rs:31`, so the parser swallows the next token).
  Given a value, `bool_flag` maps the string `"true"` to `false` on purpose
  (`verbs/cells/util.rs:122`), so the cap succeeds and writes **no**
  `trace.no_mistakes`. Verified on a real capped cell. Settle the mistakes
  answer with `bee mailbox reflect --no-mistakes` (bare — that one is a true
  bare-only flag), or put a `mistakes` array in `--report`.
- A `small`-lane cap refuses without a registered execution worker. `tiny` lanes
  run inline and skip that requirement, and `--inline-reason` is the audited
  escape on the higher lanes.
- `bee cells finish` additionally requires a `cell: <id>` trailer on the commit
  the report names; `bee cells cap` does not check for one. A recipe that swaps
  the porcelain in for the plumbing must add the trailer.
- A cap needs non-empty `--files` from the `small` lane up. A cell that changed
  nothing is a drop or a NOOP, not a cap.
- Claiming without a recorded route warns the first time and **refuses the
  second** time in the same session. Record the route before claiming.
- A claim on a repo with no `.bee/logs/test-results.json` warns that it cannot
  tell whether the base is green. That warning goes to stderr and is not a
  failure.
- With `--json`, a refusal body goes to **stdout** and the shell may still look
  successful. Assert on the recorded `.exit` file.
