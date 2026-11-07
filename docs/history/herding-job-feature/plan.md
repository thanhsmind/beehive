# herding-job-feature — Plan

**Lane:** standard · **Class:** feature · **Date:** 2026-11-07

## Problem

A herding job whose cwd is the main checkout (a reviewer, advisor, judge or
scout) is labelled with whatever feature `.bee/state.json` holds at hook
time, not with the feature that dispatched it. `bee close` joins herded
sessions on that label, so feature token totals over-count one feature and
under-count another (thin-worker-boot review, finding P2-1).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | close keeps a herded job only when the activity label equals the feature | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2509 | `if activity.get("feature").and_then(Value::as_str) != Some(feature) {` |
| 2 | the activity hook falls back to the control root state.json feature | read | packages/bee-rs/crates/bee/src/hooks/activity.rs:917 | `let ReadJson::Parsed(Value::Object(state)) = read_json(&ctrl.join(".bee").join("state.json"))` |
| 3 | a main-cwd review job for thin-worker-boot was labelled with another feature | ran | .bee/mailbox/job-1794016637145-62259-1/activity.json | `"feature": "paseo-worktree-workspace"` |
| 4 | job.json already records the dispatching leader but no feature | ran | .bee/mailbox/job-1794012325950-3541941-1/job.json | `"leader_session":"51a0937f-e0af-4a92-9ada-379bd7fdf8ce"` with no `feature` key |

## Contract (cell HJF-1)

- The job's feature has one home: `job.json` `feature`.
- `bee herding run` takes `--feature <slug>` and stamps `feature` into every
  `job.json` it writes. Resolution order: `--feature`; else the feature of
  the `--cell-id` cell; else the calling session's bound lane; else no key.
- `bee dispatch prepare` adds `--feature "<slug>"` to every `bee herding run`
  command it returns (pane, Paseo, claude-session) when it resolved a
  feature (its `--feature` flag, the cell's feature, or the bound lane).
- The activity hook (`resolve_herded_feature`) reads `job.json` `feature`
  first, before the grant and directory-name rules.
- `bee close` (`herding_jobs`) takes the job's feature from `job.json`
  first and falls back to `activity.json` only when `job.json` has none.
- Old jobs without the key behave as before.
- Tests: a main-cwd job stamped with feature X while `state.json` says Y is
  labelled X by the activity hook and counted by `bee close X`, not by
  `bee close Y`; prepare commands carry `--feature`; `herding run` stamps the
  feature from the flag and from the cell; a job without the key keeps the
  old fallback.
- Docs: the `bee herding run` and `bee dispatch prepare` help text in
  `generated/registry_payload.json`; `docs/knowledge/areas/workflow-state/close-usage-record.md`
  names the job.json feature as the join key.

## Cells — current slice (preview)

| id | title | issue |
|---|---|---|
| hjf-1 | Stamp the dispatching feature into job.json and read it first in the activity hook and bee close | thin-worker-boot review P2-1 |

```json
[{"id":"hjf-1","feature":"herding-job-feature","lane":"standard","role":"code","title":"Stamp the dispatching feature into job.json and read it first in the activity hook and bee close","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/herding/run.rs","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/hooks/activity.rs","packages/bee-rs/crates/bee/src/verbs/drivers/close.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json","docs/knowledge/areas/workflow-state/close-usage-record.md"],"read_first":["docs/history/herding-job-feature/plan.md","/mnt/data/Projects/goglbe/beehive/.bee/mailbox/job-1794016637145-62259-1/report-1.md"],"action":"Follow the Contract in docs/history/herding-job-feature/plan.md. The job feature has one home, job.json feature. bee herding run takes --feature and stamps feature into every job.json it writes (resolution: --feature, then the --cell-id cell feature, then the calling session bound lane, else no key). bee dispatch prepare adds --feature \"<slug>\" to every herding run command it returns when it resolved a feature. hooks/activity.rs resolve_herded_feature reads job.json feature first. verbs/drivers/close.rs herding_jobs takes the feature from job.json first, then activity.json. Old jobs without the key keep the old behavior. Update help text in generated/registry_payload.json and the close-usage-record concept (cite docs/history/herding-job-feature/plan.md, no raw decision ids). Write tests first and watch them fail.","verify":"PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml","must_haves":{"truths":["A main-cwd job stamped with feature X while state.json says Y is labelled X and counted by bee close X, not Y","Every herding run command bee dispatch prepare returns carries --feature when a feature resolved","herding run stamps job.json feature from --feature and from the cell","A job.json without feature keeps the old fallback"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/workflow-state/close-usage-record.md"],"deps":[]}]
```
