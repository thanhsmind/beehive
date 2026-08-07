# dispatch-label-chokepoint — plan

## Why this is the fourth attempt

A dispatch label should say what the work is. work-visibility D2 (`4439bd7e`,
2026-07-24) required it. It was ignored for a month because nothing produced
one. bee 2.2.6 made `prepare` produce one — for `kind == "cell"`, on the claude
runtime, and nowhere else. A live agent list photographed the next day still
read `Execute cell etom-nid-mapping-6`.

The audit that followed found four gaps:

| Gap | Evidence |
|---|---|
| codex `task_name` is the bare cell id | `prepare.rs:687` — `Some(c) => tpl(vget(c,"id"))` |
| claude `gather`/`reviewer`/`advisor` render `<kind> (<model>)` | `prepare.rs:726` — only `kind=="cell"` reaches the title |
| cli-executor payloads carry no label at all | the Bash branch sets `command` + `stdin` only |
| the guard reads the label and only logs it | `model_guard.rs:732`, then `log_dispatch` |

The pattern behind all four: **every fix targeted one branch of a fan-out while
the rule was stated over the whole fan-out.** Labels exist across runtimes ×
kinds × transports. Each patch touched one cell of that grid, and the untouched
cells never showed up as failures — they showed up weeks later as a screenshot.

So this plan does not patch a fifth cell. It closes the grid and adds the device
that makes an uncovered cell impossible to ship.

## Shape

Two cells, disjoint files, parallel.

### dlc-1 — every runtime × kind produces a subject

`verbs/drivers/prepare.rs`, `verbs/drivers/tests.rs`

- The subject is computed **once**, before the transport branch, and every
  branch uses it. Today it is computed inside the Agent arm, which is exactly
  how codex was missed.
- `kind == "cell"`: subject is `<id>: <title>`, as today.
- Other kinds: `prepare` accepts an optional `--purpose <one line>`, and the
  subject becomes `<kind>: <purpose>`. Without it the label keeps today's bytes
  — back-compatible — but the guard (dlc-2) will say so.
- codex `task_name` carries the same subject, sanitized to whatever that field
  accepts, instead of the bare id.
- cli-exec has no label field in its payload. That is a **recorded limit**, not
  silent: the cell states it in a comment so the next reader does not think it
  was overlooked.

**The anti-recurrence device**: a matrix test enumerating every
`(runtime, kind)` pair the dispatcher supports, asserting each produced label
carries a subject and is not merely a model name. It reads the supported sets
from the constants (`DISPATCH_RUNTIMES`, `DISPATCH_KINDS`) rather than a
hand-written list, so **adding a runtime or a kind fails the test until it is
labelled**. This is the part that makes "again" impossible rather than
unlikely.

### dlc-2 — the guard repairs a label at the chokepoint

`hooks/model_guard.rs`

Every dispatch passes the model-guard hook — including one written by hand that
never called `prepare`, which is precisely the case the screenshot shows and the
case no amount of fixing `prepare` can reach.

The hook already reads the label (`model_guard.rs:732`). It gains one step: when
the dispatch names a cell (the worker prompt carries `Assigned cell id: <id>`)
and the label does not carry that cell's title, rewrite it to the prepared form
and announce the repair on the advisory channel.

**Repair, never refuse.** The precedent is `ask-guard-autofix` D1/D2: a
mechanically fixable violation is repaired and announced, and the deny is
reserved for what cannot be fixed. Nobody loses a dispatch over a label.

Every failure to resolve — no cell id in the prompt, no cell record, an
unreadable record — leaves the label untouched. The repair is best-effort by
construction; it never blocks and never errors.

## Verification

`commands.test` at every cap. Beyond it:

- dlc-1: the matrix test above; plus a case per gap — codex `task_name` carries
  the title, a non-cell kind with `--purpose` renders it, a non-cell kind
  without `--purpose` keeps today's exact bytes.
- dlc-2: a bare-id label on a cell dispatch is rewritten; an already-correct
  label is left byte-identical; a dispatch naming no cell is untouched; an
  unreadable cell record is untouched and does not error.

## What this does not do

- No dispatch is refused over a label.
- The cli-exec transport gains no label — no field exists to carry one. Named
  as a limit, in the code and in the area spec.
