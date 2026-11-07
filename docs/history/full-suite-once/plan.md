# full-suite-once — Plan

**Lane:** standard · **Class:** feature · **Date:** 2026-11-07

## Problem

The owner rule (CONTEXT.md D1): the full declared suite runs once per
feature, at its end, before the merge to main. Nothing writes or checks it
today. The preamble prints the full suite as the standard command, planners
copy it into cell verify, doctrine says "run `commands.test` over what
changed", and a knowledge concept still says the doors run the suite.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Cell add already runs a verify-shape check, the home for D2 | read | packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:131 | `pub(crate) fn assert_verify_sentinel_allowed(root: &Path, verb: &str, verify: &Value) -> MR<()> {` |
| 2 | Cell update runs the same check on a verify patch | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs:568 | `assert_verify_sentinel_allowed(&root, "updateCell", verify)?;` |
| 3 | Merge reads proof and spawns nothing (test-doctrine D7) | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:302 | `// D7/D8 (docs/history/test-doctrine/CONTEXT.md, td-3): \`bee worktree` |
| 4 | The `bee test` record carries no commit today | read | packages/bee-rs/crates/bee/src/verbs/test_runner.rs:24 | `//   - Record: {ran_at, green, commands:[{command, exit, duration_ms,` |
| 5 | The preamble prints the full suite as the standard test command | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:716 | `lines.push("### Standard commands (host project)".to_string());` |
| 6 | Doctrine tells a leader to run the suite before ending work | read | packages/bee/LEADER.md:324 | `` `commands.test` over what changed when one is recorded, and name the `` |
| 7 | The cell show annotation says main runs verify at feature close | read | packages/bee-rs/crates/bee/src/verbs/cells/mod.rs:375 | `out.insert("verify_owner".into(), Value::String(VERIFY_OWNER_ANNOTATION.into()));` |

## Decisions

- D1 (dc19f610) — the owner rule.
- D2 (82a5bff7) — the cell plan check at `cells add` / `cells update`.
- D3 (feddf5c3) — the feature-end door is `bee worktree merge`, reading a
  fresh green `bee test` record; D7 holds.

## Contract — cell fso-1 (D2, cell plan check)

- A verify is "bare full suite" when one of its command segments (split on
  `&&`, `||`, `;`, newline; trimmed; leading `NAME=value` env assignments
  dropped on both sides) equals a declared `commands.test` command (string
  or each array entry, env assignments dropped the same way), or equals
  `bee test` / `.bee/bin/bee test` with no further argument. A segment that
  adds anything after the suite command (a `--test`, `--lib`, `-p`, a name
  filter, `--` args) is scoped and passes.
- `cells add` (each cell of a batch, reported with the other problems of
  the batch) and `cells update` (on a verify or `full_suite_reason` patch)
  refuse it with a typed message that starts `verify runs the full suite`,
  names the cell id, and ends with a `FIX:` line naming a scoped form built
  from the declared command (`<commands.test> --test <name>` or
  `<commands.test> <filter>`) and the escape `full_suite_reason`.
- `full_suite_reason` is a new optional plan field (string). Non-empty
  allows the bare suite; it is kept on the cell and shown by `cells show`.
  `cells update` accepts it as a plan field.
- A repo with no `commands.test` or the no-test sentinel never refuses.
- Tests first, seen red, in `tests/full_suite_verify.rs` (CLI-level, a temp
  onboarded repo): bare suite refused on add; the PATH-prefixed spelling
  refused; `bee test` refused; a `--test x` form passes; a reason passes and
  is kept; update of verify to the bare suite refused; no-test repo passes.

## Contract — cell fso-2 (D3, feature-end door)

- `bee test` adds `head` (the `git rev-parse HEAD` of the checkout it ran
  in, null outside git) to `.bee/logs/test-results.json`, in the store root
  it already writes when run from a worktree.
- `bee worktree merge` (zero-mutation precondition, beside the proof-debt
  check, before any main write): when the feature diff
  (`merge-base(main, worktree HEAD)..worktree HEAD`) changes any path that
  is not under `.bee/` or `docs/` and is not a `*.md` file, and
  `commands.test` is declared and is not the no-test sentinel, it reads the
  worktree's `bee test` record and refuses unless: `green` is true; its
  commands equal the declared `commands.test`; `head` is set; and every path
  changed between `head` and the worktree HEAD is bookkeeping (`.bee/`,
  `docs/`, `*.md`). Refusal codes: `WORKTREE_MERGE_FULL_SUITE_MISSING`,
  `WORKTREE_MERGE_FULL_SUITE_RED`, `WORKTREE_MERGE_FULL_SUITE_STALE`; each
  message ends with `FIX: run bee test in <worktree path>, then merge again
  (or pass --full-suite-waiver "<reason>")`.
- `--full-suite-waiver "<reason>"` (non-empty) skips the check and appends
  a `merge` audit row whose note starts `full-suite waiver:`.
- The door spawns no test. A docs-only diff and a no-test repo pass with no
  record.
- Help text via `bee dev regen`.
- Tests first, seen red, in `tests/merge_full_suite.rs`: code diff with no
  record refused (MISSING); red record refused; record at an older head with
  a later code commit refused (STALE); later docs-only commit passes; green
  fresh record merges; waiver merges and writes the audit row; docs-only
  feature merges with no record. Plus a unit test that `bee test` writes
  `head`.

## Contract — cell fso-3 (D1, doctrine, preamble, skills, knowledge)

- `packages/bee/LEADER.md` rule `agents-proof-at-cap` gains the owner rule
  in its own words: a cell proves itself with its related tests; the full
  declared suite runs once per feature, at its end, as `bee test` in the
  worktree before `bee worktree merge` (the door refuses without a fresh
  green record); a leader never re-runs a worker's recorded proof and never
  runs the full suite per cell; release.sh and CI keep their own full runs.
  Line 324 drops "run `commands.test` over what changed" and points at that
  rule. `.bee/LEADER.md` and any rendered copy refreshed the way the repo
  renders them; rule index unchanged unless an id moves.
- Preamble "Standard commands" (`session_preamble/budget.rs` and the
  compaction mirror in `hooks/compaction.rs`): the test line labels the
  command as the full suite, run once per feature with `bee test` before
  `bee worktree merge`, and says a cell verify adds a filter. Byte-pinned
  tests updated.
- `VERIFY_OWNER_ANNOTATION` (`verbs/cells/mod.rs`) and the `cells show`
  help: the verify is the cell's scoped proof that its worker runs; the full
  suite runs once per feature before merge.
- Skills: `skills/bee-swarming/references/worker-details.md:179`
  (no `bee test` re-run per cell — re-run the related tests),
  `swarming-reference.md`, `bee-hive/references/routing-and-contracts.md`,
  `bee-planning/references/planning-reference.md` (cell verify = a scoped
  form; `full_suite_reason` for a cross-cutting change), and any other
  skill text that tells an agent to run the full suite per cell or to
  re-run a worker's proof. Generated skill trees via `bee dev regen`.
- Knowledge: fix `docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md`
  lines ~21 (doors no longer run the suite: D7) and ~146 (CI runs on every
  push); put the feature-end door in
  `docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md`
  and the cell check in `docs/knowledge/areas/verify-pipeline/overview.md`,
  citing this plan's D-numbers. One fact, one home: the rule text lives in
  LEADER.md; the others point to it.

## Cells — current slice

Order: fso-1 and fso-2 in parallel (no shared file; fso-2 alone runs the
regen for help text), then fso-3 (it names fso-2's flag and owns the last
regen).

```json
[{"id":"fso-1","feature":"full-suite-once","lane":"standard","role":"code","title":"Refuse a cell whose verify is the bare full suite","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/verbs/cells/validate.rs","packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs","packages/bee-rs/crates/bee/src/verbs/cells/mod.rs","packages/bee-rs/crates/bee/src/verbs/cells/tests.rs","packages/bee-rs/crates/bee/tests/full_suite_verify.rs"],"read_first":["docs/history/full-suite-once/plan.md","docs/history/full-suite-once/CONTEXT.md"],"action":"Follow the Contract for fso-1 in docs/history/full-suite-once/plan.md: cells add and cells update refuse a verify that runs the bare declared full suite (or bare bee test) with a FIX line naming a scoped form, unless the cell carries a non-empty full_suite_reason. Write the behavior tests first and watch them fail.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test full_suite_verify && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::cells","must_haves":{"truths":["cells add refuses a verify that is the bare declared full suite, with a FIX line naming a scoped form","a verify with a test filter after the suite command is accepted","a non-empty full_suite_reason allows the bare suite and stays on the cell","cells update refuses a verify patch to the bare full suite","a repo with no commands.test or the no-test sentinel never refuses"]},"affects_skills":[],"affects_specs":[],"decisions":["dc19f610","82a5bff7"],"deps":[]},
{"id":"fso-2","feature":"full-suite-once","lane":"standard","role":"code","title":"Make worktree merge read a fresh green full-suite record","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/verbs/test_runner.rs","packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs","packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs","packages/bee-rs/crates/bee/src/verbs/worktree/mod.rs","packages/bee-rs/crates/bee/src/verbs/worktree/full_suite.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json","packages/bee-rs/crates/bee/tests/merge_full_suite.rs"],"read_first":["docs/history/full-suite-once/plan.md","docs/history/full-suite-once/CONTEXT.md","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],"action":"Follow the Contract for fso-2 in docs/history/full-suite-once/plan.md: bee test stamps head in its record; bee worktree merge refuses a code-touching feature without a fresh green bee test record from the worktree (MISSING/RED/STALE), accepts --full-suite-waiver with a merge audit row, passes docs-only diffs and no-test repos, and spawns no test. Write the behavior tests first and watch them fail.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test merge_full_suite && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::test_runner verbs::worktree","must_haves":{"truths":["bee test writes the checkout HEAD into its record","merge refuses a code-touching feature with no bee test record, naming bee test in the worktree","merge refuses a red record and a record whose head is older than a later code commit","a later docs-only commit keeps the record fresh","--full-suite-waiver merges and writes a merge audit row","a docs-only feature and a no-test repo merge with no record","the merge door runs no test itself"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],"decisions":["dc19f610","feddf5c3"],"deps":[]},
{"id":"fso-3","feature":"full-suite-once","lane":"standard","role":"code","title":"Write the once-per-feature suite rule into doctrine, preamble, skills and knowledge","behavior_change":true,"files":["packages/bee/LEADER.md",".bee/LEADER.md","packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs","packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs","packages/bee-rs/crates/bee/src/hooks/compaction.rs","packages/bee-rs/crates/bee/src/verbs/cells/mod.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json","skills/bee-swarming/references/worker-details.md","skills/bee-swarming/references/swarming-reference.md","skills/bee-hive/references/routing-and-contracts.md","skills/bee-planning/references/planning-reference.md","docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md","docs/knowledge/areas/verify-pipeline/overview.md","docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md","docs/history/codex-harness-hardening/release-manifest.json"],"read_first":["docs/history/full-suite-once/plan.md","docs/history/full-suite-once/CONTEXT.md"],"action":"Follow the Contract for fso-3 in docs/history/full-suite-once/plan.md: write the owner rule into the agents-proof-at-cap rule of packages/bee/LEADER.md (its one home), replace LEADER.md:324's suite line with a pointer, relabel the preamble and compaction Standard commands test line, fix the verify_owner annotation, fix the skill passages that order a per-cell full run or a re-run of a worker's proof, fix the stale knowledge concept and add the door and the cell check to their area concepts. Run bee dev regen for rendered trees.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test instruction_laws --test pointer_integrity --test registry_contracts --test agents_block_render_parity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --bin bee -- hooks::session_preamble hooks::compaction verbs::cells && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json","must_haves":{"truths":["the owner rule has one home in the agents-proof-at-cap rule of packages/bee/LEADER.md","LEADER.md no longer tells an agent to run commands.test before ending work","the preamble test line says the full suite runs once per feature before merge and a cell verify adds a filter","no skill tells an agent to run the full suite per cell or to re-run a worker's proof","the suite-topology concept no longer says doors run the suite or that the full run is daily"]},"affects_skills":["skills/bee-swarming/SKILL.md","skills/bee-hive/SKILL.md","skills/bee-planning/SKILL.md"],"affects_specs":["docs/knowledge/areas/verify-pipeline/suite-topology-and-discovery.md","docs/knowledge/areas/verify-pipeline/overview.md"],"decisions":["dc19f610","82a5bff7","feddf5c3"],"deps":["fso-1","fso-2"]}]
```
