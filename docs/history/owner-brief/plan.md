---
mode: standard
---

# owner-brief — plan

Route: class `feature` · lane `standard` · flags `public-contracts,external-systems,multi-domain` · product files 5.
Class playbook: `skills/bee-planning/playbooks/feature.md`.

## Summary

Today a backlog item reaches the owner's board with a title and one sentence, and the owner cannot decide on it. After this change, the door that adds an item takes a six-paragraph owner brief, checks it, and stores it at `docs/briefs/<id>.md`; an item added without one is still added but flagged. One new command lists the owner's view — active features with their in-flight items folded inside, plus the open items — and names every missing or broken brief. `bee doctor` notes the count. The skills and the knowledge teach agents to write the brief at Lock and when they move scope out to the backlog.

Mode: `standard` — 3 risk flags: public-contracts (the brief file format other repos parse), external-systems (waggledance and agent-works read it), multi-domain (CLI, doctor, skills).
Why this is the least workflow that protects the work: one code cell and one docs cell in parallel; the format is fixed by consumers, so the proof is a validator test suite pinned to the contract.

## Requirements (from CONTEXT.md)

- D1 (35853825): contract path, frontmatter keys and enums, six bold-labelled paragraphs, 120–220 words.
- D2 (fb0d79e9): no ids, paths, backticked code or call shapes in the body.
- D3 (701b2139): `brief.language` `vi` (default) or `en`; label sets fixed.
- D4 (95917163): `--brief-file` on `pbi add` and `propose`; invalid refuses and writes nothing; absent adds and flags.
- D5 (f14f8d2e): live = `proposed` or `in-flight`.
- D6 (1bf04c53): active feature = phase not `idle` / `compounding-complete`, default record or lane.
- D7 (357417c6): an in-flight PBI under an active feature folds into it and needs no own brief.
- D8 (344df11d): `bee backlog briefs [--id] [--json]` lists and checks; exit 1 on any problem.
- D9 (68bdb614): doctor advisory `owner_briefs` note row, never voting.
- D10 (3ace608d): `docs/backlog.md` unchanged; owner view is `bee backlog briefs`.
- D11 (f2985740): Lock writes the feature brief; deferral adds the PBI with `--brief-file` and the cost paragraph carries the reason.
- D12 (086e81a7): skills and one knowledge concept carry a good and a bad example.
- D13 (a3ac72ff): no staleness digest, no backfill.

## Load-bearing claims

Labels: `read` = opened at that line, `ran` = executed and output held. Match rule: the evidence column is a verbatim byte substring of the anchored line(s); multi-line evidence joins with ` / `.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | bee's PBI status set has no `ready` (D5) | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:66` | `const PBI_STATUSES: [&str; 5] = ["proposed", "in-flight", "parked", "done", "declined"];` |
| 2 | `pbi add` parses a fixed flag list, so `--brief-file` must be added there | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:676-677` | `"add" => run_pbi_add(` / `parse_shape(rest, &["title", "cos", "status", "feature", "id"])?,` |
| 3 | `propose` parses its own flag list | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:671` | `"propose" => run_propose(parse_shape(&args[2..], &["story", "cos", "feature"])?, t0),` |
| 4 | Doctor already carries a non-voting advisory to copy for D9 | read | `packages/bee-rs/crates/bee/src/doctor.rs:1505` | `let team_advisory = team_config_advisory(root);` |
| 5 | Config is read raw through one helper | read | `packages/bee-rs/crates/bee/src/state.rs:161` | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 6 | The native verb list names each backlog verb | read | `packages/bee-rs/crates/bee/src/router.rs:60` | `"backlog pbi add",` |
| 7 | A lane record carries its phase as a top-level field (D6) | ran | `rg -n '"phase"' .bee/lanes/owner-brief.json` (main checkout) | `5:  "phase": "exploring",` |

## Discovery

Read `backlog.rs` (`add_pbi`, `run_pbi_add`, `run_propose`, `try_native`, `run_render`), `doctor.rs` (`mechanical_rows_with_env`, `doctor_report`), `state.rs:161`, `router.rs:45-75`, the registry entries for `backlog.propose` and `backlog.pbi.add`, and `docs/backlog.md`. No brief concept exists in code; `docs/briefs/` does not exist. The write guard holds `docs/briefs/` until the execution gate, so this feature's own brief waits as `docs/history/owner-brief/brief-draft.md`.

## Approach

Recommended path: a new module `verbs/brief.rs` holds the contract (validator, language, owner view); `backlog.rs` wires `--brief-file` into the two add doors and adds the `briefs` verb; `doctor.rs` adds one advisory. Docs and skills land in parallel from the locked decisions alone.

Rejected alternatives:
- Require the brief (refuse an add with none): the spec says recorded and flagged, and a migration-era `--id` add would break.
- Agent writes `docs/briefs/<id>.md` before the call: the id is generated inside the add, so the agent cannot know the path.
- A doctor voting row: would turn every runtime check red in a repo that has not backfilled.
- Fold inside `docs/backlog.md`: breaks the render's byte-determinism contract for no owner gain (D10).

Plan-step hat wave: not run. Named deviation — the wave threshold (gates-and-delegation "Hat wave", Threshold) gives it to big, vague or high-risk work; this spec is clear, every delegated choice is recorded, and the consumers fix the format.

Risk map:

| component | risk | lands in | proof needed |
|---|---|---|---|
| validator drifts from what consumers parse | MEDIUM | ob-1 | the draft brief passes; each defect class refuses by name |
| a refused add leaves a half write | MEDIUM | ob-1 | test: malformed brief → no PBI event and no brief file |
| registry example fails `registry_dispatch` | LOW | ob-1 | `registry_dispatch` and `registry_contracts` green |
| skill render drift | LOW | ob-2 | `bee dev regen` clean, release-manifest check green |

Waves: ob-1 and ob-2 run in parallel — no shared file.

## Shape

Epic map. Outcome: every new item and feature carries a checked owner brief, and the owner view folds in-flight work into its feature.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | brief contract in the CLI | the add door is where the context lives | S1: ob-1 | validator, add, briefs verb, doctor tests green |
| E2 | teach the agents | cold agents copy examples | S1: ob-2 | regen clean; skills and concept name the verb |

Slice queue: S1 only.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ob-1 | Check owner briefs at the add door and list the owner view | `verbs/brief.rs` (new), `verbs/backlog.rs`, `verbs/mod.rs`, `doctor.rs`, `router.rs`, `generated/registry_payload.json` | — | `bee backlog pbi add --brief-file x.md` stores `docs/briefs/<id>.md` or refuses with every defect; with no brief it warns; `bee backlog briefs` lists features with folded items and names missing briefs | crate tests for brief, backlog, doctor and the registry green |
| ob-2 | Teach agents to write owner briefs at Lock and at deferral | skills (shaping, planning, capturing), knowledge concept, `docs/briefs/owner-brief.md`, regen output, release manifest | — | the Lock step and the deferral line name `--brief-file` and `bee backlog briefs`; the concept shows one good and one bad brief | regen clean, release-manifest check green, pointer check |

```json
[
  {
    "id": "ob-1",
    "feature": "owner-brief",
    "title": "Check owner briefs at the add door and list the owner view",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["35853825-ede5-40c2-8598-a6beffa5acdd", "fb0d79e9-7787-472d-953a-c50095b5e7e0", "701b2139-2e9c-487a-a874-2faff0c020ab", "95917163-6c3f-4dbb-bb0f-3c004c7545f7", "f14f8d2e-6140-4a66-bd96-b93295b07b54", "1bf04c53-dfc9-46ed-897e-e3ff0a3150c3", "357417c6-e763-4739-a479-90c6a4429370", "344df11d-8580-4f39-ae29-d2b37b96143e", "68bdb614-3607-481a-b6f6-30b3ee8449af", "3ace608d-e4f2-4f73-92cb-fd0c7975a88a"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/brief.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/src/router.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/owner-brief/CONTEXT.md",
      "docs/history/owner-brief/brief-draft.md",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/tests/registry_dispatch.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement D1-D10 of docs/history/owner-brief/CONTEXT.md. Steps in order.\n\n1. RED FIRST. Write the tests below, run them, and watch them fail before the code exists.\n\n2. New module packages/bee-rs/crates/bee/src/verbs/brief.rs, registered as `pub mod brief;` in verbs/mod.rs beside `pub mod backlog;`. It holds: (a) `pub(crate) fn brief_rel(id: &str) -> String` returning `docs/briefs/<id>.md`. (b) `pub(crate) fn brief_language(root: &Path) -> (String, Option<String>)` reading `brief.language` through `crate::state::read_config_raw`: absent -> (\"vi\", None); \"vi\" or \"en\" -> that; any other string -> (\"vi\", Some(<the value>)) (per D3). (c) `pub(crate) fn validate_brief(text: &str, language: &str) -> Vec<String>` returning every defect as a plain sentence, empty when valid. Normalize CRLF to LF. Rules (per D1, D2, D3): the text starts with a `---` line and a second `---` line closes the frontmatter; each non-empty frontmatter line is `key: value` with no leading whitespace (no nesting); keys title, audience, size, ask are required; title is non-empty and at most 80 characters (count chars, not bytes); audience in {owner, internal}; size in {S, M, L}; ask in {build, park, drop, answer, test}; other keys are allowed. Body = everything after the closing `---`. The six labels for vi are exactly `**Là gì:**`, `**Vì sao:**`, `**Nguồn gốc:**`, `**Chi phí và rủi ro:**`, `**Anh cần làm:**`, `**Chuẩn xong:**`; for en `**What:**`, `**Why:**`, `**Origin:**`, `**Cost and risk:**`, `**Your call:**`, `**Done when:**`. The first non-blank body line starts with the first label; each label starts exactly one line, the six appear in that order, and no other line starts with `**`; each label's section (its line after the label plus the lines up to the next label) holds non-blank text; the last section holds 2 to 4 lines starting with `- `. Word count = whitespace-separated body tokens containing at least one alphanumeric char (labels included, bare `-` excluded); it must be 120 to 220 inclusive, and the defect names the count. Forbidden in the body (each named with the token found): a token matching `p-` followed by 8 lowercase hex chars; a token (stripped of surrounding punctuation `()[]{},.;:!?\"'`) containing `/`; a token ending in .rs .md .json .js .mjs .ts .py .sh .toml .yml .yaml; any backtick char; any `identifier(` call shape (an ASCII letter or underscore run immediately followed by `(`). (d) `pub(crate) enum BriefStatus { Ok, Missing, Malformed(Vec<String>) }` and `pub(crate) fn brief_status(root: &Path, id: &str) -> BriefStatus`. (e) `pub(crate) fn owner_view(root: &Path) -> Value`: active features = the `feature` of `.bee/state.json` plus each `.bee/lanes/*.json` record's `feature`, keeping those whose `phase` is not `idle` and not `compounding-complete` (per D6), de-duplicated, sorted; PBIs from `super::backlog::fold_pbi_records(root)`; live = status proposed or in-flight (per D5); an in-flight PBI whose `feature` is an active feature is folded into that feature (per D7); every other live PBI is its own item. Output `{\"language\": <lang>, \"unknown_language\": <value or null>, \"items\": [{\"kind\": \"feature\"|\"pbi\", \"id\", \"title\" (PBI title; a feature uses its brief frontmatter title when the brief parses, else its slug), \"status\" (feature phase or PBI status), \"brief\": {\"status\": \"ok\"|\"missing\"|\"malformed\", \"path\", \"defects\": [..]}, \"folded\": [{\"id\", \"title\"}] (features only)}], \"problems\": <count of items whose brief is not ok>}`, features first, then PBIs in fold order.\n\n3. packages/bee-rs/crates/bee/src/verbs/backlog.rs (per D4, D8): add `brief-file` to the parse_shape lists of `pbi add` (`parse_shape(rest, &[\"title\", \"cos\", \"status\", \"feature\", \"id\"])?`) and `propose` (`run_propose(parse_shape(&args[2..], &[\"story\", \"cos\", \"feature\"])?, t0)`). In run_pbi_add and run_propose, after the preamble and BEFORE add_pbi: when --brief-file is given, read it (relative to the process cwd; unreadable -> emit_error naming the path, nothing written) and validate it with brief_language; any defect -> emit_error whose message starts `backlog pbi add: the owner brief <path> is malformed — nothing was added:` (or `backlog propose: ...`) followed by every defect, and nothing written. After a successful add with a brief: create `docs/briefs/` under ctx.root and write the file's bytes unchanged to `docs/briefs/<id>.md`. The success value gains `\"brief\": \"ok\"|\"missing\"` and `\"brief_path\": \"docs/briefs/<id>.md\"` (\"ok\" also when no flag was given but a valid file already sits there); when missing, the text output gains a second line `warning: no owner brief — write docs/briefs/<id>.md and check it with: bee backlog briefs --id <id>`. Add verb `briefs` to try_native: `\"briefs\" => run_briefs(parse_shape(&args[2..], &[\"id\"])?, t0)`. With --id: report that one brief (`ok docs/briefs/<id>.md`, or `MISSING`/`MALFORMED` plus one indented defect per line); exit 1 when not ok. Without --id: print owner_view, one line per item `<ok|MISSING|MALFORMED> <feature|pbi> <id> — <title>`, defects indented below, folded PBIs indented as `  folded <id> — <title>`, then a summary line `<n> item(s), <p> without a valid owner brief`; exit 1 when problems > 0, else 0. --json prints the same value (with the usual envelope the other backlog verbs use) and keeps the exit code rule. Do not change run_render or docs/backlog.md (per D10).\n\n4. packages/bee-rs/crates/bee/src/router.rs: add `\"backlog briefs\",` to the native list after `\"backlog pbi list\",`. packages/bee-rs/crates/bee/src/generated/registry_payload.json: add the `brief-file` string parameter to `backlog.pbi.add` and `backlog.propose` (description: path of an owner brief draft; validated against docs/briefs contract and copied to docs/briefs/<id>.md; malformed refuses the add; absent adds and warns), and a new `backlog.briefs` entry (invoke `bee backlog briefs`, parameters id and json, a description stating D5-D8 in plain words) whose examples[0] satisfies tests/registry_dispatch.rs's contract — read that test first; pick an example that exits the way the test expects in its fixture repo.\n\n5. packages/bee-rs/crates/bee/src/doctor.rs (per D9): add `fn owner_briefs_advisory(root: &Path) -> Option<(Value, String)>` beside `team_config_advisory`, calling `crate::verbs::brief::owner_view`; None when problems == 0; else a row `{\"row\": \"owner_briefs\", \"status\": \"note\", \"detail\": ...}` and the detail `<p> live item(s) or active feature(s) have no valid owner brief — run: bee backlog briefs`. In doctor_report push it after the team advisory and add a `  note owner_briefs ...` line in the same style; overall_status must not change.\n\n6. Tests (no comments anywhere): in brief.rs `#[cfg(test)] mod tests`: the vi draft at docs/history/owner-brief/brief-draft.md (copy its text into a const) validates clean; each defect class refuses by name (missing frontmatter, nested key, title of 81 chars, bad audience, bad size, bad ask, missing label, labels out of order, 119 words, 221 words, a 5-bullet last section, a p-1234abcd token, a path token, a .rs token, a backtick, a `foo(` call); an en brief validates under en and fails under vi; brief_language reads absent/vi/en/unknown; owner_view folds an in-flight PBI under an active lane feature, keeps a proposed PBI with the same feature as its own item, skips parked/done/declined and idle or compounding-complete features. In backlog.rs's tests module: pbi add with a valid --brief-file writes docs/briefs/<id>.md byte-identical and returns brief ok; with a malformed one refuses and leaves .bee/backlog.jsonl and docs/briefs untouched; without the flag adds and returns brief missing with the warning line; propose takes --brief-file the same way; briefs --id exit codes for ok/missing/malformed; briefs listing exit 1 with a missing brief and 0 when all ok. In doctor/tests.rs: the owner_briefs note appears with a missing brief and overall_status is unchanged. Follow each tests module's existing temp-repo helpers.\n\n7. Run the verify command; all green. NO CODE COMMENTS of any form in what you add. One commit, imperative subject, trailer `cell: ob-1`.",
    "must_haves": {
      "truths": [
        "bee backlog pbi add --brief-file with a valid brief stores it byte-identical at docs/briefs/<id>.md and reports brief ok",
        "a malformed brief refuses the add naming every defect, and no PBI event and no brief file is written",
        "pbi add and propose with no brief still add the item, report brief missing and print the warning line naming docs/briefs/<id>.md",
        "bee backlog briefs lists active features with their in-flight PBIs folded inside, lists unfolded proposed and in-flight PBIs, names each missing or malformed brief, and exits 1 when any exists",
        "bee backlog briefs --id checks one brief and exits 1 when it is missing or malformed",
        "the validator enforces the D1 frontmatter, the D3 label set for the configured language, 120-220 words and the D2 forbidden tokens",
        "bee doctor shows an owner_briefs note without changing overall_status",
        "docs/backlog.md rendering is unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/brief.rs", "substantive": "brief_rel, brief_language, validate_brief, BriefStatus, brief_status, owner_view and their unit tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/backlog.rs", "substantive": "--brief-file on pbi add and propose, run_briefs, and their tests"},
        {"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "owner_briefs_advisory wired into doctor_report as a note"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "brief-file parameters and the backlog.briefs entry"}
      ],
      "key_links": [
        "run_pbi_add and run_propose call validate_brief before add_pbi",
        "doctor_report calls owner_briefs_advisory which calls owner_view"
      ],
      "prohibitions": [
        "No change to run_render or docs/backlog.md",
        "No change to the PBI event shape in .bee/backlog.jsonl",
        "No code comment of any form",
        "No edit outside the listed files"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- brief backlog doctor && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_dispatch --test registry_contracts"
  },
  {
    "id": "ob-2",
    "feature": "owner-brief",
    "title": "Teach agents to write owner briefs at Lock and at deferral",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["35853825-ede5-40c2-8598-a6beffa5acdd", "fb0d79e9-7787-472d-953a-c50095b5e7e0", "701b2139-2e9c-487a-a874-2faff0c020ab", "95917163-6c3f-4dbb-bb0f-3c004c7545f7", "357417c6-e763-4739-a479-90c6a4429370", "344df11d-8580-4f39-ae29-d2b37b96143e", "3ace608d-e4f2-4f73-92cb-fd0c7975a88a", "f2985740-2550-4d4a-b85e-c267d04bbb60", "086e81a7-ed93-4739-af04-6b36eea1407e"],
    "files": [
      "docs/knowledge/areas/decision-memory/owner-brief-contract.md",
      "docs/knowledge/areas/decision-memory/index.md",
      "docs/knowledge/areas/decision-memory/backlog-store.md",
      "docs/briefs/owner-brief.md",
      "skills/bee-shaping/SKILL.md",
      "skills/bee-planning/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      ".claude-plugin/skills/*",
      ".codex-plugin/skills/*",
      ".claude/skills/*",
      ".agents/skills/*",
      ".opencode/skills/*",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/owner-brief/CONTEXT.md",
      "docs/history/owner-brief/spec.md",
      "docs/history/owner-brief/brief-draft.md",
      "docs/knowledge/areas/decision-memory/backlog-store.md",
      "skills/bee-shaping/SKILL.md"
    ],
    "affects_skills": ["skills/bee-shaping/SKILL.md", "skills/bee-planning/SKILL.md", "skills/bee-capturing/SKILL.md"],
    "affects_specs": ["docs/knowledge/areas/decision-memory/owner-brief-contract.md"],
    "action": "Write every sentence through the bee-technical-writing skill. Per D1-D12 of docs/history/owner-brief/CONTEXT.md.\n\n1. New knowledge concept docs/knowledge/areas/decision-memory/owner-brief-contract.md, in the frontmatter and section shape of backlog-store.md beside it. It is the ONE home of the contract: the path, the frontmatter keys and enums, the six labels for vi and en and the `brief.language` key (D3), the 120-220 word rule and how words are counted (D1), the forbidden body content (D2), who writes it and when (the adding agent through `bee backlog pbi add --brief-file` or `bee backlog propose --brief-file`; bee-shaping Lock for `docs/briefs/<feature>.md`; deferral with the reason and cost in the cost paragraph — D4, D11), what happens with no brief (added and flagged), the check `bee backlog briefs [--id <id>] [--json]` and its exit code (D8), the doctor note (D9), the live and active definitions (D5, D6), and the folding rule stated for consumers in one plain paragraph (D7) — consumers such as waggledance and agent-works apply the same rule; `docs/backlog.md` is the internal ledger (D10). Include one GOOD example: the full text of docs/history/owner-brief/brief-draft.md in a fenced block. Include one BAD example: the one-line CoS `A comment by a person on a mirrored task reaches the mapped project's leader as a dispatch; deferred from wd-mirror shaping`, followed by why it fails (no why, no cost, no reason for the deferral, no ask, an internal slug the owner cannot read). Pointers section names packages/bee-rs/crates/bee/src/verbs/brief.rs, verbs/backlog.rs and doctor.rs. Add its index line to docs/knowledge/areas/decision-memory/index.md in the existing list style, and one sentence plus link in backlog-store.md where the PBI add door is described.\n\n2. skills/bee-shaping/SKILL.md: in `## Lock (single writer)`, add a short paragraph: Lock also writes `docs/briefs/<feature>.md`, the owner brief, drawn from the Feature Boundary and the Locked Decisions in the owner language, and checks it with `bee backlog briefs --id <feature>`; the contract lives in the concept (link it by repo-relative path). Replace the two `bee backlog add` deferral mentions that defer product scope (the `Defer scope creep with` line in Explore and the `Deferred ideas that are real future work get` line in Lock) with `bee backlog pbi add --brief-file <draft>` whose cost paragraph says why the scope moved out and what waiting costs. Keep the Qualify and Brief sections as they are.\n\n3. skills/bee-planning/SKILL.md: in `## Scope integrity`, add one or two sentences: scope that moves out of the plan into the backlog goes through `bee backlog pbi add --brief-file <draft>`, with the reason for moving it and its cost in the cost paragraph; link the concept.\n\n4. skills/bee-capturing/SKILL.md: the deferral bullet in `## Capture the moment it settles` that sends user-deferred work to `bee backlog add` sends product work to `bee backlog pbi add --brief-file <draft>` instead (friction and findings keep `bee backlog add`); link the concept.\n\n5. Copy docs/history/owner-brief/brief-draft.md byte-identical to docs/briefs/owner-brief.md.\n\n6. Run `.bee/bin/bee dev regen` (render-skill-trees, onboard --apply, release-manifest --write) so every rendered skill copy and docs/history/codex-harness-hardening/release-manifest.json match (never hand-edit a rendered copy); commit the regen output with the rest. Then prove: `.bee/bin/bee dev release-manifest --check` green, `rg -l 'brief-file' skills/bee-shaping/SKILL.md skills/bee-planning/SKILL.md skills/bee-capturing/SKILL.md docs/knowledge/areas/decision-memory/owner-brief-contract.md` prints four paths, and `cmp docs/briefs/owner-brief.md docs/history/owner-brief/brief-draft.md` exits 0. One commit, imperative subject, trailer `cell: ob-2`.",
    "must_haves": {
      "truths": [
        "the knowledge concept states the full brief contract, the check verb, the doctor note, the live and active definitions and the folding rule for consumers",
        "the concept carries one good example and the p-bea1cb2a one-line CoS as the bad example with why it fails",
        "bee-shaping Lock writes docs/briefs/<feature>.md and checks it with bee backlog briefs --id",
        "shaping, planning and capturing send deferred product scope through bee backlog pbi add --brief-file with the reason and cost in the cost paragraph",
        "docs/briefs/owner-brief.md exists and equals the draft",
        "rendered skill copies match their sources after bee dev regen"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/decision-memory/owner-brief-contract.md", "substantive": "the one home of the owner brief contract with a good and a bad example"},
        {"path": "docs/briefs/owner-brief.md", "substantive": "this feature's own owner brief"}
      ],
      "key_links": [
        "the three skills link docs/knowledge/areas/decision-memory/owner-brief-contract.md",
        "docs/knowledge/areas/decision-memory/index.md lists the concept"
      ],
      "prohibitions": [
        "No hand edit of a rendered skill copy",
        "No second copy of the label set or word rule outside the concept",
        "No claim about CLI behavior that D4, D8 or D9 does not lock",
        "No edit to code"
      ]
    },
    "verify": ".bee/bin/bee dev release-manifest --check && rg -l 'brief-file' skills/bee-shaping/SKILL.md skills/bee-planning/SKILL.md skills/bee-capturing/SKILL.md docs/knowledge/areas/decision-memory/owner-brief-contract.md && cmp docs/briefs/owner-brief.md docs/history/owner-brief/brief-draft.md"
  }
]
```

## Test matrix

- Happy: the vi draft validates; `pbi add --brief-file` stores it; `briefs` lists it ok. Pass when exit 0 and `docs/briefs/<id>.md` equals the input.
- Edge: en language; unknown language; exactly 120 and 220 words; an in-flight PBI under an active feature folds; a proposed PBI with the same feature stays its own item. Pass when each assertion in ob-1 step 6 holds.
- Error: each D1/D2 defect refuses by name; a malformed add writes nothing; a missing brief exits 1 in `briefs`. Pass when the refusal text names the defect and `.bee/backlog.jsonl` is unchanged.

## Open Questions

(none)

## Out of scope

- Staleness digest and backfill (D13) — filed as a proposed PBI with its own brief at close.
- Any change to `docs/backlog.md` (D10).
