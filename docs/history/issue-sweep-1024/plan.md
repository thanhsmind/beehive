---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: issue-sweep-1024

## Summary

Nine bug reports are open on GitHub. Each one stops real work: a guard that
refuses a safe command, a merge that cannot finish, a check that rejects a
correct cell, a worker that keeps running after bee said it failed. This sweep
fixes each one to the behavior its issue asks for, with a test that fails
first, and closes the issue with the fix commit.

Issue #69 (a cross-project dashboard) is a feature request. It needs product
decisions first, so it is not in this sweep.

Mode: `high-risk` — 4 risk flags: audit-security (two guards get narrower
refusals), public-contracts (registry schema, JSON outcome), multi-domain,
external-systems (paseo spawn).
Why this is the least workflow that protects the work: the guard changes widen
what passes a safety net, so they get the high-risk gate and an advisor check;
the other fixes ride the same feature because they share the release.

## Requirements

Locked decision `8f114141-d74d-463f-b4a6-529c0db32761` (contract
`contract:issue-sweep-1024`):

- D1 #131: the idle intake gate lets `git fetch` and a push of `main` through.
- D2 #135: staging or committing `.bee/config.json` in a granted feature
  worktree is not refused as a content write.
- D3 #132: `bee worktree merge` auto-commits the skills version stamp files
  (`.bee-skills-version.json` under each skills root) on main instead of
  refusing.
- D4 #130: the CLI-shape guard accepts `cells cap` with `--report-file` in
  place of `--report`.
- D5 #134: an escalated cell under a v2 role plan dispatches through the model
  guard, and `cells escalate --off` clears the flag.
- D6 #133: `cells leader-check` counts a directory in `files_changed` as
  touched when a commit touched a path under it.
- D7 #129: a `herding run` that times out on paseo spawn but finds the created
  agent does not report `spawn_failed` and never invites a second agent on the
  same cell.
- D8 #128: state writes and reads from a granted worktree resolve the same
  record.
- D9 #136: `bee doctor` names a configured design role with no `bee-design`
  skill installed, and the `--design-packs` remedy.
- No safety refusal is removed beyond the exact command shape each issue names.

## Load-bearing claims

Labels: `read` = file opened at that line; `ran` = command run this session.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The idle intake gate message lives in the write guard paths module | ran | `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:592` | `"bee intake gate: no bee work is active (phase: {}) — {} is blocked. {}{}{}",` |
| 2 | The config guard refusal lives in the write guard main module | ran | `rg -n "cannot be modified via the shell" packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs` | one match in `hooks/write_guard/main.rs` |
| 3 | The main-dirty merge refusal is decided in the worktree merge module | ran | `packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:249` | `// main (WORKTREE_MERGE_MAIN_DIRTY, phases.rs), but the dirt is routinely` |
| 4 | The CLI-shape guard is one module | ran | `rg -ln "does not match .*schema" packages/bee-rs/crates/bee/src/hooks` | `hooks/cli_shape.rs` |
| 5 | escalate knows an `off` key but its handler is in handlers_close | ran | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:2027-2028` | `pub(crate) fn run_escalate(flags: rsv::Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {` / `if !rsv::keys_known(&flags, &["id", "reason", "off"]) {` |
| 6 | The planned-role check lives in the model guard | ran | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:811` | `format!("bee-model-guard: planned_role_mismatch: could not read cell \"{cell_id}\""),` |
| 7 | The leader-check overlap message lives in leader_check | ran | `packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:316` | `"{VERB}: cell \"{id}\" files_changed names path(s) the cell's commit(s) never touched: {}.",` |
| 8 | The spawn-failed remedy is built in herding run | ran | `packages/bee-rs/crates/bee/src/herding/run.rs:1911` | `fn spawn_failed_remedy(cell_id: Option<&str>, agent: Option<&str>, pane_id: Option<&str>) -> String {` |
| 9 | Two state path helpers exist, both rooted at the given root | ran | `packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:38-39` | `pub(crate) fn state_path(root: &Path) -> PathBuf {` / `root.join(".bee").join("state.json")` |
| 10 | Doctor builds its rows in one function | ran | `packages/bee-rs/crates/bee/src/doctor.rs:146` | `fn mechanical_rows(root: &Path, runtime: Runtime) -> Vec<Row> {` |
| 11 | The cap refusal of #130 reproduces on 2.61.0 | ran | `bee cells cap --id dpi-2 ... --report-file <path>` (this session) | `does not match cells.cap's schema — required, missing (--report) (field: report)` |

## Discovery

Each issue names its command and refusal text; the anchors above place each
refusal in one module. Three issues share the write guard (#131, #135) or the
registry (#130, #134), so those pairs are grouped or ordered. #128 and #129 are
marked unverified in their reports; their cells start by reproducing.

## Approach

One cell per issue group, each red first: a test that reproduces the issue's
exact command and refusal, then the smallest fix. Every guard change narrows a
refusal to the exact shape the issue names and keeps a test that the nearby
dangerous shape is still refused.

Rejected alternatives:

- *Turn guards off (`guards.idle_gate` false).* Removes the net for every
  command, not the two safe ones.
- *Fix the issues one feature each.* Nine gates and nine merges for fixes that
  ship together.

Smaller path check: is there a cheaper shape that still honours D1–D9? Fewer
cells would put unrelated modules in one worker's hands and serialise them;
more cells would split the shared write guard and registry files. Eight is the
floor that keeps files disjoint.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Intake gate and config guard | HIGH — widens what passes a safety net | iss-1 | red-first test per issue; a test that `git push` of a non-main ref and a shell write to config.json stay refused |
| Merge auto-commit of the stamp | MEDIUM — writes on main | iss-2 | test that only `.bee-skills-version.json` under a skills root is auto-committed |
| CLI-shape schema | LOW | iss-3 | test `cells cap --report-file` passes, no report at all still refused |
| Escalate under v2 | MEDIUM | iss-4 | test payload passes the model guard; `--off` clears |
| Leader-check directory | LOW | iss-5 | test directory entry passes, unrelated directory refused |
| Spawn timeout | MEDIUM — external process | iss-6 | test with a fake transport that times out but lists the agent |
| Worktree state record | MEDIUM | iss-7 | test write then read from a granted worktree hit one record |
| Doctor design row | LOW | iss-8 | test row present with role and no skill, absent with skill |

Waves: wave 1 runs iss-1, iss-2, iss-3, iss-5, iss-6, iss-7, iss-8 in parallel;
iss-4 waits on iss-3 because both edit the command registry and the flag pin.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "eight Rust fixes with their tests"},
    {"stage": "tests", "classification": "not-applicable", "role": "test", "reason": "each code cell writes its own red-first tests"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "anchors located at plan time, claims 1-11"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers it"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "advisor", "classification": "required", "role": "advisor", "reason": "high-risk gate consult on the guard changes"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "a knowledge concept names a changed refusal", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "the leader tends one wave"},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "no rendered UI"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "one advisor consult covers the high-risk gate; issues carry their own facts"},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "covered by the advisor consult"},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "each issue states its value"},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "smaller-path check inline in Approach"},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "each issue states its impact"},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "as above"},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "as above"}
  ]
}
```

## Shape

One slice, eight cells, two waves.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| iss-1 | Let git fetch, a push of main, and config.json staging through the guards | `hooks/write_guard/*` | — | `git fetch` on idle main and `git add .bee/config.json` in a worktree run | `cargo test ... write_guard::` — green:unit |
| iss-2 | Auto-commit the skills version stamps in worktree merge | `verbs/worktree/merge.rs`, `phases.rs`, `tests.rs` | — | merge after an upgrade no longer deadlocks | `cargo test ... skills_version_stamp` — green:unit |
| iss-3 | Accept cells cap --report-file in the CLI-shape guard | `hooks/cli_shape.rs`, registry, `catalog.rs` | — | `cells cap --report-file` is no longer refused before bee runs | `cargo test ... report_file` — green:unit |
| iss-4 | Dispatch escalated cells under v2 plans and let escalate --off clear | `handlers_close.rs`, `drivers/prepare.rs`, `hooks/model_guard.rs`, registry, `catalog.rs` | iss-3 | an escalated cell dispatches; `--off` clears it | `cargo test ... escalate_v2` — green:unit |
| iss-5 | Count a directory in files_changed as touched in leader-check | `verbs/cells/leader_check.rs` | — | leader-check passes a cell capped with a directory | `cargo test ... leader_check_directory` — green:unit |
| iss-6 | Report a paseo spawn timeout with a live agent as running | `herding/run.rs` | — | no `spawn_failed` while the agent runs | `cargo test ... spawn_timeout` — green:unit |
| iss-7 | Resolve one state record for writes and reads in a granted worktree | `state.rs`, `verbs/state_group/store.rs` | — | `bee gate` in a worktree changes what `bee close` there reads | `cargo test ... worktree_state_record` — green:unit |
| iss-8 | Name a design role with no design skill in bee doctor | `doctor.rs` | — | doctor prints the gap and `--design-packs` | `cargo test ... doctor_design_skill` — green:unit |

```json
[
  {
    "id": "iss-1",
    "feature": "issue-sweep-1024",
    "title": "Let git fetch, a push of main, and config.json staging through the guards",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issues #131 and #135 (read them with `gh issue view 131` and `gh issue view 135`). Red first; name EVERY new test with `issue_131` or `issue_135`. #131: with phase idle in the main checkout, the intake gate (hooks/write_guard/paths.rs:592) refuses `git fetch` and `git push origin main`. Make `git fetch` (any remote, any flags) read-only-eligible, and make `git push <remote> main` (and `HEAD:main`) bookkeeping-eligible on main. Keep refused: a push of any other ref, `--force`/`-f`/`--mirror`/`--delete`, and every other unmodelled git subcommand. #135: in a granted feature worktree, `git add .bee/config.json` and `git commit` that carries it are refused by the config guard (hooks/write_guard/main.rs, `cannot be modified via the shell`). Staging and committing are not content writes: let `git add`/`git commit` naming .bee/config.json through, while a shell write that changes the file's bytes (redirect, sed -i, tee, cp, mv over it) stays refused. Tests must pin both the newly allowed shapes and the still-refused neighbours. Do not change any other guard. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml issue_13",
    "must_haves": {
      "truths": [
        "git fetch on an idle main checkout is not refused by the intake gate",
        "git push origin main on an idle main checkout is not refused; a force push or a push of another ref still is",
        "git add .bee/config.json in a granted worktree is not refused; a shell redirect into .bee/config.json still is"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs", "substantive": "fetch and push-main modelled; no TODO stubs"}
      ],
      "key_links": ["each allowed shape has a still-refused neighbour test"],
      "prohibitions": ["No guard turned off", "No refusal widened beyond the issue's command shapes", "No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-2",
    "feature": "issue-sweep-1024",
    "title": "Auto-commit the skills version stamps in worktree merge",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #132 (`gh issue view 132`). Red first; name EVERY new test with `skills_version_stamp`. After a bee upgrade, `<root>/.claude/skills/.bee-skills-version.json`, `.agents/skills/.bee-skills-version.json` and `.opencode/skills/.bee-skills-version.json` are modified in main, and `bee worktree merge` refuses WORKTREE_MERGE_MAIN_DIRTY while the worktree-first guard refuses committing them by hand. Add exactly those stamp files (the file name `.bee-skills-version.json` directly under a known skills root) to the bookkeeping the merge auto-commits on main, the same way it already auto-commits .bee/ and docs/ bookkeeping (merge.rs around line 249). Any other dirty path under a skills root still refuses. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml skills_version_stamp",
    "must_haves": {
      "truths": [
        "a merge with only modified .bee-skills-version.json files in main auto-commits them and proceeds",
        "a merge with another modified file under a skills root still refuses WORKTREE_MERGE_MAIN_DIRTY"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs", "substantive": "stamp files in the auto-commit set"}],
      "key_links": ["the stamp rule names the exact file name under a skills root"],
      "prohibitions": ["No other skills file auto-committed", "No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-3",
    "feature": "issue-sweep-1024",
    "title": "Accept cells cap --report-file in the CLI-shape guard",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #130 (`gh issue view 130`). Red first; name EVERY new test with `report_file`. `bee cells cap --id X --files a,b --report-file <path>` is refused by the PreToolUse CLI-shape guard with `required, missing (--report)`, though the handler accepts --report-file (D12, #102) and `cells finish` with the same flags passes. Make the guard accept cells cap when --report-file is given instead of --report: either declare report-file in the cells.cap registry entry and express the one-of requirement the way cells.finish already does, or teach cli_shape the alternative — follow whichever cells.finish uses so the two verbs agree. A cap with neither --report nor --report-file stays refused. If a registry flag changes, update the PINNED_FLAG_COUNT pin in catalog.rs. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml report_file",
    "must_haves": {
      "truths": [
        "the CLI-shape guard passes cells cap with --report-file and no --report",
        "the CLI-shape guard still refuses cells cap with neither"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs", "substantive": "one-of report or report-file honoured"}],
      "key_links": ["cells cap and cells finish resolve the report requirement the same way"],
      "prohibitions": ["No edit to any existing assertion other than a flag-count pin", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-4",
    "feature": "issue-sweep-1024",
    "title": "Dispatch escalated cells under v2 plans and let escalate --off clear",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["iss-3"],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #134 (`gh issue view 134`). Red first; name EVERY new test with `escalate_v2`, and put them inline in the files this cell owns (not verbs/cells/tests.rs or verbs/drivers/tests.rs, which other work may touch). Two defects: (1) under an approved bee-plan/v2 role plan, an escalated cell gets a `[bee-tier: ceiling]` payload from dispatch prepare, and the model guard refuses it with planned_role_mismatch (model_guard.rs ~811) because only the planned role is accepted. An escalated cell must dispatch: either the guard accepts the escalation marker for a cell whose record says escalated, or prepare emits a payload the guard accepts — pick the one that keeps the planned-role check strict for non-escalated cells. (2) `bee cells escalate --id X --off` is refused as unsupported argument shape, and `--off true` leaves escalate true and re-runs the budget check. Make --off a boolean flag (registry entry for cells.escalate; update the PINNED_FLAG_COUNT pin in catalog.rs only if the distinct flag-name set changes) and make run_escalate (handlers_close.rs:2027) clear the escalation without the budget check. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml escalate_v2",
    "must_haves": {
      "truths": [
        "an escalated cell under a v2 role plan gets a payload the model guard accepts",
        "a non-escalated cell declaring a role other than its planned role is still refused",
        "bee cells escalate --id X --off clears the escalation without a budget check"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "off path clears escalate"}],
      "key_links": ["the guard and prepare agree on the escalated payload"],
      "prohibitions": ["No weakening of the planned-role check for non-escalated cells", "No edit to any existing assertion other than a flag-count pin", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-5",
    "feature": "issue-sweep-1024",
    "title": "Count a directory in files_changed as touched in leader-check",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #133 (`gh issue view 133`). Red first; name EVERY new test with `leader_check_directory` and put them inline in leader_check.rs. `bee cells leader-check` refuses with `files_changed names path(s) the cell's commit(s) never touched: packages/bee/design` (leader_check.rs:316) when files_changed holds a directory and the commit touched files under it. A files_changed entry counts as touched when a commit path equals it OR lies under it as a directory prefix (component-wise, so `packages/bee/des` does NOT match `packages/bee/design/x`). An entry with no commit path at or under it is still refused. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml leader_check_directory",
    "must_haves": {
      "truths": [
        "a directory in files_changed with a commit path under it passes",
        "a partial-name prefix does not match",
        "an entry with no commit path at or under it is still refused"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs", "substantive": "component-wise prefix match"}],
      "key_links": ["the match is path-component based"],
      "prohibitions": ["No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-6",
    "feature": "issue-sweep-1024",
    "title": "Report a paseo spawn timeout with a live agent as running",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #129 (`gh issue view 129`). Red first; name EVERY new test with `spawn_timeout`. On the paseo transport, `bee herding run` returns outcome spawn_failed, retryable true, remedy unclaim, when the paseo CLI times out after 15s, while the agent was in fact created and keeps running unsupervised; the retry remedy then invites a second agent on the same cell. Reproduce with the existing fake transport seam (see the test that returns `Err(\"paseo timed out after 15s\")`, run.rs ~14369). Fix: on a spawn timeout, look the agent up by the name the run gave it (the transport's list/lookup); if found, continue supervising it as a normal running agent; if not found, keep spawn_failed. A spawn_failed outcome must never be retryable while an agent with that name exists. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml spawn_timeout",
    "must_haves": {
      "truths": [
        "a spawn timeout whose agent exists is supervised, not reported spawn_failed",
        "a spawn timeout with no agent is still spawn_failed",
        "spawn_failed is not retryable while an agent with that name exists"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "timeout lookup by agent name"}],
      "key_links": ["the lookup uses the same name the spawn used"],
      "prohibitions": ["No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-7",
    "feature": "issue-sweep-1024",
    "title": "Resolve one state record for writes and reads in a granted worktree",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/state.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "packages/bee-rs/crates/bee/src/state.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #128 (`gh issue view 128`). Red first; name EVERY new test with `worktree_state_record` and put them inline in the files this cell owns. In a granted feature worktree, `bee gate --name uat --approved true` and `bee state set` wrote main's default .bee/state.json while `bee status` and `bee close` read the worktree's own .bee/state.json, so the uat gate could never be approved there. First reproduce in a test: a temp main repo plus a registered worktree, write a gate from the worktree, read it back through the path status/close use. Then make the write and read paths resolve the same record for a session in a granted worktree — the worktree's own record, which status and close already read — while a write from main still targets main's record and lane records keep their current routing. If the fix needs a file outside this cell's list, stop and report it as BLOCKED with the file named. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml worktree_state_record",
    "must_haves": {
      "truths": [
        "a gate written from a granted worktree is the gate bee status and bee close read there",
        "a write from main still lands in main's record"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs", "substantive": "one resolver for write and read"}],
      "key_links": ["write and read share one resolver"],
      "prohibitions": ["No change to lane-record routing", "No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "iss-8",
    "feature": "issue-sweep-1024",
    "title": "Name a design role with no design skill in bee doctor",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["8f114141-d74d-463f-b4a6-529c0db32761"],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue #136 (`gh issue view 136`). Red first; name EVERY new test with `doctor_design_skill` and put them inline in doctor.rs. Design skills reach a project only by pick (design-os-in-bee D4), so a project can have the design role and .claude/agents/bee-design.md without any bee-design skill, and bee doctor says nothing. Add one doctor row in mechanical_rows (doctor.rs:146): when a design role is configured in .bee/config.json team or the bee-design agent file exists, and no `bee-design` skill directory exists in any project skill home (.claude/skills, .agents/skills, .opencode/skills), the row warns (never fails) that design work runs without the design taste, and names the remedy `bee onboard --apply --design-packs <web,charts,motion,apple,figma>` or `--design-packs none` to record no pick. With the skill present, or with design.packs recorded as an empty list (none), the row is ok. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor_design_skill",
    "must_haves": {
      "truths": [
        "doctor warns when the design role or agent exists and no bee-design skill is installed",
        "the warning names --design-packs",
        "doctor is ok when the skill exists or the pick is none"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "design skill row"}],
      "key_links": ["the row is a warn, never a fail"],
      "prohibitions": ["No edit to any existing assertion", "No code comments"]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

| Case | Scenario | Pass when |
|---|---|---|
| #131 happy | `git fetch -q origin` on idle main | guard allows |
| #131 edge | `git push --force origin main`, `git push origin other` | guard refuses |
| #135 happy | `git add .bee/config.json` in a granted worktree | guard allows |
| #135 edge | `echo x > .bee/config.json` | guard refuses |
| #132 happy | only stamp files dirty in main | merge auto-commits and proceeds |
| #132 edge | another skills file dirty | merge refuses MAIN_DIRTY |
| #130 happy | `cells cap --report-file p` | guard allows |
| #130 error | `cells cap` with neither | guard refuses |
| #134 happy | escalated cell, v2 plan | payload passes the model guard |
| #134 edge | `escalate --off` | escalate cleared, no budget check |
| #133 happy | directory in files_changed | leader-check passes |
| #133 edge | partial-name prefix | refused |
| #129 happy | timeout, agent listed | supervised as running |
| #129 error | timeout, no agent | spawn_failed |
| #128 happy | gate from worktree | status in worktree reads it |
| #136 happy | design role, no skill | doctor warns with --design-packs |

## Open Questions

- #128 and #129 are marked unverified in their reports; if a cell cannot
  reproduce, it reports BLOCKED with what it saw rather than guessing a fix.

## Out of scope

- #69, the cross-project dashboard: a feature request that needs shaping.
