---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Pi full workflow parity

## Summary

Pi will finish Bee's workflow tail without a stale handoff blocking the next session. A user can dismiss a pause handoff, close the workflow, and see no active compatibility projection. Planned-next claims remain protected by adoption.

Mode: `high-risk` — 5 risk flags: audit-security, public-contracts, cross-platform, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: one core cell owns the state transition and one dependent cell proves the installed Pi path.

Class playbook: `bugfix`, from `.agents/skills/bee-planning/references/planning-reference.md` ("Class playbooks").

## Requirements (from CONTEXT.md)

- D1: Keep one lifecycle definition across the CLI, Pi plugin, and generated host helpers.
- D2: Support only `@earendil-works/pi-coding-agent`.
- D3: Keep Pi worker dispatch herding-only.
- D4: Dismiss a pause handoff and close the workflow without an active `HANDOFF.json` projection.
- D5: Keep blocking Pi checks fail-closed and advisory checks fail-open.
- D6: Require semantic parity or a tested named exclusion for each host capability.

## Load-bearing claims

Labels are `read`, `ran`, or `guessed`; evidence must match its anchor exactly.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Projection rebuild currently selects open mailbox records without excluding closed workflows. | read | `packages/bee-rs/crates/bee/src/verbs/workflow_store/projections.rs:287` | `.filter(\|r\| matches!(r.get("status"), Some(Value::String(s)) if s == "open"))` |
| 2 | Every workflow close selector currently changes only the workflow status at its mutation point. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:148` | `patch.insert("status".into(), json!("closed"));` |
| 3 | Existing adoption intentionally rejects every handoff that is not planned-next. | read | `packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs:420` | `if !matches!(candidate.get("kind"), Some(Value::String(s)) if s == "planned-next") {` |
| 4 | Mailbox records must remain on disk for audit history after they clear. | read | `docs/knowledge/areas/workflow-state/handoff.md:94-96` | `different role's, or a different workflow's, handoff. Every record — cleared / or not — stays on disk under its own sequence number for audit history, / never deleted. `bee`'s `state handoff write/show/adopt` verbs resolve` |
| 5 | The current Pi sandbox lifecycle ends its declared coverage at concurrent dispatch allocation. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:5642` | `/// - collision-safe concurrent dispatch job id allocation` |
| 6 | The CLI registry payload is hand-maintained because no regeneration chain exists. | read | `docs/decisions/index.md:2013` | `- 3358743e · 2026-08-05 · worktree-reclaim D5: packages/bee-rs/crates/bee/src/generated/registry_payload.json is hand-edited in this repo, with the reason recorded, because its declared regen chain does not exist here. Every cell that adds or changes a CLI command edits the payload directly and re-runs tests/registry_contracts.rs plus tests/registry_dispatch.rs as the proof.` |

## Discovery

The real mailbox writer keeps append-only records and clears by status. The close verb writes `status: closed` but does not touch handoffs or rebuild their projection. The real Pi sandbox already onboards a repository and drives gates, cells, proof, and dispatch, but it does not drive handoff dismissal or workflow close.

Feature map checked: `.bee/verify/verify-app/features/README.md`, last modified 2026-09-13. It has no matching feature file, so this plan treats the workflow tail as a new mapped path.

## Approach

Use `docs/history/pi-full-workflow-parity/approach.md`. The command is pause-only and keeps mailbox history. Close refuses to erase planned-next authority, clears pause state through one helper, and rebuilds a projection that cannot select a closed workflow.

Waves: pfp-1 runs first because it defines the public command and state behavior. pfp-2 follows because its sandbox path calls that command and verifies the close result. The serial edge is a real interface dependency.

SMALLER PATH: no — filtering closed workflows alone hides the symptom but leaves an open authoritative record; deleting only the projection is recreated by the next rebuild.

## Shape

Feature outcome: Pi completes the user-visible workflow tail while preserving mailbox history and planned-next claim fencing.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Safe handoff completion | Pause dismissal, close integration, projection defense, public command | The confirmed stale pause survives workflow close and blocks orientation. | Current slice, pfp-1 | Red-then-green state tests plus registry contracts. |
| Installed Pi parity | Onboarded repository lifecycle tail | In-process tests do not prove the installed Pi and CLI path. | Current slice, pfp-2 | Exact Pi sandbox test reaches dismiss, close, orient, and absent projection. |

Current slice: both epics. This slice contains all required work.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pfp-1 | Add safe pause dismissal and close cleanup | workflow-store, state-group, workflow tests, registry, existing knowledge owners | — | Closing a workflow cannot leave a pause handoff active, while planned-next stays protected. | Handoff-focused tests and registry contract suites pass. |
| pfp-2 | Prove the complete Pi workflow tail | `pi_plugin_contracts.rs` | pfp-1 | The real onboarded Pi sandbox dismisses, closes, orients, and leaves no active projection. | Exact Pi lifecycle end-to-end test passes. |

```json
[
  {
    "id": "pfp-1",
    "feature": "pi-full-workflow-parity",
    "title": "Add safe pause dismissal and close cleanup",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["0510d81d-24e9-46f7-bf53-701cfaccf978"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/projections.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/tests/workflow_verbs.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "docs/decisions/cli-provenance.md",
      "docs/knowledge/areas/workflow-state/handoff.md",
      "docs/knowledge/areas/workflow-state/workflow-records-and-projections.md"
    ],
    "read_first": [
      "docs/history/pi-full-workflow-parity/CONTEXT.md",
      "docs/history/pi-full-workflow-parity/plan.md",
      "docs/knowledge/areas/workflow-state/handoff.md",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/tests/workflow_verbs.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/workflow-state/handoff.md",
      "docs/knowledge/areas/workflow-state/workflow-records-and-projections.md"
    ],
    "action": "Apply red-before-green. Add a public `bee state handoff dismiss` command that marks only the selected open pause record cleared, keeps its mailbox file, rebuilds the compatibility projection, and supports the C1 legacy path. Refuse a planned-next record without changing its claim or mailbox bytes. Route the new command through the state dispatcher and hand-maintained registry. Refactor all workflow close selectors through one guarded helper. Before any multi-record close mutation, preflight every target and refuse the whole operation if one has open planned-next authority. Then clear open pause records, close the records, and rebuild the projection. Make projection rebuild ignore closed workflows as crash-recovery defense. Update the existing knowledge owners and CLI provenance. Preserve D1-D6 and acceptance 0510d81d.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml handoff && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts --test registry_dispatch",
    "must_haves": {
      "truths": [
        "A pause handoff dismissal marks the mailbox record cleared and removes the active legacy projection.",
        "A planned-next dismissal refuses and preserves both the handoff and its claim.",
        "Every workflow close selector leaves no active pause projection and cannot silently discard planned-next authority; a multi-record refusal changes no target workflow.",
        "Projection rebuild never selects a closed workflow.",
        "The public command appears in help and dispatches through the native Rust path."
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs", "substantive": "pause-only mailbox dismissal primitive with history preservation"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs", "substantive": "public handoff dismiss handler and C1 path"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs", "substantive": "shared close transition that coordinates handoff cleanup"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "complete hand-maintained command declaration and runnable example"}
      ],
      "key_links": [
        "set_gate.rs dispatches state handoff dismiss to sessions.rs",
        "workflow close uses the mailbox cleanup primitive before reporting success",
        "all mailbox mutations rebuild the single compatibility projection",
        "knowledge pages describe the same state transitions the tests assert"
      ],
      "prohibitions": [
        "Do not delete mailbox history.",
        "Do not clear or adopt a planned-next handoff through dismiss.",
        "Do not change Pi dispatch from herding-only.",
        "Do not add OMP support.",
        "Do not weaken Pi blocking or advisory failure policy."
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pfp-2",
    "feature": "pi-full-workflow-parity",
    "title": "Prove the complete Pi workflow tail",
    "lane": "high-risk",
    "role": "test",
    "deps": ["pfp-1"],
    "decisions": ["0510d81d-24e9-46f7-bf53-701cfaccf978"],
    "files": [
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-full-workflow-parity/CONTEXT.md",
      "docs/history/pi-full-workflow-parity/plan.md",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      ".pi/extensions/bee-guard.ts"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Extend `pi_lifecycle_end_to_end_onboarded_repo_parity` after its current cap and dispatch checks. Through the installed `bee_target`, write a pause handoff, prove it is projected, dismiss it, close the workflow, run `bee orient --json`, and assert no handoff blocker or `.bee/HANDOFF.json` remains. Also drive a planned-next dismissal refusal with an owned claim and assert the claim and mailbox record remain unchanged. Keep the configured Pi transport herding-only and preserve all existing belt failure-policy checks. Apply test-behavior-not-structure and acceptance 0510d81d.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts pi_lifecycle_end_to_end_onboarded_repo_parity -- --exact --nocapture",
    "must_haves": {
      "truths": [
        "The onboarded Pi sandbox completes pause dismissal and workflow close through the installed CLI.",
        "Orient reports no stale handoff blocker after close.",
        "The compatibility HANDOFF.json file is absent after the workflow tail.",
        "The sandbox proves planned-next dismissal refusal without claim loss."
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "end-to-end Pi workflow-tail assertions in the existing sandbox test"}
      ],
      "key_links": [
        "The Pi sandbox invokes the same installed binary created by onboarding.",
        "The final assertions inspect both orient output and the compatibility file."
      ],
      "prohibitions": [
        "Do not replace the end-to-end path with direct state-file writes for the operation under test.",
        "Do not add a native Pi worker transport or OMP target.",
        "Do not relax any existing Pi extension assertion."
      ]
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| User types | Same command path for any local CLI actor; no authorization branch applies. | pfp-1 | Pass when no actor-specific state branch exists. |
| Input extremes | No handoff, one pause, duplicate dismiss, and multiple role slots. | pfp-1 | Pass when no handoff and duplicate calls refuse clearly, while the selected role alone clears. |
| Timing | Hold workflow and handoff locks in the documented order and rebuild after mutation. | pfp-1 | Pass when concurrent-safe helpers use the existing locks and tests complete without deadlock. |
| Scale | Closed and active workflows coexist with open handoffs. | pfp-1 | Pass when projection selects only an open handoff from an active workflow. |
| State transitions | Open pause to cleared; open planned-next to refusal; active workflow to closed. | pfp-1 | Pass when each status and refusal matches literally and all non-target bytes remain unchanged. |
| Environment | Workflow mailbox and C1 legacy repository paths. | pfp-1 | Pass when both paths dismiss pause safely and planned-next still refuses. |
| Error cascades | Projection rebuild follows the authoritative mutation. | pfp-1 | Pass when successful output occurs only after projection rebuild succeeds. |
| Authorization | No remote user or permission boundary applies. | pfp-1 | Pass when the command adds no authorization semantics. |
| Data integrity | Mailbox audit history, planned-next claim fence, and all-or-none preflight for multi-record close. | pfp-1 | Pass when cleared records stay on disk, refused claim bytes stay unchanged, and one unsafe target prevents every close mutation. |
| Integration | Dispatcher, registry, close selectors, Pi onboarding, and orient. | pfp-1, pfp-2 | Pass when registry suites and the exact Pi sandbox test pass. |
| Compliance | No new personal data or logs. | pfp-1 | Pass when the new record fields contain only status and timestamps. |
| Business logic | Zero, one, and many workflow records; active versus closed. | pfp-1 | Pass when zero uses C1, one clears correctly, and closed records cannot project. |

The changed scenario runs on main before the fix through the new red tests, then on head through the same commands.

## Open Questions

(none)

## Out of scope

- OMP support.
- Native Pi worker dispatch.
- Retirement of the legacy `HANDOFF.json` projection.
- Changes to Pi belt event mapping or failure policy.
