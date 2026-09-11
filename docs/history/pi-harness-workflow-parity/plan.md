---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Pi harness workflow parity

## Summary

Pi sessions will use the same lifecycle checks as Claude Code sessions. The change blocks early main-checkout writes, shows exact work before approval, and stores proof another session can run.

Mode: `high-risk`. Risk flags: audit/security, public contracts, multi-domain, covered contract change, and proof enforcement.

This is the least workflow that protects the work because each reproduced gap crosses a separate enforcement boundary.

Class playbook: `.agents/skills/bee-planning/references/planning-reference.md` ("Class playbooks", `feature`).

## Requirements from CONTEXT.md

- D1 keeps shared rules in the Rust CLI.
- D2 denies source writes before execution approval and outside the feature worktree.
- D3 shows and binds the exact current-slice cell packet.
- D4 stores replayable structured proof and checks the approved command.
- D5 keeps intent and purpose in the correct worker brief.
- D6 recognizes Pi session identity through one ordered helper.
- D7 repairs the two remaining Waggledance records after the harness passes.
- D8 allocates unique job ids for concurrent herding runs.

## Load-bearing claims

Labels are `read` or `ran`. A `read` quote is copied from the named source. A `ran` row names the exact command and result.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Non-cell prompt rendering omits `purpose`. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:938-945` | `let original_request = original_request_block(root, None);` / `("brief", brief.unwrap_or("")),` / `("expertise", expertise.unwrap_or("")),` / `("original_request", &original_request),` |
| 2 | A supplied feature can fall through to the active feature. | read | `packages/bee-rs/crates/bee/src/verbs/intent_group.rs:275-281` | `if let Some(f) = feature.map(js_trim).filter(|f| !f.is_empty()) {` / `push(f);` / `}` / `if let Ok(Some(active)) = active_feature(root) {` / `push(&active);` / `}` |
| 3 | Claim session lookup omits Pi. | read | `packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:639` | `env_nonempty("BEE_SESSION_ID").or_else(|| env_nonempty("CLAUDE_CODE_SESSION_ID"))` |
| 4 | New trace proof fields start as null. | read | `packages/bee-rs/crates/bee/src/verbs/cells/trace.rs:36-38` | `m.insert("verification_evidence".into(), Value::Null);` / `m.insert("verify_output".into(), Value::Null);` / `m.insert("verify_passed".into(), Value::Null);` |
| 5 | Plan hashing reads only the control checkout path. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs:78-80` | `pub(crate) fn advisor_plan_path(root: &Path, feature: &str) -> PathBuf {` / `root.join("docs").join("history").join(feature).join("plan.md")` / `}` |
| 6 | The no-grant worktree refusal only runs during `swarming`. | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:864-869` | `let phase = match record.get("phase") {` / `Some(Value::String(p)) => p.clone(),` / `_ => String::new(),` / `};` / `if phase != "swarming" {` / `return Ok(None);` |
| 7 | Default herding job ids use only millisecond time. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:364` | `job_id.map(str::to_string).unwrap_or_else(|| format!("job-{}", chrono::Utc::now().timestamp_millis())),` |
| 8 | Three concurrent runs selected one job id. | ran | `cat brief | .bee/bin/bee herding run --task-file - --json --agent "agy-flash" --ceiling 1800` | `job-1789132580732` occurred in all three results; two returned `agent_name_taken`. |

## Discovery

The audit report identified four unresolved findings. Direct runs also reproduced purpose loss, Pi session-id loss, and millisecond job-id collisions.

The current gate code validates load-bearing claims but does not parse a cell packet. The current cap code validates a proof string but leaves legacy structured proof fields null.

Gather dispatches could not provide code digests because `--purpose` disappeared from all three prompts. This plan uses direct anchored reads until the dispatch fix lands.

## Approach

See `docs/history/pi-harness-workflow-parity/approach.md`.

Waves: cells 1, 2, 3, 5, and 6 can start in parallel because their source files do not overlap. Cell 4 follows cells 2 and 3 because it binds plan packets to both dispatch and cap contracts. Cell 7 follows all code cells and runs generated-file parity. Cell 8 is a fix-first cell after the full suite exposed ambient Pi identity leakage and a missing registry flag.

Smaller path: no smaller path covers all locked decisions. A Pi-only change leaves shared CLI defects. A prose-only change leaves each reproduced bypass open.

## Shape

The current slice ships the complete enforcement chain. The walking path starts with a Pi prompt, reaches an approved cell, dispatches the correct intent, blocks an invalid write, and records replayable proof.

## Cells, current slice preview

The JSON below is the exact execution packet. The gate preview must render it before approval.

```json
[
  {
    "id": "pihp-1",
    "feature": "pi-harness-workflow-parity",
    "title": "Use one runtime session identity order",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D6", "d59befe3-73c6-415a-91f1-f43b485a1eeb"],
    "files": ["packages/bee-rs/crates/bee/src/session_identity.rs", "packages/bee-rs/crates/bee/src/main.rs", "packages/bee-rs/crates/bee/src/lock.rs", "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs", "packages/bee-rs/crates/bee/src/verbs/work.rs", "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs", "packages/bee-rs/crates/bee/src/verbs/reservations/leases.rs", "packages/bee-rs/crates/bee/src/verbs/reservations/reserve.rs", "packages/bee-rs/crates/bee/src/verbs/reservations/tests.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/waiting_on.rs", "packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs", "packages/bee-rs/crates/bee/src/verbs/status_full/cells.rs", "packages/bee-rs/crates/bee/tests/work_verbs.rs", "packages/bee-rs/crates/bee/tests/session_release.rs"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs", "packages/bee-rs/crates/bee/src/verbs/work.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/the-intent-anchor-and-compaction-survival.md"],
    "action": "Add one environment-only session identity helper. Replace every direct BEE_SESSION_ID and CLAUDE_CODE_SESSION_ID chain with it. Preserve explicit flag precedence and single-live-session fallback. Add Pi-only and precedence tests before the fix.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pi_session_identity",
    "must_haves": {
      "truths": ["PI_SESSION_ID identifies a Pi caller when no explicit or Bee or Claude id exists", "BEE_SESSION_ID and CLAUDE_CODE_SESSION_ID keep their current precedence", "All listed session-owned commands call one helper"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/session_identity.rs", "substantive": "owns the ordered runtime environment lookup and tests"}],
      "key_links": ["work, claims, reservations, state, status, hooks, and locks call the shared helper"],
      "prohibitions": ["Do not change explicit flag precedence", "Do not remove single-live-session fallback"]
    },
    "behavior_change": true
  },
  {
    "id": "pihp-2",
    "feature": "pi-harness-workflow-parity",
    "title": "Keep dispatch purpose and intent feature scoped",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D5", "d59befe3-73c6-415a-91f1-f43b485a1eeb"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/intent_group.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "packages/bee/prompts/gather.md", "packages/bee/prompts/reviewer.md", "packages/bee/prompts/advisor.md"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "packages/bee-rs/crates/bee/src/verbs/intent_group.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md", "docs/knowledge/areas/hook-runtime/the-intent-anchor-and-compaction-survival.md"],
    "action": "Write failing tests for a cross-feature anchor and a non-cell purpose. Make an explicit feature lookup exclusive. Resolve the bound lane feature for non-cell prompts. Add a separate conditional purpose variable to gather, reviewer, and advisor templates. Keep advisor brief-file content separate and do not change omitted-purpose bytes.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pihp_dispatch_context",
    "must_haves": {
      "truths": ["A missing requested-feature anchor never falls through to another active feature", "A non-empty purpose appears byte-for-byte in the worker prompt", "An omitted purpose keeps the previous prompt bytes"],
      "artifacts": [{"path": "packages/bee/prompts/gather.md", "substantive": "contains the conditional purpose block"}],
      "key_links": ["prepare passes purpose and the resolved lane feature to prompt_body_for"],
      "prohibitions": ["Do not read the default intent anchor for dispatch", "Do not weaken prompt template skew checks"]
    },
    "regen_obligation_ack": "wave-barrier",
    "behavior_change": true
  },
  {
    "id": "pihp-3",
    "feature": "pi-harness-workflow-parity",
    "title": "Require replayable proof at cap",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D4", "agents-proof-at-cap"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "packages/bee-rs/crates/bee/src/verbs/cells/trace.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", ".bee/verify/verify-app/features/cells-and-proof.md"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "docs/history/proof-strength-and-expiry/CONTEXT.md", ".bee/verify/verify-app/features/cells-and-proof.md", "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/verification"],
    "action": "Write failing behavior tests. Compare the parsed proof command with the cell verify command using exact trimmed bytes. Reject mismatches before writes. On success, fill verify_command, verify_output, verify_passed, and verification_evidence from the one parsed proof tuple. Keep historical reads compatible.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pihp_replayable_proof",
    "must_haves": {
      "truths": ["Descriptive proof text cannot replace the approved verify command", "A successful cap stores four non-null structured proof fields", "Old caps remain readable"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "has one pre-write proof equality check and one structured trace write"}],
      "key_links": ["parse_tests_proof feeds both validation and stored fields"],
      "prohibitions": ["Do not run tests inside cells finish", "Do not reject historical bare-green caps on read"]
    },
    "behavior_change": true
  },
  {
    "id": "pihp-4",
    "feature": "pi-harness-workflow-parity",
    "title": "Bind gate approval to exact cell packets",
    "lane": "high-risk",
    "role": "code",
    "deps": ["pihp-2", "pihp-3"],
    "decisions": ["D3", "D4", "d59befe3-73c6-415a-91f1-f43b485a1eeb"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/mod.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs", "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "skills/bee-planning/SKILL.md", "skills/bee-planning/references/planning-reference.md", ".bee/verify/verify-app/features/feature-gates.md"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", ".bee/verify/verify-app/features/feature-gates.md", "packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs"],
    "affects_skills": ["skills/bee-planning/SKILL.md", "skills/bee-planning/references/planning-reference.md"],
    "affects_specs": ["docs/knowledge/areas/workflow-state/gates.md", "docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Add a gate preview mode that reads the feature worktree plan, parses the exact JSON cell packet, validates required execution fields, and records its plan hash. Refuse shape approval without a fresh preview. Compare cells add input with the approved packet. Update the planning source skill and verification map, then run the required regen chain.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pihp_gate_packet",
    "must_haves": {
      "truths": ["The preview output shows action, files, read_first, must_haves, and exact verify for every current cell", "Shape approval refuses a missing or stale preview", "Cells add refuses a packet that differs from the approved preview", "Gate and advisor reads use the feature worktree plan"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "parses, validates, hashes, and renders preview packets"}],
      "key_links": ["gate preview stores the plan hash that approval checks", "cells add compares against the packet approved for the current plan revision"],
      "prohibitions": ["Do not persist cells before execution approval", "Do not accept a guessed or malformed packet", "Do not hand-edit generated skill trees"]
    },
    "regen_obligation_ack": "wave-barrier",
    "behavior_change": true
  },
  {
    "id": "pihp-5",
    "feature": "pi-harness-workflow-parity",
    "title": "Block active-feature source writes before swarming",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D1", "D2", "252b7418-bbcb-422c-9984-4309e1af9aa5"],
    "files": ["packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs", "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md", "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs", "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md"],
    "action": "Add failing Claude-shaped and Pi-shaped tests for a code feature on main during exploring and planning. Apply the no-grant worktree refusal in every active code phase while preserving docs, solo tiny, non-git, corrupt-grant, and explicit-off behavior.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pre_gate_main_write",
    "must_haves": {
      "truths": ["A standard or high-risk feature cannot write source on main during exploring or planning", "Claude and Pi hook inputs return the same denial", "Existing named exemptions still pass"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "pins active-phase denial and each preserved exemption"}],
      "key_links": ["Pi plugin contract drives the same Rust write guard"],
      "prohibitions": ["Do not treat an unblocked hook as approval", "Do not remove the solo tiny or docs exemption"]
    },
    "behavior_change": true
  },
  {
    "id": "pihp-6",
    "feature": "pi-harness-workflow-parity",
    "title": "Allocate unique concurrent herding job ids",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["D8", "9daa8378-6c01-4635-a61e-7409338c28da"],
    "files": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "packages/bee-rs/crates/bee/src/herding/run.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md"],
    "action": "Write a failing concurrent uniqueness test. Generate default job ids from milliseconds, process id, and a process-local atomic counter. Keep explicit --job-id and continue behavior unchanged.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee pihp_unique_job_id",
    "must_haves": {
      "truths": ["Concurrent default allocations in one process are unique", "Concurrent CLI processes cannot share a default id while both are live", "Explicit and continued job ids do not change"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "contains the collision-safe allocator and behavior tests"}],
      "key_links": ["parse_run_options uses the allocator only when no id was supplied"],
      "prohibitions": ["Do not add sleeps", "Do not serialize callers"]
    },
    "behavior_change": true
  },
  {
    "id": "pihp-7",
    "feature": "pi-harness-workflow-parity",
    "title": "Prove and publish runtime parity",
    "lane": "high-risk",
    "role": "test",
    "deps": ["pihp-1", "pihp-4", "pihp-5", "pihp-6"],
    "decisions": ["D1", "D7", "d59befe3-73c6-415a-91f1-f43b485a1eeb"],
    "files": ["packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", ".bee/verify/verify-app/features/README.md", ".bee/verify/verify-app/features/feature-gates.md", ".bee/verify/verify-app/features/cells-and-proof.md", "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md", "docs/knowledge/areas/workflow-state/dispatch.md", "docs/knowledge/areas/hook-runtime/the-intent-anchor-and-compaction-survival.md", "docs/history/codex-harness-hardening/release-manifest.json", ".agents/skills/bee-planning/**", ".claude/skills/bee-planning/**", ".codex/skills/bee-planning/**", ".opencode/skills/bee-planning/**", ".pi/skills/bee-planning/**", ".bee/bin/prompts/**"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", ".bee/verify/verify-app/features/README.md", ".bee/verify/verify-app/features/feature-gates.md", ".bee/verify/verify-app/features/cells-and-proof.md"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md", "docs/knowledge/areas/workflow-state/dispatch.md", "docs/knowledge/areas/hook-runtime/the-intent-anchor-and-compaction-survival.md"],
    "action": "Drive the Pi plugin against a throwaway onboarded repository. Cover session identity, early write denial, gate packet preview, correct intent and purpose, replayable proof, and concurrent dispatch. Update the existing verification maps and knowledge homes only where behavior changed.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev regen && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": ["The Pi end-to-end path passes each acceptance item that Claude Code already supports", "Generated files and the release manifest are current", "Verification maps name how a user drives each changed feature"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "drives the complete Pi lifecycle path"}],
      "key_links": ["verification maps point to executable CLI and Pi contract tests"],
      "prohibitions": ["Do not create a second knowledge page for an existing fact", "Do not claim live proof from unit tests alone"]
    },
    "behavior_change": true
  },
  {
    "id": "pihp-8",
    "feature": "pi-harness-workflow-parity",
    "title": "Keep the full suite isolated under Pi",
    "lane": "high-risk",
    "role": "test",
    "deps": ["pihp-1", "pihp-4", "pihp-7"],
    "decisions": ["D1", "D6", "agents-never-build-on-red"],
    "files": ["packages/bee-rs/crates/bee/src/session_identity.rs", "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "packages/bee-rs/crates/bee/src/catalog.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs", "packages/bee-rs/crates/bee/tests/concurrency.rs", "packages/bee-rs/crates/bee/tests/workflow_verbs.rs", "packages/bee-rs/crates/bee/tests/registry_dispatch.rs", "packages/bee-rs/crates/bee/tests/registry_contracts.rs", "packages/bee-rs/crates/bee/tests/hook_contracts.rs", "packages/bee-rs/crates/bee/tests/discovery_verbs.rs", "packages/bee-rs/crates/bee/tests/cells_archive_sweep.rs", "packages/bee-rs/crates/bee/tests/revision_deadlock_visibility.rs", "packages/bee-rs/crates/bee/tests/route_lane_targeting.rs"],
    "read_first": ["docs/history/pi-harness-workflow-parity/CONTEXT.md", "packages/bee-rs/crates/bee/src/session_identity.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs", "packages/bee-rs/crates/bee/tests/workflow_verbs.rs", "packages/bee-rs/crates/bee/tests/registry_contracts.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep product Pi session lookup unchanged. Capture the ambient Pi test-runner identity without a first-call race. Isolate every sessionless black-box subprocess in the failed targets and shared fixture set unless a test supplies its own identity. Align proof and gate fixtures with the approved exact-proof and preview contracts. Declare preview in both gate registry entries and extend deletion coverage. Re-run the failed dispatch test and edit it only if identity isolation does not repair it. Make the previously red full suite pass.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": ["Two consecutive full-suite runs pass when invoked from live Pi sessions", "Product commands still resolve PI_SESSION_ID after Bee and Claude identities", "Unit and black-box fixtures ignore only inherited Pi identity without first-call order dependence", "Proof and gate fixtures obey the new product contracts", "Both gate registry entries declare preview"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/tests/workflow_verbs.rs", "substantive": "removes ambient Pi identity from isolated black-box commands"}],
      "key_links": ["test command helpers remove only inherited PI_SESSION_ID while Pi-specific integration tests set their own value", "registry payload matches run_gate accepted flags", "fixture proof commands and gate previews match their production contracts"],
      "prohibitions": ["Do not remove PI_SESSION_ID from production lookup", "Do not weaken the full-suite command", "Do not bypass exact proof or gate preview", "Do not hide command failures"]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass condition |
|---|---|---|---|
| User types | Not applicable. Session identity is runtime-owned, not role-owned. | pihp-1 | No role branch exists. |
| Input extremes | Test missing, blank, malformed, Unicode, and conflicting session ids and proof commands. | pihp-1, pihp-3 | Each case uses the specified precedence or refuses without a write. |
| Timing | Start allocations within one millisecond and repeat preview after a plan revision. | pihp-4, pihp-6 | Job ids differ and stale previews refuse. |
| Scale | Parse zero, one, and seven preview cells. | pihp-4 | Zero refuses, one and seven preserve every packet. |
| State transitions | Exercise exploring, planning, swarming, revoked execution, capped, and reopened states. | pihp-3, pihp-4, pihp-5 | Each state permits or denies exactly as CONTEXT.md specifies. |
| Environment | Run Pi-only, Claude-only, Bee-only, mixed, absent, and ambient test-runner session environments. | pihp-1, pihp-7, pihp-8 | Product precedence stays ordered, and isolated fixtures do not inherit the outer Pi session. |
| Error cascades | Make intent, plan, and workflow state missing or corrupt. | pihp-2, pihp-4 | The command fails closed where attribution is required and writes no partial state. |
| Authorization | Treat the feature and worktree identity as the write authority. | pihp-2, pihp-5 | Another feature cannot grant intent or write access. |
| Data integrity | Interrupt preview or cap validation before the store write. | pihp-3, pihp-4 | Cell and workflow JSON stay unchanged on refusal. |
| Integration | Drive Pi event translation into the shared Rust guard and CLI. | pihp-5, pihp-7 | Pi and Claude-shaped inputs produce matching decisions. |
| Compliance | Keep prompts and diagnostics free of raw secret or full session values. | pihp-1, pihp-7 | Tests assert that output does not expose the raw id beyond existing approved records. |
| Business logic | Check each precedence boundary and exact proof equality. | pihp-1, pihp-3 | One-under, exact, and one-over cases have deterministic results. |

## Plan consult synthesis

Two configured herding seats returned. Three native Agent seats were unavailable in this Pi session and are recorded as dropped.

The plan accepts one clarification: all non-cell templates use a separate conditional `purpose` variable. Advisor `brief` remains reserved for `--brief-file`.

The plan dismisses the suggested null-verify exception. New cells require a non-empty `verify`, and historical caps stay on the compatible read path.

The plan dismisses automatic preview creation during bypass. Full bypass changes who approves, not which preparation steps run.

## Open questions

None.

## Post-merge repair

Run the original Waggledance `idm-1` verify command. Reopen and cap the cell through the CLI with that exact command in its proof line. Then supersede the broad responsive decision with the later dashboard-specific decision.

## Out of scope

The slice does not change dashboard product code, add another runtime, or migrate all historical caps.
