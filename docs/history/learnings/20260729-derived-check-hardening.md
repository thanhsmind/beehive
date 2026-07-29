# Learnings — derived-check-hardening (2026-07-29)

**Feature:** turned the findings of `validation-diet`'s compounding pass into standing
checks — the cap door consults the impact registry, two hand-run sweeps became suites,
the six hand-copied phase memberships got a parity suite, and the completion door stopped
missing the flag its own downstream obligations key off.

**Scale:** 9 cells. **Outcome:** `PASS run_verify: 117 suite(s)` (115 → 117; two new
suites). Feature-verify record green.
**Decisions:** E1-E9, `docs/history/derived-check-hardening/CONTEXT.md`.

---

## M1 — The cell built to catch the miss committed the miss

`dch-1` wired the impact registry into the cap door so a unit whose check command
omits a direct-edge suite gets named. It did so with a **static top-level import** of an
out-of-tree module into `packages/bee/lib/cells.mjs`.

`packages/bee/hooks/test_write_guard.mjs` vendors only `.bee/bin/lib/*.mjs` into a bare
temp root with no sibling `scripts/` directory. The import threw `ERR_MODULE_NOT_FOUND`,
the hook's fail-open path swallowed it, and **six assertions silently stopped being
checked**. `dch-1`'s own verify never runs that file.

So the cell that exists to warn "your verify does not cover the suites your files
touch" shipped with a verify that did not cover the suite its file touched. It took
`dch-8` to fix, and the fix was the same shape the warning would have recommended:
resolve the dependency lazily, treat an absent module exactly like an absent registry —
a silent skip, never a throw.

**What generalises.** A guard cannot exempt its own construction from the property it
guards. When adding a check for class X, the first thing to run it against is the diff
that adds it. Concretely: an out-of-tree static import inside a file that gets vendored
alone is always wrong, because vendoring is by definition a context where the tree is
not there.

## M2 — A test that copies live config and hardcodes a value from it will go red on a config change, not a code change

`packages/bee/hooks/test_model_guard.mjs` copies the real `.bee/config.json` into its
fixtures and then wrote the literal `"sonnet"` at eight-plus sites meaning "a model that
is configured." The guard under test derives from that config — correctly; config is the
sole authority and there is deliberately no hardcoded allowlist.

The owner changed `models.claude.generation.model` to a different model. The guard was
right, the config was right, and the suite went red with a message that described
neither: *"sonnet is not a model configured for any claude tier."*

The tell was that the failure appeared with **no code change to the subject** — the
whole feature had touched neither the guard nor the config. A suite that can go red from
a configuration edit is comparing a literal against derived ground truth, and `dch-9`
fixed it by resolving the names through the same resolvers the guard itself uses.

That worker went further than asked and found the same trap on the other runtime's
generation slot, and made one stderr assertion **stronger** than what it replaced —
requiring every derived member to appear rather than four named literals.

**What generalises.** If a test copies a live config, every value it then asserts must
come from that copy. The moment a literal appears next to a copied config, the test has
two sources of truth and only one of them updates.

## M3 — The guard proved itself on the orchestrator, twice, mid-feature

While dispatching workers for this very feature, the model guard refused two of my own
dispatches: the tier marker said `generation` while the model parameter named a
different model than the one config resolves that tier to.

Both refusals were correct and both were mine. It is worth recording that the mechanism
under repair caught its own maintainer in the ordinary course of the repair — that is
the strongest evidence a guard is real rather than decorative, and it is the kind of
evidence that only shows up when the tooling is dogfooded rather than tested in
isolation.

## M4 — Deriving locations is sometimes honestly impossible, and saying so beats faking it

`dch-6` was asked to derive its scan locations rather than hardcode six paths, because a
hardcoded list of the very thing being checked for drift is the same defect one level up.

The worker tried and reported back that a fully hardcode-free scan was not achievable:
grouping by current value is self-defeating (a drifted copy simply forms its own group),
and a purely structural scan false-positives on a genuinely different membership that
shares the same shape. It used the escape hatch honestly — hardcoding only the three
*constant names* (a domain fact) and the two *lib roots* (named in the cell), discovering
every file and line within them at check time.

That is the right answer, and the value is in it being **reported** rather than quietly
approximated. A line-shift or a file-move can no longer desync the suite; only renaming
the concept itself can, which is a much rarer and much more visible act.

## M5 — Two residuals ship open, by decision, and the records say so

The owner made two calls with the tradeoff stated:

- **The cap-door registry check warns and never refuses** (E1). A refusal would gate
  every future unit on the registry being fresh. The warning is weaker than the finding
  wanted, and it is written down as weaker.
- **CI still runs once a day** (E2). The cron moved from 16:00 to 23:00. That changes
  *when* the detection window opens, not how wide it is: the base branch can still carry
  a red for up to 24 hours, and CI still only files an issue rather than blocking.

Both friction rows stay open at their recorded severity, and both are filed as Open Gaps
in the state layer. Recording them as resolved would have made the backlog lie about the
repo's actual coverage — which is precisely the failure mode this feature exists to
reduce.

## Patterns confirmed, not newly promoted

Everything this feature built was the mechanisation of three patterns promoted at the
previous feature's close. No new critical pattern is promoted here: M1 and M2 are fresh
*instances* of
[[pattern-20260728-a-derivation-the-tooling-computes-but-doctrine-forbids-where-it-is-needed]],
and M4 is an honest boundary case of
[[pattern-20260728-one-membership-hand-copied-six-times-has-no-owner-and-no-alarm]].
Promoting them again would dilute the list rather than sharpen it.

The one thing worth carrying forward as a habit, not a pattern: **run a new check against
the diff that introduces it before running it against anything else.**
