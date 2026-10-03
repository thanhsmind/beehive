---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: pi-slp-operations

## Summary
When bee refuses a call, some refusals tell the model what to run next and
some do not. Real Pi sessions show small models follow a fix that names a
command within one or two tries. When the fix names no command, or asks for
something an agent cannot do ("open a session"), they repeat the call, delete
state files by hand, or write through python. This change rewrites seven such
refusals so each one ends with one command or one path the model can use. No
refusal starts or stops refusing; only its words change.

Mode: `standard` — 2 risk flags: covered-contract-change, multi-domain
Why this is the least workflow that protects the work: four disjoint text-only cells, each with the tests that pin its words.

## Requirements (from CONTEXT.md)
- D1: seven refusal families each end with one runnable bee command or one concrete path built from values the code holds.
- D2: text only; refusal triggers, reason and code keys, exit codes unchanged.
- D3: write-guard denies name the in-worktree path, or `bee worktree merge --id <id>` from main; never "open a session".
- D4: CLAIMED says whether the caller holds the claim and names the next step or `bee cells claim-next`.

## Load-bearing claims
Labels: `read` (opened at the anchor). Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | role_plan_required names no command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1635 | stage deployment requires an approved v2 plan with a role plan. |
| 2 | consumed authorization names no command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4220 | this dispatch authorization has already been consumed; dispatch a new deployment run. |
| 3 | a second consumed site exists | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4556 | this dispatch authorization has already been consumed; dispatch a new deployment run. |
| 4 | wrong_feature names no command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4411 | lane file not found in store. |
| 5 | startFeature names no command | read | packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs:311 | FIX: close or resolve that workflow before starting a new one for the same feature. |
| 6 | CLAIMED names no next step | read | packages/bee-rs/crates/bee/src/verbs/cells/claims.rs:751 | is already claimed by session \"{owner}\" ({}). |
| 7 | the other-worktree deny asks the agent to open a session | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:566 | FIX: open a session with cwd={worktree_root} to work there |
| 8 | the main-path deny names no path | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:552 | FIX: run this from a session rooted there. |
| 9 | the preview mismatch names no fix | read | packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:728 | differs from approved preview packet (action mismatch). |

## Discovery
Five Pi sessions of this repo were mined at the advisor tier (research report,
"Transcript evidence"). The refusals above are the ones the models could not
act on. Fix verbs that exist today: `bee state session bind`,
`bee state workflows close`, `bee cells claim-next`, `bee worktree merge`,
`bee gate --preview`, `bee gate --name shape --approved false`.

## Approach
Playbook: `skills/bee-planning/playbooks/refactor.md` (class refactor: wording
only, behavior kept). Each cell rewrites its fix strings and updates the tests
that pin them, red first: a new assert for the runnable command fails on the
old text.

Rejected: one shared "refusal builder" for all seven — the families live in
four modules with different return shapes (JSON `fix`, `Err(String)`, typed
code); a shared helper adds a layer for seven strings.

Risk map:
- prepare.rs refusals / LOW / rfx-1 / drivers tests on fix text
- CLAIMED same-session detection / MEDIUM / rfx-2 / claim test for both holder cases
- write-guard path computation / MEDIUM / rfx-3 / write_guard tests with a main path and another worktree's path
- preview mismatch / LOW / rfx-4 / plan_packets tests

Waves: rfx-1 to rfx-4 in parallel — disjoint files.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"rfx-1 to rfx-4 change Rust refusal text with their tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own red-first tests."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader captures at close."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The transcript mining ran before the plan; the hat wave is the plan check."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape, no competing designs."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Plan-step hat wave."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"Plan-step hat wave."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"Plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Plan-step hat wave."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Plan-step hat wave."}
  ]
}
```

## Shape
Phase plan:

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Seven refusals end with a runnable command or path | Transcripts show small models follow named commands and flail on everything else | each refusal's test shows the command in its text | Slice 3: allowed-operations packet |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| rfx-1 | Name a runnable fix in the deployment dispatch refusals | prepare.rs, drivers/tests.rs | — | role_plan_required, consumed and wrong_feature name the command to run | cargo test --release -p bee --bin bee verbs::drivers |
| rfx-2 | Name a runnable fix when a feature or cell is already taken | feature.rs, claims.rs, state_group/tests.rs, cells/tests.rs, tests/concurrency.rs | — | startFeature names bind or close; CLAIMED says who holds it and what to run | state_group, cells and concurrency tests |
| rfx-3 | Name a writable path or a merge command in write-guard denies | hook_local.rs, write_guard/tests.rs | — | a main path deny names the worktree path; another worktree's deny names the merge | cargo test --release -p bee --bin bee hooks::write_guard |
| rfx-4 | Show the mismatch and the fix in a preview-packet refusal | plan_packets.rs | — | the refusal shows expected vs got and the command that reopens the shape | cargo test --release -p bee --bin bee verbs::state_group::plan_packets |

```json
[
  {
    "id": "rfx-1",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name a runnable fix in the deployment dispatch refusals",
    "deps": [],
    "decisions": ["D1", "D2", "b9a6dfe6-4762-4025-9918-910a371a8a24", "d361a53f-2317-4fca-b919-be571e88bc73", "66f4fd2d-2327-4efd-a3af-942936133e6d"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2, rewrite only the `fix` strings of these refusals in packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs. Find each with rg: (1) `r.insert(\"fix\".into(), Value::String(\"stage deployment requires an approved v2 plan with a role plan.\".into()));` (about line 1635, reason role_plan_required): name the plan file `docs/history/<feature>/plan.md` with the resolved feature, its `## Role assignments` section, and the next command `bee gate --preview`. (2) both `this dispatch authorization has already been consumed; dispatch a new deployment run.` sites (about lines 4220 and 4556, reason deploy_authorization_consumed): name `bee dispatch prepare --stage deployment` with every flag value the function holds (feature, release version, runtime when known); use an angle-bracket placeholder only for a value it does not hold. (3) `feature \\\"{feat}\\\" lane file not found in store.` (about line 4411, reason deploy_authorization_wrong_feature) and `dispatch record is missing feature name.` (same function): name `bee state lanes` to list live lanes, then `bee dispatch prepare --stage deployment --feature <live lane>`. Keep every `reason` key, `type`, and return path unchanged (D2). Tests red first in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs: for each rewritten refusal, assert the reason key is unchanged and the fix contains the named command (`bee gate --preview`, `bee dispatch prepare --stage deployment`, `bee state lanes`). Find existing tests with `rg -n \"role_plan_required|deploy_authorization_consumed|deploy_authorization_wrong_feature\" packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs` and update any that pin the old text. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::drivers",
    "must_haves": {
      "truths": [
        "role_plan_required names the feature plan file and bee gate --preview",
        "deploy_authorization_consumed names bee dispatch prepare --stage deployment with the values the code holds",
        "deploy_authorization_wrong_feature names bee state lanes and a dispatch prepare for a live lane",
        "every reason key and return path is unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "rewritten fix strings at the four refusal families"}
      ],
      "key_links": ["drivers tests assert each new fix command beside the unchanged reason key"],
      "prohibitions": ["No change to when any refusal fires", "No change to any reason key"]
    }
  },
  {
    "id": "rfx-2",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name a runnable fix when a feature or cell is already taken",
    "deps": [],
    "decisions": ["D1", "D2", "D4", "b9a6dfe6-4762-4025-9918-910a371a8a24", "d361a53f-2317-4fca-b919-be571e88bc73", "b1be5bf9-cfad-4144-ad21-7564f927bb07", "66f4fd2d-2327-4efd-a3af-942936133e6d"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/tests/concurrency.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2. In packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs the startFeature refusal (about line 311) ends `FIX: close or resolve that workflow before starting a new one for the same feature.`; replace that FIX with two runnable commands built from the values in hand: continue it with `bee state session bind --lane <feature>` (add `--session-id <id>` only if the function holds the session id), or end it with `bee state workflows close` naming the workflow id (check `bee state workflows close --help` for its exact flags). Per D4, in packages/bee-rs/crates/bee/src/verbs/cells/claims.rs the CLAIMED refusal (about line 751, `is already claimed by session \\\"{owner}\\\" ({}).`): when the holder session equals the calling session, say the caller already holds this claim and name the next step without --claim (`bee dispatch prepare --cell <id>` with the cell id); otherwise name `bee cells claim-next` to pick other work. Thread the caller session into that function only if it is not already there. Keep the code `CLAIMED` and every return path unchanged (D2). Tests red first: in packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs for startFeature (both commands present), and in packages/bee-rs/crates/bee/src/verbs/cells/tests.rs for CLAIMED with the same session and with another session. Find pins of the old text with `rg -n \"live workflow already exists|already claimed by session\" packages/bee-rs/crates/bee/src packages/bee-rs/crates/bee/tests` and update them, including tests/concurrency.rs if it pins the text. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group && cargo test --release -p bee --bin bee verbs::cells && cargo test --release -p bee --test concurrency",
    "must_haves": {
      "truths": [
        "the startFeature refusal names bee state session bind and bee state workflows close",
        "a CLAIMED refusal held by the calling session says so and names the next step without --claim",
        "a CLAIMED refusal held by another session names bee cells claim-next",
        "the CLAIMED code and every return path are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs", "substantive": "startFeature fix names bind and close"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs", "substantive": "CLAIMED text branches on the holder"}
      ],
      "key_links": ["the CLAIMED text reads the caller session it compares against the holder"],
      "prohibitions": ["No change to when a claim is refused", "No change to the CLAIMED code"]
    }
  },
  {
    "id": "rfx-3",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name a writable path or a merge command in write-guard denies",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "b9a6dfe6-4762-4025-9918-910a371a8a24", "d361a53f-2317-4fca-b919-be571e88bc73", "cb57963d-b94d-4a67-bfc1-2d724fa2a569", "66f4fd2d-2327-4efd-a3af-942936133e6d"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3. In packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs two deny texts sit in one function: (a) `this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there.` (about line 552): replace the FIX with the same relative path inside the current worktree — compute the target's path relative to main_root and join it onto the current worktree root the function already resolved; say `write <that path> instead`. (b) `it resolves inside worktree \\\"{id}\\\". FIX: open a session with cwd={worktree_root} to work there, or merge it back from main via `bee worktree merge --id {id}`.` (about line 566): drop the open-a-session clause; the FIX names `bee worktree merge --id {id}` run from main, and says the file belongs to that worktree. Keep the opening words `bee write guard denied this target: it could not be canonically contained inside the physical worktree` unchanged, keep when each deny fires unchanged (D2). Tests red first in packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs: a main-checkout path from a worktree session yields a deny containing the in-worktree path; another worktree's path yields a deny containing `bee worktree merge --id` and not `open a session`. Find existing pins with `rg -n \"session rooted there|open a session with cwd\" packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` and update them. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee hooks::write_guard",
    "must_haves": {
      "truths": [
        "a main-checkout path denied in a worktree session names the same relative path inside the current worktree",
        "another worktree's path deny names bee worktree merge --id and never open a session",
        "when each deny fires is unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs", "substantive": "both deny texts carry an actionable fix"}
      ],
      "key_links": ["the in-worktree path is computed from the target relative to main_root"],
      "prohibitions": ["No change to which targets are denied", "No change to the deny opening words"]
    }
  },
  {
    "id": "rfx-4",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Show the mismatch and the fix in a preview-packet refusal",
    "deps": [],
    "decisions": ["D1", "D2", "b9a6dfe6-4762-4025-9918-910a371a8a24", "d361a53f-2317-4fca-b919-be571e88bc73", "66f4fd2d-2327-4efd-a3af-942936133e6d"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2. In packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs the eight refusals `addCells: cell \\\"{id}\\\" differs from approved preview packet (<field> mismatch).` (about lines 728-784, fields action, verify, files, read_first, must_haves, title, lane, role) name no fix. Keep each first sentence byte-identical, then append: the expected value from the approved packet and the incoming value, each cut to 120 characters, and `FIX: copy <field> for cell <id> from the approved packet in docs/history/<feature>/plan.md, or reopen the shape with bee gate --name shape --approved false` (use the feature the function holds; a placeholder only if it holds none). Prefer one small local helper over eight copies of the same format. Keep the refusal trigger and return type unchanged (D2). Tests red first in the inline test module of the same file: a mismatch refusal still starts with the old sentence and contains the expected value, the incoming value and `bee gate --name shape --approved false`. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group::plan_packets",
    "must_haves": {
      "truths": [
        "a preview mismatch refusal keeps its first sentence and adds expected and incoming values",
        "a preview mismatch refusal names copying from plan.md or bee gate --name shape --approved false"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "mismatch refusals carry values and a fix"}
      ],
      "key_links": ["every field mismatch uses the same helper"],
      "prohibitions": ["No change to when a mismatch refuses", "No change to the first sentence"]
    }
  }
]
```

## Test matrix
- Happy: each refusal's test sees its named command or path. Pass when every verify command is green.
- Edge: CLAIMED by the same session vs another session; a main path vs another worktree's path. Pass when both branches have a test and both are green.
- Error: every refusal still fires where it fired before. Pass when the existing refusal tests stay green with only their text pins updated.

## Open Questions
(none)

## Out of scope
The allowed-operations packet, identity-derived actor, record and directory
(slice 3), and guard accuracy for paths outside every checkout.
