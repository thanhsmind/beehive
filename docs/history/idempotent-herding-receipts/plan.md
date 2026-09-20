---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Idempotent Herding Receipts — Slice 1

## Summary

Today a finished background job can be delivered to a session twice, and the
message bee attaches tells the reader to throw the second copy away. That is
right for a true duplicate and wrong for a job that has since produced a real
second answer — the second answer gets discarded. This plan fixes exactly that,
and nothing else.

The other two pieces the feature was shaped for — bee handing back an answer it
already has instead of starting a second worker, and a cleanup command for the
job folders — are **not in this plan**. The hat wave found four questions their
spec cannot answer, and found that the folders are 14 MB, not a problem. Both go
back to shaping with those questions named. See § Out of scope.

Mode: `high-risk` — kept, not earned. Once the deleting slice left this plan the
work is 3 flags (public-contracts, covered-contract-change, multi-domain) over
2 files, which routes `standard`. `bee route --set` refused the demotion:
*"the feature's route is lane 'high-risk' and lane 'standard' would move it off
high-risk; rule violated: high-risk lanes never demote."* **Named deviation:**
the plan is written and gated at high-risk ceremony for a standard-sized slice.
That is the rule working — a lane that was high-risk once does not get to
forget it because its scope shrank — and the extra ceremony is already paid,
since the hat wave ran and is what shrank the scope.

Why this is the least workflow that protects the work: one behavior changes,
one reader sees it, one test pins it. Four of the fourteen claims below were
wrong until the wave re-opened the files.

## Requirements (from CONTEXT.md)

- **D1** (`2eb61b7b`) — Receipts cover every `bee herding run`. Same `job_id` +
  round + same brief returns the stored result and spawns nothing. Delivery
  stays at-least-once; `a05636f8` is not superseded and no exactly-once claim
  is added anywhere.
- **D2** (`2f189c37`) — The replay key is `job_id` + round, never `job_id`
  alone. The drain's injected header gains the round.
- **D3** (`42a59eea`) — Same key, different brief digest → error loudly, name
  the stored receipt path and both digests, spawn nothing.
- **D4** (`adfa3f1c`) — The receipt lives in the existing
  `.bee/mailbox/<job-id>/`; add only a brief digest. A `bee herding prune` verb
  collects finished job mailboxes.

## Load-bearing claims

Labels: `read` = the file was opened at the named line; `ran` = the named
command was executed and its output kept; `guessed` = neither, and no such row
may survive the gate. Match rule: every row below is something the shape breaks
without.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A caller can supply an explicit `job_id`, so D3's conflict case is reachable, not hypothetical | read | `packages/bee-rs/crates/bee/src/herding/run.rs:325` | `"--job-id" => {` … `job_id = flags.get(i + 1).copied();` |
| 2 | Absent the flag, each invocation mints a fresh id, so D1 never fires by accident | read | `packages/bee-rs/crates/bee/src/herding/run.rs:284,411` | `format!("job-{millis}-{pid}-{counter}")` · `job_id.map(str::to_string).unwrap_or_else(allocate_job_id),` |
| 3 | `--continue` reuses the SAME job_id and computes round = prior + 1 — the multi-round case D2 rests on | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3323-3327` | `let prior_round = match mailbox::latest_result_round(&entries) {` … `let next_round = prior_round + 1;` |
| 4 | `execute()` is NOT a funnel — `run()` branches on `no_pane` itself, and the detached path returns earlier still. The only point above every dispatch is in `run()`, above the detach branch | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3944-3963` | `return match spawn_detached_runner(flags, &opts) {` · `let result = if opts.no_pane {` · `execute_no_pane(&opts)` · `} else {` · `execute(&opts, transport.as_ref().unwrap().as_ref())` |
| 5 | The mailbox path helpers are already round-keyed, so a digest path is one more sibling, not a new store (D4) | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:76,92` | `pub(crate) fn brief_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {` · `pub(crate) fn result_path(bee_dir: &Path, job_id: &str, round: u32) -> PathBuf {` |
| 6 | The drain always reads the HIGHEST round present, which is how a round-2 result reaches a session under an already-seen job id | read | `.pi/extensions/bee-guard.ts:720-737` | `function latestResultFile(mailbox: string): string \| null {` … `if (!best \|\| round > best.round) best = { round, file: path.join(mailbox, name) }` |
| 7 | The injected header carries NO round today — the defect D2 names | read | `.pi/extensions/bee-guard.ts:791-798` | `push("job_id", marker.job_id)` · `push("seat", marker.seat)` · `push("cell_id", marker.cell_id)` · `push("status", result.status)` |
| 8 | The header's own sentence instructs the reader to drop the second result | read | `.pi/extensions/bee-guard.ts:802` | `"already handled in this session is a REPLAY, not a second result."` |
| 9 | The existing contract test asserts the at-least-once wording and the redelivery count — D1 keeps at-least-once, so these rows stay green | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:3140,3157` | `run.messages.len(),` `2,` · `run.messages[0].text.contains("at-least-once") && run.messages[0].text.contains("dedupe key"),` |
| 10 | Nothing collects the mailbox tree. Its size is NOT material: 344 dirs / 14 MB in the MAIN checkout, 0 in this worktree — the count is per-checkout and the earlier bare "341" named no checkout | ran | `rg -n "fn .*(prune\|gc\|cleanup\|sweep)" packages/bee-rs/crates/bee/src/herding/mailbox.rs` → no matches; `ls .bee/mailbox \| wc -l` → `0` here, `344` in `/home/thanhsmind/Projects/goglbe/beehive` | (no matches) · `0` · `344` |
| 11 | The herding verb table is one `match` — a new `prune` verb is one arm | read | `packages/bee-rs/crates/bee/src/herding.rs:130-148` | `"interrupt" => Some(job_verbs::interrupt(rest)),` · `"cancel" => Some(job_verbs::cancel(rest)),` |
| 12 | A `--continue` re-creates the marker each round, so the drain sees a fresh marker per round and the marker itself needs no round field | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2296-2302` | `if !opts.dry_run {` · `write_inbox_marker(&opts.main_root.join(".bee"), opts);` |
| 13 | EXACTLY ONE site pins the header row set with `assert_eq!` on a full vec. The other `fenced_rows` call site asserts by containment and is additive-safe | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:3273` (exact) vs `:2786-2794` (containment) | `fenced_rows(text),` under `assert_eq!(` · vs `assert!(rows.contains(&format!("seat: {seat}")), …)` |
| 14 | **RETRACTED.** The usage-limit resume does NOT re-send a different payload: the paused-limit block (`run.rs:3176-3240`) writes no brief — every `brief_path` write is at `:2635` or `:3374`, both outside it — so `brief-N.txt` is unchanged and its digest matches. A paused job also has no `result-N.json` for `resume_round`, so D3's "already has a stored receipt" precondition is unmet twice over | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2635,3374` (the only brief writes) | `let brief_file = mailbox::brief_path(&bee_dir, &opts.job_id, 1);` · `let brief_file = mailbox::brief_path(&bee_dir, job_id, next_round);` |
| 15 | Both transports write the inbox marker, so the marker is not what distinguishes them — only `run()`'s own branch is (claim 4) | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2892-2896` | `pub(super) fn execute_no_pane(opts: &Options) -> ExecResult {` · `if !opts.dry_run {` · `write_inbox_marker(&bee_dir, opts);` |

## Discovery

Claim 12 is the finding that shrank this plan. The first draft added a round
field to the result-inbox marker. Reading `execute()` showed the marker is
rewritten on every non-dry-run invocation, `--continue` included, so the drain
already receives one marker per round and the round is recoverable from the
result filename the drain itself parses (claim 6). The marker change was
deleted; slice 1 is now two files, not three.

This plan was drafted, then audited by the plan-step hat wave. The wave changed
it in four places, and every change came from opening a file rather than from
reasoning about one.

**Claim 4 was wrong twice before it was right.** The first draft put the
pre-flight beside `write_inbox_marker`, reasoning that the `--no-pane` path
returns before it. It does not: `execute_no_pane` writes the marker itself
(claim 15, `run.rs:2895`). The real fact is one level up — `run()` branches on
`no_pane` on its own and the detached path returns earlier still (claim 4,
`run.rs:3944-3963`), so `execute()` is reached only by direct callers, i.e. the
tests. A pre-flight at the top of `execute()` would have passed its own tests
and still spawned both a second native worker and a second detached runner.

**Claim 13 was overstated.** The draft said two tests pin the header row set.
One does (`:3273`); the other asserts by containment (`:2786-2794`) and is
additive-safe. The risk map and test matrix carried the wrong count and are
corrected.

**Claim 14 is retracted, and with it the question that was about to be put to
the user.** The draft read the usage-limit resume as a same-key-different-payload
conflict and escalated it as a CONTEXT amendment to D3. The resume path writes
no brief at all, so the digest cannot differ, and a paused job has no stored
result to conflict with. The escalation was a phantom; it is withdrawn rather
than spent.

**Claim 10's number was not reproducible.** `341` came from the main checkout
and named no checkout; this worktree holds 0. The row now names both, and the
size — 14 MB — is what moved slice 3 out of this plan.

## Approach

**Recommended path.** One slice, one cell. The drain header gains a `round` row
and its dedupe sentence becomes `job_id + round`. Red first: a contract test
drives a round-2 injection under an already-seen job id and fails for the
reported reason before anything is fixed.

**Rejected alternatives.**

- A new receipt store (a `receipts.jsonl` beside the wave ledger) — a
  pass-through over the mailbox dir and a second copy of the truth (claim 5).
- Adding a round field to the result-inbox marker — unnecessary: the drain
  already parses the round out of the result filename (claim 6), and both
  transports rewrite the marker every round anyway (claims 12, 15).
- Making injection exactly-once — would supersede `a05636f8`; CONTEXT D1
  deliberately keeps at-least-once.
- Fixing only the dedupe sentence and adding no `round` row — leaves the reader
  told the key is `job_id + round` with no round to read. D2's defect survives.
- Carrying D1, D3 and D4 in this plan — see § Out of scope. Not a scope
  reduction: the decisions stand, the slices wait, and the questions that make
  them wait are named.

**SMALLER PATH check.** Is there a cheaper shape that still honors D2? No. The
row and the sentence are one change: either alone is incoherent. The one cheaper
variant — sentence only — is rejected above on evidence. PASS, no redraft.

**Materiality, stated honestly.** The `hat-risks` seat counted the live
mailbox: 344 dirs, 310 carrying `result-1.json`, **0 carrying `result-2.json`**.
The multi-round shape has never occurred on disk in this repo, so this fixes a
**latent** defect, not one biting today. It ships anyway because the failure
mode is silent data loss, the fix is two files, and the test is what makes the
shape impossible to regress.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Drain header row set | LOW — exactly ONE site pins the vec with `assert_eq!` (claim 13, `:3273`); the other `fenced_rows` site asserts by containment and is additive-safe | ihr-1 | The new contract test red first, then green, with `:3273` updated and the containment site untouched |
| Dedupe sentence rewording | LOW — claim 9's two assertions pin the substrings `at-least-once` and `dedupe key`, both of which survive; D1 keeps the guarantee | ihr-1 | `a_restart_before_the_turn_settled_redelivers_the_same_job_id_at_least_once` still green |
| Round threading | LOW — the value already exists in `latestResultFile`'s regex capture (claim 6); no new parse | ihr-1 | An unparseable round omits the row and still injects |

Waves: one cell, no parallelism, no serial edge.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "plan-hat-wave", "classification": "required", "role": "advisor", "reason": "High-risk lane: the plan-step hat wave is both the plan check and the gate's advisor consult."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "Fixed hat-wave seat."},
    {"stage": "hat-risks", "classification": "required", "role": "hat-risks", "reason": "Fixed hat-wave seat; this feature deletes files, which is this seat's subject."},
    {"stage": "hat-value", "classification": "required", "role": "hat-value", "reason": "Fixed hat-wave seat."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Fixed hat-wave seat; cites the inline SMALLER PATH check above."},
    {"stage": "hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "Fixed hat-wave seat."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every load-bearing claim was read directly in this repo; no unfamiliar territory and no external source."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "A later slice needs a narrow fact from a known file.", "reason": "Cheap narrow lookups during execution."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "A slice needs a multi-file hunt the leader does not already hold.", "reason": "Mechanical multi-file reads delegate down-tier."},
    {"stage": "cell-execution", "classification": "required", "role": "code", "reason": "Every cell in slice 1 writes Rust or TypeScript and its tests."},
    {"stage": "test-authoring", "classification": "required", "role": "test", "reason": "Slice 1 is red-first; the failing contract test is authored before the fix."},
    {"stage": "docs", "classification": "conditional", "role": "docs", "condition": "Slice 2 or 3 changes a documented CLI contract.", "reason": "config-reference and the herding knowledge area carry the verb table."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The user invokes an independent review.", "reason": "Independent review is user-invoked only, never an automatic stage."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session; no cold observer tick needed."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No release in this feature."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage resolves to a named role above; the fall-through tail is unused."},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "Slice 2 and 3 cells are drafted after slice 1 caps."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape — the repo's own mailbox pattern is the precedent to copy."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "See blind-lane-1."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "See blind-lane-1."}
  ]
}
```

## Shape

**Feature outcome.** A dispatch that already ran is safe to meet twice: its
answer is never silently dropped, never silently duplicated, and its folder does
not live forever.

**Repo-reality basis.** The mailbox already keys every artifact by
`job_id` + round (claim 5), and `execute()` already is the one funnel both
transports pass through (claim 4). Nothing here invents a mechanism; each slice
adds one field or one arm to a structure that already exists.

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| A result is never mistaken for a replay | Delivery correctness — a real answer would be discarded | Claims 6, 7, 8: the drain injects the highest round under a header that names no round, beside a sentence telling the reader to drop it | Slice 1 (this plan) | A round-2 injection carries `round: 2`; red first |
| A dispatch that already ran does not run twice | Duplicate work and conflicting reuse (D1, D3) | Claim 1: a caller can reuse an id | Not planned — back to shaping, § Out of scope | — |
| The mailbox does not grow forever | Retention, and the delete hazard that comes with it (D4) | Claim 10: no collector, 14 MB | Not planned — back to shaping, § Out of scope | — |

**Slice queue.** One slice: the drain names the round. No deps, no successor in
this plan.

**Current slice to prepare: slice 1.**

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ihr-1 | Name the round in the result injection so a later round is not read as a replay | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` | — | A second result for the same job arrives with its round on the header, and the note now says a repeat of the same job **and round** is the replay — so a genuine round-2 answer is no longer discarded | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the new round-2 case red first for the reported reason, then green |

```json
[
  {
    "id": "ihr-1",
    "feature": "idempotent-herding-receipts",
    "title": "Name the round in the result injection so a later round is not read as a replay",
    "lane": "high-risk",
    "role": "code",
    "deps": [],
    "decisions": ["2f189c37", "a05636f8-d308-466c-95a6-80ca3490c585"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/idempotent-herding-receipts/plan.md",
      "docs/history/idempotent-herding-receipts/CONTEXT.md",
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/src/herding/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST, inside this cell: write the failing test before the fix exists, run it, and watch it fail for the REPORTED reason — a round-2 result reaching a session under an already-seen job id with no round on the header. Only then change the belt.\n\nIn packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs add one test to the existing result-inbox drain family (the siblings are at :3079, :3112 and :3231; match their stub-under-node harness exactly — same node subprocess, same stub `pi` object, same stub `.bee/bin/bee`, same named-skip posture via `node_or_skip`). The test drives the sequence the plan's claim 6 and claim 12 describe: a job mailbox holding `result-1.json`, one marker, one injection; then `result-2.json` written and the marker re-created (a `--continue` round, per claim 12 — `write_inbox_marker` runs again on every non-dry-run invocation); then a second drain tick. Assert the second injection carries `round: 2`, NOT `round: 1`, and that the two injections are distinguishable by that row alone.\n\nIn .pi/extensions/bee-guard.ts make exactly two changes. (1) `renderResultInjection` (:781) gains a `round` row. Take the round from the result FILE NAME the drain already parsed — `latestResultFile` (:720) matches `/^result-(\\d+)\\.json$/` and already computes `best.round`, so thread that number through rather than re-deriving it, and never read it from the marker (the marker has no round, claim 12, and adding one was rejected). Place the row directly after `job_id` so job and round read as one key. A result whose round cannot be parsed omits the row and still injects — `headerValue` already drops empty values, so nothing throws (test matrix dimension 2). (2) The data-posture sentence at :802 changes its dedupe key from `job_id` to `job_id + round`: a repeat of the SAME job id AND the SAME round is the replay; a same job id at a HIGHER round is a new result and must not be dropped. Keep the literal substrings \"at-least-once\" and \"dedupe key\" in that sentence — claim 9 shows two existing assertions pin them, and D1 keeps the at-least-once guarantee unchanged.\n\nThen update the ONE exact-row assertion claim 13 names: `fenced_rows(text)` compared with `assert_eq!` on a full vec at :3273, inside the_injected_fence_carries_header_rows_only_never_the_report_body. Add the new row in its new position. Do NOT touch the other `fenced_rows` call site at :2786-2794 — it asserts by containment (`rows.contains(&format!(\"seat: {seat}\"))`), so an added row is already safe there, and editing it would be churn. The plan-step hat wave corrected an earlier draft that claimed two sites; there is one.\n\nDo NOT add a round field to the result-inbox marker, do NOT change `write_inbox_marker` in run.rs, do NOT touch the report body path, and do NOT weaken the at-least-once guarantee or add any exactly-once claim anywhere in code, tests or docs (D1, a05636f8).\n\nREGEN, inside this cell, after the belt edit and before the cap: `.pi/extensions` is a root the release manifest hashes, so run the full chain in order — `.bee/bin/bee dev regen` (render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write) — and commit the regenerated docs/history/codex-harness-hardening/release-manifest.json with the rest. The cell's verify ends in `bee dev release-manifest --check`, which is what proves the manifest matches the shipped extension.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the new test fails first, for the reported reason: the second injection carries no round and is indistinguishable from the first",
        "after the fix, a round-2 result injected under an already-seen job id carries the row `round: 2`",
        "the round is taken from the result filename the drain already parsed, never from the marker",
        "a result whose round cannot be parsed omits the row, injects anyway, and throws nothing",
        "a_restart_before_the_turn_settled_redelivers_the_same_job_id_at_least_once still passes: two messages, and the text still contains both `at-least-once` and `dedupe key`",
        "the_injected_fence_carries_header_rows_only_never_the_report_body passes with the new row set, and still counts exactly two fences",
        "the containment-style fenced_rows site at :2786-2794 is left unedited and still green",
        "bee dev regen ran after the belt edit and the regenerated release manifest is committed with the change",
        "bee dev release-manifest --check passes, so the manifest matches the shipped extension",
        "the dedupe sentence names job_id AND round together as the key",
        "no string anywhere in the diff claims exactly-once delivery"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "renderResultInjection gains the round row threaded from latestResultFile's parsed round; the dedupe sentence names job_id + round"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "the new round-2 drain test, plus the ONE exact-row assertion at :3273 updated to the new row set"},
        {"path": "docs/history/codex-harness-hardening/release-manifest.json", "substantive": "regenerated by bee dev regen so the manifest hash matches the edited belt"}
      ],
      "key_links": [
        "the round value flows from latestResultFile's existing regex capture into renderResultInjection, with no second parse",
        "the round row sits directly after job_id so the two read as one key",
        "the at-least-once and dedupe key substrings survive in the reworded sentence"
      ],
      "prohibitions": [
        "Do not add a round field to the result-inbox marker or change write_inbox_marker",
        "Do not change the at-least-once guarantee or add an exactly-once claim",
        "Do not let the report body ride the injection",
        "Do not touch fn execute, mailbox.rs, or anything belonging to slice 2 or 3"
      ]
    },
    "behavior_change": true
  }
]
```

## Test matrix

High-risk lane: the twelve dimensions, probes written only where the dimension
applies to slice 1. Dimensions with no probe are named with the reason.

| # | Dimension | Probe (slice 1) | Pass when |
|---|---|---|---|
| 1 | User types | n/a — one reader, the orchestrator model | — |
| 2 | Input extremes | A result whose `round` is absent or unparseable (legacy mailbox) | The header omits the row and injects; nothing throws |
| 3 | Timing | Round 2 finishes while round 1's claim is still `.processing` | Exactly one injection per tick; the surviving claim is requeued, not lost |
| 4 | Scale | n/a — one marker per job, one injection per tick, unchanged by this slice | — |
| 5 | State transitions | Crash between injection and `agent_settled`, then restart, with round 2 now on disk | Two messages; the second carries `round: 2`, not `round: 1` |
| 6 | Environment | A repo with no `.bee` directory imports the belt | No timer, no throw, no injection (existing passivity rows stay green) |
| 7 | Error cascades | `sendUserMessage` throws on the round-2 injection | The claim is requeued and the turn survives — the drain stays advisory |
| 8 | Authorization | n/a — no trust boundary is crossed; the header is data, not instructions, and that posture is unchanged | — |
| 9 | Data integrity | Header rows for a result whose `report_path` holds a backtick or a newline | Every value stays one backtick-free line; no fence escape |
| 10 | Integration | The ONE exact-row assertion (claim 13, `:3273`) updated to the new row set; the containment site (`:2786-2794`) left untouched and still green | `fenced_rows` equals the new vec including `round: N`; `at-least-once` and `dedupe key` still present (claim 9); redelivery still counts 2 |
| 11 | Compliance | n/a — no regulated data | — |
| 12 | Business logic | Same job, same round, delivered twice | The reader can still recognise it as a replay — the guarantee is unchanged |

Behavior-change row (required, this cell changes existing behavior): run the
round-2 restart scenario on `main` and on `head`. Pass when main injects a
header with no round under an already-seen job id, and head injects the same
header carrying `round: 2`.

## Verify

`cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
— the repo's declared `commands.test`, matching what CI runs. Scope reason: the
Pi belt's contract suite (`pi_plugin_contracts.rs`) runs the real
`.pi/extensions/bee-guard.ts` under a real node subprocess, so the only way to
prove the belt changed correctly is that manifest's own suite; the three sibling
belt suites live there too and the row-set parity test spans them.

## Open Questions

(none for this slice)

The usage-limit-resume question an earlier draft raised is **withdrawn**, not
deferred: claim 14 is retracted on evidence, and the `hat-risks` seat reached
the same conclusion independently. It was never a CONTEXT amendment.

Every remaining question belongs to the two unplanned slices and is listed with
them below, so none of them is load-bearing here.

## Out of scope

### SPLIT RECOMMENDED — D1, D3 and D4 wait for a second shaping pass

The decisions stand. They are not reduced, reinterpreted, or dropped. They are
not *plannable yet*: the hat wave found four questions their spec cannot answer
and one hazard class with no undo. The user chooses what waits (AGENTS.md,
scope integrity).

**The receipt and its refusal (D1, D3) — four spec gaps:**

1. **What does a replay return to its caller?** D1 says "returns the stored
   receipt and spawns nothing". The spec never says which `RunOutcome`, what
   stdout, what exit code — or whether the inbox marker is still written. If
   the pre-flight returns above `write_inbox_marker` (claims 12, 15), the
   replay writes no marker, the drain never injects, and the session receives
   nothing. That is the opposite of "returns the receipt".
2. **What round is a non-`--continue` re-run?** The key is `job_id` + round,
   but `next_round` is computed only on the continue branch (claim 3) and
   `execute_no_pane` hard-codes round 1. For the exact case D3 exists
   for — an explicitly reused `--job-id` (claim 1) — the round is undefined.
   The `hat-risks` seat showed the cost: a pre-flight that assumes round 1
   refuses **every** `--continue` on every runtime.
3. **Which bytes does the digest cover?** D4 says a digest of `brief-N.txt`.
   But `render_brief` embeds the absolute worktree root and the configured
   proof command, so the same task in a re-created worktree digests
   differently and D3 refuses a legitimate run. Digesting the task and files
   instead is within Agent's Discretion, but it is a real choice and the spec
   does not make it.
4. **Where does a detached refusal become visible?** On an `--inbox-session`
   run the parent prints a success envelope and exits before the child reaches
   any pre-flight (claim 4). A D3 refusal there goes to a stderr nobody reads,
   with no marker written — "refuses loudly" is silent on exactly the runtime
   the drain serves.

**The prune verb (D4) — fails materiality, and its delete has no undo:**

- `.bee/mailbox/` is gitignored. No history, no trash, no undo.
- The plan's own "no terminal result" test is wrong for three live shapes the
  `hat-risks` seat named: finished-but-undelivered, multi-round in flight
  (`result-1.json` + `brief-2.txt` is a live worker), and usage-limit paused.
- `hat-value` scored it a materiality FAIL: 344 dirs, **14 MB**, gitignored,
  causing no problem today. It was also the single reason this feature was
  routed `high-risk`.
- When I asked the user to choose, I gave them the count (341 dirs) but not the
  size. 14 MB is the fact that changes the answer, and they have not seen it.

**Next move for both:** one `bee-shaping` pass that answers the four questions
above and re-decides D4's prune on the 14 MB figure. Not a new feature; a
second pass on this one.

### Never in scope

- Rules 1, 2, 4 and 5 of `docs/history/research/pi-workflows-xia.md` § Five
  rules worth taking — one backlog row, each needing its own shaping.
- Exactly-once injection — would supersede `a05636f8` (CONTEXT D1).
- Any change to cells, gates, proof, worktrees, or the dispatch door's role
  resolution.
