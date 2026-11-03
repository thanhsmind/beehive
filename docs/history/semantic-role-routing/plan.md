---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Semantic Role Routing

## Summary

Store semantic role assignments in the approved plan packet. Enforce them at
cell, non-cell, native-dispatch, and release doors. Add a structured cell
re-route operation. Preserve the existing role-to-agent resolver. Also repair
direct Pi movement between two Bee-managed worktrees.

The earlier docs-only packet was superseded by decision `68bc3484`. Commits for
`slr-1` and `slr-2` remain useful groundwork. They do not complete this plan.

## Requirements

- D1 `fc2bb09a`: Select roles during planning from work meaning and role descriptions.
- D2: Classify each workflow stage as required, conditional, or not applicable.
- D3 `68bc3484`: Store assignments in the approved packet and enforce them in code.
- D4: Re-route a cell only through a structured record tied to a `role-reroute` decision.
- D5: Keep semantic selection free of keyword matching. Keep one deterministic resolver.
- D6 `c0a4d406`: Run an approved release through `deploy`; keep `scripts/release.sh`.
- D7: Keep `bee-plan/v1` and plans without role assignments compatible.
- D8 `bf919268`: Support verified Pi worktree-to-worktree session replacement.
- D9: Keep independent review conditional and user-invoked.

## Load-bearing claims

| # | Claim | Label | Anchor | Evidence |
|---|-------|-------|--------|----------|
| 1 | Gate preview already parses and stores an immutable cell packet. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:81-317` | `parse_plan_packets` validates cells, and `run_gate_preview_body` stores the packet with the plan SHA-256. |
| 2 | A cell role is already required and compared with its approved packet value. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:130-157,511-523` | Missing roles and role mismatches are typed refusals. |
| 3 | An explicit dispatch role currently overrides a cell role. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1479-1494` | `--role` wins before the cell role is read. |
| 4 | Native dispatch guards already read role markers and assigned cell IDs. | read | `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:712-967,1420-1498` | The guard validates markers and can load the assigned cell. |
| 5 | Cell update does not permit a role change. | read | `bee cells update --help` | The documented plan fields omit `role`; frozen and unknown keys refuse. |
| 6 | Release publication has one implementation but no role authorization check. | read | `scripts/release.sh:1-308` | The script checks branch, tree, tests, push, CI, and assets, but not the dispatch role. |
| 7 | Pi rejects a transition whose source differs from active `ctx.cwd`. | read | `.pi/extensions/bee-guard.ts:1140-1155` | `canonSource !== canonCtxCwd` returns `null`. |
| 8 | `worktree enter` currently refuses linked worktrees. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:171-199` | `ctx.kind != "ordinary"` is a typed refusal. |

## Role assignments

This block is part of the Gate 2 packet. The planner inspected the complete Pi
roster from `bee team show --runtime pi` and used each role description.

```json
{
  "schema_version": "1.0",
  "runtime": "pi",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"Plan structure, cells, and gate packet are planning work."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow known-location fact is needed.","reason":"This is the configured narrow lookup role."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"Execution needs a bounded read-only digest broader than one extraction.","reason":"This role owns read-only lookup work."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"The packet changes public workflow contracts."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Dispatch and release refusals can block work."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"The feature adds enforcement and must justify its cost."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"The plan must test smaller enforcement shapes."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Users will see new typed refusals and worktree behavior."},
    {"stage":"implementation","classification":"required","role":"code","reason":"Rust, TypeScript, and shell behavior must change with tests."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"The user-facing CLI, Pi relocation, reroute, and release refusal need behavior proof."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"Skills, knowledge, help, and verification maps must stay aligned."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"The user explicitly invokes independent review.","reason":"Review remains a separate user-invoked pass."},
    {"stage":"deployment","classification":"conditional","role":"deploy","condition":"The user approves a release version after the feature closes.","reason":"The configured deploy role publishes, waits for CI, and verifies assets."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The five named hats supply the required plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended supervisor loop is part of this feature."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"The implementation has repository precedent and does not need blind convergence."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"The implementation has repository precedent and does not need blind convergence."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"The implementation has repository precedent and does not need blind convergence."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every planned job has a more specific configured role."}
  ]
}
```

## Interface and Compatibility Contract

### Role-plan packet

The discriminator is the exact frontmatter value
`artifact_contract: bee-plan/v2`. A v2 plan has exactly one level-2
`Role assignments` section. Its first fenced JSON value is an object with these
exact top-level fields:

- `schema_version`: exact string `1.0`;
- `runtime`: one supported dispatch runtime;
- `roster_sha256`: SHA-256 of the sorted canonical `{role,description}` rows
  from `bee team show --runtime <runtime> --json`;
- `stages`: a non-empty array.

Each stage has the exact required fields `stage`, `classification`, `role`, and
`reason`. A conditional row also requires `condition`. Stage names are unique.
Classifications are `required`, `conditional`, or `not-applicable`. Every row
names one configured role, including a not-applicable row. The set of row roles
must cover every role in the roster. This proves that planning considered the
complete roster and descriptions. Duplicate sections, duplicate stages,
unknown fields, malformed JSON, roster drift, missing roles, and incomplete
roster coverage are typed refusals.

Gate preview stores the object as `role_plan` beside `cells`. Text output shows
runtime, roster digest, stage, classification, role, condition, and reason. The
plan SHA-256 makes cells and role assignments one immutable approval unit.

A plan without the v2 discriminator and without a role section takes the exact
legacy path. Its preview and packet gain no new key. A role section without the
v2 discriminator refuses with `role_plan_requires_v2`. A v2 plan without one
valid role section refuses with `role_plan_required`.

### Runtime and dispatch wire contract

The approved role plan is runtime-specific. Any cell or non-cell dispatch for
that feature on another runtime refuses with `role_plan_runtime_mismatch` and
requires a new plan revision. Gate preview validates every listed role against
`team.<runtime>` and the recorded roster digest. Prepare repeats the digest
check and refuses `role_plan_roster_stale` before it creates a payload or log.

`dispatch prepare` adds `--feature <slug>` and `--stage <name>`. For non-cell
work, feature resolution is: explicit `--feature`, then the calling session's
bound lane. The command now accepts `--session-id` for this read. If no approved
v2 packet resolves, legacy behavior remains unchanged.

After a v2 Gate 2 approval, every non-cell dispatch requires its stage. Missing,
unknown, and not-applicable stages refuse. The requested role must equal the
stage role. Cell dispatch derives its feature and planned role from the cell;
an explicit `--role` can only repeat the current cell role. The returned payload,
economics block, and dispatch log add `feature`, `stage`, `planned_role`,
`plan_sha256`, and `role_reroute_decision` when present. Existing role-to-agent
resolution runs only after these checks.

Prepared native prompts add anchored `[bee-feature: ...]` and `[bee-stage: ...]`
markers beside `[bee-tier: ...]`. The native guard resolves the main store,
loads the assigned cell or approved stage, and compares the marker role. Missing
or mismatched identity refuses before dispatch. Legacy prompts stay
byte-identical.

### Re-route serialization contract

`bee cells reroute --id <id> --role <role> --decision <id>` acquires the same
per-cell lock used by claim. It reads and checks status, claim, approved packet,
role plan, runtime roster, and decision while that lock is held. It accepts only
open or blocked cells without a claim. The target role must be configured for
the approved role-plan runtime. An unchanged role refuses with `role_unchanged`.

The cited decision must exist, belong to the same feature, and carry tag
`role-reroute`. One atomic write appends this record and updates `role`:

```json
{"from":"code","to":"test","decision":"<uuid>","at":"<iso>","plan_sha256":"<approved-sha>"}
```

The approved packet retains the original role. A valid `role_reroutes` chain is
the only exception to exact packet-role equality after cell creation. Every
entry's `from` must equal the prior effective role. Claim and prepare serialize
against reroute, so execution cannot observe a half-updated role.

### Release authorization protocol

`dispatch prepare` accepts `--release-version <semver>` only for stage
`deployment` and role `deploy`. It binds one authorization record to:

- dispatch ID and issuer session;
- feature and approved plan SHA-256;
- runtime, stage `deployment`, and role `deploy`;
- requested version and current main commit;
- creation time with a two-hour lifetime.

The returned herding command exports `BEE_DISPATCH_ID` and
`BEE_RELEASE_VERSION`. The dispatch log is authoritative for all other fields.
A new `bee dispatch authorize --id <id> --release-version <version>` reads the
log, the approved packet, current main commit, and current time. It atomically
creates one consumed marker only after all values match. Reuse refuses.

`scripts/release.sh` calls this command before version edits, regeneration,
tagging, pushing, or any resume path. Missing, unknown, stale, consumed,
wrong-feature, wrong-plan, wrong-commit, wrong-stage, wrong-role, and
wrong-version authorizations refuse with no release mutation. A failed release
needs a new deploy dispatch. The feature test only drives authorization and
preflight; it does not publish a version.

### Pi source and control roots

`enter_worktree_core` receives separate `control_root` and `source_root` values.
Grant and Git-link checks use the canonical main control root. Transition
`sourceCwd` uses the canonical active checkout root. `run_enter` accepts only
`ordinary` and `linked-valid` contexts, resolves main through the linked Git
metadata, and rejects a target equal to the source before marker emission.

No Pi extension source change is expected. Its existing validation requires
`sourceCwd === ctx.cwd`, and deferred revalidation reruns `worktree enter` from
that directory. Pi contract tests prove this fact. If a red test shows an
extension change is necessary, execution must record a re-route before adding
that file.

### Refusal and recovery matrix

| Reason | Door | Mutation before refusal | Recovery |
|--------|------|-------------------------|----------|
| `role_plan_required` / `role_plan_requires_v2` | Gate preview | None | Repair `plan.md`, preview again. |
| `role_plan_roster_stale` | Preview or prepare | None | Run `team show`, revise the plan, approve a new revision. |
| `role_plan_runtime_mismatch` | Prepare or native guard | None | Revise the runtime-specific role plan. |
| `stage_required` / `stage_unknown` / `stage_not_applicable` | Prepare or native guard | None | Supply the approved stage or revise the plan. |
| `planned_role_mismatch` | Prepare or native guard | None | Use the planned role or run `cells reroute`. |
| `role_reroute_*` | Cells reroute | None | Repair the decision, role, status, or claim conflict. |
| `deploy_authorization_*` | Dispatch authorize or release script | None except a successful consume marker | Create a new approved deploy dispatch. |
| `same_worktree` / existing transition refusals | Worktree enter | None | Select another granted worktree or repair its grant/link. |

## Cells — current slice preview

```json
[
  {
    "id":"slr-3",
    "feature":"semantic-role-routing",
    "role":"code",
    "lane":"standard",
    "title":"Store and display approved workflow role assignments",
    "action":"Add red-first tests, then implement the Role-plan packet contract: exact v2 discriminator and section, exact JSON fields, runtime and roster digest validation, complete configured-role coverage, unique stages, classification rules, stored role_plan output, rendered visibility, duplicate and malformed refusals, and a byte-identical legacy path.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml plan_packets",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",".agents/skills/bee-planning/references/planning-reference.md"],
    "deps":[],
    "decisions":["fc2bb09a","68bc3484"],
    "must_haves":{"truths":["Gate preview JSON and text show every role stage and roster identity.","The approved packet stores the exact validated role_plan object.","bee-plan/v2 refuses missing, duplicate, malformed, incomplete, unknown-field, unconfigured-role, and stale-roster blocks.","Legacy plans without a role block keep byte-identical behavior."]},
    "affects_skills":["skills/bee-planning/references/planning-reference.md"],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Role assignments are visible, immutable with the plan hash, validated for v2, and optional for legacy plans."
  },
  {
    "id":"slr-4",
    "feature":"semantic-role-routing",
    "role":"code",
    "lane":"high-risk",
    "title":"Enforce approved roles at dispatch preparation",
    "action":"Add red-first tests, then implement the Runtime and dispatch wire contract. Add feature and stage inputs, session-lane lookup, runtime and roster checks, exact typed refusals, cell-role equality, non-cell stage enforcement, anchored native identity markers, and audit fields. Keep legacy payloads byte-identical and run the existing deterministic resolver only after enforcement.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml drivers::tests",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],
    "deps":["slr-3"],
    "decisions":["fc2bb09a","68bc3484"],
    "must_haves":{"truths":["An explicit cell role mismatch is refused before payload creation or logging.","An approved v2 non-cell dispatch without a stage is refused.","Required and conditional stage dispatches use the approved runtime role.","Runtime, roster, stage, plan hash, and planned role are visible in dispatch audit output.","No keyword mapping or second role resolver is added."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"All dispatch prepare paths follow the approved cell or stage role and retain the existing resolver."
  },
  {
    "id":"slr-5",
    "feature":"semantic-role-routing",
    "role":"code",
    "lane":"high-risk",
    "title":"Add controlled cell role re-routing and native guard checks",
    "action":"Add red-first tests, then implement the Re-route serialization contract under the cell claim lock. Validate status, no claim, configured role, decision feature and tag, unchanged role, plan hash, and history chain. Update native Claude and Codex guards to validate anchored feature, stage, cell identity, and current effective role without resolving an agent twice.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml reroute && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml model_guard",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs","packages/bee-rs/crates/bee/src/hooks/model_guard.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/cells/mod.rs","packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs","packages/bee-rs/crates/bee/src/verbs/cells/validate.rs","packages/bee-rs/crates/bee/src/verbs/cells/tests.rs","packages/bee-rs/crates/bee/src/hooks/model_guard.rs","packages/bee-rs/crates/bee/tests/hook_contracts.rs"],
    "deps":["slr-3"],
    "decisions":["fc2bb09a","68bc3484"],
    "must_haves":{"truths":["Only open or blocked unclaimed cells can reroute under the claim lock.","The cited decision must match the feature and carry role-reroute.","Every accepted reroute records old role, new configured role, decision, time, and plan hash.","The immutable approved packet remains the original assignment and only a valid history chain changes the effective role.","Native direct dispatch cannot bypass cell or stage assignments."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Cell role changes are explicit, structured, audited, and enforced by native guards."
  },
  {
    "id":"slr-6",
    "feature":"semantic-role-routing",
    "role":"code",
    "lane":"high-risk",
    "title":"Require deploy-role authorization for release execution",
    "action":"Add red-first tests, then implement the Release authorization protocol. Add release-version input, bind the dispatch log to session, feature, plan hash, runtime, deployment stage, deploy role, version, main commit, and time, export only the dispatch ID and version, add atomic two-hour one-use authorization, and make scripts/release.sh consume it before every mutation or resume path.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml deploy && bash -n scripts/release.sh",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md","scripts/release.sh","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs","scripts/release.sh"],
    "deps":["slr-4"],
    "decisions":["c0a4d406","68bc3484"],
    "must_haves":{"truths":["A direct release script call refuses before mutation.","Only an unconsumed deployment dispatch under deploy authorizes its exact version, plan, and main commit.","Stale, replayed, forged, wrong-feature, wrong-plan, wrong-stage, wrong-role, wrong-version, and wrong-commit permits refuse.","Existing release safety checks remain unchanged after authorization."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md"],
    "acceptance":"Approved releases cannot bypass the deploy role, while scripts/release.sh remains the implementation."
  },
  {
    "id":"slr-7",
    "feature":"semantic-role-routing",
    "role":"code",
    "lane":"standard",
    "title":"Support verified Pi switching between linked worktrees",
    "action":"Reproduce the linked-to-linked failure in tests, then split enter_worktree_core into canonical control_root and source_root inputs. Allow ordinary and linked-valid entry, use main for grant and Git-link authority, emit the active root as sourceCwd, reject same-worktree entry before marker emission, and prove that the unchanged Pi validator accepts the new marker while forged paths still fail.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml enter_worktree && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml pi_plugin_contracts",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md","docs/history/pi-worktree-session-relocation/CONTEXT.md","packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",".pi/extensions/bee-guard.ts"],
    "files":["packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs","packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs","packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "deps":[],
    "decisions":["bf919268"],
    "must_haves":{"truths":["Linked worktree A can emit a valid enter transition to linked worktree B.","The transition source is A and still equals Pi ctx.cwd.","Same-worktree, absent-grant, bad-link, wrong-session, and forged-source cases refuse.","Main-to-worktree and worktree-to-main behavior stay unchanged."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/worktree-first.md"],
    "acceptance":"Pi changes session automatically from one granted linked worktree to another without weakening authenticity."
  },
  {
    "id":"slr-8",
    "feature":"semantic-role-routing",
    "role":"test",
    "lane":"high-risk",
    "title":"Prove semantic routing, deployment refusal, and Pi relocation live",
    "action":"Regenerate command and help artifacts, update knowledge and verification maps, then drive a fresh installed sandbox. Prove v1 compatibility, v2 role visibility, runtime and stage refusals, a normal cell role, conditional review without auto-start, accepted and refused reroutes, stale and replayed deploy permits, direct release refusal, authorized deploy preflight without publication, and linked-worktree A to B Pi relocation. Preserve evidence and record green:live.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check",
    "read_first":["docs/history/semantic-role-routing/CONTEXT.md",".bee/verify/verify-app/features/README.md",".bee/verify/verify-app/features/worktree-and-close.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "files":["packages/bee-rs/crates/bee/src/generated/registry_payload.json",".bee/verify/verify-app/features/README.md",".bee/verify/verify-app/features/semantic-role-routing.md",".bee/verify/verify-app/features/worktree-and-close.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md","docs/knowledge/areas/worktree-parallelism/entering-creating-and-registering.md","docs/config-reference.md","docs/product-description/cross-cutting/configuration.md","docs/product-description/verification/areas.md"],
    "deps":["slr-4","slr-5","slr-6","slr-7"],
    "decisions":["fc2bb09a","68bc3484","bf919268","c0a4d406"],
    "must_haves":{"truths":["The complete related Rust and contract suite passes.","Fresh installed behavior proves role visibility and refusals.","Conditional review remains dormant until user invocation.","A real Pi session relocates from linked worktree A to B.","Release preflight refuses direct execution and accepts deploy authorization without publishing an unapproved version."]},
    "affects_skills":[],
    "affects_specs":["docs/specs/doctrine-layer.md","docs/specs/worktree-first.md"],
    "acceptance":"All new enforcement and relocation paths have retained green:live evidence from a disposable repository."
  }
]
```

## Test Matrix

| Behavior | Unit or contract proof | Live proof |
|----------|------------------------|------------|
| v2 role-plan validation and visibility | `plan_packets` tests | Gate preview JSON in sandbox |
| Cell role mismatch | driver tests | Refused prepare command |
| Required and conditional stages | driver tests | Required stage accepted; missing stage refused; review not auto-started |
| Structured reroute | cells and hook tests | One accepted and three refused reroutes |
| Deploy authorization | driver and shell tests | Direct release refusal; authorized preflight without publish |
| Pi linked-to-linked relocation | worktree and Pi contract tests | Active session changes from worktree A to B |
| Legacy plans | plan packet regression tests | Existing v1 sandbox plan still previews |

## Delivery Order

1. Run the five advisor hats against this revision.
2. Record advisor synthesis and conflict verdicts.
3. Preview and auto-approve Gate 2 under bypass `full`.
4. Add only `slr-3` through `slr-8` from the approved packet.
5. Execute `slr-3` and `slr-7` concurrently because their files are disjoint.
6. Execute `slr-4` and `slr-5` after `slr-3`; then execute `slr-6` after `slr-4`.
7. Test deployment authorization only. Do not publish a release.
8. Run live verification and complete capture.

## Open Questions

None.

## Out of Scope

- Keyword classification.
- A second role-to-agent resolver.
- Automatic independent review.
- Publishing a release version as part of this feature.
