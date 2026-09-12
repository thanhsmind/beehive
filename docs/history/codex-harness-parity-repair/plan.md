---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Codex harness parity repair

## Summary

Bee will dispatch valid Codex workers and record activity on supported events. Later slices repair remaining suite failures and prove a fresh installed session.

Mode: `high-risk` — audit-security, public-contracts, multi-domain, and covered-contract-change.

Class playbook: `.agents/skills/bee-planning/references/planning-reference.md` ("Class playbooks", `feature`).

## Requirements from CONTEXT.md

- D1 requires equal workflow results.
- D2 requires the live Codex spawn schema.
- D3 requires full-suite and installed-runtime proof.

## Load-bearing claims

Evidence is copied from the named anchors.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The default Codex branch writes the display subject into `task_name`. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1911-1914` | `payload.insert(` / `"task_name".into(),` / `Value::String(one_line(Some(&Value::String(subject.clone())), TASK_NAME_MAX)),` / `);` |
| 2 | The Codex native override emits `agent_type`. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1739-1747` | `tool = "spawn_agent".into();` / `payload.insert(` / `"agent_type".into(),` / `Value::String(if agent_type.is_empty() {` / `"worker".to_string()` / `} else {` / `agent_type.clone()` / `}),` / `);` |
| 3 | The current test requires an invalid task name. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:1100-1104` | `assert_eq!(v.get("tool"), Some(&json!("spawn_agent")));` / `assert_eq!(` / `v.get("payload").unwrap().get("task_name"),` / `Some(&json!("c-1: cap the test scrubber"))` / `);` |
| 4 | The activity group is Claude-only. | read | `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs:116-123` | `/// The agent-activity probe (agent-activity-hook D1). Claude-only in this` / `/// slice: Codex exposes no PostToolUseFailure / PermissionRequest /` / `/// Notification event, and the three events it does share carry no session` / `/// activity record on that runtime yet.` / `const ACTIVITY: Group = group!(CLAUDE_ONLY, None, [("bee-activity.mjs", "bee: activity")]);` |
| 5 | Codex already exposes shared prompt and tool events. | read | `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs:136-181` | `name: "UserPromptSubmit",` / `name: "PreToolUse",` / `name: "PostToolUse",` |

## Discovery

The Pi merge carries purpose propagation, unique herding ids, and ambient-session suite isolation. Codex still has invalid spawn payloads and no activity wiring.

## Approach

See `docs/history/codex-harness-parity-repair/approach.md`.

Waves: `chpr-1` and `chpr-2` are disjoint. Full-suite and release work follow both.

## Shape

This slice repairs dispatch and activity. Exact full-suite output selects later repair files.

## Cells — current slice preview

```json
[
  {
    "id": "chpr-1",
    "feature": "codex-harness-parity-repair",
    "title": "Emit callable Codex spawn payloads",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["f94e2341-3621-4f66-bcab-dfbf9e26490c", "5f4f9c7b-f917-40d4-b9af-5084c7f3cbf1"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "docs/knowledge/areas/workflow-state/dispatch.md"],
    "read_first": ["docs/history/codex-harness-parity-repair/CONTEXT.md", "docs/knowledge/areas/workflow-state/dispatch.md", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/workflow-state/dispatch.md"],
    "action": "Write failing tests for punctuation task names and the native override agent_type key. Add one Codex payload constructor. Emit a lowercase underscore task_name, message, fork_turns, and optional model fields. Put the complete subject and purpose in message. Route all Codex spawn branches through it.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee codex_spawn_payload",
    "must_haves": {
      "truths": ["Every Codex task_name matches ^[a-z0-9_]+$", "No Codex payload contains agent_type", "The complete purpose is present in message", "All Codex spawn paths use one constructor"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "owns one Codex payload constructor"}],
      "key_links": ["all Codex spawn branches call the constructor"],
      "prohibitions": ["Do not change Claude payload fields", "Do not truncate the assignment"]
    },
    "behavior_change": true
  },
  {
    "id": "chpr-2",
    "feature": "codex-harness-parity-repair",
    "title": "Record activity on supported Codex events",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["a4a9ab6a-4dca-4364-8d25-96bd78f704f0"],
    "files": ["packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs", "packages/bee/hooks/hooks.json", ".codex/hooks.json", "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"],
    "read_first": ["docs/history/codex-harness-parity-repair/CONTEXT.md", "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md", "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"],
    "action": "Split activity into shared and Claude-only groups. Add Codex activity to UserPromptSubmit, PreToolUse, PostToolUse, and Stop. Keep PostToolUseFailure, PermissionRequest, Notification, and SessionEnd Claude-only. Update assertions, generated manifests, and the capability record.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee devtools::hook_manifests",
    "must_haves": {
      "truths": ["Codex activity runs on four shared events", "Claude keeps all eight events", "SubagentStop never changes session activity", "Codex contains no unsupported event"],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs", "substantive": "owns shared and Claude-only activity groups"}],
      "key_links": ["both generated Codex manifests match the catalog"],
      "prohibitions": ["Do not add unsupported event names", "Do not weaken doctor attestation"]
    },
    "regen_obligation_ack": "wave-barrier",
    "behavior_change": true
  }
]
```

## Test matrix

| Dimension | Probe | Pass condition |
|---|---|---|
| User types | Every worker role | Each role starts |
| Input extremes | Blank, punctuation, Unicode, long, repeated | Valid name and complete message |
| Timing | Parallel repeated dispatch | Names do not collide |
| Scale | Runtime-kind matrix | Every Codex branch covered |
| State transitions | Prompt, pre-tool, post-tool, stop | Expected activity state |
| Environment | Default, override, escalation, herding | Each contract holds |
| Error cascades | Missing purpose or malformed subject | Deterministic fallback |
| Authorization | Model guard reads returned payload | Prepared call accepted |
| Data integrity | Regenerate Codex manifests | Bytes match catalog |
| Integration | Execute exact prepared calls | Workers start |
| Compliance | Secret/session text excluded from task name | Stable identifier only |
| Business logic | Claude and Codex comparison | Shared results match |

## Open Questions

None for this slice. Full-suite output selects later repair files.

## Out of scope

Pi behavior already merged under `pi-harness-workflow-parity`.
