---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Close the mistake loop end to end

## Summary

A mistake now travels the whole loop. The close door asks for an answer
per named cell. Reflections that close never saw are filed at session
start, repeated soft labels are sent back for a new decision, and a
reworded mistake that matches a guarded group reopens it. One ledger
answers "open or closed" for every reader, and the debt shows in the
session preamble, orient, status and doctor. Closing a group needs a
guard command that runs green and the red output it once gave; closed
guards are re-run, and the top group becomes one PBI that the normal
flow builds. knowledge check warns when an evidence_ref names nothing.

Mode: `standard` — risk flags: public contracts (new verbs, a stricter
close door, new status key), multi-domain (backlog, mailbox, close,
preamble, doctor, knowledge). Not high-risk: every write is append-only
and fail-open, and the door change has a printed remedy.

## Requirements (from CONTEXT.md)

- D1: one fix-at ledger, states Open, Closed, Recurred.
- D2: the mistakes door clears only named cells; reflect gains --cell.
- D3: session takeup files unfiled check and architecture reflections as unattributed.
- D4: a doctrine or none key seen in two runs files fix-at:rejudge.
- D5: a reworded mistake matching a Closed group reopens it.
- D6: debt in preamble, orient, status and a doctor advisory; trigger 6e79d14e resolved by it.
- D7: the fix-at close verb needs guard proof; backlog add refuses a hand close on a fix-at layer.
- D8: bee backlog fix-at verify re-runs closed guards.
- D9: fix-at next and promote, takeup auto-promote with a limit of one, upkeep kind fix-at (no new skill: CONTEXT.md D9 text).
- D12: the plan-check refinements (layer-matched closes, kept rejudge holds, one normalization, row-counted advisory, guardless groups skipped, control root).
- D10: knowledge check reads evidence_ref.
- D11: the doctrine names the loop.
- C1 to C6: contracts and their test-name markers.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Anchors are at the feature base.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One closing-run answer clears the whole door | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1605 | `let answered_here = mistakes.count > 0 && closing_run_answered_mistakes(root);` |
| 2 | The closing-run check takes no cell ids | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:302 | `fn closing_run_answered_mistakes(root: &Path) -> bool {` |
| 3 | The open/closed rule lives in feedback | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1729 | `fn fix_at_load(root: &Path) -> Option<FixAtLoad> {` |
| 4 | The same rule lives again in backlog | read | packages/bee-rs/crates/bee/src/verbs/backlog.rs:438 | `pub(crate) fn current_epoch_finding_feature(root: &Path, layer: &str, title: &str) -> Option<String> {` |
| 5 | Manual triggers never auto-fire | read | packages/bee-rs/crates/bee/src/verbs/triggers/mod.rs:42 | `manual`-tier triggers NEVER auto-fire |
| 6 | Upkeep knows two kinds | read | packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs:21 | `enum Kind {` |
| 7 | Every run with entries can be listed | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1037 | `pub(crate) fn runs_with_entries(root: &Path) -> Vec<String> {` |
| 8 | Session takeup composes digests | read | packages/bee-rs/crates/bee/src/verbs/work.rs:311 | `compose_digests_for_finished_periods(&ctx.root);` |
| 9 | The preamble has a droppable block list | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:523 | `const DROPPABLE: [(&str, &str); 4] = [` |
| 10 | Knowledge check reads only bee.evidence | read | packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs:749 | `if let Some(value) = bee.get("evidence") {` |
| 11 | Upkeep skills are hub-served | read | packages/bee-rs/crates/bee/src/onboard/skills.rs:92 | `"bee-verify-upkeep",` |
| 12 | reflect parses a fixed flag list | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2918 | `parse_shape(&rest, &["wrong", "better", "fix-at", "source", "session-id"])?,` |
| 13 | 343 fix-at rows exist | ran | jq over .bee/backlog.jsonl layer counts | 299 fix-at:check, 44 fix-at:architecture |

## Discovery

Four read-only gathers (haiku) mapped the code before this plan: triggers,
doctor and upkeep; the open/closed rule, rank and preamble; mistake intake;
evidence_ref, verb wiring and the owning concepts. Reports under
`.bee/mailbox/job-1795071539598-207303-1/` and its three sibling jobs. The
gathers found the manual-tier law (claim 5), so D6 delivers the trigger's
ask as a doctor advisory instead of changing the law, and found that upkeep
skills are hub-served (claim 11), so D9 carries the loop with verbs.

## Approach

Recommended path: six code cells, each followed by a tester that shares no
context with it, one docs cell with a checker, and the writer cell. The
ledger lands first because every later cell reads it.

After the merge the leader rebuilds and reinstalls .bee/bin/bee, then runs the loop once live: resolves trigger more-than-20-backlog-rows-with-layer-fix__6e79d14e with the doctor advisory named as its outcome (D6, D12), closes the strip_prefix group with the fix-at close verb and the clippy guard as its guard-cmd, runs bee backlog fix-at verify, and runs bee backlog fix-at promote so the top group becomes a Guard PBI.

Rejected: a count predicate for triggers (breaks a stated law for one
trigger); a new bee-fix-at-upkeep skill (outward hub publish); blocking
close until a guard lands (stops every feature for unrelated debt); an
LLM similarity match for recurrence (mailbox D8 forbids judgement in the
miner; a four-word key is deterministic).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"mlc-1 to mlc-6 write Rust."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"mlc-1t to mlc-6t add the new tests, apart from the implementers."},
    {"stage":"docs","classification":"required","role":"docs","reason":"mlc-7 writes the doctrine lines and runs the regen chain."},
    {"stage":"slice-judge","classification":"required","role":"review","reason":"mlc-7c runs the parity and pointer checks on the docs cell."},
    {"stage":"writing","classification":"required","role":"writer","reason":"mlc-8 syncs the concepts and writes delivery.md."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"The four planning gathers."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Default plan-check seat."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default seat; cites the SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default seat; the door, preamble and verbs are user-facing."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Not high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Not high-risk."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release only on the owner's word, through scripts/release.sh."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Risk map

- The stricter door blocks an orchestrator close — MEDIUM — the one fact: the refusal prints both ready commands with the ids filled in — Lands in: mlc-2 — proof: mistakes_door tests (4).
- The ledger changes prose_load numbers — MEDIUM — the one fact: existing feedback tests stay green unchanged — Lands in: mlc-1 — proof: `-- fix_at` and the feedback tests.
- The takeup sweep slows session start — LOW — the one fact: it reads entries files only, about 600 lines today — Lands in: mlc-3 — proof: tester timing note.
- A four-word key merges two different mistakes — LOW — the one fact: it only matches within one layer and only reopens a Closed group, which the next verify or close settles — Lands in: mlc-3 — proof: fix_at_intake (4)(5).
- fix-at close runs a command — LOW — the one fact: the agent writes the command, the same trust as commands.test, with a timeout — Lands in: mlc-5 — proof: fix_at_verbs (1)(2).
- The new preamble block pushes the preamble past budget — LOW — the one fact: the block is DROPPABLE — Lands in: mlc-4 — proof: fix_at_debt (2).

Waves: serial, mlc-1 → mlc-1t → mlc-2 → mlc-2t → mlc-3 → mlc-3t → mlc-4 → mlc-4t → mlc-5 → mlc-5t → mlc-6 → mlc-6t → mlc-7 → mlc-7c → mlc-8 → mlc-9 (test-role checker for the docs cells). Reason: every code cell builds the one bee crate in one worktree (a single scarce resource), and mlc-3 to mlc-5 read the ledger mlc-1 adds.

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature.md`, situation `skills/bee-planning/playbooks/feature/feature.md` (cited, not copied). Step 3 (statechart): the group state machine has three states and one input order (rows by ts); the fold is pure and the tests walk every transition, so no TLC model. Step 8 (formal model): no claim or lock code moves. Architect skipped: the data shape is fixed by D1 (one Group record with a closed state set) and reuses the existing row and tombstone.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| One answer for open or closed | ledger | D1 | 1 | fix_at_ledger |
| Honest intake | door, sweep, rejudge, alias | D2-D5 | 1 | mistakes_door, fix_at_intake |
| Visible debt | preamble, orient, status, doctor | D6 | 1 | fix_at_debt |
| Proof and pull | close, verify, next, promote, upkeep | D7-D9 | 1 | fix_at_verbs |
| Proven evidence | evidence_ref | D10 | 1 | evidence_ref |
| Doctrine | AGENTS block, LEADER, promotion, static rules | D11 | 1 | parity and pointer tests |

PLAN CHECK (hat wave, three seats; reports under `.bee/mailbox/job-1795072073328-232929-1/`, `job-1795072074978-233318-1/` and the user-impact job). BLOCKERS closed: the instruction law pins the hand close line, so mlc-7 owns tests/instruction_laws.rs and its needles now name fix-at close and reflect --cell; the flag-vocabulary pin, so mlc-5 owns catalog.rs; a --kept rejudge refiled at the next sweep, closed by D12 (newer-than-close) and mlc-3t (6); the trigger had no owner, so the leader resolves it after merge (Approach); orient, takeup auto-promote and the verify audit row had no test, now in mlc-4t (5) and mlc-5t (5)(8)(9). WARNINGS applied: layer-matched closes (D12, mlc-1t (7)(8)(9)); one normalization (D12); row-counted advisory (D12); guardless groups skipped by verify (D12); control root for takeup (D12); the door detail carries the commands with --cell last; unique test prefixes mistakes_door_ and evidence_ref_check_ so the green counts are honest; every tester shows each new test red once. NOTES applied: worker-cell.md dropped from mlc-7; third base and exercised in mlc-6t.

SMALLER PATH: doctrine only (tell agents to close rows with proof) keeps the fix in prose, the layer this feature exists to shrink. A doctor row alone shows the debt but pulls nothing. Dropping D5 keeps recurrence exact-title only, which hides failed guards. PASS.

## Cells — current slice (preview)

| id | title | files | deps | role | proof |
|---|---|---|---|---|---|
| mlc-1 | Fold fix-at backlog rows into one ledger | verbs/fix_at.rs, verbs/mod.rs, verbs/feedback.rs, verbs/backlog.rs, verbs/drivers/close.rs | — | code | `cargo test … --bin bee -- fix_at` |
| mlc-1t | Test the fix-at ledger | verbs/fix_at.rs | mlc-1 | test | `cargo test … --bin bee -- fix_at_ledger` |
| mlc-2 | Clear the mistakes door only for named cells | verbs/mailbox.rs, verbs/drivers/close.rs, generated/registry_payload.json | mlc-1t | code | `cargo test … --bin bee -- mistakes` |
| mlc-2t | Test that the mistakes door names cells | verbs/drivers/close.rs, verbs/mailbox.rs | mlc-2 | test | `cargo test … --bin bee -- mistakes_door_` |
| mlc-3 | File missed and repeated mistakes and catch reworded recurrences | verbs/fix_at.rs, verbs/work.rs, verbs/drivers/close.rs | mlc-2t | code | `cargo test … --bin bee -- fix_at` |
| mlc-3t | Test fix-at intake | verbs/fix_at.rs, verbs/drivers/close.rs | mlc-3 | test | `cargo test … --bin bee -- fix_at_intake` |
| mlc-4 | Show fix-at debt in preamble, orient, status and doctor | verbs/fix_at.rs, hooks/session_preamble/budget.rs, verbs/status_full/orient.rs, verbs/status_full/build.rs, doctor.rs | mlc-3t | code | `cargo test … --bin bee -- fix_at` |
| mlc-4t | Test fix-at debt surfaces | verbs/fix_at.rs, hooks/session_preamble/budget.rs, doctor.rs, verbs/status_full/orient.rs, verbs/status_full/build.rs | mlc-4 | test | `cargo test … --bin bee -- fix_at_debt` |
| mlc-5 | Add the fix-at close, verify, next and promote verbs and the upkeep kind | verbs/fix_at.rs, verbs/backlog.rs, generated/registry_payload.json, router.rs, verbs/upkeep_group.rs, audit.rs, verbs/work.rs, catalog.rs | mlc-4t | code | `cargo test … --bin bee -- backlog` |
| mlc-5t | Test the fix-at verbs | verbs/backlog.rs, verbs/upkeep_group.rs, verbs/work.rs | mlc-5 | test | `cargo test … --bin bee -- fix_at_verbs` |
| mlc-6 | Warn when evidence_ref names nothing | verbs/knowledge/check.rs | mlc-5t | code | `cargo test … --bin bee -- knowledge` |
| mlc-6t | Test the evidence_ref checks | verbs/knowledge/tests.rs | mlc-6 | test | `cargo test … --bin bee -- evidence_ref_check_` |
| mlc-7 | Name the closed loop in the doctrine | packages/bee/AGENTS.block.md, packages/bee/LEADER.md, skills/bee-capturing/references/promotion.md, expertise/static-rules.md, packages/bee-rs/crates/bee/tests/instruction_laws.rs, docs/history/codex-harness-hardening/release-manifest.json, AGENTS.md | mlc-6t | docs | `cargo test … --test instruction_laws && /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev release-manifest --check` |
| mlc-7c | Check the doctrine edit | docs/history/mistake-loop-closed/reports/mlc-7c.md | mlc-7 | review | `cargo test … --test pointer_integrity` |
| mlc-8 | Sync the concepts and write the delivery note | docs/knowledge/areas/human-mailbox/overview.md, docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md, docs/knowledge/areas/okf-profile/conformance-check.md, docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md, docs/history/mistake-loop-closed/delivery.md, docs/briefs/mistake-loop-closed.md | mlc-7c | writer | `/mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check` |
| mlc-9 | Check the doctrine and concept sync on the test role | docs/history/mistake-loop-closed/reports/mlc-9.md | mlc-7, mlc-7c, mlc-8 | test | `cargo test … --test instruction_laws` |

```json
[
  {
    "id": "mlc-1",
    "feature": "mistake-loop-closed",
    "title": "Fold fix-at backlog rows into one ledger",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "70b7fc87-8a4f-44bc-955f-b997cd5c42a8",
      "ab99fc6e-0606-420b-8ab1-97ba0ef92ac4",
      "3f2e2a7c-0ac8-4efa-8d03-e742da2db902"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/feedback-digest/"
    ],
    "action": "Implement CONTEXT.md D1. Create verbs/fix_at.rs and register it in verbs/mod.rs. Types: enum GroupState { Open, Closed, Recurred }; struct Group { layer: String, title: String (the newest finding's raw title), key: String (normalize_title), state: GroupState, findings: usize, oldest_open_ts: Option<String>, newest_finding_ts: String, latest_close: Option<serde_json::Value>, features: Vec<String> }; struct Ledger { groups: Vec<Group> } with a pure fn Ledger::from_rows(rows: &[serde_json::Value]) -> Ledger and an IO fn Ledger::load(root: &Path) -> Option<Ledger> that reads .bee/backlog.jsonl the way feedback.rs fix_at_load does. Layers: fix-at:check, fix-at:architecture, fix-at:rejudge. Rules per D12: a close is a row whose type or kind is backlog-closed; a close carrying a fix-at layer matches only its own layer, a close with no fix-at layer matches every layer by normalize_title; a finding is an is_finding_row row on a fix-at layer. A group is Closed when its latest matching close is newer than its newest finding; Recurred when a finding is newer than a close; Open when it never closed. oldest_open_ts is the oldest finding ts in the current epoch (after the latest close). Add Ledger::current_epoch_feature(layer, title) -> Option<String> with the exact semantics of backlog.rs current_epoch_finding_feature, and make that backlog.rs function a thin caller of it (or replace its call in close.rs). Rebuild feedback.rs fix_at_load on Ledger::from_rows, keeping every prose_load key and value identical (fix_at_check_open, fix_at_architecture_open, fix_at_closed count groups as today; Recurred counts as open). Existing tests in feedback.rs, backlog.rs and close.rs must stay green unchanged; if one goes red because of the D12 layer rule, stop and report it. Also run cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- feedback and -- backlog and quote both counts. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at",
    "must_haves": {
      "truths": [
        "Ledger::from_rows is a pure function over backlog rows and returns one group per layer and normalized title",
        "prose_load fix-at counts are unchanged on the existing tests",
        "the close dedupe reads its current-epoch answer from the ledger",
        "the open/closed rule has one home in verbs/fix_at.rs"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "the Ledger type and its pure fold"
        }
      ],
      "key_links": [
        "feedback fix_at_load calls Ledger::from_rows",
        "close or backlog current-epoch check calls the Ledger"
      ],
      "prohibitions": [
        "No new prose_load key or changed value",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "Every later cell needs one answer for open or closed; the rule lives twice today (D1).",
      "out_of_scope": [
        "new prose_load keys",
        "any verb or flag",
        "the preamble"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-1t",
    "feature": "mistake-loop-closed",
    "title": "Test the fix-at ledger",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-1"
    ],
    "decisions": [
      "70b7fc87-8a4f-44bc-955f-b997cd5c42a8",
      "3f2e2a7c-0ac8-4efa-8d03-e742da2db902"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests in verbs/fix_at.rs (a #[cfg(test)] mod) whose names contain fix_at_ledger, per contract fix-at-ledger: (1) two findings with whitespace-variant titles fold into one Open group with findings 2 and the older ts as oldest_open_ts; (2) a backlog-closed row newer than the findings makes the group Closed and carries that row as latest_close; (3) a finding newer than the close makes it Recurred and oldest_open_ts is the post-close finding; (4) check and architecture with one title are two groups; (5) rows of other layers and non-finding types are ignored; (6) current_epoch_feature returns the owning feature only in the current epoch; (7) a row with kind backlog-closed and no type closes like a type row; (8) a close carrying fix-at:check closes the check group and leaves the architecture group of the same title Open, while a layerless close closes both; (9) a non-finding row type on a fix-at layer is not counted. Do not change non-test code; if a test exposes a bug, stop and report it. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_ledger",
    "must_haves": {
      "truths": [
        "nine fix_at_ledger tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "the fix_at_ledger tests"
        }
      ],
      "key_links": [
        "the tests call Ledger::from_rows"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract fix-at-ledger.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-2",
    "feature": "mistake-loop-closed",
    "title": "Clear the mistakes door only for named cells",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "mlc-1t"
    ],
    "decisions": [
      "54375506-e60d-4974-8d4a-06067fdaa2a5",
      "26dc231e-eb87-4db2-bc10-f758b8899838"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/"
    ],
    "action": "Implement CONTEXT.md D2. In verbs/mailbox.rs add cells: Vec<String> to struct Entry, read like files in from_value (absent reads as empty), written in to_value only when non-empty (the same additive rule source uses). bee mailbox reflect gains --cell taking a comma-separated list (add cell to the parse_shape list at the reflect arm); both the reflection and the --no-mistakes branch store it. In verbs/drivers/close.rs replace closing_run_answered_mistakes with a function returning the set of cell ids named by mistakes-answer entries of the closing run; the door is blocking when any debt id is not in that set; the Door command field stays a static string, so the detail and the refusal text carry the unanswered ids and two ready commands with --cell LAST: bee mailbox reflect --wrong <...> --better <...> --fix-at <layer> --cell <ids>, and bee mailbox reflect --no-mistakes --cell <ids>. Update the mailbox.reflect entry in generated/registry_payload.json (parameters and description) for --cell. Update the existing tests the_refusals_own_remedy_clears_the_mistakes_door (pass --cell) and close_refuses_a_feature_whose_capped_cell_never_answered_about_mistakes (only if its text assertion breaks); those two test edits are required by the change. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- mistakes",
    "must_haves": {
      "truths": [
        "a closing-run answer that names cell X clears X only",
        "an answer with no --cell clears no debt cell",
        "the refusal prints the reflect and no-mistakes commands with the unanswered ids",
        "an entry without cells reads and writes exactly as before"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/mailbox.rs",
          "substantive": "the cells field and the --cell flag"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "the door reading named cells"
        }
      ],
      "key_links": [
        "the mistakes door reads the cells named by the closing run's answers"
      ],
      "prohibitions": [
        "No change to mistakes_debt's legacy rule",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "One blanket answer erased the debt of every capped cell (D2).",
      "out_of_scope": [
        "mistakes_debt legacy rule",
        "the fix-at filing at close"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-2t",
    "feature": "mistake-loop-closed",
    "title": "Test that the mistakes door names cells",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-2"
    ],
    "decisions": [
      "54375506-e60d-4974-8d4a-06067fdaa2a5",
      "26dc231e-eb87-4db2-bc10-f758b8899838"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests whose names start with mistakes_door_, per contract mistakes-door-names-cells, in the close.rs and mailbox.rs test modules, reusing init_bee_repo and unanswered_cell: (1) two unanswered cells, a closing-run reflection with --cell naming one: close still refuses and names the other; (2) naming both clears the door; (3) a --no-mistakes answer with no --cell clears nothing; (4) the refusal text contains both printed commands with the ids; (5) an entry round-trips with cells and without (no cells key written when empty); (6) a reflection with no --cell is in the run's entries and in the composed letter, and the door still blocks. The filter mistakes_door_ matches zero tests before this cell. Do not change non-test code; report any bug. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- mistakes_door_",
    "must_haves": {
      "truths": [
        "six mistakes_door_ tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "the mistakes_door tests"
        }
      ],
      "key_links": [
        "the tests drive the close door"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract mistakes-door-names-cells.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-3",
    "feature": "mistake-loop-closed",
    "title": "File missed and repeated mistakes and catch reworded recurrences",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "mlc-2t"
    ],
    "decisions": [
      "53a86f8e-28c8-4b7f-adea-fca7fc73d868",
      "1f70ed97-20e1-4929-bb0b-913eb4b219a4",
      "5c3b98fc-d7b5-48c6-8c90-eb837fd09cb7",
      "ab99fc6e-0606-420b-8ab1-97ba0ef92ac4",
      "9fc55629-04c0-4e36-92ce-45b7fc63ab6c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/"
    ],
    "action": "Implement CONTEXT.md D3, D4, D5 with the D12 refinements. In verbs/fix_at.rs add a layer-free key fn (first four words after mailbox_digest.rs normalize_shape, used on BOTH the mistake side and the group-title side; None under four words) and fn alias_title(ledger, layer, title) -> Filing { New, Skip, UnderClosed(String) }: a four-word key match on an Open or Recurred group of the same layer is Skip, on a Closed group is UnderClosed(that group's title). Make close.rs file_fix_at_rows use it: UnderClosed files under the closed group's exact title with the original wording kept in the detail. Add fn sweep_unfiled(root: &Path) -> SweepReport { filed, rejudged } in fix_at.rs: read every run (mailbox::runs_with_entries, read_entries); for each check or architecture reflection apply D3 (feature unattributed, file only when no current-epoch finding and the entry is newer than the group's latest close, through alias_title); group doctrine and none reflections by the layer-free key and file one fix-at:rejudge finding (title = the earliest wording, detail names the runs) when two or more distinct runs share it, counting only runs whose matching entry is newer than the latest close of any group with that key (so a --kept close holds), and no current-epoch fix-at row of any layer matches the key. Titles are built exactly as close.rs builds them (truncate_chars_head to BACKLOG_MAX_TITLE) so a mistake close already filed is never refiled. The sweep reads entries and reads and writes the backlog through the control root (hooks::session_init::control_root_for), never a worktree root. Rows go through backlog::backlog_finding_row with severity P3 and are appended with fsutil append_jsonl; every error is fail-open. Call sweep_unfiled from verbs/work.rs right after compose_digests_for_finished_periods at the session takeup site. The sweep must be idempotent: a second run files nothing. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at",
    "must_haves": {
      "truths": [
        "an unfiled check reflection is filed once under feature unattributed",
        "a doctrine key seen in two runs files one fix-at:rejudge row",
        "a reworded mistake matching a Closed group files under its title and the group turns Recurred",
        "a second sweep files nothing"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "sweep_unfiled and alias_title"
        }
      ],
      "key_links": [
        "work.rs session takeup calls sweep_unfiled",
        "file_fix_at_rows calls alias_title"
      ],
      "prohibitions": [
        "Never invent a feature other than unattributed",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "Reflections from runs that never closed were lost, soft labels were never re-judged, and reworded recurrences hid failed guards (D3, D4, D5).",
      "out_of_scope": [
        "the lesson miner",
        "the debt surfaces",
        "new verbs"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-3t",
    "feature": "mistake-loop-closed",
    "title": "Test fix-at intake",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-3"
    ],
    "decisions": [
      "53a86f8e-28c8-4b7f-adea-fca7fc73d868",
      "1f70ed97-20e1-4929-bb0b-913eb4b219a4",
      "5c3b98fc-d7b5-48c6-8c90-eb837fd09cb7",
      "9fc55629-04c0-4e36-92ce-45b7fc63ab6c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests whose names contain fix_at_intake, per contract fix-at-intake: (1) a check reflection in a run with no close is filed once as unattributed and a second sweep files nothing; (2) a check reflection older than the group's latest close is not filed; (3) a doctrine reflection key seen in two runs files one fix-at:rejudge row, seen in one run files none; (4) at close, a reworded mistake whose four-word key matches a Closed group files under the closed title and Ledger reports Recurred; (5) a match on an Open group counts as skipped; (6) a rejudge group closed with a kept close row survives a second sweep with nothing filed; (7) an open fix-at:check finding with the same four-word key suppresses the rejudge row; (8) a mistake whose wording is longer than BACKLOG_MAX_TITLE, filed at close, is not refiled by the sweep; (9) the sweep files a reworded mistake under a Closed group's title and the detail keeps the original wording. Build mailbox entries through the mailbox API, not by hand. Do not change non-test code; report any bug. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_intake",
    "must_haves": {
      "truths": [
        "nine fix_at_intake tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "the fix_at_intake tests"
        }
      ],
      "key_links": [
        "the tests call sweep_unfiled"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract fix-at-intake.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-4",
    "feature": "mistake-loop-closed",
    "title": "Show fix-at debt in preamble, orient, status and doctor",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "mlc-3t"
    ],
    "decisions": [
      "b62e676b-5d97-4f2d-ab0e-94cafcdcb0c7",
      "ab99fc6e-0606-420b-8ab1-97ba0ef92ac4",
      "ffbc76fb-c745-4bf5-b1b3-8d2720d7c174"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/build.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/"
    ],
    "action": "Implement CONTEXT.md D6. In verbs/fix_at.rs add struct Debt { check_open, architecture_open, rejudge_open, recurred, oldest_open_days: Option<u64> (from oldest_open_ts), stale_open_rows: usize (open finding rows whose own ts is more than 30 days before now, per D12) } and fn debt(root, now) -> Option<Debt> from the Ledger, reading through the control root. Preamble (budget.rs, after the capture queue block): when any group is open, push '### Fix-at debt: N open (check X, architecture Y, rejudge Z; R recurred), oldest D days' and one bullet '- next: bee backlog fix-at next, then bee backlog fix-at promote'; make it droppable under budget pressure: add it to DROPPABLE (grow the array) with the fetch target 'bee backlog fix-at next' for the Omitted line, and push its span into blocks like the other droppable blocks. Orient: one report-only blocker in the same voice as the promote-proposal blocker. bee status --json: key fix_at_debt with the Debt fields, null when no store. Doctor: a non-voting advisory (the advisory pattern beside owner_briefs_advisory) that warns when stale_open_rows > 20 and names bee backlog fix-at next. All reads fail open. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at",
    "must_haves": {
      "truths": [
        "the preamble shows one fix-at debt block when debt exists and none when it does not",
        "orient, status --json and doctor report the same counts from fix_at::debt",
        "the doctor advisory never changes the overall doctor status"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "the Debt summary"
        }
      ],
      "key_links": [
        "budget.rs, orient.rs, build.rs and doctor.rs read fix_at::debt"
      ],
      "prohibitions": [
        "The preamble stays within PREAMBLE_BUDGET_BYTES",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "343 rows sat because nothing showed the debt (D6).",
      "out_of_scope": [
        "new verbs",
        "trigger records"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-4t",
    "feature": "mistake-loop-closed",
    "title": "Test fix-at debt surfaces",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-4"
    ],
    "decisions": [
      "b62e676b-5d97-4f2d-ab0e-94cafcdcb0c7",
      "ffbc76fb-c745-4bf5-b1b3-8d2720d7c174"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/build.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests whose names contain fix_at_debt, per contract fix-at-debt (put each in the test module of the file it drives): (1) debt counts per layer, recurred and oldest days from a fixture backlog; (2) the preamble carries the block with those numbers, and no block on an empty store; (3) the doctor advisory warns at 21 stale open rows and not at 20, and overall status is unchanged; (4) status --json fix_at_debt equals fix_at::debt; (5) orient's blockers carry the fix-at line with debt and not without; (6) under a forced small budget the block is dropped and the Omitted line names bee backlog fix-at next; (7) a row 31 days old is stale and a row 29 days old is not. Do not change non-test code; report any bug. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_debt",
    "must_haves": {
      "truths": [
        "seven fix_at_debt tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
          "substantive": "the fix_at_debt tests"
        }
      ],
      "key_links": [
        "the tests call fix_at::debt"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract fix-at-debt.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-5",
    "feature": "mistake-loop-closed",
    "title": "Add the fix-at close, verify, next and promote verbs and the upkeep kind",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "mlc-4t"
    ],
    "decisions": [
      "d4eeb160-64e1-4605-86aa-da3ac8dc9d6c",
      "218f08e2-348a-4c12-8d0b-7e8ebdb3a57d",
      "925dc9db-8ea5-48e3-9b24-f95125ef28ab",
      "925dc9db-8ea5-48e3-9b24-f95125ef28ab",
      "ab99fc6e-0606-420b-8ab1-97ba0ef92ac4",
      "a7f3fbd2-ee19-4d08-adb0-c1b4c33d445a"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/fix_at.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/router.rs",
      "packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs",
      "packages/bee-rs/crates/bee/src/audit.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/router.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/feedback-digest/"
    ],
    "action": "Implement CONTEXT.md D7, D8, D9 and the D9 refinement. bee backlog fix-at <close|verify|next|promote> dispatched from backlog.rs try_native (a fix-at arm with a sub-match, like pbi). close: flags layer, title, guard, guard-cmd, red-proof, commit, kept; refuses unknown or already Closed groups, empty guard/guard-cmd/red-proof (kept replaces all three only for fix-at:rejudge); runs guard-cmd with sh -c from the repo root with a 600 s timeout and refuses on non-zero exit, quoting the tail of its output; on success appends a backlog-closed row {ts,type,title,detail,severity P3,layer,feature,guard,guard_cmd,red_proof,commit} via a new row builder (backlog_finding_row keeps its seven keys). bee backlog add refuses --type backlog-closed when --layer starts with fix-at: and names the fix-at close verb. verify: skips a Closed group with no guard_cmd and lists it as unverifiable (a kept rejudge, a legacy hand close); re-runs every other Closed group's guard_cmd; a non-zero exit appends a finding row (same layer and title, detail 'guard went red: <cmd>') and the verb exits 1; it logs one audit row of kind fix-at-upkeep (add the kind to audit.rs KINDS). next: prints the top group (Recurred, then findings desc, then oldest) with --json. promote: files one proposed PBI titled 'Guard: <title>' through the same writer the pbi add verb uses, refusing while a PBI whose title starts with 'Guard: ' is proposed or in flight. Session takeup in work.rs runs promote silently after sweep_unfiled when debt exists. Session takeup reads and writes through the control root. upkeep_group.rs gains Kind::FixAt (name fix-at, audit kind fix-at-upkeep, JSON item skill null and invoke 'bee backlog fix-at verify, then bee backlog fix-at promote', text run line the same command pair, present when the ledger has any group, due when a Closed group exists and no fix-at-upkeep row is younger than 7 days); extend the --kind parse and refusal. Add registry_payload.json entries backlog.fix-at.close, backlog.fix-at.verify, backlog.fix-at.next, backlog.fix-at.promote with schemas and examples, and the router shape strings. The new flag names (guard, guard-cmd, red-proof, commit, kept) grow the pinned flag vocabulary: bump PINNED_FLAG_COUNT in catalog.rs by the number of truly new names after the reuse check the test demands, and put the reason in the commit body. Also run cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_contracts, --bin bee -- distinct_flag_vocabulary and --bin bee -- upkeep, and quote each count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- backlog",
    "must_haves": {
      "truths": [
        "fix-at close refuses without guard proof or on a red guard-cmd and appends the proof keys on success",
        "bee backlog add refuses backlog-closed on a fix-at layer",
        "fix-at verify reopens a red guard as Recurred and logs a fix-at-upkeep audit row",
        "fix-at promote keeps at most one Guard PBI open",
        "bee upkeep due reports kind fix-at"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
          "substantive": "the fix-at sub-verbs"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
          "substantive": "the four registry entries"
        }
      ],
      "key_links": [
        "try_native dispatches fix-at",
        "upkeep_group.rs reads the fix-at ledger"
      ],
      "prohibitions": [
        "backlog_finding_row keeps its seven keys",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A free-text close proved nothing and nothing pulled the next group (D7, D8, D9).",
      "out_of_scope": [
        "the mistakes door",
        "knowledge check",
        "a new skill"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-5t",
    "feature": "mistake-loop-closed",
    "title": "Test the fix-at verbs",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-5"
    ],
    "decisions": [
      "d4eeb160-64e1-4605-86aa-da3ac8dc9d6c",
      "218f08e2-348a-4c12-8d0b-7e8ebdb3a57d",
      "925dc9db-8ea5-48e3-9b24-f95125ef28ab",
      "a7f3fbd2-ee19-4d08-adb0-c1b4c33d445a"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
      "packages/bee-rs/crates/bee/src/verbs/upkeep_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/work.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests whose names contain fix_at_verbs, per contract fix-at-verbs: (1) close refuses with an empty --guard, no --guard-cmd, an empty --red-proof, a guard-cmd 'false', an unknown group, an already Closed group, and --kept on a non-rejudge layer; (2) close with guard-cmd 'test -f .bee/backlog.jsonl' (proving it runs from the repo root) appends one backlog-closed row carrying guard, guard_cmd, red_proof and commit, and the group turns Closed; a Recurred group can be closed; (3) rejudge closes with --kept and no command; (4) the backlog add verb with type backlog-closed and layer fix-at:check refuses and names fix-at close; (5) verify with a Closed group whose guard_cmd is 'false' appends a finding, exits 1, the group is Recurred, and one fix-at-upkeep audit row is written; a kept rejudge group is listed unverifiable and stays Closed; (6) next orders Recurred first, then more findings, then older; (7) promote files one Guard PBI, refuses a second while it is proposed and while it is in flight, and files nothing with no debt; (8) upkeep due --kind fix-at is due with a Closed group and no audit row, and not due after verify logged one; (9) session takeup with open debt files exactly one Guard PBI, a second takeup files none, and no debt files none. Do not change non-test code; report any bug. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_verbs",
    "must_haves": {
      "truths": [
        "nine fix_at_verbs tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
          "substantive": "the fix_at_verbs tests"
        }
      ],
      "key_links": [
        "the tests drive the fix-at sub-verbs"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract fix-at-verbs.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-6",
    "feature": "mistake-loop-closed",
    "title": "Warn when evidence_ref names nothing",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "mlc-5t"
    ],
    "decisions": [
      "e5cfaedd-9d2b-4448-bfce-a738a3b43cda",
      "5837f5d2-5a95-491f-b79a-927908e2980b"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/okf-profile/"
    ],
    "action": "Implement CONTEXT.md D10 in verbs/knowledge/check.rs beside the invalid_evidence_state block: read bee.evidence_ref; split on whitespace, ';', '(' and ')'; strip quotes, backticks, trailing ':' ',' and a '#fragment'; a token with a '/' and a file extension that exists under none of repo_root, repo_root/packages/bee-rs/crates/bee/src, repo_root/packages/bee-rs/crates/bee warns evidence_ref_missing_path naming the token; when bee.evidence is wired or exercised and evidence_ref is absent or empty, warn evidence_without_ref. Skip both when repo_root is None. Warnings only. Then run /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check in the worktree and report the count of each new code over docs/knowledge. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- knowledge",
    "must_haves": {
      "truths": [
        "a missing path token warns evidence_ref_missing_path",
        "an existing crate-relative path is silent",
        "wired with no evidence_ref warns evidence_without_ref"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
          "substantive": "the evidence_ref checks"
        }
      ],
      "key_links": [
        "check_bundle pushes the two new warning codes"
      ],
      "prohibitions": [
        "Never an error outside --strict",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A wired evidence rung could name nothing (D10).",
      "out_of_scope": [
        "bee.evidence values",
        "the backlog"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "mlc-6t",
    "feature": "mistake-loop-closed",
    "title": "Test the evidence_ref checks",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-6"
    ],
    "decisions": [
      "e5cfaedd-9d2b-4448-bfce-a738a3b43cda",
      "5837f5d2-5a95-491f-b79a-927908e2980b"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add tests whose names start with evidence_ref_check_, per contract evidence-ref-check, beside invalid_evidence_state_warns_by_file_and_value_never_errors using bundle, Cx and put: (1) a missing packages/ path warns evidence_ref_missing_path; (2) a crate-relative verbs/... path that exists under the src base is silent; (3) a path that exists under packages/bee-rs/crates/bee (the third base) is silent; (4) a value with two paths separated by ';' warns only for the missing one; (5) wired with no evidence_ref warns evidence_without_ref, and so does exercised; (6) prose with no path is silent; (7) --strict fails on the warning. The filter evidence_ref_check_ matches zero tests before this cell. Do not change non-test code; report any bug. Before capping, prove each new test can fail: break the guarded line of non-test code locally, run the filter, quote the red output, restore the line exactly (git diff of non-test files must be empty), then run the filter green and quote the count. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- evidence_ref_check_",
    "must_haves": {
      "truths": [
        "seven evidence_ref_check_ tests pass and each was shown red once",
        "no non-test code changed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/knowledge/tests.rs",
          "substantive": "the evidence_ref tests"
        }
      ],
      "key_links": [
        "the tests call check_bundle"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A tester apart from the implementer pins contract evidence-ref-check.",
      "out_of_scope": [
        "non-test code"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-7",
    "feature": "mistake-loop-closed",
    "title": "Name the closed loop in the doctrine",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [
      "mlc-6t"
    ],
    "decisions": [
      "3fd971c2-8849-4760-86a3-f5846b463dca",
      "54375506-e60d-4974-8d4a-06067fdaa2a5",
      "d4eeb160-64e1-4605-86aa-da3ac8dc9d6c",
      "925dc9db-8ea5-48e3-9b24-f95125ef28ab"
    ],
    "files": [
      "packages/bee/AGENTS.block.md",
      "packages/bee/LEADER.md",
      "skills/bee-capturing/references/promotion.md",
      "expertise/static-rules.md",
      "packages/bee-rs/crates/bee/tests/instruction_laws.rs",
      "docs/history/codex-harness-hardening/release-manifest.json",
      "AGENTS.md"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee/AGENTS.block.md",
      "packages/bee/LEADER.md",
      "skills/bee-capturing/references/promotion.md",
      "expertise/static-rules.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement CONTEXT.md D11. In packages/bee/LEADER.md, extend the existing mistake bullet: a closing session answers for named cells with bee mailbox reflect --cell <ids>, and the door clears only the cells named. In packages/bee/AGENTS.block.md ADD one short bullet saying the same (no mistake bullet exists there today). In expertise/static-rules.md ('Show it red, then green'): replace the hand-written bee backlog add --type backlog-closed line with bee backlog fix-at close --layer <l> --title <t> --guard <what> --guard-cmd <command> --red-proof <red output> --commit <sha>. In skills/bee-capturing/references/promotion.md name the fix-at close verb with its proof flags where the tree points at closing a row, and name bee backlog fix-at next and promote as the way to pick the next group. In tests/instruction_laws.rs (fix_at_close_missing and its law near line 1414) change the needles from --type backlog-closed to the fix-at close verb, --guard-cmd and --red-proof, and add one needle for reflect --cell in packages/bee/AGENTS.block.md, so the law turns red when the doctrine is missing. Keep every other line byte for byte. Then run /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen once from the worktree root; restore any hub-served skill tree it re-renders under .claude/skills with git restore and git clean -fd .claude/skills before committing. Proof: cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws, --test agents_block_render_parity and --test pointer_integrity. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws && /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "the doctrine names reflect --cell and the fix-at close verb with its proof flags",
        "no doctrine line still tells an agent to append backlog-closed by hand on a fix-at layer",
        "the instruction law needles name fix-at close, --guard-cmd, --red-proof and reflect --cell"
      ],
      "artifacts": [
        {
          "path": "skills/bee-capturing/references/promotion.md",
          "substantive": "the fix-at close step"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/instruction_laws.rs",
          "substantive": "the updated law needles"
        }
      ],
      "key_links": [
        "AGENTS.md renders from packages/bee/AGENTS.block.md"
      ],
      "prohibitions": [
        "Never hand-edit AGENTS.md or .bee/LEADER.md",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A verb no doctrine names is unreachable (D11).",
      "out_of_scope": [
        "Rust code",
        "hub-served skills"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-7c",
    "feature": "mistake-loop-closed",
    "title": "Check the doctrine edit",
    "lane": "standard",
    "role": "review",
    "status": "open",
    "deps": [
      "mlc-7"
    ],
    "decisions": [
      "3fd971c2-8849-4760-86a3-f5846b463dca",
      "a7f3fbd2-ee19-4d08-adb0-c1b4c33d445a"
    ],
    "files": [
      "docs/history/mistake-loop-closed/reports/mlc-7c.md"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "packages/bee/AGENTS.block.md",
      "skills/bee-capturing/references/promotion.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Checker for docs cell mlc-7. Edit no file except docs/history/mistake-loop-closed/reports/mlc-7c.md. Run cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity, --test rule_index_parity, --test agents_block_render_parity, --test instruction_laws, and /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev release-manifest --check; then the content checks: rg -n 'reflect --cell' packages/bee/LEADER.md packages/bee/AGENTS.block.md AGENTS.md (each file must match), rg -n 'fix-at close' expertise/static-rules.md skills/bee-capturing/references/promotion.md (both must match), rg -n 'type backlog-closed' expertise/static-rules.md (must match nothing). Record each command with its result in the report; return BLOCKED with the output on any red. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity",
    "must_haves": {
      "truths": [
        "every parity and pointer check is recorded green in the report"
      ],
      "artifacts": [
        {
          "path": "docs/history/mistake-loop-closed/reports/mlc-7c.md",
          "substantive": "the check report"
        }
      ],
      "key_links": [
        "the report names each command"
      ],
      "prohibitions": [
        "Edit only the report",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "A docs cell needs a second worker for its parity and pointer checks.",
      "out_of_scope": [
        "every file except its report"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-8",
    "feature": "mistake-loop-closed",
    "title": "Sync the concepts and write the delivery note",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "mlc-7c"
    ],
    "decisions": [
      "70b7fc87-8a4f-44bc-955f-b997cd5c42a8",
      "54375506-e60d-4974-8d4a-06067fdaa2a5",
      "53a86f8e-28c8-4b7f-adea-fca7fc73d868",
      "1f70ed97-20e1-4929-bb0b-913eb4b219a4",
      "5c3b98fc-d7b5-48c6-8c90-eb837fd09cb7",
      "b62e676b-5d97-4f2d-ab0e-94cafcdcb0c7",
      "d4eeb160-64e1-4605-86aa-da3ac8dc9d6c",
      "218f08e2-348a-4c12-8d0b-7e8ebdb3a57d",
      "925dc9db-8ea5-48e3-9b24-f95125ef28ab",
      "e5cfaedd-9d2b-4448-bfce-a738a3b43cda",
      "3fd971c2-8849-4760-86a3-f5846b463dca"
    ],
    "files": [
      "docs/knowledge/areas/human-mailbox/overview.md",
      "docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md",
      "docs/knowledge/areas/okf-profile/conformance-check.md",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
      "docs/history/mistake-loop-closed/delivery.md",
      "docs/briefs/mistake-loop-closed.md"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      "docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/",
      "docs/knowledge/areas/feedback-digest/",
      "docs/knowledge/areas/okf-profile/",
      "docs/knowledge/areas/hook-runtime/"
    ],
    "action": "Sync the owning concepts to the shipped behavior, citing this feature as 'mistake-loop-closed D<n>' (never raw decision ids). human-mailbox overview: rule 17 and the mistakes door rule for D2, D3, D4, D5; feedback-digest ranking: the ledger, fix-at close/verify/next/promote, upkeep kind fix-at (D1, D7, D8, D9); okf conformance-check: two new warning rows next to invalid_evidence_state (D10); token economy: the Fix-at debt block and its DROPPABLE registration (D6). Write docs/history/mistake-loop-closed/delivery.md (what shipped per decision, the proof per cell) and docs/briefs/mistake-loop-closed.md copied from docs/briefs/p-59a26c99.md with its title adjusted, checked with bee backlog briefs --id mistake-loop-closed. Avoid the words deferred, until and later. Proof: /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "/mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "the four concepts describe the shipped loop",
        "knowledge check passes with no new warning in the edited concepts"
      ],
      "artifacts": [
        {
          "path": "docs/history/mistake-loop-closed/delivery.md",
          "substantive": "the delivery note"
        }
      ],
      "key_links": [
        "each concept cites mistake-loop-closed D-numbers"
      ],
      "prohibitions": [
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "The concepts are the state layer and must match the shipped behavior.",
      "out_of_scope": [
        "Rust code",
        "doctrine files"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "mlc-9",
    "feature": "mistake-loop-closed",
    "title": "Check the doctrine and concept sync on the test role",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "mlc-7",
      "mlc-7c",
      "mlc-8"
    ],
    "decisions": [
      "3fd971c2-8849-4760-86a3-f5846b463dca",
      "a7f3fbd2-ee19-4d08-adb0-c1b4c33d445a"
    ],
    "files": [
      "docs/history/mistake-loop-closed/reports/mlc-9.md"
    ],
    "read_first": [
      "docs/history/mistake-loop-closed/CONTEXT.md",
      "docs/history/mistake-loop-closed/delivery.md",
      "docs/history/mistake-loop-closed/reports/mlc-7c.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Checker on the test role for docs cells mlc-7, mlc-7c and mlc-8. Edit no file except docs/history/mistake-loop-closed/reports/mlc-9.md. Run cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws, --test pointer_integrity, --test rule_index_parity, --test agents_block_render_parity, --bin bee -- no_shipped_command_spelling, and /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check; confirm each of the four concepts mlc-8 edited names the shipped behavior of its decisions (rg for fix-at close, reflect --cell, Fix-at debt, evidence_ref_missing_path). Record each command with its result in the report; return BLOCKED with the output on any red. Write no code comment of any kind. Never edit .bee/*.json or .bee/*.jsonl by hand. Run only the cell's verify filter, never the full suite. Allowed commands: cargo build, cargo test with a filter, cargo clippy, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee dev regen, /mnt/data/Projects/goglbe/beehive/.bee/bin/bee knowledge check. Quote the count of tests the filter matched in the cap report.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws",
    "must_haves": {
      "truths": [
        "every doctrine and concept check is recorded green in the report"
      ],
      "artifacts": [
        {
          "path": "docs/history/mistake-loop-closed/reports/mlc-9.md",
          "substantive": "the check report"
        }
      ],
      "key_links": [
        "the report names each command"
      ],
      "prohibitions": [
        "Edit only the report",
        "No code comments",
        "Never rewrite .bee/backlog.jsonl"
      ],
      "why": "The merge door needs a test-role checker behind each docs cell.",
      "out_of_scope": [
        "every file except its report"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Contract | Tests | Cell |
|---|---|---|
| fix-at-ledger | fix_at_ledger | mlc-1t |
| mistakes-door-names-cells | mistakes_door | mlc-2t |
| fix-at-intake | fix_at_intake | mlc-3t |
| fix-at-debt | fix_at_debt | mlc-4t |
| fix-at-verbs | fix_at_verbs | mlc-5t |
| evidence-ref-check | evidence_ref | mlc-6t |

## Open Questions

None.

## Out of scope

- A count predicate for triggers (CONTEXT.md Deferred Ideas).
- Rewriting .bee/backlog.jsonl.
