# wave-guard-gaps — locked context

Two guard gaps, both found by the `skill-report-stamps` wave (three workers,
one shared worktree). Both are reproduced. Do not re-derive them.

## Gap 1 — `affects_skills` has an undocumented, unvalidated format

`affects_skills` holds repo-relative **paths**, not skill names. The proof is
the existing test at
`packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:6780`, which uses
`["skills/predicted/SKILL.md"]`.

Nothing enforces or documents that:

- `bee cells add` validates only "array of strings"
  (`addCell: "affects_skills" must be an array of strings.`).
- `bee cells add --help` says "flat arrays, [] if none" and names no format.
- So a bare name (`"bee-reviewing"`) is accepted at add time and only
  explodes at **cap**, inside `sync_refusal` check (c)
  (`packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs:75-107`).

The refusal that results reads like a bee bug rather than an input error:

```
touched but unpredicted: skills/bee-reviewing/SKILL.md;
predicted but untouched: bee-reviewing
```

All three workers in the wave read that as "the door can never match" and
capped with `--sync-ack`. A guard that always fires teaches agents to ignore
it — that is the actual damage.

**The door's comparison is CORRECT and is not to be changed.** The defect is
that the wrong format is caught at the wrong end of the cell's life.

### Locked fix

1. `bee cells add` refuses an `affects_skills` entry that is not a
   repo-relative path under `skills/`. The refusal names the entry and the
   exact replacement: for a bare `<name>` that resolves to an existing
   `skills/<name>/SKILL.md`, say so literally. Whole-batch validation rules
   are unchanged — every bad entry is named in one call, nothing is written.
2. The same validation applies wherever a cell's `affects_skills` is written
   (`cells update`), so a backfill cannot smuggle in a name.
3. `bee cells add --help` names the format on the `affects_skills` line.
4. `sync_refusal` check (c) keeps its comparison, but when an unfulfilled
   prediction is a bare name whose `skills/<name>/SKILL.md` exists, the
   refusal says so and names the path. This is the belt-and-braces line for
   cells written before the validation lands.

`affects_specs` has no cap-time door today; leave it alone.

## Gap 2 — the concurrent-worker git guard is blind inside a granted worktree

`gc-2` in `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs`
(`evaluate_git_invocation`) is meant to stop a worker committing the shared
index while siblings hold work in the same checkout. The classifier
(`classify_concurrent_tree_verb`,
`packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:294-331`) is
correct: a bare `git commit` with no `-- <paths>` is classified, with the
reason "with no explicit `-- <paths>` pathspec it commits whatever sits in
the SHARED index".

It never fired during the wave. Reproduction, run with identical live state,
two probe reservations plus three live sessions:

```
# from the MAIN checkout
$ echo '{"tool_name":"Bash","tool_input":{"command":"git commit -m wip"}}' \
    | .bee/bin/bee hook write-guard
bee concurrent-worker git guard: `git commit` is refused because 3 workers
are live in this checkout. …
EXIT=2

# from the granted WORKTREE, same state, same second
$ echo '{"tool_name":"Bash","tool_input":{"command":"git commit -m wip"}}' \
    | .bee/bin/bee hook write-guard
EXIT=0        # silent
```

Cause, in `resolve_live_worker_count`
(`packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:395-438`):

- `own_workspace = ctx.workspace_id` — inside a granted worktree this is the
  worktree id, in main it is `"main"`.
- Agent-attributed leases are read from `root`'s own store. A worktree's
  lease store is empty: the wave's reservations were written to the control
  root. Confirmed:
  `/home/…/beehive--wt--skill-report-stamps/.bee/reservations.json` is
  `{"reservations": []}`.
- Live worker sessions come from `active_worker_session_ids(control_root)`,
  then every session whose `session_workspace_id(...) != own_workspace` is
  skipped. The wave's three workers ran **in the worktree** but under the
  orchestrator's session, which is stamped **main** — so all three were
  skipped.

Both halves resolve to zero. `count > 1` is therefore unreachable inside a
worktree, which is exactly where parallel workers run.

The cross-checkout data needed to fix this already exists: the mirrored
**holds** ledger carries `holder` (worktree id or `main`), `feature`,
`session`, and `cell` — that is what `bee reservations list` renders under
its `cross_worktree:` section.

### Locked fix

Inside a granted worktree the guard must count the workers that actually
share **that worktree's** index, and it must reach the control root to do it.
Reuse the existing mirrored-holds/lease reading rather than inventing a
second reservation reader.

Three constraints, all of them hard:

1. **Deny-more only, never deny-less.** Every case that denies today must
   still deny. The main-checkout verdict is unchanged, byte for byte.
2. **A solo session in a worktree stays unblocked.** One worker in its own
   worktree must keep committing normally. A guard that blocks a solo worker
   is a worse defect than the one being fixed, so this case gets its own
   test.
3. **Fail-safe on unresolvable state.** The existing
   `WorkerCount::Unresolved` path already treats an unreadable store as "more
   than one worker". Keep that shape; do not add a silent zero.

The reproduction above is the acceptance test: with sibling workers live in a
granted worktree, the hook must exit 2 for a bare `git commit`, and must stay
exit 0 for a genuinely path-scoped `git commit -- <path>`.

## Out of scope

- Changing `sync_refusal`'s comparison semantics (gap 1 fix is validation and
  wording only).
- The `bee backlog add` unknown-`--type` refusal that does not name accepted
  values. Filed separately as a P3; not this feature.
- Any change to what `git commit -a`, `stash`, `reset`, `clean` are
  classified as. The classifier is correct.
