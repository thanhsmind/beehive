# pic-2 — Prove the preview id-clash refusal red then green

Contract: preview-id-clash (fd9a503f), decision D1 (e03c4bf8).

## Tests

In `packages/bee-rs/crates/bee/tests/gate_preview.rs`, building on the three tests pic-1 added (89f62b698):

- `preview_refuses_id_held_by_archived_cell_of_other_feature`: `.bee/cells/archive/old-feat/c-1.json` with feature `old-feat`. The fixture plan.md packet names `c-1`.
- `preview_refuses_id_held_by_live_cell_of_other_feature`: `.bee/cells/c-1.json` with feature `other-feat`.
- `preview_passes_id_held_by_cell_of_same_feature`: a live `c-1` of `preview-feat` previews and writes `gate_preview`.

Both refusal tests now share `assert_refused`, which checks five things: a non-zero exit, `"c-1"`, `archived cell` or `live cell`, the quoted holding feature, the `rename the id in plan.md` hint, and no `gate_preview` on the lane record. Before this change, pic-1's live test did not check for the id or the rename hint. The archived seed was also always filed under `archive/old-feat`, whatever its feature. It is now filed under its own feature.

## Run 1 — red, plan_packets.rs at 89f62b698~1

    git show 89f62b698~1:packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs > packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs
    cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test gate_preview preview_refuses

    thread 'preview_refuses_id_held_by_archived_cell_of_other_feature' panicked at crates/bee/tests/gate_preview.rs:487:5:
    Gate preview for feature "preview-feat" (1 cell(s), plan sha256: 2011b88...):
    failures:
        preview_refuses_id_held_by_archived_cell_of_other_feature
        preview_refuses_id_held_by_live_cell_of_other_feature
    test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s

Line 487 is `assert!(!out.status.success())`. The parent previewed the clashing id and succeeded, which is the reported bug. Restored with `git checkout HEAD -- plan_packets.rs`. `git status --short` then listed only `M packages/bee-rs/crates/bee/tests/gate_preview.rs`.

## Run 2 — green, cell verify at HEAD

    PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test gate_preview

    test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

All three clash tests appear by name as `ok`.

## Run 3 — full proof

    cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml

Exit 0. 49 `test result: ok` lines, 0 `FAILED`, 6109 passed in total.

No production code was edited.
