# js-parity-cleanup — CONTEXT

## Request (verbatim)

> Nợ parity JavaScript không còn oracle. Sau khi xóa cây Node, ~100k dòng Rust
> vẫn mang theo js_strict_eq, spread_gates, UTF-16 slicing, glob matcher tự
> viết — mô phỏng đúng "những gì Node từng làm" nhưng giờ chỉ còn comment làm
> chứng. Cộng thêm Bail::NeedsNode — abstraction chết từ thời strangler vẫn
> xuyên qua signatures. Đây là maintenance liability thuần, nên dọn.

## Evidence base (3-worker inventory, 2026-08-04)

- Node tree is fully deleted: zero `.mjs`/`.cjs` files in the repo; 40+
  `bee.mjs` references survive only in comments; `crates/bee/Cargo.toml:5`
  description still claims delegation to Node.
- `state::Bail::NeedsNode` (state.rs:27-31): zero construction sites; every
  `Result<_, Bail>` fn only `Ok`s; ~50 production call sites thread the dead
  error (state.rs's own comment estimated "~15"). Unreachable
  `From<Bail> for Delegate` (drivers/mod.rs:193) and `From<Bail> for Ex`
  (status_full/mod.rs:174).
- `js_strict_eq`: 3 definitions (canonical reservations/jsval.rs:77 +
  duplicates in hooks/chain_nudge.rs:245, hooks/state_sync.rs:197), ~80 call
  sites in ~20 files. All observed inputs are JSON primitives (ids, statuses,
  phases, bools, epoch numbers). Divergence from native `Value ==` exists only
  for object/array operands (reference-false vs deep-eq) — latent, never
  exercised.
- `spread_gates`: 2 diverging definitions. state.rs:130 is the full JS-spread
  emulation (string → char-indexed keys, array → index keys);
  state_group/store.rs:70 is partial and still returns `Err(Exotic)`
  ("delegate to Node") for string/array — with no Node behind it;
  hooks/compaction.rs:201 masks that with a silent defaults fallback. The two
  paths already disagree observably.
- UTF-16 slicing: ~7 independent implementations across 15 files (~41
  occurrences), incl. a byte-identical pair (drivers/close.rs:161 ↔
  test_runner.rs:408) with circular "mirror of" comments. Semantics are
  test-locked (drivers/tests.rs:1142, test_runner.rs:553,
  status_full/tests.rs:402, release_manifest.rs:1023). The release-manifest
  code-unit sort comparator (devtools/mod.rs:282) is a REAL reproduction
  constraint: stamped manifests were sorted by UTF-16 code units.
- Glob matcher (hooks/write_guard/paths.rs:36-127): one production caller
  (checks.rs:332 via is_exclusive_path), grammar limited to `*`/`**`/`**/`,
  carries a JS-regex `\n` quirk, provenance comment names guards.mjs. No
  glob/globset crate in the dependency tree today.

## Locked decisions

- **D1 — js_strict_eq dies.** All call sites move to native
  `serde_json::Value` equality (`==`). The object/array latent divergence is
  accepted: no current call site passes non-primitives, and native deep-eq is
  the *more* correct behavior for a store that has no reference identity.
  All three definitions and the jsval parity module go away.
- **D2 — spread_gates unifies to Rust-native.** One function: object → keys
  merged over defaults; every non-object shape (null, bool, number, string,
  array) → defaults. The char-indexed string spread and index-keyed array
  spread are dropped — they emulated a JS accident, the Rust store never
  writes such shapes, and hand-editing `.bee/*.json` is already prohibited
  doctrine. `Err(Exotic)` branch and compaction's masking fallback die with
  it. Behavior change (exotic shapes → defaults instead of char/index keys or
  bail) is logged as a decision.
- **D3 — UTF-16 helpers consolidate to one module; truncation goes
  char-based.** All slicing/truncation helpers merge into a single utility
  module. Display/log truncation caps (failure excerpt 500, decision line
  160, backlog field caps) switch to Rust-native `char` counting — outputs
  change only for astral-plane input; tests updated accordingly and the
  change logged. EXCEPTION: the release-manifest sort comparator keeps
  code-unit ordering (renamed/re-commented with its real, non-JS rationale:
  byte-for-byte reproduction of already-stamped manifests).
- **D4 — glob matcher: prefer `globset` crate, fall back to de-JS'd
  in-place.** The executing cell validates the existing `glob_matcher_vectors`
  against `globset`; if the narrow grammar maps cleanly, swap and delete the
  hand-rolled matcher; if it diverges, keep the local matcher but strip the
  `\n` quirk and JS provenance, re-specifying the grammar natively. Either
  way the JS-mirror framing dies.
- **D5 — Bail enum deleted wholesale.** The 6 `Result<_, Bail>` signatures
  become infallible (return `T` / `Option<T>` directly); ~50 call sites
  simplify; both dead `From` impls removed. `status_full::Ex::Bail`,
  `feedback::{Scope,Listing}::NeedsNode`, `ManifestOut::NeedsNode`, and
  `prompt_context::NeedsNode` are LIVE, distinct types — untouched here.
- **D6 — live NeedsNode delegation signals are out of scope** → backlog item
  (they signal "delegate to Node" at runtime with no Node behind them; that
  is a behavior question, not a dead-code sweep).
- **D7 — stale-oracle comment sweep.** Cargo.toml description, `bee.mjs`
  references, "strangler front door", and CUTOVER headers that claim Node
  still exists are updated/removed; comments that explain *current* semantics
  stay.

## Done means

`cargo test --release` green in the worktree; zero occurrences of
`js_strict_eq`, `spread_gates` (old dual form), `state::Bail`/its `NeedsNode`,
the jsval parity module, and the guards.mjs-provenance matcher framing;
one UTF-16 utility module with only the manifest-sort comparator keeping
code-unit semantics; behavior changes D2/D3 logged as decisions.
