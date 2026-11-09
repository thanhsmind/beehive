---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: tiny-bound-delegation

## Summary

Today a leader can call a change `tiny` because its core edit is one line,
then do it alone, even when the change touches six files and rewrites
tests. With this change `tiny` means one hand-authored file and nothing
more, and bee checks it twice: `bee route` refuses a tiny route that
counts more than one file, and `bee cells cap` refuses an inline tiny cap
that changed more than one counted file unless the leader records why.
Everything larger goes to the team through `bee dispatch prepare`, because
each role runs its own model (owner preference, D1). Real tiny work keeps
its inline option.

Mode: `standard` — 2 risk flags: multi-domain, covered-contract-change
Why this is the least workflow that protects the work: a new refusal at
route and at cap changes CLI behavior that existing tests assert, and the
doctrine text spans three skills and LEADER.md; no hard-gate flag applies.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`d7959afc`): owner prefers team delegation for all non-tiny work.
- D2: tiny = one hand-authored file plus mechanical companions, no test
  rewrite, one cheap proof; small trigger unchanged.
- D3: from small up, execution is dispatched; the existing cap check
  (`--inline-reason`) stays the one escape.
- D4: cap refuses an inline tiny cell with more than one counted file
  unless `--inline-reason`.
- D5: route refuses `--lane tiny` with `--files` above 1.

## Load-bearing claims

Labels are `read` (file opened at that line) or `ran`; evidence is the exact
line or the command output.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Cap already refuses small and larger cells with no registered worker unless `--inline-reason` is given | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:482-507 | `if lane == "small" \|\| lane == "standard" \|\| lane == "high-risk" {` |
| 2 | The registered-worker test is one reusable function | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:186 | `pub(crate) fn registered_worker_for_cell(root: &Path, id: &str, worker: Option<&str>) -> MR<bool> {` |
| 3 | Route validation reads `--files` into one product-file count before the route map is built | read | packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:611-623 | `let mut product_files: Option<f64> = None;` |
| 4 | The existing bookkeeping test in full_suite.rs treats all of `docs/` and every `.md` as bookkeeping, so it is the wrong counter for D2 (a doc beside code must count) | read | packages/bee-rs/crates/bee/src/verbs/worktree/full_suite.rs:8-10 | `path.starts_with(".bee/") \|\| path.starts_with("docs/") \|\| path.ends_with(".md")` |
| 5 | The release manifest lives under docs/history, so a skill edit's regen output is not a counted file | read | packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:94 | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |
| 6 | The tiny carve-out at cap is deliberate: the A/B measures inline tiny cells | read | docs/decisions/ab-tiny-protocol.md:9-11 | `this A/B stays the single mechanism` |
| 7 | The tiny lane trigger lives in one table in bee-planning | read | skills/bee-planning/SKILL.md:36 | `` | `tiny` | 0–1 flags, ≤2 product files, one direct task | `` |
| 8 | The demotion ladder states the old tiny threshold | read | skills/bee-hive/references/scout-and-ticks.md:74 | `` (`small`: ≤3, `tiny`: ≤2) `` |
| 9 | No test asserts the old tiny wording, so the doctrine cell owes no test edit | ran | `rg -n "≤2 product\|tiny may run inline" packages/bee-rs/crates/bee/tests packages/bee-rs/crates/bee/src -g '*.rs'` | (no output) |

## Discovery

One read gather (extraction role; the read role's model provider had no
API key and its worker died) mapped the cap path, the lane read at cap and
the merge doors. It found the `--inline-reason` escape and the
registered-worker refusal for small and up (claims 1, 2), so D3 needs no
new code. No check counts `files_changed` against a lane today. The plan
check ran inline on the session model (R16b): 4 counted product files
(handlers_close.rs, cells/tests.rs, workflows.rs, state_group/tests.rs) and
zero hard-gate flags.

## Approach

Recommended path: one helper `counted_tiny_files(paths) -> Vec<String>`
returns every path not under `.bee/` or `docs/history/`. The cap path calls
it for a `tiny` cell when the trace worker is not a registered worker for
the cell and `--inline-reason` is absent; more than one counted path
refuses (D4). Route validation refuses `lane == "tiny"` with
`product_files > 1` (D5). The doctrine cell rewrites the tiny trigger and
every sentence that restates it, and names D1 as the reason for the
delegation default (D2, D3).

Rejected: a warning instead of a refusal (the trigger ran under full
bypass and would not stop for a warning); reusing full_suite.rs
`is_bookkeeping` (claim 4: it would exempt the doc-beside-code case D2
counts); a merge-time door (the cap is where files_changed becomes known
and the remedy is still cheap); removing inline tiny (claim 6).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Cap refusal blocks a legitimate inline tiny with a mechanical bump | LOW — fact: `--inline-reason` already escapes the small+ refusal at handlers_close.rs:496 and is recorded on the trace | tbd-1 | test: two counted files plus `--inline-reason` caps and records the reason |
| Existing tests cap tiny cells inline with several files | MEDIUM — fact: cells/tests.rs holds many `"lane": "tiny"` fixtures | tbd-1 | the cells test filter green after the change |
| Route refusal breaks feature start or existing route tests | LOW — fact: state_group/tests.rs:2564 routes tiny with `files` 1 | tbd-1 | state_group test filter green |
| Doctrine copies disagree after the edit | LOW | tbd-2 | `rg -n "≤2 product\|tiny\`: ≤2" skills packages/bee/LEADER.md` returns nothing; manifest check green |

Waves: tbd-1 and tbd-2 have disjoint product files and run in parallel.
Both touch hashed skill files, so both carry `regen_obligation_ack:
"wave-barrier"` and the leader runs `bee dev regen` and the manifest check
once at wave close.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"generation","reason":"tbd-1 writes Rust code and tests; the code slot's model provider (paseo-pi-flash, hive) has no API key on this machine, so the generation slot carries the code cell."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"tbd-2 rewrites doctrine text in skills, LEADER.md and the knowledge area."},
    {"stage":"fact-extraction","classification":"required","role":"extraction","reason":"The discovery gather ran on the extraction slot."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The read slot's provider has no API key; extraction carried the gather."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride each cell."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Standard lane with at most five product files and no hard-gate flag: the plan check runs inline (R16b)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline plan check (R16b)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline plan check (R16b)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"generation is used directly for implementation."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No consult needed."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"code-slot","classification":"not-applicable","role":"code","reason":"The code slot model provider (paseo-pi-flash, hive) has no API key on this machine; generation carries tbd-1."},
    {"stage":"writer-slot","classification":"not-applicable","role":"writer","reason":"docs carries the doctrine cell."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Phase plan (one slice):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | route and cap tiny-bound checks, tiny doctrine rewrite | the doctrine without the check repeats the trigger incident; the check without the doctrine refuses with no explanation | `bee route --set --lane tiny --files 2` refuses naming small; an inline tiny cap of two counted files refuses naming the reroute | hosts get it on their next refresh |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| tbd-1 | Refuse an oversize tiny at route and at inline cap | cells/handlers_close.rs, cells/tests.rs, state_group/workflows.rs, state_group/tests.rs, skills/bee-swarming/references/swarming-reference.md | — | `bee route --set --lane tiny --files 2` refuses naming `--lane small`; an inline tiny cap of two counted files refuses naming the reroute and `--inline-reason`; a dispatched tiny cap and a `.bee/` or `docs/history/` path do not count | cells and state_group test filters green |
| tbd-2 | Define tiny as one hand-authored file and name the delegation reason | skills/bee-planning/SKILL.md, skills/bee-hive/SKILL.md, skills/bee-hive/references/scout-and-ticks.md, skills/bee-hive/references/routing-and-contracts.md, skills/bee-hive/references/gates-and-delegation.md, skills/bee-hive/references/go-mode.md, skills/bee-swarming/SKILL.md, packages/bee/LEADER.md, docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md | — | every place that states the tiny trigger says one hand-authored file; LEADER.md names the owner reason for dispatching small and up; the knowledge area carries the new rule | old-wording rg empty; `bee knowledge check` 0 errors |

```json
[
  {
    "id": "tbd-1",
    "feature": "tiny-bound-delegation",
    "title": "Refuse an oversize tiny at route and at inline cap",
    "lane": "standard",
    "role": "generation",
    "deps": [],
    "decisions": ["4ee41332-a000-492e-ac2c-afd59c8efa02", "48130f16-8a89-44a5-892f-113e0b49b0f9", "4479e6fd-7614-4d52-b298-86b6ebd584c3"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs",
      "skills/bee-swarming/references/swarming-reference.md"
    ],
    "read_first": [
      "docs/history/tiny-bound-delegation/CONTEXT.md",
      "docs/history/tiny-bound-delegation/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs"
    ],
    "affects_skills": ["skills/bee-swarming/references/swarming-reference.md"],
    "affects_specs": [],
    "regen_obligation_ack": "wave-barrier",
    "action": "Make the tiny bound a check at route and at cap (D4 contract tiny-cap-bound 4ee41332, D5 contract tiny-route-bound 48130f16, D2 4479e6fd). Cap (handlers_close.rs, beside the small/standard/high-risk branch, anchor: `if lane == \"small\" || lane == \"standard\" || lane == \"high-risk\" {`): add one helper that returns the files_changed paths not under `.bee/` and not under `docs/history/` (do NOT reuse worktree/full_suite.rs `is_bookkeeping`, which exempts all docs and .md). When lane is `tiny`, `--inline-reason` is absent, the trace worker is not a registered worker for this cell (reuse `registered_worker_for_cell`), and the helper returns more than one path, refuse with an error that names the counted paths, the bound (tiny = one hand-authored file plus mechanical companions), the reroute (`bee route --set --lane small ...`, then dispatch the next cell with `bee dispatch prepare --claim`) and the escape (`--inline-reason \"<why>\"`, recorded on trace.inline_reason). With `--inline-reason` the cap proceeds and records it exactly as the existing branch does. Route (workflows.rs `validate_route_set_flags`, anchor: `let mut product_files: Option<f64> = None;`): when lane is `tiny` and the product-file count is greater than 1, push an invalid entry naming the one-file tiny bound and `--lane small`. Fix any existing fixture in cells/tests.rs or state_group/tests.rs that this new rule now refuses by giving it a registered worker, one counted file, or lane small, keeping each test's original intent. Tests red-first for every row of the plan's test matrix. skills/bee-swarming/references/swarming-reference.md: where it says tiny may execute inline (rg `may instead execute inline`), add one sentence stating the cap refusal and its escape. Do not run bee dev regen; the leader runs it at wave close.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::cells verbs::state_group",
    "must_haves": {
      "truths": [
        "an inline tiny cap with more than one counted file is refused unless --inline-reason is given",
        "paths under .bee/ and docs/history/ are not counted",
        "a tiny cap by a registered worker is not refused by the new check",
        "route --set with lane tiny and files greater than 1 is refused naming --lane small"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "tiny inline cap bound check and counting helper"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs", "substantive": "tiny route file bound"}
      ],
      "key_links": [
        "the cap check reuses registered_worker_for_cell",
        "swarming-reference.md names the cap refusal and the --inline-reason escape"
      ],
      "prohibitions": [
        "Do not change the small/standard/high-risk refusal or its wording",
        "Do not add a new cap flag",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "tbd-2",
    "feature": "tiny-bound-delegation",
    "title": "Define tiny as one hand-authored file and name the delegation reason",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["d7959afc-58f6-493c-972a-60d75133b63f", "4479e6fd-7614-4d52-b298-86b6ebd584c3", "10bfd461-8a68-4033-b1a2-41aae689f6a2"],
    "files": [
      "skills/bee-planning/SKILL.md",
      "skills/bee-hive/SKILL.md",
      "skills/bee-hive/references/scout-and-ticks.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "skills/bee-hive/references/go-mode.md",
      "skills/bee-swarming/SKILL.md",
      "packages/bee/LEADER.md",
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"
    ],
    "read_first": [
      "docs/history/tiny-bound-delegation/CONTEXT.md",
      "docs/history/tiny-bound-delegation/plan.md",
      "skills/bee-planning/SKILL.md",
      "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"
    ],
    "affects_skills": ["skills/bee-planning/SKILL.md", "skills/bee-hive/SKILL.md", "skills/bee-swarming/SKILL.md"],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md"],
    "regen_obligation_ack": "wave-barrier",
    "action": "Rewrite every statement of the tiny trigger to the D2 bound (4479e6fd) and name the owner's reason for delegating small and up (D1 d7959afc, D3 10bfd461). skills/bee-planning/SKILL.md lane table (anchor: `| \\`tiny\\` | 0–1 flags, ≤2 product files, one direct task |`): tiny = 0–1 flags, one hand-authored file (mechanical companions such as a version bump or regen output do not count), no existing test rewritten, one cheap proof; add one line under the table: size from the whole blast radius (tests to update, docs, manifests, proof runs), never from the core edit; a second hand-authored file of any kind, a doc beside code included, is small; the small row is unchanged. skills/bee-hive/references/scout-and-ticks.md (anchor: `(\\`small\\`: ≤3, \\`tiny\\`: ≤2)` and the demotion bullet `measured touch set is ≤2 product files`): tiny threshold becomes one hand-authored file. routing-and-contracts.md tiny row and Tiny/small fast path, gates-and-delegation.md (rg `1-file tiny fix` and the execution-worker paragraph), go-mode.md line 7, skills/bee-hive/SKILL.md bee-swarming row, skills/bee-swarming/SKILL.md (rg `may run inline`): keep inline tiny legal, and state that from small up execution is dispatched through bee dispatch prepare because each role runs its own configured model; inline outside tiny is refused at cap without --inline-reason, and an inline tiny cap with more than one counted file (paths outside .bee/ and docs/history/) is refused the same way; bee route refuses --lane tiny with --files above 1. packages/bee/LEADER.md rule agents-never-zero-execution-workers (anchor: `From \\`small\\` up, cells run through dispatched workers`): add the tiny bound in one clause and the owner reason in one sentence; keep the rule markers. docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md: add rule R42 stating D1-D5 by CONTEXT D-number citing docs/history/tiny-bound-delegation/CONTEXT.md (never raw decision ids), and amend R13 and R16a where they state the tiny file count; add the feature to the decisions frontmatter list the same way. Write in ASD-STE100 plain English, short sentences. Do not run bee dev regen; the leader runs it at wave close.",
    "verify": "! rg -n '≤2 product files' skills packages/bee/LEADER.md && .bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "the bee-planning lane table defines tiny as one hand-authored file with no rewritten test",
        "no skill or LEADER.md sentence still states the old tiny count of two product files",
        "LEADER.md names the owner reason for dispatching small and up",
        "the doctrine-layer knowledge area carries the tiny bound rule citing the feature CONTEXT"
      ],
      "artifacts": [
        {"path": "skills/bee-planning/SKILL.md", "substantive": "tiny row rewritten"},
        {"path": "packages/bee/LEADER.md", "substantive": "never-zero-execution-workers rule carries the bound and the reason"},
        {"path": "docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md", "substantive": "R42 added"}
      ],
      "key_links": [
        "every tiny trigger statement in skills matches the bee-planning table",
        "the doctrine text names the cap refusal and the route refusal that tbd-1 ships"
      ],
      "prohibitions": [
        "Do not change the small, standard or high-risk triggers",
        "Do not remove inline tiny execution",
        "Do not cite raw decision ids in docs/knowledge"
      ],
      "open_choices": []
    },
    "behavior_change": true
  }
]
```

## Test matrix

| Scenario | Cell | Pass when |
|---|---|---|
| Error: route tiny with files 2 | tbd-1 | refusal names `--lane small` |
| Happy: route tiny with files 1 or 0 | tbd-1 | route recorded |
| Error: inline tiny cap, files a.rs and b.rs, no registered worker | tbd-1 | refusal names both files, the bound, the reroute and `--inline-reason` |
| Happy: same cap with `--inline-reason` | tbd-1 | capped, `trace.inline_reason` set |
| Happy: inline tiny cap of one file plus `.bee/x` and `docs/history/f/y.md` | tbd-1 | capped |
| Happy: tiny cap of two files by a registered worker | tbd-1 | capped |
| Edge: a doc outside docs/history beside code (`docs/knowledge/a.md` + `src/a.rs`) inline | tbd-1 | refused (two counted) |

## Open Questions

(none)

## Out of scope

- The inline-versus-dispatched A/B for real tiny cells.
- The `small` lane size bound and its file cap.
- A merge-time door for the same check.
