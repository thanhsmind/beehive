# claim-time-worktree-redirect — plan (v2)

**Lane:** standard · **Route flags:** covered-contract-change · **Files:** 7 product
**v1 (refusal-shaped) was rejected by plan-check** — P1-a narrow-door
deadlock, P1-b dispatch kill. v2 is annotation-shaped.

## Goal

The wrong flow (claim from main, then edit into a granted worktree, guard
deny, self-correct) meets its first machine signal AT CLAIM TIME: the claim
output itself names where execution must run. Claim semantics, the narrow
door, dispatch, and the guard all stay unchanged (D2, D5).

## Shape — 2 cells, one slice, file-disjoint → parallel

### C1 — code: claim output annotates the execution location (Rust + tests)

Files: `verbs/cells/handlers_write.rs`, `verbs/cells/handlers_select.rs`,
`verbs/cells/tests.rs` (all under `packages/bee-rs/crates/bee/src/`).

- Shared helper (cells module): given main store root + the claimed cell's
  `feature`, resolve the granted worktree via
  `crate::hooks::write_guard::find_feature_worktree_grant` (crate-visible
  through `write_guard/mod.rs:90`; three-state
  `Found/NotFound/Unresolvable`, `hook_local.rs:637-644`). `Found` →
  annotate; `NotFound`/`Unresolvable` → no annotation (fail open, D1).
- `cells claim` (`run_claim` / after `claim_cell_from_flags_ex` succeeds,
  `handlers_write.rs:602-646`): append one line to the emit text —
  `worktree: <root> — execution runs from a session rooted there; a
  subagent dispatched from main inherits main's cwd and cannot write
  here.` Add `worktree_root` to the emitted JSON object.
- `cells claim-next` (`run_claim_next`, `handlers_select.rs:658-843`):
  same line on its success text; `worktree_root` beside `ok/cell/claim`.
- No refusal path, no candidate_ok change, no NO_APPROVED_WORK change —
  existing claim assertions (cells/tests.rs:899-1180, tests/concurrency.rs)
  stay green because success text only gains a suffix line and refusal
  strings are untouched. Any test pinning the FULL success text exactly
  gets updated in the same cell.
- Tests (extend `wf_worktree_fixture`, `tests.rs:4644`): grant present →
  claim + claim-next carry the line + field; no grant → absent;
  unresolvable grant entry → absent (fail-open as a named case). No
  `set_current_dir` anywhere (D6).

### C2 — doctrine + prompt + knowledge sync

- `packages/bee/prompts/worker-cell.md`: first instruction — self-check
  effective cwd vs `{{worktree_root}}`; mismatch →
  `[BLOCKED: session cwd is not the worktree]`, zero edits (D3). The
  existing Location block stays.
- `skills/bee-swarming/SKILL.md` + `packages/bee/AGENTS.block.md`
  (+ `bee dev regen` → `AGENTS.md`): claim from main, then move the
  session into the worktree BEFORE dispatching execution workers
  (EnterWorktree / session rooted there / herding pane); a worker
  dispatched while cwd is main cannot write into the worktree (D4).
- `docs/knowledge/areas/worktree-parallelism/routing-and-visibility.md`:
  record claim-time annotation as the first signal in the redirect chain
  (claim line → worker self-check → write guard).

## Verify

`commands.test`: `cargo test --release --manifest-path
packages/bee-rs/Cargo.toml` at each `bee cells finish` (baseline green:
1868 passed). C2 additionally leaves the regen chain clean.

## Cost if wrong

Annotation is additive: worst case is a noisy extra line on claim output.
No semantics change anywhere; rollback is deleting the suffix.
