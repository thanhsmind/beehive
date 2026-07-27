# rust-port-13 — status readers A (backlog, capture, decisions, cells status helpers)

**Status:** [DONE]

**Outcome:** Ported the first half of the status read spine into `bee-core`: `read_backlog_counts` (fold-first PBI layer + legacy `docs/backlog.md` table fallback, `product_root`-aware), `capture_queue` (pending-stub fold), `active_decisions` (supersession/redaction exclusion, decision-propagation tag overlay, `all:true` archive-union), and the cells.mjs status helpers `archived_totals`/`ready_cells`/`scribing_debt`/`global_scribing_debt`/`tier_mix`/`ceiling_scarcity_warning` — all zero-subprocess, read-only. Every reader is oracle-diffed against the real frozen mjs modules (`tests/support/status_readers_a_oracle.mjs`) over a host-real fixture (prebuilt `queen-bench --generate` binary, used as-is) plus edge fixtures (empty store, 7-shape corrupt-jsonl-tail tables per reader, unknown-field round-trips). The oracle diff caught two real divergences before green: `fold_pbis`'s hand-rolled `.trim()` didn't strip a leading BOM the way JS's `String.trim()` does (fixed by reusing `fsutil::read_jsonl`), and `tierMix`'s `ceilingShare` rendered `0.0` instead of JS's unified-number `0` (fixed with a `js_number` JSON-shape helper).

**Files:**
- `crates/bee-core/src/backlog.rs` (new — `read_backlog_counts`, PBI fold, legacy table parse)
- `crates/bee-core/src/capture.rs` (new — `capture_queue`, pending-stub fold)
- `crates/bee-core/src/decisions.rs` (new — `active_decisions`, tag overlay, archive union)
- `crates/bee-core/src/cells.rs` (extended — `read_cell`, `list_cells_where`, `ready_cells`, `archived_totals`, `scribing_debt`, `global_scribing_debt`, `tier_mix`, `ceiling_scarcity_warning`)
- `crates/bee-core/src/config.rs` (extended — `resolve_product_root`)
- `crates/bee-core/src/lib.rs` (module registration)
- `crates/bee-core/tests/status_readers_a.rs` (new — this cell's mandated single integration target, 23 tests)
- `crates/bee-core/tests/support/status_readers_a_oracle.mjs` (new — node oracle driver)

**Verify:** `cargo test --manifest-path crates/Cargo.toml -p bee-core --test status_readers_a` — 23 passed, 0 failed. Full trace + verification evidence: `.bee/cells/rust-port-13.json`.
