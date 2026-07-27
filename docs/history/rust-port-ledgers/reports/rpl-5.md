# rpl-5 — decisions query verbs (active, search, archive, tag)

**[DONE]** — the `decisions` QUERY half is ported behind the rpl-1 seam, byte-identical to the
frozen mjs across 36 `--cmd-check` scenarios (21 of them new), and both gaps carried from the
rpl-4 goal-check are closed by executed oracles. `render` stays unported by design (rpl-10).

Files touched:

- `crates/bee-core/src/decisions.rs` — `archive_decisions`, `tag_decisions_batch`/`tag_decision`,
  the jsonl atomic-rewrite and batch-append primitives, the short8 UTF-16 fix, `canonicalize` exported
- `crates/bee-core/src/jsdate.rs` — `date_parse_ms`, the wider `Date.parse` the user-supplied
  `--before`/`--since` arguments need
- `crates/queen-bee/src/ledger/decisions.rs` — the four handlers, the dp-1 filter stack, `formatDecision`
- `crates/bee-parity/src/cmdcheck.rs` — 21 scenarios with four seed corpora and their own mutators
- `crates/queen-bee/tests/decisions_composed_oracle.rs` — new; the two carried gaps

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

## Declared, not masked

`date_parse_ms` implements the ECMA-262 Date Time String Format. Two V8 behaviours are declared
divergences rather than reproduced — an offset-less date-time is local in ECMA-262 and UTC here,
and V8's legacy fallback parser (`2026/01/01`, `Jan 1 2099`, and `2026-02-30` rolling over to March
2nd) is not reproduced. Both are pinned by name in tests, and the ES-grammar refusals the two
runtimes share are pinned alongside them. Logged as decision `a6c73472`.

## Note for rpl-10

`render` is the only decisions verb still unregistered, and
`the_group_registers_every_verb_except_render` asserts its absence so the group cannot be mistaken
for finished. `buildDecisionIndexBody` consumes `activeDecisions(root, {all})` — the overlay-applied
read path this cell just ported — so rpl-10 inherits it rather than re-deriving it.
