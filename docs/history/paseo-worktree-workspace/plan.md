---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Paseo worktree workspaces

## Summary

When bee runs inside Paseo, a bee worktree now shows in Paseo as its own
workspace. `bee worktree new` makes (or reuses) the Paseo workspace and
prints its id. Merge cleanup, prune and unregister archive it. A Paseo
herded worker starts in that workspace and is not a child of the agent that
started it. Outside Paseo nothing changes.

Mode: `standard` — 3 risk flags: external systems (the Paseo CLI and daemon),
public contracts (`worktree new` output, the bee workspace record, the
`paseo run` argv), multi-domain (worktree verbs and the herding transport).
Why this is the least workflow that protects the work: every Paseo call is
best-effort except the run's refusal, so the risk is a wrong argv or a lost
id, and unit tests on a fake `PaseoCli` plus one live run catch both.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1: inside Paseo = `PASEO_AGENT_ID` or `PASEO_CLI` set and not empty; unit tests see outside unless a seam is set.
- D2: `worktree new` ensures a Paseo workspace (reuse by canonical `cwd` from `workspace ls`, else `workspace create --path --isolation local --project <projectId>` with the project from `project ls` whose canonical `path` is the main checkout), records `paseo_workspace_id` on the bee workspace record, prints it; a failure warns and never fails the worktree.
- D3: merge cleanup, prune and unregister archive the recorded workspace before the record goes; a failure warns on stderr.
- D4: `herding run` on Paseo passes `--workspace <id>` (recorded, else ensured inside Paseo); inside Paseo with no id it refuses before spawn; outside Paseo with no id the argv is unchanged.
- D5: every paseo child process drops `PASEO_AGENT_ID` and `PASEO_AGENT_CWD`; the worker `--env` pairs do not carry them.
- D6: paseo JSON is the first JSON document in stdout.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `paseo run` joins the caller workspace without `--workspace` | ran | paseo run --help | defaults to the caller workspace when agent-scoped |
| 2 | `--project` takes a project id, not the name `workspace ls` shows | ran | paseo workspace create --path <wt> --isolation local --project beehive --json | Unknown project: beehive |
| 3 | `project ls` gives the id and path to match | ran | paseo project ls --json | "projectId": "prj_602f0e2c856da05a" |
| 4 | Create returns the workspace id as JSON | ran | paseo workspace create --path <wt> --isolation local --project prj_602f0e2c856da05a --json | "workspaceId": "wks_69fcacfc975748a7" |
| 5 | Paseo stores the main checkout under a symlinked path | ran | paseo workspace ls --json | "cwd":"/home/thanhsmind/Projects/goglbe/beehive" |
| 6 | The run argv is built in one function with `--cwd` | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:186-187 | argv.push("--cwd".to_string()); |
| 7 | The one spawn site of a Paseo worker calls `run_argv` | read | packages/bee-rs/crates/bee/src/herding/run.rs:3574 | let argv = paseo::run_argv( |
| 8 | `RealPaseoCli` spawns paseo with the inherited env | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:782 | None => std::process::Command::new(&self.command) |
| 9 | The wait source spawns paseo on its own | read | packages/bee-rs/crates/bee/src/herding/run.rs:4093 | match std::process::Command::new(&self.command) |
| 10 | Worker env starts from the whole ambient env | read | packages/bee-rs/crates/bee/src/herding/run.rs:2863 | let mut env: BTreeMap<String, String> = ambient |
| 11 | All three teardown paths drop the bee record in one function | read | packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:702 | let _ = ws::unregister_workspace(main_root, id); |
| 12 | `worktree new` holds the worktree-admin lock only inside `create_feature_worktree` | read | packages/bee-rs/crates/bee/src/verbs/worktree/create.rs:98-100 | let out = create_feature_worktree_locked(main_root, feature, base_ref, companion); |
| 13 | The bee workspace record is a JSON map written under its own lock | read | packages/bee-rs/crates/bee/src/verbs/workspace_store.rs:371 | pub(crate) fn register_workspace( |
| 14 | This session is inside Paseo | ran | env \| rg PASEO_AGENT_ID | PASEO_AGENT_ID=fe017d43-6cc0-4822-b211-06a3ae411295 |
| 15 | The prepare probe spawns the configured command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3110 | let mut command = std::process::Command::new(cmd); |

## Discovery

The leader read the Paseo transport (`herding/paseo.rs`, `herding/run.rs`
spawn site), the worktree verbs (`create.rs`, `merge.rs` teardown,
`prune.rs`, `handlers.rs` new rendering) and the bee workspace store, and
probed the installed CLI live (claims 1-5). The probe made workspace
`wks_69fcacfc975748a7` for this feature's own worktree; it stays, and D2's
reuse step finds it.

## Approach

Paseo helpers first (pure argv, parse and ensure functions on the
`PaseoCli` seam, plus the env scrub in the runner). The worktree side and
the run side then call them. Every helper takes `&dyn PaseoCli` so tests use
a fake; only thin wrappers build `RealPaseoCli` from
`paseo_command(read_main_config(main_root))` with an 8 s timeout per call. `paseo_ws::ensure_and_record` is the one ensure-and-record path for both `worktree new` and `herding run`.

The `worktree new` Paseo step runs after the worktree-admin lock is
released, so a slow daemon never holds the lock. The archive step sits in
`teardown_worktree` just before `unregister_workspace`, so merge cleanup,
prune and unregister all get it from one place.

Rejected: a separate `.bee/paseo-workspaces.json` (a second home for a fact
the workspace record already owns); `--isolation worktree` (Paseo would make
its own checkout); failing `worktree new` on a Paseo error (Paseo is a
viewer here, not the owner of the worktree).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Real workspaces made by the test suite | MEDIUM | pw-1 | D1 seam: unit tests run outside Paseo by default |
| Lost workspace id (record removed before archive) | MEDIUM | pw-2 | teardown test with a recorded id on the fake CLI |
| Worker still joins caller workspace | MEDIUM | pw-3 | argv test carries `--workspace`; live run shows the worker in the worktree workspace with no parent |
| Symlinked paths never match | LOW | pw-1 | match test with a symlinked temp dir |

Waves: serial — pw-2 and pw-3 call pw-1's helpers; pw-3 reads pw-2's record
helpers; pw-4 documents all. Leader live proof after pw-3: build the
binary, run `bee worktree new` for a throwaway feature inside this Paseo
session, read `paseo workspace ls`, run `bee worktree unregister`, read it
again; before that, run one cheap `bee herding run --agent paseo-*` from the
throwaway worktree and read `paseo inspect` (workspace = the worktree's,
`ParentAgentId` empty), and check the calling session's agents survive.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"pw-1 to pw-3 change Rust verbs and the herding transport."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"pw-4 updates the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check."}
  ]
}
```

## Shape

Epic map. Outcome: a Paseo user sees every bee worktree and its workers in
the right workspace.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Helpers | paseo argv, parse, ensure, env scrub | one home for every Paseo workspace call | pw-1 | unit tests on a fake CLI |
| Lifecycle | new registers, teardown archives | worktrees are invisible in Paseo | pw-2 | worktree tests + live new/unregister |
| Transport | `--workspace` on run, no parent | workers join the caller workspace | pw-3 | argv and spawn tests + live check |
| Record | docs | the concept must match | pw-4 | pointer check |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pw-1 | Add Paseo workspace helpers and drop agent-scope env from every paseo child | herding/paseo.rs, herding/run.rs, herding/paseo_steer.rs, verbs/drivers/prepare.rs | — | nothing yet; helpers and a clean child env | cargo test herding::; prepare |
| pw-2 | Register worktrees as Paseo workspaces and archive them on teardown | verbs/workspace_store.rs, verbs/worktree/paseo_ws.rs (new), verbs/worktree/mod.rs, verbs/worktree/create.rs, verbs/worktree/handlers.rs, verbs/worktree/merge.rs, verbs/worktree/tests.rs | pw-1 | `bee worktree new` prints `paseo: workspace wks_…`; merge cleanup and prune archive it | cargo test worktree; workspace_store |
| pw-3 | Start Paseo workers in their worktree's workspace | herding/paseo.rs, herding/run.rs | pw-1, pw-2 | a herded Paseo worker shows under the worktree's workspace, not as a child of the leader | cargo test herding:: |
| pw-4 | Document Paseo worktree workspaces | docs/knowledge/areas/bee-herding/the-paseo-channel.md | pw-1, pw-2, pw-3 | the concept names the lifecycle and the run rule | pointer check |

Paths under `herding/` and `verbs/` are in `packages/bee-rs/crates/bee/src/`. The cell packet:

```json
[
  {
    "id": "pw-1",
    "feature": "paseo-worktree-workspace",
    "title": "Add Paseo workspace helpers and drop agent-scope env from every paseo child",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D5",
      "D6"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo_steer.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "docs/history/paseo-worktree-workspace/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/path_identity.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "In herding/paseo.rs add, as pub items: (a) `AGENT_SCOPE_VARS: [&str; 2] = [\"PASEO_AGENT_ID\", \"PASEO_AGENT_CWD\"]`. (b) `inside_paseo() -> bool` per D1: true when PASEO_AGENT_ID or PASEO_CLI is set and non-empty; under cfg(test) it reads a thread_local seam that defaults to false (same pattern as TEST_DISCOVERY at the top of the file) plus a cfg(test) setter `set_inside_paseo_for_test(bool)`. (c) `first_json_document(stdout: &str) -> Option<Value>` per D6: pull the existing scan out of `parse_run_agent_id` (paseo.rs:219-233; the same loop repeats near lines 265, 351, 486, 503, 622) into this one helper, start at `{` OR `[`, and keep the skip-a-failed-parse-and-continue behavior so `[login-shell-env] {` noise is skipped; make `parse_run_agent_id` use it with unchanged results (switch the other copies only where results stay identical). (d) argv builders `workspace_ls_argv()` = [workspace, ls, --json], `project_ls_argv()` = [project, ls, --json], `workspace_create_argv(path: &str, project: Option<&str>)` = [workspace, create, --path, <path>, --isolation, local, (--project, <id>), --json], `workspace_archive_argv(id)` = [workspace, archive, <id>, --json] (do not reuse `archive_argv`, which archives an agent). (e) Path compare: use `crate::path_identity::canonical_paths_equal` (path_identity.rs:200); add no new compare helper. (f) `ensure_workspace(cli: &dyn PaseoCli, main_root: &Path, dir: &Path) -> Result<String, String>` per D2: call workspace ls; if an array row has `cwd` equal to `dir` by canonical_paths_equal, return its `workspaceId`; else call project ls and take the `projectId` whose `path` equals main_root (None when no match); call workspace create; return the `workspaceId` of the first JSON document; every failure is Err carrying paseo's own error text (`error.message` of a JSON error object when present, else the raw text). (g) `archive_workspace(cli: &dyn PaseoCli, id: &str) -> Result<(), String>`. D5, all spawners: in `run_with_deadline` and the no-timeout branch of `RealPaseoCli::call` (paseo.rs), in `RealWaitSource::arm` (run.rs, `Command::new(&self.command)` near line 4093), in `RealSteerRunner::steer` (paseo_steer.rs:108) and in `run_probe_command_bounded` (verbs/drivers/prepare.rs:3110), `env_remove` each AGENT_SCOPE_VARS entry. D5 worker env: in run.rs `execute_paseo`, the block that turns `child_env` into `env_pairs` (near line 3556) — extract a small pub(crate) fn `paseo_env_pairs(child_env: BTreeMap<String,String>) -> Vec<(String,String)>` that drops AGENT_SCOPE_VARS keys, and use it there. Unit tests: first_json_document skips multi-line `[login-shell-env] {` noise and plain log lines and returns an object or an array; parse_run_agent_id results unchanged; ensure_workspace reuses a ls row whose cwd is a symlink of dir (temp dir + std::os::unix::fs::symlink under cfg(unix)); ensure_workspace creates with --project <id> when project ls matches main_root and without --project when none matches; ensure_workspace returns Err with paseo's message on a create error object; argv shapes; paseo_env_pairs drops both vars and keeps others; RealPaseoCli on `sh -c 'echo ${PASEO_AGENT_ID:-absent}'` (or equivalent) with PASEO_AGENT_ID set in the process env prints absent — any test that sets or reads process env vars holds `crate::hooks::herding_env_lock()` (hooks/mod.rs:123) and restores the env after. No test calls a real paseo binary. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare",
    "must_haves": {
      "truths": [
        "ensure_workspace reuses an existing workspace whose cwd is the same directory through a symlink",
        "ensure_workspace passes --project <projectId> of the main checkout, and omits --project when none matches",
        "first_json_document reads a JSON object or array after wrapper log lines, and parse_run_agent_id is unchanged",
        "a paseo child process does not see PASEO_AGENT_ID or PASEO_AGENT_CWD (runner, wait source, steer runner, prepare probe)",
        "inside_paseo is false in unit tests unless the seam sets it",
        "paseo_env_pairs carries neither agent-scope variable"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/paseo.rs",
          "substantive": "workspace helpers, inside_paseo seam, env scrub, and their tests"
        }
      ],
      "key_links": [
        "RealPaseoCli is the runner every helper uses in production"
      ],
      "prohibitions": [
        "No change to existing argv builders' output",
        "No code comments"
      ]
    }
  },
  {
    "id": "pw-2",
    "feature": "paseo-worktree-workspace",
    "title": "Register worktrees as Paseo workspaces and archive them on teardown",
    "lane": "standard",
    "role": "code",
    "deps": [
      "pw-1"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/workspace_store.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/paseo_ws.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/create.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/paseo-worktree-workspace/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/verbs/workspace_store.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/create.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "(1) workspace_store.rs: add `set_paseo_workspace_id(root, id, paseo_id) -> W<()>` that takes the workspace lock, reads the record (read_workspace_record), inserts `paseo_workspace_id`, writes atomically; and `record_for_dir(root, dir) -> Option<Map<String, Value>>` returning the record from list_workspaces whose `root` equals `dir` or is an ancestor of it (canonicalize both with std::fs::canonicalize, fall back to the raw path, then Path::starts_with; the longest root wins). (2) New verbs/worktree/paseo_ws.rs, declared in mod.rs, the ONE home for ensure-and-record: `ensure_and_record(cli: &dyn PaseoCli, main_root, dir) -> Result<String, String>`: a record_for_dir record with a non-empty `paseo_workspace_id` returns it with no paseo call; else herding::paseo::ensure_workspace, then set_paseo_workspace_id when a record matched; returns the id. `real_cli(main_root)` builds `RealPaseoCli::new(paseo_command(&read_main_config(main_root))).with_timeout(8 s)`. `register(main_root, worktree_root) -> Option<Value>`: None when `!herding::paseo::inside_paseo()`; else prints `▸ paseo: registering workspace…` on stderr and returns Some({\"id\": <wks>}) or Some({\"error\": <text>}) from ensure_and_record with the real CLI. `archive_with_cli(cli, main_root, id)` reads the record; when it carries a non-empty `paseo_workspace_id` it calls archive_workspace and prints one stderr line `paseo: archived workspace <wks>` or `paseo: could not archive workspace <wks>: <error> — FIX: paseo workspace archive <wks>`; `archive(main_root, id)` builds the real CLI only when the record carries an id (D3: no inside_paseo check). (3) create.rs: `Created` gains `paseo_workspace: Option<Value>`; post_add sets None; `create_feature_worktree` calls paseo_ws::register AFTER `guard.release()` on Ok and stores the value; fix the two hand-built `Created { … }` in tests.rs (near lines 699 and 736). (4) handlers.rs `new_result_and_text`: when Some, JSON gets key `paseoWorkspace` (camelCase like `worktreeRoot`) with the value, and text gains a line `  paseo:       workspace <wks>` or `  paseo:       workspace not registered — <error>`; when None the JSON and text stay byte-identical to today. (5) merge.rs `teardown_worktree`: call paseo_ws::archive(main_root, id) immediately before `ws::unregister_workspace(main_root, id)`. Tests (in paseo_ws.rs and tests.rs; fake PaseoCli only): set_paseo_workspace_id round trip; record_for_dir finds the record for a subdirectory of its root; ensure_and_record records the id from a fake create; ensure_and_record makes no call when the record already has an id; on a create error it returns Err and the record has no field; register returns None outside Paseo (default seam); archive_with_cli calls `workspace archive <wks> --json` when the record has the id and makes no call when it does not; new_result_and_text with None, Some(id), Some(error). A test that seeds a record with `paseo_workspace_id` calls archive_with_cli and NEVER teardown_worktree or merge/prune, so no test reaches a real daemon. Existing worktree tests stay green. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee worktree && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee workspace_store",
    "must_haves": {
      "truths": [
        "worktree new inside Paseo records paseo_workspace_id and prints it",
        "a Paseo failure leaves the worktree created and prints a warning line",
        "outside Paseo worktree new makes no paseo call and its JSON and text are unchanged",
        "teardown archives a recorded workspace before the record is removed, and a failed archive prints the FIX command",
        "a record without the id makes no paseo call at teardown"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/paseo_ws.rs",
          "substantive": "register and archive with fake-CLI tests"
        }
      ],
      "key_links": [
        "create_feature_worktree calls register after the lock release",
        "teardown_worktree calls archive before unregister_workspace",
        "ensure_and_record is the one ensure-and-record path"
      ],
      "prohibitions": [
        "No paseo call while the worktree-admin lock is held in worktree new",
        "No rollback of the worktree on a Paseo error",
        "No code comments"
      ]
    }
  },
  {
    "id": "pw-3",
    "feature": "paseo-worktree-workspace",
    "title": "Start Paseo workers in their worktree's workspace",
    "lane": "standard",
    "role": "code",
    "deps": [
      "pw-1",
      "pw-2"
    ],
    "decisions": [
      "D1",
      "D4",
      "D5"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "docs/history/paseo-worktree-workspace/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/paseo_ws.rs",
      "packages/bee-rs/crates/bee/src/verbs/workspace_store.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "(1) paseo.rs `run_argv` gains `workspace: Option<&str>`; when Some, push `--workspace <id>` right before `--cwd`. Update every caller (run.rs spawn site and the paseo.rs tests; existing tests pass None and keep their expected argv). (2) run.rs `execute_paseo` (near line 3420), after the version check and before run_argv: resolve the workspace per D4 with pw-2's single path — call `crate::verbs::worktree::paseo_ws::ensure_and_record(cli, &opts.main_root, &opts.cwd)` when `paseo::inside_paseo()` is true, or when `record_for_dir` finds a record that already carries `paseo_workspace_id` (that id then comes back with no paseo call). Outside Paseo with no recorded id pass None. An Err inside Paseo returns `RunOutcome::SpawnFailed(format!(\"paseo workspace for {cwd} could not be found or created: {err} — FIX: paseo workspace create --path {cwd} --isolation local, then re-run\"))` before any `paseo run` call (D4: never the caller workspace). Make paseo_ws / ensure_and_record pub(crate) as needed. Tests (run.rs, fake PaseoCli that records argv as existing paseo spawn tests do; set the D1 seam with paseo::set_inside_paseo_for_test and reset it): a recorded id yields `--workspace <id>` in the run argv with no ls/create call; inside Paseo with no record, ensure is called and its id used; inside Paseo with ensure failing, the run is SpawnFailed naming the FIX and no `run` argv was called; outside Paseo with no record, argv has no `--workspace`. No code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::",
    "must_haves": {
      "truths": [
        "a Paseo run from a worktree with a recorded id passes --workspace <id>",
        "inside Paseo with no id the run ensures one or refuses before spawn with a paseo FIX line",
        "outside Paseo with no id the argv is unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/run.rs",
          "substantive": "workspace resolution before paseo run, with tests"
        }
      ],
      "key_links": [
        "run.rs resolves the workspace through paseo_ws::ensure_and_record"
      ],
      "prohibitions": [
        "No fallback to the caller workspace inside Paseo",
        "No code comments"
      ]
    }
  },
  {
    "id": "pw-4",
    "feature": "paseo-worktree-workspace",
    "title": "Document Paseo worktree workspaces",
    "lane": "standard",
    "role": "docs",
    "deps": [
      "pw-1",
      "pw-2",
      "pw-3"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D4",
      "D5",
      "D6"
    ],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "read_first": [
      "docs/history/paseo-worktree-workspace/CONTEXT.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "Add a section to the Paseo channel concept in its existing style (ASD-STE100, short sentences): when bee counts as inside Paseo; worktree new ensures and records the workspace (reuse by cwd, project id from project ls, local isolation, printed id, warn-only); merge cleanup, prune and unregister archive it; herding runs pass --workspace and refuse inside Paseo instead of joining the caller workspace; every paseo child drops PASEO_AGENT_ID and PASEO_AGENT_CWD so the worker is not a child agent; JSON is read past wrapper log lines. Cite the feature's decision store id (find it with `bee decisions search --text paseo_workspace_id`). Add verbs/worktree/paseo_ws.rs to Pointers. Also state: a human `bee worktree new` in a Paseo terminal (PASEO_CLI set) also registers a workspace; `herding fork` still imports into the caller workspace (out of scope); a failed archive prints its FIX command; the JSON key is `paseoWorkspace`.",
    "verify": "rg -n 'paseo_workspace_id|--workspace|PASEO_AGENT_CWD|paseo_ws.rs' docs/knowledge/areas/bee-herding/the-paseo-channel.md",
    "must_haves": {
      "truths": [
        "The concept names the worktree lifecycle in Paseo and the run workspace rule",
        "Pointers list paseo_ws.rs"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
          "substantive": "a section for worktree workspaces"
        }
      ],
      "key_links": [
        "the section cites the decision"
      ],
      "prohibitions": [
        "No code comments added anywhere"
      ]
    }
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Empty/absent | no project matches the main checkout | pw-1 | create runs without --project |
| Malformed input | wrapper log lines before JSON | pw-1 | the JSON is read |
| Path aliasing | ls cwd is a symlink of the worktree | pw-1 | the workspace is reused |
| External failure | create returns an error object | pw-1, pw-2 | Err with the message; worktree kept, warning printed |
| Wrong context | outside Paseo | pw-2, pw-3 | no paseo call; argv unchanged |
| Lost id | teardown with a recorded id | pw-2 | archive called before the record goes |
| Caller scope | inside Paseo, ensure fails | pw-3 | run refused before `paseo run` |
| Env leak | PASEO_AGENT_ID set in bee's env | pw-1, pw-3 | absent in every paseo child and in --env |
| Regression | existing worktree, herding, prepare tests | each code cell | the cell's verify commands green |

## Open Questions

(none)

## Out of scope

- Paseo-owned worktrees.
- Herdr and tmux transports.
- Back-filling workspaces for older worktrees outside D4's ensure step.
- `bee herding fork` (`paseo import --cwd`, fork.rs:222) still joins the caller workspace.
- Skipping archive when the workspace holds live agents: merge cleanup and prune already refuse a worktree a live bee session holds (`live_session_holds`).
