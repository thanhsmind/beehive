---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: knowledge-freshness-mechanic

## Summary

Today a knowledge entry can describe code that changed months ago and
nothing says so. With this change every area entry names the files it
depends on and the commit at which it was last confirmed. The bundle check
grades each entry confirmed, suspect, orphan or unverified from git, and
search, show, context and the session preamble print that grade. A worker
who changes a file an entry depends on must re-confirm or edit that entry
before the cell caps. Promote stops generating delivery logs, the
delivery-log tree leaves the bundle, and the knowledge report counts how
often agents actually pull knowledge, per feature. Every host project gets
the binary and the skills through onboarding, then runs one seed command.

Mode: `standard` — 3 risk flags: public-contracts, multi-domain,
covered-contract-change
Why this is the least workflow that protects the work: new frontmatter
fields, a new cap-door check and a changed promote output are public
contracts, but no hard-gate flag applies.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

- D1 (`2b11f025`): pull model; nothing new is pushed into briefs.
- D2 (`93e1001d`): document homes rule; every area entry anchors to code or
  a verify feature.
- D3 (`27ad96a5`): `bee.verified_at` + `bee.verified_commit`; grades
  confirmed / suspect / orphan (plus `unverified` when no commit is
  recorded); search, show, context and preamble print the grade.
- D4 (`e7e80d78`): cap sync door re-confirms anchored concepts; promote
  proposes patterns and invariant bullets only; `work/` leaves the bundle.
- D5 (`28fb074d`): delete before write — move `work/`, author no glossary or
  event catalogue.
- D6 (`e7e133dc`): pull rate per feature in `bee knowledge report`.
- D7 (`6a502ad0`): portable — binary, OKF profile area, bee-capturing skill.
- D8 (`4148025d`): `bee.anchors` explicit path list, seeded once from
  resolving Pointers paths.

## Load-bearing claims

Labels are `read` (file opened at that line) or `ran`; evidence is the exact
line or the command output.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `knowledge check` already resolves path-shaped frontmatter entries against the repo root, so anchor resolution follows an existing pattern | read | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:868 | `if let (Some(root), Some(Value::Array(sources))) = (repo_root, bee.get("sources")) {` |
| 2 | Search rows are built from a `SearchHit` struct that already carries a timestamp, so a grade is one more field | read | packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs:135 | `pub(crate) struct SearchHit {` |
| 3 | The JSON search row is assembled field by field in one place | read | packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs:248 | `row.insert("why".into(), Value::String(hit.why.clone()));` |
| 4 | `show` is served by `run_show` in author.rs | read | packages/bee-rs/crates/bee/src/verbs/knowledge/author.rs:392 | `pub(crate) fn run_show(flags: Flags, json: bool, pre_json: bool, t0: Instant) -> Option<ExitCode> {` |
| 5 | Knowledge verbs route through one single-token match in routing.rs, where a new verb lands | read | packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs:34 | `"check" => run_check(flags, json, pre_json, t0),` |
| 6 | The cap-time sync door is one function taking the touched path set, so an anchored-concept check is a fourth branch | read | packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs:20 | `pub(crate) fn sync_refusal(root: &Path, cell: &Map<String, Value>, touched: &[String]) -> Option<String> {` |
| 7 | The cap calls the door with the MAIN root while its touched set comes from the worktree, so branch (d) needs the worktree root passed in | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:752 | `if let Some(refusal) = sync_refusal(root, &cell_map, &touched_set) {` |
| 8 | Promote writes the delivery draft under `<work_dir>/delivery.md`; removing section (a) removes the record | read | packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs:505 | `format!("{work_dir}/delivery.md")` |
| 9 | The verbs' bundle walk is one function; a `work/` skip lands there for index, check, list, search and context | read | packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs:38 | `pub(crate) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {` |
| 10 | The session preamble has its OWN bundle walk, so the `work/` skip must land there too | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs:219 | `pub(crate) fn list_bundle_markdown(dir: &Path) -> Vec<String> {` |
| 11 | The preamble reads `work/<feature>/work-item.md` directly and announces it, so that reader must stop announcing a moved tree | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs:228 | `let rel = format!("work/{feature}/work-item.md");` |
| 12 | `knowledge context --work` already falls back to docs/history when no work-item concept exists | read | packages/bee-rs/crates/bee/src/verbs/knowledge/anchor.rs:3 | `// whose bee.id matches `work` always wins; otherwise docs/history/<work>/` |
| 13 | Timings rows are written by one function with `{ts, cmd, ms, ok}`; a feature field is added there | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:182 | `pub(crate) fn record_timing(root: &Path, cmd: &str, t0: Instant, ok: bool) {` |
| 14 | The session id is readable from the environment without listing session records | read | packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:432 | `if let Some(v) = env_nonempty("BEE_SESSION_ID") {` |
| 15 | The report verb builds its sections in `build_report`, where pull rate is one more section | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:253 | `pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Value, Vec<String>) {` |
| 16 | The preamble's bundle-level lines (not the per-feature context lines) are rendered by `project_map_lines`, where a count line belongs | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs:279 | `pub(crate) fn project_map_lines(root: &Path, bundle: bool) -> Vec<String> {` |
| 17 | New frontmatter values must satisfy the canonical emitter or `not_canonical` fires | read | packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs:22 | `pub(crate) fn is_plain_safe(value: &str) -> bool {` |
| 18 | bee already shells to git through a shared helper re-exported from the worktree module | read | packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:85 | `pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {` |
| 19 | Served verbs are declared in the generated registry as `{name, invoke, description}` rows | ran | `rg -o '"name": "knowledge.check", "invoke": "bee knowledge check"' packages/bee-rs/crates/bee/src/generated/registry_payload.json` | `"name": "knowledge.check", "invoke": "bee knowledge check"` |
| 20 | The served-vs-declared registry test scans devtools, herding and blind only, never the knowledge namespace, so an undeclared knowledge verb is not caught today | read | packages/bee-rs/crates/bee/tests/registry_dispatch.rs:432 | `for verb in match_arm_verbs(&crate_src("devtools/mod.rs"), "pub fn try_native", 5) {` |
| 21 | okf-profile owns the knowledge verbs' code but no skill, so sync check (a) skips cells that touch only that code | read | docs/knowledge/areas/okf-profile/overview.md:13 | `owns.code: ["packages/bee-rs/crates/bee/src/verbs/knowledge/*"]` / `owns.skills: []` |
| 22 | hook-runtime owns hooks/* code but no skill, so sync check (a) skips kfm-2's preamble edits | read | docs/knowledge/areas/hook-runtime/overview.md:14 | `owns.code: ["packages/bee-rs/crates/bee/src/hooks/*"]` / `owns.skills: []` |
| 23 | The cap sync door's ownership check fires for cells/* code, so a cell touching sync_door.rs must touch a workflow-state skill | read | docs/knowledge/areas/workflow-state/overview.md:15 | `owns.skills: ["skills/bee-planning/*", "skills/bee-swarming/*", "skills/bee-reviewing/*", "skills/bee-capturing/*"]` |
| 24 | The release manifest hashes skill files, so a cell touching skills/ must regen and check it | ran | `rg -c 'skills/bee-swarming/SKILL.md\|skills/bee-capturing/SKILL.md' docs/history/codex-harness-hardening/release-manifest.json` | `12` |
| 25 | Onboarding (which `bee dev regen` runs with `--apply`) installs the built binary into .bee/bin, so a cell that needs a verb from an earlier cell rebuilds and regens | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:638 | `"copy_binary" => {` |
| 26 | Every knowledge verb resolves its root to the MAIN store root even inside a granted worktree, so bundle reads and writes need an explicit worktree-aware docs root | read | packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs:42 | `let root = match resolve_store_root(&cwd) {` |
| 27 | The close door already resolves "the docs tree of the checkout this command runs in" for a feature; the knowledge verbs reuse that resolver | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:835 | `pub(crate) fn docs_root_for_feature(root: &Path, feature: &str) -> PathBuf {` |
| 28 | One area concept's `sources` names a path under docs/knowledge/work/, which becomes a dangling_source that blocks close after the move unless repointed | read | docs/knowledge/areas/workflow-state/close-usage-record.md:12 | `sources: ["docs/knowledge/work/usage-in-bee-store/delivery.md"` |
| 29 | Three area concepts are rule homes (`bee.applied_at`), so seeding their frontmatter trips sync check (b) at kfm-6's cap unless acked | read | docs/knowledge/areas/workflow-state/capture-queue-and-the-blocker-threshold.md:14 | `applied_at: [skills/bee-capturing/SKILL.md, packages/bee-rs/crates/bee/src/verbs/capture.rs]` |
| 30 | Area concepts carry no verified field today; only authored `timestamp` | ran | `rg --no-filename -o '^  ?(timestamp\|updated\|verified_at\|last_verified\|checked_against\|sources\|lifecycle):' docs/knowledge/areas \| sort \| uniq -c` | `143   lifecycle:` / `141   sources:` |

## Discovery

Direct reads of the knowledge verbs, the cap door and the preamble, then a
three-seat hat wave (facts-gaps, alternatives, user-impact; digests under
`docs/history/knowledge-freshness-mechanic/reports/`). The wave's blockers
and the answers folded in here: the cap door grades against the main root
while the worktree holds the cell's commits (claims 7, 26, 27 — every
bundle read and write now goes through a worktree-aware docs root); the
preamble has a second bundle walk and a direct work-item reader (claims 10,
11 — both in kfm-4); a rewritten sha is present but not an ancestor
(ancestry test, not `cat-file`); kfm-6 needs a rebuilt binary (claim 25);
the registry test never scans knowledge verbs (claim 20 — kfm-1 extends
it); a moved `sources` path blocks close (claim 28 — kfm-4 repoints it);
seeding every concept to `unverified` would make the door refuse
everything (seed now stamps each concept's own last commit, and
`unverified` passes at cap); patterns stay out of grading (map
"Not yet specified": patterns keep a date). Ownership: okf-profile and
hook-runtime own code but no skill (claims 21, 22), so only kfm-3 (cells/*)
must touch a skill (claim 23). The deferred question "move or exclude" is
answered: D5 names the move, and the walk skip is the portable part a host
needs regardless. Measured before planning: 95 of 139 area concepts had an
anchored Rust file change after the concept's last touch; `knowledge
search` ran 89 times in 35,478 CLI invocations.

## Approach

Recommended path: one new module `verbs/knowledge/freshness.rs` owns the
grade (D3, D8), a worktree-aware `bundle_root` resolver, and the two new
verbs `knowledge seed-anchors` and `knowledge confirm` (single-token names;
D8's "anchors seed" wording is the one named deviation, taken because
routing.rs dispatches on one token). Check, search, show, context and the
preamble call its one `grade_bundle` pass (kfm-1, kfm-2). The cap door
gains branch (d) over the same pass, against the worktree's bundle (kfm-3,
D4). Promote loses its delivery section, both walks skip a top-level
`work/`, the preamble stops announcing a work-item anchor, and the tree
moves to `docs/history/knowledge-work/` with every in-bundle path string
repointed (kfm-4, D4, D5). Timings rows gain the env-bound feature and the
report counts pulls against all calls (kfm-5, D6). The OKF profile area and
bee-capturing document the homes rule and the re-confirm step, and the seed
verb runs once over bee's areas (kfm-6, D2, D7).

Grade rule (the one semantic every cell must keep):

- `orphan` when any anchor path is missing on disk under the bundle root's
  checkout.
- else `unverified` when the concept has no anchors, or `verified_commit`
  is absent, or `git merge-base --is-ancestor <verified_commit> HEAD`
  fails (unknown sha, or a sha rewritten by squash or rebase), or git is
  unavailable — `reason` names which.
- else `suspect` when any anchor was changed by a commit in
  `verified_commit..HEAD` that did NOT also touch the concept file.
- else `confirmed`.

A commit that touches both the anchor and the concept counts as a
confirmation: a worker edits code and concept in one cell commit without
grading itself suspect. The whole bundle is graded in one pass: one
`git log --format=%H --name-only --first-parent -m <oldest verified_commit>..HEAD`
over the checkout plus one `merge-base --is-ancestor` per distinct
`verified_commit`; never one git call per anchor. Paths: anchors are
repo-relative; a concept's repo path is `docs/knowledge/<rel>`; the
touched set at cap is repo-relative already. Bundle root: every verb and
the door read and write the docs tree of the checkout the command runs in,
through the same resolver the close door uses (claim 27); from main that is
main, from a granted worktree it is the worktree.

Seeding stamps a baseline: `seed-anchors` writes `bee.anchors` from the
body's backticked repo paths that exist (skipping `.bee/`, `docs/` and any
`/generated/` segment), and sets `verified_commit` to the concept file's
own last commit (`git log -1 --format=%H -- <concept>`) and `verified_at`
to that commit's date, so the first grade is honest (the 95-of-139 signal
appears as `suspect`, not as a wall of `unverified`).

Cap door (d) refuses on `suspect` and `orphan` only; `unverified` passes
(a host that never seeded, or a concept outside git, is never blocked).
The refusal names every concept, the grade, and two remedies: for suspect,
`bee knowledge confirm --id <id>` (or `--path <file>` for every concept
anchoring that file) then commit the concept with the cell, adding it to
the cell's files and reservation; for orphan, fix `bee.anchors`; and
`--sync-ack` as the recorded escape.

Rejected: grading from the Pointers prose (the regex parse that rotted);
deriving `verified_commit` from the concept's last touch at every grade
(a cosmetic re-emit would silently confirm; D3 names the stamped field —
the seed uses the last touch once, as a baseline); a review calendar;
keeping `work/` and excluding it by index only (D5 names the move; the
walk skip is still shipped because a host may hold a work/ tree); a
lane-window join for pull rate (lane records carry no end time, and
parallel features overlap); pushing the manifest into briefs (D1).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Grade semantics for squash/rebase, merge commits, same-commit edits | MEDIUM — fact: `git log --format=%H <vc>..HEAD -- sync_door.rs` then `git show --name-only --format= <sha>` listed the touching commit and its five files in this repo (hat facts-gaps, ran) | kfm-1 | tests on a fixture repo with real commits: four grades, same-commit confirmed, non-ancestor sha unverified, merge commit handled |
| Bundle root inside a worktree | MEDIUM — fact: frame.rs:42 resolves to the main store root; close.rs:835 already picks the worktree docs tree | kfm-1, kfm-3 | test: confirm run from a granted worktree writes the worktree's concept; (d) grades the worktree bundle |
| New fields break canonical emit | LOW | kfm-1 | seeded fixture reports zero not_canonical rows |
| Cap door (d) refuses legitimate caps | MEDIUM — fact: handlers_close.rs:734-752 builds touched_set from the worktree HEAD numstat and passes the main root to the door | kfm-3 | tests: suspect refusal, orphan refusal, unverified pass, same-commit pass, confirm-then-pass, --sync-ack escape, no-bundle pass |
| Moving 453 files breaks context, index, check, preamble or close | MEDIUM — fact: anchor.rs falls back to docs/history; close.usage-record.md:12 names a work/ path in sources | kfm-4 | `bee knowledge check --json` has zero dangling_source rows; `index --check` green; `context --work windows-ci-1107` answers; preamble and brief tests green |
| Timings join attributes pulls to the wrong feature | LOW | kfm-5 | tests: env-bound session writes feature, no env id writes null |
| kfm-6 runs a verb the installed binary lacks | LOW — fact: onboard apply copies the built binary (claim 25) | kfm-6 | `.bee/bin/bee --help --names` lists seed-anchors before the seed runs |
| Skill and manifest drift | LOW | kfm-3, kfm-6 | `bee dev release-manifest --check` green |

Waves: kfm-1, kfm-4 and kfm-5 are the roots and share
`knowledge/tests.rs`, so the overlap rule serializes them (kfm-1 first, then
kfm-4, then kfm-5); kfm-2 and kfm-3 wait for kfm-1 and are disjoint from
each other but kfm-2 shares `knowledge/tests.rs` with the roots. kfm-6 is
last: it depends on every other cell, because it documents the door and the
report, seeds anchors that the door would otherwise refuse the other cells
on, and shares `docs/knowledge/index.md` with kfm-4.

Knowledge sync for this feature's own concepts, the verify-app feature
index and the handbook is the leader's capture step at scribe time.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"Rust cells kfm-1..kfm-5 write code and tests."},
    {"stage":"test-and-live-proof","classification":"conditional","role":"test","condition":"a cell's worker reports a red it cannot clear in two attempts","reason":"Tests ride each cell; the test role repairs a stuck red."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"kfm-6 writes the OKF profile concept and the bee-capturing step and runs the seed verb."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"Leader gathers at scribe time run on --kind gather."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane with more than five product files: the plan-step hat wave ran."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat; ran."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat; ran."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow extraction needed."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One clear design."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One clear design."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One clear design."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Phase plan (one slice):

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | anchors + grades + verbs, grade printing, cap door (d), promote and work/ move, pull rate, docs + seed | the mechanic is one contract; shipping half of it leaves hosts with fields nobody grades | `bee knowledge check` lists suspect concepts; search prints a grade; a cap that touches a suspect concept's anchor refuses until confirm; report prints pulls per feature | the bee-only content cut (next feature) |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| kfm-1 | Grade concept freshness from anchors and git, with seed and confirm verbs | knowledge/freshness.rs (new), knowledge/check.rs, knowledge/mod.rs, knowledge/routing.rs, knowledge/tests.rs, generated/registry_payload.json, tests/registry_dispatch.rs | — | `bee knowledge check` prints a freshness count line and one WARN row per suspect or orphan concept, or a seed hint when no concept carries anchors; `bee knowledge seed-anchors` fills `bee.anchors` and stamps each concept's own last commit; `bee knowledge confirm --id|--path` stamps verified_at and verified_commit; all three work on the worktree's bundle when run there | knowledge and registry tests green |
| kfm-2 | Print the grade wherever a concept is read | knowledge/search.rs, knowledge/author.rs, knowledge/context.rs, knowledge/tests.rs, session_preamble/render.rs, session_preamble/knowledge.rs, session_preamble/tests.rs | kfm-1 | search, show and context rows carry grade, verified_at and verified_commit in text and JSON; the preamble's project map prints `knowledge: N confirmed, M suspect, K orphan, U unverified` or a skipped note past 500 ms | knowledge and session_preamble tests green |
| kfm-3 | Refuse a cap that leaves a suspect or orphan concept unconfirmed | cells/sync_door.rs, cells/handlers_close.rs, cells/tests.rs, skills/bee-swarming/SKILL.md, docs/history/codex-harness-hardening/release-manifest.json | kfm-1 | a cap whose touched files anchor a suspect or orphan concept names the concept, its grade and the remedy; unverified passes; `--sync-ack` still escapes; the worker contract names the re-confirm step and the reservation | cells sync_door tests and `bee dev release-manifest --check` green |
| kfm-4 | Retire delivery records and move work/ out of the bundle | knowledge/promote.rs, knowledge/walk.rs, knowledge/tests.rs, session_preamble/knowledge.rs, session_preamble/render.rs, session_preamble/tests.rs, drivers/tests.rs, skills/bee-hive/references/scout-and-ticks.md, release-manifest.json, docs/knowledge/work/* (deleted), docs/history/knowledge-work/* (new), docs/knowledge/areas/* and patterns/* (path strings only), docs/knowledge/index.md, docs/knowledge/*/index.md | kfm-1 | promote proposes patterns and area bullets only; `bee knowledge check --json` has zero dangling_source rows and `index --check` is green; the preamble no longer announces a work-item anchor; `context --work <closed feature>` still answers from docs/history | knowledge, session_preamble and drivers tests, check, index --check and manifest check green |
| kfm-5 | Count knowledge pulls per feature | verbs/mod.rs, knowledge/report.rs, knowledge/tests.rs | kfm-4 | every timings row carries the env-bound feature or null; `bee knowledge report` prints a pull-rate table per feature: search, context, show, total pulls, all calls, rate | knowledge tests green |
| kfm-6 | Write the homes rule and re-confirm step, and seed bee's own anchors | docs/knowledge/areas/okf-profile/freshness-and-document-homes.md (new), okf-profile/index.md, okf-profile/overview.md, okf-profile/context-and-promote.md, okf-profile/concept-model-and-authoring.md, skills/bee-capturing/SKILL.md, docs/knowledge/areas/*, docs/knowledge/index.md, release-manifest.json | kfm-1, kfm-2, kfm-3, kfm-4, kfm-5 | a new OKF concept states the four homes, the grade rule by pointer, the cap step, the seed step for a host and the pull-rate measure; bee-capturing names re-confirm; every area concept with a resolving backticked repo path carries `bee.anchors` and a baseline commit, and `knowledge check` grades it | `bee knowledge check` 0 errors, `index --check` and `bee dev release-manifest --check` green |
| kfm-7 | Derive the preamble pathological-budget fixture bound from the render (fix-first, plan rev 1) | session_preamble/tests.rs | kfm-1 | the tighter-budget preamble test is green again with a bound computed from the digest section length, not a literal; drop order and PREAMBLE_BUDGET_BYTES unchanged | session_preamble tests green |

```json
[
  {
    "id": "kfm-1",
    "feature": "knowledge-freshness-mechanic",
    "title": "Grade concept freshness from anchors and git, with seed and confirm verbs",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["27ad96a5-ac3a-46bf-b80a-cb18978ccb5d", "4148025d-e997-4642-bab4-da00104c5795"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/tests/registry_dispatch.rs"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frame.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs",
      "packages/bee-rs/crates/bee/tests/registry_dispatch.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add the freshness grade to the knowledge bundle (D3 27ad96a5, D8 4148025d) exactly by the Grade rule and the Seeding paragraph in plan.md Approach. New module verbs/knowledge/freshness.rs exports: (1) `bundle_root(root, feature: Option<&str>) -> PathBuf` — the docs/knowledge directory of the checkout the command runs in, built the way drivers/close.rs `docs_root_for_feature` picks a granted worktree's tree (anchor: `pub(crate) fn docs_root_for_feature(root: &Path, feature: &str) -> PathBuf {`); every knowledge verb that reads or writes concepts (check, list, search, show, context, new, seed-anchors, confirm) resolves its bundle through it, so a command run inside a granted worktree reads and writes that worktree's docs tree; (2) `grade_bundle(checkout_root, concepts) -> BTreeMap<rel_path, Grade>` and `grade_concept(checkout_root, rel_path, bee: &Map<String, Value>) -> Grade` where `Grade { grade: confirmed|suspect|orphan|unverified, verified_at: Option<String>, verified_commit: Option<String>, anchors: Vec<String>, changed: Vec<String>, reason: Option<String> }` — the signature takes the bundle-relative path and the parsed `bee` map, not walk::Concept, so the session preamble's own Concept type can call it; implement the one-pass rule from plan.md: one `git log --format=%H --name-only --first-parent -m <oldest verified_commit>..HEAD` through verbs/worktree/git.rs `run_git` (anchor: `pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {`) plus one `git merge-base --is-ancestor <vc> HEAD` per distinct verified_commit, never a git call per anchor; a concept's repo path for the same-commit exemption is `docs/knowledge/<rel>`. Three frontmatter fields under `bee`: `anchors` (flat list), `verified_at` (ISO date), `verified_commit` (full sha); every write goes through the canonical emitter (frontmatter.rs `emit_scalar` / `is_plain_safe`) and a written file must report zero not_canonical. In check.rs, after the profile pass, grade every concept under areas/ only (never patterns/, never work/, never index.md) and emit: text summary line `freshness: <n> confirmed, <n> suspect, <n> orphan, <n> unverified`; `--json` object `freshness: {confirmed, suspect, orphan, unverified, rows: [{path, grade, verified_at, verified_commit, changed, reason}]}`; one profile WARNING per suspect concept (code `stale_anchor`, message naming the changed anchors and the remedy `knowledge confirm --id <bee.id>`) and per orphan concept (code `orphan_anchor`, naming the missing path and the remedy: fix bee.anchors); unverified is counted, never warned; when areas/ has concepts and none carries bee.anchors, print one NOTE `no concept carries bee.anchors — run bee knowledge seed-anchors once`. Two new single-token verbs routed in routing.rs (anchor: `\"check\" => run_check(flags, json, pre_json, t0),`) and declared in generated/registry_payload.json as `{name, invoke, description}` rows beside `knowledge.check` (round-trip the JSON keeping its byte format): `bee knowledge seed-anchors [--dry-run]` walks areas/ only and, for every concept with NO `bee.anchors` key, collects the backticked repo-relative paths in its body that exist under the checkout (same backtick-path shape `dangling_source_candidate` accepts), skipping any path under `.bee/`, `docs/`, or containing a `/generated/` segment; writes them as `bee.anchors`; sets `bee.verified_commit` to `git log -1 --format=%H -- docs/knowledge/<rel>` and `bee.verified_at` to that commit's ISO date (a concept with no commit gets anchors only); prints per-file counts and how many concepts got zero anchors. `knowledge confirm (--id <bee.id>)+ | --path <repo path>` stamps `verified_at` (today, UTC) and `verified_commit` (current HEAD of the checkout) on each named concept, or on every areas/ concept whose anchors contain --path; refusals, each naming its remedy: unknown id → `no concept with bee.id <id> — bee knowledge list`; concept with no anchors → `<path> has no bee.anchors — run bee knowledge seed-anchors or add anchors`; --path matching no concept → `no concept anchors <path>`; confirm never clears orphan and says so when the grade stays orphan. Extend tests/registry_dispatch.rs `every_namespace_the_dispatcher_serves_is_declared_in_the_registry` (anchor: `for verb in match_arm_verbs(&crate_src(\"devtools/mod.rs\"), \"pub fn try_native\", 5) {`) to also scan verbs/knowledge/routing.rs's match arms as `bee knowledge <verb>`, so an undeclared knowledge verb fails the test. Tests red-first in knowledge/tests.rs on a fixture repo with real git commits: each of the four grades; anchor changed in a commit that also touched the concept → confirmed; a present sha that is not an ancestor of HEAD → unverified with a reason; a merge commit touching an anchor is seen; seed writes anchors and the baseline commit only where anchors are absent and the file stays canonical; confirm stamps both fields, handles --path, and each refusal names its remedy; a confirm run from a granted worktree writes the worktree's concept and leaves main untouched; check's --json carries the freshness object, the stale_anchor / orphan_anchor rows and the seed NOTE.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge registry",
    "must_haves": {
      "truths": [
        "grade_concept returns confirmed, suspect, orphan or unverified by the rule in plan.md Approach, with unverified for a present-but-non-ancestor sha and for an unavailable git",
        "an anchor changed in a commit that also touched the concept file does not make the concept suspect",
        "grading the whole bundle makes at most one git log call plus one merge-base call per distinct verified_commit",
        "every knowledge verb run inside a granted worktree reads and writes that worktree's docs/knowledge",
        "bee knowledge check prints the freshness count line, carries the freshness object in --json, emits stale_anchor and orphan_anchor warnings and the seed NOTE, and never grades patterns/",
        "bee knowledge seed-anchors writes bee.anchors plus a baseline verified_commit and verified_at only to areas/ concepts without anchors, skips .bee/, docs/ and generated paths, and keeps every written file canonical",
        "knowledge confirm (by id, or by anchored path) stamps verified_at and verified_commit and every refusal names its remedy",
        "the registry served-vs-declared test scans knowledge verbs"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs", "substantive": "bundle_root, grade_bundle, grade_concept, seed-anchors and confirm"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "knowledge.seed-anchors and knowledge.confirm declared"}
      ],
      "key_links": [
        "check.rs calls grade_bundle for every areas/ concept",
        "routing.rs serves seed-anchors and confirm",
        "tests/registry_dispatch.rs scans verbs/knowledge/routing.rs"
      ],
      "prohibitions": [
        "Do not grade or seed anything under docs/knowledge/patterns/ or docs/knowledge/work/",
        "Do not change any existing finding code or its wording",
        "Do not run one git invocation per anchor",
        "No code comments"
      ],
      "open_choices": ["Keep the seed and confirm verbs in freshness.rs instead of author.rs"]
    },
    "behavior_change": true
  },
  {
    "id": "kfm-2",
    "feature": "knowledge-freshness-mechanic",
    "title": "Print the grade wherever a concept is read",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-1"],
    "decisions": ["2b11f025-942e-4a18-86b1-fdb28a89f254", "27ad96a5-ac3a-46bf-b80a-cb18978ccb5d"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/author.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/context.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Make the freshness grade visible at every pull point (D1 2b11f025, D3 27ad96a5), using only freshness::grade_bundle / grade_concept from kfm-1 — never a second grading rule. search.rs: `SearchHit` (anchor: `pub(crate) struct SearchHit {`) gains grade, verified_at and verified_commit; the text line becomes `<path> — <title> — [<grade> @ <verified_at or never>] — <why>` and the JSON row (anchor: `row.insert(\"why\".into(), Value::String(hit.why.clone()));`) gains `grade`, `verified_at`, `verified_commit`; a patterns/ hit prints `[pattern]` in place of a grade. author.rs `run_show` prints a `freshness:` line after the metadata and adds the same three keys to its JSON. context.rs: every manifest entry carries `grade` and `verified_at`; a suspect or orphan entry's inclusion reason ends with ` (grade: <grade> — <first changed or missing anchor>)`; ranking is unchanged. session_preamble: in render.rs `project_map_lines` (anchor: `pub(crate) fn project_map_lines(root: &Path, bundle: bool) -> Vec<String> {`), NOT in the per-feature knowledge_context_lines, add ONE line `- knowledge: <n> confirmed, <m> suspect, <k> orphan, <u> unverified — suspect first: bee knowledge check` computed over areas/ with grade_bundle; omit the line when the bundle is absent or areas/ holds zero concepts; wrap the computation in a 500 ms budget and past it print `- knowledge: freshness skipped (slow) — bee knowledge check` instead; bridge the preamble's own Concept (path + parsed data) to grade_concept's (rel_path, bee map) signature. The preamble byte budget tests stay green. Tests red-first: a search row and a show output carry the grade of a fixture concept in two grades; a patterns hit prints [pattern]; a context manifest entry carries grade and the suspect suffix; the preamble line counts a fixture bundle, is absent without a bundle and with zero area concepts, and the skipped form renders when the budget is exceeded (inject the budget).",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge session_preamble",
    "must_haves": {
      "truths": [
        "knowledge search text and JSON rows carry grade, verified_at and verified_commit",
        "knowledge show prints the grade and carries it in JSON",
        "knowledge context entries carry grade and a suspect or orphan entry names its first changed or missing anchor in the reason",
        "the session preamble's project map prints one freshness count line, omitted for no bundle or zero area concepts, and a skipped note past 500 ms"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs", "substantive": "SearchHit carries the grade fields and both outputs print them"}
      ],
      "key_links": [
        "search.rs, author.rs, context.rs and render.rs all call freshness::grade_bundle or grade_concept"
      ],
      "prohibitions": [
        "Do not change search ranking or context ranking",
        "Do not add a second grading rule anywhere",
        "Do not put the count line in knowledge_context_lines",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "kfm-3",
    "feature": "knowledge-freshness-mechanic",
    "title": "Refuse a cap that leaves a suspect or orphan concept unconfirmed",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-1"],
    "decisions": ["e7e80d78-f172-410d-b142-7b2bd464105e"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
      "skills/bee-swarming/SKILL.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs"
    ],
    "affects_skills": ["skills/bee-swarming/SKILL.md"],
    "affects_specs": [],
    "action": "Add check (d) to the cap-time sync door (D4 e7e80d78) by the Cap door paragraph in plan.md Approach. sync_door.rs `sync_refusal` (anchor: `pub(crate) fn sync_refusal(root: &Path, cell: &Map<String, Value>, touched: &[String]) -> Option<String> {`) gains a `history_root: &Path` parameter (the checkout whose HEAD the cell committed into), and handlers_close.rs passes the `history_root` it already builds touched_set from (anchor: `if let Some(refusal) = sync_refusal(root, &cell_map, &touched_set) {`). Check (d), after (c): grade the areas/ concepts of `history_root`'s docs/knowledge with freshness::grade_bundle; for every concept whose `bee.anchors` (normalized with normalize_cell_path) intersects `touched`, the cap passes when the concept's own repo path is in `touched`, or its grade is confirmed or unverified; it refuses on suspect or orphan with `capCell: SYNC_DOOR — cell \"<id>\" touches <paths> anchored by knowledge concept(s): <concept path> (<grade>)…. FIX: for suspect, knowledge confirm --id <bee.id> (or --path <file>), then add the concept file to the cell's files and reservation and commit it with the cell; for orphan, fix the concept's bee.anchors; or pass --sync-ack \"<reason>\" to cap anyway (the reason is recorded on trace.sync_ack and trace.deviations).` A missing docs/knowledge/ or an empty areas/ makes (d) a no-op. The existing --sync-ack escape covers (d) with no new flag. In skills/bee-swarming/SKILL.md, under the worker contract (\"Execute\"), add one step before the commit: run `bee knowledge check --json`; for every suspect concept anchored by a file you changed, fix the concept or run `knowledge confirm --id <id>`, reserve and commit the concept file with the cell — cite this feature's CONTEXT.md D4 by path, no second copy of the grade rule. Because skills/ is hashed by the release manifest, run `cargo build --release --manifest-path packages/bee-rs/Cargo.toml` then `bee dev regen` and include the regenerated manifest. Tests red-first in cells/tests.rs, every test name starting with `sync_door_`, on a fixture repo with a bundle and real commits: a cap touching an anchored file with the concept suspect refuses and names the concept, the grade and the confirm remedy; the same for orphan with the anchors remedy; an unverified concept passes; passes when the concept file is in the touched set; passes after `knowledge confirm` with the concept committed; passes with --sync-ack and records the reason; a touched file anchored by no concept passes; a repo without a bundle caps as before; a cell committed in a granted worktree is graded against the worktree's bundle, not main's.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells::tests::sync_door && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "a cap whose touched files anchor a suspect or orphan concept refuses, naming the concept, its grade and the remedy for that grade",
        "a cap passes when the concept is unverified, in the touched set, confirmed at that commit, or escaped with --sync-ack",
        "the door grades the worktree's bundle when the cell committed in a granted worktree",
        "a repo without a bundle caps exactly as before",
        "the bee-swarming worker contract names the re-confirm step and the reservation"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs", "substantive": "check (d) over bee.anchors using freshness::grade_bundle against history_root"},
        {"path": "skills/bee-swarming/SKILL.md", "substantive": "one re-confirm step in the worker contract"}
      ],
      "key_links": [
        "handlers_close.rs passes history_root into sync_refusal",
        "sync_door.rs calls knowledge::freshness::grade_bundle",
        "the release manifest hash for skills/bee-swarming/SKILL.md matches the edited file"
      ],
      "prohibitions": [
        "Do not change checks (a), (b) or (c) or their wording",
        "Do not add a new cap flag",
        "Do not refuse on unverified",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "kfm-4",
    "feature": "knowledge-freshness-mechanic",
    "title": "Retire delivery records and move work/ out of the bundle",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-1"],
    "decisions": ["e7e80d78-f172-410d-b142-7b2bd464105e", "28fb074d-82c5-4962-85ba-a430b13b9292"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs",
      "skills/bee-hive/references/scout-and-ticks.md",
      "docs/history/codex-harness-hardening/release-manifest.json",
      "docs/knowledge/work/*",
      "docs/history/knowledge-work/*",
      "docs/knowledge/areas/*",
      "docs/knowledge/patterns/*",
      "docs/knowledge/index.md",
      "docs/knowledge/areas/index.md",
      "docs/knowledge/patterns/index.md"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/anchor.rs"
    ],
    "affects_skills": ["skills/bee-hive/references/scout-and-ticks.md"],
    "affects_specs": [],
    "action": "Stop generating delivery records and take the delivery-log tree out of the bundle (D4 e7e80d78, D5 28fb074d). promote.rs: remove section (a), the delivery draft (anchor: `format!(\"{work_dir}/delivery.md\")`), from the text proposal and the --json output; the proposal keeps (b) area update bullets and (c) pattern candidates and says in its header that it proposes those two only; update the promote tests in knowledge/tests.rs that assert `delivery` (rg anchors: `assert_eq!(p[\"delivery\"][\"path\"], \"work/delivery.md\");` and `assert_eq!(p[\"delivery\"][\"path\"], \"work/hist-1/delivery.md\");`). Both bundle walks skip a top-level `work/` directory: verbs walk.rs `list_bundle_markdown` (anchor: `pub(crate) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {`) and the preamble's own session_preamble/knowledge.rs `list_bundle_markdown` (anchor: `pub(crate) fn list_bundle_markdown(dir: &Path) -> Vec<String> {`); add a test for each that a bundle with work/ lists nothing under it. render.rs `work_item_candidate` (anchor: `let rel = format!(\"work/{feature}/work-item.md\");`) is removed with its call so the preamble never announces a work-item anchor; resolve_anchor then takes its docs/history arm (anchor.rs: `// whose bee.id matches `work` always wins; otherwise docs/history/<work>/`); update the preamble and brief tests that build work/ fixtures (session_preamble/tests.rs and drivers/tests.rs — rg `docs/knowledge/work` in both) to the history anchor. In skills/bee-hive/references/scout-and-ticks.md, the line that offers to author a work item under docs/knowledge/ (rg `work item` near line 251) now points at docs/history/<feature>/ instead; skills/ is hashed, so run `bee dev regen` and include the manifest. Move the tree with `git mv docs/knowledge/work docs/history/knowledge-work` (one commit with the code), then repoint every literal `docs/knowledge/work/` string inside docs/knowledge/areas/ and docs/knowledge/patterns/ — frontmatter sources and body alike, path strings only, no sentence rewrites — to `docs/history/knowledge-work/` (rg anchor: `sources: [\"docs/knowledge/work/usage-in-bee-store/delivery.md\"` in workflow-state/close-usage-record.md), run `bee knowledge index`, and confirm `bee knowledge check --json` reports 0 OKF errors and ZERO dangling_source rows, `bee knowledge index --check` 0 stale, and `bee knowledge context --work windows-ci-1107 --budget 4000` still returns a manifest from the docs/history anchor. Do not change any other concept content.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge session_preamble drivers::tests::prepare && .bee/bin/bee knowledge check --json && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "bee knowledge promote emits no delivery section in text or JSON",
        "docs/knowledge/work/ no longer exists and docs/history/knowledge-work/ holds the moved files",
        "both bundle walks ignore a top-level work/ directory and the preamble announces no work-item anchor",
        "bee knowledge check --json reports zero dangling_source rows after the move",
        "knowledge context for a closed feature still returns a manifest from the docs/history anchor"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs", "substantive": "top-level work/ skipped in the verbs' bundle walk"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_preamble/knowledge.rs", "substantive": "top-level work/ skipped in the preamble's bundle walk"},
        {"path": "docs/knowledge/index.md", "substantive": "regenerated without a work section"}
      ],
      "key_links": [
        "every bundle reader, verbs and preamble, skips work/",
        "no sources entry in docs/knowledge names docs/knowledge/work/"
      ],
      "prohibitions": [
        "Do not rewrite any sentence in areas/ or patterns/; repoint path strings only",
        "Do not delete the moved files; they move to docs/history/knowledge-work/",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "kfm-5",
    "feature": "knowledge-freshness-mechanic",
    "title": "Count knowledge pulls per feature",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-4"],
    "decisions": ["e7e133dc-dd86-42c5-8089-712ab6555f4b"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/store.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Measure pull rate (D6 e7e133dc). verbs/mod.rs `record_timing` (anchor: `pub(crate) fn record_timing(root: &Path, cmd: &str, t0: Instant, ok: bool) {`) adds a `feature` field to every timings row: when the environment carries a session id (store.rs anchor: `if let Some(v) = env_nonempty(\"BEE_SESSION_ID\") {`) read that session's bound lane feature through state_group/store.rs `session_binding(Some(id), root)`; with no env id, or any read failure, write null — never the session-listing fallback, never a refusal, never a slowdown; the existing `{ts, cmd, ms, ok}` keys and the stderr line are unchanged. report.rs `build_report` (anchor: `pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Value, Vec<String>) {`) adds a `pull_rate` section read from .bee/logs/timings.jsonl: per feature (null grouped as `(unbound)`) the counts of `knowledge search`, `knowledge context`, `knowledge show`, their sum `pulls`, the count of all rows `calls`, and `rate` = pulls / calls rounded to three decimals, plus a total row; text output prints a small table under a `Pull rate` heading; malformed rows are skipped and counted in `malformed_count`; a missing log prints an empty table. Tests red-first: a row written with BEE_SESSION_ID bound to a lane carries that feature; without the env id the row carries null; the report groups a fixture log by feature, computes rate, and skips a malformed line.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge timings",
    "must_haves": {
      "truths": [
        "every new timings row carries feature (string or null) beside the unchanged ts, cmd, ms, ok keys, read from the env session id only",
        "bee knowledge report prints a pull-rate table per feature with search, context, show, pulls, calls and rate"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs", "substantive": "pull_rate section built from timings.jsonl"}
      ],
      "key_links": [
        "record_timing reads the bound lane through session_binding with the env session id"
      ],
      "prohibitions": [
        "Do not make record_timing list session records, refuse, or slow the command on any failure",
        "Do not change existing report sections",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "kfm-6",
    "feature": "knowledge-freshness-mechanic",
    "title": "Write the homes rule and re-confirm step, and seed bee's own anchors",
    "lane": "standard",
    "role": "docs",
    "deps": ["kfm-1", "kfm-2", "kfm-3", "kfm-4", "kfm-5"],
    "decisions": ["93e1001d-bdf6-42fb-9297-9f4d8589a5ef", "6a502ad0-7273-4c91-9266-065e56eec25c", "4148025d-e997-4642-bab4-da00104c5795"],
    "files": [
      "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md",
      "docs/knowledge/areas/okf-profile/index.md",
      "docs/knowledge/areas/okf-profile/overview.md",
      "docs/knowledge/areas/okf-profile/context-and-promote.md",
      "docs/knowledge/areas/okf-profile/concept-model-and-authoring.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/areas/*",
      "docs/knowledge/index.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "docs/knowledge/areas/okf-profile/overview.md",
      "docs/knowledge/areas/okf-profile/concept-model-and-authoring.md",
      "skills/bee-capturing/SKILL.md"
    ],
    "affects_skills": ["skills/bee-capturing/SKILL.md"],
    "affects_specs": [],
    "action": "Document the mechanic and apply it to bee's bundle (D2 93e1001d, D7 6a502ad0, D8 4148025d). First make the installed binary carry kfm-1..kfm-5: run `cargo build --release --manifest-path packages/bee-rs/Cargo.toml` then `bee dev regen` (its onboard --apply step copies the built binary into .bee/bin) and confirm `.bee/bin/bee --help --names` lists `knowledge seed-anchors` before going on. Write one new concept docs/knowledge/areas/okf-profile/freshness-and-document-homes.md through `knowledge new --type <the concept type the sibling okf-profile files use> --area okf-profile` and then edit its body: (1) the document homes rule — the four kinds that live in docs/knowledge (vision, glossary, area map with boundaries and invariants, decisions, patterns) and the three that live in code (verify features and tests, generated contracts, runnable runbooks), with the sentence that every area entry anchors to at least one code file or verify feature; (2) the three fields `bee.anchors`, `bee.verified_at`, `bee.verified_commit` and the grade rule, cited from plan.md Approach by pointer, not copied, plus the one-line fact that patterns carry a date and no grade; (3) the cap re-confirm step, that unverified passes, and the `--sync-ack` escape; (4) the host steps: the binary and skills arrive through onboarding, then the owner runs `bee knowledge seed-anchors` once, and `knowledge check` prints the NOTE until that happens; (5) the pull-rate measure and what a flat rate means (the lean layer is deleted next); (6) a Pointers section naming freshness.rs, sync_door.rs, report.rs and this feature's CONTEXT.md, and its own `bee.anchors` set to those three Rust files. Link it from okf-profile/overview.md (one bullet). In okf-profile/context-and-promote.md and concept-model-and-authoring.md, change only the sentences that still describe the delivery draft and the docs/knowledge/work/ work-item location (rg `delivery` and `work/` in both) to say promote proposes area bullets and pattern candidates only and that work-item records live under docs/history/. In skills/bee-capturing/SKILL.md add, in the capture discipline, one short paragraph: when a captured behaviour change touches a file an area concept anchors, re-confirm or edit that concept (`knowledge confirm --id`), new area entries carry `bee.anchors`, and a bundle with no anchors is seeded once with `bee knowledge seed-anchors` — cite the new concept by path, no second copy of the rule; skills/ is hashed, so run `bee dev regen` again and include the manifest. Then run `bee knowledge seed-anchors` once over areas/ and commit the seeded frontmatter; record in the cap report how many concepts received anchors and how many got none. Three seeded concepts are rule homes with `bee.applied_at` (workflow-state/capture-queue-and-the-blocker-threshold.md, doctrine-layer/delegation-threshold.md, hook-runtime/governed-paths-and-the-intake-gate.md); their frontmatter change trips sync check (b) at cap, so cap with `--sync-ack \"seed wrote bee.anchors frontmatter only; applied_at files unchanged\"` — this ack is predicted here, not a deviation. Finish with `bee knowledge check` (0 OKF errors, 0 profile errors; the stale_anchor warnings it now prints are expected and are the next feature's input — quote the freshness count line in the cap), `bee knowledge index` then `index --check`, and `bee dev release-manifest --check`.",
    "verify": ".bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md states the homes rule, the three fields, the grade rule by pointer, the cap step, the host seed step and the pull-rate measure, and is linked from the area overview",
        "skills/bee-capturing/SKILL.md names re-confirm, bee.anchors on new entries and the one-time seed, citing the concept",
        "every area concept with at least one resolving backticked repo path carries bee.anchors and a baseline verified_commit",
        "bee knowledge check reports 0 OKF errors and 0 profile errors after seeding, with the freshness count line quoted in the cap",
        "the installed .bee/bin/bee used for the seed carries the seed-anchors verb"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md", "substantive": "the homes rule and the freshness mechanic, with Pointers and its own anchors"},
        {"path": "skills/bee-capturing/SKILL.md", "substantive": "re-confirm and seed paragraph in the capture discipline"}
      ],
      "key_links": [
        "okf-profile/overview.md links the new concept",
        "the release manifest hash for skills/bee-capturing/SKILL.md matches the edited file"
      ],
      "prohibitions": [
        "Do not rewrite or cut any existing area narrative beyond the named delivery and work-item sentences; the content cut is the next feature",
        "Do not write a glossary or an event catalogue",
        "Do not hand-edit frontmatter the seed verb writes",
        "Do not seed patterns/",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": false
  },
  {
    "id": "kfm-7",
    "feature": "knowledge-freshness-mechanic",
    "title": "Derive the preamble pathological-budget fixture bound from the render",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-1"],
    "decisions": ["da4ccfeb-495d-471d-af2f-80f8cec4d3c0"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix-first cell (never build on red). kfm-1 added two verb names to the command surface and the test `a_tighter_budget_drops_the_digest_next_and_keeps_the_command_surface` in hooks/session_preamble/tests.rs went red: its hard-coded budget 8800 sat 12 bytes above the pathological fixture's floor, and the new names push the kept-surface render to 8813 bytes. The contract (decision da4ccfeb, contract:session-preamble-budget) is the DROP ORDER — recent decisions, then the critical-patterns digest, then command surface, then project map — not the number 8800. Rewrite that test (and its sibling `an_over_budget_render_drops_recent_decisions_first_and_names_where_to_fetch_it` only if it shares the brittle constant) so the budget is derived from the render: render once at PREAMBLE_BUDGET_BYTES, measure the byte length of the `### Critical patterns (digest)` section, and set the tighter budget to (that render's length − digest length + a small slack such as 8) so that the digest no longer fits but the command surface still does; keep every existing assertion about which sections remain and the Omitted line. Red-first: confirm the test is red at HEAD before the change and green after; the sibling budget tests and `the_preamble_budget_sits_below_the_host_inline_limit` stay untouched and green. Never widen a literal budget by hand.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- session_preamble",
    "must_haves": {
      "truths": [
        "a_tighter_budget_drops_the_digest_next_and_keeps_the_command_surface is green at HEAD and its budget is computed from the render, not a literal",
        "every other session_preamble test is unchanged and green",
        "PREAMBLE_BUDGET_BYTES and the drop order are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs", "substantive": "the tighter-budget test derives its bound from the digest section length"}
      ],
      "key_links": [
        "the derived budget sits strictly between the kept-surface render size and the digest-included render size"
      ],
      "prohibitions": [
        "Do not change budget.rs or any renderer code",
        "Do not widen a literal budget constant",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": false
  }
]
```

## Test matrix

| Scenario | Cell | Pass when |
|---|---|---|
| Happy: concept with anchors confirmed at HEAD | kfm-1 | `grade == confirmed`, `changed == []` |
| Edge: anchor changed in a commit that also touched the concept | kfm-1 | `grade == confirmed` |
| Edge: anchor changed in a later commit without the concept | kfm-1 | `grade == suspect`, `changed` names the anchor |
| Edge: anchor changed by a merge commit | kfm-1 | the merge commit is seen; grade follows the rule |
| Error: anchor path missing | kfm-1 | `grade == orphan` |
| Error: verified_commit present but not an ancestor of HEAD; unknown sha; git missing | kfm-1 | `grade == unverified`, `reason` non-empty |
| Happy: seed writes anchors and the baseline commit, file stays canonical | kfm-1 | `knowledge check` has no not_canonical row for the seeded file |
| Happy: confirm from a granted worktree | kfm-1 | the worktree's concept is stamped; main's copy unchanged |
| Happy: search, show, context rows carry grade | kfm-2 | JSON row has `grade`, `verified_at`, `verified_commit` |
| Edge: preamble without a bundle, with zero area concepts, past the budget | kfm-2 | no line; no line; skipped note |
| Error: cap touches anchored file, concept suspect / orphan | kfm-3 | refusal names the concept, grade and the remedy for that grade |
| Happy: unverified passes; same cap with concept in touched set; after confirm; with --sync-ack | kfm-3 | cap succeeds; sync_ack reason recorded when used |
| Edge: cell committed in a granted worktree | kfm-3 | the door grades the worktree's bundle |
| Edge: repo with no bundle | kfm-3 | cap behaves as on main (same scenario run on main and on head) |
| Happy: promote on a fixture | kfm-4 | output has no `delivery` key and no delivery.md written |
| Edge: bundle with a work/ dir, verbs walk and preamble walk | kfm-4 | nothing listed under work/ by either |
| Happy: check after the move | kfm-4 | zero dangling_source rows in `--json` |
| Happy: context --work on a closed feature after the move | kfm-4 | manifest returned from the docs/history anchor |
| Happy: env-bound session timing row | kfm-5 | row has `feature == <slug>` |
| Edge: no env session id | kfm-5 | row has `feature == null` |
| Error: malformed timings line | kfm-5 | skipped, `malformed_count == 1` |
| Happy: seeded bundle | kfm-6 | `bee knowledge check` 0 errors; freshness line printed |

## Open Questions

(none)

## Out of scope

- Cutting bee's own area narratives (map ticket 002) — next feature.
- Glossary and event catalogue (tickets 003, 004) — wait on pull rate.
- A freshness grade on patterns — patterns keep a date; neither graded nor seeded.
- Instrumenting cell outcomes by knowledge-used arm.
- A worker that escapes (d) with `--sync-ack` also skips (a)–(c): the existing one-ack door shape is kept and noted.
