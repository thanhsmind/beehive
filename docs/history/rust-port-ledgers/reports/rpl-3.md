# rpl-3 — [DONE]

**Outcome:** `bee-core::datamark` (neutralizer + rejector, exact-translated on regex-lite with a
hand-enumerated JS `\s` class) is proven against the real frozen mjs by a node-child oracle, and the
`capture` group now runs through the rpl-1 seam with 17 `--cmd-check` scenarios clean.

## The finding this cell exists to produce

`ignore\u{00A0}all\u{00A0}previous\u{00A0}instructions` — three U+00A0 NO-BREAK SPACEs where the eye
sees ordinary spaces. The real `decisions.mjs` **rejects** it; a `regex-lite` translation using a
bare `\s` — the translation this cell was originally scoped to write — **accepts** it, because
`regex-lite`'s `\s` is ASCII-only while ECMAScript's is `WhiteSpace ∪ LineTerminator`. A parity red
and a live content-safety bypass in the same string. Switching to the full `regex` crate does not
fix it either (its `\s` excludes U+FEFF and includes U+0085 — divergent from JS in both directions),
so the fix is the hand-enumerated `JS_WHITESPACE` class with `regex-lite` kept and no new dependency.

Pinned permanently, and inside this cell's own verify, by
`datamark_oracle.rs::the_nbsp_bypass_mjs_rejects_it_and_a_bare_backslash_s_port_would_accept_it`,
which measures all three verdicts side by side rather than asserting the claim.

## Files touched

- `crates/bee-core/src/datamark.rs` (new) — the two code paths, `js_trim`, `at_least_units`
- `crates/bee-core/src/feedback.rs` (new) — `KIND_ALIASES` / `NORMALIZED_KINDS`, the D7 enumeration
  authority rpl-6 consumes
- `crates/bee-core/src/capture.rs` — read-only projection EXTENDED with `add`/`flush`
- `crates/bee-core/src/{fsutil.rs,lib.rs}`
- `crates/bee-core/tests/datamark_oracle.rs` + `tests/support/datamark_oracle.mjs` (new)
- `crates/queen-bee/src/ledger/capture.rs` (new), `ledger/mod.rs`, `groups.rs` (one registration line)
- `crates/queen-bee/src/dispatch.rs` — stale rpl-1 pin updated (test module only)
- `crates/bee-parity/src/cmdcheck.rs` — 17 `capture` scenarios + the group's registration floor
- `crates/queen-bench/src/fixture.rs` — `.bee/capture-queue.jsonl` seeded, `CAPTURE_PENDING_FLOOR_COUNT`

## Deviations

1. **Stale pin, `crates/bee-parity/src/cmdcheck.rs`** — `unported_ledger_groups_still_have_zero_scenarios_registered`
   asserted `capture` had zero scenarios. Registering the group makes that false. Rewritten around a
   `PORTED_GROUPS` list, plus an assertion that the pin still guards something.
2. **Stale pin, `crates/queen-bee/src/dispatch.rs`** — `the_shipped_table_registers_no_ledger_group_yet`
   asserted the dispatcher registers NO group. Already false after rpl-2; it stayed green only
   because `cargo test` aborts after the first failing target and bee-parity failed first. Now an
   exact-set assertion over the ported groups.
3. **Lone surrogates dropped from the corpus, with the reason stated** — a Rust `String` cannot hold
   one and every path into these functions has already validated UTF-8, so it is out of scope for the
   differ rather than an unachievable requirement. Documented at the head of `datamark_oracle.rs`.
4. **`capture add` parity scenarios run `--json`** — `add`'s human text interpolates a fresh
   `crypto.randomUUID()` as bare prose, and `bee-parity::normalize` masks by JSON key name only (by
   design, no pattern scrubber). Under `--json` the same value is a real key and masks on both legs.
5. **The refusal scenarios' negative control is the REGISTRY, not the queue** — `addCaptureStub`
   refuses before it reads or writes anything, so no queue perturbation can move their output. A
   queue-aimed control there would be a control that cannot fire.

## Outstanding

None. `crates/target/` build artifacts and the two `.bee/bin/lib/` files in the working tree are not
this cell's (D1 freeze) and were left unstaged.

Full trace, verification evidence and verify output: `.bee/cells/rpl-3.json`.
