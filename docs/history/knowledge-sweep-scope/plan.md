# knowledge-sweep-scope — plan

Lane: `standard`. Flags: covered-contract-change, public-contracts.

## The problem, measured

`bee worktree merge` auto-commits main's dirty bookkeeping before merging, so
the dirty-main guard does not deadlock against the worktree-first rule. The
roots it sweeps come from `main_bookkeeping_roots`
(`packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:184-190`):

```rust
let mut roots = vec![".bee".to_string(), "docs/decisions".to_string(), "docs/knowledge".to_string()];
if let Some(feature) = feature {
    roots.push(format!("docs/history/{feature}"));
}
```

`docs/history/<feature>` is scoped to the merging feature. The other three are
blanket. Note the asymmetry is deliberate for history — there is already a test
proving a peer's `docs/history/<other>` is NOT swept and still refuses
(`tests.rs:1804-1833`). The intent to scope exists; it just was not carried to
`docs/knowledge`.

Real incident, 2026-08-18: merging `uat-stop-placement` produced bookkeeping
commit `7429dfda`, which swallowed 21 insertions of a SIBLING session's spec
sync to `docs/knowledge/areas/workflow-state/gates.md` — work belonging to
feature `start-feature-reservation-scope`. Nothing was lost; the authorship and
the feature attribution were both wrong. The peer session found it, not a test.

`docs/knowledge/` is the one root in that list holding AUTHORED prose. `.bee` is
the machine-written control plane and `docs/decisions` is its rendered index —
sweeping those is bookkeeping. Sweeping authored capture is taking someone
else's work.

This is the same shared-index sweep the concurrent-worker git guard refuses a
bare `git add` for while siblings are live — and then the merge path does the
equivalent itself.

## Shape

One cell. Scope `docs/knowledge` the way `docs/history` is already scoped, using
the data bee already keeps.

`feature_touched_files(root, feature)` (`verbs/drivers/close.rs:1388`) is already
`pub(crate)` and already returns the merging feature's own capped cells'
`files_changed` — `close.rs`'s own doc-deferral scan uses it the same way. Call
it, filter to `docs/knowledge/`, and pass those exact paths instead of the blanket
root.

Anything else dirty under `docs/knowledge/` then stays uncommitted, and the
EXISTING dirty-main refusal names it — that path is already built and already
tested. The message a sibling gets is "commit your own capture", which is the
honest instruction; it is what the peer's own session did by hand.

`.bee` and `docs/decisions` stay blanket. They are machine-written and shared,
and narrowing them would trade a real deadlock for an attribution nicety.

## Scope boundary

`packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` is reserved by another
session's cell `ddb-1` and is NOT edited — `feature_touched_files` is called, not
changed.

## Test scoping

The worktree suite owns this. Six existing tests pin the prefix list
(`tests.rs:1622-1893`) — `docs_knowledge_dirt_is_auto_committed_alongside_bee`
(`tests.rs:1758-1795`) is the one whose expectation changes, and it changes to
"this feature's own knowledge dirt is swept, a sibling feature's is not",
mirroring `another_features_docs_history_dirt_still_refuses_and_is_named` exactly.

## What this deliberately does NOT do

- Narrow `.bee` or `docs/decisions`.
- Change the dirty-main refusal, its message shape, or the
  `worktree_merge_commit_bookkeeping` opt-out.
- Touch `bee close`'s own `.bee`-only bookkeeping commit.
