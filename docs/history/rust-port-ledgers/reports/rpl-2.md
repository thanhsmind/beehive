# rpl-2 — intent group (set, show, advance, clear)

**[DONE]** — worker `Ada`, lane `high-risk`, `behavior_change: true`.

The `intent` group is ported end to end and is the first group running through the
rpl-1 dispatch seam: store logic in `crates/bee-core/src/intent.rs`, the four verbs in
`crates/queen-bee/src/ledger/`, a seeded `.bee/intent/` fixture in `queen-bench`, and
24 `--cmd-check` scenarios that run mjs and queen-bee over the same tree, each with its
own negative control asserted on the channel it declares.

## Files touched

- `crates/bee-core/src/intent.rs` (new, 743 lines) — UTF-16-code-unit sanitizer, one
  atomic JSON file per key, fail-open reads, `next_action`-only advance, `NO_WORK_PHASES`
  resolved through `bee-core::state`.
- `crates/queen-bee/src/ledger/{mod.rs,intent.rs}` (new) — the four verb handlers.
- `crates/queen-bee/src/{dispatch.rs,groups.rs,lib.rs}` — group registered behind the seam.
- `crates/bee-core/src/lib.rs` — module export.
- `crates/queen-bench/src/fixture.rs` — seeds a five-key intent store behind a pinned
  floor; no existing floor lowered.
- `crates/bee-parity/src/{cmdcheck.rs,normalize.rs}` — the scenario table and its controls.

## The one red, and what it actually was

The final red was a true finding by the negative control, not a flake. The
surrogate-pair scenario planted its mutation at `.bee/intent/<119 a's>-.json` — the key
both runtimes *print* — while the anchor is stored at `<119 a's>.json`, because
`intentPath` (`packages/bee/lib/intent.mjs:69-71`) sanitizes a key `writeIntent`
(`:187`) had already sanitized, and `sanitizeIntentKey` is not idempotent: `/-+$/`
strips the trailing dash the 120-code-unit cut had just exposed. The mutation was
landing on a file neither runtime ever opens, so nothing could move.

The candidate cause that would have been a real port bug — the two legs sanitizing to
*different* filenames — was ruled out with evidence before anything was edited:
`DiffReport::is_clean()` includes `tree_diffs`, and this scenario's mjs-vs-queen-bee
parity leg was clean, so both runtimes produced byte-identical file sets and contents.
`crates/bee-core/src/intent.rs:75-77` re-sanitizes in `intent_path` exactly as the mjs
oracle does.

The scenario was **corrected, not widened**: the mutation now targets the path
`intent set` genuinely reads (`writeIntent` reads its anchor and refuses a differing
request, `intent.mjs:188-195`), so the declared stdout control fires for a legitimate
reason. `expect_exit` and every positive assertion are unchanged, and a new unit test,
`cmdcheck::tests::the_surrogate_anchor_is_stored_under_a_shorter_name_than_it_prints`,
pins the printed-key/on-disk-name divergence so the mutation cannot drift back onto a
file no runtime opens.

## Verification

`cargo build --release --manifest-path crates/Cargo.toml && cargo run --release
--manifest-path crates/Cargo.toml -p bee-parity -- --cmd-check --group intent` →
`PASS`, 24 of 29 registered scenarios run, every one zero-diff with its control fired.
`cargo test --release -p bee-parity -p bee-core -p queen-bench` → 231 passed.

Full trace, verify output, and `verification_evidence`: `.bee/cells/rpl-2.json`.

Cap emitted `JUDGE_STANDARD_INSUFFICIENT (F5)` — the judge routed this cap through the
`deliberate_exceptions` door and therefore did not itself enforce the D3
`red_failure_evidence` floor. The evidence in the trace does meet it (1999 characters,
quoting the verbatim red and its diagnosis); the two declared exceptions are the
unparseable-anchor scenario (its stderr embeds V8's vs serde_json's own parser message —
a divergence in the shared `readJson` primitive, not in this group) and the deliberate
absence of a Rust mirror of `packages/bee/tests/test_intent.mjs`.

## Known divergence, out of this cell's scope

An **unparseable** anchor makes `readJson` warn on stderr with the runtime's own JSON
parser message. That text can never match between V8 and serde_json. It belongs to the
shared `readJson` primitive rather than to `intent`, and no scenario here asserts it.
