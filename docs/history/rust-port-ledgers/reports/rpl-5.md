# rpl-5 — decisions query verbs (active, search, archive, tag)

**[DONE]** (round 2) — the `decisions` QUERY half is ported behind the rpl-1 seam, byte-identical to
the frozen mjs across 38 `--cmd-check` scenarios (23 of them new), and both gaps carried from the
rpl-4 goal-check are closed by executed oracles. `render` stays unported by design (rpl-10).

Round 2 fixed the one thing rpl-5's own goal-check returned `NEEDS_REVISION` on: a silent
over-archiving divergence in the `--before`/`--since` cutoff. See "The cutoff bug" below.

Files touched:

- `crates/bee-core/src/decisions.rs` — `archive_decisions`, `tag_decisions_batch`/`tag_decision`,
  the jsonl atomic-rewrite and batch-append primitives, the short8 UTF-16 fix, `canonicalize` exported
- `crates/bee-core/src/jsdate.rs` — `date_parse_ms`, the wider `Date.parse` the user-supplied
  `--before`/`--since` arguments need; round 2 made the offset-less date-time fail closed and
  widened the declared non-reproduction list
- `crates/queen-bee/src/ledger/decisions.rs` — the four handlers, the dp-1 filter stack, `formatDecision`
- `crates/bee-parity/src/cmdcheck.rs` — 23 scenarios with four seed corpora and their own mutators
- `crates/queen-bee/tests/decisions_composed_oracle.rs` — new; the two carried gaps, plus round 2's
  declared cutoff divergence

Full trace, verification evidence and red-failure evidence: `.bee/cells/rpl-5.json`.

## What the fixture problem forced

`queen-bench --generate` writes monotone filler — every row `type: "decide"`, one shared date,
`scope: "repo"`, no `tags` key, no supersede/redact/tag event, no archive sidecar. Against that
store every assertion this cell owes is vacuous, so each scenario seeds the rows it asserts on.
The corpora are shaped so the right answer differs from every plausible wrong one, not just from
the empty set: the ranking corpus scores 3/2/2/1 with the two-hit rows appended in an order that
makes the stable sort observable (correct answer A, D, B, C — a permutation of both file order
and reverse-file order), and the token corpus holds `si-1`, `si-10` and `si-1-extra` together so a
substring match and a bare `\b` match both produce a wrong answer.

**The harness rejected four of my first controls** as non-discriminating, each for a different
reason, and each rejection is recorded at the mutator that replaced it: a mutation below a
`--recent 1` slice, a mutation on a row outside a `--tag` answer, a tag-value swap that could not
move an `--untagged` answer, and an un-supersede that changed nothing past the age threshold.

## Two carried gaps

**Composed sweep-with-hits handler.** Closed by an oracle test driving `decisions supersede` as a
real process on both runtimes over a `docs/**` tree with three citations across two files, comparing
stdout and the resulting capture-queue rows. It cannot go through `--cmd-check` because the fresh
event uuid sits as a `dids` array element and inside `outcome` prose, positions `normalize.rs`'s
key-gated masking cannot reach — so each leg's own uuids are read back out of its own store and
substituted exactly. Timestamps collapse to one placeholder with their `toISOString()` precision
asserted separately; the first version pinned them per-stub and failed on whether two stubs happened
to share a millisecond, which is a property of the machine and not of either implementation.

**short8 UTF-16.** `supersede_decision` measured its needle in scalar values where
`decisions.mjs:465` measures UTF-16 code units. Fixed by extracting `js_slice_prefix`, the same
measurement `js_excerpt` in that file already used, and pinned end-to-end with an astral target id
whose two readings are mutually exclusive: the scalar reading finds ZERO citations where mjs finds
one, so the sweep silently misses a real citation and never queues its capture stub.

## The cutoff bug (round 2)

Round 1 shipped `date_parse_ms` reading an **offset-less date-time as UTC**. The goal-check
reproduced what that cost: on a host at `TZ=Asia/Bangkok`,
`decisions archive --before 2026-07-26T00:00:00 --json` made mjs archive one row and the port
archive two — **both exiting 0, no warning** — because V8 read the cutoff as `+07:00` (17:00Z the
previous day). `archive` deletes rows from the active store and has no inverse verb, and on a
negative-offset host the same error inverts into keeping a row mjs would have deleted.

The fix is **fail-closed, not local-time emulation**: the no-designator arm now returns `None`, so
an offset-less date-time gets the same loud refusal the legacy-fallback forms already produced.
Emulating ECMA-262 local time would need a local-offset source `std` does not have, and guessing
one trades a divergence a cross-runtime harness can see for one it cannot. The documented surface
is untouched — date-only forms *are* UTC in ECMA-262, so `--before 2099-01-01` still works.

Three things the same goal-check asked for came with it:

- **The coverage hole that let it through.** Every `--since`/`--before` in the harness was a full
  `toISOString()` value or an obviously-invalid string; the documented date-only form had no
  scenario at all. Two now exist — `--before 2099-01-01` (the registry's own example) and
  `--before 2026-07-26`, where the resolved instant is observable to the millisecond because the
  control moves one filler row across the strict `<` boundary.
- **The declared non-reproduction list, which was incomplete.** A lowercase `z`, a space separator,
  `+HHmm` without a colon, and expanded `+00YYYY` years are all finite in V8 (measured) and `None`
  here. They were fail-closed in practice but undeclared on paper — an undeclared choice is
  indistinguishable from an oversight — so they are now named in `jsdate.rs` and pinned by test.
- **The `2026/01/01` wording divergence, pinned rather than chased.** Both runtimes exit 1 and
  leave the store byte-untouched, but say different things: mjs accepts the cutoff and then finds
  nothing qualifying, the port refuses the cutoff itself. Pinned as a declared artifact in
  `decisions_composed_oracle.rs`, the same way rpl-11 pinned the `readJson` warning.

## Declared, not masked

`date_parse_ms` implements the ECMA-262 Date Time String Format. What it does NOT reproduce is
declared, listed in full, and **fail-closed in every case** — there is no form on which it silently
resolves to a different instant than V8. The offset-less date-time and V8's legacy fallback parser
(`2026/01/01`, `Jan 1 2099`, `2026-02-30` rolling over to March 2nd, plus the four same-class forms
named above) all refuse. Pinned by name in tests, alongside the ES-grammar refusals the two
runtimes share. Logged as decision `a6c73472`.

Because a declared divergence is one the runtimes *must not* agree on, it cannot live in
`--cmd-check` — that harness's contract is zero diff. `decisions_composed_oracle.rs` is where those
comparisons go, and it now asserts both halves at once: the exact V8 reading on the mjs leg (with
`TZ` pinned, so a UTC test host cannot make the test vacuous — which is how the bug survived a
green suite the first time) and the port's refusal over a byte-untouched store.

## Note for rpl-10

`render` is the only decisions verb still unregistered, and
`the_group_registers_every_verb_except_render` asserts its absence so the group cannot be mistaken
for finished. `buildDecisionIndexBody` consumes `activeDecisions(root, {all})` — the overlay-applied
read path this cell just ported — so rpl-10 inherits it rather than re-deriving it.
