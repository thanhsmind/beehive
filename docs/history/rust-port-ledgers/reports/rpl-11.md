# rpl-11 — [DONE]

**Outcome.** The shared `readJson` parse-failure warning is now a **declared
stderr artifact** rather than an unreconcilable divergence. Its invariant
prefix (`bee: could not parse JSON at <path> — `, path included) and its
invariant suffix (`. Using fallback; fix the file.`) are compared
byte-for-byte between the legs; only the parser text between them is replaced
with `<PARSE_ERROR>`, and only when it matches the JSON-parser dialect of the
runtime that produced it. Every remaining ledger group can now register
unparseable-whole-JSON-store scenarios.

Full trace, verification evidence and verify output:
[`.bee/cells/rpl-11.json`](../../../../.bee/cells/rpl-11.json).

## Files touched

| File | What |
|---|---|
| `crates/bee-parity/src/normalize.rs` | `reconcile_parse_warnings` + the two per-leg dialect predicates, wired into `normalize_stderr`; 9 new test rows in the existing `tests` module |
| `crates/bee-parity/src/cmdcheck.rs` | the `seam/unparseable-whole-json-store` scenario, its seed and its negative control |
| `docs/history/rust-port-ledgers/reports/rpl-11.md` | this report |

Accepted-divergence decision `718cbb97` logged under scope `rust-port`, tags
`rust-port,parity,fsutil`.

## What was measured, not assumed

The mjs sentence was **executed**, not transcribed. A node child (v24.14.1)
ran the frozen `packages/bee/lib/fsutil.mjs` `readJson` over 15 deliberately
unparseable files and its stderr was hexdumped:

```
bee: could not parse JSON at .bee/tmp/rpl-11/bad.json — Expected property name or '}' in JSON at position 2 (line 1 column 3). Using fallback; fix the file.
```

`od -c` pinned the three things the cell called load-bearing: the separator is
bytes `342 200 224` (U+2014 EM DASH) with exactly one ASCII space on each
side; the sentence ends with a literal `.`; `console.warn` adds a single
trailing `\n`. The run also revealed an **invariant suffix** the source text
alone would not have made obvious — `. Using fallback; fix the file.` follows
the parser text — so the reconciliation asserts a prefix *and* a suffix, with
the differing text sandwiched between them.

The same sweep mapped V8's message space into three families, including two
with **no** position clause at all (`Unexpected end of JSON input`;
`Unexpected token 'h', "hello" is not valid JSON`). A naive "must end in
`(line N column N)`" rule would have rejected both.

## The shape of the fix (rpl-1's precedent, applied)

`strip_runtime_stderr_artifacts` handles the `[bee] <cmd> <n>ms` line by
*asserting* it rather than ignoring it. This follows that exactly:

- the warning is **replaced, never removed**, so a leg that emits none where
  the other does is still a diff;
- the path lives inside the asserted invariant prefix, so two legs blaming
  two different files stay a diff;
- a warning missing the invariant suffix is an **error** — the frozen
  sentence moved, so the normalization no longer describes reality;
- the tail is **never accepted unconditionally**: it is checked against the
  dialect of the runtime that produced it. Empty, prose, or the *other*
  runtime's dialect are all refusals, tested in both directions. An `Err`
  from `normalize_stderr` is already surfaced as a `stderr_diff` by
  `differ::diff_legs`, so a refusal fails loudly rather than crashing.

## Three findings the next cells must not re-discover

1. **`.bee/cells/*.json` records cannot carry this coverage.** `bee-core`
   `cells.rs list_cells` parses them with a bare `serde_json::from_str` and
   skips a corrupt one **silently** — no warning at all, where mjs's
   `readJson` would warn. A corrupt-cell scenario would diff on the *absence*
   of a warning, not on its text. The scenario uses
   `.bee/cells/archive/summary.json` instead, which both legs read through the
   shared primitive exactly once (`bee.mjs:738` → `cells.mjs:809`;
   `status.rs:562` → `bee-core cells.rs:279`).

2. **The negative control has to make the store *parseable*, not differently
   corrupt.** Perturbing the corrupt bytes only moves the parser tail — which
   is masked — so such a control would pass while proving nothing. Making the
   store valid removes the warning from the mutated leg entirely, which is the
   one property the reconciliation must never lose.

3. **Node 18 is still in the CI matrix** (`.github/workflows/ci.yml`
   `node-version: [18, 20, 22]`) and its V8 emits the positional family
   *without* the `(line N column N)` clause. The predicate accepts both forms;
   a version-pinned rule would have gone red on the oldest matrix leg only.

## The divergence, measured side by side

One corrupt `.bee/cells/archive/summary.json`, the two real binaries:

```
mjs:       … — Unexpected token 'o', ..."seable": not json at"... is not valid JSON. Using fallback; fix the file.
queen-bee: … — expected ident at line 1 column 26. Using fallback; fix the file.
```

Nothing in common but the prefix, the path and the suffix — which is exactly
what the scenario's `Equals` assertion now pins on both legs.

## Notes

- No blanket stderr strip was introduced; `strip_runtime_stderr_artifacts`'s
  refusal of asymmetric strips is unchanged and its four tests still pass.
- `bee-parity` remains dependency-free by design, so its unit tests cannot
  link `serde_json` to generate the port's error text. That is why the
  cmd-check scenario is load-bearing rather than decorative: it puts the real
  binary's actual stderr through the predicate on every run.
- No ledger group was registered and no group logic was ported.
- No advisor consult was triggered: the only red was the required red-first
  phase, with a self-diagnosed root cause.
