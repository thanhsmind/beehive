# model-role-split — learnings

Harvested 2026-08-25 from CONTEXT.md, plan.md, 34 capped cells, the r2 review
report, and this session's own merge and close. Evidence-backed only; where a
lesson restates a pattern the bundle already promotes, it is recorded as a
**recurrence** rather than as a new finding.

## A cost word cannot name a job, and the store said so before anyone asked

The retired selector was `extraction` / `generation` / `ceiling` — three points
on a price curve. The feature exists because nobody can configure "the model
that is good at generation": generation is not a thing a model is good at. Jobs
are: one model plans well, another tests well, another designs, another writes
code.

The store had already recorded the failure. Measured across all 506 stored
cells at charting time: 269 said `generation` (the default anyway), 215 said
nothing, and `extraction` was chosen exactly twice. **95 percent of cells
carried no signal in the field.** A selector that nobody sets is not a selector
that people forget to use — it is a selector whose vocabulary does not match the
choice they are trying to express.

The general form: when an optional field goes unset at scale, read it as
evidence about the field's *vocabulary*, not about user discipline. Count the
field before redesigning around it.

## The store ran the natural experiment for required-vs-optional

D7 made `role` required. The argument was not taste, it was the same store's own
control group: the required field (`lane`) was present on 506 of 506 cells; the
optional one (`tier`) on 291. An optional role would have reproduced the `tier`
outcome exactly — a configured per-job model firing on about half the cells that
wanted it, with the miss silent every time.

Where a codebase already carries one required and one optional field of similar
shape, the fill rates are a free experiment. Use them instead of arguing.

## One question, two doors, two answers

`bee dispatch prepare --role advisor` **refused** on a host with no advisor
configured, while `[bee-tier: advisor]` through the model guard **passed** and
silently inherited the session model — the exact outcome the refusal text
existed to prevent. Same question ("is this role configured?"), two
implementations, two answers.

This is the defect D1 was written to remove, and it survived D1 anyway: the
feature collapsed the *parser* to one copy, but the two doors still asked their
question differently around it. Collapsing a shared implementation does not
collapse the callers' framing of the question. When two surfaces answer one
question, pin them with a test that runs the same input through both — not with
two tests that each check one door.

Related: `known_roles` still skips a coercion `resolve_role_named` applies.
Latent today because both callers pass gated literals, filed as P3.

## A migration that scans outside its lock does not half-write — it fully overwrites

`cells backfill-roles` read every cell to build its plan, took the
`cells-archive` lock only for the write phase, then wrote each whole object
back. The doc block claimed a concurrent `update`/`cap`/`claim` "cannot
half-write behind this pass". True, and beside the point: the concurrent write
completed cleanly and was then *fully* overwritten by a stale clone.

Demonstrated on a 40 000-cell store: an operator's `cells escalate --off`
committed to disk at t=25ms; the backfill finished at t=1.007s; the flag was
back to `true` afterwards. Invisible to casual testing because only writers that
complete *inside the scan phase* are lost — a budget-checked `--on` arriving
during the write phase was correctly refused by the lock at 319ms.

The rule: a read-modify-write pass over shared state must hold its lock across
the **read**, or re-read each record under the lock before writing it. A lock
that covers only the write phase protects the file's structure and none of its
content. The fix took the second route — re-read under the lock, merge only the
three fields the migration owns.

## A piped test command reports the pipe's exit code, not the suite's

`cargo test ... | tail -60` exits with `tail`'s status. A red suite reported
`exit code 0`, and the failure detail scrolled out of the 60 kept lines. This
session read that as green before catching it.

Two independent halves, both needed:

- `--no-fail-fast`, or cargo stops at the first failing target and every later
  target goes unrun while the summary still looks like a full pass. This bit the
  feature during execution: slices 1–3 reported green from runs that never
  reached `registry_dispatch`, which was red.
- Never filter a test run through `tail`. Filter through `grep -E '^test
  result|^error|^failures:'` so every target's verdict survives regardless of
  output volume.

The repo's own recorded `commands.test` still omits `--no-fail-fast`; filed as
a P2.

## A one-line generated JSON file cannot be merged as text

`generated/registry_payload.json` is a single line. A git conflict in it is one
unresolvable hunk, so "resolve the conflict" degenerates to whole-file ours or
theirs — and either pick silently drops one side's entire command set.

What worked: merge it as **data**. Parse ours, theirs, and the merge base
(`git show :1:<path>`), then decide each command entry by **who changed it
against the base**, never by which side you trust. That recovered main's two new
verbs, correctly dropped a verb main had not yet seen retired, kept both of ours,
and text-merged one description both sides had extended at the same offset.
Reproduce the serialization exactly and round-trip-check it against the incoming
side's own line before writing, or the next merge conflicts on formatting noise.

Generalizes to any single-line generated artifact: lockfiles, bundled manifests,
minified output. The merge strategy belongs to the file's *structure*, not to
git's line model.

## Recurrences — already-promoted patterns that fired again

Recorded rather than re-promoted, per the promotion tree. Each is a candidate
for escalation from prose to a durable owner.

- **`a-test-that-derives-its-fixture-from-the-constant-under-test`
  (2026-08-19).** Fired twice in this feature. First: `HOST_BEFORE_ROLES` was
  byte-identical to `default_models("claude")`, so the safety tests could not
  fail; fixed with rotated `-custom` values plus a fixture-integrity test.
  Then the *same* defect shipped again in `defaults_apply_without_config`, where
  three of four assertions are tautological against the built-in defaults — the
  fix was not carried into `onboard`. Twice in one feature, after the pattern was
  already written down, is the signal that prose is not holding it. **Escalation
  candidate: a test-time check that fails when a fixture is byte-equal to the
  constant it is meant to detect drift from.**
- **`source-shipped-without-reinstalling-the-called-binary-is-inert`
  (2026-08-05).** The installed `.bee/bin/bee` was 46 commits stale, which
  presented as a `dispatch prepare --kind cell` refusal blaming the caller's
  argument shape. The real cause was a prompt-hash skew between the installed
  binary and the source prompt. A sibling session independently spent a day on
  the same class. Filed as a P2 asking the router to name the skew instead of the
  arguments.
- **`plausibility-is-not-evidence` (2026-08-25).** A one-sample rule ("the `w0`
  entry is the victim") survived peer review because both sessions shared the
  prior; only new instrumentation showed the real victim is whichever racer won
  the takeover. Two readers agreeing is not corroboration when they inherited the
  same assumption — only new data is.

## What held

Recorded because a learnings file that lists only defects misreports the work.

`normalize_models` and `resolve_role` exist exactly once and the guard calls
through; `CLAUDE_TIERS`, `CODEX_TIERS`, `MODEL_NORMALIZE_SLOTS`,
`MODEL_VALIDATE_SLOTS` and all four `MODEL_TIERS` copies are gone, with no fifth
legality list surviving. The escalation ration is pinned at both boundaries —
40 percent allowed, 43 percent refused. No secret reaches a log or terminal, the
chain is data-only and never joined into a command line, and role names reach no
path, process, or shell. One test function was deleted repo-wide, with a named
replacement.
