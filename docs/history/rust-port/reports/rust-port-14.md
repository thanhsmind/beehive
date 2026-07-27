# rust-port-14 — status readers B1: review derivation via in-process gix

**[DONE]** — worker Dave, lane high-risk, commit `424002f`.

Full trace, verification evidence and verify output: `.bee/cells/rust-port-14.json`.

## Outcome

`reviews.mjs`'s `listCandidates` / `listReviews` / `deriveCandidateStatus` and
`bee.mjs`'s `buildReviewBlock` are ported to `bee_core::reviews`. Both git
questions the mjs derivation answers by spawning — `merge-base --is-ancestor`
and `rev-list <ref>..HEAD --count` — are now answered **in process with gix**.
Zero subprocess on the review-derivation path (D5 headline), proven by a
fixture that removes `git` from `PATH` entirely and still derives the same
non-degraded block.

Tri-state ancestry is preserved end to end: an unknown object resolves to
*unresolved* → `review stale` + `range unresolvable`, never `covered: false`.

## Files touched

- `crates/bee-core/src/reviews.rs` (new)
- `crates/bee-core/tests/status_readers_b1.rs` (new, 23 tests, one target)
- `crates/bee-core/tests/support/status_readers_b1_oracle.mjs` (new, tracked node driver)
- `crates/bee-core/src/lib.rs`, `crates/bee-core/Cargo.toml`, `crates/Cargo.lock`

## What the fixtures caught

Two findings came from the divergence-class fixtures rather than from reading:

- **`refs/replace` was silently ignored.** gix 0.86 binds `core.useReplaceRefs`
  to a variable named `is_disabled` and loads no replacement table when it is
  true (`src/open/repository.rs:559-565`), so a `git replace --graft` repo got
  ancestry answers that disagreed with the CLI. Compensated at open, documented
  at the call site, and pinned by `divergence_refs_replace_graft` — which goes
  red if upstream corrects the inversion.
- **The shallow-clone fallback was dead code.** A seeded-mutation probe deleted
  the `rev_walk` fallback and nothing went red, which exposed that the shallow
  fixture was vacuous. The fixture was strengthened to cross the graft in both
  question types; gix answered both correctly, so the unreachable fallback was
  removed rather than kept as decorative defensiveness.

## D5 budget (approach.md:21)

`approach.md:21` made the gix choice conditional on the query set measuring
under 2 ms on this repo. **It does not**: on the real 971-commit history
(62 candidates, 6 sessions, 21 distinct git questions) the bare query set costs
**12.4–15.2 ms** median on the release profile. So the escape hatch that same
line prescribes is implemented — an **mtime-keyed cache fronts the query set**,
never a reversion to spawning git. Fronted hot read: **1.59–1.82 ms** median,
still zero subprocess.

The cache is safe by construction, not by luck: only *definite* answers are
stored (so an object that arrives later is always re-asked), only *object-id*
specs are stored (so a ref name can never silently move under a cached answer),
and the key covers resolved HEAD, the shallow boundary, replace refs and
alternates. `git_cache_invalidates_when_head_moves` and
`git_cache_never_persists_unresolved_answers` bind both properties.

## Notes for downstream

- **rust-port-15** consumes `reviews::build_review_block(root)` for the status
  command's `review` block. Be aware it produces an additive runtime artifact,
  `.bee/runtime/review-git-cache.json` — never read by the frozen mjs layer,
  self-invalidating, and best-effort on every read and write.
- gix is pinned `default-features = false` (features: `revision`, `sha1`). This
  is load-bearing for D8: the lock grew by 121 pure-Rust packages with no
  openssl / curl / reqwest / libgit2 / libz-sys / zlib-ng / cmake, so musl
  static builds stay viable.

## Outstanding questions

None.
