# Proof Completeness Rules — Context

**Feature slug:** proof-completeness-rules
**Date:** 2026-09-20
**Shaping session:** complete
**Scope:** Quick
**Domain types:** ORGANIZE

## What was asked

Spec drop `3d41a292` (waggledance@aicoworker) proposed adopting four proof
practices distilled from openJev-verdict-2.0 in
`docs/history/research/openjev-verdict-2-xia.md`. The owner chose the two
one-line rules now and left the two shaped ones for later under trigger `the-owner-decides-whether-to-adopt-the-r__c03a77c0`.

## What was found

The brief's own Bottom Line agrees with that split: these two are "one-line
AGENTS.md edits and route through bee-shaping as one `tiny` docs-lane item",
while the floors item "changes a recorded artifact shape (the proof line) …
routes as a feature, not as `tiny`".

Two facts from reading the repo, both of which changed the plan:

1. **`AGENTS.md` is not the source.** It is RENDERED from
   `packages/bee/AGENTS.block.md` by `bee dev regen`
   (`packages/bee-rs/crates/bee/tests/agents_block_render_parity.rs:1-15`).
   That test pins the bytes between `<!-- BEE:START -->` and `<!-- BEE:END -->`
   against `render_agents_block`. Editing `AGENTS.md` directly would be editing
   the output, and the next regen would drop it. The edit goes in the block.
2. **Marked rules cost three copies.** `tests/rule_index_parity.rs` pins every
   `<!-- rule: … -->` id across `AGENTS.md`, `AGENTS.block.md` and the
   rule-homes index, each row needing an indented `spoken:` line. The section
   already carries unmarked bullets, and the test only governs marked ones
   (`every_marked_rule_has_an_index_row_and_every_row_a_marker`), so unmarked
   bullets are legal and carry no parity obligation.

## What will be done

Two unmarked bullets into `## Prove, then say so` in
`packages/bee/AGENTS.block.md`, between the freshness bullet and the
"Evidence is what the build already emits" bullet:

- **Completeness beside freshness.** Report the whole proof, naming what was
  skipped, filtered or narrowed beside what passed. The existing rule governs
  only freshness, so a scoped-green cap that quotes the green suite and omits
  the one it never ran passes the rule as written.
- **Cheapest proof first.** Before an expensive wave or long run, run the short
  check that would catch a broken environment and stop on its red; and a run
  narrowed by a flag or filter says so where its result is reported.

Then `bee dev regen` to render `AGENTS.md`, because the block is in the
release-manifest payload and the render fence is byte-pinned.

## Locked Decisions

Decision log: `c03a77c0`.

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Adopt exactly these two, as UNMARKED bullets. | `c03a77c0`. Marking them pulls in the three-copy parity obligation the owner did not ask for; marking later is additive, under trigger `the-owner-decides-whether-to-adopt-the-r__c03a77c0`. |
| D2 | The edit lands in `packages/bee/AGENTS.block.md`, never in `AGENTS.md`. | `AGENTS.md` is the rendered output; a block edit that is never regenerated is "a rule no agent ever reads" (the fence test's own words). |
| D3 | The other two practices — judge ABSTAIN, baseline floors in cap proofs — are NOT in scope. | The brief routes floors as a feature, and flags that the confirm-checklist behind the smoke item may collide with the unattended control loop under `gate_bypass`. Both stay parked on the spec-drop backlog row. |
| D4 | The code half of the smoke practice — making a narrowing flag print its own warning — is NOT in scope. Only the rule text lands here. | That is a change to verb output across several commands, not a doctrine line. |

## Proof

Docs lane, so the proof is parity and pointer checks: `bee dev regen` runs
clean, `agents_block_render_parity` and `rule_index_parity` stay green, and
`bee dev release-manifest --check` matches, because the block is a hashed
payload root.

## Deferred under trigger `the-owner-decides-whether-to-adopt-the-r__c03a77c0`

Each item below is also carried by the spec-drop backlog row for `3d41a292`.

- Marking the two rules as invocable ids, with their index rows and spoken
  lines — additive, filed only if the owner wants them invocable by name.
- The two shaped practices and the smoke warning's code half — on the spec-drop
  backlog row for `3d41a292`.
