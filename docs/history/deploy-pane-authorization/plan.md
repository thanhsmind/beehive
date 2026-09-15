# Plan: deploy-pane-authorization

Lane: small · class: bugfix · flags: authorization · product files: 3

## Summary

A pi release still cannot pass authorization. The deploy dispatch runs as a
herding worker in its own pane, and `scripts/release.sh` runs there. The
`export BEE_DISPATCH_ID=… BEE_RELEASE_VERSION=…` that prepare puts in front of
`bee herding run` reaches only the leader's shell: the pane gets the agent's
registry env plus `BEE_HERDING_WORKER` and `BEE_HERDING_JOB_ID`. So release.sh
stops on an unset `BEE_DISPATCH_ID`, and even with it, `authorize` would
resolve the worker's session, not the issuer's. One cell carries the dispatch
id, the release version, and the issuer session into that pane, test first.
The user chose this path (option A).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Prepare exports the dispatch id and version only in front of the herding command. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2550` | `let new_cmd = format!("export BEE_DISPATCH_ID=\"{dispatch_id}\" BEE_RELEASE_VERSION=\"{ver}\"; {cmd}");` |
| 2 | The pane env is the registry env plus two markers only. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2418-2420` | `let mut pane_env = env.clone();` then `pane_env.insert("BEE_HERDING_WORKER".to_string(), "1".to_string());` then `pane_env.insert("BEE_HERDING_JOB_ID".to_string(), opts.job_id.clone());` |
| 3 | release.sh refuses without BEE_DISPATCH_ID and authorizes without a session flag. | read | `scripts/release.sh:102-106` | `\|\| fail "release authorization required — BEE_DISPATCH_ID is unset` then `"$BEE_BIN" dispatch authorize --id "$BEE_DISPATCH_ID" --release-version "$AUTH_VERSION"` |
| 4 | Authorize refuses when the acting session differs from the issuer session. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4276` | `session ID does not match the issuer session recorded at dispatch.` |
| 5 | The env chain checks BEE_SESSION_ID first. | read | `packages/bee-rs/crates/bee/src/session_identity.rs:15-19` | `"BEE_SESSION_ID",` then `"CLAUDE_CODE_SESSION_ID",` then `"PI_SESSION_ID",` |
| 6 | The pi deploy worker pane saw only the two herding markers. | ran | pi transcript `~/.pi/agent/sessions/--home-thanhsmind-Projects-goglbe-beehive--/2026-09-14T15-57-42-412Z_01a0a0a3-768c-747b-a74c-ed7a40241814.jsonl` | `BEE_HERDING_JOB_ID=job-1789437412092-` and `BEE_HERDING_WORKER=1` |
| 7 | The prepare-side issuer session is stamped after the export line is built. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2662` | `if let Some(sid) = crate::verbs::state_group::resolve_session_id(session_id, root)` |

## Cells — current slice preview

```json
[
  {
    "id": "dpa-1",
    "feature": "deploy-pane-authorization",
    "title": "Carry the deploy permit and issuer session into the worker pane",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/session_identity.rs",
      "scripts/release.sh"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "BUG: a pi deploy dispatch cannot release. `scripts/release.sh` runs inside the herding worker pane and needs BEE_DISPATCH_ID, then calls `bee dispatch authorize` with no --session-id, so the acting session comes from the env chain (BEE_SESSION_ID first, session_identity.rs). But prepare's `export BEE_DISPATCH_ID=... BEE_RELEASE_VERSION=...; bee herding run ...` reaches only the leader shell and the `bee herding run` process; the pane gets `pane_env = env.clone()` (registry env) plus BEE_HERDING_WORKER and BEE_HERDING_JOB_ID only. Anchors: prepare.rs `2550:                    let new_cmd = format!(\"export BEE_DISPATCH_ID=\\\"{dispatch_id}\\\" BEE_RELEASE_VERSION=\\\"{ver}\\\"; {cmd}\");` inside `if v2.stage.as_deref() == Some(\"deployment\") {`; prepare.rs `2662:            if let Some(sid) = crate::verbs::state_group::resolve_session_id(session_id, root)` (issuer stamp, built AFTER the export line — resolve the issuer once and use it in both places); run.rs `141:struct Options {`, builder `388:    Ok(Options {`, `4596:    fn test_options(main_root: &Path, dry_run: bool) -> Options {`, `4672:    fn continue_options(main_root: &Path, dry_run: bool) -> Options {`, pane env `2418:    let mut pane_env = env.clone();`, export test `6794:    fn the_export_line_is_always_sent_before_agent_start_marker_present_even_for_array_shape_entries() {`; deploy prepare test tests.rs `10558:        assert!(cmd.contains(\"export BEE_DISPATCH_ID=\"), \"command must export BEE_DISPATCH_ID: {cmd}\");`. STEP 1 (red first): (a) in tests.rs, in the deploy prepare test that calls prepare_dispatch_wire with Some(\"sess-1\") and Some(\"2.38.0\"), assert the command also exports `BEE_SESSION_ID=\"sess-1\"`; (b) in run.rs tests, add a test that builds test_options with the new passthrough field holding BEE_DISPATCH_ID, BEE_RELEASE_VERSION and BEE_SESSION_ID, runs execute with FakeHerdr, and asserts the first pane_run export line carries all three next to the markers; (c) unit-test the builder's selection through an injectable lookup (the pattern of `resolve_env_session_id_from` in session_identity.rs), never by setting real process env: with BEE_DISPATCH_ID set it selects the three non-empty vars, without BEE_DISPATCH_ID it selects nothing even when BEE_SESSION_ID is set. Run and watch (a) and (b) fail. STEP 2 (fix): prepare.rs adds `BEE_SESSION_ID=\"<issuer>\"` to that deployment export only when an issuer session resolves; run.rs adds an Options field (e.g. `pane_env_passthrough: BTreeMap<String, String>`) filled in the builder from process env through that lookup, and the fresh-spawn path merges it into pane_env BEFORE the two marker inserts so the markers still win; test_options and continue_options get an empty map; the --continue path is unchanged. Do not change authorize_dispatch_permit, release.sh, or any refusal. Never forward BEE_SESSION_ID when BEE_DISPATCH_ID is absent: an ordinary herding worker must keep its own identity. STEP 3: run the verify command.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_ && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run::",
    "must_haves": {
      "truths": [
        "a deployment prepare with an issuer session exports BEE_SESSION_ID equal to that issuer beside BEE_DISPATCH_ID and BEE_RELEASE_VERSION",
        "a fresh herding spawn whose caller env has BEE_DISPATCH_ID sends a pane export line carrying BEE_DISPATCH_ID, BEE_RELEASE_VERSION and BEE_SESSION_ID",
        "a herding spawn without BEE_DISPATCH_ID forwards none of the three, even when BEE_SESSION_ID is set"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "deployment export line includes the resolved issuer session"},
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "Options passthrough field, builder selection via injectable lookup, merge into pane_env before markers, tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "deploy prepare test asserts BEE_SESSION_ID export"}
      ],
      "key_links": ["run.rs fresh-spawn pane_env merges Options passthrough before BEE_HERDING_WORKER and BEE_HERDING_JOB_ID"],
      "prohibitions": [
        "No change to authorize_dispatch_permit or its refusal matrix",
        "No change to scripts/release.sh",
        "BEE_SESSION_ID is never forwarded into a pane without BEE_DISPATCH_ID",
        "Existing export-line test expectations for non-deploy spawns stay unchanged"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Proof and close

The cell's `verify` runs the deploy prepare and authorization tests and the
herding run tests in release mode, the mode CI uses. The release then runs
`scripts/release.sh 2.37.3` through a pi deploy dispatch, which runs the full
declared suite before it tags.
