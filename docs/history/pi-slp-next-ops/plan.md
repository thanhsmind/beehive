---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: pi-slp-next-ops

## Summary
A model asks bee "where am I and what do I run next?" in two places: `bee
orient` and the short hint bee adds to every turn. Today orient can answer
for the wrong feature, its next command is either a read-only listing or
"open your session at …", which no agent can do, and the per-turn hint has
no command at all. After this change orient reads the session's own feature,
names one command that runs, and says which folder to run it from; the
per-turn hint shows that same command. The worktree messages that said "open
your session at" now name `bee worktree enter --id <id>`.

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: one shared next-operation function, two readers, and a text-only cell; each with its own tests.

## Requirements (from CONTEXT.md)
- D1: a lane-bound session's orient reads that lane's record (resolve_pipeline); unbound stays on the default record.
- D2: one shared function returns the next command (worktree enter / dispatch wave / handoff show / null) and run_from.
- D3: prompt-context adds `run: <command> (from <run_from>)` when D2 yields a command.
- D4: worktree-first denies and route notices name `bee worktree enter --id`; the generic containment deny says outside-project paths belong to the user.

## Load-bearing claims
Labels: `read` or `ran`. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | orient's command for ready cells is a browse | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:40 | return json!("bee cells ready --json"); |
| 2 | orient's worktree guidance is not runnable | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:128 | json!(format!("open your session at {worktree_root}")) |
| 3 | orient's status reads the default record | read | packages/bee-rs/crates/bee/src/verbs/status_full/store.rs:270 | let parsed = rj(ctx, &ctx.root.join(".bee").join("state.json"))?; |
| 4 | build_status starts from that read | read | packages/bee-rs/crates/bee/src/verbs/status_full/build.rs:111 | let state = read_state_full(ctx)?; |
| 5 | the lane-aware resolver exists | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:461 | pub(crate) fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Pipeline { |
| 6 | the per-turn hint prints prose only | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:940 | lines.push(format!("next: {}", jsjson::js_to_string(&next_action))); |
| 7 | the granted-worktree deny asks for an impossible act | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:817 | FIX: open your session at {grant_root} and make this edit there |
| 8 | the no-grant deny does too | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:907 | then open your session at the printed worktree path and make |
| 9 | the route notice does too | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:739 | NEXT: open your session at {worktree_root} |
| 10 | the route no-grant notice does too | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:748 | then open your session at the printed worktree path |
| 11 | the generic deny names no owner for outside paths | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:19 | FIX: use a plain in-worktree path without traversal |
| 12 | worktree enter takes --id | ran | .bee/bin/bee worktree enter --help | --id* (str) — The granted worktree's git-verified id |
| 13 | dispatch wave exists for a feature | ran | .bee/bin/bee --help --names | bee dispatch wave — Prepare cells of the CURRENT schedule wave |

## Discovery
A herding gather mapped every `next` producer and reader (report at main
`.bee/mailbox/job-1791000191214-2743467-1/report-1.md`). Pi reads the
preamble once and the `prompt-context` delta every turn
(`.pi/extensions/bee-guard/events.ts`), and never runs a returned command.

## Approach
Playbook: `skills/bee-planning/playbooks/feature.md` (class feature).

nop-1 adds one function in `verbs/status_full/orient.rs`, `next_operation`,
that returns `{command, run_from}` from the facts orient already holds, and
makes `build_orient` use the session's lane record (D1, D2). nop-2 makes
`prompt-context` call the same function for its record and print the run
line (D3). nop-3 rewrites the worktree texts (D4). One home for the command
rule: nop-2 calls nop-1's function, never a copy.

Rejected: a new `bee next` verb — orient already answers "what next"; a
second verb is a second home. Rejected: making the Pi belt run the command
itself — pi-1-0-upgrade D5 keeps decisions in the CLI, and running a
dispatch is already `bee_dispatch`'s job.

Risk map:
- orient record switch / MEDIUM / nop-1 / lane-bound and unbound orient tests
- shared function used by a hook / MEDIUM / nop-2 / prompt-context test with and without a command
- text rewrites / LOW / nop-3 / write_guard and workflows tests

Waves: nop-1 and nop-3 in parallel (disjoint files); nop-2 after nop-1 (it calls nop-1's function).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"nop-1 to nop-3 change Rust with their tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own red-first tests."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"The leader captures at close."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"The next-step map gather ran before the plan."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Standard lane; the hat wave is the plan check."},
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
| 1 | orient and the per-turn hint name one runnable next command for the session's own feature | Transcripts show models act on the wrong record and flail on prose | a lane-bound `bee orient --json` shows its lane and a runnable `next.command` with `run_from` | Slice 4: measurement |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| nop-1 | Give orient one runnable next command from the session's own record | orient.rs, store.rs, build.rs, status_full/tests.rs | — | orient shows the bound lane and a runnable command with run_from | status_full tests |
| nop-2 | Show the runnable next command in the per-turn hint | prompt_context.rs | nop-1 | each turn's hint carries `run: …` when there is a command | prompt_context tests |
| nop-3 | Name bee worktree enter in worktree denies and notices | hook_local.rs, workflows.rs, write_guard/tests.rs, state_group/tests.rs | — | no message says "open your session at" | write_guard and state_group tests |

```json
[
  {
    "id": "nop-1",
    "feature": "pi-slp-next-ops",
    "lane": "standard",
    "role": "code",
    "change_class": "feature",
    "title": "Give orient one runnable next command from the session's own record",
    "deps": [],
    "decisions": ["D1", "D2", "b3f14c45-93a6-4d4e-855b-6518f4933ebb", "fbde3d6f-8a77-47ff-a90c-d08711a28db7", "80987a3f-83ae-4b51-a954-f40d1f1a259c"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/build.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-next-ops/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D1: build_orient (verbs/status_full/orient.rs) gets its status from build_status(ctx, false), which starts with read_state_full(ctx) reading ctx.root/.bee/state.json (store.rs about line 270). For orient only, when the calling session is bound to a lane, read that lane's record instead, using the same resolution as resolve_pipeline in hooks/session_preamble/state.rs (about line 461, `pub(crate) fn resolve_pipeline(root: &Path, session_id: Option<&str>) -> Pipeline {`); find how prompt_context.rs obtains the session id it passes there and do the same. An unbound session, or a resolve that falls back to the default, reads the default record exactly as today. `bee status` keeps its current behavior. Per D2: replace orient_next_command (orient.rs about line 35) with one pub(crate) function next_operation returning the command and run_from: (a) when orient's worktree context says location main with a granted worktree id, `bee worktree enter --id <id>` with the real id; (b) a handoff keeps `bee state handoff show --json`; (c) ready cells with the execution gate approved give `bee dispatch wave --runtime <runtime> --feature <feature> --json` with the real feature (runtime stays a placeholder); (d) otherwise null. Keep the existing priority order between these unless a test pins another. run_from is the absolute main checkout root (the control root) whenever the command is non-null. build_orient adds next.run_from (null when command is null). Change the worktree guidance string at orient.rs about line 128 (`open your session at {worktree_root}`) to name the same `bee worktree enter --id <id>` command. Tests red first in verbs/status_full/tests.rs: a lane-bound session's orient reports its lane's feature, not the default record's; each next_operation branch returns its command and run_from; the guidance no longer starts with `open your session at` (update the pin at tests.rs about line 772). Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee verbs::status_full",
    "must_haves": {
      "truths": [
        "a lane-bound session's orient reports its lane's feature and phase",
        "an unbound session's orient reads the default record as before",
        "orient next.command is bee worktree enter --id, bee state handoff show --json, bee dispatch wave --runtime <runtime> --feature <feature> --json, or null",
        "orient next.run_from is the main checkout root when next.command is non-null",
        "orient worktree guidance never says open your session at"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs", "substantive": "pub(crate) next_operation used by build_orient"}
      ],
      "key_links": ["build_orient reads the lane record through the resolve_pipeline resolution for a bound session"],
      "prohibitions": ["No change to bee status output", "No second copy of the next-command rule"]
    }
  },
  {
    "id": "nop-2",
    "feature": "pi-slp-next-ops",
    "lane": "standard",
    "role": "code",
    "change_class": "feature",
    "title": "Show the runnable next command in the per-turn hint",
    "deps": ["nop-1"],
    "decisions": ["D2", "D3", "fbde3d6f-8a77-47ff-a90c-d08711a28db7", "42a3429b-0408-46c4-ba43-95b813810044", "80987a3f-83ae-4b51-a954-f40d1f1a259c"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-next-ops/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3: in packages/bee-rs/crates/bee/src/hooks/prompt_context.rs the hint pushes `lines.push(format!(\"next: {}\", jsjson::js_to_string(&next_action)));` (about line 940). Right after that line, when nop-1's pub(crate) next_operation function (verbs/status_full/orient.rs) yields a non-null command for the same record the hint already resolved, push one line `run: <command> (from <run_from>)`. Call that function; never copy its rule (D2). If calling it needs facts the hook does not have, build them with the smallest read of the same sources orient uses, and keep the hook fast. Nothing else in the hint changes. Tests red first in the #[cfg(test)] mod tests of the same file: a record with ready cells under an approved execution gate yields the run line with `bee dispatch wave`; a record with nothing to run yields no run line. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee prompt_context",
    "must_haves": {
      "truths": [
        "the per-turn hint prints run: <command> (from <run_from>) when next_operation yields a command",
        "the per-turn hint prints no run line when next_operation yields null"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs", "substantive": "run line built from next_operation"}
      ],
      "key_links": ["prompt_context calls verbs::status_full next_operation"],
      "prohibitions": ["No copy of the next-command rule in the hook", "No other hint line changes"]
    }
  },
  {
    "id": "nop-3",
    "feature": "pi-slp-next-ops",
    "lane": "standard",
    "role": "code",
    "change_class": "refactor",
    "title": "Name bee worktree enter in worktree denies and notices",
    "deps": [],
    "decisions": ["D4", "dfd7ae10-e654-47c1-852f-49848bd930e4", "80987a3f-83ae-4b51-a954-f40d1f1a259c"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"
    ],
    "read_first": [
      "docs/history/pi-slp-next-ops/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4, text only; when each message fires does not change. In packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs: (1) the worktree-first deny with a grant (about line 817, `FIX: open your session at {grant_root} and make this edit there, then land it from main with ...`) names `bee worktree enter --id {grant_id}` and to make the edit in {grant_root} after it; keep the merge sentence and the override sentence. (2) the no-grant deny (about line 907, `FIX: run \\`bee worktree new --feature {feature}\\`, then open your session at the printed worktree path and make this edit there.`) names `bee worktree new --feature {feature}`, then `bee worktree enter --id <the id it prints>`. (3) GENERIC_CONTAINMENT_MESSAGE and GENERIC_BASH_CONTAINMENT_MESSAGE (about lines 17-23): keep the first sentence; the FIX adds that a path outside this project belongs to the user — ask them to change it, never write it through another tool. In packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs the route notices (about lines 739 and 748) replace `open your session at {worktree_root}` with `bee worktree enter --id {id}`, and `then open your session at the printed worktree path` with `then bee worktree enter --id <the id it prints>`. Tests red first: in hooks/write_guard/tests.rs assert each deny names `bee worktree enter --id` (or the outside-project sentence) and none says `open your session at`; in verbs/state_group/tests.rs the same for both notices. Find pins with `rg -n \"open your session at|in-worktree path without traversal\" packages/bee-rs/crates/bee/src` and update them. Write no code comments.",
    "verify": "cd packages/bee-rs && cargo test --release -p bee --bin bee hooks::write_guard && cargo test --release -p bee --bin bee verbs::state_group",
    "must_haves": {
      "truths": [
        "both worktree-first denies name bee worktree enter --id and never open your session at",
        "both route worktree notices name bee worktree enter --id and never open your session at",
        "the generic containment denies say a path outside the project belongs to the user"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs", "substantive": "deny texts name worktree enter or the outside-project owner"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs", "substantive": "notices name worktree enter"}
      ],
      "key_links": ["write_guard and state_group tests pin the new texts"],
      "prohibitions": ["No change to when any deny or notice fires", "Writes outside the project stay denied"]
    }
  }
]
```

## Test matrix
- Happy: a lane-bound orient shows its lane and a runnable command; the hint shows the run line. Pass when status_full and prompt_context tests are green.
- Edge: unbound orient unchanged; null command means no run line and run_from null. Pass when those tests are green.
- Error: every deny and notice still fires where it fired. Pass when write_guard and state_group tests stay green with only text pins updated.

## Open Questions
(none)

## Out of scope
Deploy-permit identity binding and gate `--actor` labels (CONTEXT.md), and
slice 4, the measurement.
