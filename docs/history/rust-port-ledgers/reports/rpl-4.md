# rpl-4 — decisions WRITE path on the ported D9 lock

**[DONE]** — `bee decisions log|supersede|redact` now run in `queen-bee`, byte-identical to the frozen mjs oracle, appending under the same cross-process lock mjs uses. Verify green: 25 suites / 446 tests ok, conformance suite 6/6, `--cmd-check --group decisions` 15/15 zero-diff with every negative control firing.

Full trace, structured evidence and verify output: `.bee/cells/rpl-4.json`.

## Files touched

| File | What |
|---|---|
| `crates/bee-core/src/decisions.rs` | write side: `with_decisions_lock_sync`, tag taxonomy gate, `docs/**` citation sweep, `log_decision`/`supersede_decision`/`redact_decision` |
| `crates/bee-core/src/capture.rs` | `add_capture_stub_lists` + `normalize_list_items` (deviation 1) |
| `crates/bee-core/tests/decisions_lock_conformance.rs` | NEW — the cross-runtime proofs `--cmd-check` cannot make |
| `crates/bee-core/tests/support/decisions_driver.mjs` | NEW — node driver over the frozen `decisions.mjs` / `capture.mjs` |
| `crates/queen-bee/src/ledger/decisions.rs` | NEW — the group's three write verbs + usage fallback |
| `crates/queen-bee/src/ledger/mod.rs`, `groups.rs` | one `pub mod` line, one `register` line (the rpl-1 seam, unchanged elsewhere) |
| `crates/queen-bee/src/dispatch.rs` | ported-groups exact-set pin |
| `crates/bee-parity/src/cmdcheck.rs` | 15 decisions scenarios + obligations test + `PORTED_GROUPS` |
| `crates/bee-parity/src/normalize.rs` | `scanned_at` declared volatile, IsoMillisZ-gated |

## Two bugs this cell found

1. **`readdir` order is not the same in the two runtimes.** `fs.readdirSync` is libuv `scandir` + `strcmp`, so Node hands mjs a **byte-sorted** list; `std::fs::read_dir` yields raw directory order. The citation sweep's `files[]` is serialized *into the supersede event*, so this was a persisted-store byte divergence, not a cosmetic one. Fixed in `collect_sweep_files`; caught by the oracle comparison, and no docs-less `--cmd-check` fixture could ever have shown it.
2. **One of my own negative controls could not fire.** The harness refused `supersede-blank-id-refused-by-the-store` because its control renamed `decisions.log` for a `decisions supersede` argv. Fixed with `mutate_registry_decisions_supersede`.

Separately, the workspace-wide run at the head of the verify caught `bee-parity`'s `unported_ledger_groups_still_have_zero_scenarios_registered` — the cross-cutting pin class the cell warned about. Keeping that run un-narrowed was load-bearing.

## The red floor, and why the first one was thrown away

The first race test raced mjs appends against Rust appends. It **passed with the lock deleted** — one small `O_APPEND` write is already kernel-atomic, so the lock was not what it measured. A test that stays green with the code under test removed is not a proof, so it was rebuilt around what `decisions.mjs:39-47` says the lock actually guards: `archiveDecisions`'s read-prune-**rename**. The mjs leg now runs the real frozen archiver while Rust appends. With the lock removed that version fails with `5 of 90 reported appends were LOST`.

Two further floors, each reverting one piece of the code under test: removing the readdir sort reds the sweep oracle; reverting the seam registration reds `--cmd-check` with a real cross-runtime divergence (mjs logs, queen-bee refuses) rather than the harness's own zero-scenarios refusal.

## Declared, not masked (rpl-11 discipline)

- **The citation-sweep-with-hits path is not a `--cmd-check` scenario.** Each queued capture stub embeds the fresh event uuid inside a `dids` **array** and inside `outcome` **prose** — positions `normalize`'s key-gated, deny-by-default masking cannot reach. Making it green would need pattern masking (which that design rejects) or excluding the capture queue from the tree diff (which would blind rpl-3's whole surface). It is compared against the frozen mjs oracle in the conformance suite instead, where the ids are known values and substitution is exact.
- **`contention.jsonl` is unprovable through `--cmd-check`** (`.bee/logs` is in the tree-diff exclusion set), so it is read directly after a real contention — including the exact **16-attempt** retry budget matching on both runtimes and the byte-identical typed busy refusal.
- **Text-mode `log` and `supersede` are `--json`-only** in the harness (both print a fresh uuid as bare prose). `redact` *is* proven in text mode, because its line echoes the argv-supplied id.
- **`js_excerpt`** uses `from_utf16_lossy`, so a cut splitting a surrogate pair yields U+FFFD where mjs keeps a lone surrogate. Reachable only when a docs line's 158th UTF-16 unit is a high surrogate; documented at the function, deliberately untested.

## Deviations

1. **auto-add** — `bee_core::capture` gained `add_capture_stub_lists`. rpl-3 ported only `normalizeList`'s *string* branch ("the ONE input shape the CLI can produce"), but `handleDecisionsSupersede` always passes real **arrays**; joining them to reuse the string branch would split a docs path containing a comma into two entries. `add_capture_stub`'s public signature is unchanged and `ledger/capture.rs` was not touched.
2. **auto-fix** — the `readdir` sort above.
3. **auto-fix** — the two cross-cutting group pins.

## Notes for rpl-5

- `decisions active|search|archive|tag|render` resolve in the registry, find no handler, and take `dispatch.rs`'s honest "not ported into this binary yet" refusal. No scenario claims them.
- The group's `usage_fallback` already names **all eight** verbs — it describes the bee CLI, not the port's progress, so it needs no change when the read side lands.
- `archiveDecisions` and the `tag` write verb are unported by design; the conformance race deliberately uses the frozen mjs `archiveDecisions` as its rewriting counterparty.
- `serde_json` `preserve_order` is load-bearing and the appended JSONL line does **not** go through `jsonout`. That is safe only because every key a decisions event can carry is a fixed non-numeric literal — pinned by `event_keys_are_never_integer_like`, which will start failing the moment a dynamic key appears.
