# budget-fence-removal — Learnings

**Date:** 2026-07-29
**Lane:** standard · **Cells:** bfr-1..bfr-7 capped, bfr-8 dropped (user scope cut)
**Close:** full verify green, 117/117 suites · feature-verify `85718036`
**Governing decision:** `8f63adb4` · **Scope amendment:** `3a756908`

## What the work was

bee had bound itself to hard byte ceilings on its own instruction text: a blocking verify
suite (`scripts/skill_budget_fence.mjs`) enforcing a per-skill byte budget through a one-way
ratchet that would lower a recorded number but refuse to raise it. Its failure message was the
whole problem in one line:

> `pay for new text by removing text, not by appending`

A user ruled that this optimizes for smallness rather than information, and that a size diet is
a deliberate one-off event, never a standing law. The fence, its baseline, the `AGENTS.md` byte
assertions, the authoring doctrine, the knowledge concept describing it, and seven decisions
authorizing it were all removed. Guards that assert *meaning* were kept and are still blocking.

## The strongest finding: a size law does not schedule a re-read, it just taxes one

The old doctrine carried a defence of itself. `placement-and-anchoring.md` argued that paying for
an addition by trimming was valuable *because* the trim forced a re-read that surfaced stale
cross-references — two of them, found that way.

This feature ran the re-read directly, with no size law involved, and found **18** defects of the
same class. The observation was real; the conclusion drawn from it was not. A size law is an
expensive and lossy scheduler for a document review. Schedule the review.

## Failures, ranked by what they cost

### 1. Fixing the line a report cited, not the law it was evidence of — four times, in one file

`placement-and-anchoring.md` stated the abolished rule in four registers: the frontmatter
`decisions:` list, Business Rule R5, an Edge Cases bullet, and a Pointers line. Cell `bfr-7` fixed
the Pointers line its action named. The scribing sync caught R5. A compounding analyst caught the
Edge Cases bullet — **still live in HEAD at that point** — and a fourth statement in a neighbouring
concept.

The failure mode recurred inside its own fix, twice. `docs/knowledge/` is what `bee knowledge
context` feeds to future planning, so a false concept there is read as ground truth.

Promoted as a pattern. The real fix is executable and is filed: extend
`scripts/tests/test_instruction_size_law.mjs` to grep `docs/knowledge/` bodies for the shape of a
size law, excluding frontmatter provenance citations (backlog, P2).

### 2. A selector that matches nothing reports green

`bfr-3`'s verify was `run_verify.mjs --only test_instruction_size_law,...` — naming the suite the
cell was about to create. Dry-run against `filterSuitesByOnly` showed it selected **three** suites,
not four, and would have passed without ever running the new one. Fixed by invoking the suite's
path directly ahead of the `--only` form.

Generalizes to any substring-based test selector: a token that matches nothing is silently a no-op,
and the run is green for the wrong reason. Promoted as a pattern.

### 3. A cell that adds a suite owes the derived-registry regen, and nothing enforces it

`bfr-1` regenerated `scripts/impact-registry.json` after deleting the fence. `bfr-3` then *added* a
suite, which changes `run_verify.mjs`'s live `SUITES`, from which the registry is derived. Slice 1's
feature verify came back 116/117 on exactly that drift. Cost a whole fix-first cell (`bfr-4`,
commit `417d0f11`).

`REGEN_GUARDS` keys obligations on manifest- and ledger-covered roots; nothing keys on "this cell
adds a file under a `DISCOVERY_ROOTS` path". Filed with a concrete shape (backlog, P2).

### 4. A fixture riding a real declaration must seed every declared literal

`test_verify_cache.mjs` case (10) was retargeted onto `packages/bee/tests/test_misc.mjs`, whose
declaration lists two literal inputs plus a glob. The plan's edit table named one. `closureShaFor()`
returns `null` when *any* declared literal is missing — meaning "never cache" — so the suite ran
green and the cache entry was simply absent, surfacing far away as `undefined !== "green"`.

The plan quoted the correct two-element array in one section and synthesized a one-element
instruction in another. A synthesis error inside one document, not a research gap.

### 5. A locked decision was cut without the amendment its own governance clause required

`CONTEXT.md` states that changing a locked decision requires the user, a new D-ID, or an explicit
supersession note — "never a silent edit". The user cut D9's scope mid-flight; the cut was recorded
only in a dropped cell's reason and a backlog row. Compounding caught the gap and closed it with
decision `3a756908`.

D9 also mixed altitudes by locking "as their own cell", which is cell shape rather than a product
outcome — the same class of defect a pre-code review had already flagged across D3/D5/D7 and that
the CONTEXT revision fixed everywhere except there.

### 6. The agent saw the scope overrun one turn before the user raised it

`bfr-6`'s sweep returned 18 defects against a 13-row inventory. The agent wrote "my pointer table
was short 5 of 18" and kept executing. The user then asked whether the work had drifted from
removing the byte-ceiling law — and it had.

The signal was in hand and was read as a process win rather than as a scope question. A sweep that
returns materially more than its inventory is a scope event, not just a completeness result.

### 7. Three errors of the exact kind the feature existed to prevent

Worth recording plainly, because they are the argument for the change rather than an aside:

- **Line counts asserted, not measured.** The first `CONTEXT.md` draft claimed 472 / 237 / 71 lines
  for three files that measure 471 / 236 / 70. A review caught it; `wc -l` settled it before the
  file was ever committed.
- **A locked decision that would have disarmed the suite it protected.** D5 as first written said to
  delete `test_agents_budget.mjs:232-236`. The byte figures are at `:233-235`; `:236` is
  `if (failed > 0) process.exit(1);`, the suite's only red path. Implemented literally under
  "implement them exactly, never reinterpreted", every guard the decision claimed to keep would have
  become non-blocking.
- **A vacuously green verify.** Finding 2 above.

**A note on what compounding can see.** A failure analyst checked the first item against the
artifacts and could not reproduce it — correctly, because the error never reached a commit. Only the
session transcript holds it. Retrospectives built purely from committed artifacts systematically
miss every defect that review caught, which is to say they miss the ones the process handled well
and over-weight the ones it did not.

## What held up

- **The kept/deleted line is principled, and was independently checked.** Every surviving guard is a
  presence or equality assertion — is the rendered block identical to its template, are all 17 named
  rules present, are the markers paired. Every deleted one was a `bytes <= threshold` comparison.
- **`bfr-3` proved the removal by defect class, not by name.** Its own negative control seeds a file
  containing `const skillTextCeilingBytes = 9001;` — a brand-new identifier matching nothing that was
  deleted — and the suite goes red. A name grep would pass forever while the law returned renamed.
- **"Treat the inventory as a floor" was written into the cell and it paid.** The instruction to
  re-sweep rather than work the table is what surfaced 18 against 13 — including two rows that were
  *in* the table but whose file the cell scope had omitted, which the table alone could never reveal.
  This reinforces the existing `enumerated-move trap` pattern rather than replacing it.
- **Serial ordering was derived from interpreted state, not file overlap.** Slice 2 ran serial
  because `onboard_bee.mjs --apply` re-vendors the `.bee/bin/bee.mjs` that a later cell *executes* —
  a dependency no path-overlap check would surface.

## Open friction filed

| Severity | Item |
|---|---|
| P2 | Slice-tail test-cell guard refuses every phase departure while a `change_class: test` cell is open — including into `swarming`, where it is open by design. Dropping the cell does not clear it; `testCellDebt` counts dropped cells. No bypass level lifts it. |
| P2 | Extend the size-law regression suite to `docs/knowledge/` (finding 1). |
| P2 | `REGEN_GUARDS` rule for cells adding an auto-discovered suite (finding 3). |
| P3 | Nine stale numbered-rule pointers remain, deferred by the user's scope cut; three may predate the byte diet entirely and want a human read before being treated as D9 cleanup. |
| P3 | `verify-cache-inputs.json` can carry an orphan declaration matching no suite in `SUITES`. |

## Suite census

117 suites in the registry (118 before, minus the two fence entries, plus the new regression suite).
