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
all of this through onboarding.

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
| 5 | Knowledge verbs route through one match in routing.rs, where a new verb lands | read | packages/bee-rs/crates/bee/src/verbs/knowledge/routing.rs:34 | `"check" => run_check(flags, json, pre_json, t0),` |
| 6 | The cap-time sync door is one function taking the touched path set, so an anchored-concept check is a fourth branch | read | packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs:20 | `pub(crate) fn sync_refusal(root: &Path, cell: &Map<String, Value>, touched: &[String]) -> Option<String> {` |
| 7 | The cap builds `touched_set` from the HEAD commit numstat plus files_changed before calling the door | read | packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:752 | `if let Some(refusal) = sync_refusal(root, &cell_map, &touched_set) {` |
| 8 | Promote writes the delivery draft under `<work_dir>/delivery.md`; removing section (a) removes the record | read | packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs:505 | `format!("{work_dir}/delivery.md")` |
| 9 | The bundle walk is one function; a `work/` skip lands there for index, check and context alike | read | packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs:38 | `pub(crate) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {` |
| 10 | `knowledge context --work` already falls back to docs/history when no work-item concept exists, so moving `work/` does not break it | read | packages/bee-rs/crates/bee/src/verbs/knowledge/anchor.rs:3 | `// whose bee.id matches `work` always wins; otherwise docs/history/<work>/` |
| 11 | Timings rows are written by one function with `{ts, cmd, ms, ok}`; a feature field is added there | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:182 | `pub(crate) fn record_timing(root: &Path, cmd: &str, t0: Instant, ok: bool) {` |
| 12 | The calling session's bound lane is readable through one store function | read | packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:456 | `pub(crate) fn session_binding(flag: Option<&str>, root: &Path) -> R2<(Option<String>, Option<String>)> {` |
| 13 | The report verb builds its sections in `build_report`, where pull rate is one more section | read | packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs:253 | `pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Value, Vec<String>) {` |
| 14 | The preamble's knowledge lines are rendered by one function | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs:252 | `pub(crate) fn knowledge_context_lines(root: &Path, record: &JMap) -> Vec<String> {` |
| 15 | New frontmatter values must satisfy the canonical emitter or `not_canonical` fires | read | packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs:22 | `pub(crate) fn is_plain_safe(value: &str) -> bool {` |
| 16 | bee already shells to git through one helper | read | packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:85 | `pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {` |
| 17 | Served verbs are declared in the generated registry as `{name, invoke, description}` rows; the registry tests fail on an undeclared verb | ran | `python3 -I -c "import json; d=json.load(open('packages/bee-rs/crates/bee/src/generated/registry_payload.json')); print(json.dumps(d['commands'][129])[:60])"` | `{"name": "knowledge.check", "invoke": "bee knowledge check"` |
| 18 | The cap sync door's ownership check fires for cells/* code, so a cell touching sync_door.rs must touch a workflow-state skill | read | docs/knowledge/areas/workflow-state/overview.md:15 | `owns.skills: ["skills/bee-planning/*", "skills/bee-swarming/*", "skills/bee-reviewing/*", "skills/bee-capturing/*"]` |
| 19 | The release manifest hashes skill files, so a cell touching skills/ must regen and check it | ran | `rg -c 'skills/bee-swarming/SKILL.md\|skills/bee-capturing/SKILL.md' docs/history/codex-harness-hardening/release-manifest.json` | `12` |
| 20 | Area concepts carry no verified field today; only authored `timestamp` | ran | `rg --no-filename -o '^  ?(timestamp\|updated\|verified_at\|last_verified\|checked_against\|sources\|lifecycle):' docs/knowledge/areas \| sort \| uniq -c` | `143   lifecycle:` / `141   sources:` |

## Discovery

Direct reads of the knowledge verbs, the cap door and the preamble
(anchors above). Findings: anchor-shaped resolution, verb routing, the
registry row shape and the git helper all exist; no knowledge verb is owned
by an area (`owns.code` lists name only discovery, worktree, decision-memory
and workflow-state code), so kfm-1, kfm-2, kfm-4 and kfm-5 pass the
ownership check untouched, while kfm-3 touches cells/* and therefore
updates the bee-swarming skill (claim 18). Measured before planning: 95 of
139 area concepts had an anchored Rust file change after the concept's last
touch; `knowledge search` ran 89 times in 35,478 CLI invocations.

## Approach

Recommended path: one new module `verbs/knowledge/freshness.rs` owns the
grade (D3, D8) and the two new verbs `knowledge anchors seed` and
`knowledge confirm`; check, search, show, context and the preamble call its
one `grade_concept` reader (kfm-1, kfm-2). The cap door gains branch (d)
over the same reader (kfm-3, D4). Promote loses its delivery section and
the walk skips a top-level `work/` while the tree moves to
`docs/history/knowledge-work/` (kfm-4, D4, D5). Timings rows gain the
bound feature and the report counts pulls (kfm-5, D6). The OKF profile
area and bee-capturing document the homes rule and the re-confirm step, and
the seed verb runs once over bee's bundle (kfm-6, D2, D7).

Grade rule (the one semantic the worker must keep): `orphan` when any
anchor path is missing on disk; else `unverified` when `verified_commit` is
absent, unresolvable, or git is unavailable (the reason is named); else
`suspect` when any anchor was changed by a commit after `verified_commit`
that did not also touch the concept file; else `confirmed`. "After" means
reachable from HEAD and not reachable from `verified_commit`
(`verified_commit..HEAD`), so a squash or rebase that rewrites the sha
grades `unverified`, never silently confirmed. A commit that touches both
the anchor and the concept counts as a confirmation, which is how a worker
edits code and the concept in one cell commit without grading itself
suspect.

Rejected: grading from the Pointers prose (the regex parse that rotted);
grading from `bee.sources` (provenance strings, not paths); a review
calendar (nobody runs it); keeping `work/` and excluding it by index only
(still rots, still confuses a tree walk); pushing the manifest into briefs
(D1 rejected it).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Grade semantics wrong for squash/rebase or same-commit edits | MEDIUM — fact: git.rs `run_git` exists; the rule above is pinned by unit tests on a fixture repo with real commits | kfm-1 | tests: four grades, same-commit edit confirmed, unknown sha unverified |
| New fields break canonical emit | LOW | kfm-1 | `knowledge check` on the fixture reports 0 not_canonical for seeded files |
| Cap door (d) refuses legitimate caps | MEDIUM — fact: handlers_close.rs:752 builds touched_set from HEAD numstat; a concept edited in the same commit is in that set | kfm-3 | tests: refusal, same-commit pass, --sync-ack escape, confirm then pass |
| Moving 453 files breaks context/index/check | MEDIUM — fact: anchor.rs falls back to docs/history | kfm-4 | `bee knowledge check`, `index --check`, `context --work <closed feature>` green after the move |
| Timings join attributes pulls to the wrong feature | LOW | kfm-5 | tests: bound session writes feature, unbound writes null |
| Skill and manifest drift | LOW | kfm-3, kfm-6 | `bee dev release-manifest --check` green |

Waves: kfm-1, kfm-4 and kfm-5 run first in parallel (disjoint files except
knowledge/tests.rs, which serializes them by the overlap rule). kfm-2 and
kfm-3 wait for kfm-1 (they call its reader) and run in parallel. kfm-6 waits
for kfm-1 (it runs the seed verb) and kfm-4 (the moved tree must not be
seeded).

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
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane with more than five product files: the plan-step hat wave runs."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat."},
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
| 1 | anchors + grades + verbs, grade printing, cap door (d), promote and work/ move, pull rate, docs + seed | the mechanic is one contract; shipping half of it leaves hosts with fields nobody grades | `bee knowledge check` lists suspect concepts; search prints a grade; a cap that touches an anchored file refuses until confirm; report prints pulls per feature | the bee-only content cut (next feature) |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| kfm-1 | Grade concept freshness from anchors and git, with seed and confirm verbs | knowledge/freshness.rs (new), knowledge/check.rs, knowledge/mod.rs, knowledge/routing.rs, knowledge/tests.rs, generated/registry_payload.json | — | `bee knowledge check` prints a freshness count line and one WARN row per suspect or orphan concept; `bee knowledge anchors seed` fills `bee.anchors` from resolving Pointers; `bee knowledge confirm --id` stamps verified_at and verified_commit | knowledge and registry tests green |
| kfm-2 | Print the grade wherever a concept is read | knowledge/search.rs, knowledge/author.rs, knowledge/context.rs, knowledge/tests.rs, session_preamble/render.rs, session_preamble/knowledge.rs, session_preamble/tests.rs | kfm-1 | search, show and context rows carry grade, verified_at and verified_commit in text and JSON; the preamble prints `knowledge: N confirmed, M suspect, K orphan, U unverified` | knowledge and session_preamble tests green |
| kfm-3 | Refuse a cap that leaves an anchored concept unconfirmed | cells/sync_door.rs, cells/tests.rs, skills/bee-swarming/SKILL.md, docs/history/codex-harness-hardening/release-manifest.json | kfm-1 | a cap whose touched files anchor a concept not confirmed at that commit names the concept and the remedy; `--sync-ack` still escapes; the worker contract names the re-confirm step | cells tests and `bee dev release-manifest --check` green |
| kfm-4 | Retire delivery records and move work/ out of the bundle | knowledge/promote.rs, knowledge/walk.rs, knowledge/tests.rs, docs/knowledge/work/* (deleted), docs/history/knowledge-work/* (new), docs/knowledge/index.md, docs/knowledge/*/index.md | — | promote proposes patterns and area bullets only; `bee knowledge check` and `index --check` are green with 321 concepts; `context --work <closed feature>` still answers from docs/history | knowledge tests, check and index --check green |
| kfm-5 | Count knowledge pulls per feature | verbs/mod.rs, knowledge/report.rs, knowledge/tests.rs | — | every timings row carries the bound feature or null; `bee knowledge report` prints a pull-rate table per feature | knowledge tests green |
| kfm-6 | Write the homes rule and re-confirm step, and seed bee's own anchors | docs/knowledge/areas/okf-profile/freshness-and-document-homes.md (new), docs/knowledge/areas/okf-profile/index.md, docs/knowledge/areas/okf-profile/overview.md, skills/bee-capturing/SKILL.md, docs/knowledge/areas/*/*.md, docs/knowledge/patterns/*.md, docs/history/codex-harness-hardening/release-manifest.json | kfm-1, kfm-4 | a new OKF concept states the four homes, the grade rule and the cap step; bee-capturing names re-confirm; every area and pattern concept with a resolving Pointers path carries `bee.anchors` and `knowledge check` grades it | `bee knowledge check` 0 errors and `bee dev release-manifest --check` green |

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
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/frontmatter.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/author.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add a freshness grade to the knowledge bundle (D3 27ad96a5, D8 4148025d). New module verbs/knowledge/freshness.rs exports one reader `grade_concept(root, concept) -> Grade { grade: confirmed|suspect|orphan|unverified, verified_at: Option<String>, verified_commit: Option<String>, anchors: Vec<String>, changed: Vec<String>, reason: Option<String> }` over three new frontmatter fields under `bee`: `anchors` (flat list of repo-relative paths), `verified_at` (ISO date) and `verified_commit` (full sha). Grade rule, exactly: `orphan` when any anchor path is missing on disk; else `unverified` when there are no anchors, or verified_commit is absent or does not resolve (`git cat-file -e <sha>^{commit}` fails), or git is unavailable — `reason` names which; else `suspect` when any anchor was changed by a commit in `verified_commit..HEAD` that did NOT also touch the concept file (use `git log --format=%H <verified_commit>..HEAD -- <anchor>` and, for each such commit, `git show --name-only --format= <sha>` to test whether the concept path is in it); else `confirmed`. Shell to git through verbs/worktree/git.rs `run_git` (anchor: `pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> GitOut {`). In check.rs, after the existing profile pass, grade every concept under areas/ and patterns/ (never work/, never index.md) and emit: one summary line `freshness: <n> confirmed, <n> suspect, <n> orphan, <n> unverified` in text, a `freshness` object in --json with those counts plus `rows: [{path, grade, verified_at, verified_commit, changed, reason}]`, and one profile WARNING row per suspect concept (code `stale_anchor`, message naming the changed anchor paths and the remedy `bee knowledge confirm --id <id>`) and per orphan concept (code `orphan_anchor`, naming the missing path). Unverified is counted, never warned. Two new verbs routed in routing.rs (anchor: `\"check\" => run_check(flags, json, pre_json, t0),`) and declared in generated/registry_payload.json as `{name, invoke, description}` rows like the existing `knowledge.check` row (the registry tests fail on an undeclared served verb; edit the JSON by a round-trip that keeps its byte format): `bee knowledge anchors seed [--dry-run]` walks areas/ and patterns/, and for every concept with NO `bee.anchors` key collects the backticked repo-relative paths in its body that exist on disk (same backtick-path shape dangling_source_candidate accepts; skip paths under .bee/ and docs/), writes them as `bee.anchors` through the canonical emitter (frontmatter.rs `emit_scalar`/`is_plain_safe` — a seeded file must report zero not_canonical), leaves `verified_*` unset, and prints per-file counts plus how many concepts got zero anchors; `bee knowledge confirm --id <bee.id>` refuses an unknown id or a concept with no anchors, otherwise sets `bee.verified_at` to today (UTC, ISO date) and `bee.verified_commit` to the current HEAD sha and re-emits canonically. Tests red-first in knowledge/tests.rs on a fixture repo with real git commits: each of the four grades; an anchor changed in a commit that also touched the concept grades confirmed; an unknown verified_commit grades unverified with a reason; seed writes anchors only where absent and the file stays canonical; confirm stamps both fields and refuses an unknown id; check's --json carries the freshness object and the stale_anchor / orphan_anchor rows.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge registry",
    "must_haves": {
      "truths": [
        "grade_concept returns confirmed, suspect, orphan or unverified by the rule in plan.md Approach, and never confirmed when git is unavailable",
        "an anchor changed in a commit that also touched the concept file does not make the concept suspect",
        "bee knowledge check prints the freshness count line, carries the freshness object in --json, and emits stale_anchor and orphan_anchor warnings",
        "bee knowledge anchors seed writes bee.anchors only to concepts without one and keeps every written file canonical",
        "bee knowledge confirm --id stamps verified_at and verified_commit and refuses an unknown id"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs", "substantive": "grade_concept reader plus the seed and confirm verbs"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "knowledge.anchors.seed and knowledge.confirm declared"}
      ],
      "key_links": [
        "check.rs calls grade_concept for every areas/ and patterns/ concept",
        "routing.rs serves anchors seed and confirm"
      ],
      "prohibitions": [
        "Do not grade or seed anything under docs/knowledge/work/",
        "Do not change any existing finding code or its wording",
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
    "action": "Make the freshness grade visible at every pull point (D1 2b11f025, D3 27ad96a5), using only `grade_concept` from kfm-1 — never a second grading rule. search.rs: `SearchHit` (anchor: `pub(crate) struct SearchHit {`) gains grade, verified_at and verified_commit; the text line becomes `<path> — <title> — [<grade> @ <verified_at or never>] — <why>` and the JSON row (anchor: `row.insert(\"why\".into(), Value::String(hit.why.clone()));`) gains `grade`, `verified_at`, `verified_commit`. author.rs `run_show` prints a `freshness:` line after the metadata and adds the same three keys to its JSON. context.rs: every manifest entry (ranks 1-5) carries `grade` and `verified_at`; a suspect or orphan entry's inclusion reason ends with ` (grade: suspect — <first changed anchor>)` so the agent sees it in the reason column; ranking is unchanged. session_preamble: render.rs `knowledge_context_lines` (anchor: `pub(crate) fn knowledge_context_lines(root: &Path, record: &JMap) -> Vec<String> {`) adds ONE line `- knowledge: <n> confirmed, <m> suspect, <k> orphan, <u> unverified — suspect first: bee knowledge check` computed over areas/ and patterns/; the line is omitted when the bundle is absent, and the preamble byte budget must not grow by more than that one line (existing budget tests stay green). Tests red-first: a search row and a show output carry the grade of a fixture concept in each of two grades; a context manifest entry carries grade and the suspect reason suffix; the preamble line counts a fixture bundle correctly and is absent without a bundle.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge session_preamble",
    "must_haves": {
      "truths": [
        "knowledge search text and JSON rows carry grade, verified_at and verified_commit",
        "knowledge show prints the grade and carries it in JSON",
        "knowledge context entries carry grade and a suspect or orphan entry names its first changed anchor in the reason",
        "the session preamble prints one freshness count line and nothing else new"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/search.rs", "substantive": "SearchHit carries the grade fields and both outputs print them"}
      ],
      "key_links": [
        "search.rs, author.rs, context.rs and render.rs all call freshness::grade_concept"
      ],
      "prohibitions": [
        "Do not change search ranking or context ranking",
        "Do not add a second grading rule anywhere",
        "No code comments"
      ],
      "open_choices": []
    },
    "behavior_change": true
  },
  {
    "id": "kfm-3",
    "feature": "knowledge-freshness-mechanic",
    "title": "Refuse a cap that leaves an anchored concept unconfirmed",
    "lane": "standard",
    "role": "code",
    "deps": ["kfm-1"],
    "decisions": ["e7e80d78-f172-410d-b142-7b2bd464105e"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs",
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
    "action": "Add check (d) to the cap-time sync door (D4 e7e80d78) in sync_door.rs `sync_refusal` (anchor: `pub(crate) fn sync_refusal(root: &Path, cell: &Map<String, Value>, touched: &[String]) -> Option<String> {`), after check (c): for every areas/ or patterns/ concept whose `bee.anchors` (normalized with normalize_cell_path) intersects `touched`, the cap passes when the concept's own path is in `touched` (edited or confirmed in this cell's commit) OR `grade_concept` returns confirmed; otherwise refuse with `capCell: SYNC_DOOR — cell \"<id>\" touches <paths> anchored by knowledge concept(s) <concept paths> that are not re-confirmed. FIX: bee knowledge confirm --id <bee.id> (then commit the concept with the cell), edit the concept, or pass --sync-ack \"<reason>\" to cap anyway (the reason is recorded on trace.sync_ack and trace.deviations).` The existing `--sync-ack` escape in handlers_close.rs covers (d) with no new flag. A bundle with no concepts, or a repo without docs/knowledge/, makes (d) a no-op. In skills/bee-swarming/SKILL.md, under the worker contract (\"Execute\"), add the step: before committing, run `bee knowledge check --json` and for every suspect concept anchored by a file you changed, either fix the concept or run `bee knowledge confirm --id <id>`, and commit the concept file with the cell — one sentence, cited to this feature's CONTEXT.md D4, no second copy of the grade rule. Because skills/ is hashed by the release manifest, run `bee dev regen` and include the regenerated manifest. Tests red-first in cells/tests.rs with a fixture repo and bundle: a cap touching an anchored file with the concept unconfirmed refuses and names the concept and the confirm remedy; the same cap passes when the concept file is in the touched set; passes after `knowledge confirm` with the concept committed; passes with --sync-ack and records the reason; a touched file anchored by no concept passes untouched.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- cells::tests::sync && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "a cap whose touched files anchor an unconfirmed concept refuses, naming the concept and bee knowledge confirm as the remedy",
        "a cap passes when the concept is in the touched set, confirmed at that commit, or escaped with --sync-ack",
        "a repo without a bundle caps exactly as before",
        "the bee-swarming worker contract names the re-confirm step"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs", "substantive": "check (d) over bee.anchors using freshness::grade_concept"},
        {"path": "skills/bee-swarming/SKILL.md", "substantive": "one re-confirm step in the worker contract"}
      ],
      "key_links": [
        "sync_door.rs calls knowledge::freshness::grade_concept",
        "the release manifest hash for skills/bee-swarming/SKILL.md matches the edited file"
      ],
      "prohibitions": [
        "Do not change checks (a), (b) or (c) or their wording",
        "Do not add a new cap flag",
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
    "deps": [],
    "decisions": ["e7e80d78-f172-410d-b142-7b2bd464105e", "28fb074d-82c5-4962-85ba-a430b13b9292"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
      "docs/knowledge/work/*",
      "docs/history/knowledge-work/*",
      "docs/knowledge/index.md",
      "docs/knowledge/areas/index.md",
      "docs/knowledge/patterns/index.md"
    ],
    "read_first": [
      "docs/history/knowledge-freshness-mechanic/CONTEXT.md",
      "docs/history/knowledge-freshness-mechanic/plan.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/promote.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/anchor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Stop generating delivery records and take the delivery-log tree out of the bundle (D4 e7e80d78, D5 28fb074d). promote.rs: remove section (a), the delivery draft (anchor: `format!(\"{work_dir}/delivery.md\")`), from both the text proposal and the --json output; the proposal keeps (b) area update bullets and (c) pattern candidates and says in its header that it proposes those two only; update the promote tests in knowledge/tests.rs that assert `delivery` (rg anchors: `assert_eq!(p[\"delivery\"][\"path\"], \"work/delivery.md\");` and `assert_eq!(p[\"delivery\"][\"path\"], \"work/hist-1/delivery.md\");`). walk.rs `list_bundle_markdown` (anchor: `pub(crate) fn list_bundle_markdown(dir: &Path) -> Option<Vec<String>> {`) skips a top-level `work/` directory, so check, index, context, list, search and the preamble never see it; add a test that a bundle with work/ lists nothing under it. Move the tree with `git mv docs/knowledge/work docs/history/knowledge-work` (453 files; one commit with the code), then run `bee knowledge index` so the root and section indexes no longer list a work section, and confirm `bee knowledge check` reports 0 OKF errors and `bee knowledge index --check` 0 stale. Confirm `bee knowledge context --work <a closed feature, e.g. windows-ci-1107> --budget 4000` still answers from the docs/history anchor (anchor.rs: `// whose bee.id matches `work` always wins; otherwise docs/history/<work>/`). Do not touch areas/ or patterns/ content.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge && .bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check",
    "must_haves": {
      "truths": [
        "bee knowledge promote emits no delivery section in text or JSON",
        "docs/knowledge/work/ no longer exists and docs/history/knowledge-work/ holds the moved files",
        "bee knowledge check, index --check and list ignore a top-level work/ directory",
        "bee knowledge context --work on a closed feature still returns a manifest from the docs/history anchor"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/walk.rs", "substantive": "top-level work/ skipped in the bundle walk"},
        {"path": "docs/knowledge/index.md", "substantive": "regenerated without a work section"}
      ],
      "key_links": [
        "every bundle reader goes through list_bundle_markdown and therefore skips work/"
      ],
      "prohibitions": [
        "Do not edit any concept under areas/ or patterns/",
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
    "deps": [],
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
    "action": "Measure pull rate (D6 e7e133dc). verbs/mod.rs `record_timing` (anchor: `pub(crate) fn record_timing(root: &Path, cmd: &str, t0: Instant, ok: bool) {`) adds a `feature` field to every timings row: the calling session's bound lane feature from state_group/store.rs `session_binding(None, root)` (anchor: `pub(crate) fn session_binding(flag: Option<&str>, root: &Path) -> R2<(Option<String>, Option<String>)> {`), or null when the session is unbound or the read fails — fail-open, never a refusal, and the existing `{ts, cmd, ms, ok}` keys and the stderr line are unchanged. report.rs `build_report` (anchor: `pub(crate) fn build_report(root: &Path, concepts: &[Concept]) -> (Vec<Value>, Vec<Value>, Value, Vec<String>) {`) adds a `pull_rate` section read from .bee/logs/timings.jsonl: per feature (null grouped as `(unbound)`), counts of `knowledge search`, `knowledge context` and their sum, plus a total row; text output prints it as a small table under a `Pull rate` heading; malformed rows are skipped and counted in `malformed_count` like timings report does; a missing log prints an empty table. Tests red-first: a timings row written by a session bound to a lane carries that feature; an unbound session writes null; the report groups a fixture log by feature and skips a malformed line.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- knowledge timings",
    "must_haves": {
      "truths": [
        "every new timings row carries feature (string or null) beside the unchanged ts, cmd, ms, ok keys",
        "bee knowledge report prints a pull-rate table per feature with search, context and total counts"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs", "substantive": "pull_rate section built from timings.jsonl"}
      ],
      "key_links": [
        "record_timing reads the bound lane through session_binding"
      ],
      "prohibitions": [
        "Do not make record_timing refuse or slow the command on any failure",
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
    "deps": ["kfm-1", "kfm-4"],
    "decisions": ["93e1001d-bdf6-42fb-9297-9f4d8589a5ef", "6a502ad0-7273-4c91-9266-065e56eec25c", "4148025d-e997-4642-bab4-da00104c5795"],
    "files": [
      "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md",
      "docs/knowledge/areas/okf-profile/index.md",
      "docs/knowledge/areas/okf-profile/overview.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/areas/*",
      "docs/knowledge/patterns/*",
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
    "action": "Document the mechanic and apply it to bee's bundle (D2 93e1001d, D7 6a502ad0, D8 4148025d). Write one new concept docs/knowledge/areas/okf-profile/freshness-and-document-homes.md through `bee knowledge new --type <the area concept type used by the sibling files> --area okf-profile` and then edit its body: (1) the document homes rule — the four kinds that live in docs/knowledge (vision, glossary, area map with boundaries and invariants, decisions, patterns) and the three that live in code (verify features and tests, generated contracts, runnable runbooks), with the sentence that every area entry anchors to at least one code file or verify feature; (2) the three fields `bee.anchors`, `bee.verified_at`, `bee.verified_commit` and the grade rule, cited from plan.md Approach by pointer, not copied; (3) the cap re-confirm step and the `--sync-ack` escape; (4) the pull-rate measure and what a flat rate means (the lean layer is deleted next); (5) a Pointers section naming freshness.rs, sync_door.rs, report.rs and this feature's CONTEXT.md; and set its own `bee.anchors` to those three Rust files. Link it from okf-profile/overview.md (one bullet) and regenerate indexes with `bee knowledge index`. In skills/bee-capturing/SKILL.md add, in the capture discipline, one short paragraph: when a captured behaviour change touches a file an area concept anchors, re-confirm or edit that concept (`bee knowledge confirm --id`), and new area entries carry `bee.anchors` — cite the new concept by path, no second copy of the rule; because skills/ is hashed, run `bee dev regen` and include the regenerated manifest. Then run `bee knowledge anchors seed` once over areas/ and patterns/ (kfm-1's verb; kfm-4 has already moved work/ out) and commit the seeded frontmatter; record in the cap report how many concepts received anchors and how many got none. Finish with `bee knowledge check` (0 OKF errors, 0 profile errors; the stale_anchor warnings it now prints are expected and are the next feature's input — quote the freshness count line in the cap) and `bee dev release-manifest --check`.",
    "verify": ".bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md states the homes rule, the three fields, the grade rule by pointer, the cap step and the pull-rate measure, and is linked from the area overview",
        "skills/bee-capturing/SKILL.md names the re-confirm step and bee.anchors on new entries, citing the concept",
        "every area and pattern concept with at least one resolving backticked repo path carries bee.anchors",
        "bee knowledge check reports 0 OKF errors and 0 profile errors after seeding"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/okf-profile/freshness-and-document-homes.md", "substantive": "the homes rule and the freshness mechanic, with Pointers and its own anchors"},
        {"path": "skills/bee-capturing/SKILL.md", "substantive": "re-confirm paragraph in the capture discipline"}
      ],
      "key_links": [
        "okf-profile/overview.md links the new concept",
        "the release manifest hash for skills/bee-capturing/SKILL.md matches the edited file"
      ],
      "prohibitions": [
        "Do not rewrite or cut any existing area narrative; that is the next feature",
        "Do not write a glossary or an event catalogue",
        "Do not hand-edit frontmatter the seed verb writes",
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
| Error: anchor path missing | kfm-1 | `grade == orphan` |
| Error: verified_commit unknown sha or git missing | kfm-1 | `grade == unverified`, `reason` non-empty |
| Happy: seed writes anchors, file stays canonical | kfm-1 | `knowledge check` has no not_canonical row for the seeded file |
| Happy: search, show, context rows carry grade | kfm-2 | JSON row has `grade`, `verified_at`, `verified_commit` |
| Edge: preamble without a bundle | kfm-2 | no freshness line, byte budget unchanged |
| Error: cap touches anchored file, concept unconfirmed | kfm-3 | refusal text names the concept and `bee knowledge confirm --id` |
| Happy: same cap with concept in touched set, or after confirm, or with --sync-ack | kfm-3 | cap succeeds; sync_ack reason recorded when used |
| Edge: repo with no bundle | kfm-3 | cap behaves as on main (same scenario run on main and head) |
| Happy: promote on a fixture | kfm-4 | output has no `delivery` key and no delivery.md written |
| Edge: bundle with a work/ dir | kfm-4 | list and check return nothing under work/ |
| Happy: context --work on a closed feature after the move | kfm-4 | manifest returned from the docs/history anchor |
| Happy: bound session timing row | kfm-5 | row has `feature == <slug>` |
| Edge: unbound session | kfm-5 | row has `feature == null` |
| Error: malformed timings line | kfm-5 | skipped, `malformed_count == 1` |
| Happy: seeded bundle | kfm-6 | `bee knowledge check` 0 errors; freshness line printed |

## Open Questions

(none)

## Out of scope

- Cutting bee's own area narratives (map ticket 002) — next feature.
- Glossary and event catalogue (tickets 003, 004) — wait on pull rate.
- A freshness grade on patterns beyond the date — patterns keep a date.
- Instrumenting cell outcomes by knowledge-used arm.
