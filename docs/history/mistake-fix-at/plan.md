---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Mistake Fix-At

Route: class `feature` · lane `standard` · flags `public-contracts`,
`covered-contract-change`, `multi-domain` · product files 12.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

Revision 2 — rewritten after the plan-step hat wave. What the wave changed
is listed under `## Hat wave` below.

## Summary

Every mistake an agent writes down now says which layer fixes it:
`architecture`, `check`, `doctrine`, or `none`. The two verbs that record a
mistake refuse one without that word, and the refusal itself lists the four
words and the remedy, so a cold worker recovers in one retry. When a feature
closes, each mistake marked `check` or `architecture` becomes one backlog row
on the spot, and the close line says where to read them. The weekly lesson
miner stops matching whole sentences and matches layer plus the first four
words, and the lesson it logs says what matched and quotes both runs. The
instruction lands in the three places an agent reads before it writes a
mistake.

Mode: `standard` — 3 risk flags: public-contracts (two CLI verbs gain a
required flag), covered-contract-change (existing reflect, cap, close and
miner tests assert the current shape), multi-domain (mailbox, cells, close,
digest, doctrine).
Why this is the least workflow that protects the work: a required flag on a
public verb and a new close-time write are contract changes that need a plan
and a frozen shape; nothing here touches auth, data loss or an external
system, so high-risk would be ceremony.

## Requirements (from CONTEXT.md)

- D1 (af14e1d6): `bee mailbox reflect` and `bee cells cap --mistake` take a
  required `--fix-at` from `architecture | check | doctrine | none`; missing
  or outside the vocabulary → refused through the same door `--wrong`/`--better`
  use. `--no-mistakes` unchanged.
- D2 (6e79d14e): at `bee close`, every reflection of the closing feature with
  fix-at `check` or `architecture` files exactly one backlog row (type
  `finding`, severity `P3`, layer `fix-at:<value>`, title = the wrong text,
  detail = the better text plus its run or cell). Never twice for the same
  entry. `doctrine`/`none` file nothing. Source set recorded by the D2a
  decision (`d1d83be7`, touches D2): the feature's capped cells'
  `trace.mistakes` plus the closing run's own mailbox entries.
- D3 (9173fbd2): the weekly lesson miner keys a reflection by fix-at plus the
  normalized first four words of its wrong text. Four-word minimum, two
  distinct runs, once-ever token: unchanged, for `doctrine`/`none`.
- D4 (5c43da33): the fix-at instruction lands in `AGENTS.md`,
  `packages/bee/AGENTS.block.md`, and the rendered worker/cap prompt, each
  naming the vocabulary and that the flag is required. No backfill; old rows
  read as `none`.
- D5 (402cf686): bee-capturing's promotion tree reads the closing feature's
  fix-at reflections at step 1; a `check`/`architecture` one may ship
  in-feature as a tiny cell and mark its D2 row done; `doctrine` routes
  through steps 3–4 unchanged.

## Load-bearing claims

Labels: `read` (opened the file at that line, saw those bytes), `ran`
(executed the command, hold its output). Match rule: the evidence column is a
verbatim byte substring of the anchored line(s); multi-line joins with " / ".
Every row is load-bearing; no `guessed` row.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The two-part door has one home, and D1's third part extends it there | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:795` | `pub(crate) fn read_reflection<'a>(` |
| 2 | The refusal text says "two required parts" and its remedy names only the two flags — D1 rewrites this one string so the refusal lists the four values | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:806` | `"a reflection has two required parts, and it is missing {} (D3) — remedy: pass --wrong (what went wrong) and --better (what would have been better), each with text.",` |
| 3 | The entry row's last key is `better`; a `fix_at` key appended after it is additive | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:668` | `"better": self.better,` |
| 4 | Old entry rows read back with `None` for an absent key, so no backfill is needed (D4) | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:739` | `better: opt_string(m, "better"),` |
| 5 | The reflection constructor is the one place a reflection entry is built | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:679` | `pub(crate) fn reflection(at: &str, wrong: &str, better: &str) -> Self {` |
| 6 | The letter frontmatter emits `better` last as the additive key — `fix_at` emits after it | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1365` | `emit_opt(&mut out, 4, "better", item.better.as_deref());` |
| 7 | The cap's mistake line is parsed by `read_mistake`, which ends at the same door | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:830` | `pub(crate) fn read_mistake(v: &Value) -> Result<(String, String), String> {` |
| 8 | `run_reflect` builds the entry from the door's two parts — D1's third part rides here | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2943` | `let entry = Entry::reflection(&now_iso(), wrong, better);` |
| 9 | `trace.mistakes` items are `{wrong, better}` objects — `fix_at` is a third key | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:572` | `.map(\|(wrong, better)\| json!({ "wrong": wrong, "better": better }))` |
| 10 | The cap flag is a single raw string; a sibling `fix_at` flag has a home beside it | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:123` | `pub(crate) mistake: Option<String>,` |
| 11 | The cap reads flags and report once, then mirrors into the mailbox with the same constructor | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1036` | `&mailbox::Entry::reflection(&utc_now(), wrong, better),` |
| 12 | The report's `mistakes` refusal text spells the two-part line — D1 changes it to three | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:211` | `"cells finish: --report key \"mistakes\" must be an array — one entry per mistake, each \"<what went wrong> — <what would have been better>\"; an empty array states that this cell hit none."` |
| 13 | The cap's allowed flags are a fixed-size array (`15` → `16`) checked before parsing; `cells finish` shares it (one parse site) | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:35` | `pub(crate) const CAP_FLAGS: [&str; 15] = [` |
| 14 | That array is what refuses an unknown cap flag | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:1130` | `if !rsv::keys_known(&flags, &CAP_FLAGS) {` |
| 15 | `close` already walks the feature's capped cells, archive included — D2 reuses that walk | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:269` | `for cell in list_cells_including_archive(root, feature, Some("capped"))? {` |
| 16 | Nothing reads `trace.mistakes` items after the door — the door only counts presence | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:274` | `let recorded = matches!(vget(&trace, "mistakes"), Some(Value::Array(a)) if !a.is_empty());` |
| 17 | `close` reads the closing run's own mailbox entries — D2 covers those reflections too | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:299` | `mailbox::read_entries(&control, &run).iter().any(mailbox::Entry::is_mistakes_answer)` |
| 18 | The green tail begins here; D2's walk runs after it | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2925` | `let headline = format!("Tests GREEN for \"{feature}\" — {}", proof_door_detail(&proof));` |
| 19 | Retirement moves the cells; the walk runs before this line, the text line prints after the Retired line | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2976` | `let retired = auto_archive_on_close(root, feature);` |
| 20 | The tail's lines name the feature first and end with the next move — the new line copies that voice | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2986` | `"Retired \"{feature}\": {moved} cell(s) moved out of the active scan (bee cells unarchive --feature {feature} to reverse)."` |
| 21 | The shape token is a sha over the whole normalized sentence | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:685` | `hasher.update(normalized.as_bytes());` |
| 22 | The miner discriminates reflections by `better` on the letter item, so `fix_at` must ride the letter | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:711` | `let better = item.better.as_deref()?;` |
| 23 | The four-word brake is a named constant D3 keeps | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:621` | `const MIN_SHAPE_WORDS: usize = 4;` |
| 24 | The two-run brake is a named constant D3 keeps | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:624` | `const MIN_DISTINCT_RUNS: usize = 2;` |
| 25 | Spent tokens are matched by prefix over rationale words; a new key yields new digests, so no old token collides | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:834` | `if word.starts_with(SHAPE_TOKEN_PREFIX) {` |
| 26 | Every trouble line is normalized and word-counted at one site in `mine_shapes` — so `trouble_lines` must hand back the key and the verbatim apart, and apply the brake to the `what` | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:796` | `let normalized = normalize_shape(&line);` |
| 27 | `trouble_lines` returns bare strings today — the key/verbatim pair is a signature change | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:740` | `fn trouble_lines(letter: &Letter) -> Vec<String> {` |
| 28 | The lesson rationale is one `format!` mfa-3 rewrites to name the key and the other run's sentence | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:851` | `fn lesson_rationale(period: &Period, shape: &Shape) -> String {` |
| 29 | The backlog row shape lives inside `run_add`, so D2 needs the row builder lifted | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:799` | `fn run_add(parsed: ParsedArgs, queue_submit: bool, t0: Instant) -> Option<ExitCode> {` |
| 30 | The row's key order is fixed and D2 keeps it | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:833` | `// Row key order: ts, type, title, detail, severity, layer, feature.` |
| 31 | The append itself already has one home — close calls it, nothing is lifted on that side | read | `packages/bee-rs/crates/bee/src/fsutil.rs:170` | `pub fn append_jsonl(file: &Path, value: &Value) -> std::io::Result<()> {` |
| 32 | The backlog path helper is private today — the dedupe reader needs it `pub(crate)` | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:73` | `fn backlog_jsonl_path(root: &Path) -> PathBuf {` |
| 33 | A finding-row predicate already exists for the dedupe fold | read | `packages/bee-rs/crates/bee/src/verbs/backlog.rs:385` | `fn is_finding_row(row: &Value) -> bool {` |
| 34 | A new flag NAME bumps the pinned count on purpose | read | `packages/bee-rs/crates/bee/src/catalog.rs:800` | `const PINNED_FLAG_COUNT: usize = 210;` |
| 35 | The registry payload is hand-edited here; its tests are the proof | read | `docs/decisions/index.md:2013` | `3358743e · 2026-08-05 · worktree-reclaim D5: packages/bee-rs/crates/bee/src/generated/registry_payload.json is hand-edited in this repo` |
| 36 | The dispatcher's worker prompt is compiled into the binary from the source template | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs:33` | `pub(crate) const PROMPT_WORKER_CELL: &str = include_str!("../../../../../../bee/prompts/worker-cell.md");` |
| 37 | The prompt's Result form spells the two-part mistake — D4's third home | read | `packages/bee/prompts/worker-cell.md:65` | `"mistakes": ["<what went wrong> — <what would have been better>", "..."]` |
| 38 | The doctrine bullet lives in the template that renders both homes | read | `packages/bee/AGENTS.block.md:301` | `memory at the end: \`bee mailbox reflect --wrong "<what went wrong>"\`` |
| 39 | AGENTS.md's bee block is a generated region — never hand-edited | read | `AGENTS.md:7` | `<!-- BEE:START -->` |
| 40 | Onboarding renders that block from the on-disk template, so the template is the one source for D4 homes 1 and 2 | read | `packages/bee-rs/crates/bee/src/onboard/source.rs:67` | `agents_block_template: templates_dir.join("AGENTS.block.md"),` |
| 41 | Onboarding copies the vendored prompt from the on-disk template, so regen with the old vendored binary still renders the new prompt | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:811` | `let source = read_text_if_exists(&engine.templates_prompts_dir.join(name));` |
| 42 | The regen chain is three fixed steps and its second step is the onboarding that renders both copies | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:143` | `Step { name: "render-skill-trees", invoke: Box::new(\|\| skill_trees::run(&[])) },` |
| 43 | A lib test compares the compiled-in prompt with the vendored copy on disk — the docs cell must run regen before it and run it | read | `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs:9551` | `fn every_template_carries_the_original_request_block_and_matches_disk() {` |
| 44 | `bee dispatch prepare` refuses on the same skew — so the prompt edit lands in the LAST cell and the vendored binary is reinstalled after merge | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3545` | `if let Some(skew) = prompt_skew(check_root, prompt_name) {` |
| 45 | `packages/bee` is a manifest root — the docs cell owes the manifest check | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:85` | `"packages/bee",` |
| 46 | `skills` is a manifest root too | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:89` | `"skills",` |
| 47 | The manifest file a covered cell must list | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:94` | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |
| 48 | The promotion tree's step 1 names three inputs and not the run's own mistakes (D5) | read | `skills/bee-capturing/references/promotion.md:110` | `1. Seen twice (review finding, user correction, repeated deviation)` |
| 49 | The knowledge overview's rule 15 must be synced in the same change | read | `docs/knowledge/areas/human-mailbox/overview.md:160` | `15. **Lesson mining reads only trouble** (LD4, widened by RBL D3): the` |
| 50 | Recent cells spelled a multi-step verify with `&&`, `--manifest-path`, `--test` targets and the vendored binary path | read | `.bee/cells/archive/finding-recheck-trigger/frt-1.json:22` | `cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch` |
| 51 | No lesson has ever been mined | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && .bee/bin/bee decisions search --tag lesson --json \| python3 -c "import sys,json; d=json.load(sys.stdin); r=d if isinstance(d,list) else d.get('results',d.get('decisions',[])); print(len(r))"` | `0` |
| 52 | 465 reflection entries exist with no layer — they read as `none` (D4); the store lives in the main checkout | ran | `cd /home/thanhsmind/Projects/goglbe/beehive && rg -o '"kind":\s*"(reflection\|no_mistakes\|no-mistakes)"' -r '$1' --no-filename .bee/human-mailbox \| sort \| uniq -c` | `104 no-mistakes` / `465 reflection` |

## Discovery

Inspected the reflect verb, the cap handler, the close driver, the lesson
miner, the backlog writer, the catalog pin, the prompt source, the regen
chain and the three doctrine homes (claims 1–50). Finding: the chain
reflect → letter → miner → lesson exists end to end and has produced zero
lessons (claim 51) because the key is the whole sentence (claim 21); nothing
reads `trace.mistakes` items after the door (claim 16). Feature-map files
read: `.bee/verify/verify-app/features/cells-and-proof.md` (2026-09-19) and
`worktree-and-close.md` (2026-09-16) — the `--no-mistakes` gotcha's clause
"or put a `mistakes` array in `--report`" goes false the moment mfa-1 lands,
so that file rides mfa-1.

## Approach

Recommended path — data shape first (feature playbook step 1): the `fix_at`
field rides the entry row (claim 3), the letter item (claim 6) and the cap
trace (claim 9) under one name, `fix_at`, with one vocabulary constant in
`mailbox.rs`. The door grows a third part (claims 1–2, D1) and its refusal
becomes self-sufficient: it names the missing flag, lists the four values,
and gives the remedy — because a host that upgrades the binary before it
re-onboards meets this refusal with stale doctrine. The cap's `--fix-at`
flag applies to its `--mistake` line; report array items carry their own,
as a third ` — ` segment or a `fix_at` object key (D1). Because the three
signatures' callers live in the cap handler, the door change and the cap
change are one compile unit and one cell. Close files D2 rows in the green
tail (claim 18) before retirement (claim 19), through a lifted row builder
(claim 29) and the existing appender (claim 31), deduped by an exact
`(feature, layer, title)` fold over `read_jsonl` + `is_finding_row` (claims
32–33), fail-open with a warning, and prints one line in the tail's voice
naming the command that reads the rows (claim 20). The miner hands back the
key and the verbatim apart (claims 26–27), applies the four-word brake to
the `what`, and the lesson row says what matched and quotes the other run
(claim 28). The doctrine and prompt sources change in one docs cell that
runs LAST, because the prompt edit skews the vendored binary's dispatcher
(claim 44); the vendored binary is reinstalled after merge, on main.

Rejected alternatives:
- Third segment of `--mistake` instead of a `--fix-at` flag on the verb —
  rejected: `bee mailbox reflect` has no line form, so the two verbs would
  learn two spellings; the report array keeps the segment form because a
  worker writes a list there.
- Dedupe by a new stored key on the backlog row — rejected: the row's key
  order is a contract (claim 30) and the digest reads it.
- Keep the door and the cap in two cells — rejected after the wave: the
  door's signature change cannot compile without editing the cap handler.
- Backfill 465 old rows with a guessed layer — rejected per D4.

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| `read_reflection` third part | MEDIUM — every reflect/cap caller now needs `--fix-at`, hosts on a new binary included | mfa-1, mfa-4 | refusal names `--fix-at`, lists the four values and the remedy; doctrine and prompt updated in the same slice; `green:live` drive after merge |
| Catalog pin + registry payload | LOW — a missed pin bump reds `catalog.rs`'s own test | mfa-1 | `cargo test … catalog` and the two registry test targets green |
| Close-time backlog write | MEDIUM — a write in the green tail must never refuse a close | mfa-2 | test: unwritable backlog → close still exits 0 with a warning line |
| Rows sink in the digest | LOW — P3 rows cluster by whole title and rank last in `bee feedback rank`; `bee backlog rank` reads `docs/backlog.md`, not the JSONL | mfa-2 (the close line names `bee backlog findings --feature <feature>`) | out of scope: a layer-aware reader (deferred idea in CONTEXT.md) |
| Miner key change | LOW — old tokens cannot collide (claim 25); old letters carry no `fix_at` | mfa-3 | tests: same layer + head → one lesson; different layer → none; letters without `fix_at` key as `none`; rationale names the key and the other sentence |
| Vendored binary and prompt skew | MEDIUM — the prompt edit skews the vendored binary (claim 44); during the wave every cap runs the OLD binary, which reads a three-segment line as two (the third folds into `better`) and refuses `--fix-at` as unknown | mfa-4 last; post-merge reinstall on main | mfa-4 runs regen then the skew test (claim 43); after merge: rebuild, reinstall `.bee/bin/bee`, regen, and drive reflect + cap through that path (`green:live`) |
| Hosts | LOW — a host that upgrades the binary before `bee onboard --apply` is refused with stale doctrine | mfa-1 (self-sufficient refusal) | the refusal alone recovers the agent; re-onboard renders the block and prompt |
| Manifest/regen | LOW — mechanical, refused at cells add if missed | mfa-4 | `.bee/bin/bee dev release-manifest --check` green |

Waves: mfa-1 alone (the data shape every other cell reads) → mfa-2 ∥ mfa-3
(close side; miner side — disjoint files) → mfa-4 (doctrine, prompt, skill,
knowledge, feature map: one regen, one manifest write, last because of the
skew). After merge, on main: rebuild, reinstall the vendored binary, regen,
drive `bee mailbox reflect` and `bee cells cap` once each (`green:live`).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "implement", "classification": "required", "role": "code", "reason": "Rust changes in mailbox, cells, close, digest, backlog, catalog"},
    {"stage": "tests", "classification": "required", "role": "test", "reason": "red-first tests ride each code cell; a test-only repair cell would use this role"},
    {"stage": "doctrine", "classification": "required", "role": "docs", "reason": "D4 homes, promotion.md, knowledge overview, feature map"},
    {"stage": "lookup", "classification": "conditional", "role": "read", "condition": "a worker needs a digest of a file outside its cell", "reason": "read-only gathers"},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "a narrow fact lookup during execution", "reason": "cheap reader"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "no free-form generation stage in this feature"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes bee-reviewing", "reason": "independent review is user-invoked"},
    {"stage": "advisor", "classification": "required", "role": "advisor", "reason": "the plan-step hat wave synthesis is recorded as the advisor ref"},
    {"stage": "plan-hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "plan-step wave seat"},
    {"stage": "plan-hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "planning", "classification": "not-applicable", "role": "plan", "reason": "the leader plans in-session"},
    {"stage": "supervise", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit in this feature"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes: one shape, precedent in repo"},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"}
  ]
}
```

## Shape

Milestone-shaped; one slice carries the whole feature because every locked
decision is a contract change on the same chain and a half-landed chain (a
required flag with no prompt naming it) is worse than none.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — data shape and doors | `fix_at` on entry, letter item, cap trace; `--fix-at` on reflect and cap; report segment; catalog and registry; the cap recipe in the feature map | everything else reads this shape | `bee mailbox reflect --wrong … --better …` is refused by a message that names `--fix-at`, lists the four values and the remedy; with `--fix-at check` it writes a row carrying `"fix_at":"check"` | mfa-2, mfa-3 |
| 2 — close files, miner keys | D2 rows in the green tail with a tail-voice line; D3 key with an honest lesson row | the shape exists | `bee close` prints `Filed for "<feature>": N fix-at row(s) …` and `.bee/backlog.jsonl` gains rows with `layer: fix-at:check`; a re-close adds none | mfa-4 |
| 3 — the three homes and the state layer | AGENTS.block.md (renders AGENTS.md), worker prompt; promotion.md step 1; knowledge overview; close recipe in the feature map; one regen | the flag spelling is final; the prompt edit skews the old binary so it lands last | AGENTS.md's block and the vendored prompt show the three-part line; skew test and manifest check green | merge, then the post-merge reinstall |

## Smaller path check

*Is there a cheaper shape that still honours every locked decision?*

- Optional `--fix-at` defaulting to `none` — FAIL: D1 says required, and
  CONTEXT.md's rejected reading names why.
- File D2 rows at cap time instead of close — FAIL: a cap is worker-run and
  runs per cell; the feature-scoped dedupe and the closing run's own
  reflections (claim 17) exist only at close.
- Skip the miner change and rely on D2 alone — FAIL: D3 is locked, and
  `doctrine` reflections have no other road to a lesson.
- Six cells → four. ADOPTED: the door and the cap are one compile unit
  (their signature change has callers in the cap handler), and the two
  docs cells ran one regen each against the same manifest file — one docs
  cell, last, runs it once.
- A new backlog scanner and writer — ADOPTED the cheaper spelling: fold over
  `read_jsonl` + `is_finding_row`, append with `fsutil::append_jsonl`; only
  the seven-key row builder is lifted.

## Hat wave

Three seats, plan-step, one feature. Decision logged (tag
`plan-hat-wave`, `2ad8669e`). What each changed:

| Seat | Finding acted on | Change |
|---|---|---|
| facts-gaps | mfa-1 changed three signatures whose callers live in the cap handler, the close tests and the digest tests, none in its `files` — its verify could not compile | door and cap merged into mfa-1; the two test-helper callers named as compile-fix files |
| facts-gaps | the prompt edit reds `every_template_carries_the_original_request_block_and_matches_disk`, which no cell ran, and the docs cell had no step that produces the vendored prompt copy | docs cell runs `bee dev regen` first (the copy comes from the on-disk template, claim 41), then that test target; claims 41–44 added |
| facts-gaps | `CAP_FLAGS`, the regen chain, the generated `AGENTS.md` region and "nothing reads `trace.mistakes`" were load-bearing prose with no row | claims 13, 14, 16, 39, 42 added; rows 19 and 29 split into contiguous anchors (23–24, 45–46); row 52 re-anchored to the checkout that holds the store |
| facts-gaps | D2's "every reflection of the closing feature" is narrower than the wording — run-level reflections of earlier sessions carry no feature | D2a decision `d1d83be7` logged (touches D2); the source set named in the plan and the cell |
| facts-gaps | `MistakesAnswer::Recorded`'s new element type was left to the worker | named: a `Mistake { wrong, better, fix_at }` struct |
| alternatives | mfa-5 + mfa-6 each ran regen and rewrote the manifest | one docs cell (mfa-4), last |
| alternatives | the dedupe and the append re-implemented `read_jsonl`, `is_finding_row` and `fsutil::append_jsonl` | mfa-2 reuses them; only `backlog_finding_row` is lifted; `backlog_jsonl_path` becomes `pub(crate)` |
| alternatives | pushing the KEY through `trouble_lines` would make `Shape.verbatim` the key and count the four-word brake on the wrong string | `trouble_lines` returns `(key, verbatim)`; the brake applies to the `what` before the key is built |
| alternatives | the `registry` filter ran none of `registry_contracts.rs` | verify spelled with `--test registry_contracts --test registry_dispatch` (claim 50) |
| alternatives | the prompt edit skews the vendored dispatcher (`prepare.rs:3545`) — mfa-5 as placed would have refused mfa-6's own dispatch | the prompt edit is in the last cell; the post-merge reinstall is named in the plan |
| user-impact | the missing-part refusal would keep saying "two", name the flag, and then tell the agent to pass the two flags it already passed | mfa-1 rewrites the door string: names the missing flag(s), lists the four values, gives the remedy; the registry description says three parts |
| user-impact | the vocabulary has no tie-break ("a known debug-only red" fits three words) | the D4 doctrine line carries one sentence: pick the strongest layer someone would build now; a known, accepted state is `doctrine`; `none` is a one-off nobody would guard |
| user-impact | the close line named no feature, led with the count and gave no next move, and the rows are invisible to `bee backlog rank` | line reworded in the tail's voice with `bee backlog findings --feature <feature>`; slot after the Retired line; the sinking rank recorded as out of scope |
| user-impact | the lesson row would claim "reported the same thing" over sentences that differ after word four, quoting only run A | decision text names the layer and the head; the rationale quotes the other run's sentence |
| user-impact | the `--no-mistakes` gotcha in `cells-and-proof.md` goes false at mfa-1 and would stay false for two waves | that file rides mfa-1 |

Dismissed, with reason:
- alternatives 1 ("keep the split, add compile-fix files") — the cap handler
  edit IS the cap change; a compile-fix that leaves `--mistake` accepting no
  layer would be a cell that violates D1 for one wave. Merged instead.
- user-impact 3's host-side remedy beyond the refusal (a re-onboard nudge in
  the registry drift hint) — outside every locked decision; the
  self-sufficient refusal is the recovery, and re-onboarding is the existing
  upgrade step.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| mfa-1 | Name the fix-at layer on every mistake record, at the verb and at the cap | `verbs/mailbox.rs`, `verbs/cells/handlers_close.rs`, `verbs/cells/finish_support.rs`, `catalog.rs`, `generated/registry_payload.json`, `verbs/drivers/close.rs` (test caller only), `verbs/mailbox_digest.rs` (test helper only), `.bee/verify/verify-app/features/cells-and-proof.md` | — | `bee mailbox reflect` and `bee cells cap --mistake` refuse without `--fix-at`, and the refusal names the flag, the four values and the remedy; a reflection row, its letter item and `trace.mistakes[]` carry `fix_at`; old rows still read; the cap recipe says three parts | `cargo test … -p bee mailbox`, `… handlers_close`, `… finish_support`, `… catalog`, the two registry test targets — green, red-first on the door |
| mfa-2 | File one backlog row per mechanizable mistake at close | `verbs/drivers/close.rs`, `verbs/backlog.rs` | mfa-1 | `bee close` prints `Filed for "<feature>": N fix-at row(s) to .bee/backlog.jsonl (M already there) — bee backlog findings --feature <feature> to read them.`; the rows exist; a second close adds none; an unwritable backlog warns and never refuses | `cargo test … -p bee drivers::close`, `… backlog` green, red-first on the filing and the dedupe |
| mfa-3 | Key the weekly lesson miner by layer and first four words, and say so in the lesson | `verbs/mailbox_digest.rs` | mfa-1 | two runs whose reflections share a layer and a four-word head become one lesson whose decision names the layer and the head and whose rationale quotes the other run; different layers never merge; letters without `fix_at` key as `none` | `cargo test … -p bee mailbox_digest` green, red-first |
| mfa-4 | Teach the three homes, the promotion tree, the knowledge layer and the close recipe the fix-at line | `packages/bee/AGENTS.block.md`, `packages/bee/prompts/worker-cell.md`, `AGENTS.md` (rendered), `.bee/bin/prompts/worker-cell.md` (rendered), `skills/bee-capturing/references/promotion.md`, `docs/knowledge/areas/human-mailbox/overview.md`, `.bee/verify/verify-app/features/worktree-and-close.md`, release manifest | mfa-1, mfa-2, mfa-3 | AGENTS.md's block and the vendored worker prompt show the three-part line, the flag, the four values and the tie-break; promotion.md step 1 names fix-at reflections; the overview states the new key and the close-time row; the close recipe drives the new line | `.bee/bin/bee dev regen` then the prompt-skew test target, the block parity test target and `.bee/bin/bee dev release-manifest --check` green; mirrored skill copies diffed |

```json
[
  {
    "id": "mfa-1",
    "feature": "mistake-fix-at",
    "title": "Name the fix-at layer on every mistake record, at the verb and at the cap",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["af14e1d6-6c5e-4034-b257-0401bd0e0a1f", "5c43da33-0b88-4dab-9faa-891a57a74e24"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs",
      ".bee/verify/verify-app/features/cells-and-proof.md"
    ],
    "read_first": [
      "docs/history/mistake-fix-at/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/human-mailbox/overview.md", ".bee/verify/verify-app/features/cells-and-proof.md"],
    "action": "Give every mistake record its fix-at layer (per D1), at the verb and at the cap, as the data shape the rest of the feature reads. DATA SHAPE, in verbs/mailbox.rs: add one vocabulary constant `pub(crate) const FIX_AT_VALUES: [&str; 4] = [\"architecture\", \"check\", \"doctrine\", \"none\"]` beside `KIND_REFLECTION`; add `pub fix_at: Option<String>` to `Entry` and to `LetterItem` (rg hit, twice: `    pub better: Option<String>,`), emitted AFTER `better` in both `to_value` sites (rg hit, twice: `            \"better\": self.better,`) and in the letter frontmatter (rg hit: `            emit_opt(&mut out, 4, \"better\", item.better.as_deref());`), parsed with `opt_string(m, \"fix_at\")` at both `from_value` sites (rg hit, twice: `            better: opt_string(m, \"better\"),`) so a row or letter without the key reads `None` (per D4: old rows are `none`). Change `Entry::reflection` (rg hit: `    pub(crate) fn reflection(at: &str, wrong: &str, better: &str) -> Self {`) to take `fix_at: &str` and store it; `Entry::to_item` carries it. THE DOOR: extend `read_reflection` (rg hit: `pub(crate) fn read_reflection<'a>(`) to take a third `fix_at: Option<&str>` and return three parts. Rewrite its one refusal string (rg hit: `\"a reflection has two required parts, and it is missing {} (D3) — remedy: pass --wrong (what went wrong) and --better (what would have been better), each with text.\",`) so it is self-sufficient for a cold worker or a host on a new binary with old doctrine: say a reflection has THREE required parts, name exactly the missing flag(s), list the four values `architecture | check | doctrine | none` with a five-word gloss each (architecture: a code change makes it impossible; check: a test, hook, guard or doctor row catches it; doctrine: only prose can carry it; none: a one-off nobody would guard), and give the remedy naming all three flags. A `--fix-at` outside the vocabulary is refused by a second string listing the four values and the remedy. Extend `read_mistake` (rg hit: `pub(crate) fn read_mistake(v: &Value) -> Result<(String, String), String> {`) to read the third part as a third ` — ` segment of the string form (split the first TWO separators only, so a `better` may not itself contain the separator — say so in the refusal) or a `fix_at` key of the object form, returning three parts through the same door. In `run_reflect` (rg hit: `    let entry = Entry::reflection(&now_iso(), wrong, better);`) read `--fix-at` via `mark_flag(&parsed, \"fix-at\")`, pass it through the door, store it, and add `fix_at` to the success JSON. THE CAP, in verbs/cells/handlers_close.rs: add `\"fix-at\"` to `CAP_FLAGS` and bump its size literal (rg hit: `pub(crate) const CAP_FLAGS: [&str; 15] = [`); add `pub(crate) fix_at: Option<String>` to `CapFlags` beside `mistake` (rg hit: `    pub(crate) mistake: Option<String>,`), parsed in `cap_flags_from` with `opt_string_flag(flags, \"fix-at\")` beside the `mistake` read; define `pub(crate) struct Mistake { pub wrong: String, pub better: String, pub fix_at: String }` and make `MistakesAnswer::Recorded(Vec<Mistake>)`; in `read_mistakes_answer` (rg hit: `pub(crate) fn read_mistakes_answer(id: &str, f: &CapFlags, report: &Value) -> MR<MistakesAnswer> {`) build the object form `{wrong, better, fix_at}` from `--mistake` plus `f.fix_at` before handing it to `mailbox::read_mistake`, so a `--mistake` with no `--fix-at` is refused by the door naming `--fix-at`; each report `mistakes` item goes through `read_mistake` as it is; dedupe on all three fields. Write `fix_at` as the third key of each `trace.mistakes` object (rg hit: `                            .map(|(wrong, better)| json!({ \"wrong\": wrong, \"better\": better }))`) and pass it to `mailbox::Entry::reflection` in `record_cap_in_mailbox` (rg hit: `                    &mailbox::Entry::reflection(&utc_now(), wrong, better),`). `cells finish` shares this parse site (`run_cap(finish: bool, …)`), so no second site exists. In verbs/cells/finish_support.rs update the `mistakes` refusal text (rg hit: `must be an array — one entry per mistake`) to spell the three parts `<what went wrong> — <what would have been better> — <fix-at>`. COMPILE-FIX ONLY, no behavior: the test caller in verbs/drivers/close.rs (rg hit: `            &crate::verbs::mailbox::Entry::reflection(`) and the test helper in verbs/mailbox_digest.rs (rg hit: `.map(|(wrong, better)| Entry::reflection(stamp, wrong, better))`) pass `\"none\"`; touch nothing else in those two files. CATALOG AND REGISTRY: in catalog.rs add the `fix-at` flag to `mailbox reflect`, `cells cap` and `cells finish` following the entry shape the existing `no-mistakes` flag uses, and bump `PINNED_FLAG_COUNT` from 210 to 211 with a dated comment in the block above it (rg hit: `        const PINNED_FLAG_COUNT: usize = 210;`) saying the one new name is `fix-at`, reused by all three verbs; `--fix-at` takes a value, so `FLAG_ALONE_BOOLEANS` is untouched. In generated/registry_payload.json (hand-edited here, decision 3358743e): add `fix-at` to the `properties` of `mailbox.reflect`, `cells.cap` and `cells.finish`, its description naming the four values and that it is required with --wrong/--better or --mistake; change the `mailbox.reflect` description's 'A reflection has TWO required parts' to three. FEATURE MAP: in .bee/verify/verify-app/features/cells-and-proof.md, the `--no-mistakes` gotcha's clause `or put a \\`mistakes\\` array in \\`--report\\`` and the `mistakes is an optional sixth` line now say each item has three parts and name `--fix-at`. TESTS, red-first, in mailbox.rs's test module beside `a_reflection_missing_a_part_is_refused_and_the_refusal_names_it`: a reflection missing `--fix-at` is refused and the text names `--fix-at`, contains all four values and the word `remedy`; a value outside the vocabulary is refused listing the four; `read_mistake` takes the three-segment string and the object key; an entry row without `fix_at` reads back `None`; a letter round-trips `fix_at`. In handlers_close.rs beside `a_half_written_mistake_is_refused_by_the_two_part_door`: a cap with `--mistake` and no `--fix-at` is refused naming it; a cap with `--fix-at check` writes `trace.mistakes[0].fix_at == \"check\"` and the mailbox entry carries the same; a report item with only two segments is refused naming the third; an empty `mistakes` array and `--no-mistakes` still record the clean run. Where an existing test asserts the two-part text, extend it to the three-part text rather than deleting it. Do NOT reinstall .bee/bin/bee in this cell: the vendored binary stays the old build until the post-merge step.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee handlers_close && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee finish_support && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee catalog && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts --test registry_dispatch",
    "must_haves": {
      "truths": [
        "bee mailbox reflect --wrong x --better y is refused; the refusal names --fix-at, lists architecture, check, doctrine, none, and states the remedy with all three flags",
        "bee mailbox reflect --wrong x --better y --fix-at check appends an entry row whose last key is fix_at with value check, and the letter item carries it",
        "bee cells cap --mistake \"a — b\" without --fix-at is refused naming --fix-at; with --fix-at check it writes trace.mistakes[0] as {wrong, better, fix_at} and one mailbox reflection entry with fix_at check",
        "a --report mistakes item \"a — b — architecture\" and {\"wrong\":\"a\",\"better\":\"b\",\"fix_at\":\"architecture\"} both record fix_at architecture; a two-segment item is refused naming the third part",
        "an entry row, a letter item and a trace.mistakes object written before this change parse with fix_at None; --no-mistakes and an empty mistakes array are unchanged",
        "PINNED_FLAG_COUNT is 211 and the pin test is green; registry_payload.json lists fix-at on mailbox.reflect, cells.cap and cells.finish and both registry test targets are green",
        "cells-and-proof.md's cap gotchas say three parts and name --fix-at"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/mailbox.rs", "substantive": "FIX_AT_VALUES, fix_at on Entry and LetterItem, three-part read_reflection with a self-sufficient refusal, three-part read_mistake, run_reflect reading --fix-at, tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs", "substantive": "CAP_FLAGS with fix-at, CapFlags.fix_at, Mistake struct, three-part MistakesAnswer, trace.mistakes with fix_at, tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs", "substantive": "three-part refusal text for the mistakes key"},
        {"path": "packages/bee-rs/crates/bee/src/catalog.rs", "substantive": "fix-at declared on the three verbs; PINNED_FLAG_COUNT 211 with reason"},
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "fix-at property on mailbox.reflect, cells.cap, cells.finish; three-part description"},
        {"path": ".bee/verify/verify-app/features/cells-and-proof.md", "substantive": "cap gotchas naming the three parts and --fix-at"}
      ],
      "key_links": [
        "run_reflect and read_mistakes_answer both end at read_reflection — the three-part rule has one home",
        "record_cap_in_mailbox passes fix_at into Entry::reflection; LetterItem.fix_at is emitted after better and parsed back"
      ],
      "prohibitions": [
        "No change to filing, arming, recovery or digest composition; close.rs and mailbox_digest.rs change only at the two named test callers",
        "The letter frontmatter, the entry row and trace.mistakes only grow: every existing key keeps its name, order and shape",
        "The --report contract keeps its five required keys; mistakes stays optional and array-shaped",
        "--no-mistakes keeps its meaning and its refusal when paired with --wrong/--better",
        "No reinstall of .bee/bin/bee"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "mfa-2",
    "feature": "mistake-fix-at",
    "title": "File one backlog row per mechanizable mistake at close",
    "lane": "standard",
    "role": "code",
    "deps": ["mfa-1"],
    "decisions": ["6e79d14e-7961-4a3f-b0a2-c9d9af288c07"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "read_first": [
      "docs/history/mistake-fix-at/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "affects_skills": ["skills/bee-capturing/references/promotion.md"],
    "affects_specs": ["docs/knowledge/areas/human-mailbox/overview.md", ".bee/verify/verify-app/features/worktree-and-close.md"],
    "action": "At close, file one backlog row for every mechanizable mistake of the closing feature (per D2, source set per the D2a decision). In verbs/backlog.rs: make `backlog_jsonl_path` `pub(crate)` (rg hit: `fn backlog_jsonl_path(root: &Path) -> PathBuf {`); lift ONLY the seven-key row builder out of `run_add` (rg hit: `fn run_add(parsed: ParsedArgs, queue_submit: bool, t0: Instant) -> Option<ExitCode> {`) into `pub(crate) fn backlog_finding_row(ty: &str, title: &str, detail: &str, severity: &str, layer: &str, feature: &str) -> Map<String, Value>` that keeps the exact key order and carries its comment (rg hit: `    // Row key order: ts, type, title, detail, severity, layer, feature.`), and have `run_add` call it — the append stays `fsutil::append_jsonl` (rg hit: `    if append_jsonl(&path, &Value::Object(line.clone())).is_err() {`); add `pub(crate) fn backlog_finding_exists(root: &Path, feature: &str, layer: &str, title: &str) -> bool` as a fold over `read_jsonl(&backlog_jsonl_path(root)).rows` filtered by `is_finding_row` (rg hit: `fn is_finding_row(row: &Value) -> bool {`) with exact equality on the three fields. In verbs/drivers/close.rs, in the green tail AFTER the headline (rg hit: `    let headline = format!(\"Tests GREEN for \\\"{feature}\\\" — {}\", proof_door_detail(&proof));`) and BEFORE retirement (rg hit: `    let retired = auto_archive_on_close(root, feature);`): collect the feature's mistakes from two sources — each capped cell's `trace.mistakes` objects, walked with `list_cells_including_archive(root, feature, Some(\"capped\"))` exactly as `mistakes_debt` walks (rg hit: `pub(crate) fn mistakes_debt(root: &Path, feature: &str) -> D<DebtSummary> {`), and the closing run's own reflection entries, resolved the way `closing_run_answered_mistakes` resolves the run and control root (rg hit: `fn closing_run_answered_mistakes(root: &Path) -> bool {`). For each whose `fix_at` is `check` or `architecture`: title = the wrong text truncated to `BACKLOG_MAX_TITLE` characters, detail = the better text plus ` (from cell <id>)` or ` (from run <run>)`, severity `P3`, layer `fix-at:<value>`, type `finding`, feature = the closing feature; skip when `backlog_finding_exists` says so; never file for `doctrine`, `none`, or a missing fix_at. Count filed and skipped; add `fix_at_rows: {\"filed\": N, \"skipped\": M}` to the close result JSON; print ONE text line in the tail's voice, placed right after the Retired line (rg hit: `\"Retired \\\"{feature}\\\": {moved} cell(s) moved out of the active scan (bee cells unarchive --feature {feature} to reverse).\"`): `Filed for \"{feature}\": {N} fix-at row(s) to .bee/backlog.jsonl ({M} already there) — bee backlog findings --feature {feature} to read them.` Fail-open: a write error prints one warning line naming the path and never changes the exit code. TESTS, red-first, in close.rs's test module beside `close_refuses_a_feature_whose_capped_cell_never_answered_about_mistakes` (reuse `init_bee_repo` and the `capped_cell` fixture shape, giving the cells `trace.mistakes` objects with `fix_at`): two capped cells each carrying one `check` mistake → exactly two rows with `layer: fix-at:check`, `type: finding`, `severity: P3`, `feature` set, and the close exits 0 with the Filed line naming the feature; the same close run again → zero new rows and the line says 2 already there; a `doctrine` mistake and a mistake with no `fix_at` file nothing; a read-only .bee/backlog.jsonl → close still exits 0 and prints the warning. Add one backlog.rs test that `backlog_finding_row` yields the seven keys in order and one that `backlog_finding_exists` matches exactly and only on the three fields.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers::close && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee backlog",
    "must_haves": {
      "truths": [
        "a green close of a feature with two check mistakes appends exactly two rows to .bee/backlog.jsonl, each with type finding, severity P3, layer fix-at:check, feature set to the closing feature",
        "the close text carries one line starting Filed for \"<feature>\": that names the count, the path, the already-there count and the bee backlog findings command, placed after the Retired line",
        "running the same close again appends zero rows and reports them as already there",
        "doctrine and none mistakes and mistakes with no fix_at file no row",
        "an unwritable backlog file prints one warning and the close exit code is unchanged",
        "bee backlog add still writes the seven keys in the same order"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "substantive": "fix-at row filing in the green tail before retirement, result field, tail-voice text line, tests"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/backlog.rs", "substantive": "backlog_finding_row lifted out of run_add, backlog_finding_exists fold, pub(crate) backlog_jsonl_path, tests"}
      ],
      "key_links": [
        "close.rs appends with fsutil::append_jsonl through backlog::backlog_finding_row — no second row shape",
        "the walk reuses list_cells_including_archive with Some(\"capped\") so archived cells count"
      ],
      "prohibitions": [
        "No new blocking door: the mistakes door's refusal and remedy text are unchanged",
        "No write before the green tail — a refused close files nothing",
        "The backlog row key order and the four required fields are unchanged"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "mfa-3",
    "feature": "mistake-fix-at",
    "title": "Key the weekly lesson miner by layer and first four words, and say so in the lesson",
    "lane": "standard",
    "role": "code",
    "deps": ["mfa-1"],
    "decisions": ["9173fbd2-4453-476c-9502-d86290ae5513"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs"],
    "read_first": [
      "docs/history/mistake-fix-at/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/human-mailbox/overview.md"],
    "action": "Change what the weekly lesson miner keys a reflection on (per D3), and make the lesson row say what matched. In verbs/mailbox_digest.rs change `trouble_lines` (rg hit: `fn trouble_lines(letter: &Letter) -> Vec<String> {`) to return `Vec<(String, String)>` = (key, verbatim). For a broken-or-unfinished bullet and a departure line, key and verbatim are both the whole line as today. For a reflection item (rg hit: `fn reflection_what(item: &LetterItem) -> Option<&str> {`): apply the four-word brake to the normalized `what` FIRST (rg hit: `const MIN_SHAPE_WORDS: usize = 4;`) and skip a shorter one; then key = `<fix_at> <first four words of normalize_shape(what)>` where `fix_at` is the item's `fix_at` or `none` when the letter carries none (per D4), and verbatim = the whole `what`. In `mine_shapes` (rg hit: `let normalized = normalize_shape(&line);`) normalize and tokenize the KEY, store the first letter's verbatim as `Shape.verbatim`, and add `others: Vec<String>` to `Shape` holding the verbatim of every other run that matched (one per run, letter order); the brake in `mine_shapes` stays for the non-reflection lines. Change `lesson_decision` (rg hit: `fn lesson_decision(shape: &Shape) -> String {`) so a reflection lesson reads `Separate runs reported the same mistake shape (fix-at <layer>, \"<head>…\"): \"<verbatim>\"` and a non-reflection lesson keeps today's text; change `lesson_rationale` (rg hit: `fn lesson_rationale(period: &Period, shape: &Shape) -> String {`) to add, before the letters citation, `Matched on fix-at <layer> + the first four words \"<head>\"; the other letter(s) said: \"<verbatim B>\"[, \"<verbatim C>\"]` for a reflection shape; the `Stable id for this wording: <token>` tail stays, since `spent_tokens` reads it (rg hit: `if word.starts_with(SHAPE_TOKEN_PREFIX) {`). Update the header comment block above `LESSON_TAG` (the 'FOUR WORDS' and 'ONLY trouble' brakes) to state the reflection key. TESTS, red-first, beside `one_reflection_shape_in_two_runs_becomes_exactly_one_cited_lesson` (extend `file_answer_letter` or add a sibling that writes `fix_at` on the item): two runs with reflections `check` + the same four-word head and different tails → exactly one lesson whose decision names `check` and the head and quotes run A's sentence, and whose rationale quotes run B's sentence; the same head under `check` vs `doctrine` → no lesson; two letters without `fix_at` and the same head → one lesson keyed `none`; a reflection whose `what` has three words → skipped even with a layer; a broken bullet still keys on its whole line and `one_broken_shape_in_two_runs_becomes_exactly_one_cited_lesson` stays green. Confirm by test that a token spent by an old whole-sentence lesson row does not block the new key.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee mailbox_digest",
    "must_haves": {
      "truths": [
        "two distinct runs whose reflections share fix_at and the first four normalized words of wrong yield exactly one lesson-tagged decision",
        "that decision names the layer and the four-word head and quotes the first run's sentence; its rationale quotes the other run's sentence and keeps the shape token",
        "the same head under two different fix_at values yields no lesson",
        "letters filed before fix_at existed are keyed as none and still mine",
        "a reflection shorter than four normalized words never becomes a shape, layer or not",
        "broken-or-unfinished bullets and departures keep the whole-line key and their existing tests stay green; the once-ever token rule and the two-run rule are unchanged"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs", "substantive": "trouble_lines returns (key, verbatim); reflection key = fix_at + four-word head with the brake on the what; Shape.others; lesson_decision and lesson_rationale name the key and quote the other run; header comment; tests"}
      ],
      "key_links": [
        "trouble_lines reads LetterItem.fix_at that mfa-1 added — the miner never reads the entries store",
        "spent_tokens still finds the token in the rationale"
      ],
      "prohibitions": [
        "No fuzzy matching, stemming or stop-word removal",
        "The no-mistakes answer is still never mined",
        "Digest composition and filing are untouched"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "mfa-4",
    "feature": "mistake-fix-at",
    "title": "Teach the three homes, the promotion tree, the knowledge layer and the close recipe the fix-at line",
    "lane": "standard",
    "role": "docs",
    "deps": ["mfa-1", "mfa-2", "mfa-3"],
    "decisions": ["5c43da33-0b88-4dab-9faa-891a57a74e24", "402cf686-9c6a-44a0-9f68-95f2592c9cec", "6e79d14e-7961-4a3f-b0a2-c9d9af288c07", "9173fbd2-4453-476c-9502-d86290ae5513"],
    "files": [
      "packages/bee/AGENTS.block.md",
      "packages/bee/prompts/worker-cell.md",
      "AGENTS.md",
      ".bee/bin/prompts/worker-cell.md",
      "skills/bee-capturing/references/promotion.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      ".bee/verify/verify-app/features/worktree-and-close.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/mistake-fix-at/CONTEXT.md",
      "packages/bee/AGENTS.block.md",
      "packages/bee/prompts/worker-cell.md",
      "skills/bee-capturing/references/promotion.md",
      "docs/knowledge/areas/human-mailbox/overview.md"
    ],
    "affects_skills": ["skills/bee-capturing/references/promotion.md"],
    "affects_specs": ["docs/knowledge/areas/human-mailbox/overview.md", ".bee/verify/verify-app/features/worktree-and-close.md"],
    "action": "Land the fix-at instruction in the three homes D4 names through their two SOURCES, then sync the state layer to the behavior mfa-1..mfa-3 landed, with ONE regen. AGENTS.md's bee block (between `<!-- BEE:START -->` and `<!-- BEE:END -->`) is a RENDER of packages/bee/AGENTS.block.md — onboarding reads the template from disk (`agents_block_template: templates_dir.join(\"AGENTS.block.md\")` in onboard/source.rs) — so never hand-edit AGENTS.md. (1) packages/bee/AGENTS.block.md, the bullet starting `- Write a mistake down the MOMENT you notice it` (rg hit: `  memory at the end: \\`bee mailbox reflect --wrong \"<what went wrong>\"`): add `--fix-at <architecture|check|doctrine|none>` to the reflect command; define the four words in one sentence each (architecture: a code change makes the mistake impossible; check: a test, hook, guard or doctor row catches it; doctrine: only prose can carry it; none: a one-off nobody would guard); add the tie-break sentence: pick the strongest layer someone would actually build now — a known and accepted state is `doctrine`, a mistake nobody would guard against is `none`; state that the flag is required; extend the cap spelling to `--mistake \"<wrong> — <better>\" --fix-at <layer>`. (2) packages/bee/prompts/worker-cell.md: in the Result-form paragraph change `mistakes carries one entry per mistake you made, each in TWO parts` to THREE parts with the third named `<fix-at>`, the four values listed and the tie-break in one clause; update the fenced JSON example (rg hit: `\"mistakes\": [\"<what went wrong> — <what would have been better>\", \"...\"]`) to `\"<what went wrong> — <what would have been better> — <fix-at>\"`; update the closing parenthetical's `bee mailbox reflect` spelling to include `--fix-at <layer>`. (3) skills/bee-capturing/references/promotion.md, the Promotion Decision Tree step 1 (rg hit: `1. Seen twice (review finding, user correction, repeated deviation)`): add the closing feature's fix-at reflections as an input — read from the capped cells' trace.mistakes and the run's letter — and state per D5 that a `check`/`architecture` reflection already has its backlog row from close (`bee backlog findings --feature <feature>`), may be taken in-feature as a tiny cell that ships the check (then mark that row done with the reason), and otherwise stands for grooming; a `doctrine` reflection routes through steps 3–4 unchanged; `none` is not promoted. Keep the rest of the tree byte-for-byte. (4) docs/knowledge/areas/human-mailbox/overview.md: extend the `reflection` row of the entry-kind table (rg hit: `| **reflection** |`) with the required fix-at part and its four values; extend rule 15 (rg hit: `15. **Lesson mining reads only trouble**`) to state the reflection key is fix-at plus the first four normalized words of `what`, the brake on the `what`, and that the lesson row names the key and quotes the other run; add one rule after 16 stating the close-time filing of D2 (one P3 `finding` row, layer `fix-at:<value>`, never twice, fail-open, source set per D2a, the Filed line and its command); add decision ids af14e1d6, 6e79d14e, 9173fbd2, 5c43da33, 402cf686 and d1d83be7 (D2a) to the frontmatter decisions list. (5) .bee/verify/verify-app/features/worktree-and-close.md: add the close text line `Filed for \"<feature>\": …` and the backlog rows to the drive recipe and the gotchas. (6) Run `.bee/bin/bee dev regen` ONCE from the worktree root: render-skill-trees mirrors promotion.md into .claude/skills, .agents/skills and .claude-plugin/skills; onboard --apply re-renders the block into AGENTS.md and copies the prompt to .bee/bin/prompts/worker-cell.md from the on-disk template (`read_text_if_exists(&engine.templates_prompts_dir.join(name))` in onboard/plan.rs); release-manifest --write refreshes docs/history/codex-harness-hardening/release-manifest.json. Commit every listed file plus the three mirrored promotion.md copies. Do NOT reinstall .bee/bin/bee: the dispatcher's own copy of the prompt is compiled into the binary (`include_str!` in drivers/prompt.rs), so `bee dispatch prepare` in a checkout whose vendored binary is old will report prompt skew from now until the post-merge reinstall on main — that is expected and named in the plan; this cell is last so no dispatch follows it inside the wave. PROOF: the skew test target `verbs::drivers::tests` (it holds `every_template_carries_the_original_request_block_and_matches_disk`) green, `--test agents_block_render_parity` green, `.bee/bin/bee dev release-manifest --check` green, `diff -q` silent for all three promotion.md mirrors, `rg -q fix-at` on the block, the prompt, promotion.md and overview.md; read back the rendered AGENTS.md bullet and .bee/bin/prompts/worker-cell.md line 65 and quote them in the cap report (`green:live`).",
    "verify": ".bee/bin/bee dev regen && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::drivers::tests && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test agents_block_render_parity && .bee/bin/bee dev release-manifest --check && diff -q skills/bee-capturing/references/promotion.md .claude-plugin/skills/bee-capturing/references/promotion.md && diff -q skills/bee-capturing/references/promotion.md .claude/skills/bee-capturing/references/promotion.md && diff -q skills/bee-capturing/references/promotion.md .agents/skills/bee-capturing/references/promotion.md && rg -q fix-at packages/bee/AGENTS.block.md AGENTS.md packages/bee/prompts/worker-cell.md .bee/bin/prompts/worker-cell.md skills/bee-capturing/references/promotion.md docs/knowledge/areas/human-mailbox/overview.md .bee/verify/verify-app/features/worktree-and-close.md",
    "must_haves": {
      "truths": [
        "packages/bee/AGENTS.block.md's reflect bullet names --fix-at, the four values, the tie-break, and that it is required; AGENTS.md carries the same bullet as its rendered block and the block parity test is green",
        "packages/bee/prompts/worker-cell.md's Result form spells the three-part mistake line and lists the four values; .bee/bin/prompts/worker-cell.md matches it and the skew test is green",
        "promotion.md step 1 names fix-at reflections as an input and states the check/architecture and doctrine routes per D5; the three mirrored copies equal the source",
        "overview.md's reflection row names the required fix-at part, rule 15 names the new key and the honest lesson row, a rule states the close-time row filing, and the decisions list carries the six ids",
        "worktree-and-close.md drives the Filed line",
        ".bee/bin/bee dev release-manifest --check is green after one regen"
      ],
      "artifacts": [
        {"path": "packages/bee/AGENTS.block.md", "substantive": "the reflect bullet with --fix-at, the four values, the tie-break, the cap spelling"},
        {"path": "AGENTS.md", "substantive": "the rendered bee block carrying that bullet — written by regen, never by hand"},
        {"path": "packages/bee/prompts/worker-cell.md", "substantive": "three-part mistakes line in prose and in the JSON example"},
        {"path": ".bee/bin/prompts/worker-cell.md", "substantive": "the regen copy of the prompt"},
        {"path": "skills/bee-capturing/references/promotion.md", "substantive": "step 1 extended with fix-at reflections and the D5 routes"},
        {"path": "docs/knowledge/areas/human-mailbox/overview.md", "substantive": "reflection row, rule 15, the close-filing rule, decisions list"},
        {"path": ".bee/verify/verify-app/features/worktree-and-close.md", "substantive": "close recipe drives the Filed line"}
      ],
      "key_links": [
        "AGENTS.md's block equals the rendered template — regen reports no agents_block drift afterwards",
        "overview.md cites the decision ids so bee close's routing door finds them homed",
        "release-manifest.json is refreshed in the same commit; the skill mirrors are regenerated, never hand-edited"
      ],
      "prohibitions": [
        "No hand edit of AGENTS.md or of any mirrored skill copy",
        "No edit to promotion.md outside step 1",
        "No new knowledge file — one fact, one home",
        "No Rust edits; no reinstall of .bee/bin/bee"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  }
]
```

## Test matrix

Triad per cell; each writer judges existing coverage first and authors only
the gap.

| Cell | Happy path | Edge | Error | Pass when |
|---|---|---|---|---|
| mfa-1 | reflect with `--fix-at check` writes the row; cap with `--mistake` + `--fix-at` writes trace and mailbox | old row, old letter item, old trace object parse with `None`; empty `mistakes` array = clean | missing or bad `--fix-at` refused at the verb and at the cap; two-segment report item refused | refusal text names `--fix-at`, the four values and `remedy`; row's last key is `fix_at`; `trace.mistakes[0].fix_at == "check"` |
| mfa-2 | two check mistakes → two rows and the Filed line | re-close → zero new | unwritable backlog → warning, exit unchanged | `.bee/backlog.jsonl` line count +2 then +0; text `Filed for "demo": 2 fix-at row(s) …` then `… (2 already there)` |
| mfa-3 | same layer + head, two runs → one lesson naming layer and head, quoting both | letters without `fix_at` key as none; three-word `what` skipped | different layers → no lesson | exactly one `lesson`-tagged row; decision contains `fix-at check`; rationale contains run B's sentence and the token |
| mfa-4 | regen renders AGENTS.md and the prompt copy; mirrors equal source | — | skew test and parity test red before regen, green after | `diff -q` silent ×3; manifest check ok; `rg -q fix-at` over the seven files exits 0 |
| existing behavior | reflect/cap/close/miner suites on main vs head | — | — | every pre-existing test either passes unchanged or is extended (never deleted) with its extension named in the cap |

## Test scoping

Each code cell's proof is the release-mode filtered run its `verify` names
(CI mode, per the repo's known debug-only red). The full declared
`commands.test` runs once at wave close before merge, and CI runs it on the
push.

## Open Questions

- (none blocking) Whether `bee cells finish`'s help text needs its own
  sentence for the third part beyond the registry description — decided by
  mfa-1's writer while editing the payload; the cap report names the choice.

## Out of scope

- Backfilling the 465 existing reflections (D4).
- A layer-aware reader for the rows: `bee backlog rank` reads
  `docs/backlog.md`, the feedback digest clusters by whole title and ranks
  P3 rows last, and `bee status`/`bee orient` never mention backlog rows —
  the Filed line's `bee backlog findings --feature <feature>` is the one
  road today. A `bee doctor` row counting stale `fix-at:check` rows stays the
  deferred idea in CONTEXT.md.
- A re-onboard nudge for hosts that upgrade the binary first — the
  self-sufficient refusal is the recovery; `bee onboard --apply` is the
  existing upgrade step.
- Any change to `bee knowledge promote`'s proposal renderer — D5 lands in
  the skill, not the verb.
