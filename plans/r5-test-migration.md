# R5 — Node test-suite migration to Rust

Companion to `plans/rust-port.md` (phase R5). The port itself is essentially
complete: nearly every verb, hook and dev tool serves natively. What still
blocks deleting the Node runtime is **coverage** — ~37k lines of `.mjs` tests
encode contracts the Rust suite does not assert. This file is the audit of
that gap: every Node test file classified, what was ported, and what was
deliberately left.

Verdicts:

- **COVERED** — the file's contracts are already asserted by existing Rust
  tests. Nothing to do; the citation is the evidence.
- **GAP** — real contracts with no Rust assertion. This is the work.
- **NODE-ONLY** — the file tests machinery that dies with the runtime (ESM
  loading, `.mjs` shim wiring, the Node dispatcher's own plumbing, the
  `.bee/bin/lib` `.mjs` mirror, `run_verify.mjs`'s suite inventory). No port.
- **BLOCKED** — a GAP whose subject verb is still delegated to Node. The test
  cannot be ported until the verb is; each entry names the blocker.

## Counts

| verdict | files |
|---|---|
| COVERED | 9 |
| GAP (portable today) | 47 |
| GAP (blocked on an unported verb) | 18 |
| NODE-ONLY | 15 |
| **total** | **89** |

`race_claims_child.mjs`, `race_decisions_child.mjs` and `race_lease_child.mjs`
are counted as NODE-ONLY harness children; the invariants they prove are
tracked as a single shared gap (see "Concurrency" below).

## Test-count delta

Verified on the live tree, `cargo build --release && cargo test --release`,
three consecutive runs, all green:

| | unit (`--bin bee`) | `front_door` | `concurrency` | `hook_contracts` | `registry_contracts` | total |
|---|---|---|---|---|---|---|
| before | 604 | 4 | — | — | — | **608** |
| after | 698 | 4 | 13 | 10 | 5 | **730** |

**+122 tests**, all 608 originals still passing. Three additional tests are
`#[ignore]`d by construction: they are child-process harness bodies that their
parent tests invoke by name (see § Two mechanics the ports needed). They are
not skipped coverage.

Env-limited skips that fired on this host, each naming its capability:
the four companion-mount rows in `write_guard.rs` and the encoded-transcript-
layout row in `status_full.rs` (symlink privilege and the NTFS colon defect
respectively). Everything else ran.

## Priority order used

Risk, not file size. The guard suites first — they encode safety decisions and
a wrong verdict is a data-loss incident, not a cosmetic drift — then the
process-boundary contracts, then the rest.

---

## Classification

### Guards and hooks (highest risk)

| Node file | verdict | where its coverage lives |
|---|---|---|
| `hooks/test_write_guard.mjs` (1791 L) | GAP → **ported** | `src/hooks/write_guard.rs` — 63 pre-existing tests cover rows 1–70; R5 added rows 72–77 (the `isSharedNestedCheckoutTarget` exclusion arms) and the D8/D9/D12 tokenizer matrix. See § Ported. |
| `hooks/test_model_guard.mjs` (1127 L) | GAP → **ported** | `src/hooks/model_guard.rs` — 11 pre-existing tests cover the marker/param/tier tables; R5 added the fail-open arms (rows 11/12/15/16 + presence gate). |
| `hooks/test_hook_contracts.mjs` (4366 L) | GAP → **ported** (wrapper table) | new `crates/bee/tests/hook_contracts.rs` — the adversarial-input matrix, the Codex advisory-shape rows, and the apply_patch rows, black-box over the shipped binary. The catalog-drift / shim-binary / installed-route halves are NODE-ONLY (they diff `.codex/hooks.json` against `catalog.mjs`'s `.mjs` command spellings). |
| `tests/test_guards.mjs` (1918 L) | COVERED | `src/hooks/write_guard.rs` maps ~1:1 (NET branches 1–7, AskUserQuestion, the gc-2 git branch, msn-21 ownership, xwh-4 cross-worktree). Its four `buildSessionPreamble` checks are part of the preamble gap below. |
| `tests/test_guards_tokenizer.mjs` (171 L) | GAP → **ported** | `src/hooks/write_guard.rs::{d9_every_separator_form_splits_glued_and_spaced_alike, d9_separator_lookalikes_are_not_boundaries, d8_staging_a_cli_owned_file_is_not_a_direct_edit_target, d12_companion_marker_is_direct_edit_denied}`. Its tokenizer-equivalence corpus is NODE-ONLY: it exists to catch drift between two hand-synced `.mjs` copies, and Rust has one tokenizer. |
| `tests/test_bee_write_guard_hook.mjs` (691 L) | GAP → **ported** | Guard bodies covered by `write_guard.rs`. Check (d), the CLI-shape schema guard, is now NATIVE in `hooks/cli_shape.rs` — its § (a) rows are mirrored there and wired-path rows live in `write_guard.rs`. |
| `hooks/test_bypass_stop_net.mjs` (221 L) | GAP (thin) | Fire/no-fire matrix + loop guard covered by `src/hooks/session_close.rs::{bypass_net_blocks_planning_once_then_steps_aside, bypass_net_high_risk_consult_sentence_and_mode_floor}`. The "PreCompact never blocks" contract is blocked: session-close's PreCompact branch delegates. |

### CLI / dispatcher

| Node file | verdict | where its coverage lives |
|---|---|---|
| `tests/test_bee_cli.mjs` (5764 L) | GAP — see § Not ported | Largest single file. Its value is split three ways: dispatcher error/validate/nearest-match machinery (NODE-ONLY by campaign rule 1 — a verb serves only argv shapes proven equivalent, everything else returns before output), verb behavior (covered per-verb in the modules below), and the `RECOVERY_LAYOUT_UNREPRESENTABLE` skip, which exists solely for the `encodeProjectDir` drive-colon defect and is scheduled for unpinning at R6. |
| `scripts/tests/test_conformance.mjs` (815 L) | GAP | Several scenarios have unit-level Rust analogs; the four `doctor` scenarios have none — `doctor` is unported entirely. |
| `tests/test_cli_state.mjs` (3422 L) | GAP (largely covered) | `src/verbs/state_group.rs` (34 tests) + `workflow_store.rs`. The `state start-feature` block is BLOCKED. |
| `tests/test_cli_cells.mjs` (1474 L) | GAP | `src/verbs/cells.rs`, `test_runner.rs`, `backlog.rs`. `--queue-submit` is BLOCKED. |
| `tests/test_herding_cli.mjs` (273 L) | BLOCKED | `herding` is absent from `router.rs::PORTED`. |

### Verb modules

| Node file | verdict | where its coverage lives |
|---|---|---|
| `tests/test_decisions_propagation.mjs` (1731 L) | COVERED | `src/verbs/decisions.rs` (24 tests) |
| `tests/test_state_projection.mjs` (770 L) | COVERED | `src/verbs/workflow_store.rs` (22) + `src/hooks/state_sync.rs` (8) |
| `tests/test_backlog_capture.mjs` (1066 L) | COVERED | `src/verbs/backlog.rs` (17) + `capture.rs` (4) + `drivers.rs` |
| `tests/test_reservations.mjs` (423 L) | COVERED | `src/verbs/reservations.rs` (23) + `src/roots.rs` (15) |
| `scripts/tests/test_dispatch_prepare.mjs` (1178 L) | COVERED | `src/verbs/drivers.rs` (~20 tests) |
| `scripts/tests/test_worktree_grant_resolve.mjs` (102 L) | COVERED | `src/roots.rs` over real `git worktree add` fixtures |
| `scripts/tests/test_skill_render.mjs` (318 L) | COVERED | `src/onboard/render.rs` (9) + `src/devtools/skill_trees.rs` (7) |
| `scripts/tests/test_impact_registry.mjs` (332 L) | COVERED | `src/devtools/impact_registry.rs` (14). Three narrow `run()` exit-code legs unasserted. |
| `tests/test_state.mjs` (1148 L) | GAP | `src/roots.rs`, `state.rs`, `workflow_store.rs`. `startFeature` BLOCKED. |
| `tests/test_cells.mjs` (2514 L) | GAP → **ported (partial)** | `src/verbs/cells.rs` — see § Ported |
| `tests/test_claims.mjs` (858 L) | GAP → **ported (partial)** | `src/verbs/cells.rs` — see § Ported |
| `tests/test_knowledge.mjs` (1497 L) | GAP → **ported (partial)** | `src/verbs/knowledge.rs` — see § Ported |
| `tests/test_recovery.mjs` (790 L) | GAP → **ported (partial)** | `src/verbs/status_full.rs` — see § Ported |
| `tests/test_contention_status.mjs` (200 L) | GAP → **ported** | `src/verbs/status_full.rs` — see § Ported |
| `scripts/tests/test_config_validate.mjs` (541 L) | GAP → **ported** | `src/verbs/status_full.rs` — see § Ported |
| `scripts/tests/test_ship_visibility.mjs` (157 L) | GAP → **ported (partial)** | `src/verbs/status_full.rs`. The session-preamble line is unported behavior, not just an untested one. |
| `tests/test_scratch.mjs` (572 L) | GAP → **ported** | `src/verbs/tmp_group.rs` — see § Ported |
| `tests/test_intent.mjs` (409 L) | GAP → **ported** | `src/verbs/intent_group.rs` — see § Ported |
| `tests/test_reviews.mjs` (1105 L) | GAP | `src/verbs/reviews.rs` (5 tests) covers the read side; the `createReview` / `recordOnReview` write paths and the git-degradation arms are unasserted. |
| `tests/test_feedback.mjs` (1134 L) | GAP (mostly BLOCKED) | Local arm covered by `src/verbs/feedback.rs` (10). The whole `mergeDigests` foreign/dogfood arm is unported by design (`feedback.rs:1135` delegates when `dogfood_repos` is non-empty). |
| `tests/test_bundle_mode.mjs` (1037 L) | GAP | `scribingTarget` has **no Rust port at all** — the single largest un-planned gap. See § Not ported. |
| `tests/test_misc.mjs` (3752 L) | GAP (mixed, ~60% covered) | Spread across `decisions.rs`, `prompt_context.rs`, `onboard/notices.rs`, `drivers.rs`, `cells.rs`, `reservations.rs`. Its `buildSessionPreamble` block (~20 contracts) is BLOCKED. |
| `tests/test_perf.mjs` (390 L) | GAP | `src/hooks/session_close.rs` has the pipeline and 2 tests; ~10 contracts unasserted. |
| `tests/test_workflow_store.mjs` (439 L) | GAP (thin) | `src/verbs/workflow_store.rs`. `listWorkflows`' unreadable-entry tolerance is now NATIVE for the three ordinary skip reasons; only the V8-parse and libuv-errno wordings delegate. |
| `tests/test_lease_store.mjs` (457 L) | GAP | Only the write half is ported, inline in `reservations.rs`. Renew/fence/rollback/deterministic-order are unported. |
| `tests/test_msn_invariants.mjs` (653 L) | GAP (half NODE-ONLY) | The source-marker index mechanism dies; inv 5/6/10 are the concurrency gap. |
| `tests/test_write_policy.mjs` (291 L) | GAP (partly BLOCKED) | `observe` and `shared-disjoint` are in `cells.rs`/`write_guard.rs` with no test; `isolated` is blocked on workspace-store. |
| `tests/test_config_validate.mjs` (165 L) | BLOCKED | `bee config set/unset` is absent from `PORTED`. |
| `tests/test_worktree_store.mjs` (158 L), `tests/test_worktree_store_merge.mjs` (371 L), `tests/test_workspace_store.mjs` (366 L), `tests/test_integration_queue.mjs` (414 L) | BLOCKED | `worktree new\|merge`, workspace-store, integration queue |
| `tests/test_herding.mjs` (216 L) | GAP (out of runtime scope) | Targets shell scripts under `skills/bee-herding/` that survive the deletion. |
| `scripts/test_onboard_bee.mjs` (4987 L) | GAP (broad, shallow residue) | ~85 Rust tests across `src/onboard/*` cover the spine; §10f–§10l fail-closed arms (symlink, ancestor overlap, realpath identity, host_helpers ladder) are unasserted. |
| `scripts/test_plugin_distribution.mjs` (423 L) | GAP | `plugin_distribution.mjs` is entirely unported. |
| `scripts/test_split_brain_regression.mjs` (245 L) | GAP (thin) | Invariant (B) covered by `onboard/source.rs`; invariant (A) — the live-tree structural guard — has no Rust equivalent. |

### Concurrency (one shared gap)

`test_claim_race`, `test_reservation_race`, `test_store_lock`,
`test_state_write_concurrency`, `test_state_projection_race`,
`test_render_race`, `test_worktree_holds_race`, and the three
`race_*_child.mjs` harness children all prove one class of contract: the
O_EXCL mutual-exclusion invariants **under real OS-process interleaving**.
The Rust tree has no concurrency test at all — every primitive is covered
single-process only. See § Not ported.

### NODE-ONLY (15)

`test_check_filter`, `test_hook_vendor_closure`, `test_ledger_parity`,
`test_lib_mirror`, `test_run_verify_impacted`, `test_verify_cache`,
`test_verify_manifest`, `test_verify_timeout`, `test_workflow_step_paths`,
`race_claims_child`, `race_decisions_child`, `race_lease_child`, plus the
catalog/shim/installed-route halves of `test_hook_contracts`, the tokenizer
-equivalence corpus of `test_guards_tokenizer`, and the vendored-`.mjs`
byte-identity blocks inside `test_misc`.

Each dies for the same reason: its subject is the `.mjs` payload, the ESM
import graph, or the Node suite runner. `run_verify.mjs`'s three suites
(`cache`, `manifest`, `timeout`) are replaced by `cargo test`, which owns
caching, inventory and per-test timeouts natively.

---

## Ported in R5

### Guards — `src/hooks/write_guard.rs`

| new test | ports | why it matters |
|---|---|---|
| `row72_71_plain_nested_checkout_flags_only_when_concurrent` | rows 71/72 | the primitive's positive and no-op arms, called directly (the wired rows go through the hook and never reach the exclusions) |
| `row73_registered_submodule_is_never_flagged` | row 73 | a `.gitmodules`-registered submodule is excluded even when concurrent — with two controls (no registration → flagged; a registration naming a *different* path → still flagged) so the exclusion cannot pass vacuously |
| `rows74_77_verified_companion_mount_exclusions` | rows 74–77 | verified-mount solo/concurrent, marker mismatch, unmarked symlink. **Env-limited**: probes real symlink creation and skips loudly per row naming Developer Mode / an elevated shell — it skips on this host |
| `d9_every_separator_form_splits_glued_and_spaced_alike` | `test_guards_tokenizer.mjs` D9 | all five separator forms (`;` `&&` `&` `\|` `\|\|`) glued AND spaced. The pre-existing test asserted only two |
| `d9_separator_lookalikes_are_not_boundaries` | D9 follow-up | `2>&1`/`1>&2`, a quoted separator, a backslash-escaped separator |
| `d8_staging_a_cli_owned_file_is_not_a_direct_edit_target` | D8 | with the control that a real mutation of the same file IS a target |
| `d12_companion_marker_is_direct_edit_denied` | D12 | `.bee/companion-session.json` joins `DIRECT_EDIT_DENY` |

Node's row-73 fixture spends a real `git submodule add`; the Rust fixture writes
the two artifacts the primitive actually reads (`.gitmodules` + a nested `.git`).
Same code path, no git dependency — noted in the test.

### Guards — `src/hooks/model_guard.rs`

| new test | ports |
|---|---|
| `rows11_16_unparseable_and_non_object_stdin_fail_open` | rows 11/15/16 — junk stdin, empty stdin, top-level `null`, top-level array, a non-string `cwd`, and an array carrying a dispatch-shaped element |
| `row12_no_repo_root_is_silent_success` | row 12, with the control that the same payload inside a real fixture denies |
| `missing_vendored_lib_is_silent_success` | the presence gate, plus "a gated-off run leaves no dispatch telemetry" |

### Hook process contract — new `crates/bee/tests/hook_contracts.rs` (10 tests)

Black-box over the shipped binary, ported from the wrapper table of
`test_hook_contracts.mjs`:

- `every_hook_fails_open_on_adversarial_stdin` — 9 hooks × 7 rows (empty, junk,
  `null`, `[]`, object `cwd`, missing `cwd`, embedded NULs)
- `every_hook_survives_a_two_megabyte_payload`
- `advisory_events_never_block_a_turn` — PreCompact/SubagentStop/Stop must be
  silent or a parseable JSON object with a string `systemMessage`, never
  `decision:"block"`
- `chain_nudge_advisory_names_the_returning_worker` — the payload's
  `agent_name` reaches the rendered message, and no placeholder survives
- `apply_patch_targeting_state_json_is_denied_by_the_write_guard` (+ the safe-target control)
- `apply_patch_is_ignored_by_the_model_guard`
- `a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr`
- `a_model_guard_deny_reaches_the_host_as_exit_two_on_stderr`
- `an_unknown_hook_name_is_a_named_refusal_not_a_crash`
- `no_bee_root_is_silent_success_for_every_hook`

Two mechanics were needed to make this file mean anything:

1. **The delegation tripwire.** Every row runs with `BEE_HOOK_NO_DELEGATE=1`.
   Without it, a hook that delegated to Node would answer correctly and the row
   would pass while testing nothing. Rows that delegate print a loud `SKIP`
   naming the hook, and each matrix asserts that enough rows stayed native —
   a suite where everything delegated must never report green. Measured today:
   only `session-init` (all rows) and `session-close`'s PreCompact branch
   delegate; both are documented delegate classes in `plans/rust-port.md`.
2. **The vendored-lib fixture.** Most hooks gate on `.bee/bin/lib/state.mjs`
   existing, and the write guard byte-compares the whole 26-file closure. A
   fixture without it makes every hook exit 0 silently — the entire matrix
   would have been a vacuous pass. The fixture vendors the repo's own copy,
   the same `copyLib()` discipline the `.mjs` suites use.

### Registry — new `crates/bee/tests/registry_contracts.rs` (5 tests)

The schema half of `test_bee_cli.mjs`, asserted over
`src/generated/registry_payload.json` — the bytes the binary ships:

- `every_entry_declares_a_valid_json_schema_parameters_object` — including
  "every `required` field is declared in `properties`" (a required field with
  no property is a validator that can never be satisfied)
- `every_entry_carries_a_description_an_invoke_and_at_least_one_example` —
  and no `invoke` may still name a `.mjs` script (the R6a spelling rule)
- `every_example_is_spelled_against_a_declared_command` — longest-prefix match,
  so `bee state gate` is not satisfied by `bee state`
- `command_names_and_invokes_are_unique`
- `the_payload_declares_the_schema_version_the_binary_was_built_against`

Together with the pre-existing `front_door.rs::embedded_registry_payload_is_fresh`
(which pins the payload against the live `command-registry.mjs`), a malformed
OR stale registry can no longer reach a release.

### Small modules

| new test | file | ports |
|---|---|---|
| `ship_visibility_passes_the_two_known_values_and_normalizes_the_rest` | `src/state.rs` | `test_ship_visibility.mjs` — the normalizer had no test at all |
| `every_atomic_write_picks_a_distinct_tmp_name` | `src/fsutil.rs` | the in-process half of `test_state_write_concurrency.mjs`: 512 tmp names, all distinct, all siblings of the target |
| `a_failed_atomic_write_leaves_no_tmp_and_no_partial_target` | `src/fsutil.rs` | `writeJsonAtomic`'s failed-rename arm |

### Verb modules

| file | before → after | ports |
|---|---|---|
| `src/verbs/status_full.rs` | 17 → 36 | **recovery** (`test_recovery.mjs`): the clean-end-trio truth table, the bounded transcript window, last-durable-settlement across three stores with lane scoping, the five-rung crash-candidate exclusion ladder each paired with its firing control, both work-signal arms, the fast path, and the `scan_transcript_roots` config arms. **contention** (`test_contention_status.mjs`): the whole summary, malformed-line skipping, and a tail-window test *stronger than the oracle* — a well-formed record placed before the 64 KB boundary, so a full-file scan would report the wrong count. **config validation** (`test_config_validate.mjs`): every problem code, every writable-advice token paired with a clean control, and the point of the cell end-to-end — a malformed cli tier is loud, not silently reverted. **ship_visibility** rendering. |
| `src/verbs/cells.rs` | 43 → 51 | `test_claims.mjs` / `test_cells.mjs`: the sweep's gate-held / gate-free / fresh-heartbeat trio with no gate-file leak, the `releaseClaim` NOT_OWNER→NOT_FOUND ladder, the D5 sessionless-adoption rule (zero / one / two live sessions), `resolveSessionId` precedence, `addCells` batch aggregation and cycle folding, the `verify:"none"` matrix, and capping in a no-test repo. |
| `src/verbs/knowledge.rs` | 18 → 34 | `test_knowledge.mjs`: the four profile warnings, the three profile errors (including `duplicate_authoritative_for` over the *hardened* subject), the `--strict` flip, `not_canonical` round-trip warnings, the four relevance-cut invariants, and `index --check`. |
| `src/verbs/reviews.rs` | 7 → 23 | `test_reviews.mjs`: the whole `createReview` / `recordOnReview` write path (SPEC §8 round-trip, A6 auto-exclude, immutable scope fields, SET-vs-APPEND kinds, refusal ordering), `candidate add --mode`, corrupt-entry degradation, and the five `deriveCandidateStatus` git arms. |
| `src/verbs/tmp_group.rs` | 5 → 10 | `test_scratch.mjs`: the targetless-invocation refusal, dry-run parity, `bytes_freed`/`files_freed` against a manual walk, `--all`, deliverable survival under every flag combination, and the closed-record sweep. |
| `src/verbs/intent_group.rs` | 5 → 10 | `test_intent.mjs`: D1 idempotent set / typed refusal / `--force`, `advance()` touching only `next_action`, the anchorless refusal, `clear()` idempotence, and the absent/half-written read shapes. |
| `src/verbs/workflow_store.rs` | 24 → 25* | `test_workflow_store.mjs`: absent-store listing, the C4 lock-through property, two-id non-blocking. |
| `src/verbs/reservations.rs` | 24 → 27 | `test_lease_store.mjs`: TTL ≤ 0 never-expires across sweep + ledger + listing, zero residue on a lost reserve, malformed request refused before any file exists. |

\* one of the three requested `workflow_store` contracts had nothing to test —
see § Not ported.

### Concurrency — new `crates/bee/tests/concurrency.rs` (13 tests)

The Rust tree had no concurrency test at all. All five Node race suites landed,
black-box over the shipped binary with genuinely concurrent OS processes:

- `test_claim_race` → one winner, N−1 typed `CLAIMED` refusals naming the
  owner session and expiry
- `test_reservation_race` → N distinct paths all survive; one shared path
  yields one winner and typed conflicts naming the holder
- `test_store_lock` → serialized mutators with no lost update, stale-lock
  takeover, and a typed busy naming a live holder
- `test_state_write_concurrency` → a reader never observes a torn `state.json`
- `test_worktree_holds_race` → every mirrored hold survives

Four **deliberate-RED controls** are included: the same race with the exclusion
removed must fail the same assertion. On win32 these are a net gain over the
oracle, which env-skips its controls entirely (`WIN32_UNGUARDED_RENAME`).

No defect was found — every invariant held. Stability: 5 consecutive runs, one
single-threaded run, and three simultaneous copies of the binary (~150
concurrent `bee` processes) all green, plus the three full-suite runs above.

### Two mechanics the ports needed

Both are worth knowing before adding to these suites:

1. **Delegation tripwires, two shapes.** Hooks have a built-in one
   (`BEE_HOOK_NO_DELEGATE`, hooks/mod.rs). Verbs do not, so `concurrency.rs`
   uses the sabotage shape instead: `BEE_JS_ENTRY` points at a nonexistent
   file, and `js_fallback.rs` treats a set-but-wrong entry as a hard 127. A
   dedicated test (`the_delegation_tripwire_actually_bites`) proves the
   detector fires — without it every probe would be vacuous.
2. **Child-process harnesses for cwd- and env-bound code.** `try_native`
   reaches the repo through `current_dir()`, and several claims paths read
   `BEE_SESSION_ID` live. Mutating either in-process would race every other
   test in the shared binary, so those cases re-invoke the test binary as its
   own child (`current_exe()` + `--exact <child> --ignored`). This also avoids
   a real trap: `target/release/bee.exe` is NOT rebuilt by
   `cargo test --bin bee`, so driving it would silently test stale bytes.

One honesty fix while passing through: `devtools/skill_trees.rs::
a_symlink_in_the_source_refuses_before_any_output` returned silently when the
host could not create a symlink. A silent return reads as coverage the suite
does not have; it now prints a skip line naming the capability. Same for
`tmp_group.rs::symlinked_root_is_refused_wholesale`.

---

## Deliberately NOT ported

### 1. The instruction-layer gates are homeless, not covered

Seven Node files assert things about `skills/**/*.md`, `README.md` and
`AGENTS.md` that survive the runtime deletion untouched:
`test_always_loaded_rules`, `test_bypass_matrix`, `test_doctrine_parity`,
`test_gate_bypass_doctrine`, `test_skill_pointers`, `test_instruction_size_law`,
and CHECK 2 of `test_scan_set_hygiene`.

They are not NODE-ONLY — they happen to be written in Node, but their subject
is shipped prose. They were not ported because a Rust `#[test]` walking the
docs tree is a *new home*, not a migration, and choosing that home (a `bee dev`
subcommand? a CI step? an integration test?) is a design decision that belongs
with the R6a instruction-layer work, not with a test sweep.

**A specific hazard worth naming:** `test_instruction_size_law` and
`test_scan_set_hygiene` scan `scripts/**` and `packages/bee/**`. After the
deletion those scans go *vacuously green* rather than red — a law that passes
because its subject was removed. Re-point them or retire them explicitly.

### 2. Contracts blocked on an unported verb

Porting these would mean testing Node through the delegate, which proves
nothing about the Rust path. Each is listed against its blocker in
`plans/rust-port.md` § "Coverage debts R6 must close":

| blocker | tests stranded |
|---|---|
| `buildSessionPreamble` (session-init delegates wholesale) | ~20 in `test_misc`, 4 in `test_cli_state`, 4 in `test_guards`, ~10 in `test_bundle_mode`, the draft-PR line in `test_ship_visibility`, all of `test_compact_capsule` |
| `state start-feature` (write-policy + workspace-store half) | ~17 in `test_cli_state`, ~10 in `test_state` |
| `worktree new\|merge` | `test_worktree_cli`, `test_worktree_companion`, `test_worktree_merge_queue`, `test_worktree_store_merge`, `test_integration_queue`, 3 blocks in `test_misc` |
| `state compact-*` | `test_compact_verbs`, the `appendCompactionRecord`/`compactCheck` half of `test_compaction_module` |
| `session-close`'s PreCompact branch | the PreCompact rows in `test_compaction_advisories` and `test_bypass_stop_net` |
| `feedback collect` dogfood arm | ~20 in `test_feedback` |
| `backlog add --queue-submit` | 3 in `test_cli_cells` |
| `cells *` inside a linked worktree | 3 topology checks |
| `doctor` (never ported, no plan entry) | 4 scenarios in `test_conformance`, the unlock row in `test_native_probe` |
| `bee config set/unset` | `tests/test_config_validate.mjs` |
| `herding` | `test_herding_cli` |
| `scribingTarget` (never ported, **no plan entry**) | ~35 in `test_bundle_mode` |

`scribingTarget` deserves its own line: it is the largest un-planned gap found
in this audit. It lives only in `packages/bee/lib/knowledge.mjs` and has no
runtime caller outside that module and its test — so before porting it, confirm
it is live surface and not dead code.

### 3. Structural delegations that blocked deletion — CLOSED 2026-08-01

Two behaviors were pinned in Rust as *deliberate delegations*, meaning the
contract had no native owner at all once `bee.mjs` is gone. Both are now
implemented natively; see `plans/rust-port.md` § "Hard blockers" for the full
account and the byte-diff evidence.

- **write-guard check (d), the CLI-shape schema guard** → new
  `crates/bee/src/hooks/cli_shape.rs` (registry resolution + validate-args +
  the exact refusal bytes), wired from `write_guard.rs`. `bee_cli_shapes_delegate`
  is replaced by `row5_5b_plain_bee_cli_invocations_still_pass`,
  `rows5c_5d_a_malformed_bee_cli_call_is_denied_at_exit_two`,
  `a_well_formed_bee_cli_call_reaches_the_ordinary_verdict`,
  `check_d_never_overwrites_a_denial_an_earlier_check_computed` and
  `a_tampered_registry_still_delegates_before_check_d_can_answer`, plus 24 unit
  rows in `cli_shape.rs`. Nothing here delegates any more; the byte gate still
  does, one layer up.
- **`listWorkflows` unreadable-entry tolerance** → `workflow_store.rs` warns
  natively for the three ordinary skips.
  `list_workflows_delegates_when_any_entry_would_be_skipped` is replaced by
  `list_workflows_skips_the_three_ordinary_shapes_and_keeps_the_readable_ones`,
  `the_warn_line_is_console_warns_own_shape`,
  `only_the_two_v8_worded_arms_still_delegate` and
  `a_delegating_scan_emits_no_warn_before_it_bails`. Residue: the V8-parse-message
  arm and the libuv-errno arm, named by the third of those tests.

### 4. Contracts with no Rust implementation at all

Found while porting — these are not test debt, they are *missing code*. Each
was left as a named comment block in the Rust test module rather than faked:

| contract | where it should live | evidence |
|---|---|---|
| `createWorkflow` / `readWorkflow` | `verbs/workflow_store.rs` | `rg createWorkflow` finds only two FIX-hint strings. Record creation is still Node's. |
| `renewLease` / `renewLeasesBySession` / `LEASE_MISSING` | `verbs/reservations.rs` | never renews. A narrowed renew exists as `renew_lease_path` in `hooks/state_sync.rs` and `hooks/prompt_context.rs`. |
| `LEASE_FENCE_STALE` (renew and release) | `verbs/reservations.rs` | zero hits for `fence`/`presentedEpoch`; `reserve_locked` stamps `epoch: 0` and nothing ever compares it. |
| `CLAIM_FENCE_STALE` / `renewClaimTTL` (invariant 10) | `verbs/cells.rs` | `claim_cell_file` stamps `fence_epoch: 1`; nothing consumes it. |
| `adoptClaim` | `verbs/cells.rs` | not in this port's claims subset. |
| multi-resource batch acquire (partial rollback, hash-sorted order, `LEASE_INVALID_REQUEST`) | `verbs/reservations.rs` | there is no batch — `reserve_locked` acquires exactly one resource. The single-resource echoes ARE ported. |
| homoglyph/NFKC folding of an authority subject | `verbs/knowledge.rs` | `normalize_subject_ascii` models only the ASCII slice; a non-ASCII claim delegates. The delegation is now pinned. |

### 5. Contracts reachable only from a CLI-level harness

`resetCellBudget`'s audit ordering, `claimNextCell`'s lane fall-through and
`NO_APPROVED_WORK`, `addCells`' non-array/empty refusals, and the
archive pre-flight collision scan all live inside `dispatch(...)` closures that
print and return an `ExitCode` rather than returning data, and resolve their
root from the process cwd. The engines underneath are covered; the closures
need a `tests/`-level harness (the child-process shape `tmp_group.rs` and
`cells.rs` now use would work).

### 6. Companion-mount write behavior

`test_write_guard.mjs` rows 65/66/69/82 assert that a write inside a *verified*
companion mount is allowed. In Rust that whole class delegates: a companion
target fails containment, and a present marker returns `Nd`. The primitive
underneath (`is_shared_nested_checkout_target`) IS ported and is now tested
(rows 72–77), but the wired allow-path is Node's. Likewise rows 83/84 — a
detection error denies fail-closed in Node, while Rust returns `Nd` and
delegates. Both are behavior debts for R6, not test debts, and porting a test
for them today would assert Node's behavior through the delegate.
