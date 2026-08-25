# decision-attribution — plan (rev 1)

Route: `bugfix` · lane `standard` · 3 flags · 4–5 product files.
Decisions: `decision-attribution D1`–`D5`, locked in the store 2026-08-25.

## Ordering is load-bearing

**S1 must precede S3.** The migration corrects history; until the leak is
closed, every unbound `decisions log` call in any live session writes a fresh
wrong stamp. Correcting history first would race new bad writes and report a
clean run over a store that is already dirty again. Stop the leak, then fix the
past.

S2 may land before or after S3; it is placed second because D1 without D2
leaves Discovery permanently unattributed, and that is the state the fix would
otherwise ship in.

## S1 — Stop the borrow (D1, D4)

The walking skeleton: after this slice the bug is fixed end to end, with or
without the rest.

- `verbs_read.rs:613` — resolve `bound_feature` from the calling session's
  bound lane only. Do not consult the default `.bee/state.json` record. When
  nothing resolves, insert no `feature` key.
- The existing call passes `no_lane: false` into
  `resolve_mutation_target`; the read must instead distinguish a `Target::Lane`
  result from a `Target::Default` one and accept only the former.
- **Red first (D4).** New test: a fixture whose `.bee/state.json` EXISTS and
  names a foreign feature, with no bound lane. Assert the emitted event has no
  `feature` key. Run it against the unfixed code and record that it fails —
  the existing `..._when_no_feature_is_bound` test uses a fixture with no state
  file and passes either way, so it is not evidence.
- Leave `resolve_mutation_target` untouched (D3).

Proof: the new test red before, green after, plus
`cargo test --release --no-fail-fast -p bee` green over the decisions module.

## S2 — Give Discovery a door (D2)

- `bee decisions log` gains `--feature <slug>`; explicit value beats the bound
  lane.
- Declare it in the command registry payload so `bee --help` and the CLI-shape
  guard both know it. That file is one-line generated JSON — edit it as data,
  never as text.
- Validate the slug the same way other feature slugs are validated; an empty
  or malformed value refuses with a typed FIX line rather than being ignored.
- Test: explicit `--feature` wins over a bound lane; explicit `--feature` works
  with no lane at all.

## S3 — Correct the 23 (D5)

- New verb `bee decisions reattribute`, `--dry-run` first and always offered.
- Predicate, deliberately narrow: act only where the record's own `decision`
  text opens with `<slug> D<n>` and that `<slug>` differs from the stamped
  `feature`. Anything else is left alone — the verb cannot invent an
  attribution, only correct one the record itself contradicts.
- Writes only the `feature` field. Never touches `decision`, `rationale`,
  `alternatives`, `date`, `id`, or relations.
- Idempotent: a second run reports zero.
- Under the decisions lock, and re-reading each record inside the lock before
  writing it — the lesson from `cells backfill-roles`, where a scan outside
  the lock silently reversed a concurrent write.
- Tests: the 23-record shape corrects; a record whose text names its own
  stamped feature is untouched; a record with no `<slug> D<n>` prefix is
  untouched; a second run is a no-op.
- Then run it for real against the live store and report the counts.

## Test matrix

| Case | Slice | Why |
|---|---|---|
| state.json exists, foreign feature, no bound lane → no stamp | S1 | the actual bug; must fail red first |
| no state.json at all, no bound lane → no stamp | S1 | existing behavior, must not regress |
| bound lane → stamped with the lane | S1 | the correct path must survive the fix |
| `--feature` beats a bound lane | S2 | D2's precedence rule |
| `--feature` with no lane | S2 | the Discovery case |
| malformed `--feature` refuses | S2 | no silent ignore |
| text names another feature → corrected | S3 | the migration's whole job |
| text names its own feature → untouched | S3 | the predicate's floor |
| no `D<n>` prefix → untouched | S3 | the verb never invents |
| second run → zero changes | S3 | idempotence |

## Risks

- **The registry payload is one-line generated JSON.** A conflict in it cannot
  be merged as text. Merge it as data against the merge base.
- **Other sessions are live** and append to `.bee/decisions.jsonl` continuously.
  S3 must hold the decisions lock across its read, and its counts will differ
  from the 23 measured today if a discovery session logs more in the meantime.
  That is expected, not a failure — report the real number.
- **`--no-fail-fast` is required** on every suite run, and no piping through
  `tail`. The recorded `commands.test` omits it (filed P2).
