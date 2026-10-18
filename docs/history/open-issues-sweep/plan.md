---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Fix the open bee-report issues

## Summary

This fixes twelve defects that users filed against bee. After it lands:
an escalated cell runs on the session model; `cells unclaim` frees the
claim; a Paseo deploy worker can use its release permit; the Paseo
"start the daemon" hint shows only when the daemon is down; the Windows
build compiles; decisions and backlog items logged in a feature
worktree reach the shared store; `triggers add` refuses an unknown
decision; the plan freeze follows the real gate; `bee close` stops
flagging files nobody may edit; a closed feature's promote reminder can
be cleared; onboarding stops overwriting newer verify-app edits; and a
worker can pass its cap report from a file.

Mode: `standard` — 3 risk flags: external systems (Paseo), public
contracts (CLI output and refusals), multi-domain (dispatch, cells,
herding, stores, hooks, onboarding).
Why this is the least workflow that protects the work: each fix is a
small change at one traced root cause, proven by a red-first unit test
next to the existing tests for that code.

Playbook: `skills/bee-planning/playbooks/bugfix.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1: an escalated cell resolves to the session model even with a v2 planned role; only an explicit `--role` outranks it.
- D2: `cells unclaim` and `cells reopen` remove the claim store row.
- D3: a herding child with `BEE_DISPATCH_ID` in the passthrough keeps the passthrough `BEE_SESSION_ID`; other leader session vars are stripped.
- D4: the daemon-start FIX appears only when the Paseo daemon does not answer.
- D5: the unix-only test helper in `herding/run.rs` is `cfg(unix)` so Windows compiles.
- D6: in a granted worktree, `decisions *` and `backlog *` use the control-root store.
- D7: `triggers add` refuses a decision id that is not in the store.
- D8: the plan-freeze fallback reads the control root's `.bee/state.json`.
- D9: the doc-deferral scan skips a frozen plan.md and generated knowledge index files.
- D10: the promote reminder and the idle `scribing-run` refusal name one command that clears the reminder, and it works from phase idle.
- D11: onboard never overwrites a verify-app projection file newer than its source; it reports it blocked.
- D12: `cells finish` and `cells cap` accept `--report-file <path>`.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The escalate check needs no role, but the v2 path fills the role | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2223 | let escalated_cell = role.is_none() |
| 2 | The role is set from the v2 planned role before that check | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2016 | v2_effective_role.as_deref() |
| 3 | unclaim flips the status and never clears the claim row | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1756 | cell_map.insert("status".into(), Value::String("open".into())); |
| 4 | A claim-row remover already exists for cap | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:206 | pub(crate) fn release_claim_file_best_effort(root: &Path, id: &str) { |
| 5 | build_child_env strips session vars after the passthrough merge | read | packages/bee-rs/crates/bee/src/herding/run.rs:2906 | // Strip leader session IDs even if present in passthrough or agent_env |
| 6 | The daemon FIX is appended to every paseo run error | read | packages/bee-rs/crates/bee/src/herding/run.rs:3645 | "{e}{agent_msg} — FIX: start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)" |
| 7 | A daemon probe already exists | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:878 | pub fn daemon_reachable() -> bool { |
| 8 | Windows CI fails on a unix import in a test helper | read | packages/bee-rs/crates/bee/src/herding/run.rs:12231 | use std::os::unix::fs::PermissionsExt; |
| 9 | decisions resolve through the WIDE door, worktree-local | read | packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs:58 | let root = match crate::roots::resolve_store_root_any(start) { |
| 10 | backlog resolves through the same WIDE door | read | packages/bee-rs/crates/bee/src/verbs/backlog.rs:54 | use crate::roots::{resolve_store_root_any as resolve_store_root, Roots}; |
| 11 | triggers add stores the id with no lookup | read | packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:519 | let short8 = truncate_chars_head(decision, 8); |
| 12 | plan-freeze falls back to the state map it was handed | read | packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:284 | if state_feature != Some(feature) { |
| 13 | doc-deferral scans every touched docs/ file | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1291 | let mut files: Vec<String> = touched.into_iter().filter(\|f\| f.starts_with("docs/")).collect(); |
| 14 | Generated knowledge index files carry a fixed header | read | packages/bee-rs/crates/bee/src/verbs/knowledge/index.rs:22 | GENERATED FILE — do not hand-edit. |
| 15 | scribing-run refuses from idle with no route to clear | read | packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:293 | Legal from: swarming, reviewing, scribing. FIX: if execution really is done |
| 16 | onboard copies any drifted verify projection | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:575 | if verify_skill_drifted(&source_walk, &target_walk) { |
| 17 | cells finish reads the report only from a flag | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1172 | let report = opt_string_flag(flags, "report")?; |
| 18 | prune is already fixed for #104/#106 | ran | bee worktree prune --dry-run --older-than-days 0 | would remove, 98.7 MB reclaimable |
| 19 | cells add and dispatch prepare already map a granted worktree to main | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:36 | pub(crate) fn resolve_root_serving_granted(cwd: &Path) -> (Roots, Option<PathBuf>) { |

## Discovery

The leader read each issue and traced its root cause at the anchors
above; one read-only gather confirmed the herding items (claims 5-7).
Issues #104 and #106 are already fixed by commit e40edc403 (claim 18).
Issue #102's refusal comes from the Claude Code worktree-isolation
guard, not from bee; D12 gives bee a way around it.

## Approach

Each cell changes the one root-cause site and adds a red-first test next
to the existing tests for that code. D6 reuses the resolver that `cells add`
and dispatch already use to map a granted worktree to its control root
(claim 19); both store families call it. Rejected: syncing the worktree decisions store into
main at merge (two homes for one fact, and lost in hosts where `.bee` is
ignored); allowing `scribing-run` from idle (it records spec sync for
executed work, and close already queues a promote item).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Store re-root breaks decisions tests that pin worktree-local stores | MEDIUM | ois-4 | `cargo test decisions` and `backlog` green after the change |
| Session var kept for a non-permit worker | MEDIUM | ois-3 | test: no `BEE_DISPATCH_ID` means `BEE_SESSION_ID` stripped |
| Parallel cells break each other's build in one worktree | MEDIUM | all | workers wait and re-run on a compile error in a file outside their list |

Waves: ois-1 to ois-6, ois-8 and ois-9 run in parallel (disjoint
files). ois-7 waits for ois-1 (both edit `verbs/drivers/tests.rs`).
ois-10 waits for ois-2 (both edit `verbs/cells/handlers_close.rs` and
`verbs/cells/tests.rs`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ois-1 to ois-10 change Rust code and its tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each code cell; the live proof is the leader's."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader syncs knowledge at capture."},
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

Epic map. Outcome: every open bee-report defect has a fix and a test.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Dispatch and claims | escalate, unclaim, report file | workers get the wrong transport or a stuck claim | ois-1, ois-2, ois-10 | prepare and cells tests |
| Herding | session env, daemon hint, Windows build | deploy permit refused, wrong FIX, red CI | ois-3 | herding tests |
| Stores | decisions, backlog, triggers | worktree records never reach the shared store | ois-4, ois-5 | decisions, backlog, triggers tests |
| Guards and doors | plan freeze, doc-deferral, promote reminder | refusals with no way out | ois-6, ois-7, ois-8 | write_guard, close, status tests |
| Onboarding | verify-app projections | host edits lost | ois-9 | onboard tests |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ois-1 | Honour the escalate flag when the plan assigns a role | verbs/drivers/prepare.rs, verbs/drivers/tests.rs | — | an escalated cell's prepare returns a session-model payload, no herding command | cargo test prepare |
| ois-2 | Remove the claim row on unclaim and reopen | verbs/cells/handlers_close.rs, verbs/cells/tests.rs | — | after `cells unclaim`, `cells claim` succeeds at once | cargo test cells:: |
| ois-3 | Keep the permit session for deploy workers, name the right Paseo FIX, gate the unix test helper | herding/run.rs, herding/tests.rs | — | a Paseo deploy worker sees `BEE_SESSION_ID`; a Paseo agent error shows a config FIX; Windows compiles | cargo test herding:: |
| ois-4 | Route decisions and backlog to the control-root store in a granted worktree | verbs/decisions/verbs_write.rs, verbs/decisions/tests.rs, verbs/backlog.rs | — | `decisions log` and `backlog pbi add` from a worktree land in main's store | cargo test decisions, backlog |
| ois-5 | Refuse a triggers add for an unknown decision | verbs/triggers/mod.rs | — | `triggers add --decision null` exits 1 and writes nothing | cargo test triggers |
| ois-6 | Read the control-root state in the plan-freeze fallback | hooks/write_guard/checks.rs, hooks/write_guard/tests.rs | — | after a gate unapprove, plan.md edits are allowed in the worktree | cargo test plan_freeze |
| ois-7 | Skip a frozen plan.md and generated index files in the doc-deferral scan | verbs/drivers/close.rs, verbs/drivers/tests.rs | ois-1 | `bee close --dry-run` no longer lists frozen plan.md or patterns/index.md lines | cargo test doc_deferral |
| ois-8 | Give a closed feature's promote reminder a working clear command | verbs/status_full/mod.rs, verbs/status_full/tests.rs, verbs/state_group/store.rs, verbs/state_group/tests.rs, verbs/deferred_queue.rs | — | the reminder names a command; running it from idle clears the line | cargo test promote, deferred_queue, scribing |
| ois-9 | Never overwrite a newer verify-app projection in onboard | onboard/plan.rs, onboard/tests.rs | — | onboard plan shows `blocked_verify_skill` for a newer projection | cargo test onboard |
| ois-10 | Accept the cap report from a file | verbs/cells/handlers_close.rs, verbs/cells/tests.rs, catalog.rs, generated/registry_payload.json | ois-2 | `cells finish --report-file r.json` caps like `--report` | cargo test cells:: |

Paths are under `packages/bee-rs/crates/bee/src/`. The cell packet:

```json
[
  {
    "id": "ois-1",
    "feature": "open-issues-sweep",
    "title": "Honour the escalate flag when the plan assigns a role",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "9718c2d3-84b1-4b55-ad90-5f5db74493da"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/validate.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "action": "Per D1 (issues 109, 111, 113). In verbs/drivers/prepare.rs the line `let escalated_cell = role.is_none()` (inside the role resolution block, just after the comment that starts `D5 (store 97ce5225)`) tests the `role` variable, but `let role = if role.is_some() { role } else { v2_effective_role.as_deref() };` earlier fills `role` with the approved plan's planned role, so on any v2-plan feature an escalated cell never escalates and gets the herding command. Make the check use the caller's own flag (`orig_role`, set by `let orig_role = role;`) instead of the role that the v2 plan filled, so an explicit --role still outranks the flag but a planned role does not. Make sure the v2 path does not then refuse the escalated dispatch as planned_role_mismatch and that the payload carries no herding command and no model param (Resolved::Inherit). Red first: in verbs/drivers/tests.rs add a test that builds a v2-plan feature (copy the fixture shape of an existing v2 planned_role test — rg -n planned_role verbs/drivers/tests.rs) whose cell role resolves to a herding slot, sets the cell `escalate: true`, runs prepare --kind cell, and asserts the payload is not a Bash herding command; watch it fail before the fix. Keep every existing prepare test green. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee prepare",
    "must_haves": {
      "truths": [
        "an escalated cell on a feature with a v2 role plan gets a session-model payload with no herding command",
        "an explicit --role still outranks the escalate flag"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "a v2-plan escalate test that failed before the fix"}],
      "key_links": ["the escalate check reads the caller's original --role, not the v2-filled role"],
      "prohibitions": ["No change to non-escalated dispatch output", "No code comments"]
    }
  },
  {
    "id": "ois-2",
    "feature": "open-issues-sweep",
    "title": "Remove the claim row on unclaim and reopen",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D2", "50894817-53b4-4f97-a4b9-b58967409fa0"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Per D2 (issue 108). In verbs/cells/handlers_close.rs, `pub(crate) fn unclaim_cell(` flips the cell to open but never removes the claim store row, so `cells claim` refuses CLAIMED and `dispatch prepare` refuses not_claimed until the TTL lapses. Cap already calls `release_claim_file_best_effort(root, id); // D1 Δ2: cap clears the claim` (same file) and claims.rs documents that helper as the claim-clearing call for cap/unclaim/block/drop/reopen. After a successful mutate in `unclaim_cell` call `release_claim_file_best_effort(&root, &id)`; do the same in `run_reopen` after its successful mutate. Check the block and drop paths in this file and add the call where it is missing. Red first: in verbs/cells/tests.rs add a test that claims a cell (use the existing claim test fixture — rg -n \"fn .*claim\" verbs/cells/tests.rs), unclaims it, asserts the claim file under .bee/claims is gone, and claims it again successfully; watch it fail before the fix. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells::",
    "must_haves": {
      "truths": [
        "after cells unclaim the claim store row for that cell is gone",
        "a second cells claim on the unclaimed cell succeeds",
        "cells reopen also clears the claim row"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "substantive": "unclaim-then-claim test that failed before the fix"}],
      "key_links": ["unclaim_cell calls release_claim_file_best_effort"],
      "prohibitions": ["No change to the ownership guard", "No code comments"]
    }
  },
  {
    "id": "ois-3",
    "feature": "open-issues-sweep",
    "title": "Keep the permit session for deploy workers, name the right Paseo FIX, gate the unix test helper",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D3", "D4", "D5", "8f1603db-6fb1-4948-b0e6-fe15afafbaf6"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/session_identity.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Three fixes in herding/run.rs. (1) Per D3 (issues 103, 112): `pub(crate) fn build_child_env(` strips every SESSION_ENV_VARS entry after merging the passthrough (the loop after the line `// Strip leader session IDs even if present in passthrough or agent_env`), so a Paseo deploy worker never gets the issuer `BEE_SESSION_ID` and `bee dispatch authorize` refuses deploy_authorization_wrong_session. Keep the strip, then re-insert `BEE_SESSION_ID` from `passthrough` only when `passthrough` also holds a non-empty `BEE_DISPATCH_ID` (the herdr pane path already exports it — see PASSTHROUGH_ENV_VARS). CLAUDE_CODE_SESSION_ID and PI_SESSION_ID stay stripped always. In herding/tests.rs keep `child_env_construction_excludes_leader_session_and_sets_worker_marker` passing and add a test: passthrough with BEE_DISPATCH_ID and BEE_SESSION_ID gives the child BEE_SESSION_ID and no PI_SESSION_ID; passthrough with BEE_SESSION_ID but no BEE_DISPATCH_ID gives none. (2) Per D4 (issue 98): in `execute_paseo`, the `Err(e)` arm of `cli.call_as_caller(&argv)` always appends `— FIX: start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)`. Append that FIX only when the daemon does not answer; when it answers (paseo::daemon_reachable() true, or the error text carries a paseo JSON error code such as AGENT_CREATE_FAILED — see paseo::paseo_error_text), append instead a FIX that tells the user to check the agent's provider and model in the herding agent config (name the agent). Use a seam so tests never call a real daemon (the file already has fakes for the PaseoCli; follow the existing test that asserts the daemon FIX — rg -n \"FIX: start the daemon\" herding/run.rs — and add one for the answered case). (3) Per D5 (issue 96): `fn seed_claude_guard(` in the test module uses `std::os::unix::fs::PermissionsExt` and breaks the Windows build; mark it and every test that calls it `#[cfg(unix)]`. Red first for (1) and (2). No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::",
    "must_haves": {
      "truths": [
        "a child env built with BEE_DISPATCH_ID and BEE_SESSION_ID in the passthrough carries BEE_SESSION_ID",
        "without BEE_DISPATCH_ID no leader session variable reaches the child",
        "a paseo run error from a live daemon does not name the daemon-start FIX",
        "seed_claude_guard and its callers compile only on unix"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/tests.rs", "substantive": "permit session env tests"}],
      "key_links": ["build_child_env feeds both the paseo and the no-pane child"],
      "prohibitions": ["No real paseo call in tests", "No code comments"]
    }
  },
  {
    "id": "ois-4",
    "feature": "open-issues-sweep",
    "title": "Route decisions and backlog to the control-root store in a granted worktree",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D6", "c0379e71-f6a9-45a0-b2fa-bf55197bd61c"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/verbs_write.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Per D6 (issues 97, 105, 110, 114). Today `decisions_prelude_at` in verbs/decisions/verbs_write.rs resolves `crate::roots::resolve_store_root_any(start)` and backlog.rs imports `resolve_store_root_any as resolve_store_root`; for a GRANTED worktree that returns the worktree's own store, so decisions and PBIs logged there never reach the control-root store the claim door, dispatch and cells add read. `pub(crate) fn resolve_root_serving_granted(cwd: &Path) -> (Roots, Option<PathBuf>)` in verbs/drivers/prepare.rs already maps a granted worktree to `Roots::Ordinary(main_root)` and is the door cells add and dispatch prepare use. Call it (take `.0`) in `decisions_prelude_at` and in backlog.rs's `preamble` in place of the WIDE door; do not edit prepare.rs (a sibling cell owns it). Delete header comment lines in those two files that are now false; add no new comments. Red first: in verbs/decisions/tests.rs, near the existing granted-worktree fixtures (rg -n \"granted\" verbs/decisions/tests.rs), add a test that runs decisions_prelude_at from a granted worktree and asserts the root is the main checkout; update any existing test that pinned the worktree-local store to the new contract. Add a matching backlog test in backlog.rs's test module if one exists for its preamble, else a test that backlog's root resolution from a granted worktree is the main checkout using the same fixture shape. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee decisions && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee backlog",
    "must_haves": {
      "truths": [
        "decisions log from a granted worktree writes the main checkout's .bee/decisions.jsonl",
        "backlog verbs from a granted worktree use the main checkout's .bee/backlog.jsonl",
        "an ordinary checkout resolves exactly as before"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/decisions/tests.rs", "substantive": "granted-worktree control-root test"}],
      "key_links": ["decisions_prelude_at and backlog preamble call resolve_root_serving_granted"],
      "prohibitions": ["No change to other verbs on the WIDE door", "No code comments"]
    }
  },
  {
    "id": "ois-5",
    "feature": "open-issues-sweep",
    "title": "Refuse a triggers add for an unknown decision",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D7", "60768791-01dd-4bc3-a67d-0c7407a05471"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/read.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D7 (issue 99). `fn add_record(` in verbs/triggers/mod.rs stores `truncate_chars_head(decision, 8)` with no lookup, so `--decision null` registers a trigger bound to nothing. Before any write, resolve the given id against the decisions store at the control root (`ctx.control`, the same root the trigger store uses): accept a full id or a unique prefix of at least 8 characters of an existing decision event id (active or archived — reuse the decisions read model, e.g. verbs/decisions read.rs helpers or decision_target_candidates in verbs_write.rs, never a second parser). An unknown id returns Err with `bee triggers add: --decision <id> names no decision in the store. FIX: bee decisions search --text <words> to find the id.` and writes no file. Red first: in the test module of triggers/mod.rs (see `fn add_list_resolve_round_trip`) add a test that `add_record` with `null` refuses and leaves the triggers dir empty, and that a seeded decision id is accepted; adjust existing tests that add triggers so they seed a decision first. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee triggers",
    "must_haves": {
      "truths": [
        "triggers add with a decision id not in the store exits with an error and writes no trigger file",
        "triggers add with a real decision id still registers the trigger"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs", "substantive": "decision lookup and refusal test"}],
      "key_links": ["the lookup reads the same decisions read model decisions verbs use"],
      "prohibitions": ["No second decisions.jsonl parser", "No code comments"]
    }
  },
  {
    "id": "ois-6",
    "feature": "open-issues-sweep",
    "title": "Read the control-root state in the plan-freeze fallback",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D8", "263b3eac-f3d2-495b-974d-61ecc8f9aae2"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/gates.md"],
    "action": "Per D8 (issue 101). `pub(crate) fn plan_freeze_shape_approved(` in hooks/write_guard/checks.rs reads the control-root lane file first, then falls back to the `state` map it was handed — in a worktree that is the worktree's own .bee/state.json projection, which goes stale after `bee gate --merge --approved false` writes the control plane. In the fallback, read `<control_root>/.bee/state.json` (same read_json_g helper) and use its feature and approved_gates.shape; use the handed `state` only when the control-root file cannot be read. Red first: in hooks/write_guard/tests.rs, next to `plan_freeze_shape_approved_reads_the_default_state_when_no_lane_exists`, add a test where the control root's state.json says shape false and the handed state says shape true for the same feature, and assert false; keep the existing plan_freeze tests green (adjust fixtures that relied on the handed map only if the control root and the root are the same dir there). No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_freeze && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee plan_md",
    "must_haves": {
      "truths": [
        "with no lane record, the plan freeze follows the control root state.json, not the worktree copy"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "stale-worktree-state test"}],
      "key_links": ["plan_freeze_shape_approved reads control_root/.bee/state.json"],
      "prohibitions": ["No change to the lane-record path", "No code comments"]
    }
  },
  {
    "id": "ois-7",
    "feature": "open-issues-sweep",
    "title": "Skip a frozen plan.md and generated index files in the doc-deferral scan",
    "lane": "standard",
    "role": "code",
    "deps": ["ois-1"],
    "decisions": ["D9", "90204487-7e6c-4d41-97be-42534e40dc09"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/index.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D9 (issue 115). `pub(crate) fn doc_deferral_scan_files(` in verbs/drivers/close.rs scans every touched docs/ file plus every markdown file under docs/history/<feature>/, so the door flags lines in the feature's frozen plan.md (the plan-freeze guard refuses edits once shape is approved) and in generated files such as docs/knowledge/patterns/index.md (rendered by `bee knowledge index`; its first bytes are KNOWLEDGE_INDEX_HEADER in verbs/knowledge/index.rs, which starts with `<!--\\nGENERATED FILE — do not hand-edit.`). Drop from the returned list: (a) `docs/history/<feature>/plan.md` when the feature's shape gate is approved — reuse `crate::hooks::write_guard::checks::plan_freeze_shape_approved` (make it reachable if it is not, without changing its behavior) with the control root and the root's state; (b) any file whose content starts with the generated-file header (compare against the KNOWLEDGE_INDEX_HEADER constant's first line, never a second copy of the text). Red first: in verbs/drivers/tests.rs next to the doc_deferral_door tests (rg -n \"fn doc_deferral_door\" verbs/drivers/tests.rs) add one test with a deferral line in a frozen plan.md and one with a deferral line in a generated index file; both must not block. Keep the existing doc_deferral tests green. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doc_deferral",
    "must_haves": {
      "truths": [
        "a deferral line in a frozen plan.md does not block the close door",
        "a deferral line in a generated knowledge index file does not block the close door",
        "a deferral line in an editable doc still blocks"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "two scan-set tests"}],
      "key_links": ["doc_deferral_scan_files uses plan_freeze_shape_approved and KNOWLEDGE_INDEX_HEADER"],
      "prohibitions": ["No change to the baseline format", "No code comments"]
    }
  },
  {
    "id": "ois-8",
    "feature": "open-issues-sweep",
    "title": "Give a closed feature's promote reminder a working clear command",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D10", "fab9ed02-91d1-4346-97ad-4a59c5440062"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/close.md"],
    "action": "Per D10 (issue 107). `pub(crate) fn unapplied_promote_proposals(` in verbs/status_full/mod.rs clears a feature when its scribing ledger stamp is newer than promote-proposals.md OR a `promote` item in .bee/deferred-queue.jsonl for that feature is completed (`deferred_queue::items_for(root, \"promote\", &feature)`). `bee close` enqueues that item (enqueue_promote_deferred_record in verbs/drivers/close.rs), and `bee deferred-queue complete --id <id>` exists, but nothing tells the user, a completion may need a prior claim, and features closed before the queue existed have no item. Make one command clear the reminder from any phase: `bee deferred-queue complete --kind promote --feature <f>` — completes that feature's open promote item, creating and completing one when the feature has none, with no prior claim needed (keep `--id` working as today; refuse when neither `--id` nor `--kind`+`--feature` is given; refuse `--kind promote --feature <f>` when docs/history/<f>/promote-proposals.md does not exist). Add the flags to the parse_shape list in try_native. Make the orient/preamble reminder line for each unapplied feature name that command (find where UnappliedPromoteProposal is rendered: rg -n UnappliedPromoteProposal). In verbs/state_group/store.rs, when `scribing-run` refuses from phase idle and the feature has an unapplied promote proposal, append to the refusal the same command. If the deferred-queue verb help lives in catalog.rs / generated/registry_payload.json and the new flags fail a registry parity test, add them there and run `.bee/bin/bee dev regen`; record that in the report. Red first: a test that a feature with promote-proposals.md and no queue item is listed, then after the complete-by-feature call is not listed; a test that the idle scribing-run refusal names the command. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee promote && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deferred_queue && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee scribing",
    "must_haves": {
      "truths": [
        "bee deferred-queue complete --kind promote --feature <f> clears that feature's promote reminder from phase idle",
        "a feature closed before the queue existed can be cleared by the same command",
        "the reminder line and the idle scribing-run refusal both name the command"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/deferred_queue.rs", "substantive": "complete by kind and feature"}],
      "key_links": ["unapplied_promote_proposals reads the completed item the new path writes"],
      "prohibitions": ["No change to scribing-run's legal phases", "No code comments"]
    }
  },
  {
    "id": "ois-9",
    "feature": "open-issues-sweep",
    "title": "Never overwrite a newer verify-app projection in onboard",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["D11", "6abad4af-9c83-4cc7-a7f7-66b88b83b7d6"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D11 (issue 100). `pub fn compute_verify_skill_items(` in onboard/plan.rs pushes `copy_verify_skill` for any installed projection where `verify_skill_drifted(&source_walk, &target_walk)` is true, so `onboard --apply` replaces host edits that are newer than the canonical `.bee/verify/<name>` source. When the projection has drifted, compare each differing file: if any projection file that differs from the rendered source has a modified time newer than its source file (or exists only in the projection), push `verify_blocked_item(name, &rel, \"installed\", kind, reason)` instead, with reason `installed <rel> has edits newer than .bee/verify/<name> (<file>) - skipped, never overwritten. FIX: copy the edits into .bee/verify/<name>, then re-run bee onboard --apply`. Only an older drifted projection is still copied. Read how the walk results expose per-file bytes/paths before choosing the comparison; add a small helper rather than changing walk_skill_tree's signature if possible. Red first: in onboard/tests.rs next to the copy_verify_skill tests (rg -n copy_verify_skill onboard/tests.rs) add a test that edits a projection file after the source (set a newer mtime with std::fs::File::set_modified) and asserts plan shows blocked_verify_skill and apply leaves the file unchanged; keep the existing tests green (where they rely on overwriting an older projection, make the source newer). No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard",
    "must_haves": {
      "truths": [
        "a projection file newer than its source is reported blocked_verify_skill and not written by apply",
        "an older drifted projection is still refreshed"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/onboard/tests.rs", "substantive": "newer-projection test"}],
      "key_links": ["compute_verify_skill_items decides block vs copy per drifted projection"],
      "prohibitions": ["No removal of any projection file", "No code comments"]
    }
  },
  {
    "id": "ois-10",
    "feature": "open-issues-sweep",
    "title": "Accept the cap report from a file",
    "lane": "standard",
    "role": "code",
    "deps": ["ois-2"],
    "decisions": ["D12", "43b88488-8fe4-4cf4-85c0-80588b848a87"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/open-issues-sweep/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md"],
    "action": "Per D12 (issue 102). A host worktree guard refuses shell arguments that contain the word bash, and the cap proof must equal a cell verify that runs bash, so the --report JSON cannot be passed. In verbs/cells/handlers_close.rs add `report-file` to CAP_FLAGS and FINISH_FLAGS (bump the array lengths) and, where the flags are parsed (`let report = opt_string_flag(flags, \"report\")?;`), read the file at that path (relative to the cwd) as the raw report string when `--report-file` is given; giving both `--report` and `--report-file` refuses with `--report and --report-file are exclusive`; an unreadable file refuses naming the path. Validation stays in the existing report parser. Follow how `deviations-file` is declared and described in catalog.rs and add the `--report-file` flag description to cells cap and cells finish there; then run `.bee/bin/bee dev regen` so generated/registry_payload.json and any other generated output match, and commit those regenerated files with the cell (list them in the report files). Red first: in verbs/cells/tests.rs add a test that caps a claimed cell with --report-file holding a valid report and asserts trace.report matches, and one that both flags refuse. No code comments. If cargo fails to compile in a file outside your files list, a sibling cell is mid-edit: wait 60 seconds and re-run; never edit that file.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee registry",
    "must_haves": {
      "truths": [
        "cells finish --report-file <path> caps with the file's report exactly as --report would",
        "giving --report and --report-file together refuses"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "substantive": "report-file cap tests"}],
      "key_links": ["catalog.rs and the generated registry list --report-file for cells cap and cells finish"],
      "prohibitions": ["No change to report validation rules", "No code comments"]
    }
  }
]
```

## Test matrix

- Happy: each cell's red-first test — Pass when the named test is green after the fix and was red before it.
- Edge: ois-1 explicit `--role` still wins; ois-3 no permit means no session var; ois-4 ordinary checkout unchanged; ois-9 older projection still copied — Pass when those tests are green.
- Error: ois-5 unknown decision refuses; ois-10 both report flags refuse; ois-8 missing proposal file refuses — Pass when each exits with the named error.
- Behavior change on main vs head: ois-1 prepare for an escalated v2 cell — main returns `bee herding run`, head returns no herding command.

## Open Questions

(none)

## Out of scope

- #69 dashboard (own discovery); #104/#106 already fixed; clearing the 72 existing promote reminders.
