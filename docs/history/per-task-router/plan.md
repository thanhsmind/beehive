---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: per-task-router

## Summary

A new session no longer gets the principles of a feature it never chose.
The leader picks a class for the task in front of it and gets that class's
playbook and principles, like poteto-mode does. Every code change is built by
one worker and then checked by a second, separate tester worker. Workers get
the class principles in their prompt, say which ones they used, and cannot
cap with a weaker proof than their job needs. A bugfix tester must show the
test red before the fix and green after it.

Mode: `standard` — 3 risk flags: public contracts (cap, prepare and orient
output), changes behavior an existing test asserts (tiny inline cap, cap
flags), multi-domain (hooks, dispatch, cells, herding, doctrine)
Why this is the least workflow that protects the work: three verbs and the
always-loaded doctrine change, so it needs a plan and the hat check, but no
hard-gate territory is touched.

## Requirements (from CONTEXT.md)

- D1: implementer and tester are always two separate dispatched workers; a
  code cell never certifies itself; both carry class principles and a
  minimum proof kind.
- D2: implementer first, then an independent tester with no shared context.
- D3: every code-touching cell in every lane, tiny included; a docs-only
  cell gets a second worker for its parity and pointer checks.
- D4: routing per task; the preamble stops printing Route and principles
  from a record the session did not bind.
- D5: (1) playbook steps open the leader's todo list, skips kept with a
  reason; (2) a trigger index in LEADER.md that points, never restates;
  (3) a Principles block in every worker prompt, with a holdout share;
  (4) the cap names applied principles; (5) a minimum proof kind per class,
  enforced at cap.

Planning choices after the hat wave are store decision `58d7b2a7`; the
narrowing of older decisions is `0ada9257`.

## Load-bearing claims

Labels are `read` (opened this session) or `ran` (command run this
session); the evidence column is the bytes found at the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The preamble's knowledge lines fall back to the default record for an unbound session | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:451` | `let Some(feature) = bound else { return default_record.clone() };` |
| 2 | The preamble prints principles from the pipeline record's route | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:679` | `lines.extend(crate::principles::principle_lines(` |
| 3 | `bee orient` prints principles through the same filter | read | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:637` | `let principles = crate::principles::principle_lines(&ctx.root, class);` |
| 4 | The pipeline is `lane` only when the session record names a lane | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:492` | `Some(record) => Pipeline { ok: true, source: "lane", feature: Some(bound), record },` |
| 5 | A second bind path exists: a session bound through `session_binding` with no lane on its record | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:434` | `/// this only ADDS the fallback for the frequent case where it does not` |
| 6 | Existing preamble tests assert the Route line with no session and no lanes | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs:149` | `fn the_route_line_appears_only_for_a_recorded_route() {` |
| 7 | prepare already has a hash-bucket holdout to copy | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1207` | `if u64::from(bucket) < knowledge_holdout_percent(root) { "holdout" } else { "knowledge" }` |
| 8 | The proof result vocabulary is a closed list of three | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:99` | `pub(crate) const PROOF_RESULT_VALUES: [&str; 3] = [` |
| 9 | Cap flags are a struct with no Default, so a new field reaches every literal | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:117` | `#[derive(Clone)]` |
| 10 | The herded cap path builds cap flags itself, with knowledge silence allowed | read | `packages/bee-rs/crates/bee/src/herding/run.rs:6210` | `no_knowledge_used: false,` |
| 11 | Close and merge share one proof check | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2703` | `let proof = crate::verbs::cells::feature_proof_check(root, feature)?;` |
| 12 | Merge calls the same check | read | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:307` | `crate::verbs::cells::feature_proof_check(main_root, feature)` |
| 13 | An existing test asserts a tiny inline cap succeeds | read | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs:7444` | `fn tiny_lane_inline_cap_with_inline_reason_succeeds_and_records_reason() {` |
| 14 | The feature playbook asks for `green:live` at feature level | read | `skills/bee-planning/playbooks/feature.md:30` | `Proof line: the user-visible path driven, with its evidence` |
| 15 | Cap already has a "used, and the decision it changed" flag shape | ran | `bee cells cap --help` | `--knowledge-used (str) — One concept whose content changed a decision in this cell` |
| 16 | A `test` role is configured on the claude runtime | ran | `bee team show --runtime claude --json` | `"description": "author or repair tests, red-first",` |
| 17 | Current doctrine has the writer own its own tests, which D1 replaces | read | `skills/bee-planning/SKILL.md:171` | `The writer owns tests TDD-style as part of each cell` |
| 18 | Current doctrine lets a tiny cell run inline | read | `packages/bee/LEADER.md:217` | `*execution* workers; a tiny cell (one hand-authored file plus mechanical` |
| 19 | The session todo list is a one-way projection; playbook steps can seed it as one | read | `skills/bee-hive/references/routing-and-contracts.md:209` | `is an **ephemeral projection** of those durable records, never the reverse.` |
| 20 | orient's flags are declared in the registry payload | read | `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | `Session-start context packet` |

## Discovery

Read CONTEXT.md's read set, then the preamble resolver
(`hooks/session_preamble/state.rs`), the proof vocabulary
(`verbs/cells/finish_support.rs`), the cap flag struct, the herded cap path
(`herding/run.rs`), the close and merge proof door (`verbs/cells/proof.rs`),
the worker prompt template and the nine class playbooks. The hat wave
(report `docs/history/per-task-router/reports/hat-wave.md`) found that
existing preamble tests model a repo with no lanes (claim 6), that a new cap
field breaks every cap-flag literal (claims 9, 10), and that the three-value
vocabulary cannot express a bugfix's red-then-green (claim 8).

## Approach

Class playbook: `skills/bee-planning/playbooks/feature.md`.

Recommended path. Keep the four layers bee has and move their trigger point.

- D4: one helper decides whether a route belongs to the session: the session
  is bound by either path (claims 4, 5), or the repo has no lane records, so
  the default record is the only record (claim 6 stays green). When it does
  not, the preamble prints one hint line naming `bee orient --class <c>`
  instead of Route and principles. `bee orient` uses the same helper and
  gains `--class <c>`, which prints the class playbook path and that class's
  principles for the task in hand, writing nothing.
- D5 (3): prepare renders a Principles block for the cell's lane route
  class, reusing `principle_lines` and the holdout bucket (claim 7) under its
  own config key. A tester cell hashes its implementer's id, so a pair stays
  in one arm. The prepare record names the arm.
- D5 (4) and (5), one cell: cap gains `--principle-used "<slug> — <decision>"`
  and `--no-principle-used`, required only when the cell's prepare record put
  it in the principles arm, on the native and herded paths alike (claims 9,
  10). The vocabulary gains `green:red-first`. One Rust table holds the
  floors (decision `58d7b2a7` (a)); playbooks point to it.
- D1 to D3: `bee cells add` stamps each new cell for pairing and warns when a
  code-touching cell has no dependent `role: test` cell, or a docs-only cell
  no checker. `feature_proof_check` refuses the feature at close and merge
  (claims 11, 12) when a stamped cell lacks its capped tester or checker.
  Finish refuses an inline cap on a code-touching cell (claim 13 flips).
- Red tester path: a tester never edits product code. On a defect it
  returns `[BLOCKED]` with the red output; the leader reopens the
  implementer cell and dispatches a fresh implementer, whose prompt carries
  the tester's block as a prior round.
- Each implementer cell owns the existing test edits its change forces; its
  tester adds new tests only.
- D5 (1), (2) and the rewrite of claims 17 and 18 land in doctrine; claim 19
  stays, with playbook steps seeding the todo list as a projection.

Rejected alternatives:
- A keyword classifier in Rust for the task class: guesses wrong silently;
  the leader already classifies in its Triage line.
- Floors written only in the playbooks: host repos do not carry the skill
  files at a fixed path the cap could read.
- Pairing refused at `cells add`: breaks every fixture that adds a lone code
  cell and every later single fix cell; close and merge already run one
  shared proof door.
- A same-worker identity check between implementer and tester: each cell is
  its own dispatch, and under herding the leader caps both, so the check
  would fire falsely.
- Gating Route on `source == "lane"` alone: hides the Route for a session
  bound through `session_binding` and for every lane-less repo.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ptr-1 to ptr-5 write Rust."},
    {"stage":"testing","classification":"required","role":"test","reason":"D1: every tester and checker cell runs on the test role, separate from the implementer."},
    {"stage":"documentation","classification":"required","role":"docs","reason":"ptr-6 edits doctrine and playbooks."},
    {"stage":"consolidation","classification":"required","role":"writer","reason":"ptr-7 consolidates knowledge."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a leader lookup needs more than three files","reason":"Cheap reads."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard lane plan-step hat wave."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default hat seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default hat seat."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Five seats only on high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Five seats only on high-risk."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader writes this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release rides this feature."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Gathers use read."},
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

## Risk map

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Route gating helper | MEDIUM | ptr-1 | `the_route_line_appears_only_for_a_recorded_route` (claim 6) stays green, because a lane-less repo keeps its Route |
| Cap flag fields break literals | MEDIUM | ptr-4 | ptr-4 owns every literal site (claims 9, 10); the crate compiles and `cells` tests pass |
| Herded cap refused for silence | MEDIUM | ptr-4 | a herded cap in the holdout arm, or from an older prepare record, is not refused |
| Pairing refusal at close and merge | MEDIUM | ptr-5 | an unstamped older cell is not refused; a stamped code cell with no tester is |
| Tiny inline tests flip | LOW | ptr-5 | ptr-5 rewrites the tests at claim 13 to the refusal |
| Doctrine and rendered trees | LOW | ptr-6 | `bee dev regen` parity and rule index parity green |

Waves: the Rust cells run as one serial chain, because every one compiles
the same crate in the same worktree and a half-written file reds a
sibling's proof: ptr-1, ptr-1t, ptr-2, ptr-2t, ptr-3, ptr-3t, ptr-4,
ptr-4t, ptr-5, ptr-5t. ptr-6 and ptr-6c run beside that chain once ptr-3t
is capped; their files are Markdown only, and ptr-6 owns the one release
manifest regen for both ptr-3 and itself. ptr-7 waits on all.

## Shape

One slice. Six implementer cells, each followed by its own tester or
checker cell (D1, D3), and the writer consolidation cell with its checker.

## Cells — current slice (preview)

All `cargo test` proofs run with `--manifest-path packages/bee-rs/Cargo.toml -p bee`.

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ptr-1 | Show the preamble route only when the session owns it | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs`, `packages/bee-rs/crates/bee/src/principles.rs`, `packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs` | — | a new unbound session in a repo with lanes gets one hint line instead of a stale Route and principles | `cargo test session_preamble — green:unit — touched budget.rs` |
| ptr-1t | Test preamble route ownership | `packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs` | ptr-1 | tests pin both bind paths, the lane-less repo and the hint | `cargo test session_preamble — green:unit — tester for ptr-1` |
| ptr-2 | Route the task in hand with `bee orient --class` | `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json`, `packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs` | ptr-1t | `bee orient --class research` prints the research playbook path and its principles | `cargo test status_full — green:unit — touched orient.rs` |
| ptr-2t | Test orient class routing | `packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs` | ptr-2 | tests pin `--class` and the unowned-route case | `cargo test status_full — green:unit — tester for ptr-2` |
| ptr-3 | Hand every worker the principles of its class | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, `packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs`, `packages/bee/prompts/worker-cell.md`, `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs` | ptr-2t | a worker prompt shows a Principles block; a holdout pair shows none, and the record names the arm | `cargo test drivers — green:unit — touched prepare.rs` |
| ptr-3t | Test the worker Principles block and its holdout | `packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs` | ptr-3 | tests pin the block, the arm and the pair hashing | `cargo test drivers — green:unit — tester for ptr-3` |
| ptr-4 | Make the cap name its principles and meet its proof floor | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`, `packages/bee-rs/crates/bee/src/herding/run.rs`, `packages/bee-rs/crates/bee/src/herding/mailbox.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs`, `packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | ptr-3t | a principles-arm cap without an answer is refused; a bugfix tester capped `green:unit` is refused, naming `green:red-first` | `cargo test cells — green:unit — touched finish_support.rs` |
| ptr-4t | Test principle answers and proof floors | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` | ptr-4 | tests pin each floor, the herded path and the holdout exemption | `cargo test cells — green:unit — tester for ptr-4` |
| ptr-5 | Require a separate tester before close and merge | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs`, `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` | ptr-4t | `bee close` names the code cell with no capped tester; an inline code cap is refused | `cargo test cells — green:unit — touched proof.rs` |
| ptr-5t | Test tester pairing | `packages/bee-rs/crates/bee/src/verbs/cells/tests.rs` | ptr-5 | tests pin the add warning, the close refusal, the docs checker path and the unstamped exemption | `cargo test cells — green:unit — tester for ptr-5` |
| ptr-6 | Teach the leader per-task routing and the two-worker rule | `packages/bee/LEADER.md`, `skills/bee-planning/SKILL.md`, `skills/bee-swarming/SKILL.md`, `skills/bee-hive/references/routing-and-contracts.md`, `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-planning/playbooks/*.md`, the release manifest | ptr-3t | LEADER.md has the trigger index and the per-task route step; planning makes tester cells; playbooks point to the proof floors | `bee dev regen — green:static — parity of rendered trees` |
| ptr-6c | Check doctrine parity and pointers | its report only | ptr-6 | rule index and pointer checks green | `cargo test parity — green:static — checker for ptr-6` |
| ptr-7 | Consolidate per-task-router knowledge | `docs/knowledge/`, `docs/history/per-task-router/delivery.md` | ptr-5t, ptr-6c | knowledge concepts name the new rules | `bee knowledge check — green:static — consolidation` |
| ptr-7c | Check the consolidated knowledge | its report only | ptr-7 | knowledge check green | `bee knowledge check — green:static — checker for ptr-7` |

Packet (authoritative for the preview):

```json
[
 {
  "id": "ptr-1",
  "feature": "per-task-router",
  "title": "Show the preamble route only when the session owns it",
  "lane": "standard",
  "role": "code",
  "deps": [],
  "decisions": [
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
   "packages/bee-rs/crates/bee/src/principles.rs",
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs",
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Add one helper in principles.rs that says whether a route belongs to the session: bound through the session record lane (Pipeline source lane) or through session_binding (budget.rs active_record path), or the repo has no lane records. Gate the Route line and principle_lines in budget.rs on it. When it is false, print one hint line naming `bee orient --class <c>` instead. Update any existing preamble test your change flips; new tests belong to ptr-1t.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session_preamble",
  "must_haves": {
   "truths": [
    "A session bound by either path sees the Route line and principles",
    "A repo with no lane records keeps its default-record Route line",
    "An unbound session in a repo with lanes sees one hint line and no principles"
   ],
   "prohibitions": [
    "No change to which record the knowledge lines read"
   ]
  }
 },
 {
  "id": "ptr-1t",
  "feature": "per-task-router",
  "title": "Test preamble route ownership",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-1"
  ],
  "decisions": [
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "c5c00e89-1653-403f-8924-92d58fcaba18"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
   "packages/bee-rs/crates/bee/src/principles.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Independently verify ptr-1 against its truths. Add tests for: bound via session record, bound via session_binding only, lane-less repo, unbound session in a repo with lanes (hint line, no principles). Run the scoped proof.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee session_preamble",
  "must_haves": {
   "truths": [
    "Each of the four cases has a test that fails if the behavior regresses"
   ],
   "prohibitions": [
    "Never edit product code; only test files listed in files",
    "On a defect return [BLOCKED] with the red output; never weaken an assertion to pass"
   ]
  }
 },
 {
  "id": "ptr-2",
  "feature": "per-task-router",
  "title": "Route the task in hand with bee orient --class",
  "lane": "standard",
  "role": "code",
  "deps": [
   "ptr-1t"
  ],
  "decisions": [
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
   "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/principles.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Use the ptr-1 helper so orient prints principles only for a route the session owns. Add a `--class <c>` flag (declared in registry_payload.json) that prints the class playbook path skills/bee-planning/playbooks/<c>.md and that class's principle lines, writing nothing; an unknown class is refused naming the valid ones. Update existing orient tests your change flips.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full",
  "must_haves": {
   "truths": [
    "bee orient --class research prints the research playbook path and its principles",
    "orient in an unbound session with lanes prints no principles",
    "--class writes no state"
   ],
   "prohibitions": [
    "--class never records a route"
   ]
  }
 },
 {
  "id": "ptr-2t",
  "feature": "per-task-router",
  "title": "Test orient class routing",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-2"
  ],
  "decisions": [
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "c5c00e89-1653-403f-8924-92d58fcaba18"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/status_full/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Independently verify ptr-2. Add tests for --class output, the unknown-class refusal, no state write, and the unowned-route case.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee status_full",
  "must_haves": {
   "truths": [
    "Each ptr-2 truth has a test"
   ],
   "prohibitions": [
    "Never edit product code; only test files listed in files",
    "On a defect return [BLOCKED] with the red output; never weaken an assertion to pass"
   ]
  }
 },
 {
  "id": "ptr-3",
  "feature": "per-task-router",
  "title": "Hand every worker the principles of its class",
  "lane": "standard",
  "role": "code",
  "deps": [
   "ptr-2t"
  ],
  "decisions": [
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "676a73b1-60d8-4063-9def-f466bb4322a2"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
   "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs",
   "packages/bee/prompts/worker-cell.md",
   "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "For kind cell, render a Principles block from principle_lines for the cell's lane route class: names and one-line rules only. Add a holdout under its own config key (default share 10, same hash-bucket pattern as knowledge_holdout_percent). A role test cell hashes the id of its first dependency so a pair shares an arm. Record the arm on the prepare dispatch record as principles_arm. Update existing drivers tests your change flips.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers",
  "must_haves": {
   "truths": [
    "A principles-arm cell prompt carries a Principles block",
    "A holdout-arm prompt carries none",
    "The dispatch record names principles_arm",
    "A tester shares its implementer's arm"
   ],
   "prohibitions": [
    "No change to the Learned context holdout"
   ]
  },
  "regen_obligation_ack": "ptr-6 owns the shared release manifest and runs the full regen chain after this cell; two cells writing one manifest would collide"
 },
 {
  "id": "ptr-3t",
  "feature": "per-task-router",
  "title": "Test the worker Principles block and its holdout",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-3"
  ],
  "decisions": [
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "c5c00e89-1653-403f-8924-92d58fcaba18"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Independently verify ptr-3. Add tests for the block, the holdout arm, the record field and pair hashing.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee drivers",
  "must_haves": {
   "truths": [
    "Each ptr-3 truth has a test"
   ],
   "prohibitions": [
    "Never edit product code; only test files listed in files",
    "On a defect return [BLOCKED] with the red output; never weaken an assertion to pass"
   ]
  }
 },
 {
  "id": "ptr-4",
  "feature": "per-task-router",
  "title": "Make the cap name its principles and meet its proof floor",
  "lane": "standard",
  "role": "code",
  "deps": [
   "ptr-3t"
  ],
  "decisions": [
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "676a73b1-60d8-4063-9def-f466bb4322a2"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
   "packages/bee-rs/crates/bee/src/herding/run.rs",
   "packages/bee-rs/crates/bee/src/herding/mailbox.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs",
   "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
   "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
   "packages/bee-rs/crates/bee/src/herding/run.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Add --principle-used \"<slug> — <decision>\" and --no-principle-used to cap and finish, required only when the cell's prepare record has principles_arm principles; the herded result contract gains an optional principles_used key and run.rs passes it. Add green:red-first (a test seen red on the implementer's base commit and green on head) to PROOF_RESULT_VALUES. Add one floor table per decision 58d7b2a7 (a): implementer code cell green:unit, docs-only cell green:static, role test cell its class floor (bugfix green:red-first, release green:live, feature/refactor/perf green:unit, docs/research/spike/content green:static); static accepts any, unit accepts unit/live/red-first, red-first and live accept only themselves. Docs-only means every path in files ends .md or is under docs/. Refuse a weaker proof naming the floor. Update every CapFlags literal and existing test your change flips.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells",
  "must_haves": {
   "truths": [
    "A principles-arm cap with no answer is refused on native and herded paths",
    "A holdout-arm or older cap is not refused for silence",
    "A bugfix tester capped green:unit is refused naming green:red-first",
    "An implementer code cell capped green:static is refused"
   ],
   "prohibitions": [
    "No change to the knowledge-used obligation"
   ]
  }
 },
 {
  "id": "ptr-4t",
  "feature": "per-task-router",
  "title": "Test principle answers and proof floors",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-4"
  ],
  "decisions": [
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "c5c00e89-1653-403f-8924-92d58fcaba18"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Independently verify ptr-4. Add tests for each floor row, the acceptance ladder, the herded path, and the holdout and older-record exemptions.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells",
  "must_haves": {
   "truths": [
    "Each ptr-4 truth has a test"
   ],
   "prohibitions": [
    "Never edit product code; only test files listed in files",
    "On a defect return [BLOCKED] with the red output; never weaken an assertion to pass"
   ]
  }
 },
 {
  "id": "ptr-5",
  "feature": "per-task-router",
  "title": "Require a separate tester before close and merge",
  "lane": "standard",
  "role": "code",
  "deps": [
   "ptr-4t"
  ],
  "decisions": [
   "676a73b1-60d8-4063-9def-f466bb4322a2",
   "c5c00e89-1653-403f-8924-92d58fcaba18",
   "95e310df-fee8-43a6-8b62-5ce0d9771afb",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "0ada9257-4397-4b11-ab9a-bcf0b857535a"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/cells/validate.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/proof.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
   "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/cells/proof.rs",
   "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
   "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "At cells add, stamp each new cell with tester_rule true and print a warning when a code-touching cell has no dependent role test cell, or a docs-only cell has no dependent checker, in the feature. In feature_proof_check, refuse a feature whose stamped cell lacks a capped dependent tester or checker, naming the cell; unstamped cells are exempt. Refuse an inline cap (--inline-reason) on a code-touching cell. Rewrite the existing tiny inline cap tests your change flips.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells",
  "must_haves": {
   "truths": [
    "bee close and bee worktree merge refuse a stamped code cell with no capped tester",
    "An unstamped cell is not refused",
    "An inline cap on a code-touching cell is refused",
    "cells add warns, never refuses, for a missing tester"
   ],
   "prohibitions": [
    "Never refuse at cells add"
   ]
  }
 },
 {
  "id": "ptr-5t",
  "feature": "per-task-router",
  "title": "Test tester pairing",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-5"
  ],
  "decisions": [
   "676a73b1-60d8-4063-9def-f466bb4322a2",
   "95e310df-fee8-43a6-8b62-5ce0d9771afb",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9"
  ],
  "files": [
   "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee-rs/crates/bee/src/verbs/cells/proof.rs"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Independently verify ptr-5. Add tests for the add warning, the close refusal, the docs checker path, the unstamped exemption and the inline refusal.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee cells",
  "must_haves": {
   "truths": [
    "Each ptr-5 truth has a test"
   ],
   "prohibitions": [
    "Never edit product code; only test files listed in files",
    "On a defect return [BLOCKED] with the red output; never weaken an assertion to pass"
   ]
  }
 },
 {
  "id": "ptr-6",
  "feature": "per-task-router",
  "title": "Teach the leader per-task routing and the two-worker rule",
  "lane": "standard",
  "role": "docs",
  "deps": [
   "ptr-3t"
  ],
  "decisions": [
   "676a73b1-60d8-4063-9def-f466bb4322a2",
   "c5c00e89-1653-403f-8924-92d58fcaba18",
   "95e310df-fee8-43a6-8b62-5ce0d9771afb",
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9",
   "0ada9257-4397-4b11-ab9a-bcf0b857535a"
  ],
  "files": [
   "packages/bee/LEADER.md",
   "skills/bee-planning/SKILL.md",
   "skills/bee-swarming/SKILL.md",
   "skills/bee-hive/references/routing-and-contracts.md",
   "skills/bee-hive/references/gates-and-delegation.md",
   "docs/history/codex-harness-hardening/release-manifest.json",
   "skills/bee-planning/playbooks/bugfix.md",
   "skills/bee-planning/playbooks/content.md",
   "skills/bee-planning/playbooks/docs.md",
   "skills/bee-planning/playbooks/feature.md",
   "skills/bee-planning/playbooks/perf.md",
   "skills/bee-planning/playbooks/refactor.md",
   "skills/bee-planning/playbooks/release.md",
   "skills/bee-planning/playbooks/research.md",
   "skills/bee-planning/playbooks/spike.md"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee/LEADER.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "In packages/bee/LEADER.md: add the per-task route step (classify the task, run bee orient --class <c>, seed the session todo list with that playbook's steps, a skipped step kept with skip: and its reason) and a trigger index of at most ten lines that points from a situation to the owning skill, never restating it; rewrite the agents-never-zero-execution-workers rule so every code cell has a dispatched implementer and a separate tester and tiny no longer runs inline. In bee-planning SKILL.md replace 'The writer owns tests' with tester cells per D1-D3. In bee-swarming SKILL.md state the tester contract and the red-tester path (BLOCKED, leader reopens the implementer, fresh dispatch) and the principle answer at cap. Note in routing-and-contracts.md that playbook steps seed the todo list as a projection. Update the tiny row in gates-and-delegation.md. In each playbook's Proof line, point to the cap floor table (printed by the finish verb's help) without copying values. Run bee dev regen.",
  "verify": ".bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check",
  "must_haves": {
   "truths": [
    "LEADER.md carries the per-task route step and the trigger index",
    "No doctrine line still lets a code cell certify itself or run tiny inline",
    "Each playbook points to the floor table"
   ],
   "prohibitions": [
    "Never restate a skill's description in the trigger index",
    "Never copy floor values into playbooks"
   ]
  }
 },
 {
  "id": "ptr-6c",
  "feature": "per-task-router",
  "title": "Check doctrine parity and pointers",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-6"
  ],
  "decisions": [
   "95e310df-fee8-43a6-8b62-5ce0d9771afb"
  ],
  "files": [
   "docs/history/per-task-router/reports/ptr-6c.md"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "packages/bee/LEADER.md",
   "skills/bee-planning/SKILL.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Run the parity and pointer checks on ptr-6's files: bee dev release-manifest --check, the rule_index_parity and pointer_integrity tests. Write only the check output to your report file; on a red return [BLOCKED] with the output.",
  "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee parity",
  "must_haves": {
   "truths": [
    "Rule index, pointer and manifest checks are green"
   ],
   "prohibitions": [
    "Edit no file except your report"
   ]
  }
 },
 {
  "id": "ptr-7",
  "feature": "per-task-router",
  "title": "Consolidate per-task-router knowledge",
  "lane": "standard",
  "role": "writer",
  "deps": [
   "ptr-5t",
   "ptr-6c"
  ],
  "decisions": [
   "676a73b1-60d8-4063-9def-f466bb4322a2",
   "70c3387f-2c87-4e62-bc1b-546f8ecef03b",
   "eb8842b0-a532-4846-9200-e42db5ade0f9",
   "58d7b2a7-e0da-4b39-9038-58a9325fbad9"
  ],
  "files": [
   "docs/knowledge/",
   "docs/history/per-task-router/delivery.md"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Run bee knowledge consolidate --feature per-task-router, write the deduped knowledge and docs/history/per-task-router/delivery.md.",
  "verify": ".bee/bin/bee knowledge check",
  "must_haves": {
   "truths": [
    "Knowledge concepts name the per-task route, the tester pairing and the proof floors"
   ],
   "prohibitions": []
  }
 },
 {
  "id": "ptr-7c",
  "feature": "per-task-router",
  "title": "Check the consolidated knowledge",
  "lane": "standard",
  "role": "test",
  "deps": [
   "ptr-7"
  ],
  "decisions": [
   "95e310df-fee8-43a6-8b62-5ce0d9771afb"
  ],
  "files": [
   "docs/history/per-task-router/reports/ptr-7c.md"
  ],
  "read_first": [
   "docs/history/per-task-router/CONTEXT.md",
   "docs/history/per-task-router/plan.md",
   "docs/history/per-task-router/reports/hat-wave.md",
   "docs/history/per-task-router/delivery.md"
  ],
  "affects_skills": [],
  "affects_specs": [],
  "action": "Run bee knowledge check. Write only the check output to your report file; on a red return [BLOCKED] with the output.",
  "verify": ".bee/bin/bee knowledge check",
  "must_haves": {
   "truths": [
    "bee knowledge check is green"
   ],
   "prohibitions": [
    "Edit no file except your report"
   ]
  }
 }
]
```

## Test matrix

- Happy: a session bound through its session record shows Route and
  principles. Pass when the preamble contains `- Route: class=`.
- Happy: a session bound only through `session_binding` shows them too.
  Pass when the preamble contains `- Route: class=`.
- Happy: a repo with no lane records keeps its default-record Route. Pass
  when `the_route_line_appears_only_for_a_recorded_route` is green.
- Happy: `bee orient --class research`. Pass when the output names
  `skills/bee-planning/playbooks/research.md`.
- Happy: a bugfix tester capped `green:red-first`. Pass when cap exits 0.
- Edge: an unbound session in a repo with lanes. Pass when the preamble has
  no `- Route:` line, no principles, and one hint line naming
  `bee orient --class`. Runs on main (Route present) and on head (hint).
- Edge: a holdout pair. Pass when neither prompt has a Principles block and
  both records name the holdout arm.
- Edge: a cell added before the upgrade. Pass when close does not refuse it
  for a missing tester.
- Edge: a docs-only cell with a checker. Pass when close accepts the feature.
- Error: a stamped code cell with no capped tester. Pass when `bee close`
  refuses naming the cell.
- Error: a principles-arm cap with no answer, native and herded. Pass when
  refused naming `--principle-used`.
- Error: a bugfix tester capped `green:unit`. Pass when refused naming
  `green:red-first`.
- Error: an inline cap on a code-touching cell. Pass when refused.

## Open Questions

- A fresh read of pstack newer than `b9ddc83` waits on the owner adding the
  path to `guards.cross_project_allow`.

## Out of scope

- A Rust classifier that picks the class from prompt text.
- A `figure-it-out` fallback class.
- Re-porting pstack playbooks already covered by `pstack-playbooks-gap-check`.
