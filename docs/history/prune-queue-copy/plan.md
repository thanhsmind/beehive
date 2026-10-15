---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Prune reads the worktree capture queue by content

## Summary

`bee worktree prune` stops keeping a merged, clean worktree just because its
capture queue file is a copy of main's. It still keeps any worktree whose queue
holds a note main does not have, and it still keeps on any doubt.

Mode: `high-risk` — 1 risk flag: covered-contract-change. Taken high-risk on judgment: the change relaxes a keep-condition that guards unflushed notes against deletion.
Why this is the least workflow that protects the work: one product file, but a wrong relaxation deletes a worktree with notes nobody flushed, so it gets the five-seat plan check and a red-first test.

Playbook: `skills/bee-planning/playbooks/bugfix.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1: keep for the queue only on a pending worktree stub whose id is absent from main's queue.
- D2: every doubt keeps; a missing main queue counts as empty.
- D3: the keep message names the unique count and one example id.
- D4: the handoff keep is unchanged.

## Load-bearing claims

Labels are `read` (opened at that line) or `ran` (command output held); evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The check today keeps on any non-empty queue file | read | packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs:136 | `Ok(meta) if meta.len() > 0 => Some(format!(` |
| 2 | The classifier holds the main root it needs for the comparison | read | packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs:413 | `if let Some(reason) = interrupted_operation_present(check.main_root, check.id) {` |
| 6 | Worktree bootstrap copies only onboarding and config, never the queue | read | packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs:977 | `let onboarding = copy_if_absent("onboarding.json")?;` |
| 7 | Host repos gitignore the queue by default, so there a worktree queue holds only its own stubs | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:32 | `".bee/capture-queue.jsonl",` |
| 3 | An existing test pins the keep with a queue line that has no kind | read | packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:3838 | `fn a_non_empty_capture_queue_keeps_even_though_the_tree_reads_clean() {` |
| 4 | Every worktree is kept today for this reason | ran | .bee/bin/bee worktree prune --dry-run --older-than-days 0 | `capture-queue.jsonl is non-empty — unpromoted capture stubs are precious and invisible to the clean-tree check (D8a); keeping.` |
| 5 | The concept row that describes the keep | read | docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md:80 | `Nothing precious is sitting only in the gitignored part of the tree` |

## Discovery

The queue holds `stub` lines and `flush` lines keyed by `id` (1355 stubs and 1289 flushes in main's queue on 2026-10-15). In this repo the queue file is git-tracked despite the ignore entry, so each worktree holds its branch's committed copy (the bootstrap copies only onboarding and config, claim 6); a set comparison of 31 worktree queues against main's found 0 unique stub ids. In a host repo the queue is gitignored (claim 7), a worktree queue holds only the worktree's own stubs, and this keep-condition is the only guard against deleting them, so the comparison must be exact.

## Approach

Change only the queue half of `precious_state_present`: pass the main root in, read both queues, and keep only on a pending worktree stub that main's queue never names, or on any read or parse doubt (D1, D2, D3). The handoff half stays (D4).

Rejected alternatives:
- Untrack the queue in this repo — fixes only this repo, and host repos still need an exact check.
- Reuse the existing pending-stub readers (capture.rs, session_close nudges) — they skip unparseable lines (fail-open), which breaks D2.
- Skip the check when the file is tracked and clean — differs from D1 and does nothing for host repos.
- Compare file sizes or prefixes — a worktree may append after its copy; only ids answer the question.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Queue keep relaxation | HIGH — a wrong answer deletes unflushed notes | pqc-1 | red-first test: a copy of main's queue reads dead before the fix fails; unique pending stub keeps; doubt cases keep |

Waves: one cell.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "test", "reason": "red-first test plus the fix in prune.rs; the code role's route runs a cheap model with thinking off"},
    {"stage": "code", "classification": "not-applicable", "role": "code", "reason": "its route (paseo-pi-flash) runs thinking off; the one cell goes to role test"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "red-first tests ride the cell"},
    {"stage": "doctrine", "classification": "not-applicable", "role": "docs", "reason": "the concept row is updated inside the cell"},
    {"stage": "lookup", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of a file outside its cell", "reason": "read-only gathers"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no narrow lookups planned"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "no free-form generation stage"},
    {"stage": "review", "classification": "required", "role": "review", "reason": "high-risk slice judge over the behavior_change cell"},
    {"stage": "advisor", "classification": "required", "role": "advisor", "reason": "the plan-step hat wave synthesis is recorded as the advisor ref"},
    {"stage": "plan-hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-risks", "classification": "required", "role": "hat-risks", "reason": "high-risk five-seat wave"},
    {"stage": "plan-hat-value", "classification": "required", "role": "hat-value", "reason": "high-risk five-seat wave"},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "the leader plans in-session"},
    {"stage": "supervise", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes: one obvious shape"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"}
  ]
}
```

## Shape

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Content-aware queue keep | D1-D4 | prune never reclaims a worktree today | 1 | red-first prune tests, then the dry-run over the real worktrees |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pqc-1 | Keep a worktree for its capture queue only when it holds a note main lacks | `packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs`, `packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs`, `docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md` | — | `bee worktree prune --dry-run` no longer keeps merged clean worktrees for a copied queue | `cargo test -p bee worktree::` |

```json
[
  {
    "id": "pqc-1",
    "feature": "prune-queue-copy",
    "title": "Keep a worktree for its capture queue only when it holds a note main lacks",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["ad9d30dc-b20f-49f7-8b14-57533e054852", "f89d6cd6"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs", "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md"],
    "read_first": ["docs/history/prune-queue-copy/CONTEXT.md", "packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs", "docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/worktree-parallelism/pruning-dead-worktrees.md"],
    "action": "Red first, per contract ad9d30dc. In worktree/tests.rs, beside `fn a_non_empty_capture_queue_keeps_even_though_the_tree_reads_clean() {`, add a test where the worktree queue holds the same stub and flush lines as the main checkout's `.bee/capture-queue.jsonl` (write both files in the fixture) and assert the verdict is dead; run it and watch it fail for the reported reason (kept for capture-queue.jsonl non-empty). Then change the queue half of `fn precious_state_present(worktree_root: &Path) -> Option<String> {` in prune.rs (pass the main root from the call `if let Some(reason) = precious_state_present(&worktree_root) {` in classify_worktree, which holds `check.main_root`): write a strict local reader in prune.rs (do not reuse the capture.rs or nudges readers: they skip bad lines, which breaks D2). Split on newlines, strip a trailing carriage return, and skip blank lines. In the worktree queue, a line that does not parse as a JSON object, or whose `kind` is neither `stub` nor `flush`, or a `stub` or `flush` whose `id` is not a non-empty string, keeps. Collect pending stub ids: stub ids with no `flush` line of the same id anywhere in that file (order does not matter). In main's queue, collect every string `id` on any line (no pending fold): a missing file counts as empty; a file that exists but cannot be read, or holds a line that does not parse, keeps. Compare ids as exact strings, never trimmed or case-folded. Keep only when a pending worktree stub id is absent from main's set, with a message naming the count of such stubs, one example id, and the remedy `run bee capture list and bee capture flush --id <id> --into <doc> inside <worktree path>`. Leave the HANDOFF.json half unchanged. More tests: a unique pending stub keeps and names its id; a stub pending in the worktree but flushed in main reads dead; main naming the id only in a flush line reads dead; a worktree flush of main's pending stub reads dead; a worktree flush of its own unique stub reads dead; a flush line before its stub counts as flushed; a trailing newline and a CRLF file do not keep; an unparseable line keeps; a numeric id keeps; a missing main queue with a pending worktree stub keeps; the existing two capture-queue tests still pass unchanged. Update row 4 of the concept table and the doc comment above the check to say what now counts as precious, and that this repo tracks the queue while host repos ignore it. No new code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee worktree::",
    "must_haves": {
      "truths": ["A worktree whose queue is a copy of main's is not kept for its queue", "A worktree whose queue holds a pending stub main never names is kept, and the reason names that stub", "Every read or parse doubt keeps the worktree"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs", "substantive": "content-aware queue keep with fail-closed doubt arms"}],
      "key_links": ["classify_worktree passes the main root to the precious-state check"],
      "prohibitions": ["No change to the HANDOFF.json keep", "No change to any other prune condition", "No existing test weakened", "No code comments"]
    },
    "trace": {"behavior_change": true}
  }
]
```

## Test matrix

| Cell | Happy | Edge | Error |
|---|---|---|---|
| pqc-1 | a copied queue reads dead. Pass when verdict.is_dead() | pending in worktree but flushed in main reads dead; a worktree-flushed unique stub reads dead; a missing main queue with a pending stub keeps | an unparseable line keeps; a line with no kind keeps (existing test unchanged); a unique pending stub keeps naming its id |

## Open Questions

(none)

## Live proof after merge

The leader runs `bee worktree prune --dry-run --older-than-days 0` on main and shows the per-worktree verdicts to the user before any real prune run.

## Out of scope

- A text summary line (removed, kept, bytes) for prune — a separate output change.
- Untracking the queue in this repo.
- Removing the existing worktrees — prune does that after this merges, after a shown dry-run.
