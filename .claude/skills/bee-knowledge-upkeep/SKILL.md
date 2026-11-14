---
name: bee-knowledge-upkeep
description: >-
  Periodic audit that keeps a project's `docs/knowledge/` bundle honest: read the recorded signals, pick at most 5 concepts, one read-only source reader per concept, revise in place, one worktree and one cell of proven corrections. Use PERIODICALLY, when a repo already carries a knowledge bundle — after a release, when a loaded concept was reported wrong, when a concept grades orphan, or when a concept is loaded often and never used. Not for creating a bundle where none exists — that is bee-knowledge-setup — and not for the in-flow write of one settled fact, which stays with bee-capturing.
disable-model-invocation: true
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The signals come from `bee knowledge report` and `bee knowledge check`, and the proof from the same binary. Absent, the concepts can be re-read and edited by hand but nothing says which ones to pick or whether the bundle still validates.
---

# Knowledge upkeep — keep the bundle honest

A concept is true when it is written and drifts the day the code moves. This is
the upkeep loop for a bundle built by `bee-knowledge-setup` (or any
`docs/knowledge/` bundle). The unit of work is the concept the signals point at,
not the whole bundle: read a few concepts against their own sources, and fix
what the sources prove wrong.

Read `.bee/expertise/knowledge.md` for the craft ("Retire out loud" covers
retiring) and `bee knowledge consolidate` for the printed writing rules. This
skill restates neither.

## Outcomes

Pick one, and say which:

- **clean** — every picked concept held against its sources; nothing worth
  shipping. No worktree, no cell.
- **changed** — one worktree and one cell ship proven corrections to concepts.
- **blocked** — a reader could not finish or a proven fix could not ship safely.
  Say exactly what blocked it.

## Edit scope

Only edit `docs/knowledge/`. Never edit product code during a run: a concept
that describes behavior the code no longer has is either doc drift (fix the
concept) or a product regression (report it, keep it out of this change). The
generated `index.md` files are `bee knowledge index` output: regenerate them,
never hand-edit them.

## When a run is due

A run is due after a release, or when any signal below shows. The numbers are
starting defaults, not measured thresholds: change them here when they prove
wrong.

`bee upkeep due --kind knowledge --json` computes this table in one read-only
check and exits 10 when a run is due. Each reason it returns names one row
below; the table stays as the human statement of the same signals. A run that
was never recorded counts as due.

| Signal | Command that prints it |
|---|---|
| A concept reported wrong | `bee knowledge report` — the reported-wrong section, with the newest note under each concept |
| An orphan grade | `bee knowledge check --json` — the `freshness` object |
| A suspect concept loaded 5 or more times | the suspect list of that same `freshness` object, joined with the `loaded` list of `bee knowledge report --json` |
| A concept loaded 5 or more times and never used | `bee knowledge report` — the loaded-never-used lines |

Loaded-never-used is a weak signal while the report's silent cells outnumber its
answered ones: a concept may only be unreported. It never makes a run due alone,
and it is trimmed or retired only after the source reader confirms it is dead
weight.

## When knowledge is read, written and revised

| Moment | What happens |
|---|---|
| Shaping, planning, cell brief | Read: the loaded concepts reach the work |
| Cap | Write: the cell's own delta lands in the same commit |
| Cap, a loaded concept proved wrong | Revise: fixed in place in the same commit |
| Cap, a wrong concept the cell cannot fix | Report it with `--knowledge-wrong` |
| An upkeep run | Audit: re-read against sources, dedupe, merge, retire |

## Pass

0. **Bundle present.** With no `docs/knowledge/` concept, stop and route to
   `bee-knowledge-setup` — there is nothing to keep up yet.

1. **Signals.** Read `bee knowledge report --json` (reported wrong, the `loaded`
   list, never used) and `bee knowledge check --json` (the freshness grades).
   Orphan and suspect grades come from the check, never from the report.

2. **Pick.** At most 5 concepts, ranked: wrong count, then orphan, then
   suspect-and-loaded, then loaded-never-used. Pick only from concepts that were
   loaded for work. Name the candidates left over for the next run. The limit
   is 5 because a measured result shows revision gain flat from 5 to 20
   concepts and falling beyond, from needless revision and drift (ReHarness,
   arXiv 2610.12668).

3. **Source wave.** One read-only reader per picked concept, launched
   concurrently through `bee dispatch prepare --runtime <rt> --kind gather --json`,
   then run the tool and payload it returns. Each reader reads the concept, its
   `bee.sources` and its `bee.anchors`, and returns `held` or `drift` with
   citations. For a wrong report it also says whether the signal is diagnostic
   or noise. Readers never edit.

4. **Revise.** Each rule holds once:
   - New text is grounded in the concept's own `bee.sources` and `bee.anchors`;
     the failing work is only the signal.
   - A claim whose source cannot be read again is cut, or marked as a guess.
   - Revise in place, headline first (title, description, the `Read when:` line);
     never append a correction.
   - A noisy signal — an environment red, chance — revises nothing.
   - A loaded-never-used concept is trimmed, merged into its owner, or retired
     out loud with `bee.superseded_by`.

   Run `bee knowledge confirm --id <id>` on each concept that held.

5. **Validate.** Run `bee knowledge check --strict` and `bee knowledge index --check`.
   Regenerate with `bee knowledge index` and run both again until they pass.

6. **Ship or stop.** For changed: one worktree and one cell of proven
   corrections, landed with `bee worktree merge`. For clean or blocked: no
   worktree; report the outcome and the coverage honestly, the candidates left
   over included.

7. **Record the run.** Last, on every outcome — clean, changed or blocked:
   `bee audit log --kind knowledge-upkeep --feature <feature or upkeep> --note "<outcome> — <one line>"`.
   The newest row of this kind is the run's last-run mark. Without it the run
   stays due.

Keep concise run notes (concepts picked, held or drift, outcome) in a scratch
location; don't commit them.
