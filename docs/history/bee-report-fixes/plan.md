---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: bee-report-fixes

## Summary

Four open bee-report issues are still live in the current code. Each one
blocks a normal flow in a host project:

- #154 — after `bee cells unclaim`, re-claiming the same cell with the same
  worker through `dispatch prepare --claim` refuses on the worker's own hold.
- #155 — `bee worktree merge` fails with `WORKTREE_MERGE_ABORT_FAILED` when
  main has uncommitted `.bee` store changes.
- #156 — `bee onboard --apply` rewrites `.bee/bin/prompts/*` to the new
  version but leaves an old `.bee/bin/bee`, so every `dispatch prepare`
  refuses with prompt skew.
- #157 — a native (Agent tool) deploy-stage dispatch gets the read-only
  gather prompt and no `BEE_DISPATCH_ID`, so the deploy worker cannot run
  `scripts/release.sh` under its permit.

Issue #69 (a cross-project dashboard) is a new product, not a bug. It is out
of scope here and needs its own shaping.

Mode: `standard` — flags: multi-domain (reservations, worktree merge,
onboarding, dispatch), public-contracts (dispatch payload, onboard plan).
Four disjoint cells run in parallel; each writes its red test first.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | reserve filters conflicts by agent before the O_EXCL lease create | read | packages/bee-rs/crates/bee/src/verbs/reservations/reserve.rs:315 | `let overlap_conflicts: Vec<Resv> = find_conflicts(&control_root, &trimmed_agent, &p.path, now)?` |
| 2 | unclaim flips the cell to open and releases nothing | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1790 | `cell_map.insert("status".into(), Value::String("open".into()));` |
| 3 | worktree merge raises WORKTREE_MERGE_ABORT_FAILED when the abort does not restore main | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:700 | `"WORKTREE_MERGE_ABORT_FAILED",` |
| 4 | onboard apply copies prompts and has no binary step | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:628 | `"copy_prompt" => {` |
| 5 | the skew guard refuses dispatch on a prompt that differs from the compiled template | read | packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:265 | `"bee {cmd}: prompt skew — {} does not match the` |
| 6 | the claude Agent arm uses prompt_body, not the deployment brief | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2840 | `Value::String(format!("{marker_header}\n{prompt_body}")),` |
| 7 | the dispatch id export is added only to a command payload | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2887 | `let new_cmd = format!("export BEE_DISPATCH_ID=` |

## Evidence

- #154: `reserve_exec` (`verbs/reservations/reserve.rs`) filters
  `find_conflicts` by agent, then the O_EXCL lease create meets the
  agent's own live lease for the same cell and reports it as a conflict.
  `unclaim_cell` (`verbs/cells/handlers_close.rs`) flips the cell to open
  and never releases the cell's reservations.
- #155: issue output shows `M .bee/backlog.jsonl A .bee/cells/...` staged
  in the index when `git merge --no-ff --no-commit` runs; ort refuses.
  Code home: `verbs/worktree/` (staging and `bookkeeping_commit`).
- #156: onboard plans `copy_prompt` (`onboard/plan.rs`, `onboard/apply.rs`)
  but has no step for `.bee/bin/bee`; the skew guard is in
  `verbs/drivers/prompt.rs`.
- #157: `verbs/drivers/prepare.rs` builds `pane_prompt` from
  `deployment_prompt(ver)` for the herding and cli arms only; the claude
  Agent arm and the codex arms use `prompt_body`, and the
  `BEE_DISPATCH_ID` export is added only to a `command` payload. Cell
  rel730-1 recorded this deviation during the 2.73.0 release.

## Cells — current slice (preview)

| id | title | issue |
|---|---|---|
| brf-1 | Let the same worker re-claim a cell after unclaim (issue 154) | contract 7d3c57e8 |
| brf-2 | Merge a worktree when main has a dirty bee store (issue 155) | contract 51fe42f6 |
| brf-3 | Vendor the bee binary with its prompts on onboard (issue 156) | contract 2b906517 |
| brf-4 | Render the deployment brief for native deploy dispatches (issue 157) | contract 93ec1728 |

```json
[
  {
    "id": "brf-1",
    "feature": "bee-report-fixes",
    "title": "Let the same worker re-claim a cell after unclaim (issue 154)",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "7d3c57e8-d21d-4455-8698-8b1367ee3a56"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/reservations/reserve.rs",
      "packages/bee-rs/crates/bee/src/verbs/reservations/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "docs/history/bee-report-fixes/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue 154. Repro: claim a cell with worker W through `dispatch prepare --claim` (it reserves the cell's files), run `bee cells unclaim --id <cell>`, then run the same `dispatch prepare --claim` again with the same worker W: it refuses with `reservation conflict ... - W holds <path> (cell <cell>)` — a conflict on its own hold. Cause: reserve_exec in verbs/reservations/reserve.rs excludes the same agent from find_conflicts, but the O_EXCL lease create then meets the agent's own live lease and reports it as a conflict; unclaim_cell in verbs/cells/handlers_close.rs never releases the cell's reservations. Contract (decision 7d3c57e8): (1) reserving a path that already carries a live lease of the SAME agent for the SAME cell succeeds (ok true; refresh the lease expiry) instead of conflicting; a different agent or a different cell still conflicts exactly as before. (2) `bee cells unclaim` releases the reservations held for that cell (all agents' leases whose cell is that id), fail-open on release errors after the status flip. Write the red test first for each point (watch it fail for the reported reason), then fix. The prepare.rs unwind ladder stays as is.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- reservations cells::tests",
    "must_haves": {
      "truths": [
        "the same agent re-reserving its own live lease for the same cell gets ok true",
        "a different agent or a different cell still gets a conflict",
        "cells unclaim releases the reservations held for that cell"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/reservations/reserve.rs",
          "substantive": "same-agent same-cell lease is idempotent"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
          "substantive": "unclaim releases the cell's reservations"
        }
      ],
      "key_links": [
        "dispatch prepare --claim calls reserve_path_atomic for each cell file"
      ],
      "prohibitions": [
        "Do not weaken cross-agent or cross-cell conflict detection",
        "No code comments"
      ]
    },
    "behavior_change": true,
    "status": "open",
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "brf-2",
    "feature": "bee-report-fixes",
    "title": "Merge a worktree when main has a dirty bee store (issue 155)",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "51fe42f6-5a48-4b9c-a08e-0f7390a4ecd0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "read_first": [
      "docs/history/bee-report-fixes/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue 155. Repro: main has uncommitted changes under .bee/ (for example .bee/backlog.jsonl modified and a new .bee/cells/<id>.json) and the feature branch also changes those store files; `bee worktree merge --id <wt> --json` fails with WORKTREE_MERGE_ABORT_FAILED: `git merge --no-ff --no-commit wt/<feature>` refuses with 'Your local changes to the following files would be overwritten by merge' and the status shows the store files STAGED (`M  .bee/backlog.jsonl`, `A  .bee/cells/...`). Suspected cause: the merge stages the store into the index but the bookkeeping commit does not commit those staged files before git merge runs. Find the staging and bookkeeping_commit path under verbs/worktree/ (phases.rs and its siblings; the file list is a guess, touch only the worktree module), reproduce it in a test against a real temp git repo first and watch it fail for the reported reason, then fix. Contract (decision 51fe42f6): with a dirty .bee store on main, `bee worktree merge` succeeds — the store changes are committed in the bookkeeping commit (or otherwise never left staged in the index) before git merge starts; a merge that fails for another reason still restores main as found; no non-store dirty file is ever swept into a commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- worktree",
    "must_haves": {
      "truths": [
        "worktree merge succeeds when main has uncommitted .bee store changes",
        "no store file is left staged in the index when git merge starts",
        "a failed merge still restores main as found"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
          "substantive": "store changes committed before git merge"
        }
      ],
      "key_links": [
        "bookkeeping_commit runs before git merge --no-ff --no-commit"
      ],
      "prohibitions": [
        "Do not commit non-store dirty files",
        "No code comments"
      ]
    },
    "behavior_change": true,
    "status": "open",
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "brf-3",
    "feature": "bee-report-fixes",
    "title": "Vendor the bee binary with its prompts on onboard (issue 156)",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "2b906517-6167-4860-ab29-681a2b179cd8"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/bee-report-fixes/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
      "packages/bee-rs/crates/bee/src/onboard/source.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue 156. Repro: a host has .bee/bin/bee at an older version (2.67.0); running a newer bee `onboard --repo-root <host> --apply` from the beehive source rewrites .bee/bin/prompts/* to the new templates but leaves .bee/bin/bee old, so every `bee dispatch prepare` in the host refuses with 'prompt skew — .bee/bin/prompts/worker-cell.md does not match the worker-cell template compiled into this binary'. Cause: onboard/plan.rs plans copy_prompt but nothing for .bee/bin/bee. Contract (decision 2b906517): onboard plans a new item (for example `copy_binary`, path `.bee/bin/bee` or `.bee/bin/bee.exe` on Windows) when the host's vendored binary is missing-but-hooks-expect-it or its bytes differ from the running binary (std::env::current_exe), and apply copies it atomically (temp file in the same dir, then rename, keep the executable mode). It must never plan or write when the running binary IS the target file (same canonical path or same bytes — self-onboard of the beehive checkout, and a worktree whose .bee/bin/bee is a symlink into another checkout; see the current_exe warning in onboard/source.rs). The item shows in the dry-run plan like every other item. Write red tests first (plan item present for a stale host binary; absent when bytes match; absent when target is the running file), then fix. Keep current_exe injectable for tests.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- onboard",
    "must_haves": {
      "truths": [
        "onboard apply leaves .bee/bin/bee and .bee/bin/prompts from one build",
        "onboard never rewrites the binary when the source is the same file",
        "the binary copy shows in the dry-run plan"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "plans the binary copy"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/apply.rs",
          "substantive": "atomic binary copy"
        }
      ],
      "key_links": [
        "the prompt-skew guard in verbs/drivers/prompt.rs compares vendored prompts to the binary's compiled templates"
      ],
      "prohibitions": [
        "Do not replace a binary through a symlink into another checkout",
        "No code comments"
      ]
    },
    "behavior_change": true,
    "status": "open",
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "brf-4",
    "feature": "bee-report-fixes",
    "title": "Render the deployment brief for native deploy dispatches (issue 157)",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "93ec1728-b25b-4380-b2c1-bfd95092ef6c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "docs/history/bee-report-fixes/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/model_guard.rs",
      "scripts/release.sh"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "GitHub issue 157. Repro: with team.claude.deploy on a native model (sonnet), `bee dispatch prepare --runtime claude --kind gather --role deploy --stage deployment --feature <f> --release-version 2.73.0 --json` returns an Agent payload whose prompt says 'Gather: ... Read-only — never write ...' and carries no BEE_DISPATCH_ID, while the herding shape renders the 'Deploy-stage execution brief' with the id exported. Cause in verbs/drivers/prepare.rs: pane_prompt uses deployment_prompt(ver) only for the herding/cli arms; the claude Agent arm, the codex arms and the escalated arm use prompt_body; the BEE_DISPATCH_ID / BEE_RELEASE_VERSION / BEE_SESSION_ID export is added only to a `command` payload. Contract (decision 93ec1728): on a deployment-stage dispatch every transport carries the deployment brief; a payload with no `command` (Agent, spawn_agent) carries the brief plus explicit lines naming the dispatch id, release version and issuer session and telling the worker to export BEE_DISPATCH_ID, BEE_RELEASE_VERSION and BEE_SESSION_ID on the shell that runs scripts/release.sh; the Agent payload names a write-capable bee agent type (never bee-gather, which has no Bash), and the model-guard hook must accept that payload (check hooks/model_guard.rs; if it needs a change, report [BLOCKED] naming it rather than editing outside your files). Herding and cli payloads stay byte-identical. Write red tests first (Agent payload prompt has the deploy brief and dispatch id; agent type is not bee-gather; herding payload unchanged), then fix.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- drivers",
    "must_haves": {
      "truths": [
        "a native deploy-stage Agent payload carries the deployment brief and the dispatch id",
        "the native deploy payload names a write-capable agent type",
        "herding and cli deploy payloads stay byte-identical"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
          "substantive": "deployment brief on every transport"
        }
      ],
      "key_links": [
        "scripts/release.sh runs bee dispatch authorize with BEE_DISPATCH_ID"
      ],
      "prohibitions": [
        "Do not change non-deployment payloads",
        "No code comments"
      ]
    },
    "behavior_change": true,
    "status": "open",
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "four Rust fixes with red-first tests"},
    {"stage": "tests", "classification": "not-applicable", "role": "test", "reason": "each code cell writes its own tests red-first"},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "the reads were done at plan time; see Evidence"},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "no lookup remains"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "code covers it"},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "this plan"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "review is user-invoked"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "four reproduced bugs with located causes"},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "a concept doc names the changed behavior", "reason": "capture runs at close"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "four parallel disjoint cells"},
    {"stage": "design", "classification": "not-applicable", "role": "design", "reason": "no rendered UI changes"},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "no hat wave: reproduced bugs with located causes"},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "as above"},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "as above"},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "as above"},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "as above"},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "as above"},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "as above"}
  ]
}
```
