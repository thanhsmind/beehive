# SLP contract status + original request — learnings

- Date: 2026-08-29 · Feature: `slp-contract-original-request` (cluster 4 of the
  slp-supervisor-lead-peer map) · Lane: high-risk · 6 cells, all capped, all
  five code cells judged PASS by an independent fable judge.
- Shipped: the user's verbatim request now rides every dispatch prompt; a
  `contract:<name>` tag is writable; a decision's settled status is derived
  from the log; a claim or dispatch citing a retired or unsettled decision is
  refused; a test-writing cell citing no contract decision trips the mint trap.

## A locked decision can be unimplementable, and the plan is where you find out

D2 locked the tag spelling `contract:<name>`. The slug predicate that every
decision tag validates through refused a colon — the locked label could not be
written at all. Nothing in shaping could have known this; it took reading
`tag_pattern_test` during planning.

The move that worked: widen the predicate to admit the locked spelling, rather
than respell the tag to fit the predicate. CONTEXT.md's Agent's Discretion
clause freed "its spelling in prose" — not the tag itself — so respelling was
the unfaithful option even though it was the smaller diff.

**Generalizable:** when a locked decision collides with a validator, the
question is which one the decision actually locked. Widen the mechanism when
the decision named the spelling; respell only when it named the concept.

## Measure the field before you write a guard over it

D3 said cells cite contract decisions in the existing `cell.decisions` field.
The plan's first draft assumed that field holds store decision ids. The
plan-check measured it: 92 live cells, 81 citations, **11 resolve** (13%). The
field is dominated by local D-IDs (`D1`, `D2`) pointing into CONTEXT.md tables.

A tripwire that refused every unresolvable entry would have refused 87% of
citing cells on day one. The shipped rule passes over anything that does not
resolve, and says so in its own docs.

The same measurement killed the plan's D4 signal: `role: "test"` fires on
**0 of 92** cells, and the path heuristic catches 27 of the 67 cells that
actually name test-writing (40%). The "narrow accepted hole" the first draft
admitted — a `role: code` cell adding an inline `#[cfg(test)]` module — is the
*majority* shape in this repo.

**Generalizable:** before a guard reads a field, count what that field actually
holds across the live store. A guard authored from the schema rather than the
data refuses the wrong population. This is worth a subagent and five minutes.

## Resolving against the active set makes the retired arm unreachable

The citation resolver first resolved entries against the ACTIVE decision set.
That silently disables the rule it exists to serve: a superseded citation is
not in the active set, so it resolves to nothing, so it is passed over — the
exact case D3 refuses. The fix is to resolve against the active+archive union
and judge status against the active set.

**Generalizable:** when a guard is "resolve, then judge", the resolve step must
span a WIDER population than the judge step. Same-population resolve-and-judge
makes every negative verdict unreachable.

## A read that writes cannot sit on a refusal path

The only trigger reader persisted a `waiting → due` flip mid-read. A refusal
that promises zero mutation cannot call it. The fix was one shared walk with
the flip as a flag, not a second copy — and the byte-identity test was written
so it cannot pass vacuously: it asserts the bytes are unchanged after the
derived read, then runs the evaluating reader over the same file and asserts
the bytes DID change.

**Generalizable:** a "nothing changed" assertion is worthless unless the same
test proves something COULD have changed.

## Absence beats a stale answer under a do-not-paraphrase header

The intent anchor's key walk falls back to a shared `default` key with no TTL
and no staleness check. The live default held a four-day-old request about
already-shipped work. Rendering that into every featureless dispatch under a
header reading "VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE" would have
been meaning-*replacement* — the precise failure the feature exists to prevent.
The dispatch door reads feature-keyed only and renders nothing otherwise.

**Generalizable:** a fallback that cannot be told apart from a live value is
worse than no value, in exact proportion to how authoritative the framing is.

## A source-level CLI change is not a live CLI change

The widened tag predicate is green in source and in CI. `bee decisions log
--tags contract:x` still refuses in this checkout, because `.bee/bin/bee` is an
installed release artifact, not a build of the working tree. Every source-level
CLI change carries this gap between merge and release.

It landed safely here only because the mint trap's ramp is derived: it warns
while zero `contract:`-tagged decisions exist, which is exactly the state the
un-rebuilt binary enforces.

**Generalizable:** in a repo that vendors its own tool, "the tests are green"
and "the tool does it" are two claims. Say which one you mean.

## Two frictions worth fixing

- **The plan-freeze guard blocks `git add`.** Once a shape gate is approved,
  the guard refuses any Bash command naming `plan.md` — including staging it —
  so a frozen plan cannot reach main without addressing the parent directory
  instead. Filed as PBI `p-911f5ffe`.
- **A truncated feature slug routes nothing, silently.** Three knowledge-bundle
  citations named `slp-contract` instead of `slp-contract-original-request`,
  and one named `D5, D6` in a comma list the matcher does not read (it reads
  `-` and `/` sequences). Both read as "unrouted" with no hint that a citation
  was attempted. The routing door reports the count, never the near-miss.

## Method note

The plan went through an independent plan check before Gate 2 that returned
**SHAPE NEEDS CHANGE** with three P1s — all three are the findings above. The
revised shape split two slices apart on that evidence. A plan check that only
ever confirms is not being asked hard enough questions.
