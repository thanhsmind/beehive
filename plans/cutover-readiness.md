# Cutover readiness — the two checks that would have lied, and one verdict

Companion to `plans/rust-port.md` (§ "Hard blockers for deleting the Node
runtime") and `plans/r5-test-migration.md`. It closes two of the four items
R5 filed there, and settles the one that had no plan entry anywhere.

**This file is a SNAPSHOT.** Section 3 reconciles the hard-blocker list as it
stood during this run (working tree on top of `34c028a7`, with three other
agents concurrently landing `hooks/cli_shape.rs`, `lease_store.rs`,
`workspace_store.rs`, and edits to `verbs/{cells,reservations,workflow_store,
worktree}.rs`). Several blockers moved WHILE this was being written. Re-read
the tree, not this list, before acting on it.

---

## 1. The two laws that would have gone vacuously green — CLOSED

### The defect

`test_instruction_size_law` and `test_scan_set_hygiene` scan `scripts/**` and
`packages/bee/**`. R6 deletes both trees. The laws would then pass because
their subject vanished, not because the law held.

This was not hypothetical. Reproduced on a fixture tree with the subject trees
absent, running `test_scan_set_hygiene.mjs` **exactly as it stands at
`34c028a7`**:

```
PASS test_scan_set_hygiene (check 1): 1 file(s) under scripts/tests/** and packages/bee/** derive no unguarded git-index scan set
PASS test_scan_set_hygiene (check 2): 0 current-behavior file(s) describe none of [validating] as current
>>> EXIT=0
```

A green tick, exit 0, over **zero files**. That is worse than no check: it
reads as coverage.

(`test_instruction_size_law` was already partly protected — its check 1
carried a bare `files.length > 10` — so it went red rather than green. But
red with `expected to scan a real tree of script files, found only 2` names
neither what it wanted nor where to go next.)

### The decision: RE-POINT, not retire

All three invariants survive the deletion, so none was retired:

| invariant | why it survives |
|---|---|
| Law A — no size ceiling on instruction text is ever a standing law here (budget-fence-removal D1, decision `8f63adb4`) | The doctrine is about the repo, not about Node. The tooling that could reintroduce a fence just moves to Rust. |
| Law B check 1 (E4) — no scan set derived from `git ls-files` is read without an existence guard | The subject is genuinely present in the Rust tree: `src/onboard/notices.rs::tracked_gitignore_paths` really does shell out to `git ls-files`. |
| Law B check 2 (E8) — no live file describes a retired workflow stage as current | Its scan set is shipped prose (`skills/**`, `docs/knowledge/**`, `AGENTS.md`, …) and is untouched by the deletion. Only its *token source* (`LEGACY_PHASE_COERCIONS` in `packages/bee/lib/state.mjs`) dies. |

**One invariant was deliberately NOT re-pointed**, and is recorded here rather
than dropped: invariant 2 of `test_instruction_size_law.mjs` — the pair that
proves `test_agents_budget.mjs`'s roster guard and byte-identical render guard
still bite. Its subject is *another Node suite*, not shipped prose, so porting
it means first choosing a home for `test_agents_budget.mjs` itself. That is
R6a instruction-layer work (`plans/r5-test-migration.md` § "The instruction-
layer gates are homeless, not covered"). Its Node original now fails loudly,
never vacuously, if its subject disappears. **Open item — see § 3.**

### What changed

**New — `packages/bee-rs/crates/bee/tests/instruction_laws.rs` (9 tests).**
The post-cutover home. Every scan goes through one `collect_scan_set` guard
that refuses an absent required root or a below-floor count, naming the label,
the roots walked, the expectation, the count, the floor, and this file.

| test | scans (today's count) |
|---|---|
| `no_size_ceiling_on_instruction_text_survives_in_any_shipped_tooling_tree` | `packages/bee-rs/crates/**/*.rs` (required, ≥40) **+** `scripts/**/*.{mjs,js,cjs}` (optional — R6 deletes it — ≥10 while present); union ≥40. **147 files** |
| `no_per_skill_byte_baseline_table_survives_in_any_shipped_tooling_tree` | `*.json` under `packages/bee-rs/crates` (required), `scripts` (optional), `skills` (required); union ≥1 |
| `no_unguarded_git_index_scan_set_survives_in_the_rust_tree` | `packages/bee-rs/crates/**/*.rs`, ≥40 |
| `no_current_behavior_file_describes_a_retired_stage_as_current` | `skills/**` (≥9), `expertise/**` (≥5), `docs/knowledge/**` (≥50), `docs/specs/**` (≥5), plus `AGENTS.md` and `CLAUDE.md` by name; union ≥200. **238 files** |
| `the_vacuity_guard_refuses_an_empty_scan_set` | the guard's own proof — three arms + a live-tree control |
| `law_a_detectors_bite_on_reintroductions_under_brand_new_names` | fixtures |
| `law_b_check1_bites_on_an_unguarded_derivation_and_spares_the_known_good_shapes` | fixtures |
| `law_b_check2_derivation_and_classifier_are_proven_on_fixtures` | fixtures |
| `the_self_exclusion_list_carries_only_files_that_quote_violations_as_fixtures` | the carve-out's own policing |

Four things in there are worth knowing before editing it:

1. **A second kind of vacuity is guarded too.** Check 1 asserts that at least
   one `git ls-files` invocation exists somewhere in the scanned tree. A law
   pointed at a tree where its defect class cannot occur is passing over an
   absent subject just as surely as one pointed at an empty directory. If that
   assertion ever fires, the honest response is to retire check 1 with a
   recorded reason, not to widen the scan.

2. **Check 1 was translated, not transliterated.** In Node the E4 crash shape
   is a bare `readFileSync` on a stale index path. In Rust `read_to_string`
   returns a `Result`, so the equivalent is a read that is `.unwrap()`ed or
   `.expect()`ed. Same defect class, Rust semantics. The known-good shape it
   must spare is real code — `tracked_gitignore_paths` derives a list and
   never reads from it.

3. **Law A's shape-A detector was NARROWED, deliberately.** The Node original
   flags any identifier naming both a size unit and a limiting concept,
   assigned a number. That worked because `scripts/**` happens to contain no
   size constants. Pointed at the Rust tree it produced four hits —
   `CONTENTION_TAIL_MAX_BYTES` (a log-tail read window),
   `DEFAULT_TAIL_MAX_BYTES`, `PRIOR_ROUNDS_MAX_EVENT_LINES` and
   `LEARNED_CONTEXT_MAX_LINES` (prompt-payload caps). None is a standing law
   on authored instruction text. So a hit now also requires evidence that its
   subject IS the instruction layer, in the file path, the identifier, or the
   declaration's own doc comment. This still catches what
   budget-fence-removal deleted (the fence lived in
   `scripts/skill_budget_fence.mjs`, its baseline in
   `scripts/skill-body-budget.json` — path evidence in both) and still catches
   a rename, because a fence on skill bodies cannot be written without naming
   skills in one of those three places. A control test pins the sparing, so
   the narrowing cannot silently widen into a hole.

4. **Check 2's token source is cross-checked, not just moved.** Tokens are
   derived from the Rust tree's own coercion records (`coerceLegacyPhase: 'X'
   -> 'Y'`, carried at each site) AND from Node's `LEGACY_PHASE_COERCIONS`
   while `state.mjs` exists — and the two must agree. So the Rust derivation
   is continuously proven correct *before* the day it becomes the only one.
   Zero tokens from both sources is a named failure, never a silent scan.

**Changed — `scripts/tests/test_scan_set_hygiene.mjs`.** Scan sets unchanged.
Added `scanSetVacuity()` with floors (check 1 ≥25, check 2 ≥40 — well under
today's 149 and 239, so they catch "the subject vanished", not "someone
deleted a file"). Check 1's git-backed derivation is split out
(`unguardedScanSetCandidates`) so the selftest can drive the check with a set
of its own choosing. New `runSelftestVacuity()` proves both checks refuse an
empty set AND that neither fires on the live tree.

**Changed — `scripts/tests/test_instruction_size_law.mjs`.** Scan sets
unchanged. Added `requireNonEmptyScanSet()` covering both arms (absent root,
below-floor count) and two negative controls that prove each arm bites.

### The proof

**A/B on a fixture tree with the subject trees deleted** (`git show HEAD:` for
the before, working copy for the after):

```
###### BEFORE — 34c028a7 ######
PASS test_scan_set_hygiene (check 1): 1 file(s) ...
PASS test_scan_set_hygiene (check 2): 0 current-behavior file(s) ...
>>> EXIT=0

###### AFTER ######
FAIL test_scan_set_hygiene --selftest: check1's vacuity guard fires on the LIVE tree — either the
floor is set too high, or (the case this guard exists for) the subject tree has actually been
deleted and this check now needs re-pointing or retiring:
      test_scan_set_hygiene (check 1 — unguarded git-index scan set): SCAN SET IS EMPTY OR
      IMPLAUSIBLY SMALL — 1 file(s) matched [scripts/tests/*.mjs, packages/bee/**/*.mjs], expected
      at least 25 (the Node suite tree and the Node runtime library — the only code in this repo
      that shells out to `git ls-files` to build a scan set it then reads from). This check asserts
      nothing over that set, so it is failing rather than reporting a vacuous PASS. If the subject
      tree was deliberately removed (e.g. the R6 Node-runtime cutover), re-point this check at the
      surface that replaced it or retire it explicitly — see plans/cutover-readiness.md.
>>> EXIT=1
```

`test_instruction_size_law.mjs`, same fixture: `found only 2` became
`invariant 1 — ceiling-shaped constructs: scanned 2 file(s) under [scripts],
expected at least 10. Expected the repo's Node script tree
(scripts/**/*.{mjs,js,cjs}). ... re-point it or retire it`.

**Rust, live sabotage** — the required root temporarily re-pointed at
`packages/bee-rs/crates-DELETED`, i.e. the exact shape the cutover produces:
all four scanning tests failed, each naming the missing root and its own
expectation, e.g.

```
LAW B check 1 — unguarded git-index scan set: SCAN ROOT MISSING —
`packages/bee-rs/crates-DELETED` does not exist. Expected the Rust source tree
(packages/bee-rs/crates/**/*.rs) — the tree that shells out to `git ls-files` after the Node
runtime is deleted (src/onboard/notices.rs does so today). A law whose scan root is gone must be
re-pointed at the surface that replaced it, or retired with a recorded reason — never left to pass
over nothing. See plans/cutover-readiness.md.
```

The sabotage was reverted; the same nine tests are green.

**Permanent, not a probe.** The proof lives in the suites, not in this
document: `the_vacuity_guard_refuses_an_empty_scan_set` (Rust, three arms plus
a live-tree control), `runSelftestVacuity()` (Node, both checks), and the two
`negative control: the vacuity guard refuses …` checks
(`test_instruction_size_law.mjs`).

**Suite state.** `cargo build --release && cargo test --release` on a clean
`34c028a7` snapshot with only these three files applied: **739 passed, 0
failed, 3 ignored** — the 730-test baseline intact (698 unit + 13 concurrency
+ 4 front_door + 10 hook_contracts + 5 registry_contracts) plus 9 new. The
live working tree has 4 unrelated failures in `hooks/cli_shape.rs` and
`verbs/cells.rs`, both files owned by other agents' in-flight work; no file
touched here appears in them. Node: `test_instruction_size_law.mjs` 10/10,
`test_scan_set_hygiene.mjs` exit 0 (check 1 over 149 files, check 2 over 239).

---

## 2. `scribingTarget` — verdict: **DEAD as runtime surface. Do not port.**

Deletion is the owner's call at cutover; nothing was removed. The evidence.

### Where it lives

`export function scribingTarget(root, { area, subject = null, intent = 'auto' } = {})`
— `packages/bee/lib/knowledge.mjs:937` (docblock at `:890`, body to `:1057`),
byte-identical in the vendored mirror `.bee/bin/lib/knowledge.mjs:937`. A pure
resolver: no writes, returns seven keys
(`bundle_mode, action, area, subject, path, owner, regenerate_index`).

### Why DEAD

1. **Zero runtime callers, repo-wide.** The only `import` of it anywhere is
   `packages/bee/tests/test_bundle_mode.mjs:35`. There is no caller inside
   `knowledge.mjs` either — its other three hits are the docblock and two
   error strings.
2. **Not imported by the entry point.** `packages/bee/bee.mjs:260-269` names
   eight knowledge exports; `scribingTarget` is not among them. Same in the
   vendored `.bee/bin/bee.mjs`.
3. **Not in the command registry.** `command-registry.mjs:1771-1854` registers
   `knowledge.check/index/list/context/promote` only.
4. **Not on the CLI surface.** `--help --all --json` (123 commands, identical
   from `bee.exe` and `bee.mjs`) contains zero matches for `scribingTarget`,
   `scribing-target`, `bundle mode` or `bundle_mode`. The only `scribing`-named
   verb is `state scribing-run`, which is unrelated (it stamps
   `last_scribing_run` and advances phase).
5. **No state or config surface.** No `scribing_target` key in any `.bee/`
   file, config schema, or `docs/knowledge/` frontmatter; the seven-key return
   is never persisted (`"bundle_mode"` has zero hits under `.bee/`).
6. **Its only invocation path was deleted, twice, and blocked in between.**
   It was never a verb — it was invoked by agents typing
   `node -e "import('.bee/bin/lib/knowledge.mjs')…"`, prescribed in
   `bee-scribing`'s SKILL.md. The internals-reach write guard then denied that
   shape (`docs/knowledge/areas/hook-runtime/internals-reach-bash-guard.md:81-99`
   — "there is no CLI verb yet that exposes them"). `b1ddb008` (2026-07-28)
   demoted the call from SKILL.md to a reference file; `93b95d2b` (2026-07-31,
   the nine-skill consolidation) deleted every remaining mention across
   `skills/`, `.claude/`, `.agents/`, `.claude-plugin/`, `.codex-plugin/`. The
   successor skill `bee-capturing` references neither `scribingTarget` nor
   `bundleMode`.
7. **The backlog item that would revive it invalidates itself.** PBI
   `p-0530164c` (`proposed`) asks for `bee knowledge scribing-target`, and
   `docs/backlog.md:19` already reads: *"its successor `bee-capturing`'s
   SKILL.md no longer carries the inline calls, so re-verify the gap before
   building."* Plus a P3 friction row at `.bee/backlog.jsonl:538`.
8. **No source change since 2026-07-22.** Last commit touching the body:
   `a073eb4d`. Everything after is renames (`8bff2a8d`), doc deletions
   (`b1ddb008`, `93b95d2b`), and index re-renders (`cf30bdc5`).
9. **No Rust port, no plan entry** — zero hits under `packages/bee-rs/`.

### What argued LIVE, and why it does not carry

- Two decisions from 2026-07-22 (`5c6e88d4`, `d712c6e6`) mandate
  `resolveProductRoot` semantics for it. They are about *fixing* the function,
  not about it being reachable, and both predate the removal of its call path.
- Cell `f3-4` protected it (*"do not touch scribingTarget or the anti-fork
  gate — f3-3 owns them"*) — a 2026-07-22 concurrency boundary, not a
  liveness claim.
- 28 test cases exercise it. Tests are not callers.

### The one thing that IS load-bearing, and is not the code

Two live current-behavior documents still name `scribingTarget` as the
mechanism that ENFORCES a rule:

- `docs/handbook/stages/scribing.md:34-35` — *"One area = one file/concept,
  forever — never fork (anti-fork gate via `bundleMode`/`scribingTarget`)."*
- `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md:308-310` —
  lists it as a live implementation pointer.

So the anti-fork rule reads as machine-enforced while no code path enforces
it. That is true **today**, before any deletion — the enforcement stopped when
the invocation path was removed on 2026-07-31, not when the function is
eventually deleted. Deleting the function without settling this converts a
rule that looks enforced into prose without anyone noticing. Settling it is
either (a) demote the docs to state that the anti-fork rule is a discipline,
not a gate, or (b) re-implement it somewhere that actually runs — a
`bee knowledge target` verb in Rust, which is what PBI `p-0530164c` asked for.
**That choice is the owner's, and it is the real decision hiding behind
"port it or delete it".**

### What deleting entails (for the owner, at cutover)

- `packages/bee/lib/knowledge.mjs`: the docblock (`:890`) and body
  (`:937-1057`). Its private helper `subjectSlug` (`:879`) goes with it — it
  has no other caller. **`normalizeSubject` (`:872`), `foldEncoding` (`:854`)
  and `CONFUSABLE_FOLD` (`:841`) must STAY**: `normalizeSubject` is also used
  at `:680` by the authority-claim grouping, which is live.
- `.bee/bin/lib/knowledge.mjs` — generated mirror, re-vendored, not hand-edited.
- **Tests that exist only to test dead code:** 28 of the 49 `check(...)` cases
  in `packages/bee/tests/test_bundle_mode.mjs` (1037 L). The file itself
  SURVIVES — 21 cases do not touch it: 7 `bundleMode` predicate checks
  (incl. the live-checkout pin), 1 divorced-topology check, 3 instruction-layer
  checks, and the 10 `buildSessionPreamble` checks that occupy roughly lines
  761-1037. This is a partial deletion, not a file deletion. No other test file
  references it.
- Docs to demote or re-point: `docs/handbook/stages/scribing.md:34-35`,
  `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md:308-310`,
  `docs/knowledge/areas/hook-runtime/internals-reach-bash-guard.md:81-99`,
  `docs/specs/reading-map.md:126`, `docs/knowledge/log.md:520`.
- Backlog to close or re-scope: `p-0530164c`, `.bee/backlog.jsonl:538`.
- Rust: nothing. There is nothing to delete and nothing to port.

`plans/rust-port.md` § "Hard blockers" and `plans/r5-test-migration.md`
§ 2 should both be updated to point here instead of asking the question again.

---

## 3. What still blocks deleting `bee.mjs` — snapshot

Reconciled against `plans/rust-port.md` § "Hard blockers for deleting the Node
runtime". **Moving target**: three agents were landing work in this tree
during this run, so several rows changed under observation. Evidence is what
the tree said at that moment.

### Closed

| blocker | evidence at this run |
|---|---|
| **Two laws go vacuously green** | § 1 above. Guarded in Node, re-pointed into Rust, refusal proven in both. |
| **`scribingTarget` has no plan entry** | § 2 above. Settled as DEAD; the owner's remaining choice is about the anti-fork DOC, not about porting code. |
| **write-guard check (d), the CLI-shape schema guard** | Was a pinned delegation. Now native: new `src/hooks/cli_shape.rs`; `write_guard.rs:23` reads *"CHECK (d) IS NATIVE (R6 blocker closed)"*, wired at `:4220`. **Caveat: this is the source of 3 of the 4 live-tree test failures** (`no_shipped_command_spelling_is_refused_by_the_widened_guard`) — the widened guard is currently refusing prose in shipped docs that merely looks like a `bee` invocation. Landed but not yet green. |
| **`listWorkflows` unreadable-entry tolerance** | Was a pinned delegation. `workflow_store.rs:64` now reads *"list_workflows REPRODUCES the skip warn natively"*; the deterministic skip line is at `:277`. |

### Landed but not yet wired (as of this run)

The seven "no Rust implementation at all" contracts have implementations now,
in a new `src/lease_store.rs` and in `workflow_store.rs`/`cells.rs` — but the
release build still warns `never used` for `create_workflow`,
`generate_workflow_id`, `NewWorkflow::for_feature`, `renew_leases_by_session`,
`list_all_lease_files`, `sweep_expired_leases`, `list_leases`. Implemented is
not the same as reachable; someone must confirm the verbs route to them.

| contract | where it is now |
|---|---|
| `createWorkflow` | `verbs/workflow_store.rs` (14 hits) — unwired |
| `renewLease` / `LEASE_MISSING` | new `src/lease_store.rs` — partly unwired |
| `LEASE_FENCE_STALE` | `lease_store.rs`, `verbs/reservations.rs`, `verbs/worktree.rs` |
| `CLAIM_FENCE_STALE` / `renewClaimTTL` | `verbs/cells.rs` (9 hits) |
| `adoptClaim` | `verbs/cells.rs`, `verbs/state_group.rs` — **3 of the 4 live-tree failures are its new tests** (`adopt_agrees_with_the_state_group_port_on_the_shared_fixture`, `adopt_rewrites_ownership_in_place_and_bumps_the_fence_by_exactly_one`, `an_adoption_fences_out_the_previous_holders_later_writes`) |
| multi-resource batch acquire | `lease_store.rs:363` `acquireLeases`, with hash-sorted-order and rollback tests |

### Still open

| blocker | status |
|---|---|
| **Homoglyph / NFKC folding of an authority subject** | UNCHANGED. `verbs/knowledge.rs:971` is still `normalize_subject_ascii`, and `:1140` still carries `// non-ASCII → delegate`. The last of the seven. |
| **`test_agents_budget.mjs` has no post-cutover home** | New, recorded here (§ 1). Its two meaning guards — the doctrine-section roster and the byte-identical `AGENTS.md` render — assert things about shipped prose that survives the deletion, but they are written as a Node suite invoked by another Node suite. R6a work: pick a home (a `bee dev` subcommand, a CI step, or an integration test alongside `instruction_laws.rs`) and move both. Until then, deleting `scripts/tests/` deletes the only check that `AGENTS.md` matches its template byte-for-byte. |
| **`encodeProjectDir` drive colon** | UNCHANGED, and by design — `rust-port.md` schedules the three-site fix for R6 itself, when there is no Node left to match. |
| **Coverage debts** (`rust-port.md` § "Coverage debts R6 must close") | Not re-audited here. `state start-feature`'s write-policy half now has a `verbs/workspace_store.rs` in the tree (untracked, in flight); everything else in that section was out of scope for this run. |

### Recommended order

1. Get the live tree green — the 4 failures above are in landed work, not in
   anything listed as blocked.
2. Confirm the lease/workflow/claim implementations are actually routed to,
   not merely present (the `never used` warnings are the tell).
3. Close homoglyph folding — the last of the seven.
4. Choose `test_agents_budget.mjs`'s home, as part of R6a, before `scripts/`
   is deleted rather than after.
5. Settle the anti-fork doc question (§ 2), then delete `scribingTarget`.
