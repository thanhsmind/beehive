---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Herding caps on a clean report

## Summary

A dispatched worker does its job, commits, runs its tests, and hands back a
report that already contains its proof line — then stops one step short of
recording it. The cell sits half-finished, and somebody has to notice.

This makes the runner finish the step. When a job was given a cell and the
worker's report carries a real proof line, the runner records it, using the
worker's own words, and prints one line saying so. When the proof line is
missing or malformed, it records nothing and says that loudly instead.

Mode: `high-risk` — 3 risk flags: audit-security, public-contracts,
covered-contract-change.
Why this is the least workflow that protects the work: a cap is the proof
record, so changing who writes it is a change to an audit trail, even when the
bytes are identical.

## Requirements (from CONTEXT.md)

- **D1** Cell id present AND valid proof line → herding records the cap with the worker's bytes.
- **D2** Validity is the EXISTING shape parser plus the EXISTING vocabulary constant; herding defines neither.
- **D3** Missing, malformed or off-vocabulary → caps nothing, reported loudly, naming the cell and the fault.
- **D4** A job with no cell id is untouched.
- **D5** No new trust: this changes only who calls the cap.
- **D6** Reuse the callable cap entry point; no shelling out, no duplicated cap logic.
- **D7** One visible line per cap; a refusal line is never silenced.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The runner already knows which cell the job was dispatched for. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:180` | `cell_id: Option<String>,` on the job options, parsed from `--cell` at `:370` and stored at `:439` |
| 2 | The worker's proof line already reaches the runner on the result envelope. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3698` | `m.insert("proof".into(), Value::String(r.proof.clone()));` |
| 3 | A callable cap exists, so no shelling out is needed. | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:169` | `pub(crate) fn cap_cell_from_flags(root: &Path, f: &CapFlags, finish: bool) -> MR<Value> {` |
| 4 | The runner calling into the cells module is established, not new. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3525` | `match crate::verbs::cells::record_dissent(` |
| 5 | The proof SHAPE parser already exists and must be reused. | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:110` | `pub(crate) fn parse_tests_proof(s: &str) -> Option<(String, String, String)> {` — splits on the first two separators only, so the reason may contain one |
| 6 | The proof VOCABULARY already exists, and its own doc says a second copy is the failure mode. | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:81` | `pub(crate) const PROOF_RESULT_VALUES: [&str; 3]`, above it: *"Each value's meaning is written beside it HERE and nowhere else: a meaning restated in a second place is how a closed vocabulary decays back into three free-text values."* |
| 7 | The defect is total, not intermittent: every dispatched worker left its cell uncapped. | ran | `.bee/bin/bee cells show --id pnsd-N --json` for each of pnsd-1..pnsd-6 | each returned `status: claimed`, `outcome: None`, `capped_at: None` after the worker had committed and returned a populated report |

## Discovery

Everything this change needs already exists and was read at its line: the cell
id, the proof line, a callable cap, and a precedent for calling it from the
runner. The only new thing is the decision to call it, plus the validity check —
and that check must borrow the existing parser and the existing vocabulary
rather than restate them, because the vocabulary constant's own comment names
restatement as the way it decays.

## Approach

**Recommended path.** One slice, two cells. `hcocr-1` adds the cap-on-clean-report
path with its unit tests. `hcocr-2` drives it for real in a throwaway sandbox,
because the previous feature shipped a belt calling a hook that did not exist and
its tests passed on source shape alone — unit-green is not evidence that a
mechanical step fires.

Rejected alternatives:
- Shell out to `bee cells finish` — rejected by D6; a callable entry point exists and the runner already uses that road.
- Re-implement the proof check inside herding — rejected by D2; the vocabulary's own doc comment forbids the second copy.
- Fail the run loudly instead of capping — rejected by the owner on 2026-09-19; it removes the silence but leaves the manual work that is the unreliable part.

**SMALLER PATH check.** Could `hcocr-2` be dropped and the feature ship on unit
tests? No — that is precisely the shape that shipped an inert tool gate one
feature ago, and this change's whole value is that a mechanical step actually
fires. FAIL. Kept at two cells.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Capping on a worker's report | HIGH | `hcocr-1` | a valid proof line caps with the worker's exact bytes; a malformed one caps nothing and says so |
| A cell-less job | MEDIUM | `hcocr-1` | gather/advisor/hat/reviewer output byte-unchanged |
| Firing more than once, or on a dry run | MEDIUM | `hcocr-1` | one cap per job; `--dry-run` never caps |
| A mechanical step that silently does not fire | HIGH | `hcocr-2` | driven for real, `green:live` |

Waves: `hcocr-1` then `hcocr-2`, serial — the drive needs the path.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "planning", "classification": "required", "role": "plan", "reason": "High-risk lane: the change is small but it rewrites who authors an audit record."},
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "Rust in the herding runner."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "The value of this feature is that a mechanical step fires; only a live drive shows that."},
    {"stage": "documentation-and-capture", "classification": "conditional", "role": "docs", "reason": "Only if the verify-app feature file for herding needs a new sub-feature."},
    {"stage": "read-only-gather", "classification": "conditional", "role": "read", "reason": "The code context is already read and anchored; a gather is needed only if execution finds more call sites."},
    {"stage": "fact-extraction", "classification": "conditional", "role": "extraction", "condition": "a single already-located fact is needed during execution", "reason": "Cheap tier for narrow lookups."},
    {"stage": "generation-fallback", "classification": "conditional", "role": "generation", "condition": "a role with no configured slot is requested", "reason": "Fallback only."},
    {"stage": "independent-review", "classification": "conditional", "role": "review", "condition": "the user invokes a review", "reason": "Review is user-invoked."},
    {"stage": "generic-advisor", "classification": "required", "role": "advisor", "reason": "High-risk work owes an advisor consult; the hat-wave synthesis serves it."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Single-leader feature."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No convergence lane."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No convergence lane."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No convergence lane."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "Audits the claims table against the code."},
    {"stage": "hat-risks", "classification": "required", "role": "hat-risks", "reason": "The cap is an audit record; this seat is the one that must not be skipped."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Carries the SMALLER PATH question."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Deviation, recorded: the owner chose this behaviour on 2026-09-19 against a stated alternative, with the measured 5-of-5 failure rate and the trust analysis in front of them. The value question is already answered by the person who owns it."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Deviation, recorded: the only user-visible change is one extra output line per capped cell, whose wording is Agent's Discretion. Nothing this seat would weigh is still open."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release is cut by this feature."}
  ]
}
```

## Shape

One slice: `hcocr-1` (the path and its unit tests), then `hcocr-2` (the live drive).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `hcocr-1` | Cap the cell when a dispatched worker returns a clean report | `herding/run.rs`, `herding/tests.rs` | — | A worker that finishes its work no longer leaves the cell half-done; the run says which cell it capped and on what proof | scoped `cargo test` green, covering a valid proof line, a malformed one, and a cell-less job |
| `hcocr-2` | Drive it for real and record the recipe | `.bee/verify/verify-app/features/*` | `hcocr-1` | A real dispatched worker's cell comes back capped without anyone touching it | `green:live` — driven against a launched sandbox, evidence attached |

```json
[
  {
    "id": "hcocr-1",
    "feature": "herding-caps-on-clean-report",
    "title": "Cap the cell when a dispatched worker returns a clean report",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D1", "D2", "D3", "D4", "D6", "D7", "9d2347e4-66f3-4ee7-b7cc-2751e570359f"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "docs/history/herding-caps-on-clean-report/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Make the runner finish the step its worker stopped short of. At the end of a job, when the job carried a cell id (the `cell_id` already on the job options) AND the worker's report carries a VALID proof line (the `proof` already on the result envelope), record the cap with the worker's bytes unchanged (per D1). Reuse, do not rebuild: call `cap_cell_from_flags` in verbs/cells/handlers_close.rs rather than shelling out to the bee binary — the runner already calls into that module for `record_dissent`, so this is the same road (per D6). Decide validity with the EXISTING pieces (per D2): `parse_tests_proof` in verbs/cells/finish_support.rs for the three-segment shape, and `PROOF_RESULT_VALUES` for the result word. Do not restate the vocabulary or its meanings anywhere in herding — that constant's own doc comment says a meaning restated in a second place is how the vocabulary decays, and it is right. When the proof line is absent, unparseable, or its result is outside the vocabulary: cap NOTHING and report it loudly, naming the cell and which of those three it was (per D3) — the defect being fixed is a silent omission, so a different silent omission is not a fix. A job with no cell id keeps today's bytes exactly (per D4). Cap at most once per job, and never on `--dry-run`. Print one line naming the cell and the result segment on a cap, and a refusal line that no quiet mode suppresses (per D7). Write the tests first and watch them fail: a valid proof line caps with the worker's exact bytes; a malformed line caps nothing and reports; a result word outside the vocabulary caps nothing and reports; a cell-less job is byte-identical to today; `--dry-run` caps nothing.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding",
    "must_haves": {
      "truths": [
        "A job with a cell id and a valid proof line comes back with that cell capped, carrying the worker's exact proof bytes",
        "A malformed or off-vocabulary proof line caps nothing and is reported loudly, naming the cell",
        "A job with no cell id produces byte-identical output to today",
        "A dry run never caps",
        "The cap fires at most once per job"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "the cap-on-clean-report path, calling the existing cap and the existing proof parser; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/herding/tests.rs", "substantive": "tests for valid, malformed, off-vocabulary, cell-less and dry-run, each failing before the change"}
      ],
      "key_links": [
        "validity is decided by parse_tests_proof and PROOF_RESULT_VALUES, not by any new literal in herding",
        "the cap goes through cap_cell_from_flags, not a spawned bee process"
      ],
      "prohibitions": [
        "No copy of the proof vocabulary or its meanings inside herding",
        "No shelling out to the bee binary to cap",
        "No change to what a cap validates — only to who calls it",
        "No silent refusal: every non-cap path says why"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "hcocr-2",
    "feature": "herding-caps-on-clean-report",
    "title": "Drive the automatic cap for real and record the recipe",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": ["hcocr-1"],
    "decisions": ["D1", "D3", "D7", "9d2347e4-66f3-4ee7-b7cc-2751e570359f"],
    "files": [".bee/verify/verify-app/features/cells-and-proof.md"],
    "read_first": [
      ".bee/verify/verify-app/features/cells-and-proof.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/history/herding-caps-on-clean-report/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [".bee/verify/verify-app/features/cells-and-proof.md"],
    "action": "Prove the step actually fires for a real user, not only in unit tests — one feature ago a belt shipped calling a hook that did not exist and its tests passed because they only read source shape. Rebuild and install the binary at .bee/bin/bee first; a stale vendored copy makes the whole run worthless. Then, against a launched control-bee sandbox: create a cell, dispatch a real worker for it through the door, let the worker finish, and assert WITHOUT any manual cap that the cell comes back status capped with the worker's proof line recorded on it. Then do the negative half: drive a job whose worker returns a report with no valid proof line, and assert the cell is still open or claimed AND the run said so by name. Add both as one sub-feature in the verify-app feature file that owns cells and proof, following that file's existing four-H2 contract, with the exact commands and assertions. Record the proof line as green:live.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding",
    "must_haves": {
      "truths": [
        "A real dispatched worker's cell comes back capped with no manual step",
        "The recorded proof line is the worker's own bytes",
        "A worker returning no valid proof line leaves the cell uncapped AND the run names it",
        "doctor is clean and the binary driven is the one built from this branch"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/features/cells-and-proof.md", "substantive": "a new sub-feature covering the automatic cap and its refusal path, with driving commands and assertions"}
      ],
      "key_links": ["the drive runs the rebuilt binary installed at .bee/bin/bee, not a stale vendored copy"],
      "prohibitions": [
        "No claim of green without the fresh command output beside it",
        "Do not cap this cell if the automatic cap did not fire — report it"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| # | Dimension | Scenario | Pass when |
|---|---|---|---|
| 1 | Happy path | Job with a cell id, worker returns a valid proof line | cell capped, proof bytes identical to the worker's |
| 2 | Error path | Proof line missing | no cap, a line names the cell and says the proof was missing |
| 3 | Error path | Proof line has fewer than three segments | no cap, a line names the cell and says the shape was wrong |
| 4 | Boundary | Result word outside the vocabulary | no cap, a line names the cell and the bad word |
| 5 | Boundary | Reason segment itself contains the separator | still valid — the parser splits on the first two only |
| 6 | Regression | Job with no cell id (gather, advisor, hat seat, reviewer) | output byte-identical to today |
| 7 | Idempotence | One job, one worker result | at most one cap |
| 8 | Boundary | `--dry-run` | nothing capped |
| 9 | Live | A real dispatched worker against a sandbox | cell capped with no manual step, `green:live` |
| 10 | Live | A real worker returning no valid proof line | cell not capped, and the run said so |

## Open Questions

(none)

<!-- bee:not-a-deferral: This section lists what the feature deliberately does not change, so a reader knows the boundary. Each line is a locked decision or an existing record, not work postponed. -->
## Out of scope

- What a cap validates. `bee cells finish` runs nothing and records what it is handed; that is unchanged (D5).
- The leader's completeness check against artifacts. It remains the leader's job.
- Whether a worker SHOULD also keep calling the cap itself. Both paths capping the same cell is covered by the at-most-once requirement, not by changing the worker contract.
<!-- /bee:not-a-deferral -->
