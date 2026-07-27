# rpl-1 — [DONE]

**Outcome.** The group/verb dispatch seam in `queen-bee` and the generic
`bee-parity --cmd-check` arm both land, seam-only: no ledger group is
registered, which is deliberate so the first group's red cannot be confused
with a harness fault.

Full trace, verification evidence, deviations and verify output:
[`.bee/cells/rpl-1.json`](../../../../.bee/cells/rpl-1.json).

## Files touched

| File | What |
|---|---|
| `crates/queen-bee/src/dispatch.rs` | new — the seam: registration table + the port of `bee.mjs` `main()`'s argv path |
| `crates/queen-bee/src/groups.rs` | new — THE registration table. One line per ported group; `main.rs` never changes again |
| `crates/queen-bee/src/jsonout.rs` | new — JS-compatible JSON emission (integer-like key ordering) |
| `crates/queen-bee/src/lib.rs`, `src/main.rs` | wire the seam in; route every JSON payload through `jsonout` |
| `crates/queen-bee/src/status.rs` | `to_json_stdout` now emits in JS key order |
| `crates/bee-parity/src/cmdcheck.rs` | new — the `--cmd-check` arm: scenarios, `--group`/`--all`, seeding, per-scenario controls |
| `crates/bee-parity/src/runner.rs` | arbitrary argv; `RunResult` captures **stderr** |
| `crates/bee-parity/src/differ.rs` | stderr is in the diff surface |
| `crates/bee-parity/src/normalize.rs` | key-gated **and** shape-gated deny-by-default allowlist; declared mjs stderr artifact |
| `crates/bee-parity/src/mutate.rs` | `MutationTarget` + `replace_exactly_once` — per-scenario controls |
| `crates/bee-parity/src/enrich.rs`, `src/main.rs` | `iso_now_millis` shared; the `--cmd-check` entry point |
| `crates/queen-bench/src/fixture.rs` | seeds the command registry bridge so both legs resolve the SAME registry |

## What the next cells inherit

- Register a group: one line in `queen_bee::groups::register_all`.
- Register scenarios: `bee_parity::cmdcheck::all_scenarios`. Each scenario
  **must** declare a mutation target (the store it reads) and the channel its
  control fires on, or registration refuses it.
- Pin your cell's verify to `--cmd-check --group <name>`. It exits non-zero
  while your group has zero scenarios, so a port with no scenarios cannot go
  green.
- Seeding rides **on top of** `queen-bench --generate`, never instead of it.
- Never mask a volatile field without adding it to `normalize::VOLATILE_FIELDS`
  **with a shape**; widening the mask to clear a red is a prohibition.

## Two findings the next cells must not re-discover

1. **`status`'s JSON emitter was not byte-compatible with `JSON.stringify`.**
   JS hoists integer-like object keys ahead of string keys in ascending
   numeric order; `serde_json` with `preserve_order` does not. Proven red on
   the real binaries, then fixed in `jsonout.rs`. Every `queen-bee` JSON
   payload now goes through it — do not re-introduce a bare
   `serde_json::to_string_pretty` at a new call site.

2. **`rpl-7` owes a numeric-string-key scenario.** The cell offered "record
   that no ledger object can carry such keys" as an escape; the evidence
   refutes it. `bee reviews record --kind manifest|preflight|decision|finding|uat`
   writes caller-supplied JSON into `.bee/reviews/<id>.json` with **no key
   sanitization** (`packages/bee/lib/reviews.mjs:283-327`, fed by
   `packages/bee/bee.mjs:4315-4322` and `:4398-4404`), so `{"0":…,"1":…}` is
   reachable in a real ledger store. Every other counts/tally/group-by shape
   across the six groups keys off a closed enum or renders to markdown, so
   `rpl-7` is the only cell that owes this.

## Notes

- Two `--cmd-check` flags exist beyond the cell's ask, both refusals rather
  than features: a **bare** `--cmd-check` is an error (a run with no selector
  would pass on whatever somebody else registered), and `--group <unknown>` is
  an error rather than an empty, green run.
- No advisor consult was triggered. The reds in this cell were the required
  red-first phase and two build-the-test iterations, each with an unambiguous
  self-diagnosed root cause — not a failed verify attempt against a
  supposedly-finished implementation.
