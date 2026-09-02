# Verification Contract Parity — Context

**Feature slug:** verification-contract-parity
**Date:** 2026-09-02
**Shaping session:** complete (tiny brief — no interview; this finishes a named gap, it opens no product question)
**Scope:** Quick
**Domain types:** READ

## What was asked

Finish `verification-in-the-flow`. Its `plan.md` test matrix declared two rows
that none of its five cells built, and `bee close` did not catch it — the close
doors check that capped cells carry proof lines, never that a plan's matrix was
exhausted. Recorded as decision `87f9409b`.

## What was found

Both gaps are real, not cosmetic.

1. **`verify-app` is stated in eight source surfaces with nothing holding them
   together.** `rg -l 'verify-app'` over sources (generated trees, history and
   the rendered `AGENTS.md` excluded) returns:
   `packages/bee-rs/crates/bee/src/onboard/templates.rs` (the
   `VERIFY_APP_SKILL_NAME` constant), `packages/bee-rs/crates/bee/src/onboard/tests.rs`,
   `packages/bee/AGENTS.block.md`, and the five skill bodies
   `bee-verifying`, `bee-verify-upkeep`, `bee-shaping`, `bee-planning`,
   `bee-swarming`. Rename the Rust constant and seven prose surfaces go stale in
   silence. This is the repo's own recorded pattern — a rule living in N places
   needs one test that reads all N
   (`docs/knowledge/patterns/20260826-a-rule-living-in-n-places-needs-one-test-that-reads-all-n.md`).

2. **`agents_block_render_parity` cannot catch a deleted doctrine line.** It
   pins `AGENTS.md` byte-for-byte to `packages/bee/AGENTS.block.md`. Delete the
   read-first mention from the SOURCE and regenerate, and both files agree, the
   fence stays green, and the doctrine this feature exists to add is simply
   gone. Nothing asserts the two additions are present at all.

## What will be done

One new integration test file, `packages/bee-rs/crates/bee/tests/verification_contract_parity.rs`,
in the shape of its siblings `rule_index_parity.rs` and `agents_block_render_parity.rs`:
repo text read as text, std only, no crate-internal imports, a header comment
saying what is pinned and what is deliberately not.

Two tests:

- **Name parity.** Read `VERIFY_APP_SKILL_NAME`'s value out of `templates.rs` as
  text, then assert every surface that names a verification skill uses that same
  literal — and that no surface still carries the retired `verify-<app>` form.
  The list of surfaces is derived, never hand-copied, so a sixth skill joining
  the set is covered without editing this test.
- **Doctrine presence.** Assert `packages/bee/AGENTS.block.md` still carries the
  feature map as a read-first state layer beside `docs/knowledge/`, and still
  carries the user-facing-surface case with `green:live` inside the existing
  `agents-proof-at-cap` bullet. Assert on the load-bearing substance, not on a
  whole sentence — a test that pins prose verbatim breaks on every reword and
  gets deleted.

## Boundary

It ends at that one file. No product code, no skill edit, no doctrine edit, no
new door, no config key. If a test turns red on the tree as it stands, that is a
finding to report, not a licence to edit the surface it caught.

## Decisions cited, not re-opened

`d0e3c3a0` (D1, the fixed name), `9f4f90f0` (D8, the nested source path),
`c93a6948` (D4, read-first), `036e8a79` (D5, the proof case). This feature
proves them; it does not change them.
