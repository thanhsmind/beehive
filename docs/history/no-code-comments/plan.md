---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: No Code Comments

Route: class `feature` · lane `standard` · flags `public-contracts`,
`multi-domain` · product files 12.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

## Summary

Nobody can add a comment to a code file in this repository any more.
Two layers say no: the write guard refuses the edit the moment a hooked
agent tries it, and a fence test in the normal test suite goes red when
any file's comment count rises above its committed baseline. The rule
and the two places the "why" now lives land in the doctrine every agent
reads. Existing comments stay for now; removing them is later, batched
work, and each batch lowers the baseline. A host that installs bee gets
the code but not the rule until it opts in through one config key.

Mode: `standard` — 2 risk flags: public-contracts (a new hook refusal
every hooked agent meets, a new dev verb, a new config key),
multi-domain (hook, test suite, dev tools, doctrine, knowledge).
Why this is the least workflow that protects the work: a hook refusal
and a suite test are contracts that need a frozen shape; nothing touches
auth, data or an external system.

## Requirements (from CONTEXT.md)

- D1 (38e323d6): no comment in any code file under the five roots, any
  language; exceptions: shebang, `SAFETY:` on unsafe, license header.
- D2 (80ec3cd1): a ratchet — committed per-file baseline, a suite test
  red on any count above baseline or any comment in a new code file; the
  baseline only goes down, lowered by a dev verb.
- D3 (c2477366): the write guard refuses an Edit/Write/shell write that
  adds a comment line to a code file, naming file, line and remedy.
- D4 (0ee8248d): the why has two homes — a `docs/knowledge` concept or a
  decision log entry; code cites nothing inline.
- D5 (002a935d): the rule lands in `packages/bee/AGENTS.block.md` and
  `packages/bee/prompts/worker-cell.md`.
- D6 (69326d15): cleanup is separate, batched, later.
- D7 (e3bf4a57): one config key `no_code_comments` switches the guard;
  this repo sets it true; hosts opt in; the ratchet is this repo's own.

## Load-bearing claims

Labels: `read` (opened the file at that line, saw those bytes), `ran`
(executed the command, hold its output). Match rule: the evidence column is a
verbatim byte substring of the anchored line(s). Every row is
load-bearing; no `guessed` row.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The guard already branches on the write-capable tool names — the new arm keys off the same names | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:189` | `let targets = extract_bash_targets(&command);` |
| 2 | A content-level arm precedent sits after the path checks (the config guard) — the comment arm slots beside it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:494` | `// crkg-1: config-role-key-guard arm over .bee/config.json and .bee/config.local.json` |
| 3 | Write content is read from `tool_input.content` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:544` | `let Some(content) = tool_input.get("content").and_then(Value::as_str) else {` |
| 4 | Edit content is read from `tool_input.new_string` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:580` | `let Some(new_s) = tool_input.get("new_string").and_then(Value::as_str) else {` |
| 5 | MultiEdit content is read from `tool_input.edits[]` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:618` | `let Some(Value::Array(edits)) = tool_input.get("edits") else {` |
| 6 | The guard already reads `.bee/config.json` in the native run — D7's key is read there | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:126` | `let config = read_config(&store_root_pb)?;` |
| 7 | Heredoc bodies are fenced before tokenizing, so a heredoc into a code path is readable as content | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:459` | `pub(crate) fn fence_heredocs(command: &str) -> String {` |
| 8 | Bash write targets come from one extractor | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:615` | `pub(crate) fn extract_bash_targets(command: &str) -> BashTargets {` |
| 9 | The codex apply_patch body is available as text | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs:18` | `pub(crate) fn apply_patch_text(tool_input: &Map<String, Value>) -> Option<String> {` |
| 10 | A tree-scanning fence test precedent exists and finds the repo root from the crate | read | `packages/bee-rs/crates/bee/tests/specs_fence.rs:50` | `fn repo_root() -> PathBuf {` |
| 11 | End-to-end hook tests build a fixture repo and run the real hook | read | `packages/bee-rs/crates/bee/tests/hook_contracts.rs:116` | `fn run_hook(hook: &str, stdin: &[u8], cwd: &Path) -> Output {` |
| 12 | A deny reaches the host as exit 2 with FIX on stderr — the new refusal is asserted the same way | read | `packages/bee-rs/crates/bee/tests/hook_contracts.rs:445` | `fn a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr() {` |
| 13 | Dev verbs dispatch by name in one match — the new verb lands beside `release-manifest` | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:99` | `"release-manifest" => release_manifest::run(flags),` |
| 14 | `--check` and `--write` are existing flag NAMES (release-manifest), so the new verb adds no flag name and the pin stays 211 | read | `packages/bee-rs/crates/bee/src/catalog.rs:819` | `const PINNED_FLAG_COUNT: usize = 211;` |
| 15 | CI runs the whole release suite, integration targets included — the fence test is in the net | read | `.github/workflows/ci.yml:102` | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml 2>&1 \| tee verify-output.log` |
| 16 | The worker prompt's craft bullet is where the ban is stated for workers | read | `packages/bee/prompts/worker-cell.md:53` | `- Shape what you leave behind: prefer deletion to addition, write the smallest diff that solves it, and leave the base simpler than you found it.` |
| 17 | The doctrine bullet the ban sits beside | read | `packages/bee/AGENTS.block.md:300` | `- Write a mistake down the MOMENT you notice it, never composed from` |
| 18 | The bash scripts in scope carry a shebang, so shebang detection finds extension-less code files | read | `.bee/verify/verify-app/control-bee:1` | `#!/usr/bin/env bash` |
| 19 | The write guard's owning concept exists — the rule is recorded there | read | `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md:3` | `title: Hook Runtime — the request shapes the write guard can read` |
| 20 | The feature map index lists one row per feature file | read | `.bee/verify/verify-app/features/README.md:78` | `- [Cells and the proof line](./cells-and-proof.md) covers adding, claiming and` |
| 21 | Config keys are plain top-level JSON booleans/strings read by name | read | `packages/bee-rs/crates/bee/src/state.rs:203` | `match config.get("gate_bypass") {` |
| 22 | The hook's on/off name is one constant — a deny message cites it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/mod.rs:105` | `const HOOK_NAME: &str = "write-guard";` |
| 23 | Comment lines today, bee crate | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*//' packages/bee-rs/crates/bee/src \| awk -F: '{s+=$2} END{print s+0}'` | `46060` |
| 24 | Comment lines today, fleet crate | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*//' packages/bee-rs/crates/fleet/src \| awk -F: '{s+=$2} END{print s+0}'` | `1239` |
| 25 | Block comments today: none, so the ratchet's block-comment tracking starts at zero | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*/\*' packages/bee-rs/crates/bee/src \| awk -F: '{s+=$2} END{print s+0}'` | `0` |
| 26 | `SAFETY:` lines today, the one Rust exception that exists | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -n "SAFETY:" packages/bee-rs/crates/bee/src \| wc -l` | `22` |
| 27 | The prompt and the block render from disk — regen with the old vendored binary renders the new sources (mistake-fix-at plan claims 40–42) | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:811` | `let source = read_text_if_exists(&engine.templates_prompts_dir.join(name));` |
| 28 | The dispatcher refuses on prompt skew until the binary is reinstalled — the docs cell runs last | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3545` | `if let Some(skew) = prompt_skew(check_root, prompt_name) {` |
| 29 | The manifest file a `packages/bee`- or `skills`-touching cell must list | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:94` | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |

## Discovery

Inspected the write guard's native run (claims 1–9), the fence and hook
test precedents (10–12), the dev verb dispatch and flag pin (13–14), CI
(15), the doctrine homes (16–17), a script shebang (18), the owning
concept and the feature map (19–20), the config precedent (21–22), and
counted today's comments (23–26). Finding: everything the feature needs
has a precedent in the tree; nothing is new architecture. The one
design choice with no precedent is the classifier's home — one module
both the hook and the test read (D1's path list declared once).

## Approach

Recommended path — data shape first: one module,
`packages/bee-rs/crates/bee/src/comments.rs`, owns the D1 path list,
the code-file predicate (extension `.rs`, `.sh`, `.bash`, `.py`, or an
extension-less file whose first line is a `#!` shebang naming sh, bash or
python), the per-language comment-line predicate with the three
exceptions (a `#!` first line; a Rust line whose comment text starts with
`SAFETY:`; a comment block that is the file's first lines and contains
`Copyright`, `SPDX-License-Identifier` or `Licensed under`), a count of
comment lines per file, and the added-comment-lines diff between an
old and a new text (new minus old, by line content, so a moved or
deleted comment never refuses). The ratchet (D2) is an integration test
`tests/comment_fence.rs` that walks the roots, counts with that module,
and compares against `.bee/comment-baseline.json` (`{"files": {path:
count}}`, sorted); a file absent from the baseline allows zero. The dev
verb `bee dev comment-baseline --write|--check` seeds the baseline the
first time, refuses to raise any count afterwards (naming the files),
and lowers counts that fell; `--check` prints what the test would say.
The hook arm (D3, D7) runs only when `no_code_comments` is true in the
merged config, only for a target under a D1 root that the predicate
calls a code file, and refuses with one message naming the file, the
first offending line, and the two homes for the why. Doctrine (D5) and
the knowledge concept (D4) land last with one regen.

Rejected alternatives:
- Fold the ratchet into `bee dev regen` — rejected: regen must stay a
  render chain; a step that can go red on content would block every
  regen for an unrelated reason.
- A clippy lint — rejected: clippy cannot see shell or python, and the
  ratchet must count doc comments, which no lint forbids.
- Hook always on, in hosts too — rejected per D7.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Comment predicate | MEDIUM — a `//` at line start inside a string literal counts as a comment; a baseline seeded by the same predicate stays consistent, and a later false red is fixed by rewriting the string (CONTEXT constraint) | ncc-1 | unit tests per language and per exception; the seeded baseline equals today's counts (claims 23–26) |
| Baseline drift between worktree and main | LOW — the baseline is tracked; a merge that lowers it is fine, one that raises it reds CI | ncc-1 | `--write` refuses to raise; test asserts the refusal |
| Hook false refusals | MEDIUM — an Edit that moves a comment must pass; a Bash command whose heredoc into a `.rs` path carries a comment must refuse; a `.md` or `.json` target never refuses | ncc-2 | tests: move passes, delete passes, add refuses, non-code passes, config off passes |
| Hosts | LOW — key defaults false | ncc-2 | fixture repo without the key → allow |
| Vendored binary lag | MEDIUM — the hook change is inert until `.bee/bin/bee` is reinstalled on main | post-merge | rebuild, reinstall, regen, drive an Edit that adds a comment and read the refusal (`green:live`) |
| Prompt skew | LOW — the prompt edit lands in the last cell | ncc-3 | skew test target green after regen |

Waves: ncc-1 alone (the shared module every other cell reads) → ncc-2
alone (the hook arm reads the module; hook tests) → ncc-3 (doctrine,
knowledge, feature map, config flip: one regen, last because of the
prompt skew). After merge, on main: rebuild, reinstall the vendored
binary, regen, drive the refusal once (`green:live`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust changes in comments.rs, the write guard, devtools, the fence test"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "red-first tests ride each code cell"},
    {"stage": "doctrine", "classification": "required", "role": "docs", "reason": "D5 homes, the knowledge concept, the feature map, the config flip"},
    {"stage": "lookup", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of a file outside its cell", "reason": "read-only gathers"},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "a narrow fact lookup during execution", "reason": "cheap reader"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "no free-form generation stage"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the slice judge over behavior_change cells, or the user invokes bee-reviewing", "reason": "verification of capped cells; independent review stays user-invoked"},
    {"stage": "advisor", "classification": "required", "role": "advisor", "reason": "the plan-step hat wave synthesis is recorded as the advisor ref"},
    {"stage": "plan-hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "the leader plans in-session"},
    {"stage": "supervise", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes: precedent for every part"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"}
  ]
}
```

## Shape

Milestone-shaped; one slice, because a hook refusal with no doctrine
naming the rule teaches by refusal alone, and a doctrine line with no
enforcement is the prose tier the user chose to leave.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — classifier, baseline, fence | `comments.rs`; `.bee/comment-baseline.json` seeded; `tests/comment_fence.rs`; `bee dev comment-baseline` | everything else reads the classifier | `bee dev comment-baseline --check` prints every file at or below baseline; add one `//` line to any `.rs` and the fence test names it | ncc-2 |
| 2 — the hook arm | write guard refuses added comment lines under a code path when `no_code_comments` is true | the classifier exists | an Edit whose `new_string` carries `// note` into a `.rs` file is refused with FIX naming the two homes; the same Edit into a `.md` passes | ncc-3 |
| 3 — doctrine, knowledge, map, config | block + prompt bullets; hook-runtime concept; feature file; `.bee/config.json` `no_code_comments: true`; one regen | the refusal exists to describe | AGENTS.md's block states the ban; the feature map drives the refusal and the fence | merge, then reinstall |

## Smaller path check

*Is there a cheaper shape that still honours every locked decision?*

- Ratchet only, no hook — FAIL: D3 is locked.
- Hook only, no ratchet — FAIL: D2 is locked, and herding CLI workers
  bypass hooks.
- One cell — FAIL: the hook arm and the fence share the classifier but
  their proofs are disjoint (hook fixture tests vs a tree scan), and one
  cell would carry eight files and two test targets.
- Skip the dev verb; hand-edit the baseline — FAIL: D2 says a verb lowers
  it and refuses to raise it; a hand edit can raise it.
- Fold the config flip into ncc-2 — ADOPTED the opposite: the flip rides
  the last cell, so the hook stays off in this repo until the doctrine
  that explains the refusal is in place.

## Hat wave

Three seats, plan-step, one feature. Findings and what changed are
recorded below after synthesis.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ncc-1 | Count comments one way and fence the count | `src/comments.rs` (new), `src/main.rs` (mod line), `tests/comment_fence.rs` (new), `src/devtools/comment_baseline.rs` (new), `src/devtools/mod.rs`, `generated/registry_payload.json`, `.bee/comment-baseline.json` (new) | — | `bee dev comment-baseline --write` seeds a baseline whose totals equal today's counts; `--check` and the fence test go red naming any file above baseline; `--write` refuses to raise a count | `cargo test … comments`, `… --test comment_fence`, `… --test registry_contracts --test registry_dispatch` green, red-first |
| ncc-2 | Refuse a write that adds a comment line to a code file | `src/hooks/write_guard/main.rs`, `src/hooks/write_guard/tests.rs`, `tests/hook_contracts.rs` | ncc-1 | with `no_code_comments: true`, an Edit/Write/MultiEdit/Bash heredoc/apply_patch that adds a comment line under a code path is refused naming file, line and the two homes; moves, deletes, non-code paths and repos without the key pass | `cargo test … write_guard`, `… --test hook_contracts` green, red-first |
| ncc-3 | State the ban where agents read, record it in the knowledge layer, map it, and switch it on | `packages/bee/AGENTS.block.md`, `packages/bee/prompts/worker-cell.md`, `AGENTS.md` (rendered), `.bee/bin/prompts/worker-cell.md` (rendered), `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md`, `.bee/verify/verify-app/features/comment-guard.md` (new), `.bee/verify/verify-app/features/README.md`, `.bee/config.json`, `.bee/config-sample.json`, release manifest | ncc-2 | AGENTS.md's block and the worker prompt state the ban, the three exceptions and the two homes; the concept records the guard and the ratchet; the feature map drives both; this repo's config turns the guard on | regen, skew test target, block parity, manifest check, `rg -q no_code_comments` green |

```json
[
  {
    "id": "ncc-1",
    "feature": "no-code-comments",
    "title": "Count comments one way and fence the count",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["38e323d6-abc3-4cc7-b533-eb27ef0603ed", "80ec3cd1-e51d-435c-ba50-570afa701c7c"],
    "files": [
      "packages/bee-rs/crates/bee/src/comments.rs",
      "packages/bee-rs/crates/bee/src/main.rs",
      "packages/bee-rs/crates/bee/tests/comment_fence.rs",
      "packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      ".bee/comment-baseline.json"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/specs_fence.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs",
      "packages/bee-rs/crates/bee/src/devtools/release_manifest.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"],
    "action": "Build the one classifier and the ratchet (per D1, D2). NEW MODULE packages/bee-rs/crates/bee/src/comments.rs, declared in src/main.rs beside the other top-level modules: `pub(crate) const CODE_ROOTS: [&str; 5]` = packages/bee-rs/crates/bee/src, packages/bee-rs/crates/fleet/src, packages/bee/hooks, packages/bee/lib, scripts — plus `.bee/verify` (six entries; pick the array size to match); `pub(crate) enum Lang { Rust, Shell, Python }`; `pub(crate) fn code_lang(rel: &str, first_line: &str) -> Option<Lang>` by extension (.rs → Rust; .sh/.bash → Shell; .py → Python) or, for an extension-less file, by a `#!` first line naming sh, bash or python; `pub(crate) fn is_comment_line(lang: Lang, line: &str, in_block: &mut bool) -> bool` — Rust: trimmed line starts with `//` (this covers `///` and `//!`), or `/*` opens a block that runs until `*/` (every line inside counts), Shell/Python: trimmed line starts with `#`; `pub(crate) fn is_exception(lang: Lang, line_no: usize, line: &str, at_file_top: bool) -> bool` — a `#!` line when line_no == 1; a Rust comment whose text after the marker starts with `SAFETY:`; a comment inside the file's leading comment block when that block contains `Copyright`, `SPDX-License-Identifier` or `Licensed under`; `pub(crate) fn count_comment_lines(lang: Lang, text: &str) -> usize` (comment lines minus exceptions); `pub(crate) fn added_comment_lines(lang: Lang, old: &str, new: &str) -> Vec<(usize, String)>` = the comment lines (trimmed content, exceptions removed) present in new and not in old, as a multiset difference, each with its 1-based line number in new — so a moved or deleted comment yields nothing; `pub(crate) fn walk_code_files(root: &Path) -> Vec<(String, Lang)>` over CODE_ROOTS, skipping directories named target, node_modules or .git, returning repo-relative paths sorted. BASELINE: `.bee/comment-baseline.json`, shape `{\"files\": {\"<rel>\": <count>, ...}}` with sorted keys, written and read by ONE pair of functions in the new devtools/comment_baseline.rs (`load_baseline`, `write_baseline`). DEV VERB in packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs, dispatched from devtools/mod.rs beside `\"release-manifest\" => release_manifest::run(flags),` as `\"comment-baseline\"`: `--check` walks the files, counts, and prints one line per file whose count exceeds its baseline entry (or any count > 0 for a file absent from the baseline), exits 1 when any, else prints `comment-baseline --check: <n> file(s) at or below baseline` and exits 0; `--write` with no baseline present seeds it from today's counts and says so; `--write` with a baseline present lowers any entry whose count fell, removes entries for deleted files, and REFUSES (exit 1, nothing written) naming every file whose count would rise — the baseline never goes up. Register the verb in generated/registry_payload.json (hand-edited, decision 3358743e) as dev.comment-baseline with flags check and write reusing the existing names, so catalog.rs's PINNED_FLAG_COUNT stays 211 — confirm by running the catalog test. FENCE TEST packages/bee-rs/crates/bee/tests/comment_fence.rs, modeled on tests/specs_fence.rs (`fn repo_root()` from CARGO_MANIFEST_DIR): loads the baseline, walks the roots with comments::walk_code_files, and asserts every file's count <= its baseline entry (absent → 0), failing with a message that names each offending file, its count and its baseline and says `bee dev comment-baseline --write` lowers, never raises. Seed the baseline in this cell by running the verb once and commit it. TESTS, red-first, in comments.rs's test module: each language's comment detection; `///` and `//!` count; a block comment counts every inner line; a `#!` first line does not count; a `SAFETY:` line does not count; a leading license block does not count; a `//` inside a string literal DOES count (the documented limitation, asserted so nobody 'fixes' it into an allowlist); added_comment_lines returns nothing for a move and a delete and the right line for an add; code_lang finds an extension-less bash file by shebang. In comment_baseline.rs's tests: seed, lower, refuse-to-raise. Proof also runs the registry targets.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee comments && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee comment_baseline && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test comment_fence --test registry_contracts --test registry_dispatch && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee catalog",
    "must_haves": {
      "truths": [
        "comments.rs is the one home of the code roots, the code-file predicate, the comment-line predicate with the three exceptions, the per-file count and the added-lines diff",
        ".bee/comment-baseline.json exists, sorted, and its bee-crate total equals 46060 minus the 22 SAFETY lines and the fleet total equals 1239 (or the cell's report names the exact counted totals and why they differ)",
        "bee dev comment-baseline --check exits 0 on the seeded tree and exits 1 naming the file when one comment line is added to any code file",
        "bee dev comment-baseline --write refuses to raise any count and lowers a count that fell",
        "tests/comment_fence.rs is red when a code file exceeds its baseline and names the file, count and baseline",
        "PINNED_FLAG_COUNT stays 211 and both registry test targets are green with dev.comment-baseline registered"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/comments.rs", "substantive": "CODE_ROOTS, Lang, code_lang, is_comment_line, is_exception, count_comment_lines, added_comment_lines, walk_code_files, tests"},
        {"path": "packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs", "substantive": "load/write baseline, --check, --write seed/lower/refuse, tests"},
        {"path": "packages/bee-rs/crates/bee/tests/comment_fence.rs", "substantive": "the ratchet over the tree"},
        {"path": ".bee/comment-baseline.json", "substantive": "the seeded per-file counts"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "dev.comment-baseline entry"}
      ],
      "key_links": [
        "tests/comment_fence.rs and devtools/comment_baseline.rs both call comments::walk_code_files and comments::count_comment_lines — no second counter",
        "devtools/mod.rs dispatches comment-baseline"
      ],
      "prohibitions": [
        "No comment lines are removed from any existing file in this cell (D6)",
        "The baseline is never hand-edited; the verb writes it",
        "No allowlist for string literals that look like comments",
        "The new module and test carry no comments of their own"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "ncc-2",
    "feature": "no-code-comments",
    "title": "Refuse a write that adds a comment line to a code file",
    "lane": "standard",
    "role": "code",
    "deps": ["ncc-1"],
    "decisions": ["c2477366-4b6c-4eca-a3c7-37b2062f12da", "e3bf4a57-6779-4ca2-895d-f03f9b720c1d"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/tests/hook_contracts.rs"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/comments.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", ".bee/verify/verify-app/features/comment-guard.md"],
    "action": "Add the comment arm to the write guard (per D3, D7). In packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs, after the config-role-key-guard arm (rg hit: `        // crkg-1: config-role-key-guard arm over .bee/config.json and .bee/config.local.json`) and only while `denial.is_none()`: read the merged config the guard already reads (rg hit: `let config = read_config(&store_root_pb)?;` — reuse that read or read once here) and skip the whole arm unless `config.get(\"no_code_comments\") == Some(&Value::Bool(true))`. For each resolved target rel path under crate::comments::CODE_ROOTS whose `comments::code_lang(rel, first_line_of_current_file_or_new_content)` is Some: gather the NEW text and the OLD text per tool — Write: new = `tool_input.content` (rg hit: `let Some(content) = tool_input.get(\"content\").and_then(Value::as_str) else {`), old = the file on disk or empty; Edit: new = `new_string`, old = `old_string` (rg hit: `let Some(new_s) = tool_input.get(\"new_string\").and_then(Value::as_str) else {`); MultiEdit: per edit in `edits` (rg hit: `let Some(Value::Array(edits)) = tool_input.get(\"edits\") else {`); Bash/exec: for every redirect or heredoc target the extractor already resolves (rg hits: `pub(crate) fn extract_bash_targets(command: &str) -> BashTargets {`, `pub(crate) fn fence_heredocs(command: &str) -> String {`), new = the heredoc body or the here-string/echo text when the guard can see it, old = the file on disk — when the guard cannot see the content it writes, it does not refuse on this arm (the ratchet catches it); apply_patch: new/old from the patch hunks (rg hit: `pub(crate) fn apply_patch_text(tool_input: &Map<String, Value>) -> Option<String> {`) — added lines only. Compute `comments::added_comment_lines(lang, old, new)`; when non-empty, `denial = Some(...)` with ONE message: `bee comment guard denied this write: <rel>:<line> adds a comment line (\"<trimmed line, max 80 chars>\"). This repository keeps no comments in code (no_code_comments). FIX: put the why in a docs/knowledge concept whose Pointers name this file, or log it with bee decisions log; a workaround is fixed, or filed with bee backlog add and the row cited from the knowledge concept — never from the code. Exceptions: a #! first line, a SAFETY: line on unsafe, a license header at file top.` Never refuse a target that is not a code file (.md, .json, .toml, docs, .bee state). TESTS, red-first, in write_guard/tests.rs beside the config-guard tests (use the fixture that writes .bee/config.json with `no_code_comments: true`): Edit adding `// note` into packages/bee-rs/crates/bee/src/x.rs → deny with FIX and the file:line; the same Edit with the key absent or false → allow; Edit whose old_string and new_string carry the same comment moved → allow; Edit that deletes a comment → allow; Write of a new .rs file with a `///` doc line → deny; Write of a .md with `// x` → allow; Bash heredoc `cat > scripts/x.sh <<'EOF'` with a `# note` body line → deny, with the shebang first line → allow; a `SAFETY:` line → allow; MultiEdit with one offending edit → deny naming it; apply_patch adding a `//` line under the crate → deny. In tests/hook_contracts.rs beside `fn a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr() {`: one end-to-end test that the comment deny reaches the host as exit 2 with FIX on stderr.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee write_guard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts",
    "must_haves": {
      "truths": [
        "with no_code_comments true, an Edit, Write, MultiEdit, Bash heredoc or apply_patch that adds a comment line to a file under a code root is refused with a message naming the file, the line, the trimmed comment, the config key, the two homes for the why and the three exceptions",
        "a moved or deleted comment, a non-code target, a shebang, a SAFETY line, a license header, and a repository without the key all pass this arm",
        "a Bash write whose content the guard cannot see is not refused by this arm",
        "the deny reaches the host as exit 2 with FIX on stderr",
        "every other write-guard test stays green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs", "substantive": "the comment arm reading comments.rs, gated by no_code_comments"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "the arm's tests across the five tool shapes and the pass cases"},
        {"path": "packages/bee-rs/crates/bee/tests/hook_contracts.rs", "substantive": "one end-to-end deny test"}
      ],
      "key_links": [
        "the arm calls comments::code_lang and comments::added_comment_lines — no second predicate",
        "the arm sits after the config guard and only while denial is none"
      ],
      "prohibitions": [
        "No change to any existing refusal's text or order",
        "No refusal on .md, .json, .toml, docs or .bee state paths",
        "No comment lines added to main.rs or tests.rs by this cell (the fence would red)"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "ncc-3",
    "feature": "no-code-comments",
    "title": "State the ban where agents read, record it in the knowledge layer, map it, and switch it on",
    "lane": "standard",
    "role": "docs",
    "deps": ["ncc-1", "ncc-2"],
    "decisions": ["002a935d-0da1-4ab1-acea-0c208806f31a", "0ee8248d-a4c5-4a7d-8d84-c0184b0c70f5", "e3bf4a57-6779-4ca2-895d-f03f9b720c1d"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "packages/bee/prompts/worker-cell.md",
      "AGENTS.md",
      ".bee/bin/prompts/worker-cell.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      ".bee/verify/verify-app/features/comment-guard.md",
      ".bee/verify/verify-app/features/README.md",
      ".bee/config.json",
      ".bee/config-sample.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee/AGENTS.block.md",
      "packages/bee/prompts/worker-cell.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      ".bee/verify/verify-app/features/cells-and-proof.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", ".bee/verify/verify-app/features/comment-guard.md"],
    "action": "Land the rule in the homes agents read (per D5, D4), record it in the knowledge layer, map it, and switch it on (per D7), with ONE regen; never hand-edit AGENTS.md (it renders from the block). (1) packages/bee/AGENTS.block.md: add one bullet directly BEFORE the bullet `- Write a mistake down the MOMENT you notice it` (rg hit: `- Write a mistake down the MOMENT you notice it, never composed from`): no comment in code, any language — not `//`, `///`, `//!`, `/* */`, not `#`; the why lives in a docs/knowledge concept whose Pointers name the file, or in bee decisions log; a workaround is fixed or filed with bee backlog add; the three exceptions (a #! first line, a SAFETY: line on unsafe, a license header at file top); the write guard refuses the write where no_code_comments is on and the comment fence reds the suite. (2) packages/bee/prompts/worker-cell.md: extend the craft bullet (rg hit: `- Shape what you leave behind: prefer deletion to addition, write the smallest diff that solves it, and leave the base simpler than you found it.`) with one sentence: no comments in code, the why goes to docs/knowledge or bee decisions log, the guard refuses the rest. (3) docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md: add a section for the comment arm — the config key, the code roots (name comments.rs as the one home in Pointers), what is a comment line per language, the three exceptions, the refusal shape, the documented string-literal limitation, and the ratchet (baseline file, the verb's write/check contract, the fence test) — citing decisions 38e323d6, 80ec3cd1, c2477366, 0ee8248d, 002a935d, 69326d15, e3bf4a57 in the frontmatter decisions list; state the D6 cleanup as not yet done. (4) .bee/verify/verify-app/features/comment-guard.md, modeled on cells-and-proof.md: what it is, how a user reaches it (an Edit that adds `// x` to a .rs under a code root with the key on; `bee dev comment-baseline --check`), how to drive it with control-bee (set no_code_comments true in the sandbox config, run the hook with a crafted payload via `control-bee cli -- hook write-guard` or the fixture path the hook tests use, read the refusal; run the verb), gotchas (the string-literal count, hosts default off, the baseline never rises); add its row to README.md (rg hit: `- [Cells and the proof line](./cells-and-proof.md) covers adding, claiming and`). (5) .bee/config.json: add `\"no_code_comments\": true` beside `uat_stop`; .bee/config-sample.json: add the key with false and no other change. (6) Run `.bee/bin/bee dev regen` ONCE; commit every listed file plus every rendered copy regen touched (skill mirrors of the feature map, .bee-render.json files, .bee/onboarding.json) — stage by explicit path list and check `git status` for packages/bee-rs/crates/bee/.bee/logs before the commit, never stage that log. Do NOT reinstall .bee/bin/bee. PROOF: regen green, the skew test target `verbs::drivers::tests` green, `--test agents_block_render_parity` green, `.bee/bin/bee dev release-manifest --check` green, `rg -q no_code_comments` on the block, the prompt, the concept, the feature file and .bee/config.json; read back the rendered AGENTS.md bullet and quote it (`green:live`).",
    "verify": ".bee/bin/bee dev regen && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers::tests && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test agents_block_render_parity && .bee/bin/bee dev release-manifest --check && rg -q no_code_comments packages/bee/AGENTS.block.md AGENTS.md packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md .bee/verify/verify-app/features/comment-guard.md .bee/config.json",
    "must_haves": {
      "truths": [
        "AGENTS.block.md carries the no-comment bullet with the two homes and the three exceptions, and AGENTS.md's rendered block matches (parity test green)",
        "worker-cell.md's craft bullet states the ban and the two homes; the vendored copy matches (skew test green)",
        "write-guard-request-shapes.md records the comment arm, the ratchet and the seven decision ids",
        "comment-guard.md exists and README.md lists it",
        ".bee/config.json sets no_code_comments true; config-sample.json shows the key false",
        "release-manifest --check is green after one regen"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "the no-comment bullet"},
        {"path": "AGENTS.md", "substantive": "the rendered block — written by regen"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "the craft bullet's no-comment sentence"},
        {"path": "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", "substantive": "the comment arm and ratchet section with decision ids"},
        {"path": ".bee/verify/verify-app/features/comment-guard.md", "substantive": "the feature file"},
        {"path": ".bee/config.json", "substantive": "no_code_comments: true"}
      ],
      "key_links": [
        "AGENTS.md's block equals the rendered template",
        "the concept's Pointers name packages/bee-rs/crates/bee/src/comments.rs and tests/comment_fence.rs",
        "release-manifest.json is refreshed in the same commit"
      ],
      "prohibitions": [
        "No hand edit of AGENTS.md or any mirrored copy",
        "No Rust edits; no reinstall of .bee/bin/bee",
        "No other config key changes",
        "packages/bee-rs/crates/bee/.bee/logs/timings.jsonl is never staged"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

| Cell | Happy path | Edge | Error | Pass when |
|---|---|---|---|---|
| ncc-1 | seed baseline; `--check` green on the tree | shebang, SAFETY, license, block comment, string-literal `//`, extension-less bash file | one added `//` line reds `--check` and the fence; `--write` refuses to raise | totals reported equal the census (claims 23–26 less the 22 SAFETY lines) or the difference is named |
| ncc-2 | Edit adding `// note` refused with FIX | move and delete pass; `.md` passes; key absent passes; heredoc with only a shebang passes | MultiEdit with one bad edit refused naming it; apply_patch refused | exit 2 + FIX on stderr end to end |
| ncc-3 | regen renders both homes; parity and skew tests green | — | — | `rg -q no_code_comments` over seven files exits 0; manifest check ok |
| existing behavior | full write-guard suite and hook_contracts on head | — | — | every pre-existing test passes unchanged |

## Test scoping

Each code cell's proof is the filtered release run its `verify` names; the
full declared `commands.test` runs once before merge and CI runs it on the
push. The fence test is part of that suite from ncc-1 on.

## Open Questions

- (none blocking) Whether `.bee/verify` scripts other than `control-bee`
  exist without an extension — the walker's shebang rule covers them either
  way.

## Out of scope

- Removing any existing comment (D6) — batched grooming later; each batch
  lowers the baseline through the verb.
- A clippy or rustfmt rule — the classifier covers three languages and doc
  comments, which no lint does.
- Turning the guard on in hosts (D7) — a host flips its own key.
