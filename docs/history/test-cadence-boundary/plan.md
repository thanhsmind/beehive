# test-cadence-boundary — plan (rev 2, post review wave)

Locked context: docs/history/test-cadence-boundary/CONTEXT.md
(decision `13ce1858`, test-cadence-boundary D1/D1a/D1b).

## What changes

Today the one declared test command (`commands.test`) runs at four
doors: `bee cells finish`/`cells cap` (every cap), `bee close`,
`bee worktree merge`, CI. D1 removes the per-cap run and makes the
boundary the only local run: close when the feature has no worktree,
merge when it has one. CI unchanged.

## Canonical wording (every synced surface quotes this, verbatim)

> Tests prove at the boundary: `bee close` runs `commands.test` when
> the feature has no worktree; `bee worktree merge` runs it when it
> does. A cap is commit-only proof and records `tests: boundary`.
> CI runs the same command on every push.

## Mechanism (anchors verified by review wave, 2026-08-17)

- The per-cap run sits in `cap_cell_from_flags`
  (`cells/handlers_close.rs:139`), used by BOTH `cells finish` and
  `cells cap` (caller at `:576`, `finish=false`): run at `:195`
  (`run_declared_tests`, `cells/finish_support.rs:95`), red-refusal at
  `:198-245` (writes the `tests-red` attempt verdict at `:217`), trace
  write `tests: green|undeclared` at `:461-469`, cap line rendered at
  `:536-549`.
- `--report tests` validation forces `"green"|"red"`
  (`cells/finish_support.rs:265-272`) and lands verbatim on the trace
  (`handlers_close.rs:429-430`).
- Close re-runs fresh at `drivers/close.rs:1685` and stops on red
  (`:1705-1758`); dry-run door at `:1650-1662`; root comes from
  `resolve_store_root` (`:2446`) — unchanged.
- Merge runs the command itself (`worktree/handlers.rs:530`,
  `worktree/phases.rs:698`) and aborts on red (`phases.rs:403-439`).
  Unchanged by this feature.
- Worktree detection canon: `find_granted_worktree_for_feature`
  (`status_full/topology.rs:198`); reuse pattern
  `drivers/prepare.rs:575-591`.
- `bee test` verb unchanged; contract string at
  `test_runner.rs:113-114` reworded.
- `bee cells finish --help` text ships inside
  `generated/registry_payload.json` (compiled in via
  `registry.rs:13`) — generated; edit its source and regen.

## Shape — one slice, 4 cells, disjoint files

Parallel: tcb-1, tcb-2, tcb-3. Serial after 1+2: tcb-4 (generated
registry text depends on the settled behavior wording; its file is
also under a live reservation from the staging-lane feature — see
Contention).

- **tcb-1 (standard)** — the cap stops running tests.
  In `cap_cell_from_flags`: DELETE the run + red-refusal path (applies
  to both `cells finish` and `cells cap`; no `!finish` guard — the
  path dies for both). A cap in a declared-test repo records
  `trace.tests = "boundary"`; `"undeclared"` stays for the `none`
  sentinel; `trace.results`/`ran_at` no longer written at cap. Cap
  line prints `tests: boundary` via the existing formatter. The
  `tests-red` attempt-verdict WRITER (`:217` block) goes with the
  path; readers (`cells/validate.rs:458`, `drivers/prepare.rs:160`)
  stay for historical cells. Ripple inside the same files: `test_root`
  param of `cap_cell_from_flags` and `finish_topology`'s third return
  (`finish_support.rs:373-378`, consumed at `handlers_close.rs:617,630`)
  go dead — remove them, don't leave warnings. `--report tests`
  validation (`finish_support.rs:265-272`) now accepts exactly
  `"boundary"` (declared repo) or `"undeclared"` (no-test repo);
  `"green"`/`"red"` refuse with a teach line quoting the canonical
  wording — a worker's word never earns green (D1a). Remove the
  now-dead cells copy of `run_declared_tests` (`finish_support.rs:95`)
  if nothing else calls it. Rewrite affected tests in `cells/tests.rs`
  — the 7 known fns: `test_runner_green_and_red_record_shapes:1685`,
  `green_run_clears_a_stale_failure_log_from_a_previous_red:1725`,
  `verify_none_is_accepted_only_in_a_declared_no_test_repo:3343`,
  `capping_in_a_no_test_repo_runs_no_tests_but_a_declared_red_still_refuses:3386`,
  `finish_refuses_a_non_empty_files_cap_with_no_trailer_commit:3754`,
  `finish_caps_once_the_trailer_commit_exists:3775`,
  `finish_caps_at_main_and_runs_declared_tests_against_the_worktree_tree:4777`
  — plus any the compiler flags; assert no-run-at-cap + `boundary`
  marker + new report validation. Rewritten, never deleted without
  replacement.
  Files: `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`,
  `.../cells/finish_support.rs`, `.../cells/tests.rs`.
  read_first: CONTEXT.md; `handlers_close.rs:134-250,429-470,536-580`;
  `finish_support.rs:95-160,260-280,360-380`.

- **tcb-2 (standard)** — close defers to merge when a worktree exists.
  In `drivers/close.rs`: when `find_granted_worktree_for_feature`
  (topology.rs:198; copy the prepare.rs:575-591 reuse pattern) finds a
  granted worktree for the feature — INCLUDING one kept
  pending-cleanup after a merge — the tests door does not run the
  suite; non-blocking detail uses tense-neutral wording "tests prove
  at `bee worktree merge`". No worktree → run fresh exactly as today
  (same root from `resolve_store_root`, red still stops close, D1b) —
  do NOT relocate the run into any worktree. Reword the contract
  string at `test_runner.rs:113-114` (close/merge run it; the cap does
  not). Add/adjust tests in `drivers/tests.rs` (existing
  `close_red_stops_at_the_tests_door_and_exits_one:1894` stays green
  for the no-worktree branch; add the defer branch).
  Files: `.../drivers/close.rs`, `.../drivers/tests.rs`,
  `.../test_runner.rs`.
  read_first: CONTEXT.md; `close.rs:1640-1770,2440-2450`;
  `status_full/topology.rs:198-230`; `drivers/prepare.rs:575-591`;
  `test_runner.rs:105-120`.

- **tcb-3 (small)** — instruction-surface sync, one wording. Every
  site below states the canonical wording (quote or tight paraphrase;
  never a third idea). Anchors (line numbers approximate, verify on
  read): `AGENTS.md:84`; `skills/bee-swarming/SKILL.md:67,114`;
  `skills/bee-swarming/references/swarming-reference.md:25,60,218,240,379`;
  `skills/bee-swarming/references/worker-details.md:14-22`;
  `skills/bee-planning/SKILL.md:95-98`;
  `skills/bee-planning/references/planning-reference.md:141,201,204-205,214,317`;
  `skills/bee-hive/references/gates-and-delegation.md:111,160`;
  `skills/bee-hive/references/routing-and-contracts.md:158,173`;
  `skills/bee-shaping/references/mini-brief-template.md:28`;
  `skills/bee-shaping/references/implement-plan-template.md:118,151`;
  `docs/config-reference.md:138,157,165`;
  `docs/handbook/register.md:114,129-130,163,165-167` (register must
  document `trace.tests` value `"boundary"`);
  `docs/handbook/overview.md:130-132`;
  `docs/handbook/architecture-map.md:96,148`;
  `docs/handbook/stages/planning.md:73-74`;
  `docs/handbook/stages/executing.md:24,46`;
  `docs/handbook/stages/swarming.md:68`;
  `docs/codebase-overview.md:89-91`; `.bee/config-sample.json:6`;
  `docs/specs/test-simple.md:37`. Worker Result form docs: `tests`
  field value becomes `boundary`. Historical pattern files stay as
  written (records, not instructions). `docs/knowledge/` area sync
  rides scribing after execution. If a `.bee/` write is hook-denied,
  record the deny + remedy, never work around it.
  Files: the list above.

- **tcb-4 (small, deps tcb-1+tcb-2)** — regen the shipped help.
  Find the SOURCE of the `cells finish`/`cells cap` help text that
  renders into `generated/registry_payload.json` (compiled via
  `registry.rs:13`), reword to the canonical wording (cap records
  `tests: boundary`; no red-refusal at cap; boundary runs at
  close/merge), then run the repo's regen (`bee dev regen`) so the
  generated payload matches. Never hand-edit the generated file.
  Files: registry source + `packages/bee-rs/crates/bee/src/generated/registry_payload.json`.

## Contention

`generated/registry_payload.json` is under a live lease
(staging-lane, sl-1, holder `beehive--wt--staging-lane`). tcb-4 runs
last; if the lease still holds at dispatch time, tcb-4 defers with
reason "swallowed by sl-1's regen; re-triage after staging-lane
merges" — one report line, no waiting in silence.

## Risks / why this size

- `proof-weakening` is the feature itself — compensating controls:
  D1b (red refuses close/merge), merge is the only landing door, CI on
  every push.
- Named hole (accepted): a feature closed in a worktree that is never
  merged runs the suite nowhere locally; unmerged work never lands,
  and CI covers any push. Recorded here, not silently.
- Behavior existing tests assert changes on purpose: tests rewritten
  with the behavior, never deleted bare.
- Wrong-shape cost moderate: three disjoint file clusters + one
  generated file; one revert undoes it; no data, no external systems.
- Critical pattern `source-shipped-without-reinstalling-the-called-binary-is-inert`:
  after merge, rebuild + reinstall `.bee/bin/bee` before trusting the
  new cadence.

## Out of scope

CI, `bee test` verb behavior, selective-test tooling, dead legacy
`verify_output`/`verify_passed` trace slots (left as found).
