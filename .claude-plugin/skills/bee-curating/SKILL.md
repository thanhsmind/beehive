---
name: bee-curating
description: >-
  Drain the knowledge debt agents flag — capture stubs, unsettled promote proposals, orphaned scribing — in one bounded pass. Use when /bee-curating is invoked by a scheduled job or the owner. Not for concept audits (bee-knowledge-upkeep) or recording one fact (bee-capturing).
disable-model-invocation: true
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The debt signals, the queue settlement and the run record all come from the vendored binary. Absent, the stubs and proposals can be read and merged by hand, but nothing says what is owed, what is settled, or whether the run was recorded.
---

# Curating — drain the flagged debt

Agents flag knowledge debt well and settle it badly: a capture stub, a promote
proposal at close, a feature that ended without a scribing run. This skill is
the separate pass that settles that debt, on a cadence, in a fixed budget. It
is the loop of `docs/history/curator-drain-2026-11-22/audit.md`, bounded.

Read `.bee/expertise/knowledge.md` for the craft and `bee knowledge consolidate`
for the printed writing rules. This skill restates neither, and it never audits
concepts — that is `bee-knowledge-upkeep`.

## Outcomes

Pick one, and say which:

- **clean** — nothing owed settled into the bundle: every stub, proposal and
  orphan was read and none held something new.
- **changed** — at least one concept, ledger row or queue record changed; the
  bundle change is committed.
- **blocked** — a step could not finish or a proven change could not ship
  safely. Say exactly what blocked it.

## Edit scope

Edit `docs/knowledge/` and the queue and ledger records the commands below
write. Never edit product code: a concept that describes behavior the code no
longer has is doc drift (fix the concept) or a product regression (report it,
keep it out of this pass). The generated `index.md` files are
`bee knowledge index` output: regenerate them, never hand-edit them.

## When a run is due

Run `bee upkeep due --kind curator --json`; exit 10 means a run is due. The
reasons name the debt: `capture_queue`, `promote_unsettled`,
`scribing_orphaned`. The cadence is `curator.cadence_hours` of the project's
`.bee/config.json`; debt inside the window is not due. A run the owner starts
by hand needs no due signal.

## Pass

Each step has its own cap, because a pass that tries to drain everything runs
out of context before it records anything. Name what the caps left over for
the next run.

1. **Capture queue.** `bee capture list`, then give each stub, oldest first,
   its full spec merge by the `bee-capturing` Compound step 1, and
   `bee capture flush --id <id> --into <spec>`. A stub is never dropped or
   summarized away.

2. **Promote proposals.** At most 10, oldest first, from the unapplied list
   `bee orient` prints (each is a `docs/history/<feature>/promote-proposals.md`
   that no queue record has settled; oldest by file date). For each: read the proposal,
   apply the lines that clear the `bee-capturing` promotion bars to
   `docs/knowledge/` (extend the concept that owns the fact; one fact, one
   home), then settle it:
   `bee deferred-queue complete --kind promote --feature <feature>`.
   When nothing applies, still settle it, and record why in the pass note: a
   line the bundle already states, a lesson seen once, a claim the code no
   longer backs.

3. **Orphaned scribing.** At most 5 features, heaviest first, from
   `bee status --json` (`scribing_debt.orphaned`). Sync the areas the
   feature's capped `behavior_change` cells touched into their owning
   concepts, then
   `bee state scribing-run --feature <feature> --areas "<a,b>" --next-action "<n>"`.
   For a feature that is not the active one this appends the ledger row and
   moves no phase.

Reading fans out. One read-only reader per proposal or feature, launched
concurrently through `bee dispatch prepare --runtime <rt> --kind gather --json`,
then run the tool and payload it returns. Each reader checks the proposal or the
cells against `docs/knowledge/` and the shipped source and returns what is
present, absent or in conflict, with citations. Readers never edit. Writing
stays here, in one place.

4. **Validate.** `bee knowledge check --strict` and `bee knowledge index --check`;
   regenerate with `bee knowledge index` and run both again until they pass.

5. **Commit.** One commit of the bundle changes. For clean or blocked there is
   nothing to commit.

6. **Record the run.** Last, on every outcome — clean, changed or blocked:
   `bee audit log --kind curator-run --feature curator --note "<clean|changed|blocked> — <one line>"`.
   The newest row of this kind is the run's last-run mark. Without it the
   signal never clears and the next tick dispatches the pass again.
