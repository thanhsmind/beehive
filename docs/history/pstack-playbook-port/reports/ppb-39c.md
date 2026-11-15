# ppb-39c — Check the consolidated knowledge

Verdict: pass. `bee knowledge check` is green, every anchor in the concept ppb-39 wrote is a real file, and delivery.md lists every capped cell of the feature. Two minor wording notes below; neither blocks.

## bee knowledge check

`.bee/bin/bee knowledge check` exits 0:

```
knowledge check: 328 concept(s) in 349 file(s), 0 OKF error(s), 0 profile error(s), 554 profile warning(s) — OK
freshness: 47 confirmed, 79 suspect, 0 orphan, 18 unverified
```

None of the 554 warnings is on the new concept. The concepts ppb-39 touched carry these warnings, all older than this feature:

- `areas/workflow-state/gates.md`: missing_read_when, directory_anchor (`packages/bee-rs/crates/bee/src/`), long_concept (904 lines).
- `areas/doctrine-layer/per-task-route-tester-pairing-and-proof-floors.md`: not_canonical.

The new concept is not in the 19 orphans. `areas/workflow-state/gates.md` links to it, and so does the doctrine-layer index.

## Anchors of the new concept

`areas/doctrine-layer/situation-layer-class-index-session-group-and-parity-fence.md` has 31 anchors, and all 31 are files on disk. None is a directory. I checked its main claims against the code:

- `situations.rs` parses `## Situations` and nothing else. Lines that break the grammar are skipped (test `a_malformed_line_is_skipped_and_its_neighbours_still_parse`). It reads the repo file first and falls back to a compiled-in copy of the nine class indexes through `include_str!` (`class_situations`). This matches the concept.
- `orient.rs` adds a `situations` array and prints `situation: skills/bee-planning/playbooks/{class}/<name>.md — <triggers>` lines. This matches.
- `plan_group.rs` refuses a plan with "plan.md cites no situation of {class}; cite one of: …". Tests cover four cases: an unlisted name, a class with no situations, no class, and a citation of the index only. This matches.
- `skills/bee-planning/playbooks/session/index.md` and the ten session files exist. `situations.rs` does not read them, and `packages/bee/LEADER.md` sends the leader to `session/index.md` directly. This matches.
- `LICENSE.pstack` sits beside the playbooks. This matches.

Notes:

1. The concept says every class index "closes with" the figure-it-out line. In all nine indexes the line is the last item under `## Situations`, but a `Proof floor:` line comes after it. The meaning holds; the word "closes" is a little loose.
2. The concept says 23 pstack playbooks came in. On disk there are 16 class situation files plus 10 session files. With the D3 merges and bee's own additions, that does not contradict 23. I did not reconcile the two counts one playbook at a time (unverified, not a defect).

## delivery.md against the capped cells

The store lists ppb-1 through ppb-39 as capped, with ppb-39c as the only open cell. I resolved all 76 short shas in delivery.md as commits in this branch. For each capped cell I matched the commit that carries its `cell:` trailer against delivery.md:

- ppb-1 through ppb-30c and ppb-37 through ppb-38c are cited by sha.
- ppb-31 through ppb-36c fall inside the cited range `7414b8ebe` through `dac33e0be`. `git log` over that range shows all twelve cells' commits, including ppb-36's `7cc3065dc`.
- ppb-39 is the record itself ("Knowledge consolidated and this record (cell ppb-39)").

A side finding outside this cell: the commit for ppb-36 (`7cc3065dc`) has a checker-style subject ("Check the pstack verification-skill merge…"), although ppb-36 is the merge cell. delivery.md is not wrong because of it.

## Scope

The only file this cell edited is this report.
