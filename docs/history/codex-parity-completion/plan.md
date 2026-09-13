---
artifact_contract: bee-plan/v1
mode: high-risk
---
# Plan: Codex parity completion

## Summary

Protect native Codex writes, enforce worker settings, and track each session correctly. Prove the installed runtime and update its instructions.

## Requirements

D1 in CONTEXT.md maps patch protection to cpc-1, dispatch and read-only work to cpc-2, session behavior to cpc-3, and documentation to cpc-4.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Native dispatch currently permits a configured marker | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:720 | Some(Marker::Role(role)) => allow("codex-spawn-marker", Some(role), None, None), |
| 2 | Prepared payload can carry model | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:527 | payload.insert("model".into(), Value::String(model.to_string())); |
| 3 | Transcript discovery starts at Claude storage | read | packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs:72 | let projects_root = claude_projects_root(); |
| 4 | Catalog has shared activity | read | packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs:122 | const ACTIVITY_SHARED: Group = group!(BOTH, None, [("bee-activity.mjs", "bee: activity")]); |

## Discovery

The assessment and three independent advisors traced the current Rust code. The host onboarding projection omits activity present in the source catalog. Native patch handlers exist but the matcher names only Claude tools. The live canary must establish actual names before adapter changes. Official OpenAI subagent documentation confirms inherited sandbox controls; current callable spawn exposes no sandbox override.

## Approach

Use the shared role resolver, one catalog and existing transcript consumers. Normalize native records at their boundaries. Use a restrictive CLI transport when the native child cannot enforce read-only work. Keep requested model distinct from observed model. See approach.md.

Class playbook: bee-planning/references/planning-reference.md, Class playbooks, feature. The smaller-path check rejects a second policy engine or another hook catalog. Reproduce-first and red-before-green require observed failures before fixes. Chesterton's fence requires preserving the reason for old pass-through rules until current evidence supports the change.

## Shape

One complete slice. Run cpc-1 and cpc-3 concurrently because files are disjoint. cpc-2 depends on cpc-1 live inputs. cpc-4 depends on implementation evidence. Shared Cargo output is a scarce resource; serialize build/test processes. Final regeneration is a wave barrier owned by the leader.

## Cells, current slice preview

```json
[
  {
    "id": "cpc-1",
    "feature": "codex-parity-completion",
    "title": "Protect native Codex tools through installed hooks",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "ee16df03-e88d-4f5e-9f64-400114d9d234"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs",
      "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs",
      "packages/bee-rs/crates/bee/src/hooks/adapter.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/tests/hook_contracts.rs",
      "packages/bee/hooks/hooks.json",
      "packages/bee/hooks/claude-hooks.json",
      ".codex/hooks.json",
      "scripts/codex-parity-canary.sh"
    ],
    "read_first": [
      "docs/history/codex-parity-completion/CONTEXT.md",
      "docs/history/codex-parity-completion/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md",
      ".bee/verify/verify-app/features/onboard.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"
    ],
    "action": "Per D1, first build a bounded reproducible real-Codex canary in an isolated onboarded repo, capture native tool names and hook inputs without secrets, and reproduce denied/allowed patch and shell behavior before changing guards. Use codex 0.154.0 already installed; never change real user config or hook trust. A per-invocation hook-trust override is allowed only for vetted hooks inside the disposable canary. Fix actual input normalization and matcher gaps; retain gate, privacy, reservation and multi-target/rename checks. Consolidate Codex source/host hook rendering so a newly onboarded host gets shared activity hooks too. Preserve explicit runtime differences and Windows commands. Add regression tests through the installed manifest selection path, not just direct handler calls. Capture actual observed lifecycle events and spawn fields in the canary report for cpc-2/cpc-3. Regenerate owned hook manifests with the newly built binary. Commit only assigned files once. The leader owns state writes and cap for herded execution.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hook_manifests && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks_wiring",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "An installed Codex hook denies forbidden patch and shell writes without changing bytes, and allows an approved write.",
        "Source and onboarded Codex manifests carry the same guard and shared activity behavior.",
        "The live canary reports actual observed names, versions and skipped capabilities without reading secrets."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/adapter.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/hook_contracts.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        }
      ],
      "key_links": [
        "Installed bee commands reach the updated production behavior."
      ],
      "prohibitions": [
        "Do not weaken Claude guards, alter real credentials, or claim unobserved platform support."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "cpc-2",
    "feature": "codex-parity-completion",
    "title": "Enforce native dispatch settings and read-only work",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "cpc-1"
    ],
    "decisions": [
      "ee16df03-e88d-4f5e-9f64-400114d9d234"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "packages/bee/prompts/gather.md",
      "packages/bee/prompts/reviewer.md"
    ],
    "read_first": [
      "docs/history/codex-parity-completion/CONTEXT.md",
      "docs/history/codex-parity-completion/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md",
      "docs/knowledge/areas/hook-runtime/codex-spawn-agent-dispatch-payload-schema-and-schema-agnosti.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"
    ],
    "action": "Per D1, use cpc-1 live evidence and the current callable spawn schema to govern model, reasoning_effort and fork_turns through the same role resolver used by preparation. Replace the known mismatched-override acceptance with red-first regression proof. Prepared model-shaped Codex cell dispatches must carry supported configured settings; full-history forks cannot carry overrides. Refuse wrong direct-call overrides with a named remedy if live Codex cannot apply corrected input. Escalated roles preserve the parent model without overrides; herding/CLI roles cannot escape their selected transport via a native marker. Repair the obsolete native capability delegate path only from observed version-scoped evidence, never by a blanket version guess. Native read-only gather/reviewer/advisor jobs need enforced filesystem restrictions: current spawn has no sandbox field, so use an explicit Codex CLI read-only sandbox for model-shaped non-cell slots when no verified native restriction exists; preserve configured herding/CLI transports and refuse any unenforceable native read-only request by name. Quote argv safely and preserve stdin. Prove a read succeeds and patch/shell writes fail in that sandbox, including attempted continuation. Do not invent custom-agent selection fields absent from the callable schema. Commit assigned source once; leader records cap.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee model_guard && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee verbs::drivers",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Native model/effort/fork mismatches are rejected or repaired by a live-proven host mechanism.",
        "Prepared supported model-shaped dispatches carry the selected settings and retain honest effective-model reporting.",
        "Model-shaped Codex read-only jobs have a real read-only execution boundary; unsupported native requests fail explicitly.",
        "Configured herding/CLI slots and escalation retain their intended transports."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        }
      ],
      "key_links": [
        "Installed bee commands reach the updated production behavior."
      ],
      "prohibitions": [
        "Do not weaken Claude guards, alter real credentials, or claim unobserved platform support."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "cpc-3",
    "feature": "codex-parity-completion",
    "title": "Track Codex session completion from native transcripts",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "ee16df03-e88d-4f5e-9f64-400114d9d234"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/reads.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/activity.rs"
    ],
    "read_first": [
      "docs/history/codex-parity-completion/CONTEXT.md",
      "docs/history/codex-parity-completion/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/agent-activity-record.md",
      "docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"
    ],
    "action": "Per D1, reproduce failure to locate and read a native Codex transcript with actual redacted record shapes. Use the current session record's transcript_path and verified session identity before runtime-specific fallback discovery; never choose another session's newest file. Normalize Codex final assistant messages and usage into existing consumers without double-counting cumulative token totals. Preserve all Claude behavior. Handle commentary-only, tool-only, empty/truncated/malformed records, resumed sessions and concurrent sessions. Update activity only using observed supported Codex events or verified final/turn information; do not manufacture unsupported PermissionRequest/Notification/SessionEnd emissions. Prove correct working/idle/explicit-wait transitions, and that child completion does not end the parent. Read cpc-1 canary observations when available before final live verification. Commit source once; leader records cap.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee session_close && cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee hooks::activity",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Codex final text sets the correct session's turn-end waiting mark.",
        "Codex transcript usage does not double-count cumulative events.",
        "Missing or malformed transcripts cannot borrow another session's data.",
        "Claude transcript and activity behavior remains covered; child completion does not mark parent idle."
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/reads.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/activity.rs",
          "substantive": "Production behavior and regression tests for the assigned outcome."
        }
      ],
      "key_links": [
        "Installed bee commands reach the updated production behavior."
      ],
      "prohibitions": [
        "Do not weaken Claude guards, alter real credentials, or claim unobserved platform support."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  },
  {
    "id": "cpc-4",
    "feature": "codex-parity-completion",
    "title": "Publish verified Codex parity contracts",
    "lane": "high-risk",
    "role": "docs",
    "status": "open",
    "deps": [
      "cpc-2",
      "cpc-3"
    ],
    "decisions": [
      "ee16df03-e88d-4f5e-9f64-400114d9d234"
    ],
    "files": [
      "README.md",
      "docs/06-runtime-integration.md",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md",
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md",
      "docs/knowledge/areas/hook-runtime/native-spawn-and-transport-classification.md",
      "docs/knowledge/areas/hook-runtime/codex-spawn-agent-dispatch-payload-schema-and-schema-agnosti.md",
      "docs/knowledge/areas/hook-runtime/agent-activity-record.md",
      "docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md",
      "skills/bee-swarming/references/swarming-reference.md",
      ".bee/verify/verify-app/features/README.md",
      ".bee/verify/verify-app/features/codex-runtime.md"
    ],
    "read_first": [
      "docs/history/codex-parity-completion/CONTEXT.md",
      "docs/history/codex-parity-completion/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md",
      "skills/bee-swarming/references/swarming-reference.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"
    ],
    "action": "Per D1, reconcile owning knowledge concepts and runtime documentation against cpc-1/2/3 code and actual evidence. Remove false claims that doctor is absent or that all Codex builds lack model selection. Document supported sandbox fallback and retained platform differences precisely. Update only runtime-sensitive swarming reference instructions required by changed dispatch behavior; use bee-writing-skills for that edit and pressure-test relevant scenarios. Add a verification feature recipe for installed Codex guard, dispatch, transcript and lifecycle paths. Mark source/canary/runtime-test evidence separately; never equate doctor attestation with live proof. Use existing concepts rather than creating duplicates. Leader runs regeneration and full-suite integration after this commit.",
    "verify": "git diff --check",
    "regen_obligation_ack": "wave-barrier",
    "must_haves": {
      "truths": [
        "Documentation names the actual supported paths and limits at the tested CLI version.",
        "The verification map drives deny, allow, read-only dispatch and turn completion.",
        "Installed instructions describe the supported dispatch schema and sandbox fallback."
      ],
      "artifacts": [
        {
          "path": "docs/06-runtime-integration.md",
          "substantive": "Current evidence-backed runtime parity reference."
        }
      ],
      "key_links": [
        "Installed bee commands reach the updated production behavior."
      ],
      "prohibitions": [
        "Do not weaken Claude guards, alter real credentials, or claim unobserved platform support."
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "behavior_change": true
    }
  }
]
```

## Test matrix

| Dimension | Probe | Pass when |
|---|---|---|
| User types | Cell versus read-only worker | Only authorized writes change bytes |
| Input extremes | Empty, malformed, Unicode input | Safe explicit outcome |
| Timing | Resume and simultaneous sessions | Events affect the correct session |
| Scale | Repeated usage snapshots | Each increment counts once |
| State transitions | Prompt, tool, child stop, final text | Child stop never ends the parent |
| Environment | New host, worktree, Windows wiring | Same policy or explicit capability limit |
| Error cascades | Hook and CLI failure | Visible failure, never reported as a pass |
| Authorization | Wrong model, effort, fork, transport | Invalid request refused before execution |
| Integrity | Duplicate and truncated records | Existing state remains valid |
| Integration | Real patch and shell calls | Installed guard denies and allows correctly |
| Compliance | Probe capture | No credentials or unrelated transcript content |
| Business rules | Null and explicit slots | Configuration selects the intended behavior |

Workers inspect existing coverage and author only missing behavioral tests. Each new bug reproduction runs before and after the change. Leader runs the declared full suite and the live installed canary. Onboarding map modified 2026-09-07; hook/dispatch paths are not mapped, so cpc-4 adds their recipe. Release regeneration is deferred to the declared wave barrier, never skipped.

## Open Questions

Exact native hook names and lifecycle inputs are a cpc-1 measured output. Unsupported events remain documented platform limits; supported fallbacks require live proof.

## Out of scope

No changes to real credentials, global settings, unrelated backlog, or independent review process.
