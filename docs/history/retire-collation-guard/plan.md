---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Retire Collation Guard

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change,
multi-domain. No hard-gate flag.

Why this is the least workflow that protects the work: three deletions and six
test rewrites, all in one crate, with the compiler proving reach. What earns a
plan at all is that the tests being rewritten currently assert the defect, so
"make the suite green" is not a safe instruction on its own.

## Requirements (from CONTEXT.md)

- **D1** — retire `collation_safe` in both copies (`decisions/render.rs:100`,
  `backlog.rs:1139`); they are separate functions, not one helper.
- **D2** — retire `id_sort_safe` (`backlog.rs:214`) in the same pass.
- **D3** — the router's misleading message is NOT changed.
- **D4** — `run_supersede`'s ASCII guard is left in place.

## Discovery

Two commands refuse today and both refusals are masked:

- `bee decisions render` (every shape) — one stored scope is
  `feature:opencode-support`; the colon fails `collation_safe`, so
  `build_decision_index_body` returns `Ok(None)` (`render.rs:196`), `do_render`
  turns it into `Err(Err2::Ex)` (`:290`), and the dispatcher sees `None`.
- `bee backlog pbi list` (no `--status`) — the store holds legacy ids `P72`,
  `P41`; `id_sort_safe`'s alphabet is `^p-[0-9a-f]+$`, so `:1046` returns
  `None`. Verified live: the command refuses right now, while
  `bee backlog pbi list --status proposed` works.

`bee backlog render` does NOT refuse — every PBI id in the store passes
`backlog.rs:1139`'s `collation_safe`, so that call site's `false` branch is not
currently reachable. It is retired for consistency with D1, not to fix a live
failure.

No doc, handbook page or knowledge concept documents the calibrated-alphabet
behavior as a promise; every description of it is an inline source comment.

**The guard is narrower than the comparator it guards.** `lc_primary_key`
(`backlog.rs:1118-1134`) already assigns primary keys to `_ - , ; : ! ? .`
individually, in ICU order, and falls back to `(1, 100 + c as u32)` for
everything else — a total function with no undefined region. `collation_safe`
admits only `_ - .` plus alphanumerics and space. So the colon that disables
`bee decisions render` is a character the comparator was deliberately
calibrated for; the guard simply was never widened to match its own model.

This changes the risk assessment rather than the decision. For the keys
actually present in this store, retiring the guard exposes ordering that is
already implemented and already calibrated, not ordering that has to be
invented. Everything outside the enumerated punctuation still orders
deterministically through the fallback arm.

## Approach

Delete the three functions and their call-site guards, letting the sort that
follows run unconditionally. The ordering that results is Rust's own stable
sort, which is deterministic — that determinism, not agreement with a deleted
oracle, becomes the contract, and the generated files' "byte-identical for the
same store" promise still holds.

The tests are the delicate half. Four of the six assert that an exotic key
DISABLES a command; those assertions are asserting the defect and must invert,
not be deleted — the new case is that the command succeeds and produces a
stable order. The two direct unit tests on the retired functions go away with
them. `render_content_groups_by_weight_then_collated_id`'s happy-path table
assertions stay untouched.

**Rejected alternatives.** Widening the alphabet to admit `:` — rejected by the
user's choice: it cures the symptom and the next unusual character reproduces
the failure with the same misleading message. Changing the router message —
rejected by D3.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| Test inversion | MEDIUM | Four tests currently assert the defect; rewriting them wrongly hides a regression behind green | Each inverted test asserts a successful render AND a specific stable order, never just "no panic" |
| Ordering change | LOW | Keys that previously refused now sort; keys that previously sorted are untouched, and the comparator already enumerates the punctuation involved (`lc_primary_key:1122-1133`) | The generated files' existing content is unchanged for the current store — `decisions render --check` and `backlog render --check` after the change |
| Other masked exits survive | LOW, accepted | `active_decisions` and `build_tag_overlay` keep their own `Exotic` returns (`decisions/read.rs:55-72`, `:146-152`) for null events and inconsistent date comparators; if one fires, `decisions render` masks again through the same router path | Named here, not fixed — D3 keeps the router message out of scope, so the cell must not claim the masking is cured |
| Reach | LOW | Three private/`pub(crate)` functions in one crate | Compiler |

## Shape

One cell. The three retirements share a single rationale, the same test file
neighborhoods, and one verification: the two refusing commands run.

**`rcg-1`** — retire the three guards and invert their tests.

Cell-facing specifics:
- The two `collation_safe` copies are separate functions with identical bodies;
  delete both, and their call-site conditionals at `render.rs:196`, `:230`, and
  `backlog.rs:1196`.
- `id_sort_safe` (`backlog.rs:214`) has two call sites: `:1046` and
  `:1348-1350`.
- Tests to invert (assert success plus a stable order): `backlog.rs:1786`
  (`:1819-1825`), `backlog.rs:2148` (`:2194-2199`), `decisions/tests.rs:865-871`.
- Tests to delete with their subject: `backlog.rs:1740` (`:1770-1772`),
  `backlog.rs:2061`, `decisions/tests.rs:762` (`:796-799`).
- Leave `run_supersede`'s ASCII guard and the router message alone (D3, D4).

## Test matrix

Standard — the triad at its smallest demonstrating size.

| Case | Probe |
|---|---|
| Happy path | `bee decisions render --check` and `bee backlog render --check` both run; `bee backlog pbi list` with no `--status` lists ids including the legacy `P`-prefixed ones |
| Edge | A group key carrying a colon (the real `feature:opencode-support` scope) renders into its own group; a legacy `P72` id sorts beside `p-<hex>` ids deterministically |
| Error path | An unreadable or malformed store still refuses through its own existing path, not through a retired guard |

Declared suite at cap:
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.

## Out of scope

- The router message for a handler declining a registry-valid shape (D3) —
  filed as its own P2.
- `run_supersede`'s ASCII guard (D4).
- Regenerating `docs/decisions/index.md` as part of this cell: the command
  working is the deliverable; running it is a separate, reviewable write.
