---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: No Code Comments

Route: class `feature` · lane `standard` · flags `public-contracts`,
`multi-domain` · product files 14.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

Revision 2 — rewritten after the plan-step hat wave. What the wave
changed is listed under `## Hat wave` below.

## Summary

Nobody can add a comment to a code file in this repository any more.
Two layers say no: the write guard refuses the edit the moment a hooked
agent tries it, and a fence test in the normal test suite goes red when
any file's comment count rises above its committed baseline. The rule
and the two places the "why" now lives land in the doctrine every agent
reads, as a spoken rule with an id. Existing comments stay for now;
removing them is later, batched work, and each batch lowers the
baseline. The baseline is seeded from main and every branch still in
flight, so a sibling feature that merges next week cannot turn CI red. A
host that installs bee gets the code but not the rule until it opts in
through one config key.

Mode: `standard` — 2 risk flags: public-contracts (a new hook refusal
every hooked agent meets, a new dev verb, a new config key, a new rule
id), multi-domain (hook, test suite, dev tools, doctrine, knowledge).
Why this is the least workflow that protects the work: a hook refusal
and a suite test are contracts that need a frozen shape; nothing touches
auth, data or an external system.

## Requirements (from CONTEXT.md)

- D1 (38e323d6): no comment in any code file under the five roots, any
  language; exceptions: shebang, `SAFETY:` on unsafe, license header.
- D2 (80ec3cd1): a ratchet — committed per-file baseline, a suite test
  red on any count above baseline or any comment in a new code file; the
  baseline only goes down, lowered by a dev verb.
- D2b (0add770d, touches D2): the seed is the per-file maximum over main
  and every unmerged `wt/*` branch; `--write` never raises afterwards.
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
verbatim byte substring of the anchored line. Every row is
load-bearing; no `guessed` row.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The guard already branches on the write-capable tool names — the new arm keys off the same names | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:189` | `let targets = extract_bash_targets(&command);` |
| 2 | A content-level arm precedent sits after the path checks (the config guard) — the comment arm slots beside it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:494` | `// crkg-1: config-role-key-guard arm over .bee/config.json and .bee/config.local.json` |
| 3 | Write content is read from `tool_input.content` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:544` | `let Some(content) = tool_input.get("content").and_then(Value::as_str) else {` |
| 4 | The config arm reconstructs the WHOLE proposed file for an Edit from disk plus `replacen` — the comment arm shares that reconstruction so line numbers, block state, shebang and license rules see the full file | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:566` | `let Ok(current_text) = std::fs::read_to_string(root_pb.join(rel)) else {` |
| 5 | The reconstruction step | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:598` | `current_text.replacen(old_s, new_s, 1)` |
| 6 | MultiEdit content is read from `tool_input.edits[]` and folded the same way | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:618` | `let Some(Value::Array(edits)) = tool_input.get("edits") else {` |
| 7 | The config arm hard-denies apply_patch rather than reading hunks — the comment arm reads added `+` lines itself | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:501` | `"bee config guard: \"{}\" cannot be modified via apply_patch — this target requires content-level inspection. FIX: use Edit/Write to edit config files.",` |
| 8 | A broad write sets a `**` sentinel path the arm must skip | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:359` | `rel_paths = vec!["**".to_string()];` |
| 9 | The guard's config read is the merged read (local overlay over config.json) — the arm calls it once in the write branch | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/store.rs:86` | `pub(crate) fn read_config(root: &Path) -> R<Map<String, Value>> {` |
| 10 | Heredoc bodies are REMOVED before tokenizing — the Bash arm must extract bodies itself, from the raw command | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:459` | `pub(crate) fn fence_heredocs(command: &str) -> String {` |
| 11 | Bash targets carry paths only, no content | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:576` | `pub(crate) struct BashTargets {` |
| 12 | The apply_patch envelope is available as text; no hunk parser exists | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs:18` | `pub(crate) fn apply_patch_text(tool_input: &Map<String, Value>) -> Option<String> {` |
| 13 | apply_patch targets come from the `*** Add File:`/`*** Update File:` headers | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs:30` | `pub(crate) fn extract_apply_patch_targets(patch_text: &str) -> Vec<String> {` |
| 14 | In a granted worktree the guard reads the worktree's own store (its own config), so in-flight branches without the key are never refused | read | `packages/bee-rs/crates/bee/src/hooks/adapter.rs:179` | `let mut store_root = main_root.clone();` |
| 15 | Hook tests build a fixture and write config by hand | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:313` | `std::fs::write(fx.root.join(".bee/config.json"), &cfg_str).unwrap();` |
| 16 | The fixture builder and the assertion helper the arm's tests reuse | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:54` | `fn build_fixture(phase: &str, execution_approved: bool) -> Fx {` |
| 17 | `expect_done` is the assertion door | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs:77` | `fn expect_done(payload: Value, cwd: &Path) -> Emit {` |
| 18 | Crate `bee` has no lib target — an integration test cannot import `comments::*`, so the ratchet is a bin unit test | read | `packages/bee-rs/crates/bee/Cargo.toml:2` | `name = "bee"` |
| 19 | Top-level modules are `mod x;` lines in main.rs | read | `packages/bee-rs/crates/bee/src/main.rs:12` | `mod catalog;` |
| 20 | The repo-root trick a bin unit test copies | read | `packages/bee-rs/crates/bee/tests/specs_fence.rs:51` | `PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()` |
| 21 | A tree walker with the skip list already exists in a test target — copied, attributed, never imported | read | `packages/bee-rs/crates/bee/tests/instruction_laws.rs:125` | `if matches!(name.as_ref(), "target" \| "node_modules" \| ".git") {` |
| 22 | End-to-end hook tests run the real hook | read | `packages/bee-rs/crates/bee/tests/hook_contracts.rs:116` | `fn run_hook(hook: &str, stdin: &[u8], cwd: &Path) -> Output {` |
| 23 | A deny reaches the host as exit 2 with FIX on stderr | read | `packages/bee-rs/crates/bee/tests/hook_contracts.rs:445` | `fn a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr() {` |
| 24 | Dev verbs dispatch by name — the new verb lands beside `release-manifest` | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:99` | `"release-manifest" => release_manifest::run(flags),` |
| 25 | Source-checkout-only verbs are a fixed-size list the new verb joins (5 → 6) | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:64` | `const SOURCE_CHECKOUT_DEV_VERBS: [&str; 5] = [` |
| 26 | A served dev verb with no registry entry reds the dispatch test — the entry is test law | read | `packages/bee-rs/crates/bee/tests/registry_dispatch.rs:432` | `for verb in match_arm_verbs(&crate_src("devtools/mod.rs"), "pub fn try_native", 5) {` |
| 27 | `check`/`write` are existing flag names — the pin stays 211 | read | `packages/bee-rs/crates/bee/src/catalog.rs:819` | `const PINNED_FLAG_COUNT: usize = 211;` |
| 28 | The registry payload is hand-edited here | read | `docs/decisions/index.md:2013` | `3358743e · 2026-08-05 · worktree-reclaim D5: packages/bee-rs/crates/bee/src/generated/registry_payload.json is hand-edited in this repo` |
| 29 | Atomic JSON write the baseline uses | read | `packages/bee-rs/crates/bee/src/fsutil.rs:139` | `pub fn write_json_atomic(file: &Path, value: &Value) -> std::io::Result<()> {` |
| 30 | JSON read the baseline uses | read | `packages/bee-rs/crates/bee/src/fsutil.rs:78` | `pub fn read_json(file: &Path) -> ReadJson {` |
| 31 | serde_json preserves insertion order — sorted keys must be inserted sorted (from a BTreeMap) | read | `packages/bee-rs/Cargo.toml:13` | `serde_json = { version = "1", features = ["preserve_order"] }` |
| 32 | The doc-deferral baseline already solves the sorted-insert question | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1364` | `/// the JSON \`Map\` in that already-sorted order — byte-identical across runs` |
| 33 | CI runs the whole release suite on Linux | read | `.github/workflows/ci.yml:102` | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml 2>&1 \| tee verify-output.log` |
| 34 | And on Windows — baseline keys must be `/`-normalized | read | `.github/workflows/windows.yml:64` | `run: cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |
| 35 | The declared suite is the same command | read | `.bee/config.json:11` | `"test": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml"` |
| 36 | A rule id lives in three places — the new bullet is a marked rule with an index row | read | `packages/bee-rs/crates/bee/tests/rule_index_parity.rs:3` | `// A rule id lives in THREE places: the \`<!-- rule: <id> -->\` markers in` |
| 37 | The rule marker shape in the block | read | `packages/bee/AGENTS.block.md:294` | `<!-- rule: agents-one-commit-per-cell -->` |
| 38 | The rule index section the new row joins | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:181` | `## AGENTS.md rule homes` |
| 39 | The doctrine bullet the ban sits beside | read | `packages/bee/AGENTS.block.md:300` | `- Write a mistake down the MOMENT you notice it, never composed from` |
| 40 | The worker prompt's craft bullet is where the ban is stated for workers | read | `packages/bee/prompts/worker-cell.md:53` | `- Shape what you leave behind: prefer deletion to addition, write the smallest diff that solves it, and leave the base simpler than you found it.` |
| 41 | The one Rust line today that a string literal makes look like a comment | read | `packages/bee-rs/crates/bee/src/hooks/session_close/html.rs:245` | `// expand a project row to show its per-model breakdown` |
| 42 | Shell shebangs also appear inside heredoc bodies — `#!` is an exception on any line, not only line 1 | read | `scripts/codex-parity-canary-test.sh:27` | `#!/usr/bin/env bash` |
| 43 | The write guard's owning concept exists | read | `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md:3` | `title: Hook Runtime — the request shapes the write guard can read` |
| 44 | The feature map index lists one row per feature file | read | `.bee/verify/verify-app/features/README.md:78` | `- [Cells and the proof line](./cells-and-proof.md) covers adding, claiming and` |
| 45 | Config keys are plain top-level JSON values read by name | read | `packages/bee-rs/crates/bee/src/state.rs:203` | `match config.get("gate_bypass") {` |
| 46 | Comment lines today, bee crate | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*//' packages/bee-rs/crates/bee/src \| awk -F: '{s+=$2} END{print s+0}'` | `46060` |
| 47 | Comment lines today, fleet crate | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*//' packages/bee-rs/crates/fleet/src \| awk -F: '{s+=$2} END{print s+0}'` | `1239` |
| 48 | Block comments today: none | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -c '^\s*/\*' packages/bee-rs/crates/bee/src \| awk -F: '{s+=$2} END{print s+0}'` | `0` |
| 49 | Unmerged branches carry added comment lines under the roots — the seed must include them | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && git diff main...wt/statusline-by-default -- packages/bee-rs/crates scripts .bee/verify packages/bee/hooks packages/bee/lib \| rg -c '^\+\s*(//\|#)'` | `175` |
| 50 | The prompt and the block render from disk — regen with the old vendored binary renders the new sources | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:811` | `let source = read_text_if_exists(&engine.templates_prompts_dir.join(name));` |
| 51 | The dispatcher refuses on prompt skew until the binary is reinstalled — the docs cell runs last | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3545` | `if let Some(skew) = prompt_skew(check_root, prompt_name) {` |
| 52 | The manifest file a `packages/bee`-touching cell must list | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:94` | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |

## Discovery

Inspected the write guard's native run and its config arm (claims 1–9),
the Bash and apply_patch readers (10–13), the store resolution in a
worktree (14), the hook test fixtures (15–17, 22–23), the crate shape
(18–21), the dev verb dispatch and its test law (24–28), the JSON helpers
and the order trap (29–32), CI on both platforms and the declared suite
(33–35), the rule-id convention (36–38), the doctrine homes (39–40), the
two known false-comment shapes (41–42), the concept and the map (43–44),
config (45), today's counts (46–48) and the in-flight branches (49).
Finding: everything has a precedent except three small readers the hook
arm needs (a heredoc-body reader, an apply_patch added-lines reader, and
a shared full-file reconstruction helper) — all named in ncc-3.

## Approach

Recommended path — data shape first: one module,
`packages/bee-rs/crates/bee/src/comments.rs`, owns the code-root
predicate (`packages/bee-rs/crates/<any>/src/**`, `packages/bee/hooks/**`,
`packages/bee/lib/**`, `scripts/**`, `.bee/verify/**`), the code-file
predicate (extension `.rs`, `.sh`, `.bash`, `.py`, or an extension-less
file whose first line is a `#!` shebang naming sh, bash or python), the
per-language comment-line predicate with the three exceptions (any line
starting `#!`; a Rust comment whose text starts with `SAFETY:`; a
leading comment block containing `Copyright`, `SPDX-License-Identifier`
or `Licensed under`), the per-file count, the added-lines diff (new
minus old as a multiset by trimmed content, so a move or delete never
refuses), and the tree walker (copied from `instruction_laws.rs`, `/`
-normalized keys). The ratchet (D2, D2b) lives in
`devtools/comment_baseline.rs`: the verb `bee dev comment-baseline
--write|--check` (source-checkout only), the baseline
`.bee/comment-baseline.json` `{"files": {path: count}}` built from a
BTreeMap and written with `fsutil::write_json_atomic`, and a bin unit
test that walks the tree and asserts every count ≤ baseline, printing
per file `count, baseline` and the two-sentence remedy (`--write` lowers,
never raises; a rise from a merge is fixed by deleting the merged
comment lines). The first `--write` seeds from the per-file maximum over
main and every unmerged `wt/*` branch (`git branch --no-merged main`,
each file read with `git show <branch>:<path>`); later `--write`s lower
and refuse to raise. The hook arm (D3, D7): a helper
`reconstruct_target_text` extracted from the config arm gives
`(old, new)` whole-file text for Write/Edit/MultiEdit; a
`heredoc_writes` reader in `guards.rs` pairs a heredoc body with a
`>`/`>>` target in the same simple command; an
`apply_patch_added_lines` reader in `detectors.rs` returns `+` lines per
`Add File`/`Update File` target; the arm runs only with
`no_code_comments` true, skips the `**` sentinel and every non-code
target, and refuses with one message naming file, line, the trimmed
line, that `///` and `//!` are comments too, the two homes for a why,
the home for a public API's description (the owning knowledge concept),
and the three exceptions. Doctrine (D5) lands as a marked rule
`agents-no-code-comments` with its index row; the concept records the
arm and the ratchet; the feature map drives both; the config flips on
last.

Rejected alternatives:
- `tests/comment_fence.rs` as an integration target — rejected: no lib
  target to import (claim 18).
- Fold the ratchet into `bee dev regen` — rejected: regen is a render
  chain; a content red would block unrelated regens.
- A clippy lint — rejected: three languages and doc comments.
- Seed from main alone — rejected per D2b (claim 49).
- A `--adopt-merge` escape that raises — rejected: contradicts D2.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Comment predicate | MEDIUM — a `//` at line start inside a string literal counts (one such line today, claim 41); a heredoc-body shebang is exempt (claim 42) | ncc-1 | unit tests per language and exception; the string-literal case asserted as counted |
| In-flight branches | HIGH if seeded from main — 5 branches, ~256 added lines | ncc-2 (D2b seed) | after seeding, `--check` against each unmerged branch's tree (`git worktree`-free: `git show` per file) reports 0 files above |
| Windows | MEDIUM — keys must be `/` | ncc-2 | the walker normalizes; a test asserts a key contains no `\` |
| Hook false refusals | MEDIUM — move/delete must pass; `.md`/`.json` never refused; Bash without a readable body never refused; fixtures in the arm's own tests must stay inline `json!` so the fence never counts them | ncc-3 | tests for each; ncc-3's verify includes the ratchet |
| Hosts and ungranted worktrees | LOW — key defaults false; an ungranted worktree reads main's config (claim 14) — named in the concept | ncc-3, ncc-4 | fixture without the key → allow |
| Doc-deferral door at close | LOW — the concept's "not yet done" sentence is fenced | ncc-4 | `bee close --dry-run` shows no doc-deferral block |
| Vendored binary lag | MEDIUM — the hook change is inert until `.bee/bin/bee` is reinstalled on main | post-merge | rebuild, reinstall, regen, drive an Edit that adds `/// x` and read the refusal (`green:live`) |
| Prompt skew | LOW — the prompt edit lands in the last cell | ncc-4 | skew test target green after regen |

Waves: ncc-1 alone (the predicate every other cell reads) → ncc-2 ∥
ncc-3 (devtools/registry/baseline vs hooks — disjoint files) → ncc-4
(doctrine, rule index, concept, map, config flip; one regen; last
because of the skew). After merge, on main: rebuild, reinstall the
vendored binary, regen, drive the refusal once (`green:live`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust changes in comments.rs, devtools, the write guard"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "red-first tests ride each code cell"},
    {"stage": "doctrine", "classification": "required", "role": "docs", "reason": "D5 homes, the rule index, the knowledge concept, the feature map, the config flip"},
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
| 1 — the predicate | `comments.rs` | everything else reads it | unit tests: `///` counts, `#!` never, `SAFETY:` never, a string-literal `//` counts, a move adds nothing | ncc-2, ncc-3 |
| 2 — the ratchet and the arm | verb + baseline (seeded over main and the in-flight branches) + the ratchet test; the hook arm behind the key | the predicate exists | `bee dev comment-baseline --check` green on the tree; add one `//` line and it names the file; with the key on, an Edit adding `/// x` to a `.rs` is refused with FIX | ncc-4 |
| 3 — doctrine, index, concept, map, config | marked rule in the block + prompt bullet; index row; concept section; feature file; `no_code_comments: true`; one regen | the refusal exists to describe | AGENTS.md's block states the rule by id; parity, skew and pointer tests green | merge, then reinstall |

## Smaller path check

*Is there a cheaper shape that still honours every locked decision?*

- Ratchet only, no hook — FAIL: D3 is locked.
- Hook only, no ratchet — FAIL: D2 is locked, and herding CLI workers
  bypass hooks.
- Skip the dev verb; hand-edit the baseline — FAIL: D2 says a verb
  lowers it and refuses to raise it.
- Three cells (predicate+ratchet, hook, docs) — FAIL: the predicate is
  the one MEDIUM-risk piece and deserves its own narrow proof; the
  ratchet cannot be green before the verb seeds the baseline, so
  predicate and ratchet in one cell would cap on a half-proven module.
  Four cells, with ncc-2 and ncc-3 in parallel, costs no extra wave.
- Shared reconstruction helper instead of re-reading tool_input in the
  arm — ADOPTED: it is smaller and it is the only way the arm sees
  whole-file line numbers, block state and file-top exceptions.

## Hat wave

Three seats, plan-step, one feature. Decision logged (tag
`plan-hat-wave`, `f02e89ab`). What each changed:

| Seat | Finding acted on | Change |
|---|---|---|
| alternatives | crate `bee` has no lib target — `tests/comment_fence.rs` could never call `comments::*` | the ratchet is a bin unit test in `devtools/comment_baseline.rs`; claim 18 added |
| alternatives | the Edit arm took `old_string`/`new_string` fragments — no file line numbers, no block state, no file-top exceptions | `reconstruct_target_text` extracted from the config arm and shared (claims 4–6) |
| alternatives | `SOURCE_CHECKOUT_DEV_VERBS` must grow 5 → 6; `preserve_order` defeats "sorted keys" unless inserted from a BTreeMap; the `**` sentinel; the config read sits in the read branch | named in ncc-2 and ncc-3 actions (claims 8, 9, 25, 31, 32) |
| alternatives | `-p bee comments` also matches an unrelated onboard test; `--bin bee <module>::` is exact | verify strings use `--bin bee comments::`, `--bin bee devtools::comment_baseline::`, `--bin bee hooks::write_guard::` |
| alternatives | the walker and the JSON IO re-implemented `instruction_laws.rs` and `fsutil` | walker copied with attribution; `fsutil::read_json`/`write_json_atomic` used |
| alternatives | ncc-1 carried seven files and four proofs | split: ncc-1 predicate, ncc-2 verb+baseline+ratchet |
| facts-gaps | heredoc bodies are removed by `fence_heredocs`; nothing reads them; `guards.rs` was outside the cell | `heredoc_writes` reader added to ncc-3 with `guards.rs` in files; claim 10 rewritten |
| facts-gaps | apply_patch has no hunk parser | `apply_patch_added_lines` added to ncc-3 with `detectors.rs` in files; claim 7 and 12–13 |
| facts-gaps | the docs cell's edits can red `rule_index_parity`, `pointer_integrity`, `instruction_laws`, none in its verify | the bullet is a marked rule with an index row; all three targets in ncc-4's verify (claims 36–38) |
| facts-gaps | no config fixture exists in the hook tests; `read_config` at :126 is in the read branch | tests reuse `build_fixture` + the plain write + `expect_done` (claims 15–17); the arm reads config once itself (claim 9) |
| facts-gaps | `behavior_change: false` on the cell that flips the switch | ncc-4 is `behavior_change: true` |
| facts-gaps | `CODE_ROOTS` as a literal array lost D1's `crates/*` glob; the array size contradicted its entries | a predicate `under_code_root(rel)` with the glob spelled as a rule |
| facts-gaps | `commands.test`, the flag names, the rule-marker convention and the hand-edited registry were load-bearing prose | claims 26–28, 35–38 added |
| user-impact | five unmerged branches carry ~256 added comment lines; a main-only seed reds CI on their merge with no legal unblock | D2b `0add770d` logged: seed over main plus unmerged branches; claim 49; risk row HIGH→handled |
| user-impact | the refusal never named `///`/`//!` and no home existed for a pub API's description | the refusal names both and the owning knowledge concept as the API-doc home; D4's rendering in ncc-4 says so |
| user-impact | Windows CI runs the suite; `\` keys would never match | keys `/`-normalized, asserted (claim 34) |
| user-impact | `#!` inside heredoc bodies of `scripts/codex-parity-canary-test.sh` would count and then be refused | `#!` is an exception on any line (claim 42) |
| user-impact | the concept's "not yet done" sentence would block `bee close`'s doc-deferral door | fenced in a `bee:not-a-deferral` block in ncc-4 |
| user-impact | the ratchet is invisible to `bee doctor`/`orient` until CI | filed as backlog proposal (doctor row), out of scope here |
| user-impact | an ungranted worktree reads main's config and would be refused with no doctrine on its branch | named in the concept (ncc-4) |

Dismissed, with reason:
- alternatives 4's "a `#[ignore]` test instead of a verb" — D2 locks a
  verb.
- user-impact 2a's "strip comments as a merge precondition for the five
  branches" — the user chose stop-the-bleeding over disturbing in-flight
  work; D2b's seed covers it without touching them.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ncc-1 | Decide what a comment line is, once | `src/comments.rs` (new), `src/main.rs` (mod line) | — | unit tests prove: `//`, `///`, `//!`, block lines count; `#` counts in shell/python; any `#!` line, a `SAFETY:` line and a leading license block never count; a string-literal `//` counts; a move or delete adds nothing; an extension-less bash file is found by shebang; keys are `/`-normalized | `cargo test … --bin bee comments::` green, red-first |
| ncc-2 | Seed the baseline over every live branch and fence the count | `src/devtools/comment_baseline.rs` (new), `src/devtools/mod.rs`, `generated/registry_payload.json`, `.bee/comment-baseline.json` (new) | ncc-1 | `bee dev comment-baseline --write` seeds from main plus the unmerged branches and says so; `--check` and the ratchet test are green on the tree and red naming the file when one comment line is added; `--write` refuses to raise a count and lowers one that fell | `cargo test … --bin bee devtools::comment_baseline::`, `… --test registry_contracts --test registry_dispatch`, `… --bin bee catalog::` green, red-first |
| ncc-3 | Refuse a write that adds a comment line to a code file | `src/hooks/write_guard/main.rs`, `src/hooks/write_guard/guards.rs`, `src/hooks/write_guard/detectors.rs`, `src/hooks/write_guard/tests.rs`, `tests/hook_contracts.rs` | ncc-1 | with `no_code_comments: true`, an Edit/Write/MultiEdit that adds a comment line under a code root, a Bash heredoc into one, or an apply_patch adding one is refused naming file, line, the trimmed line, `///`/`//!`, the two homes and the API-doc home; moves, deletes, non-code paths, unreadable Bash content and repos without the key pass | `cargo test … --bin bee hooks::write_guard::`, `… --test hook_contracts` green, red-first; then `… --bin bee devtools::comment_baseline::` green (the arm's own files carry no new comment) |
| ncc-4 | State the rule by id where agents read, record it in the knowledge layer, map it, and switch it on | `packages/bee/AGENTS.block.md`, `packages/bee/prompts/worker-cell.md`, `AGENTS.md` (rendered), `.bee/bin/prompts/worker-cell.md` (rendered), `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`, `docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md`, `.bee/verify/verify-app/features/comment-guard.md` (new), `.bee/verify/verify-app/features/README.md`, `.bee/config.json`, `.bee/config-sample.json`, release manifest | ncc-2, ncc-3 | AGENTS.md's block carries `<!-- rule: agents-no-code-comments -->` and its index row exists; the worker prompt states the ban; the concept records the arm, the ratchet and the ungranted-worktree edge; the feature map drives both; this repo's config turns the guard on | regen, then parity, skew, rule-index, pointer-integrity and instruction-laws targets, manifest check, `rg -q` green |

```json
[
  {
    "id": "ncc-1",
    "feature": "no-code-comments",
    "title": "Decide what a comment line is, once",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["38e323d6-abc3-4cc7-b533-eb27ef0603ed"],
    "files": [
      "packages/bee-rs/crates/bee/src/comments.rs",
      "packages/bee-rs/crates/bee/src/main.rs"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee-rs/crates/bee/tests/instruction_laws.rs",
      "packages/bee-rs/crates/bee/tests/specs_fence.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"],
    "action": "Build the one classifier (per D1). NEW MODULE packages/bee-rs/crates/bee/src/comments.rs, declared in src/main.rs with a `mod comments;` line beside the other top-level modules (rg hit: `mod catalog;`). The module carries NO comments of its own — not `//`, not `///`, not `//!` — the fence will police it. Contents: `pub(crate) fn under_code_root(rel: &str) -> bool` — true for a `/`-separated repo-relative path that is under `packages/bee-rs/crates/<any>/src/`, `packages/bee/hooks/`, `packages/bee/lib/`, `scripts/`, or `.bee/verify/` (spell the crates rule as a prefix `packages/bee-rs/crates/` followed by one segment then `/src/`, never a fixed crate list); `pub(crate) enum Lang { Rust, Shell, Python }`; `pub(crate) fn code_lang(rel: &str, first_line: &str) -> Option<Lang>` by extension (.rs → Rust; .sh/.bash → Shell; .py → Python) or, when the name has no extension, by a `#!` first line containing sh, bash or python; `pub(crate) fn is_comment_line(lang: Lang, line: &str, in_block: &mut bool) -> bool` — Rust: trimmed line starts with `//` (covers `///` and `//!`), or `/*` opens a block that runs until `*/` with every inner line counting; Shell and Python: trimmed line starts with `#`; `pub(crate) fn is_exception(lang: Lang, line: &str, in_leading_block: bool) -> bool` — any line whose trimmed text starts with `#!` (a shebang, on line 1 or inside a heredoc body — scripts/codex-parity-canary-test.sh carries ten of them); a Rust comment whose text after the marker starts with `SAFETY:`; any comment line inside the file's leading comment block when that block contains `Copyright`, `SPDX-License-Identifier` or `Licensed under`; `pub(crate) fn count_comment_lines(lang: Lang, text: &str) -> usize` (comment lines minus exceptions); `pub(crate) fn comment_lines(lang: Lang, text: &str) -> Vec<(usize, String)>` — every counted comment line with its 1-based line number and trimmed content; `pub(crate) fn added_comment_lines(lang: Lang, old: &str, new: &str) -> Vec<(usize, String)>` — the multiset difference of comment_lines(new) minus comment_lines(old) by trimmed content, so a moved or deleted comment yields nothing, each with its line number in new; `pub(crate) fn walk_code_files(root: &Path) -> Vec<(String, Lang)>` — walk the repo under the five roots, skipping directories named target, node_modules or .git (the same skip list tests/instruction_laws.rs uses at `if matches!(name.as_ref(), \"target\" | \"node_modules\" | \".git\") {` — copy it, do not import it: tests cannot be imported), return `/`-normalized repo-relative paths sorted, each with its Lang; `pub(crate) fn repo_root() -> PathBuf` is NOT here — each caller resolves its own root. Document nothing in prose; let names and tests carry the meaning. TESTS, red-first, in a `#[cfg(test)] mod tests` at the bottom: each language's comment detection; `///` and `//!` count; a `/* */` block counts every inner line and closes; `#!` on line 1 and `#!` inside a body do not count; a `SAFETY:` line does not count; a leading license block does not count and a later `Copyright` comment does; a `//` at line start inside a string literal DOES count (assert it, with the note in the test name that it is the documented limitation); added_comment_lines returns nothing for a move and a delete and the right (line, text) for an add; code_lang finds an extension-less bash file by shebang and rejects a `.md`; under_code_root accepts `packages/bee-rs/crates/fleet/src/lib.rs` and rejects `packages/bee-rs/crates/bee/tests/x.rs`, `docs/x.rs` and `.bee/state.json`; walk_code_files on a temp tree returns `/` keys with no `\\` and skips a target/ directory.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee comments::",
    "must_haves": {
      "truths": [
        "comments.rs is the one home of under_code_root, Lang, code_lang, is_comment_line, is_exception, count_comment_lines, comment_lines, added_comment_lines and walk_code_files, and carries no comment line itself",
        "every #! line, every SAFETY: line and a leading license block are never counted; ///, //!, block inner lines and a string-literal // are counted",
        "added_comment_lines is empty for a move and a delete and names the line for an add",
        "walk_code_files returns /-normalized sorted keys under the five roots and skips target, node_modules and .git",
        "code_lang finds an extension-less shebang file"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/comments.rs", "substantive": "the classifier and its tests, comment-free"},
        {"path": "packages/bee-rs/crates/bee/src/main.rs", "substantive": "mod comments;"}
      ],
      "key_links": [
        "main.rs declares the module so devtools and hooks can use crate::comments"
      ],
      "prohibitions": [
        "No comment line of any kind in comments.rs",
        "No allowlist for string literals that look like comments",
        "No change to any other file"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "ncc-2",
    "feature": "no-code-comments",
    "title": "Seed the baseline over every live branch and fence the count",
    "lane": "standard",
    "role": "code",
    "deps": ["ncc-1"],
    "decisions": ["80ec3cd1-e51d-435c-ba50-570afa701c7c", "0add770d-a748-4b83-810a-493e546d560c"],
    "files": [
      "packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      ".bee/comment-baseline.json"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/comments.rs",
      "packages/bee-rs/crates/bee/src/devtools/release_manifest.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"],
    "action": "Build the ratchet (per D2, D2b). NEW MODULE packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs, comment-free, dispatched from devtools/mod.rs beside `\"release-manifest\" => release_manifest::run(flags),` as `\"comment-baseline\" => comment_baseline::run(flags),` and added to `const SOURCE_CHECKOUT_DEV_VERBS: [&str; 5] = [` (size 5 → 6) so it refuses outside a source checkout like release-manifest does. BASELINE: `.bee/comment-baseline.json` with shape `{\"files\": {\"<rel>\": <count>, ...}}`; `pub(crate) fn baseline_path(root) -> PathBuf`; `load_baseline(root) -> BTreeMap<String, usize>` over `fsutil::read_json` (absent file → empty map); `write_baseline(root, &BTreeMap)` builds the serde_json Map by inserting in BTreeMap order (this crate's serde_json has preserve_order, so sorted insertion IS the sort — the same answer verbs/drivers/close.rs gives at `/// the JSON \\`Map\\` in that already-sorted order`) and writes with `fsutil::write_json_atomic`. COUNTING: `current_counts(root) -> BTreeMap<String, usize>` = comments::walk_code_files + comments::count_comment_lines, keys `/`-normalized. SEED (D2b): when no baseline exists, `--write` seeds from the per-file MAXIMUM over the working tree and every unmerged `wt/*` branch: run `git branch --no-merged main --format=%(refname:short)` from root, and for each branch `git ls-tree -r --name-only <branch>` filtered through comments::under_code_root and comments::code_lang, read each file with `git show <branch>:<path>` and count; print `comment-baseline --write: seeded <n> file(s) from main plus <k> unmerged branch(es) (<names>)`. If git is unavailable or a branch read fails, seed from the working tree only and say so in the output — never silently. VERB: `--check` computes current counts and prints one line `<rel>: <count> comment line(s), baseline <b>` per file whose count exceeds its entry (absent entry → 0), then the two-sentence remedy `bee dev comment-baseline --write lowers a count, never raises one. A count that rose from a merge is fixed by deleting the merged comment lines.`, exit 1; none → `comment-baseline --check: <n> file(s) at or below baseline`, exit 0. `--write` with a baseline present: lower every entry whose count fell, drop entries for files that no longer exist, keep entries above the tree unchanged (they are the in-flight headroom), and REFUSE (exit 1, nothing written) naming every file whose count would rise. Both flags absent → the usage refusal shape release-manifest uses. REGISTRY: add `dev.comment-baseline` to generated/registry_payload.json (hand-edited, decision 3358743e) with properties `check` and `write` reusing the existing names, an example `bee dev comment-baseline --check`, and a description in one paragraph; the catalog pin stays 211 — run the catalog test to prove it. THE RATCHET TEST: in this module's `#[cfg(test)] mod tests`, `every_code_file_is_at_or_below_its_comment_baseline`: resolve the repo root the way tests/specs_fence.rs does (`PathBuf::from(env!(\"CARGO_MANIFEST_DIR\")).ancestors().nth(4)`), load the baseline, compute counts, and assert nothing exceeds, failing with the same per-file lines and the two-sentence remedy. Then RUN `--write` once from the worktree root to seed `.bee/comment-baseline.json` and commit it with the code. TESTS, red-first, in the module: seed from a temp git repo with one unmerged branch carrying an extra comment → the baseline holds the branch's higher count; lower; refuse-to-raise names the file; drop a deleted file; keys carry no backslash; `--check` red text names file, count and baseline and both remedy sentences; the ratchet test itself is green on the seeded tree.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee devtools::comment_baseline:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee catalog:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch",
    "must_haves": {
      "truths": [
        ".bee/comment-baseline.json exists, sorted, /-keyed, and was seeded from the working tree plus every unmerged wt/* branch; the cap report names the branches and the totals",
        "bee dev comment-baseline --check exits 0 on the seeded tree and exits 1 naming file, count and baseline plus the two remedy sentences when one comment line is added to a code file",
        "bee dev comment-baseline --write refuses to raise any count, lowers a count that fell, drops a deleted file and keeps in-flight headroom",
        "the ratchet test every_code_file_is_at_or_below_its_comment_baseline is green on the seeded tree and red with the same message on a rise",
        "comment-baseline is in SOURCE_CHECKOUT_DEV_VERBS, dev.comment-baseline is in the registry, PINNED_FLAG_COUNT stays 211, both registry targets are green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs", "substantive": "baseline IO, seed over live branches, --check, --write, the ratchet test, tests; comment-free"},
        {"path": ".bee/comment-baseline.json", "substantive": "the seeded per-file counts"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "dev.comment-baseline entry"},
        {"path": "packages/bee-rs/crates/bee/src/devtools/mod.rs", "substantive": "dispatch arm and SOURCE_CHECKOUT_DEV_VERBS entry"}
      ],
      "key_links": [
        "comment_baseline.rs counts only through comments::walk_code_files and comments::count_comment_lines — no second counter",
        "the ratchet test and --check print the same lines"
      ],
      "prohibitions": [
        "No comment line is removed from any existing file (D6)",
        "The baseline is never hand-edited; the verb writes it",
        "No comment line in the new module",
        "No change to comments.rs or the hook"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "ncc-3",
    "feature": "no-code-comments",
    "title": "Refuse a write that adds a comment line to a code file",
    "lane": "standard",
    "role": "code",
    "deps": ["ncc-1"],
    "decisions": ["c2477366-4b6c-4eca-a3c7-37b2062f12da", "e3bf4a57-6779-4ca2-895d-f03f9b720c1d", "0ee8248d-a4c5-4a7d-8d84-c0184b0c70f5"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/tests/hook_contracts.rs"
    ],
    "read_first": [
      "docs/history/no-code-comments/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/comments.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", ".bee/verify/verify-app/features/comment-guard.md"],
    "action": "Add the comment arm to the write guard (per D3, D7, D4). All new code in this cell is comment-free; the ratchet in ncc-2 polices these files, so the proof ends with its test target. SHARED RECONSTRUCTION: extract from the config arm (rg hit: `        // crkg-1: config-role-key-guard arm over .bee/config.json and .bee/config.local.json`) one helper `fn reconstruct_target_text(tool_name: &str, tool_input: &Map<String, Value>, root_pb: &Path, rel: &str) -> Option<(String, String)>` returning (old, new) whole-file text: Write → old = disk or empty, new = `content` (rg hit: `let Some(content) = tool_input.get(\"content\").and_then(Value::as_str) else {`); Edit → old = disk (rg hit: `let Ok(current_text) = std::fs::read_to_string(root_pb.join(rel)) else {`), new = `current_text.replacen(old_s, new_s, 1)` (rg hit: that line); MultiEdit → the same fold over `edits` (rg hit: `let Some(Value::Array(edits)) = tool_input.get(\"edits\") else {`); None when the text cannot be reconstructed. Make the config arm call the helper for its own reconstruction so there is one home, keeping its refusals byte-identical (its existing tests prove that). NEW READERS: in guards.rs beside `pub(crate) fn fence_heredocs(command: &str) -> String {`, `pub(crate) fn heredoc_writes(command: &str) -> Vec<(String, String)>` — for each simple command that carries both a heredoc (`<<TAG` / `<<'TAG'` / `<<-TAG`) and a `>`/`>>` redirect target, the (target, body) pair, body = the lines between the operator line and the terminator; nothing for a heredoc with no redirect target or a redirect with no heredoc; in detectors.rs beside `pub(crate) fn extract_apply_patch_targets(patch_text: &str) -> Vec<String> {`, `pub(crate) fn apply_patch_added_lines(patch_text: &str) -> Vec<(String, Vec<String>)>` — per `*** Add File:` / `*** Update File:` target, the lines that start with `+` (marker stripped), in order. THE ARM, in main.rs directly after the config arm and only while `denial.is_none()`: read the merged config once here with `read_config(&store_root_pb)?` (the existing read at main.rs:126 is in the read-tool branch and is not reachable) and skip the whole arm unless `config.get(\"no_code_comments\") == Some(&Value::Bool(true))`; for each `rel` in `rel_paths`, skip the `**` sentinel (rg hit: `rel_paths = vec![\"**\".to_string()];`), skip unless `crate::comments::under_code_root(rel)`, then gather candidates: Write/Edit/MultiEdit → `reconstruct_target_text`, lang from `comments::code_lang(rel, first line of new)`, `added = comments::added_comment_lines(lang, &old, &new)`; Bash/exec → for each (target, body) from `heredoc_writes(&command)` whose normalized target equals rel: old = disk or empty, new = body (a `>>` append: new = disk + body), same diff; a Bash write with no readable body (echo, printf, sed, mv, cp) is NOT refused by this arm; apply_patch → for each (target, lines) from `apply_patch_added_lines` whose target equals rel: added = the `+` lines that `comments::is_comment_line` counts and `is_exception` does not, numbered by their position among the added lines (the refusal says `added line <n>`); Delete/Move targets are skipped. On the first non-empty `added`, `denial = Some(...)` with ONE message: `bee comment guard denied this write: <rel>:<line> adds a comment line (\"<trimmed, max 80 chars>\"). This repository keeps no comments in code (no_code_comments) — not //, not /// or //! doc comments, not /* */, not #. FIX: put the why in a docs/knowledge concept whose Pointers name this file, or log it with bee decisions log; a public item's description goes in the owning docs/knowledge concept; a workaround is fixed, or filed with bee backlog add and cited from the concept — never from the code. Exceptions: a #! line, a SAFETY: line on unsafe, a license header at file top.` Never refuse a target that is not under a code root or not a code file. TESTS, red-first, in write_guard/tests.rs, using `fn build_fixture(phase: &str, execution_approved: bool) -> Fx {`, a plain `std::fs::write(fx.root.join(\".bee/config.json\"), ...)` carrying `{\"no_code_comments\": true}` (the config-guard tests at tests.rs:313 show the shape), and `fn expect_done(payload: Value, cwd: &Path) -> Emit {`; EVERY fixture string stays an inline one-line `json!`/`\"...\\n...\"` literal — no multi-line raw string whose body line starts with `//` or `#`, or the ratchet counts it. Cases: Edit adding `// note` into packages/bee-rs/crates/bee/src/x.rs (write the file first) → deny with FIX, `x.rs:<line>`, the text `/// or //!`; the same Edit with the key absent → allow, with `false` → allow; Edit whose old_string and new_string carry the same comment moved → allow; Edit that deletes a comment → allow; Write of a new .rs file with a `///` line → deny; Write of a `.md` with `// x` → allow; Write under `docs/` → allow; Bash `cat > scripts/x.sh <<'EOF'` with a `# note` body line → deny, with only a `#!/usr/bin/env bash` body line → allow; Bash `echo '# x' > scripts/x.sh` → allow (unreadable); a `SAFETY:` line → allow; MultiEdit with one offending edit → deny naming it; apply_patch adding a `//` line under the crate → deny naming `added line <n>`; guards.rs unit tests for heredoc_writes (with and without a target, two heredocs, unterminated) and detectors.rs unit tests for apply_patch_added_lines (two targets, a Delete target ignored). In tests/hook_contracts.rs beside `fn a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr() {`: one end-to-end test that the comment deny reaches the host as exit 2 with FIX on stderr.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::write_guard:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test hook_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee devtools::comment_baseline::",
    "must_haves": {
      "truths": [
        "with no_code_comments true, an Edit, Write or MultiEdit that adds a comment line to a file under a code root, a Bash heredoc write into one, and an apply_patch that adds one are refused with a message naming the file, the line, the trimmed line, that /// and //! are comments, the two homes for a why, the home for a public item's description, and the three exceptions",
        "a moved or deleted comment, a non-code target, a #! line, a SAFETY line, a license header, a Bash write with no readable body, and a repository without the key all pass this arm",
        "the config arm still emits byte-identical refusals through the shared reconstruction helper",
        "the deny reaches the host as exit 2 with FIX on stderr",
        "every other write-guard test stays green and the comment ratchet stays green over the arm's own files"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs", "substantive": "reconstruct_target_text shared by the config arm and the comment arm; the comment arm gated by no_code_comments"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs", "substantive": "heredoc_writes with tests"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs", "substantive": "apply_patch_added_lines with tests"},
        {"path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs", "substantive": "the arm's tests across the five tool shapes and the pass cases, inline fixtures only"},
        {"path": "packages/bee-rs/crates/bee/tests/hook_contracts.rs", "substantive": "one end-to-end deny test"}
      ],
      "key_links": [
        "the arm calls comments::under_code_root, code_lang, added_comment_lines, is_comment_line and is_exception — no second predicate",
        "the arm sits after the config arm and only while denial is none; config is read once in the write branch"
      ],
      "prohibitions": [
        "No change to any existing refusal's text or order",
        "No refusal on .md, .json, .toml, docs or .bee state paths, and none on a Bash write whose content the guard cannot see",
        "No comment line in main.rs, guards.rs, detectors.rs, tests.rs or hook_contracts.rs from this cell; no multi-line fixture whose line starts with // or #"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "ncc-4",
    "feature": "no-code-comments",
    "title": "State the rule by id where agents read, record it in the knowledge layer, map it, and switch it on",
    "lane": "standard",
    "role": "docs",
    "deps": ["ncc-2", "ncc-3"],
    "decisions": ["002a935d-0da1-4ab1-acea-0c208806f31a", "0ee8248d-a4c5-4a7d-8d84-c0184b0c70f5", "e3bf4a57-6779-4ca2-895d-f03f9b720c1d", "69326d15-0a5d-44a4-b63b-c544686d2d61"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "packages/bee/prompts/worker-cell.md",
      "AGENTS.md",
      ".bee/bin/prompts/worker-cell.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
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
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      ".bee/verify/verify-app/features/cells-and-proof.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", ".bee/verify/verify-app/features/comment-guard.md", "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
    "action": "Land the rule where agents read it (per D5, D4), record it in the knowledge layer, map it, and switch it on (per D7), with ONE regen; never hand-edit AGENTS.md (it renders from the block). (1) packages/bee/AGENTS.block.md: add a MARKED rule directly before the bullet `- Write a mistake down the MOMENT you notice it` (rg hit: `- Write a mistake down the MOMENT you notice it, never composed from`): `<!-- rule: agents-no-code-comments -->` … `<!-- /rule -->` around one bullet, shaped like the marker at `<!-- rule: agents-one-commit-per-cell -->`: no comment in code, any language — not `//`, `///` or `//!`, not `/* */`, not `#`; the why lives in a docs/knowledge concept whose Pointers name the file, or in bee decisions log; a public item's description lives in the owning docs/knowledge concept; a workaround is fixed or filed with bee backlog add and cited from the concept; the three exceptions (a `#!` line, a `SAFETY:` line on unsafe, a license header at file top); the write guard refuses the write where `no_code_comments` is on and the comment baseline reds the suite. (2) docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, section `## AGENTS.md rule homes`: add the row for `agents-no-code-comments` in the shape of the `agents-one-commit-per-cell` row (its home section and spoken line), so tests/rule_index_parity.rs finds the id in all three places. (3) packages/bee/prompts/worker-cell.md: extend the craft bullet (rg hit: `- Shape what you leave behind: prefer deletion to addition, write the smallest diff that solves it, and leave the base simpler than you found it.`) with one sentence: no comments in code — the why goes to docs/knowledge or bee decisions log, a public item's description to its concept; the guard refuses the rest. (4) docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md: add a section for the comment arm and the ratchet — the config key and that a granted worktree reads its own config while an ungranted one reads main's; the code roots (Pointers: packages/bee-rs/crates/bee/src/comments.rs, packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs, .bee/comment-baseline.json); what a comment line is per language and the three exceptions; the refusal shape and which tool shapes it reads (Write, Edit, MultiEdit, a Bash heredoc with a redirect target, apply_patch added lines) and which it cannot (a Bash write with no readable body); the string-literal limitation; the baseline's seed over unmerged branches (D2b), `--check`/`--write`, the ratchet test name; cite 38e323d6, 80ec3cd1, 0add770d, c2477366, 0ee8248d, 002a935d, 69326d15, e3bf4a57 in the frontmatter decisions list. Write the D6 sentence about the existing comments inside a fenced `<!-- bee:not-a-deferral: D6 records the batched cleanup as separate grooming work; nothing here promises a date --> … <!-- /bee:not-a-deferral -->` block so bee close's doc-deferral door does not stop on it. (5) .bee/verify/verify-app/features/comment-guard.md, modeled on cells-and-proof.md: what it is, how a user reaches it (an Edit adding `// x` to a .rs under a code root with the key on; `bee dev comment-baseline --check`), how to drive it with control-bee (set no_code_comments true in the sandbox's .bee/config.json via `control-bee put`, then run the hook with a crafted Edit payload the way tests/hook_contracts.rs does, or drive the refusal from a hooked session; run `control-bee cli -- dev comment-baseline --check` — note it refuses outside a source checkout, so the verb is driven from the source root, not the sandbox), gotchas (string-literal counting; hosts default off; an ungranted worktree reads main's key; the baseline never rises and a merge-caused rise is fixed by deleting the merged lines); add its row to README.md (rg hit: `- [Cells and the proof line](./cells-and-proof.md) covers adding, claiming and`). (6) .bee/config.json: add `\"no_code_comments\": true` beside `uat_stop`; .bee/config-sample.json: the key with `false`, nothing else. (7) Run `.bee/bin/bee dev regen` ONCE from the worktree root; stage by explicit path list — every listed file plus every rendered copy regen touched (skill mirrors of the feature map, .bee-render.json files, .bee/onboarding.json) — and check `git status` for packages/bee-rs/crates/bee/.bee/logs before the commit; never stage that log. Do NOT reinstall .bee/bin/bee. PROOF: after the regen, run the verify's test targets and checks; read back the rendered AGENTS.md rule and quote it in the cap report (`green:live`).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee verbs::drivers::tests && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test agents_block_render_parity --test rule_index_parity --test pointer_integrity --test instruction_laws && .bee/bin/bee dev release-manifest --check && rg -q no_code_comments packages/bee/AGENTS.block.md AGENTS.md packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md .bee/verify/verify-app/features/comment-guard.md .bee/config.json && rg -q agents-no-code-comments docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md AGENTS.md",
    "must_haves": {
      "truths": [
        "AGENTS.block.md carries the marked rule agents-no-code-comments with the two homes, the API-doc home and the three exceptions; AGENTS.md's rendered block matches and the rule-index row exists (parity and rule-index tests green)",
        "worker-cell.md's craft bullet states the ban and the homes; the vendored copy matches (skew test green)",
        "write-guard-request-shapes.md records the arm, the readable and unreadable tool shapes, the ratchet, the seed rule and the eight decision ids, with the D6 sentence fenced as not-a-deferral (pointer-integrity green)",
        "comment-guard.md exists and README.md lists it",
        ".bee/config.json sets no_code_comments true; config-sample.json shows the key false",
        "release-manifest --check is green after one regen and the test-timings log was never staged"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "the marked rule"},
        {"path": "AGENTS.md", "substantive": "the rendered block — written by regen"},
        {"path": "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md", "substantive": "the rule-index row"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "the craft bullet's no-comment sentence"},
        {"path": "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md", "substantive": "the comment arm and ratchet section with decision ids"},
        {"path": ".bee/verify/verify-app/features/comment-guard.md", "substantive": "the feature file"},
        {"path": ".bee/config.json", "substantive": "no_code_comments: true"}
      ],
      "key_links": [
        "AGENTS.md's block equals the rendered template; the rule id is in the block, the render and the index",
        "the concept's Pointers name comments.rs, comment_baseline.rs and .bee/comment-baseline.json",
        "release-manifest.json is refreshed in the same commit"
      ],
      "prohibitions": [
        "No hand edit of AGENTS.md or any mirrored copy",
        "No Rust edits; no reinstall of .bee/bin/bee",
        "No other config key changes",
        "packages/bee-rs/crates/bee/.bee/logs/timings.jsonl is never staged"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

| Cell | Happy path | Edge | Error | Pass when |
|---|---|---|---|---|
| ncc-1 | each language's comment detected | `#!` anywhere, `SAFETY:`, license block, block comment, string-literal `//`, extension-less shebang file, `/` keys | — | every case is one named test |
| ncc-2 | seed over main + one unmerged branch; `--check` green | lower; drop deleted; keys without `\` | one added `//` reds `--check` and the ratchet test naming file, count, baseline and both remedies; `--write` refuses to raise | the seeded baseline's totals reported in the cap |
| ncc-3 | Edit adding `// note` refused with FIX naming `///`/`//!` | move, delete, `.md`, `docs/`, key absent, shebang-only heredoc, echo write, SAFETY | MultiEdit with one bad edit; apply_patch `+` comment; heredoc `# note` | exit 2 + FIX end to end; the ratchet stays green over the arm's files |
| ncc-4 | regen renders both homes; parity, skew, rule-index, pointer, instruction-laws green | — | — | `rg -q` over the files exits 0; manifest check ok |
| existing behavior | full write-guard suite, hook_contracts, catalog, registry on head | — | — | every pre-existing test passes unchanged |

## Test scoping

Each code cell's proof is the filtered release run its `verify` names
(`--bin bee <module>::` is exact); the full declared `commands.test` runs
once before merge and CI runs it on the push on Linux and Windows. The
ratchet test is part of that suite from ncc-2 on.

## Open Questions

- (none)

## Out of scope

- Removing any existing comment (D6) — batched grooming later; each batch
  lowers the baseline through the verb.
- A `bee doctor` row for the baseline — filed as a backlog proposal
  (2026-09-22) so a rise shows before a push.
- A clippy or rustfmt rule — the classifier covers three languages and doc
  comments, which no lint does.
- Turning the guard on in hosts (D7) — a host flips its own key.
