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
- D1: seven refusal families each end with one runnable bee command or one concrete path built from values the code holds. Plan reading (hat wave): exactly one command per refusal; a value the code does not hold is an angle-bracket placeholder; a command that refuses inside a worktree says "run from main".
- D2: text only; refusal triggers, reason and code keys, exit codes unchanged.
- D3 (amended after the hat wave): a main-path deny names the in-worktree path (a `.bee/` path names `bee --help --json` instead); another worktree's deny says to leave the file and write only inside the caller's own worktree root; never "open a session", never a merge of another worktree.
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
| 10 | the expired authorization names no command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4351 | dispatch a new deployment run. |
| 11 | the missing-feature record names no command | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:4400 | dispatch record is missing feature name. |
| 12 | the worktree helper returns main root and id, not the worktree root | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:466 | pub(crate) fn derive_current_worktree(root: &str) -> R<Option<(String, String)>> { |
| 13 | dispatch prepare requires --runtime | ran | .bee/bin/bee dispatch prepare --help | --runtime* (str) — Target runtime the payload is shaped for. |
| 14 | a cell dispatch requires --cell and --worker | ran | .bee/bin/bee dispatch prepare --help | --worker (str) — Requesting worker identity — required when --kind cell |
| 15 | session bind takes --lane | ran | .bee/bin/bee state session bind --help | --lane (str) — Lane feature name to bind the session to. |
| 16 | gate has --preview | ran | .bee/bin/bee gate --help | --preview (boo) — Preview current-slice cell packets |
| 17 | the CLAIMED text is pinned in two product docs | read | docs/product-description/lifecycle/execution.md:40 | is already claimed by session "<owner>" (<expiry>). |
| 18 | bee state lanes lists the lane records | ran | .bee/bin/bee state lanes --help | List every per-feature lane record with its phase, gates |

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

One command per refusal (hat wave): role_plan_required → `bee gate --preview`
after naming the plan file; consumed and expired → the deployment
`bee dispatch prepare` form; wrong_feature and missing feature →
`bee state lanes`, run from main; startFeature → `bee state session bind
--lane <feature>`; CLAIMED by the caller → the full cell `dispatch prepare`
form; CLAIMED by another session → `bee cells claim-next`; preview mismatch
→ the plan.md path. Rejected: echoing expected and incoming values in the
preview mismatch — not asked by D1, and multi-line fields need cutting.

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
| rfx-2 | Name a runnable fix when a feature or cell is already taken | feature.rs, claims.rs, state_group/tests.rs, cells/tests.rs, tests/concurrency.rs, two product docs | — | startFeature names session bind; CLAIMED says who holds it and what to run | state_group, cells and concurrency tests |
| rfx-3 | Name a writable path in write-guard denies | hook_local.rs, write_guard/tests.rs | — | a main path deny names the worktree path; another worktree's deny names the caller's own worktree root | cargo test --release -p bee --bin bee hooks::write_guard |
| rfx-4 | Name the plan file in a preview-packet refusal | plan_packets.rs | — | the refusal names the field and the plan file to copy it from | cargo test --release -p bee --bin bee verbs::state_group::plan_packets |

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
    "decisions": [
      "D1",
      "D2",
      "b9a6dfe6-4762-4025-9918-910a371a8a24",
      "d361a53f-2317-4fca-b919-be571e88bc73",
      "66f4fd2d-2327-4efd-a3af-942936133e6d"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2, rewrite only the `fix` strings of these refusals in packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs, one command each. Find each with rg. (1) `stage deployment requires an approved v2 plan with a role plan.` (about line 1635, reason role_plan_required): name `docs/history/<feature>/plan.md` with the resolved feature, its `## Role assignments` section, then the one command `bee gate --preview`. (2) `this dispatch authorization has already been consumed; dispatch a new deployment run.` (about lines 4220 and 4556, reason deploy_authorization_consumed) and `dispatch authorization expired (> 2 hours old); dispatch a new deployment run.` (about line 4351): name one command, `bee dispatch prepare --runtime <runtime> --kind <kind> --stage deployment --feature <feature> --release-version <version> --json`, filling every value the enclosing function holds (it holds release_version at the first consumed site; it does not hold runtime) and leaving an angle-bracket placeholder for each value it does not hold. Check `.bee/bin/bee dispatch prepare --help` and an existing deployment dispatch test for the exact required flags and the kind a deployment dispatch uses; the command must be one prepare would accept once placeholders are filled. (3) `feature \\\"{feat}\\\" lane file not found in store.` (about line 4411) and `dispatch record is missing feature name.` (about line 4400), reason deploy_authorization_wrong_feature: name the one command `bee state lanes` and say to run it from main, then dispatch for a feature it lists. Keep every `reason` key, `type`, and return path unchanged (D2). Tests red first in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs: for each rewritten refusal, assert the reason key is unchanged and the fix contains its command (`bee gate --preview`, `bee dispatch prepare --runtime`, `bee state lanes`). Find existing pins with `rg -n \"role_plan_required|deploy_authorization_consumed|deploy_authorization_wrong_feature|authorization expired\" packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs` and update any that pin the old text. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::drivers",
    "must_haves": {
      "truths": [
        "role_plan_required names the feature plan file and bee gate --preview",
        "deploy_authorization_consumed and the expired refusal name one bee dispatch prepare --runtime command with held values filled and placeholders for the rest",
        "deploy_authorization_wrong_feature names bee state lanes run from main",
        "every reason key and return path is unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
          "substantive": "rewritten fix strings at the four refusal families"
        }
      ],
      "key_links": [
        "drivers tests assert each new fix command beside the unchanged reason key"
      ],
      "prohibitions": [
        "No change to when any refusal fires",
        "No change to any reason key"
      ]
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
    "decisions": [
      "D1",
      "D2",
      "D4",
      "b9a6dfe6-4762-4025-9918-910a371a8a24",
      "d361a53f-2317-4fca-b919-be571e88bc73",
      "b1be5bf9-cfad-4144-ad21-7564f927bb07",
      "66f4fd2d-2327-4efd-a3af-942936133e6d"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/tests/concurrency.rs",
      "docs/product-description/lifecycle/execution.md",
      "docs/product-description/verification/lifecycle.md"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/product-description/lifecycle/execution.md",
      "docs/product-description/verification/lifecycle.md"
    ],
    "action": "Per D1 and D2, one command per refusal. In packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs the startFeature refusal (about line 311) ends `FIX: close or resolve that workflow before starting a new one for the same feature.`; replace that FIX with the one command that continues the live workflow, `bee state session bind --lane <feature> --session-id <session id>` (fill the feature; fill the session id only if the function holds it). Per D4, in packages/bee-rs/crates/bee/src/verbs/cells/claims.rs the CLAIMED refusal (about line 751, `is already claimed by session \\\"{owner}\\\" ({}).`): the calling session is already in scope in claim_cell_file as `session`. When the holder session and the caller session are BOTH present and equal, say the caller already holds this claim and name `bee dispatch prepare --runtime <runtime> --kind cell --cell <id> --worker <worker> --json` without --claim (fill the cell id); in every other case — another session, or either side sessionless — keep the holder text and name `bee cells claim-next`. Keep the code `CLAIMED`, the text before the new FIX, and every return path unchanged (D2). Tests red first: in packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs for startFeature (the bind command present), and in packages/bee-rs/crates/bee/src/verbs/cells/tests.rs for CLAIMED with the same session, another session, and a sessionless holder. Find pins of the old text with `rg -n \"live workflow already exists|already claimed by session\" packages/bee-rs/crates/bee/src packages/bee-rs/crates/bee/tests` and update them, including tests/concurrency.rs if it pins the text. Update the two product docs that pin the exact CLAIMED shape (docs/product-description/lifecycle/execution.md:40 and docs/product-description/verification/lifecycle.md:199) to the new text. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group && cargo test --release -p bee --bin bee verbs::cells && cargo test --release -p bee --test concurrency",
    "must_haves": {
      "truths": [
        "the startFeature refusal names bee state session bind --lane with the feature",
        "a CLAIMED refusal held by the calling session says so and names the full cell dispatch prepare form without --claim",
        "a CLAIMED refusal held by another session or a sessionless claim names bee cells claim-next",
        "the CLAIMED code and every return path are unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/feature.rs",
          "substantive": "startFeature fix names session bind"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/claims.rs",
          "substantive": "CLAIMED text branches on the holder"
        }
      ],
      "key_links": [
        "the CLAIMED text reads the caller session it compares against the holder"
      ],
      "prohibitions": [
        "No change to when a claim is refused",
        "No change to the CLAIMED code"
      ]
    }
  },
  {
    "id": "rfx-3",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name a writable path in write-guard denies",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "b9a6dfe6-4762-4025-9918-910a371a8a24",
      "d361a53f-2317-4fca-b919-be571e88bc73",
      "aa4db1c5-c0fa-478f-b41c-e5b5ee7bade2",
      "66f4fd2d-2327-4efd-a3af-942936133e6d"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3. In packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs two deny texts sit in one function. derive_current_worktree (about line 466) returns only (main_root, id); the current worktree root is that function's `root` parameter — canonicalize it the same way main_root and target_real are before you strip and join. (a) `this path belongs to the main checkout, not this worktree. FIX: run this from a session rooted there.` (about line 552): replace the FIX with the same relative path inside the current worktree — strip main_root from target_real and join the rest onto the canonical worktree root; say `write <that path> instead`. Exception: when the relative path starts with `.bee/`, never offer a path (that would steer a hand edit of bee state); the FIX says bee state changes only through `bee` verbs run from main and names `bee --help --json`. (b) `it resolves inside worktree \\\"{id}\\\". FIX: open a session with cwd={worktree_root} to work there, or merge it back from main via `bee worktree merge --id {id}`.` (about line 566): replace the whole FIX: the file belongs to worktree {id}; leave it, and write only inside your own worktree, naming the caller's canonical worktree root as a path. Never name a merge of that other worktree and never `open a session`. Keep the opening words `bee write guard denied this target: it could not be canonically contained inside the physical worktree` unchanged, keep when each deny fires unchanged (D2). Tests red first in packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs: a main-checkout source path from a worktree session yields a deny containing the in-worktree path; a main `.bee/state.json` yields no path and names `bee --help --json`; another worktree's path yields a deny containing the caller's own worktree root and neither `open a session` nor `bee worktree merge`. Find existing pins with `rg -n \"session rooted there|open a session with cwd\" packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs` and update them. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee hooks::write_guard",
    "must_haves": {
      "truths": [
        "a main-checkout path denied in a worktree session names the same relative path inside the current worktree",
        "a main .bee/ path deny names no path and names bee --help --json",
        "another worktree's path deny names the caller's own worktree root and never open a session or a merge of that worktree",
        "when each deny fires is unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
          "substantive": "both deny texts carry an actionable fix"
        }
      ],
      "key_links": [
        "the in-worktree path is computed from target_real relative to main_root, joined onto the canonical root parameter"
      ],
      "prohibitions": [
        "No change to which targets are denied",
        "No change to the deny opening words"
      ]
    }
  },
  {
    "id": "rfx-4",
    "feature": "pi-slp-operations",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name the plan file in a preview-packet refusal",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "b9a6dfe6-4762-4025-9918-910a371a8a24",
      "d361a53f-2317-4fca-b919-be571e88bc73",
      "66f4fd2d-2327-4efd-a3af-942936133e6d"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-operations/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1 and D2. In packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs the eight refusals `addCells: cell \\\"{id}\\\" differs from approved preview packet (<field> mismatch).` (about lines 728-784, fields action, verify, files, read_first, must_haves, title, lane, role) name no fix. check_cell_matches_approved_preview holds the feature. Keep each first sentence byte-identical, then append one fix: `FIX: copy <field> for cell <id> verbatim from the approved packet in docs/history/<feature>/plan.md.` One small local helper is fine for the eight sites. Do not echo the values. Keep the refusal trigger and return type unchanged (D2). Tests red first in the inline test module of the same file: a mismatch refusal still starts with the old sentence and contains `docs/history/<feature>/plan.md` with the real feature and the field name. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::state_group::plan_packets",
    "must_haves": {
      "truths": [
        "a preview mismatch refusal keeps its first sentence",
        "a preview mismatch refusal names copying the field from docs/history/<feature>/plan.md with the real feature"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
          "substantive": "mismatch refusals carry the plan file fix"
        }
      ],
      "key_links": [
        "every field mismatch uses the same fix wording"
      ],
      "prohibitions": [
        "No change to when a mismatch refuses",
        "No change to the first sentence",
        "No echo of field values"
      ]
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
