---
artifact_contract: bee-plan/v2
mode: standard
---

# Pi statusline cache totals by model

Mode: `standard` — two risk flags: public-contracts and covered-contract-change.

This is the least workflow that protects a user-visible Pi extension contract.

Class playbook: `skills/bee-planning/references/planning-reference.md` ("Class playbooks" → "feature").

## Outcome

Pi keeps its default footer. One added status segment shows active-branch token totals for each provider/model pair.

The segment uses this row shape:

```text
<provider>/<model> <new> new/<cached> cached
```

`new` is input plus output plus cache-write tokens. `cached` is cache-read tokens.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Existing bee accounting defines new tokens as input, output, and both cache-write buckets. Cached tokens are cache reads. | read | `packages/bee-rs/crates/bee/src/devtools/statusline.rs:400-401` | `new_tokens: s.r#in + s.out + s.c5 + s.c1,` and `cached_tokens: s.read,` |
| 2 | The Pi belt already has session-start and settled-turn lifecycle handlers. | read | `.pi/extensions/bee-guard.ts:1832,2038` | `pi.on("session_start", ...` and `pi.on("agent_settled", ...` |
| 5 | The contract suite loads the repository's real Pi extension. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:107` | `const PI_PLUGIN_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");` |
| 6 | The current harness supplies a fake session manager and UI. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:599,658` | `class FakeSessionManager` and `ui: {` |
| 7 | Doctor embeds the same Pi extension bytes. The release manifest fingerprints that file. | read | `packages/bee-rs/crates/bee/src/doctor.rs:47`; `docs/history/codex-harness-hardening/release-manifest.json:1650` | `const PI_EXTENSION_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");` and `"path": ".pi/extensions/bee-guard.ts"` |

## Design

1. Add pure helpers in `.pi/extensions/bee-guard.ts`.
   - Read only `ctx.sessionManager.getBranch()`.
   - Accept only branch entries whose message role is `assistant`.
   - Require non-empty provider and model strings.
   - Group with the full provider/model pair as the key and label.
   - Add finite, positive input, output, cache-write, and cache-read values.
   - Preserve the pair's first appearance order on the active branch.
2. Format each total as `<provider>/<model> <new> new/<cached> cached`.
   - Use plain integers below 1,000.
   - Use compact `k` and `m` suffixes at larger values.
   - Join model rows with ` · ` inside one status segment.
3. Refresh the status on `session_start`, `turn_end`, and `session_tree`.
   - `session_start` restores a saved session's totals.
   - `turn_end` includes the newly completed assistant message.
   - `session_tree` follows branch navigation.
4. Clear the named status when the active branch has no positive model usage.
5. Wrap branch reading, aggregation, formatting, and `setStatus` in one advisory try/catch.
   A status failure logs an advisory error and never blocks the session.
6. Extend the existing Node harness instead of creating a second test harness.
   - Give `FakeSessionManager` branch entries from each call.
   - Record `ctx.ui.setStatus` calls.
   - Return those calls in `HarnessRun`.
   - Add never-throw rows for the two newly registered event names.
7. Update the Pi runtime verification map and regenerate the release manifest.

### Smaller path

This plan does not install a custom footer. `ctx.ui.setStatus` adds one segment to Pi's default footer.

This plan does not poll the session file. Pi lifecycle events provide the required refresh points.

This plan uses one implementation cell. The extension, its real contract harness, and its fingerprint must change together.

### Rejected alternatives

- Replace Pi's footer: rejected because it would duplicate and own Pi's existing footer behavior.
- Read all session entries: rejected because sibling branches would inflate totals.
- Attribute tool-result usage: rejected because those entries do not carry a reliable provider/model identity.
- Group by model alone: rejected because two providers can use the same model identifier.
- Update on a timer: rejected because status changes only after a completed turn or branch change.

## Risk map

| Component | Risk | Reason | Lands in | Proof needed |
|---|---|---|---|---|
| Active-branch aggregation | MEDIUM | Wrong entry filtering can count sibling branches or tool usage. | psc-1 | Contract fixture with repeated and distinct provider/model pairs plus ignored non-assistant usage. |
| Lifecycle refresh | MEDIUM | Restore or branch navigation can leave stale totals. | psc-1 | One ordered contract run drives `session_start`, `turn_end`, and `session_tree`. |
| Footer integration | LOW | A custom footer would overwrite Pi defaults. | psc-1 | Contract records only `setStatus`; source contains no `setFooter`. |
| Failure isolation | LOW | Malformed branch data or UI errors must not stop Pi. | psc-1 | Existing advisory never-throw matrix covers all registered events. |
| Embedded bytes and fingerprint | LOW | Doctor and release checks fail on stale generated data. | psc-1 | Full Pi contract test, regen, manifest check, and Pi doctor. |

Waves: one cell. Splitting the harness from the extension would create a red intermediate commit and duplicate file ownership.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "pi",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"The plan must lock aggregation, refresh, and proof behavior."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow source fact becomes unclear during execution.","reason":"Known-location extraction can resolve it without changing the plan."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"Execution finds a broader Pi API question.","reason":"The read role can gather a repository and installed-doc digest."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"The inline plan check verifies source anchors and missing cases."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"The inline plan check tests the smaller setStatus design."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"The inline plan check protects footer continuity and empty-state behavior."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"The standard lane opens three hat perspectives."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"The standard lane opens three hat perspectives."},
    {"stage":"implementation","classification":"required","role":"code","reason":"One worker must change the TypeScript belt and its Rust contract harness together."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"The mapped Pi runtime contract must prove the real extension behavior."},
    {"stage":"semantic-goal-check","classification":"required","role":"review","reason":"The first standard-lane behavior slice requires checklist-judge verification."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"The Pi verification map and status-display knowledge must describe the new surface."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"The user explicitly requests independent review.","reason":"Independent review remains a separate user-invoked pass."},
    {"stage":"deployment","classification":"conditional","role":"deploy","condition":"The user asks for a release after close.","reason":"Deployment is outside this feature change."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The inline standard-lane plan check supplies the required consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended supervisor loop is needed."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"The integration point and accounting rule are known."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"No competing root-cause theory remains."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"No discovery lane is needed."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"The implementation has a specific code role."}
  ]
}
```

## Shape

| Cell | Title | Role | Files | Depends on |
|---|---|---|---|---|
| psc-1 | Show active-branch new and cached tokens by provider/model in Pi's default footer | code | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`, `.bee/verify/verify-app/features/pi-runtime.md`, release manifest | — |

## Cells — current slice preview

```json
[
  {
    "id": "psc-1",
    "feature": "pi-statusline-cache-by-model",
    "title": "Show active-branch token totals by provider and model in Pi",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["72f4a8b2-b1f4-467a-bfe0-56cf22da2565"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      ".bee/verify/verify-app/features/pi-runtime.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pi-statusline-cache-by-model/CONTEXT.md",
      "docs/history/pi-statusline-cache-by-model/plan.md",
      "docs/knowledge/areas/onboarding/status-display-vendoring.md",
      ".bee/verify/verify-app/features/pi-runtime.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/devtools/statusline.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/status-display-vendoring.md",
      ".bee/verify/verify-app/features/pi-runtime.md"
    ],
    "action": "RED FIRST. Extend the existing HARNESS_JS so FakeSessionManager.getBranch() returns per-call branch entries, the fake UI records setStatus(key, text), the JSON result returns those calls, and HarnessRun parses them. Add one contract test named with the filter `model_usage_status`. Drive the REAL extension in one ordered run: session_start restores totals from multiple assistant messages; repeated provider/model messages aggregate; another provider/model gets its own row; nested tool-result usage and non-assistant entries do not count; turn_end refreshes after a new assistant message; session_tree with an empty branch clears the named status. Assert the exact compact display and first-appearance order. Add `turn_end` and `session_tree` to never_throw_event_rows. Run the filtered test and record the expected RED because the belt has no model-usage status. Then implement pure aggregation and formatting helpers in `.pi/extensions/bee-guard.ts`. Read only ctx.sessionManager.getBranch(). Count assistant-message usage by full provider/model. Compute new as input + output + cacheWrite and cached as cacheRead. Accept only finite positive numbers. Render `<provider>/<model> <new> new/<cached> cached`; compact thousands and millions; join rows with ` · `. Refresh one stable status key on session_start, turn_end, and session_tree. Call setStatus with undefined when no positive usage exists. Keep all status work advisory: catch and log failures without throwing or blocking. Do not call setFooter. Update the Pi runtime verification map with this status contract and its command. Run `bee dev regen` so the release manifest fingerprints the changed extension.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts model_usage_status && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "Pi's default footer remains installed and one status segment shows model usage",
        "new tokens equal input plus output plus cache-write tokens",
        "cached tokens equal cache-read tokens",
        "totals use only assistant messages on the active branch",
        "each provider/model pair has an independent total",
        "the segment refreshes after turn completion, session restore, and branch navigation",
        "the segment is absent when the active branch has no positive model usage",
        "status calculation and display failures never interrupt the session"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "active-branch aggregation, compact formatting, advisory refresh handlers, and setStatus integration"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "branch-aware status harness plus exact lifecycle and aggregation contract"},
        {"path": ".bee/verify/verify-app/features/pi-runtime.md", "substantive": "mapped user-facing status proof"},
        {"path": "docs/history/codex-harness-hardening/release-manifest.json", "substantive": "updated Pi extension fingerprint"}
      ],
      "key_links": [
        "the status helper reads getBranch rather than the full session log",
        "session_start, turn_end, and session_tree call the same refresh helper",
        "the contract harness loads PI_PLUGIN_SOURCE from the real extension file",
        "the token equation matches packages/bee-rs/crates/bee/src/devtools/statusline.rs"
      ],
      "prohibitions": [
        "Do not replace Pi's footer or call ctx.ui.setFooter",
        "Do not count tool-result usage or sibling branches",
        "Do not poll the session file or add a timer",
        "Do not change Claude, Codex, or OpenCode status behavior"
      ]
    },
    "behavior_change": true
  }
]
```

## Proof

- Red evidence: the filtered `model_usage_status` contract fails before implementation because no status call exists.
- Unit contract: the filtered test proves aggregation, formatting, refresh, clearing, and ignored entries.
- Pi contract suite: `cargo test --release -p bee --test pi_plugin_contracts` proves all extension contracts remain green.
- Generated parity: `bee dev release-manifest --check` proves the extension fingerprint is current.
- Mapped user-facing proof: the Pi runtime feature drives the real extension through the Node host contract and records the footer status calls.
- Semantic goal-check: a review-role checklist judge verifies the capped behavior cell against its existing requirements.
- Close proof: run the repository command from the project preamble after the focused checks pass.
