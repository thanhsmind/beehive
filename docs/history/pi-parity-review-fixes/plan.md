# Pi parity review fixes — implementation plan

## Outcome

Resolve all four findings from `pi-harness-current-parity-review-20260913` without adding a second Pi transport.

A Pi session must receive Pi dispatch guidance before and after compaction. Its documented lifecycle events must update activity state. Workflow closure must preserve planned-next authority under concurrent writes. `bee doctor --runtime pi` must report Pi readiness.

## Locked scope

The plan implements D1–D8 in `docs/history/pi-parity-review-fixes/CONTEXT.md`.

It supports `@earendil-works/pi-coding-agent` 0.84.x. It does not add OMP support. It does not change ask/repair behavior. It keeps blocking hooks fail closed and advisory hooks fail open.

This work follows the `bugfix` class playbook in `.agents/skills/bee-planning/references/planning-reference.md` under “Class playbooks.” Each repair starts with a compiling behavior test that fails for the reported reason.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The normal session preamble always publishes Claude dispatch guidance. | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:650` | `lines.extend(crate::hooks::model_guard::dispatch_door_lines(Some(&config), "claude"));` |
| 2 | The compact capsule repeats the same hard-coded Claude runtime. | read | `packages/bee-rs/crates/bee/src/hooks/compaction.rs:1527` | `.push(crate::hooks::model_guard::dispatch_door_lines(Some(&config), "claude"));` |
| 3 | The Pi extension does not identify its runtime when it invokes session-init. | read | `.pi/extensions/bee-guard.ts:1632-1637` | `const text = runAdvisoryHook(directory, "session-init", {` |
| 4 | Pi currently claims there is no advisory pre-tool carrier. | read | `.pi/extensions/bee-guard.ts:70-72` | `PreToolUse          -> NAMED EXCLUSION: no honest advisory carrier on Pi` |
| 5 | Pi 0.84.x documents a pre-tool event that fires before `tool_call`. | read | `/home/thanhsmind/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:651-673` | `tool_execution_start` |
| 6 | Bee already maps before-tool activity to `working`. | read | `packages/bee-rs/crates/bee/src/hooks/activity.rs:124-125` | `"UserPromptSubmit" | "PreToolUse" | "PostToolUse" | "PostToolUseFailure" => {` |
| 7 | Workflow close checks planned-next authority before it takes the locks used by later mutations. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:111-132` | `// 1. Preflight all targets for open planned-next authority` |
| 8 | The mailbox writer takes only the handoff lock after its prerequisite reads. | read | `packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs:272` | `let _lock = acquire_named_lock(root, &format!("handoff:{wf_id_s}"))?;` |
| 9 | A current test permits a handoff write against a closed workflow. | read | `packages/bee-rs/crates/bee/src/verbs/workflow_store/tests.rs:1694-1704` | `ok(write_mailbox_handoff(tmp.path(), "wf-closed", &input, None));` |
| 10 | Doctor recognizes only Claude and Codex. | read | `packages/bee-rs/crates/bee/src/doctor.rs:49-52` | `enum Runtime {` |
| 11 | The existing herding readiness probe already reports the configured transport and environment. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1075-1078` | `pub(crate) fn herding_transport_probe_for(` |
| 12 | Pi and OpenCode extensions are checked-in TypeScript belts, not rendered hook manifests. | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:530-547` | `"pi" => return None,` |
| 13 | The verification map has no Pi runtime recipe. | read | `.bee/verify/verify-app/features/README.md:48-67` | `Runtime parity across supported runtimes (Claude, Codex, OpenCode, and Pi)` |
| 14 | The current Pi extension contract suite passes before repair. | ran | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` | `61 Pi tests passed` |
| 15 | Bee already embeds release artifacts with compile-time `include_str!` paths. | read | `packages/bee-rs/crates/bee/src/version.rs:32` | `const PLUGIN_MANIFEST: &str = include_str!("../../../../../.claude-plugin/plugin.json");` |
| 16 | Pi 0.84.4 emits compaction events without starting a replacement session. | read | `/home/thanhsmind/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:331-334` | `/compact or auto-compaction` |

## Faults and repairs

### F1 — runtime-aware injected dispatch

Keep existing default-claude builders for callers that have no runtime context. Add runtime-aware builder paths for session-init.

The Pi extension will send `runtime: "pi"` in its session-init payload. Session-init will accept the dispatch runtime set `claude`, `codex`, and `pi`. Missing, null, non-string, and unknown values use `claude`, which preserves all current callers and prevents arbitrary command text. Both the normal preamble and compact capsule will receive the selected runtime in the same session-init call.

Pi 0.84.4 does not emit a new `session_start` after compaction. Therefore, this repair does not invent a second Pi compaction carrier. It makes the compact-capsule branch runtime-correct whenever session-init receives `source: "compact"`, as Claude-compatible hook callers already do.

Contract tests will cover the complete runtime matrix. They will read both a normal Pi preamble and a Pi compact capsule, extract each literal `dispatch prepare` command, execute it without adding `--runtime pi`, and require a Bash/herding payload resolved from `team.pi`. Companion assertions will keep absent and explicit Claude input on `team.claude`, accept explicit Codex input, and make malformed input fall back to Claude.

### F2 — honest Pi activity carriers

Register advisory, fail-open handlers for Pi 0.84.x lifecycle events.

- `tool_execution_start` maps to Bee `PreToolUse` activity with the mapped tool name and Pi tool-call identifier.
- `ui_prompt_start` maps to Bee `Notification` with `agent_needs_input`.
- The extension tracks UI-prompt depth. Only the outermost start emits the wait event. Only the matching outermost `ui_prompt_end` maps to Bee `UserPromptSubmit` without a `prompt` field. This ends the wait span, returns activity to `working`, and does not add a work-record turn.

An end without a start is ignored. A start without an end remains `waiting_input` until the normal `Stop` boundary. Nested or duplicate starts do not end the wait on the first inner end. A tool event with no identifier still reports its mapped tool name; an unknown or missing tool name follows the existing fail-safe mapping and includes no tool input in activity data.

The existing `tool_call` handler remains the only fail-closed write-guard path. Generic Pi UI prompts do not become typed permission requests. Documentation will remove only the `PreToolUse` and `Notification` exclusions. It will keep the typed `PermissionRequest` exclusion.

### F3 — linearized workflow close and handoff writes

Use one lock order: `workflow:<id>` before `handoff:<id>`.

For multi-workflow close, normalize and sort unique workflow IDs, acquire both locks for every target in stable order, and hold all guards through planned-next preflight, pause cleanup, and closed-state writes. Use assuming-lock helpers so no nested lock acquisition occurs. Rebuild projections after releasing the transaction locks.

The mailbox writer will take the workflow lock first and the handoff lock second. It will then re-read the workflow under lock. A missing, unreadable, or closed workflow will return `Err2::Msg` before mailbox mutation. The message will name the workflow and state that no handoff was written. The CLI will preserve its normal non-zero thrown-command behavior. Existing C1 behavior remains for the separate no-workflow fallback path.

Two red tests make the lock ownership explicit. A writer test holds `workflow:<id>`, starts a handoff write, changes the workflow to closed with `update_workflow_assuming_lock`, and releases the lock. The writer must wait, then refuse, with unchanged mailbox history and projection. A public close-versus-write test starts both operations together and accepts only two linearized outcomes: the handoff wins and close refuses while authority stays visible, or close wins and the handoff refuses with no mailbox write. No test-only close API is added.

The matrix also preserves active pause and planned-next writes, close refusal on an existing planned-next record, pause cleanup during close, and the separate no-workflow C1 fallback. Missing, unreadable, and closed records all refuse without mutation.

### F4 — Pi doctor

Add `pi` to doctor runtime parsing. Pi uses `.pi/extensions/bee-guard.ts` and `.agents/skills`.

Doctor will embed `.pi/extensions/bee-guard.ts` with a compile-time `include_str!`, following `version.rs` and the Pi contract harness. The extension row will compare the installed bytes with that exact compiled reference. This requires no rendered Pi manifest and no new generation path.

Pi doctor has these required rows:

| Row | `ok` | `not_ok` | `unknown` |
|---|---|---|---|
| `hooks_file` | Installed Pi extension is readable. | Extension is absent. | Read fails for a reason other than absence. |
| `hook_handler` | Vendored Bee binary resolves. | Binary is absent. | — |
| `skills_installed` | `.agents/skills` has at least one skill directory. | Directory is absent or empty. | Directory cannot be enumerated. |
| `wiring_matches_binary` | Installed extension bytes equal the compiled extension bytes. | Bytes differ or extension is absent. | Installed bytes cannot be read. |
| `binary_freshness` | Installed binary version matches `.claude-plugin/plugin.json`; source checkout inputs are not newer when available. | Version differs, is absent, or source inputs are newer. | Version or source freshness cannot be probed. |
| `herding_transport` | Configured transport and its required pane environment are present. | Required pane environment is absent. | Transport configuration cannot be parsed or probed. |

The herding row will reuse `transport_kind_at` and `herding_transport_probe_for`, so doctor and dispatch prepare have one readiness rule. Pi host repositories will compare binary release version even when source-tree timestamp checks do not apply.

The JSON report keeps the existing row shape: `row`, `status` (`ok`, `not_ok`, or `unknown`), and `detail`. Any `not_ok` or `unknown` required row gives `overall_status: "blocked"` and exit code 1. All required rows at `ok` give `ready` and exit code 0. `attestation` remains null because attestation is Codex-only.

The verification map will add one Pi runtime recipe for the injected dispatch command, lifecycle carriers, doctor success, and each doctor refusal state.

## Edge coverage

| Dimension | Required proof |
|---|---|
| Input | Unknown or wrong-typed session runtime cannot inject an arbitrary dispatch label. |
| Timing | A late handoff writer cannot pass between close preflight and closed-state persistence. |
| State transition | A closed workflow rejects pause and planned-next writes without changing mailbox history or projection. |
| Environment | Pi doctor reports absent extension, absent skills, stale binary, malformed transport configuration, and missing pane environment without guessing ready. |
| Error cascade | New Pi activity handlers swallow advisory hook failures; the blocking `tool_call` path keeps its current denial behavior. |
| Integration | Both normal and compact Pi injections select `team.pi`; Claude stays on `team.claude`; Pi 0.84.x event payloads drive the real extension harness. |
| Reversibility | The changes add runtime branches and shared locking. No state migration or destructive rewrite is required. |
| Observability | Doctor rows name the failed Pi artifact or transport fact. Activity records show the mapped Bee event and resulting state. |
| Security | Runtime input is allowlisted. Activity payloads include identifiers and tool names, not tool input or UI response content. |

## Waves and cells

### Wave 1 — parallel

1. `pprf-1` — Make injected dispatch guidance runtime-aware and prove preamble-derived Pi dispatch.
2. `pprf-3` — Linearize workflow close and mailbox handoff writes.
3. `pprf-4` — Add fail-closed Pi doctor reporting and its user-driving map.

These cells have disjoint write sets.

### Wave 2 — after `pprf-1`

4. `pprf-2` — Wire Pi 0.84.x pre-tool and UI-prompt activity events.

This cell follows `pprf-1` because both edit `.pi/extensions/bee-guard.ts` and `pi_plugin_contracts.rs`.

## Cells — current slice (preview)

```json
[
  {
    "id": "pprf-1",
    "feature": "pi-parity-review-fixes",
    "title": "Select the runtime in injected dispatch guidance",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["7da86fdb-40de-415c-b3ac-ae84a4c89d60"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/compaction.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-parity-review-fixes/CONTEXT.md",
      "docs/history/pi-parity-review-fixes/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/hooks/compaction.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md"],
    "regen_obligation_ack": "pprf-2 owns the final serialized Pi extension bytes and runs the release-manifest regeneration after both extension edits",
    "action": "Apply red-before-green for F1 and D1-D2. Add compiling tests that show Pi session-init renders Claude dispatch guidance on both normal and compact branches. Keep existing default builders compatible, add runtime-aware paths, allow only the dispatch runtime set, and make malformed or absent runtime input fall back to Claude. Send runtime pi from the Pi extension. In the real Pi contract harness, extract each dispatch prepare command from normal and compact injected text, execute it without adding a runtime flag, and require the team.pi Bash/herding payload. Assert explicit and default Claude remain Claude and explicit Codex remains Codex. Do not change Pi transport.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_init && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts injected",
    "must_haves": {
      "truths": [
        "Normal and compact Pi session-init output publish dispatch prepare --runtime pi.",
        "Commands extracted from both injected outputs execute as team.pi herding payloads without test-added runtime flags.",
        "Absent, null, non-string, and unknown runtime values cannot inject command text and preserve Claude behavior.",
        "Pi remains herding-only and receives no Agent-tool route."
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_init.rs", "substantive": "allowlisted runtime propagation into both injected builders"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs", "substantive": "runtime-aware normal preamble rendering"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/compaction.rs", "substantive": "runtime-aware compact capsule rendering"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "preamble-derived command execution for Pi"}
      ],
      "key_links": [
        "bee-guard.ts sends runtime pi to session-init",
        "session-init passes one selected runtime to normal and compact builders",
        "both builders use model_guard::dispatch_door_lines",
        "the test executes the command parsed from injected output"
      ],
      "prohibitions": [
        "Do not add a Pi Agent tool or second worker transport.",
        "Do not hand-write --runtime pi in the proof command.",
        "Do not accept arbitrary runtime strings into rendered command text.",
        "Do not add OMP support."
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pprf-2",
    "feature": "pi-parity-review-fixes",
    "title": "Carry Pi pre-tool and user-wait activity",
    "lane": "high-risk",
    "role": "code",
    "deps": ["pprf-1"],
    "decisions": ["7da86fdb-40de-415c-b3ac-ae84a4c89d60"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs",
      "docs/config-reference.md",
      "docs/knowledge/areas/hook-runtime/agent-activity-record.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-parity-review-fixes/CONTEXT.md",
      "docs/history/pi-parity-review-fixes/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/agent-activity-record.md",
      "/home/thanhsmind/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/src/hooks/activity.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/agent-activity-record.md"],
    "action": "Apply red-before-green for F2 and D4. Add real-extension harness tests for tool_execution_start, ui_prompt_start, and ui_prompt_end before editing the belt. Map tool start to advisory PreToolUse with identifier and mapped tool name but no input content. Track nested UI-prompt depth: only outer start enters waiting_input, only matching outer end emits a prompt boundary without prompt text and returns to working, unmatched end is ignored, and an unended start remains waiting until Stop. Prove advisory failure is fail-open and tool_call stays fail-closed. Remove only the false PreToolUse and Notification exclusions. Keep typed PermissionRequest and SubagentStop exclusions. Update the existing config section and activity knowledge home. Run the full bee dev regen chain after the final extension edit and check the release manifest.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts activity && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts pi_belt && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "Pi tool_execution_start records working PreToolUse activity before the blocking tool_call path.",
        "Pi UI prompt start and matching outer end form a waiting_input to working span.",
        "Nested, missing-field, duplicate, unmatched, and unended event cases keep honest state.",
        "Every new activity handler fails open while the existing write guard still fails closed.",
        "Documentation names only capability gaps that remain true."
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "Pi 0.84.x activity carriers and prompt-depth state"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "event-order, state, payload, and fail-open contract proof"},
        {"path": "docs/config-reference.md", "substantive": "correct Pi lifecycle carrier table"},
        {"path": "docs/knowledge/areas/hook-runtime/agent-activity-record.md", "substantive": "durable Pi carrier behavior"}
      ],
      "key_links": [
        "Pi tool_execution_start calls Bee activity separately from tool_call write-guard",
        "ui_prompt_start and ui_prompt_end drive the existing Bee activity state machine",
        "contract tests invoke the checked-in extension through HARNESS_JS",
        "config and knowledge text match registered Pi events"
      ],
      "prohibitions": [
        "Do not route write-guard through an advisory event.",
        "Do not store tool input or UI response content in activity records.",
        "Do not call generic UI prompts PermissionRequest.",
        "Do not weaken advisory fail-open behavior."
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pprf-3",
    "feature": "pi-parity-review-fixes",
    "title": "Serialize workflow close with handoff writes",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["7da86fdb-40de-415c-b3ac-ae84a4c89d60"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/tests/workflow_verbs.rs",
      "docs/knowledge/areas/workflow-state/handoff.md",
      "docs/knowledge/areas/workflow-state/workflow-records-and-projections.md"
    ],
    "read_first": [
      "docs/history/pi-parity-review-fixes/CONTEXT.md",
      "docs/history/pi-parity-review-fixes/plan.md",
      "docs/knowledge/areas/workflow-state/overview.md",
      "docs/knowledge/areas/workflow-state/handoff.md",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs",
      "packages/bee-rs/crates/bee/tests/workflow_verbs.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/workflow-state/handoff.md",
      "docs/knowledge/areas/workflow-state/workflow-records-and-projections.md"
    ],
    "action": "Apply red-before-green for F3 and D3. First add a compiling writer test that holds workflow:<id>, starts a mailbox writer, closes through update_workflow_assuming_lock, then proves the writer waited and refused with mailbox history and projection unchanged. Add a public close-versus-write invariant test that accepts only the two linearized outcomes. Then make all mailbox writes and workflow closes acquire workflow:<id> before handoff:<id>. Sort and deduplicate multi-close targets, hold all locks through planned-next preflight, pause cleanup, and status writes, and use assuming-lock helpers to avoid reacquisition. Under the writer locks, fail closed on missing, unreadable, or closed workflow records with a message that names the id and says no handoff was written. Preserve active pause/planned-next writes, all-or-none close, audit history, projection rebuild, and C1 fallback.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml handoff && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test workflow_verbs",
    "must_haves": {
      "truths": [
        "Workflow close and mailbox writes for one workflow linearize under workflow then handoff locks.",
        "A planned-next handoff either remains visible on an active workflow or is refused after closure.",
        "Missing, unreadable, and closed workflow targets reject mailbox writes without mutation.",
        "Multi-target close remains all-or-none and deadlock-safe through stable lock ordering.",
        "Pause cleanup, mailbox audit history, projection rebuild, and C1 fallback remain intact."
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs", "substantive": "stable multi-record lock transaction for close"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/workflow_store/handoff.rs", "substantive": "workflow-first mailbox lock and post-lock state validation"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/workflow_store/tests.rs", "substantive": "deterministic lock and closed-state behavior tests"},
        {"path": "packages/bee-rs/crates/bee/tests/workflow_verbs.rs", "substantive": "public close-versus-write linearization proof"}
      ],
      "key_links": [
        "close and writer share workflow:<id> then handoff:<id>",
        "close uses assuming-lock mailbox cleanup and workflow update helpers",
        "writer validates workflow state after both locks are held",
        "projection rebuild runs only after the locked mutations settle"
      ],
      "prohibitions": [
        "Do not clear, hide, or overwrite planned-next authority.",
        "Do not acquire handoff before workflow for the same operation.",
        "Do not add timing sleeps as the race oracle.",
        "Do not delete mailbox audit files."
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pprf-4",
    "feature": "pi-parity-review-fixes",
    "title": "Report Pi runtime health fail closed",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["7da86fdb-40de-415c-b3ac-ae84a4c89d60"],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      ".bee/verify/verify-app/features/README.md",
      ".bee/verify/verify-app/features/pi-runtime.md",
      "docs/product-description/observability/status.md",
      "docs/product-description/verification/areas.md",
      "docs/product-description/cross-cutting/configuration.md",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"
    ],
    "read_first": [
      "docs/history/pi-parity-review-fixes/CONTEXT.md",
      "docs/history/pi-parity-review-fixes/plan.md",
      "docs/knowledge/areas/hook-runtime/overview.md",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      ".bee/verify/verify-app/features/README.md",
      ".bee/verify/verify-app/features/worktree-and-close.md",
      "docs/product-description/observability/status.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"],
    "action": "Apply red-before-green for F4 and D5-D6. Add tests showing doctor rejects runtime pi today. Add Runtime::Pi, its .pi/extensions/bee-guard.ts and .agents/skills paths, and a Pi-specific row builder. Embed the canonical extension with compile-time include_str! and compare exact installed bytes. Reuse transport_kind_at and herding_transport_probe_for for one herding rule. Preserve common binary checks; in host repos compare the installed binary release version with .claude-plugin/plugin.json even when source mtime checks do not apply. Preserve row fields row/status/detail, return blocked and exit 1 for any required not_ok or unknown, ready and exit 0 only when all required rows are ok, and keep attestation null. Test absent, unreadable, drifted, malformed-config, missing-pane, stale-binary, and ready cases. Add the missing Pi runtime verify-app recipe. The existing worktree-and-close recipe was last modified 2026-09-13 22:39:09 +0700 and has no Pi health recipe; treat this as a new map, not trusted coverage. Update existing doctor contract docs and the one knowledge home.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml doctor::tests",
    "must_haves": {
      "truths": [
        "bee doctor --runtime pi returns the standard structured report instead of unsupported_argument_shape.",
        "Pi doctor checks installed extension bytes, shared skills, binary presence and freshness, and configured herding readiness.",
        "Every unavailable or failed required fact is named as not_ok or unknown and prevents ready.",
        "A complete current Pi installation reports ready and exits zero.",
        "The user-driving map covers success and each refusal class without claiming TUI evidence."
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "Pi runtime rows and fail-closed verdict"},
        {"path": "packages/bee-rs/crates/bee/src/doctor/tests.rs", "substantive": "Pi row and exit truth table"},
        {"path": ".bee/verify/verify-app/features/pi-runtime.md", "substantive": "installed Pi runtime driving recipe"},
        {"path": "docs/product-description/observability/status.md", "substantive": "public Pi doctor contract"}
      ],
      "key_links": [
        "doctor Pi paths match onboarded Pi extension and shared skill locations",
        "extension comparison uses bytes compiled into the running Bee binary",
        "herding row calls the same transport configuration and environment probe as dispatch prepare",
        "product description, verification map, and tests assert the same row statuses and exit codes"
      ],
      "prohibitions": [
        "Do not report ready from presence alone.",
        "Do not add Pi trust attestation.",
        "Do not create a second herding readiness rule.",
        "Do not mutate the repository during doctor."
      ]
    },
    "behavior_change": true
  }
]
```

## Verification

Each bugfix cell records the observed red command and the post-fix narrow green command.

- Preamble and activity: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts`.
- Workflow race: focused workflow-store and workflow-verb tests.
- Doctor: focused doctor tests plus the Pi command against a disposable onboarded repository.
- Cross-belt parity: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test opencode_plugin_contracts`.
- Full project: `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`.
- User-facing proof: launch a fresh verify-app sandbox, onboard its real Pi extension and binary, invoke session-init with the exact payload emitted by that installed extension, extract and execute its dispatch command, and retain the JSON plus snapshots. This non-TUI installed-sandbox path is required for acceptance and is `green:live`. The real-extension Node harness separately proves Pi lifecycle events and remains `green:unit`; it never substitutes for the installed-sandbox result.

## Documentation and durable state

Update `docs/config-reference.md` at its existing Pi belt section. Add the missing Pi runtime recipe under `.bee/verify/verify-app/features/`. Do not create a second runtime catalog.

After implementation, update the existing hook-runtime and workflow-state knowledge homes. Record why any promote proposal is not applicable. Close with fresh proof and an explicit capture result.

## Hat-wave synthesis

Five advisor views ran against the gate-ready plan. They found missing runtime and doctor truth tables, an ambiguous lock test, incomplete prompt-span semantics, no named extension-byte source, and a fallback that could weaken live acceptance.

The plan now defines all of those items. It also states the honest Pi 0.84.4 compaction boundary: there is no post-compact session-start carrier, so this feature makes Bee’s compact builder runtime-correct without inventing unsupported Pi behavior. No advisor finding changes D1–D8 or removes a review finding.
