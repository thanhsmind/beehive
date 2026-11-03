# Plan: deploy-issuer-session

Lane: small · class: feature · flags: authorization · product files: 2

## Summary

A deploy dispatch loops on pi. `bee dispatch prepare` writes `issuer_session`
only from the `--session-id` flag. `bee dispatch authorize` finds the acting
session through the flag, then the env chain, then the one live session. A
flagless prepare makes a record that authorize always refuses, and the refusal
tells the leader to dispatch again. One cell makes prepare stamp the session
through the same resolver authorize uses, test first.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Prepare stamps issuer_session only from the raw session parameter, on the deployment stage. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2662-2663` | `if let Some(sid) = session_id {` then `economics.insert("issuer_session".into(), Value::String(sid.to_string()));` |
| 2 | Authorize resolves the acting session with the full resolver. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4416` | `let session_id = crate::verbs::state_group::resolve_session_id(requested_session_id, &root)` |
| 3 | The resolver takes the flag, then the env chain, then single-live-session adoption. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:386-387` | `/// claims.mjs resolveSessionId({flag, root}) — the explicit flag wins, then` then `/// the env chain, then single-live-session adoption.` |
| 4 | A record with no issuer_session is refused by authorize. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4254` | `dispatch record is missing issuer_session; deployment dispatch must be bound to an issuer session.` |
| 5 | The looping pi dispatches carried no issuer_session. | ran | `.bee/logs/dispatch.jsonl` (main checkout) | `"dispatch_id":"1ebb4d04-1ce0-45dc-938e-c350cc8f666a","issuer_session":null` |

## Cells — current slice preview

```json
[
  {
    "id": "dis-1",
    "feature": "deploy-issuer-session",
    "title": "Bind deployment dispatches to the resolved issuer session",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "docs/history/deploy-stage-execution/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "BUG: `bee dispatch prepare --stage deployment` writes `issuer_session` into the dispatch record ONLY when the raw `--session-id` flag is passed, while `bee dispatch authorize` resolves the acting session through `crate::verbs::state_group::resolve_session_id(flag, root)` (flag, then env chain BEE_SESSION_ID / CLAUDE_CODE_SESSION_ID / PI_SESSION_ID, then single-live-session adoption). A pi session that runs prepare without the flag gets a record with no issuer_session, authorize refuses with deploy_authorization_wrong_session ('dispatch record is missing issuer_session'), the fix text says re-dispatch, and the leader loops. Anchors (rg -n in prepare.rs): `1340:    session_id: Option<&str>,` (param of fn prepare_dispatch_wire); `1462:                crate::verbs::state_group::session_binding(session_id, root)`; `2658:        if v2.stage.as_deref() == Some(\"deployment\") {`; `2662:            if let Some(sid) = session_id {`; `2663:                economics.insert(\"issuer_session\".into(), Value::String(sid.to_string()));`; authorize side `4416:    let session_id = crate::verbs::state_group::resolve_session_id(requested_session_id, &root)`. STEP 1 (red first): in tests.rs, next to `fn test_deploy_authorization_authorize_success_and_reuse_refusal` (it builds the pi team config, lane feat-auth with approved_cell_packet, and .bee/sessions/sess-auth.json, then calls `prepare_dispatch_wire(... Some(\"feat-auth\"), Some(\"deployment\"), Some(\"sess-auth\"), Some(\"2.39.0\"))`), add a test that builds the same fixture but calls prepare_dispatch_wire with session flag `None` while `BEE_SESSION_ID` is set to the session id (use BEE_SESSION_ID, not PI_SESSION_ID: the chain checks BEE first and a developer shell may carry CLAUDE_CODE_SESSION_ID; follow the set_var/remove_var pattern at `9252:        unsafe { std::env::set_var(\"BEE_SESSION_ID\", sid); }`, and remove the var before any assert can panic, or use a guard). Assert observable behavior: the prepared dispatch's record in .bee/logs/dispatch.jsonl carries issuer_session equal to that id, AND `authorize_dispatch_permit(&root, dispatch_id, \"2.39.0\", Some(<id>))` returns ok. Run it and watch it FAIL for the reported reason (missing issuer_session) before you fix. STEP 2 (fix): in prepare_dispatch_wire, stamp issuer_session from the SAME resolution authorize uses — `resolve_session_id(session_id, root)` — so prepare and authorize cannot disagree; keep the stamp limited to the deployment stage and keep writing nothing when no session resolves (authorize still refuses that record). Do not change authorize_dispatch_permit or its refusal matrix. STEP 3: run the new test plus the existing deploy_authorization tests.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_authorization",
    "must_haves": {
      "truths": [
        "prepare --stage deployment without --session-id, with a session id in the env chain, writes issuer_session equal to that id",
        "authorize with the same acting session then succeeds instead of refusing deploy_authorization_wrong_session"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "issuer_session stamped from resolve_session_id, the same resolver authorize uses"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "new red-first test for the flagless prepare then authorize path"}
      ],
      "key_links": ["prepare.rs deployment branch calls crate::verbs::state_group::resolve_session_id"],
      "prohibitions": [
        "No change to authorize_dispatch_permit refusal checks",
        "Existing test_deploy_authorization_* tests stay unchanged and green"
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

The cell's `verify` runs the deploy authorization tests in release mode, the
mode CI uses. The release runs `scripts/release.sh`, which runs the full
declared suite before it tags.
