# Port notes — skill bee-fanout, cell ppb-17

Source: pstack skill `swarm` (`SKILL.md`, no reference files) at commit
`b9ddc83`, MIT, copyright 2026 Lauren Tan. The notice is
`skills/bee-fanout/LICENSE.pstack`, a verbatim copy of the pstack `LICENSE`. This
file is the only place a pstack-only tool is named; no ported file names one.

Name: `bee-fanout` (decision `6ca3cd62`), because `bee-swarming` already runs
approved cells to done. The description and the opening paragraph say the two
differ: a fanout worker owns a slice of a question or an arm of a race and
returns a report; work that changes files in a worktree is a cell.

Kept as the base, with the swaps below: the four phases (Frame, Fan out,
Aggregate, Report), the three shapes (slices, race, mixed), the declared
race rule (`first pass`, `rank all`, `best-of`), N as total workers, the
stand-alone brief, the `PASS`, `ISSUES`, `BLOCKED` report vocabulary, the
N-1 dropout rule, the compact result table and the one-report ending.

Frontmatter follows `bee-how`: `name`, a description of 237 characters (what it
does, when to use it, what it is not for) and `metadata` with `ecosystem: bee`
and a `bee-cli` dependency that degrades to the leader covering the slices
itself. The source's `disable-model-invocation: true` is dropped, as in
`bee-architect`, `bee-arena` and `bee-interrogate`: the description names the
explicit trigger.

## Tool swaps

- "Spawn all N workers in one message with `subagent_type: generalPurpose`,
  `run_in_background: true` and the configured model" became one
  `bee dispatch prepare --runtime claude --kind <gather|advisor> ... --json`
  per worker, all issued in the same turn, each returned payload run in the
  background and used exactly as returned. No model or subagent type is named.
- The worker seat: a worker that only reads and reports goes out as
  `--kind gather`; a worker whose answer is judged or compared goes out as
  `--kind advisor --role lane-1`, `lane-2`, `lane-3`, a fourth or more reusing a
  lane role, as in `bee-arena`.
- "Pick the worker model from `swarm workers` in `~/.cursor/rules/pstack-models.mdc`
  when present, otherwise a named default; for a model race, name each arm's model
  up front" became the bee team table: which model a role runs on is a team
  setting. A model race names each arm's role up front and the model each payload
  returned. The default model name is dropped.
- "Give each worker its own writable output (worktree, branch or
  `/tmp/swarm-<slug>/worker-<n>/`)" became: gather and advisor workers are
  read-only, so the leader saves each answer verbatim at
  `/tmp/fanout-<slug>/worker-<n>/`. The shared-path rule cites
  `bee-principle-separate-before-serializing-shared-state`, as `bee-arena` does.
  Workers that write files in a worktree are cells, which `bee-swarming` runs.
- A judge for `rank all` or `best-of` is `bee dispatch prepare --runtime claude
  --kind reviewer --json` with no `--role`, read-only, seeing the arms by path
  label with no role or model names. The source has no judge; it only names the
  rule. This follows `bee-arena` Phase C.
- "Open a todolist" became "open a task list" (the host's own list; no tool named).
- `bee blind check` and the blind-lane procedure in
  `bee-hive/references/gates-and-delegation.md`: a design question that must end
  in a dossier is not a fanout. Added, as in `bee-arena`, so no second design
  fan-out mechanism exists.
- Added, not in the source: the "this is not `bee-swarming`" paragraph; the rule
  that a race brief carries no leader leaning; the line that a `PASS` stands once
  the evidence it points at has been seen
  (`bee-principle-verify-before-reporting`); the line that workers all on one
  model share its blind spots; a `Handoff` section ("Invoke bee-hive skill"), as
  `bee-interrogate` has.

## Pointers kept as plain sentences

None. Every skill and principle the source names exists in bee; the source names
none.

## Dropped steps

- Cloud environment: `environment: "cloud"`, `environment: "local"` and the
  "local only when the worker needs the user's computer" rule. No bee counterpart:
  where a payload runs is the team table's and the dispatch door's choice, not the
  skill's.
- `cloud_base_branch` for a worker that must start from a non-default pushed
  branch. No bee counterpart: a dispatch payload takes no base branch.
- The cloud concurrency-limit remark ("N is total workers, not the cloud
  concurrency limit"): its subject is gone; "N is total workers" stays.
- The default worker model name and the `pstack-models.mdc` file: replaced by the
  team table, as above.
- The loop, stacked-PR tool, review bot, `poteto-agent`, arena, architect, how
  and why are not cited in the source, so nothing was dropped for them.

## Trigger fixture

Six briefs added to `packages/bee-rs/crates/bee/tests/fixtures/skill-triggers.json`:
four expect (one Vietnamese) and two near (expect `bee-swarming` and `bee-arena`),
none naming the slug.
