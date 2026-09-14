---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Deploy-stage execution

## Summary

Repair the deployment payload before release `2.37.3`. Deployment must give the approved deploy worker a mutating release brief and main-checkout cwd. Ordinary gather behavior must not change.

## Requirements

- D1 `38edea83`: Deployment runs `scripts/release.sh <authorized-version>` from the main control root.
- D2: Ordinary gather remains read-only and uses a granted feature worktree when one exists.
- D3: Existing role selection, authorization fields, expiry, and one-use consumption remain unchanged.
- D4: The worker receives the leader-selected version. The worker cannot select another version.
- D5: Independent review remains conditional and user-invoked.
- D6: This plan authorizes Pi herding only. Another runtime refuses through the approved runtime check.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Non-cell dispatch loads the prompt template for its kind. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:974-997` | `if kind != "cell" {` then `let Some(template) = load_prompt(kind)` |
| 2 | Feature identity selects a granted worktree before transport construction. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1761-1765` | `find_granted_worktree_for_feature(root, &feature)` |
| 3 | Herding uses the selected worktree as its cwd. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2267-2273` | `command.push_str(" --cwd \"");` then `command.push_str(worktree_root);` |
| 4 | The release script requires main and consumes deploy authorization before mutation. | read | `scripts/release.sh:95-107` | `[[ "$CURRENT_BRANCH" == "main" ]]` and `"${BEE_BIN}" dispatch authorize` |
| 5 | The prepared release payload is currently incompatible. | ran | `bee dispatch prepare ... --stage deployment --release-version 2.37.3 --json` | `Gather: locate and digest ... Read-only` with `--cwd "/home/thanhsmind/Projects/goglbe/beehive--wt--semantic-role-routing"` |
| 6 | The shared delegation contract says gather cannot mutate. | read | `skills/bee-hive/references/gates-and-delegation.md:131-150` | `a gather never writes` and `Herding execution is a cell, never a gather.` |
| 7 | The user-facing release recipe uses unsupported option syntax. | read | `.bee/verify/verify-app/features/semantic-role-routing.md:34-36` | `bash scripts/release.sh --version <semver>` |

## Implementation contract

Detect deployment from the already validated v2 dispatch information. Do not infer deployment from role names, keywords, or purpose text.

For a validated Pi herding dispatch at stage `deployment` and role `deploy`:

1. Build the final `payload.stdin` through an explicit stage-aware branch after role-plan validation.
2. Name the exact `scripts/release.sh <authorized-version>` positional command.
3. Tell the worker to execute that command and wait for CI and asset verification.
4. Do not prepend the embedded gather agent or include its read-only prohibition.
5. Set herding `--cwd` to the canonical main control root, whether or not the feature has a granted worktree.
6. Refuse before payload creation unless that control root is on branch `main`.

A wrong role or runtime refuses through existing role-plan checks. A deployment-shaped request without an approved v2 role plan also refuses. Non-deployment stages preserve current prompt and cwd bytes.

Add behavior tests before implementation. The red tests must assert `payload.stdin`, `payload.command`, main cwd with and without a feature worktree, branch refusal, and the no-plan refusal. Existing ordinary gather tests must prove complete prompt and cwd bytes remain unchanged with and without a worktree.

Update the source delegation contract with the narrow deployment exception. Regenerate both host projections from source. Update the doctrine knowledge entry and semantic-routing verification map. Correct its release recipe from unsupported `--version` syntax to `bash scripts/release.sh <version>`. Run `dev release-manifest --write` before `--check`.

## Authorization invariants

| Case | Required result |
|---|---|
| Unknown UUID or path-shaped forged ID | Refuse before release mutation. |
| Missing, malformed, future, over-limit, or expired timestamps | Refuse before release mutation. |
| Missing or wrong main commit, feature, plan, runtime, stage, role, version, or session | Refuse before release mutation. |
| Missing lane, approved plan, issuer session, or acting session | Refuse before release mutation. |
| Marker collision or marker write failure | Refuse without an ambiguous consumed state. |
| Replayed permit | Refuse as consumed. A failed preflight always needs a new dispatch ID. |
| Valid deployment permit | Export its dispatch ID and validated version, then let `release.sh` consume it once. The positional version is authoritative; the environment version is audit-only and must match. |
| Feature verification | Execute the exact prepared payload against a harmless `release.sh` stub. Do not tag, push, publish, or wait for release CI. |
| User-approved release after merge | Dispatch role `deploy` from main, run the real prepared payload, and verify published assets. A failure after tag push requires a new version; never move or reuse the tag. |

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "pi",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"The release boundary needs a precise repair packet."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow implementation fact is needed.","reason":"This role owns known-location extraction."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"A broader read-only digest is needed.","reason":"This role owns read-only repository lookup."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"The plan must find missing release contract facts."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"A wrong payload can publish or block a release."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"The repair must remain smaller than a new dispatch system."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"The plan must test cheaper safe shapes."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"The deploy worker and release operator observe the payload."},
    {"stage":"implementation","classification":"required","role":"code","reason":"Rust behavior and tests must change together."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"The prepared payload and release path need behavior proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"The shared contract, knowledge, and generated projections must agree."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"The user explicitly requests independent review.","reason":"Review remains a separate user-invoked pass."},
    {"stage":"deployment","classification":"conditional","role":"deploy","condition":"The repaired feature is merged and the user-approved patch release is ready.","reason":"The configured deploy role publishes from main and verifies assets."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The five required hats supply the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended supervisor loop is needed."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"The fault and expected behavior are reproduced."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"The fault and expected behavior are reproduced."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"The fault and expected behavior are reproduced."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every planned job has a specific role."}
  ]
}
```

## Cells — current slice preview

```json
[
  {
    "id":"dse-1",
    "feature":"deploy-stage-execution",
    "role":"code",
    "lane":"high-risk",
    "title":"Build an executable deployment payload rooted at main",
    "action":"Write red behavior tests for deployment payload stdin and cwd with and without a granted feature worktree, non-main root refusal, and deployment-shaped requests without an approved v2 plan. Then implement a validated Pi-herding deployment brief and unconditional main-root cwd. Preserve ordinary gather bytes, runtime and role refusals, and all authorization behavior.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::tests",
    "read_first":["docs/history/deploy-stage-execution/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],
    "deps":[],
    "decisions":["38edea83","c0a4d406"],
    "must_haves":{"truths":["Deployment stdin permits mutation and names the exact authorized positional release command.","Deployment herding cwd is main with or without a feature worktree.","Non-main, wrong-runtime, wrong-role, and no-plan deployment requests refuse before payload creation.","Ordinary gather stays byte-identical, read-only, and feature-worktree rooted.","Authorization and replay checks remain unchanged."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Focused driver tests prove an executable deployment payload without changing ordinary gather or authorization behavior."
  },
  {
    "id":"dse-2",
    "feature":"deploy-stage-execution",
    "role":"docs",
    "lane":"standard",
    "title":"Document the narrow deployment mutation exception",
    "action":"Update the source delegation contract and doctrine knowledge entry for the authenticated deployment exception. Build the current source candidate, then regenerate host projections with that candidate. Write and check the release manifest; never conceal drift with sync acknowledgement.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee && CANDIDATE=\"$(cargo metadata --no-deps --format-version 1 --manifest-path packages/bee-rs/Cargo.toml | jq -r .target_directory)/release/bee\" && \"$CANDIDATE\" dev regen && \"$CANDIDATE\" dev release-manifest --write && \"$CANDIDATE\" dev release-manifest --check",
    "read_first":["docs/history/deploy-stage-execution/CONTEXT.md","skills/bee-hive/references/gates-and-delegation.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "files":["skills/bee-hive/references/gates-and-delegation.md",".agents/skills/bee-hive/references/gates-and-delegation.md",".claude/skills/bee-hive/references/gates-and-delegation.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md","docs/history/codex-harness-hardening/release-manifest.json"],
    "deps":["dse-1"],
    "decisions":["38edea83"],
    "must_haves":{"truths":["The shared contract permits mutation only for an authorized deployment stage.","Ordinary gather remains explicitly read-only.","Source and both generated host projections match.","The release manifest matches all tracked release files."]},
    "affects_skills":["skills/bee-hive/references/gates-and-delegation.md"],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Worker-facing instructions and durable knowledge describe one narrow deployment exception with generated parity."
  },
  {
    "id":"dse-3",
    "feature":"deploy-stage-execution",
    "role":"test",
    "lane":"high-risk",
    "title":"Prove deployment payload and authorization live without publication",
    "action":"Update the semantic-routing verification map, including the positional release command. Build and install the candidate into a fresh disposable repository. Execute the exact prepared herding payload through agy-flash against a harmless release.sh stub that records cwd, argv, dispatch ID, and audit version. Prove ordinary gather isolation, successful authorization, and replay refusal. Do not tag, push, publish, or invoke the real release script. Refresh and check the release manifest.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::tests && .bee/bin/bee dev release-manifest --write && .bee/bin/bee dev release-manifest --check",
    "read_first":["docs/history/deploy-stage-execution/CONTEXT.md",".bee/verify/verify-app/features/semantic-role-routing.md","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],
    "files":[".bee/verify/verify-app/features/semantic-role-routing.md","docs/history/codex-harness-hardening/release-manifest.json"],
    "deps":["dse-2"],
    "decisions":["38edea83","c0a4d406"],
    "must_haves":{"truths":["A fresh installed exact payload executes the harmless stub with the authorized argv, environment, and main cwd.","An ordinary gather payload remains byte-identical, read-only, and worktree rooted.","The valid permit authorizes once and replay refuses.","Feature proof performs no real release mutation or publication."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Retained green:live evidence proves the repaired user-facing deployment path before merge."
  }
]
```

## Proof and close

Run each cell command exactly. After all cells cap, run the full repository command and retain the harmless live payload evidence. Merge to the canonical main checkout at `/home/thanhsmind/Projects/goglbe/beehive`, then approve UAT from fresh main proof. Dispatch release `2.37.3` only through role `deploy`; the release worker runs the exact real payload, reports the final release `OK` line, waits for green CI, and verifies published assets.
